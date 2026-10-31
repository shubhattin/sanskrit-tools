//! Search keys (`entry_keys`): SLP1 + loose key per entry.

use super::enums::KeyKind;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "entry_keys")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub entry_id: i32,
  #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
  pub key_slp1: String,
  pub dictionary_id: i32,
  #[sea_orm(column_type = "Text")]
  pub key_loose: String,
  pub kind: KeyKind,
}

impl ActiveModelBehavior for ActiveModel {}
