//! One user turn: store-aware setup, aisdk's tool loop, and the detached task that streams the
//! reply to the client and saves it.

use crate::agent::history::{UiMessage, to_model_messages, user_text};
use crate::agent::live::{self, LiveReply};
use crate::agent::models::{AgentModel, UTILITY_MODEL};
use crate::agent::openrouter::OpenRouterChat;
use crate::agent::prompt::system_prompt;
use crate::agent::store::{self, SaveTurn};
use crate::agent::tools::{ToolCtx, agent_tools};
use crate::agent::ui_stream::encode;
use crate::agent::web_search::WebSearch;
use crate::routes::Dictionary;
use crate::state::AppState;
use aisdk::core::LanguageModelRequest;
use aisdk::core::language_model::ReasoningEffort;
use futures_util::Stream;
use sea_orm::DbErr;
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

/// Function tool calls per turn before the model is made to answer.
pub const MAX_TOOL_CALLS: usize = 24;
const MAX_MESSAGE_CHARS: usize = 8_000;
const STOP_POLL: Duration = Duration::from_secs(1);
/// How long a new message waits for the previous reply to stop and save.
const PREVIOUS_REPLY_WAIT: Duration = Duration::from_secs(15);

pub struct TurnRequest {
  pub chat_id: String,
  pub message: UiMessage,
  pub model: AgentModel,
  pub dicts: Vec<String>,
}

#[derive(Debug)]
pub enum TurnError {
  BadRequest(String),
  /// `OPENROUTER_API_KEY` missing.
  NotConfigured,
  Db(DbErr),
}

impl From<DbErr> for TurnError {
  fn from(e: DbErr) -> Self {
    TurnError::Db(e)
  }
}

fn http() -> reqwest::Client {
  static HTTP: OnceLock<reqwest::Client> = OnceLock::new();
  HTTP
    .get_or_init(|| {
      reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client")
    })
    .clone()
}

fn api_key() -> Option<Arc<str>> {
  std::env::var("OPENROUTER_API_KEY")
    .ok()
    .filter(|k| !k.trim().is_empty())
    .map(|k| Arc::from(k.trim()))
}

/// `None` omits the `web_search` tool (the agent still answers from the dictionaries).
fn openai_key() -> Option<Arc<str>> {
  std::env::var("OPENAI_API_KEY")
    .ok()
    .filter(|k| !k.trim().is_empty())
    .map(|k| Arc::from(k.trim()))
}

/// Stop the reply generating for `chat_id`, wherever it runs.
pub async fn stop(state: &AppState, chat_id: &str) -> Result<(), DbErr> {
  if let Some(reply) = live::get(chat_id) {
    reply.cancel.cancel();
  }
  store::request_stop(&state.db, chat_id).await
}

/// A user message: only its text parts, trimmed to what the client may send.
fn clean_user_message(message: UiMessage) -> Result<UiMessage, TurnError> {
  if message.role != "user" || message.id.is_empty() || message.id.len() > 100 {
    return Err(TurnError::BadRequest("invalid message".into()));
  }
  let parts: Vec<Value> = message
    .parts
    .iter()
    .filter(|p| p["type"] == "text")
    .filter_map(|p| p["text"].as_str())
    .map(|text| json!({ "type": "text", "text": text }))
    .collect();
  let cleaned = UiMessage {
    id: message.id,
    role: "user".into(),
    parts,
    metadata: None,
  };
  let text = user_text(&cleaned);
  if text.trim().is_empty() {
    return Err(TurnError::BadRequest("the message is empty".into()));
  }
  if text.chars().count() > MAX_MESSAGE_CHARS {
    return Err(TurnError::BadRequest(format!(
      "the message is longer than {MAX_MESSAGE_CHARS} characters"
    )));
  }
  Ok(cleaned)
}

fn new_message_id() -> String {
  format!("msg_{}", uuid::Uuid::new_v4().simple())
}

/// The preferred slugs that name a dictionary.
fn known_dicts(dictionaries: &[Dictionary], dicts: Vec<String>) -> Vec<String> {
  dicts
    .into_iter()
    .filter(|slug| dictionaries.iter().any(|d| &d.slug == slug))
    .collect()
}

