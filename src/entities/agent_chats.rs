//! Dictionary agent conversations (`agent_chats`).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "agent_chats")]
pub struct Model {
  /// Same id `useChat` uses on the client.
  #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
  pub id: String,
  #[sea_orm(column_type = "Text")]
  pub title: String,
  /// Model of the latest turn.
  #[sea_orm(column_type = "Text")]
  pub model: String,
  /// Dictionary slugs the agent searches by default.
  pub dicts: Vec<String>,
  /// True while a reply is being generated.
  pub streaming: bool,
  /// Set by Stop; the generating request polls it and aborts.
  pub stop_requested_at: Option<DateTimeWithTimeZone>,
  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
