//! Aṣṭādhyāyī sūtras with their commentaries.

use super::{ApiError, ApiResult, DictEntries, Entry};
use crate::entities::{entries, sutras};
use crate::state::AppState;
use crate::translit::{has_devanagari, query_keys};
use axum::Json;
use axum::extract::{Path, Query, State};
use regex::Regex;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// OpenAPI route registration (paths served by [`crate::app::router`] (API stable)).
pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes!(list))
    .routes(routes!(detail))
}

#[derive(Serialize, ToSchema)]
pub struct Sutra {
  id: String,
  adhyaya: i16,
  pada: i16,
  num: i16,
  text: String,
}

impl From<sutras::Model> for Sutra {
  fn from(m: sutras::Model) -> Self {
    Sutra {
      id: m.id,
      adhyaya: m.adhyaya,
      pada: m.pada,
      num: m.num,
      text: m.text,
    }
  }
}

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

const LIST_LIMIT: usize = 200;

#[utoipa::path(
    get,
    path = "/api/sutras",
    operation_id = "sutras.list",
    tag = "sutras",
    params(("q" = Option<String>, Query, description = "Number prefix (e.g. 1.1) or text")),
    responses(
        (status = 200, description = "Sutras (max 200)", body = Vec<Sutra>),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn list(
  State(state): State<AppState>,
  Query(p): Query<ListParams>,
) -> ApiResult<Vec<Sutra>> {
  let q = p.q.unwrap_or_default();
  let q = q.trim();
  let base = sutras::Entity::find().order_by_asc(sutras::Column::Id);
  let rows: Vec<sutras::Model> = if q.is_empty() {
    base.limit(LIST_LIMIT as u64).all(&state.db).await?
  } else if let Some(prefix) = number_prefix(q) {
    base
      .filter(sutras::Column::Id.starts_with(&prefix))
      .limit(LIST_LIMIT as u64)
      .all(&state.db)
      .await?
  } else {
    let key = query_keys(q).into_iter().next().unwrap_or_default();
    let mut cond = Condition::any();
    if !key.is_empty() {
      cond = cond.add(sutras::Column::TextSlp1.contains(&key));
    }
    if has_devanagari(q) {
      cond = cond.add(sutras::Column::Text.contains(q));
    }
    // Only ~4k sūtras: rank sūtras *starting* with the query first, in Rust.
    let mut rows = base.filter(cond).all(&state.db).await?;
    rows.sort_by_key(|r| !r.text_slp1.starts_with(&key));
    rows.truncate(LIST_LIMIT);
    rows
  };
  Ok(Json(rows.into_iter().map(Sutra::from).collect()))
}

#[derive(Serialize, ToSchema)]
pub struct SutraDetail {
  sutra: Sutra,
  prev: Option<String>,
  next: Option<String>,
  commentaries: Vec<DictEntries>,
}

#[utoipa::path(
    get,
    path = "/api/sutras/{id}",
    operation_id = "sutras.get",
    tag = "sutras",
    params(("id" = String, Path, description = "Sutra id (e.g. 1.1.001)")),
    responses(
        (status = 200, description = "Sutra with commentaries", body = SutraDetail),
        (status = 404, description = "Sutra not found", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn detail(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> ApiResult<SutraDetail> {
  let id = number_prefix(&id)
    .filter(|p| !p.ends_with('.'))
    .unwrap_or(id);
  // All four reads are independent given the id — run them concurrently.
  let (sutra, prev, next, entries) = tokio::join!(
    sutras::Entity::find_by_id(id.clone()).one(&state.db),
    sutras::Entity::find()
      .filter(sutras::Column::Id.lt(&id))
      .order_by_desc(sutras::Column::Id)
      .one(&state.db),
    sutras::Entity::find()
      .filter(sutras::Column::Id.gt(&id))
      .order_by_asc(sutras::Column::Id)
      .one(&state.db),
    entries::Entity::find()
      .filter(entries::Column::SutraId.eq(&id))
      .order_by_asc(entries::Column::DictionaryId)
      .order_by_asc(entries::Column::Seq)
      .all(&state.db),
  );
  let sutra = sutra?.ok_or_else(|| ApiError::not_found("sūtra not found"))?;
  let prev = prev?;
  let next = next?;
  let entries: Vec<Entry> = entries?.into_iter().map(Entry::from).collect();
  let dicts = super::all_dictionaries(&state).await?;
  Ok(Json(SutraDetail {
    sutra: sutra.into(),
    prev: prev.map(|r| r.id),
    next: next.map(|r| r.id),
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
