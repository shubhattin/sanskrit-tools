//! Inflected form → paradigm cells (`paradigm_forms`).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "paradigm_forms")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub paradigm_id: i32,
  #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
  pub form_slp1: String,
  pub cells: Vec<i16>,
}

impl ActiveModelBehavior for ActiveModel {}