/// Exactly what the model is given for these preferred dictionaries: system prompt and the
/// function tools (the Context dialog; no server tools are sent).
pub async fn context(state: &AppState, dicts: Vec<String>) -> Result<Value, DbErr> {
  let dictionaries = state.dictionaries().await?;
  let dicts = known_dicts(&dictionaries, dicts);
  let tools = agent_tools(&ToolCtx {
    state: state.clone(),
    preferred_dicts: dicts.clone().into(),
    web_search: openai_key().map(|key| WebSearch::new(http(), key)),
  });
  let specs: Vec<Value> = tools
    .specs
    .iter()
    .map(|s| json!({ "name": s["name"], "description": s["description"], "parameters": s["parameters"] }))
    .collect();
  Ok(json!({
    "system_prompt": system_prompt(&dictionaries, &dicts),
    "tools": specs,
    "server_tools": Vec::<Value>::new(),
    "max_tool_calls": MAX_TOOL_CALLS,
    "models": AgentModel::ALL
      .iter()
      .map(|m| json!({ "id": m.id(), "provider_id": m.provider_id() }))
      .collect::<Vec<_>>(),
  }))
}

/// Start a turn; the returned stream yields its SSE frames until the reply is saved.
pub async fn start(
  state: AppState,
  req: TurnRequest,
) -> Result<impl Stream<Item = Arc<str>> + Send + 'static, TurnError> {
  let api_key = api_key().ok_or(TurnError::NotConfigured)?;
  let message = clean_user_message(req.message)?;
  let db = &state.db;

  let dictionaries = state.dictionaries().await?;
  let dicts = known_dicts(&dictionaries, req.dicts);

  // A message sent while a reply is generating replaces that reply: stop it and let it save.
  let chat = match store::find_chat(db, &req.chat_id).await? {
    Some(chat) if chat.streaming => {
      stop(&state, &chat.id).await?;
      let deadline = tokio::time::Instant::now() + PREVIOUS_REPLY_WAIT;
      loop {
        tokio::time::sleep(Duration::from_millis(250)).await;
        let current = store::find_chat(db, &chat.id).await?;
        match current {
          Some(c) if c.streaming && tokio::time::Instant::now() < deadline => continue,
          // Past the deadline the flag is stale (its instance died): a live reply polls Stop
          // every second.
          Some(c) => break c,
          None => return Err(TurnError::BadRequest("the chat was deleted".into())),
        }
      }
    }
    Some(chat) => chat,
    None => {
      if req.chat_id.is_empty()
        || req.chat_id.len() > 64
        || !req
          .chat_id
          .chars()
          .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
      {
        return Err(TurnError::BadRequest("invalid chat id".into()));
      }
      store::create_chat(db, &req.chat_id, req.model.id(), &dicts).await?
    }
  };

  if store::message_in_other_chat(db, &message.id, &chat.id).await? {
    return Err(TurnError::BadRequest("message id already used".into()));
  }
  let stored = store::load_messages(db, &chat.id).await?;
  // Regenerate / retry resends a stored message: replay the history up to it.
  let history: Vec<UiMessage> = match stored.iter().position(|m| m.id == message.id) {
    Some(i) => stored[..i].to_vec(),
    None => stored,
  };
  let first_turn = history.is_empty() && chat.title.is_empty();
  let mut conversation = history;
  conversation.push(message);
  // Stored now, so a reload during the reply shows the question above the resumed answer.
  store::save_turn(
    db,
    SaveTurn {
      chat_id: &chat.id,
      messages: &conversation,
      model: req.model.id(),
      dicts: &dicts,
      title: None,
    },
  )
  .await?;

  let cancel = CancellationToken::new();
  let tool_ctx = ToolCtx {
    state: state.clone(),
    preferred_dicts: dicts.clone().into(),
    web_search: openai_key().map(|key| WebSearch::new(http(), key)),
  };
  let tools = agent_tools(&tool_ctx);
  let model = OpenRouterChat::new(http(), api_key.clone(), req.model)
    .with_tools(tools.specs, MAX_TOOL_CALLS)
    .with_cancel(cancel.clone())
    .with_session(&chat.id);
  let meter = model.meter();
  let mut builder = LanguageModelRequest::builder()
    .model(model)
    .system(system_prompt(&dictionaries, &dicts))
    .messages(to_model_messages(&conversation));
  for tool in tools.tools {
    builder = builder.with_tool(tool);
  }
  let response = builder
    .build()
    .stream_text()
    .await
    .map_err(|e| TurnError::BadRequest(format!("could not start the reply: {e}")))?;

  let title = first_turn.then(|| {
    let text = user_text(conversation.last().expect("the new message"));
    tokio::spawn(generate_title(api_key, text))
  });

  store::set_streaming(db, &chat.id, true).await?;
  let reply_stream = LiveReply::new(cancel.clone());
  live::register(&chat.id, reply_stream.clone());

  let (tx, mut rx) = unbounded_channel::<String>();
  let pump = {
    let reply_stream = reply_stream.clone();
    tokio::spawn(async move {
      while let Some(frame) = rx.recv().await {
        reply_stream.push(frame);
      }
    })
  };
  let chat_id = chat.id.clone();
  let subscriber = LiveReply::subscribe(reply_stream.clone());
  tokio::spawn(async move {
    let poller = tokio::spawn(poll_stop(state.clone(), chat_id.clone(), cancel.clone()));
    let message_id = new_message_id();
    let reply = encode(
      response.stream,
      &message_id,
      req.model,
      meter,
      cancel.clone(),
      tx,
    )
    .await;
    poller.abort();
    if let Some(error) = &reply.error {
      eprintln!("agent reply for chat {chat_id} failed: {error}");
    }

    if reply.parts.iter().any(|p| p["type"] != "step-start") {
      conversation.push(UiMessage {
        id: message_id,
        role: "assistant".into(),
        parts: reply.parts,
        metadata: Some(reply.metadata),
      });
    }
    let title = match title {
      Some(handle) => handle.await.ok(),
      None => None,
    };
    let saved = store::save_turn(
      &state.db,
      SaveTurn {
        chat_id: &chat_id,
        messages: &conversation,
        model: req.model.id(),
        dicts: &dicts,
        title,
      },
    )
    .await;
    if let Err(e) = saved {
      eprintln!("agent: saving chat {chat_id} failed: {e}");
    }
    if let Err(e) = store::set_streaming(&state.db, &chat_id, false).await {
      eprintln!("agent: clearing streaming for {chat_id} failed: {e}");
    }
    // Clients' streams end only now, so whatever they refetch next is the saved chat.
    let _ = pump.await;
    reply_stream.finish();
    live::unregister(&chat_id, &reply_stream);
  });
  Ok(subscriber)
}

