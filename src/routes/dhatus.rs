//! Dhātupāṭha roots and the dhātu works that discuss them.

use super::{ApiError, ApiResult, DictEntries, ENTRY_SELECT, Entry, stmt};
use crate::entities::{dhatus, entries};
use crate::state::AppState;
use crate::translit::{deva_key, query_keys, root_variants};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
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
}

impl From<dhatus::Model> for Dhatu {
  fn from(m: dhatus::Model) -> Self {
    Dhatu {
      id: m.id,
      number: m.number,
      upadesha: m.upadesha,
      root: m.root,
      artha: m.artha,
      gana: m.gana,
      pada: m.pada,
      it: m.it,
      extra: m.extra,
    }
  }
}

#[derive(Deserialize)]
pub struct ListParams {
  q: Option<String>,
  gana: Option<i16>,
}

#[utoipa::path(
    get,
    path = "/api/dhatus",
    operation_id = "dhatus.list",
    tag = "dhatus",
    params(
        ("q" = Option<String>, Query, description = "Root, upadesha, artha or number filter"),
        ("gana" = Option<i16>, Query, description = "Filter by gana (1-10)"),
    ),
    responses(
        (status = 200, description = "Dhatupatha roots (whole list, max 2500)", body = Vec<Dhatu>),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn list(
  State(state): State<AppState>,
  Query(p): Query<ListParams>,
) -> ApiResult<Vec<Dhatu>> {
  let q = p.q.unwrap_or_default();
  let q = q.trim().to_string();
  let keys = if q.is_empty() { vec![] } else { query_keys(&q) };
  let mut query = dhatus::Entity::find()
    .order_by_asc(dhatus::Column::Gana)
    // Dhātupāṭha number within the gaṇa (`1.1147` → 1147)
    .order_by_asc(Expr::cust("nullif(split_part(number, '.', 2), '')::int"))
    .order_by_asc(dhatus::Column::Seq)
    .limit(2500);
  if let Some(g) = p.gana {
    query = query.filter(dhatus::Column::Gana.eq(g));
  }
  if !q.is_empty() {
    let mut cond = Condition::any()
      .add(dhatus::Column::RootSlp1.is_in(keys.clone()))
      .add(dhatus::Column::Number.eq(&q))
      .add(dhatus::Column::Upadesha.contains(&q))
      .add(dhatus::Column::Artha.contains(&q));
    if let Some(k) = keys.first() {
      cond = cond.add(dhatus::Column::RootSlp1.starts_with(k));
    }
    query = query.filter(cond);
  }
  let mut rows = query.all(&state.db).await?;
  // Exact root matches first (stable: keeps Dhātupāṭha order otherwise).
  rows.sort_by_key(|r| !keys.contains(&r.root_slp1));
  Ok(Json(rows.into_iter().map(Dhatu::from).collect()))
}

#[derive(Serialize, ToSchema)]
pub struct DhatuDetail {
  dhatu: Dhatu,
  works: Vec<DictEntries>,
}

#[utoipa::path(
    get,
    path = "/api/dhatus/{id}",
    operation_id = "dhatus.get",
    tag = "dhatus",
    params(("id" = i32, Path, description = "Dhatu id")),
    responses(
        (status = 200, description = "Dhatu with discussing works", body = DhatuDetail),
        (status = 404, description = "Dhatu not found", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn detail(State(state): State<AppState>, Path(id): Path<i32>) -> ApiResult<DhatuDetail> {
  let dhatu = dhatus::Entity::find_by_id(id)
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("dhātu not found"))?;
  // Entries linked to this root, plus unlinked dhātu-work entries whose key
  // matches the root (ambiguous roots are not linked at import time).
  let keys = root_variants(&dhatu.root_slp1, &deva_key(&dhatu.upadesha));
  let entries: Vec<Entry> = entries::Entity::find()
    .from_raw_sql(stmt(
      &format!(
        "select {ENTRY_SELECT} from entries e join dictionaries d on d.id = e.dictionary_id
         where d.kind = 'dhatu_work' and d.slug <> 'dhatupatha' and (
           e.dhatu_id = $1 or (e.dhatu_id is null and exists (
             select 1 from entry_keys k where k.entry_id = e.id and k.key_slp1 = any($2))))
         order by d.sort_order, e.seq limit 200"
      ),
      vec![id.into(), keys.into()],
    ))
    .all(&state.db)
    .await?
    .into_iter()
    .map(Entry::from)
    .collect();
  let dicts = super::all_dictionaries(&state).await?;
  Ok(Json(DhatuDetail {
    dhatu: dhatu.into(),
    works: super::group_by_dictionary(&dicts, entries),
  }))
}
