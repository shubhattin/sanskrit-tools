//! Postgres enums (`dict_kind`, `lang`, `key_kind`).

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "dict_kind")]
#[serde(rename_all = "snake_case")]
pub enum DictKind {
  #[sea_orm(string_value = "lexicon")]
  Lexicon,
  #[sea_orm(string_value = "thesaurus")]
  Thesaurus,
  #[sea_orm(string_value = "dhatu_work")]
  DhatuWork,
  #[sea_orm(string_value = "sutra_commentary")]
  SutraCommentary,
  #[sea_orm(string_value = "concordance")]
  Concordance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "lang")]
#[serde(rename_all = "snake_case")]
pub enum Lang {
  #[sea_orm(string_value = "sa")]
  Sa,
  #[sea_orm(string_value = "en")]
  En,
  #[sea_orm(string_value = "hi")]
  Hi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "key_kind")]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
  #[sea_orm(string_value = "headword")]
  Headword,
  #[sea_orm(string_value = "pratipadika")]
  Pratipadika,
  #[sea_orm(string_value = "variant")]
  Variant,
  #[sea_orm(string_value = "syn")]
  Syn,
}
