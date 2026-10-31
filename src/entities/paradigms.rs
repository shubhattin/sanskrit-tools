//! Inflection table sets (`paradigms`): full `tables` JSON, or `layout_id` +
//! `forms` text (see `crate::import::paradigms::expand_layout`).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "paradigms")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub source: String,
  pub seq: i32,
  #[sea_orm(column_type = "Text")]
  pub lemma: String,
  #[sea_orm(column_type = "Text")]
  pub lemma_slp1: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub root: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub root_slp1: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub label: Option<String>,
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub tables: Option<Json>,
  pub layout_id: Option<i32>,
  #[sea_orm(column_type = "Text", nullable)]
  pub forms: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
