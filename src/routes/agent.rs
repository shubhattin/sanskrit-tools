//! Dictionary agent endpoints (`/api/agent/*`, contract in `docs/agent.md`). Plain axum routes,
//! outside the OpenAPI document: the chat endpoint streams the AI SDK UI message protocol, which
//! the frontend consumes with `useChat`, not the generated client.

use crate::agent::history::UiMessage;
use crate::agent::models::AgentModel;
use crate::agent::store;
use crate::agent::turn::{self, TurnError, TurnRequest};
use crate::entities::agent_chats;
use crate::routes::ApiError;
use crate::state::AppState;
use axum::body::{Body, Bytes};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/api/agent/chat", post(chat))
    .route("/api/agent/chat/{id}/stream", get(resume))
    .route("/api/agent/context", get(context))
    .route("/api/agent/chats", get(list))
    .route(
      "/api/agent/chats/{id}",
      get(detail).patch(rename).delete(remove),
    )
    .route("/api/agent/chats/{id}/stop", post(stop))
}

#[derive(Serialize)]
struct ChatSummary {
  id: String,
  title: String,
  model: String,
  streaming: bool,
  created_at: String,
  updated_at: String,
}

#[derive(Serialize)]
struct Chat {
  #[serde(flatten)]
  summary: ChatSummary,
  dicts: Vec<String>,
}

impl From<agent_chats::Model> for Chat {
  fn from(m: agent_chats::Model) -> Self {
    Chat {
      dicts: m.dicts.clone(),
      summary: ChatSummary::from(m),
    }
  }
}

impl From<agent_chats::Model> for ChatSummary {
  fn from(m: agent_chats::Model) -> Self {
    ChatSummary {
      id: m.id,
      title: m.title,
      model: m.model,
      streaming: m.streaming,
      created_at: m.created_at.to_rfc3339(),
      updated_at: m.updated_at.to_rfc3339(),
    }
  }
}

#[derive(Deserialize)]
struct ChatBody {
  id: String,
  message: UiMessage,
  model: AgentModel,
  #[serde(default)]
  dicts: Vec<String>,
}

async fn chat(
  State(state): State<AppState>,
  body: Result<Json<ChatBody>, JsonRejection>,
) -> Response {
  let body = match body {
    Ok(Json(body)) => body,
    Err(e) => return ApiError::bad_request(e.body_text()).into_response(),
  };
  let request = TurnRequest {
    chat_id: body.id,
    message: body.message,
    model: body.model,
    dicts: body.dicts,
  };
  match turn::start(state, request).await {
    Ok(frames) => sse_response(frames),
    Err(TurnError::BadRequest(msg)) => ApiError::bad_request(msg).into_response(),
    Err(TurnError::NotConfigured) => ApiError(
      StatusCode::SERVICE_UNAVAILABLE,
      "The agent is not configured (OPENROUTER_API_KEY).".into(),
    )
    .into_response(),
    Err(TurnError::Db(e)) => ApiError::from(e).into_response(),
  }
}

#[derive(Deserialize)]
struct ContextParams {
  /// Comma-separated preferred dictionary slugs.
  dicts: Option<String>,
}

async fn context(
  State(state): State<AppState>,
  Query(p): Query<ContextParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
  let dicts = p
    .dicts
    .unwrap_or_default()
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(str::to_owned)
    .collect();
  Ok(Json(turn::context(&state, dicts).await?))
}

/// SSE response carrying an AI SDK UI message stream.
fn sse_response(frames: impl Stream<Item = Arc<str>> + Send + 'static) -> Response {
  let body = frames.map(|frame| Ok::<_, Infallible>(Bytes::copy_from_slice(frame.as_bytes())));
  Response::builder()
    .header(header::CONTENT_TYPE, "text/event-stream")
    .header(header::CACHE_CONTROL, "no-cache")
    .header("x-vercel-ai-ui-message-stream", "v1")
    .header("x-accel-buffering", "no")
    .body(Body::from_stream(body))
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// `useChat({ resume: true })` reconnect: the reply being generated, from its start; 204 when
/// none runs on this instance (the client then shows the saved chat).
async fn resume(Path(id): Path<String>) -> Response {
  match turn::resume(&id) {
    Some(frames) => sse_response(frames),
    None => StatusCode::NO_CONTENT.into_response(),
  }
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<ChatSummary>>, ApiError> {
  Ok(Json(
    store::list_chats(&state.db)
      .await?
      .into_iter()
      .map(ChatSummary::from)
      .collect(),
  ))
}

#[derive(Serialize)]
struct ChatDetail {
  chat: Chat,
  messages: Vec<UiMessage>,
}

async fn detail(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Json<ChatDetail>, ApiError> {
  let chat = store::find_chat(&state.db, &id)
    .await?
    .ok_or_else(|| ApiError::not_found("chat not found"))?;
  let messages = store::load_messages(&state.db, &id).await?;
  Ok(Json(ChatDetail {
    chat: chat.into(),
    messages,
  }))
}

#[derive(Deserialize)]
struct RenameBody {
  title: String,
}

async fn rename(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Result<Json<RenameBody>, JsonRejection>,
) -> Result<Json<ChatSummary>, ApiError> {
  let Json(body) = body.map_err(|e| ApiError::bad_request(e.body_text()))?;
  let title = body.title.trim();
  if title.is_empty() || title.chars().count() > 120 {
    return Err(ApiError::bad_request("title must be 1–120 characters"));
  }
  let chat = store::rename_chat(&state.db, &id, title)
    .await?
    .ok_or_else(|| ApiError::not_found("chat not found"))?;
  Ok(Json(chat.into()))
}

async fn remove(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  turn::stop(&state, &id).await?;
  if store::delete_chat(&state.db, &id).await? {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(ApiError::not_found("chat not found"))
  }
}

async fn stop(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  turn::stop(&state, &id).await?;
  Ok(StatusCode::NO_CONTENT)
}
