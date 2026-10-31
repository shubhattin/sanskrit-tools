//! Dhātupāṭha roots (`dhatus`).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dhatus")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub number: String,
  #[sea_orm(column_type = "Text")]
  pub upadesha: String,
  #[sea_orm(column_type = "Text")]
  pub root: String,
  #[sea_orm(column_type = "Text")]
  pub root_slp1: String,
  #[sea_orm(column_type = "Text")]
  pub artha: String,
  pub gana: i16,
  pub pada: Option<String>,
  pub it: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub extra: Option<String>,
  pub seq: i32,
}

impl ActiveModelBehavior for ActiveModel {}