/// The reply generating for `chat_id` on this instance, replayed from its start.
pub fn resume(chat_id: &str) -> Option<impl Stream<Item = Arc<str>> + Send + 'static> {
  live::get(chat_id).map(LiveReply::subscribe)
}

/// Polls the chat's Stop flag (set by a Stop request on any instance) until the reply ends.
async fn poll_stop(state: AppState, chat_id: String, cancel: CancellationToken) {
  loop {
    tokio::select! {
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(STOP_POLL) => {}
    }
    if store::is_stop_requested(&state.db, &chat_id)
      .await
      .unwrap_or(false)
    {
      cancel.cancel();
      return;
    }
  }
}

const TITLE_MAX_CHARS: usize = 80;

fn fallback_title(text: &str) -> String {
  let line = text.trim().lines().next().unwrap_or("").trim();
  match line.char_indices().nth(50) {
    Some((end, _)) => format!("{}…", line[..end].trim_end()),
    None if line.is_empty() => "New chat".into(),
    None => line.to_string(),
  }
}

/// A 2–6 word sidebar title from the first message, by the utility model without reasoning.
async fn generate_title(api_key: Arc<str>, first_message: String) -> String {
  let prompt: String = first_message.chars().take(2_000).collect();
  let request = async {
    LanguageModelRequest::builder()
      .model(OpenRouterChat::new(http(), api_key, UTILITY_MODEL))
      .system(
        "You name chats. Reply with only a 2–6 word title for the chat whose first message is \
         given — never answer the message. Write the title in the language of the message \
         (English if it is in English), quoting Sanskrit words in the script they were written \
         in. Title case, no quotes, no trailing period.",
      )
      .prompt(format!("<message>\n{prompt}\n</message>"))
      .reasoning_effort(ReasoningEffort::None)
      .build()
      .generate_text()
      .await
  };
  let text = match tokio::time::timeout(Duration::from_secs(20), request).await {
    Ok(Ok(response)) => response.text().unwrap_or_default(),
    _ => String::new(),
  };
  let title = text
    .trim()
    .trim_matches(|c| matches!(c, '"' | '\'' | '“' | '”'))
    .trim_end_matches('.')
    .trim();
  if title.is_empty() {
    fallback_title(&first_message)
  } else {
    title.chars().take(TITLE_MAX_CHARS).collect()
  }
}
