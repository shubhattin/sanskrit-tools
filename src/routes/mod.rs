//! JSON API under `/api` (contract: `docs/api.md`).

pub mod browse;
pub mod dhatus;
pub mod search;
pub mod sutras;

use crate::state::AppState;
use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/api/dictionaries", get(browse::list_dictionaries))
    .route("/api/dictionaries/{slug}/browse", get(browse::browse))
    .route("/api/entries/{id}", get(browse::entry))
    .route("/api/search", get(search::search))
    .route("/api/suggest", get(search::suggest))
    .route("/api/sutras", get(sutras::list))
    .route("/api/sutras/{id}", get(sutras::detail))
    .route("/api/dhatus", get(dhatus::list))
    .route("/api/dhatus/{id}", get(dhatus::detail))
}

pub struct ApiError(pub StatusCode, pub String);

impl ApiError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    ApiError(StatusCode::BAD_REQUEST, msg.into())
  }
  pub fn not_found(msg: impl Into<String>) -> Self {
    ApiError(StatusCode::NOT_FOUND, msg.into())
  }
}

impl From<sqlx::Error> for ApiError {
  fn from(e: sqlx::Error) -> Self {
    eprintln!("db error: {e}");
    ApiError(StatusCode::INTERNAL_SERVER_ERROR, "database error".into())
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
  }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Dictionary {
  pub id: i32,
  pub slug: String,
  pub name: String,
  pub short_name: String,
  pub kind: String,
  pub src_lang: String,
  pub tgt_lang: String,
  pub description: Option<String>,
  pub attribution: Option<String>,
  pub entry_count: i32,
  pub sort_order: i32,
}

pub const DICT_COLS: &str = "id, slug, name, short_name, kind::text as kind, src_lang::text as src_lang, \
  tgt_lang::text as tgt_lang, description, attribution, entry_count, sort_order";

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Entry {
  pub id: i32,
  pub dictionary_id: i32,
  pub seq: i32,
  pub headword: String,
  pub senses: Vec<String>,
  pub data: Option<serde_json::Value>,
  pub sutra_id: Option<String>,
  pub dhatu_id: Option<i32>,
}

pub const ENTRY_COLS: &str =
  "e.id, e.dictionary_id, e.seq, e.headword, e.senses, e.data, e.sutra_id, e.dhatu_id";

pub async fn all_dictionaries(state: &AppState) -> Result<Vec<Dictionary>, sqlx::Error> {
  sqlx::query_as::<_, Dictionary>(&format!(
    "select {DICT_COLS} from dictionaries where entry_count > 0 order by sort_order, id"
  ))
  .fetch_all(&state.db)
  .await
}

#[derive(Serialize)]
pub struct DictEntries {
  pub dictionary: Dictionary,
  pub entries: Vec<Entry>,
}

/// Group entries by dictionary, in dictionary sort order.
pub fn group_by_dictionary(dicts: &[Dictionary], entries: Vec<Entry>) -> Vec<DictEntries> {
  let mut out: Vec<DictEntries> = Vec::new();
  for d in dicts {
    let es: Vec<Entry> = entries
      .iter()
      .filter(|e| e.dictionary_id == d.id)
      .cloned()
      .collect();
    if !es.is_empty() {
      out.push(DictEntries {
        dictionary: d.clone(),
        entries: es,
      });
    }
  }
  out
}
