//! Dictionary agent messages (`agent_messages`): AI SDK `UIMessage`s as stored JSON.

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "agent_messages")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
  pub id: String,
  #[sea_orm(column_type = "Text")]
  pub chat_id: String,
  /// `user` | `assistant`
  #[sea_orm(column_type = "Text")]
  pub role: String,
  /// `UIMessage.parts`
  #[sea_orm(column_type = "JsonBinary")]
  pub parts: Json,
  /// `UIMessage.metadata`
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub metadata: Option<Json>,
  /// Order within the chat
  pub position: i32,
  pub created_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
