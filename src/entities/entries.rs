//! Dictionary entries (`entries`): one headword group of a work, book order.

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "entries")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub dictionary_id: i32,
  pub seq: i32,
  #[sea_orm(column_type = "Text")]
  pub headword: String,
  /// Sanitized HTML, one item per sense
  pub senses: Vec<String>,
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub data: Option<Json>,
  pub sutra_id: Option<String>,
  pub dhatu_id: Option<i32>,
  #[sea_orm(column_type = "Text", nullable)]
  pub gloss_en: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
