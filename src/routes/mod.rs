//! JSON API under `/api` (contract: `docs/api.md`).
//!
//! Queries go through SeaORM: entity queries for plain lookups, and typed raw
//! statements ([`stmt`] + `FromQueryResult`) where Postgres-specific SQL is
//! needed (trigram similarity, full-text ranking, array binds, CTEs).

pub mod browse;
pub mod dhatus;
pub mod paradigms;
pub mod search;
pub mod sutras;

use crate::entities::{dictionaries, entries, enums};
use crate::state::AppState;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::sea_query::{ArrayType, Value};
use sea_orm::{ColumnTrait, DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, Statement};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .merge(browse::openapi_router())
    .merge(search::openapi_router())
    .merge(sutras::openapi_router())
    .merge(dhatus::openapi_router())
    .merge(paradigms::openapi_router())
}

pub struct ApiError(pub StatusCode, pub String);

/// JSON error body (`{ "error": string }`) — also the OpenAPI error schema.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
  pub error: String,
}

impl ApiError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    ApiError(StatusCode::BAD_REQUEST, msg.into())
  }
  pub fn not_found(msg: impl Into<String>) -> Self {
    ApiError(StatusCode::NOT_FOUND, msg.into())
  }
}

impl From<DbErr> for ApiError {
  fn from(e: DbErr) -> Self {
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

/// Raw Postgres statement with positional (`$n`) binds.
pub fn stmt(sql: &str, values: Vec<Value>) -> Statement {
  Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

/// `int[]` bind; `None` binds SQL NULL (for `$n::int[] is null` filters).
pub fn int_array(v: Option<&Vec<i32>>) -> Value {
  match v {
    Some(v) => v.clone().into(),
    None => Value::Array(ArrayType::Int, None),
  }
}

/// Entry columns for raw statements mapped onto `entries::Model`.
pub const ENTRY_SELECT: &str =
  "e.id, e.dictionary_id, e.seq, e.headword, e.senses, e.data, e.sutra_id, e.dhatu_id, e.gloss_en";

#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct Dictionary {
  pub id: i32,
  pub slug: String,
  pub name: String,
  pub short_name: String,
  pub kind: enums::DictKind,
  pub src_lang: enums::Lang,
  pub tgt_lang: enums::Lang,
  pub description: Option<String>,
  pub attribution: Option<String>,
  pub entry_count: i32,
  pub sort_order: i32,
}

impl From<dictionaries::Model> for Dictionary {
  fn from(m: dictionaries::Model) -> Self {
    Dictionary {
      id: m.id,
      slug: m.slug,
      name: m.name,
      short_name: m.short_name,
      kind: m.kind,
      src_lang: m.src_lang,
      tgt_lang: m.tgt_lang,
      description: m.description,
      attribution: m.attribution,
      entry_count: m.entry_count,
      sort_order: m.sort_order,
    }
  }
}

#[derive(Serialize, Clone, Debug, ToSchema)]
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

impl From<entries::Model> for Entry {
  fn from(m: entries::Model) -> Self {
    Entry {
      id: m.id,
      dictionary_id: m.dictionary_id,
      seq: m.seq,
      headword: m.headword,
      senses: m.senses,
      data: m.data,
      sutra_id: m.sutra_id,
      dhatu_id: m.dhatu_id,
    }
  }
}

pub async fn all_dictionaries(state: &AppState) -> Result<Vec<Dictionary>, DbErr> {
  Ok(
    dictionaries::Entity::find()
      .filter(dictionaries::Column::EntryCount.gt(0))
      .order_by_asc(dictionaries::Column::SortOrder)
      .order_by_asc(dictionaries::Column::Id)
      .all(&state.db)
      .await?
      .into_iter()
      .map(Dictionary::from)
      .collect(),
  )
}

#[derive(Serialize, ToSchema)]
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
