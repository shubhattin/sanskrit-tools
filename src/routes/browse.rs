//! Dictionary list, single entry, and book-order browsing.

use super::{ApiError, ApiResult, Dictionary, Entry, stmt};
use crate::entities::{dictionaries, entries};
use crate::state::AppState;
use crate::translit::{loose_key, query_keys};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// OpenAPI route registration (paths served by [`crate::app::router`] (API stable)).
pub fn openapi_router() -> OpenApiRouter<crate::state::AppState> {
  OpenApiRouter::new()
    .routes(routes!(list_dictionaries))
    .routes(routes!(entry))
    .routes(routes!(browse))
}

#[utoipa::path(
    get,
    path = "/api/dictionaries",
    operation_id = "dictionaries.list",
    tag = "dictionaries",
    responses(
        (status = 200, description = "All dictionaries", body = Vec<Dictionary>),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn list_dictionaries(State(state): State<AppState>) -> ApiResult<Vec<Dictionary>> {
  Ok(Json(super::all_dictionaries(&state).await?))
}

#[derive(Serialize, ToSchema)]
pub struct EntryResponse {
  entry: Entry,
  dictionary: Dictionary,
}

#[utoipa::path(
    get,
    path = "/api/entries/{id}",
    operation_id = "entries.get",
    tag = "dictionaries",
    params(("id" = i32, Path, description = "Entry id")),
    responses(
        (status = 200, description = "Entry with its dictionary", body = EntryResponse),
        (status = 404, description = "Entry not found", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn entry(State(state): State<AppState>, Path(id): Path<i32>) -> ApiResult<EntryResponse> {
  let entry = entries::Entity::find_by_id(id)
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("entry not found"))?;
  let dictionary = dictionaries::Entity::find_by_id(entry.dictionary_id)
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("dictionary not found"))?;
  Ok(Json(EntryResponse {
    entry: entry.into(),
    dictionary: dictionary.into(),
  }))
}

#[derive(Deserialize)]
pub struct BrowseParams {
  seq: Option<i32>,
  before: Option<i32>,
  after: Option<i32>,
  q: Option<String>,
}

#[derive(Serialize, FromQueryResult, ToSchema)]
pub struct BrowseItem {
  id: i32,
  seq: i32,
  headword: String,
}

#[derive(Serialize, ToSchema)]
pub struct BrowseResponse {
  dictionary: Dictionary,
  items: Vec<BrowseItem>,
  /// The seq the window is centered on (the jump target when `q` was given)
  center: i32,
}

#[derive(FromQueryResult)]
struct MinSeq {
  seq: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/dictionaries/{slug}/browse",
    operation_id = "dictionaries.browse",
    tag = "dictionaries",
    params(
        ("slug" = String, Path, description = "Dictionary slug"),
        ("seq" = Option<i32>, Query, description = "Center seq (default 0)"),
        ("before" = Option<i32>, Query, description = "Entries before center (default 15, max 200)"),
        ("after" = Option<i32>, Query, description = "Entries after center (default 30, max 200)"),
        ("q" = Option<String>, Query, description = "Jump to headword instead of seq"),
    ),
    responses(
        (status = 200, description = "Book-order window", body = BrowseResponse),
        (status = 404, description = "Dictionary or headword not found", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn browse(
  State(state): State<AppState>,
  Path(slug): Path<String>,
  Query(p): Query<BrowseParams>,
) -> ApiResult<BrowseResponse> {
  let dictionary = dictionaries::Entity::find()
    .filter(dictionaries::Column::Slug.eq(&slug))
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("dictionary not found"))?;
  let before = p.before.unwrap_or(15).clamp(0, 200);
  let after = p.after.unwrap_or(30).clamp(0, 200);

  let mut center = p.seq.unwrap_or(0).max(0);
  if let Some(q) = p.q.as_deref().filter(|q| !q.trim().is_empty()) {
    let keys = query_keys(q);
    let loose: Vec<String> = keys.iter().map(|k| loose_key(k)).collect();
    // exact → loose / first prefix match
    let exact = MinSeq::find_by_statement(stmt(
      "select min(e.seq) as seq from entry_keys k join entries e on e.id = k.entry_id
       where k.dictionary_id = $1 and k.key_slp1 = any($2)",
      vec![dictionary.id.into(), keys.clone().into()],
    ))
    .one(&state.db)
    .await?
    .and_then(|r| r.seq);
    let found = match exact {
      Some(s) => Some(s),
      None => MinSeq::find_by_statement(stmt(
        "select min(e.seq) as seq from entry_keys k join entries e on e.id = k.entry_id
         where k.dictionary_id = $1 and (k.key_loose = any($2) or k.key_slp1 like $3 || '%')",
        vec![
          dictionary.id.into(),
          loose.into(),
          keys.first().cloned().unwrap_or_default().into(),
        ],
      ))
      .one(&state.db)
      .await?
      .and_then(|r| r.seq),
    };
    match found {
      Some(s) => center = s,
      None => return Err(ApiError::not_found("headword not found in this dictionary")),
    }
  }

  let items = entries::Entity::find()
    .select_only()
    .columns([
      entries::Column::Id,
      entries::Column::Seq,
      entries::Column::Headword,
    ])
    .filter(entries::Column::DictionaryId.eq(dictionary.id))
    .filter(entries::Column::Seq.between(center - before, center + after))
    .order_by_asc(entries::Column::Seq)
    .into_model::<BrowseItem>()
    .all(&state.db)
    .await?;
  Ok(Json(BrowseResponse {
    dictionary: dictionary.into(),
    items,
    center,
  }))
}
