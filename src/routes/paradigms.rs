//! Inflection tables (declension, kṛdanta, conjugation).

use super::{ApiError, ApiResult};
use crate::entities::{paradigm_layouts, paradigms};
use crate::state::AppState;
use crate::translit::{query_keys, root_variants};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::sea_query::Expr;
use sea_orm::{
  ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, FromQueryResult)]
pub struct ParadigmSummary {
  id: i32,
  source: String,
  lemma: String,
  root: Option<String>,
  label: Option<String>,
}

#[derive(Serialize)]
pub struct Paradigm {
  id: i32,
  source: String,
  lemma: String,
  root: Option<String>,
  label: Option<String>,
  tables: serde_json::Value,
}

#[derive(Deserialize)]
pub struct ListParams {
  q: Option<String>,
}

/// Paradigms whose stem is the query, plus — when the query is a root — its
/// conjugation and all kṛdanta stems derived from it.
pub async fn list(
  State(state): State<AppState>,
  Query(p): Query<ListParams>,
) -> ApiResult<Vec<ParadigmSummary>> {
  let keys = query_keys(p.q.as_deref().unwrap_or(""));
  if keys.is_empty() {
    return Err(ApiError::bad_request("q is required"));
  }
  let mut root_keys = Vec::new();
  for k in &keys {
    for v in root_variants(k, k) {
      if !root_keys.contains(&v) {
        root_keys.push(v);
      }
    }
  }
  use paradigms::Column as C;
  let rows = paradigms::Entity::find()
    .select_only()
    .columns([C::Id, C::Source, C::Lemma, C::Root, C::Label])
    .filter(
      Condition::any()
        .add(C::LemmaSlp1.is_in(keys))
        .add(
          Condition::all()
            .add(C::Source.eq("heritage_kridanta"))
            .add(C::RootSlp1.is_in(root_keys.clone())),
        )
        .add(
          Condition::all()
            .add(C::Source.is_in(["heritage_conj", "jnu_tinanta"]))
            .add(C::LemmaSlp1.is_in(root_keys)),
        ),
    )
    .order_by_asc(Expr::cust(
      "case source when 'heritage_decl' then 0 when 'heritage_conj' then 1 \
       when 'jnu_tinanta' then 2 else 3 end",
    ))
    .order_by_asc(C::LemmaSlp1)
    .order_by_asc(C::Seq)
    .limit(300)
    .into_model::<ParadigmSummary>()
    .all(&state.db)
    .await?;
  Ok(Json(rows))
}

pub async fn detail(State(state): State<AppState>, Path(id): Path<i32>) -> ApiResult<Paradigm> {
  let p = paradigms::Entity::find_by_id(id)
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("paradigm not found"))?;
  let tables = match (p.tables, p.layout_id, p.forms) {
    (Some(t), _, _) => t,
    (None, Some(layout_id), Some(forms)) => {
      let layout = paradigm_layouts::Entity::find_by_id(layout_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("paradigm layout not found"))?;
      crate::import::paradigms::expand_layout(&layout.layout, &forms)
    }
    _ => serde_json::Value::Array(vec![]),
  };
  Ok(Json(Paradigm {
    id: p.id,
    source: p.source,
    lemma: p.lemma,
    root: p.root,
    label: p.label,
    tables,
  }))
}
