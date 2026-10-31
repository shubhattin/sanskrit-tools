//! Dhātupāṭha roots and the dhātu works that discuss them.

use super::{ApiError, ApiResult, DictEntries, ENTRY_COLS, Entry};
use crate::state::AppState;
use crate::translit::{deva_key, query_keys, root_variants};
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Dhatu {
  id: i32,
  number: String,
  upadesha: String,
  root: String,
  artha: String,
  gana: i16,
  pada: Option<String>,
  it: Option<String>,
  extra: Option<String>,
  #[serde(skip)]
  root_slp1: String,
}

const COLS: &str = "id, number, upadesha, root, artha, gana, pada, it, extra, root_slp1";
const ORDER: &str = "gana, nullif(split_part(number, '.', 2), '')::int nulls last, seq";

#[derive(Deserialize)]
pub struct ListParams {
  q: Option<String>,
  gana: Option<i16>,
}

pub async fn list(
  State(state): State<AppState>,
  Query(p): Query<ListParams>,
) -> ApiResult<Vec<Dhatu>> {
  let q = p.q.unwrap_or_default();
  let q = q.trim().to_string();
  let keys = if q.is_empty() { vec![] } else { query_keys(&q) };
  let rows: Vec<Dhatu> = sqlx::query_as(&format!(
    "select {COLS} from dhatus
     where ($1::smallint is null or gana = $1)
       and ($2 = '' or root_slp1 = any($3) or root_slp1 like ($3)[1] || '%'
            or number = $2 or upadesha like '%' || $2 || '%' or artha like '%' || $2 || '%')
     order by (root_slp1 = any($3)) desc, {ORDER} limit 300"
  ))
  .bind(p.gana)
  .bind(&q)
  .bind(&keys)
  .fetch_all(&state.db)
  .await?;
  Ok(Json(rows))
}

#[derive(Serialize)]
pub struct DhatuDetail {
  dhatu: Dhatu,
  works: Vec<DictEntries>,
}

pub async fn detail(State(state): State<AppState>, Path(id): Path<i32>) -> ApiResult<DhatuDetail> {
  let dhatu: Dhatu = sqlx::query_as(&format!("select {COLS} from dhatus where id = $1"))
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("dhātu not found"))?;
  // Entries linked to this root, plus unlinked dhātu-work entries whose key
  // matches the root (ambiguous roots are not linked at import time).
  let keys = root_variants(&dhatu.root_slp1, &deva_key(&dhatu.upadesha));
  let entries: Vec<Entry> = sqlx::query_as(&format!(
    "select {ENTRY_COLS} from entries e join dictionaries d on d.id = e.dictionary_id
     where d.kind = 'dhatu_work' and d.slug <> 'dhatupatha' and (
       e.dhatu_id = $1 or (e.dhatu_id is null and exists (
         select 1 from entry_keys k where k.entry_id = e.id and k.key_slp1 = any($2))))
     order by d.sort_order, e.seq limit 200"
  ))
  .bind(id)
  .bind(&keys)
  .fetch_all(&state.db)
  .await?;
  let dicts = super::all_dictionaries(&state).await?;
  Ok(Json(DhatuDetail {
    dhatu,
    works: super::group_by_dictionary(&dicts, entries),
  }))
}
