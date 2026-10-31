//! Dictionary list, single entry, and book-order browsing.

use super::{ApiError, ApiResult, DICT_COLS, Dictionary, ENTRY_COLS, Entry};
use crate::state::AppState;
use crate::translit::{loose_key, query_keys};
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub async fn list_dictionaries(State(state): State<AppState>) -> ApiResult<Vec<Dictionary>> {
  Ok(Json(super::all_dictionaries(&state).await?))
}

#[derive(Serialize)]
pub struct EntryResponse {
  entry: Entry,
  dictionary: Dictionary,
}

pub async fn entry(State(state): State<AppState>, Path(id): Path<i32>) -> ApiResult<EntryResponse> {
  let entry: Entry = sqlx::query_as(&format!(
    "select {ENTRY_COLS} from entries e where e.id = $1"
  ))
  .bind(id)
  .fetch_optional(&state.db)
  .await?
  .ok_or_else(|| ApiError::not_found("entry not found"))?;
  let dictionary: Dictionary = sqlx::query_as(&format!(
    "select {DICT_COLS} from dictionaries where id = $1"
  ))
  .bind(entry.dictionary_id)
  .fetch_one(&state.db)
  .await?;
  Ok(Json(EntryResponse { entry, dictionary }))
}

#[derive(Deserialize)]
pub struct BrowseParams {
  seq: Option<i32>,
  before: Option<i32>,
  after: Option<i32>,
  q: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct BrowseItem {
  id: i32,
  seq: i32,
  headword: String,
}

#[derive(Serialize)]
pub struct BrowseResponse {
  dictionary: Dictionary,
  items: Vec<BrowseItem>,
  /// The seq the window is centered on (the jump target when `q` was given)
  center: i32,
}

pub async fn browse(
  State(state): State<AppState>,
  Path(slug): Path<String>,
  Query(p): Query<BrowseParams>,
) -> ApiResult<BrowseResponse> {
  let dictionary: Dictionary = sqlx::query_as(&format!(
    "select {DICT_COLS} from dictionaries where slug = $1"
  ))
  .bind(&slug)
  .fetch_optional(&state.db)
  .await?
  .ok_or_else(|| ApiError::not_found("dictionary not found"))?;
  let before = p.before.unwrap_or(15).clamp(0, 200);
  let after = p.after.unwrap_or(30).clamp(0, 200);

  let mut center = p.seq.unwrap_or(0).max(0);
  if let Some(q) = p.q.as_deref().filter(|q| !q.trim().is_empty()) {
    let keys = query_keys(q);
    let loose: Vec<String> = keys.iter().map(|k| loose_key(k)).collect();
    // exact → loose → first prefix match
    let found: Option<(i32,)> = sqlx::query_as(
      "select min(e.seq) from entry_keys k join entries e on e.id = k.entry_id
       where k.dictionary_id = $1 and k.key_slp1 = any($2) having min(e.seq) is not null",
    )
    .bind(dictionary.id)
    .bind(&keys)
    .fetch_optional(&state.db)
    .await?;
    let found = match found {
      Some(f) => Some(f),
      None => {
        sqlx::query_as(
          "select min(e.seq) from entry_keys k join entries e on e.id = k.entry_id
           where k.dictionary_id = $1 and (k.key_loose = any($2) or k.key_slp1 like $3 || '%')
           having min(e.seq) is not null",
        )
        .bind(dictionary.id)
        .bind(&loose)
        .bind(keys.first().cloned().unwrap_or_default())
        .fetch_optional(&state.db)
        .await?
      }
    };
    match found {
      Some((s,)) => center = s,
      None => return Err(ApiError::not_found("headword not found in this dictionary")),
    }
  }

  let items: Vec<BrowseItem> = sqlx::query_as(
    "select id, seq, headword from entries where dictionary_id = $1 and seq between $2 and $3 order by seq",
  )
  .bind(dictionary.id)
  .bind(center - before)
  .bind(center + after)
  .fetch_all(&state.db)
  .await?;
  Ok(Json(BrowseResponse {
    dictionary,
    items,
    center,
  }))
}
