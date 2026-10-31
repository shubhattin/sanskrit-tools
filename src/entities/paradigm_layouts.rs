//! Shared table layouts (`paradigm_layouts`).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "paradigm_layouts")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique)]
  pub key: String,
  #[sea_orm(column_type = "JsonBinary")]
  pub layout: Json,
}

impl ActiveModelBehavior for ActiveModel {}
