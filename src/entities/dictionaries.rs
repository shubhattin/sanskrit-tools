//! One row per work (`dictionaries`).

use super::enums::{DictKind, Lang};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dictionaries")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique)]
  pub slug: String,
  #[sea_orm(column_type = "Text")]
  pub name: String,
  pub short_name: String,
  pub kind: DictKind,
  pub src_lang: Lang,
  pub tgt_lang: Lang,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub attribution: Option<String>,
  pub sort_order: i32,
  pub entry_count: i32,
  #[sea_orm(column_type = "Text", nullable)]
  pub source_hash: Option<String>,
  pub imported_at: Option<DateTimeWithTimeZone>,
}

impl ActiveModelBehavior for ActiveModel {}
