//! Aṣṭādhyāyī sūtras (`sutras`), id `a.p.nnn`.

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sutras")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub adhyaya: i16,
  pub pada: i16,
  pub num: i16,
  #[sea_orm(column_type = "Text")]
  pub text: String,
  #[sea_orm(column_type = "Text")]
  pub text_slp1: String,
}

impl ActiveModelBehavior for ActiveModel {}
