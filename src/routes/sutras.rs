//! Aṣṭādhyāyī sūtras with their commentaries.

use super::{ApiError, ApiResult, DictEntries, ENTRY_COLS, Entry};
use crate::state::AppState;
use crate::translit::{has_devanagari, query_keys};
use axum::Json;
use axum::extract::{Path, Query, State};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::LazyLock;

#[derive(Serialize, FromRow)]
pub struct Sutra {
  id: String,
  adhyaya: i16,
  pada: i16,
  num: i16,
  text: String,
}

const COLS: &str = "id, adhyaya, pada, num, text";

#[derive(Deserialize)]
pub struct ListParams {
  q: Option<String>,
}

/// `1`, `1.1`, `1.1.1`, `1.1.001` → id prefix (`1.1.001`, or `1.1.` for a pāda).
fn number_prefix(q: &str) -> Option<String> {
  static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d)(?:\.(\d)(?:\.(\d{1,3}))?)?\.?$").unwrap());
  let c = RE.captures(q.trim())?;
  Some(match (c.get(2), c.get(3)) {
    (Some(p), Some(n)) => format!(
      "{}.{}.{:03}",
      &c[1],
      p.as_str(),
      n.as_str().parse::<u32>().ok()?
    ),
    (Some(p), None) => format!("{}.{}.", &c[1], p.as_str()),
    _ => format!("{}.", &c[1]),
  })
}

pub async fn list(
  State(state): State<AppState>,
  Query(p): Query<ListParams>,
) -> ApiResult<Vec<Sutra>> {
  let q = p.q.unwrap_or_default();
  let q = q.trim();
  let rows: Vec<Sutra> = if q.is_empty() {
    sqlx::query_as(&format!("select {COLS} from sutras order by id limit 200"))
      .fetch_all(&state.db)
      .await?
  } else if let Some(prefix) = number_prefix(q) {
    sqlx::query_as(&format!(
      "select {COLS} from sutras where id like $1 || '%' order by id limit 200"
    ))
    .bind(prefix)
    .fetch_all(&state.db)
    .await?
  } else {
    let key = query_keys(q).into_iter().next().unwrap_or_default();
    let text = if has_devanagari(q) {
      q.to_string()
    } else {
      String::new()
    };
    sqlx::query_as(&format!(
      "select {COLS} from sutras
       where ($1 <> '' and text_slp1 like '%' || $1 || '%') or ($2 <> '' and text like '%' || $2 || '%')
       order by (text_slp1 like $1 || '%') desc, id limit 200"
    ))
    .bind(key)
    .bind(text)
    .fetch_all(&state.db)
    .await?
  };
  Ok(Json(rows))
}

#[derive(Serialize)]
pub struct SutraDetail {
  sutra: Sutra,
  prev: Option<String>,
  next: Option<String>,
  commentaries: Vec<DictEntries>,
}

pub async fn detail(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<SutraDetail> {
  let id = number_prefix(&id)
    .filter(|p| !p.ends_with('.'))
    .unwrap_or(id);
  let sutra: Sutra = sqlx::query_as(&format!("select {COLS} from sutras where id = $1"))
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("sūtra not found"))?;
  let prev: Option<(String,)> =
    sqlx::query_as("select id from sutras where id < $1 order by id desc limit 1")
      .bind(&id)
      .fetch_optional(&state.db)
      .await?;
  let next: Option<(String,)> =
    sqlx::query_as("select id from sutras where id > $1 order by id limit 1")
      .bind(&id)
      .fetch_optional(&state.db)
      .await?;
  let entries: Vec<Entry> = sqlx::query_as(&format!(
    "select {ENTRY_COLS} from entries e where e.sutra_id = $1 order by e.dictionary_id, e.seq"
  ))
  .bind(&id)
  .fetch_all(&state.db)
  .await?;
  let dicts = super::all_dictionaries(&state).await?;
  Ok(Json(SutraDetail {
    sutra,
    prev: prev.map(|r| r.0),
    next: next.map(|r| r.0),
    commentaries: super::group_by_dictionary(&dicts, entries),
  }))
}

#[cfg(test)]
mod tests {
  use super::number_prefix;
  #[test]
  fn prefixes() {
    assert_eq!(number_prefix("1.1.1").as_deref(), Some("1.1.001"));
    assert_eq!(number_prefix("3.2.12").as_deref(), Some("3.2.012"));
    assert_eq!(number_prefix("1.1").as_deref(), Some("1.1."));
    assert_eq!(number_prefix("8").as_deref(), Some("8."));
    assert_eq!(number_prefix("vfdDi"), None);
  }
}
