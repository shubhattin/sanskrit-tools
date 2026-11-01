//! Chat persistence (`agent_chats`, `agent_messages`) via SeaORM.

use crate::agent::history::UiMessage;
use crate::entities::{agent_chats, agent_messages};
use chrono::Utc;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
  ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
  QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

pub async fn find_chat(
  db: &DatabaseConnection,
  id: &str,
) -> Result<Option<agent_chats::Model>, DbErr> {
  agent_chats::Entity::find_by_id(id.to_string())
    .one(db)
    .await
}

pub async fn create_chat(
  db: &DatabaseConnection,
  id: &str,
  model: &str,
  dicts: &[String],
) -> Result<agent_chats::Model, DbErr> {
  agent_chats::Entity::insert(agent_chats::ActiveModel {
    id: Set(id.to_string()),
    title: Set(String::new()),
    model: Set(model.to_string()),
    dicts: Set(dicts.to_vec()),
    ..Default::default()
  })
  .on_conflict(
    OnConflict::column(agent_chats::Column::Id)
      .do_nothing()
      .to_owned(),
  )
  .exec_without_returning(db)
  .await?;
  find_chat(db, id)
    .await?
    .ok_or_else(|| DbErr::RecordNotFound(format!("agent chat {id}")))
}

pub async fn list_chats(db: &DatabaseConnection) -> Result<Vec<agent_chats::Model>, DbErr> {
  agent_chats::Entity::find()
    .order_by_desc(agent_chats::Column::UpdatedAt)
    .limit(300)
    .all(db)
    .await
}

pub async fn load_messages(
  db: &DatabaseConnection,
  chat_id: &str,
) -> Result<Vec<UiMessage>, DbErr> {
  Ok(
    agent_messages::Entity::find()
      .filter(agent_messages::Column::ChatId.eq(chat_id))
      .order_by_asc(agent_messages::Column::Position)
      .all(db)
      .await?
      .into_iter()
      .map(|row| UiMessage {
        id: row.id,
        role: row.role,
        parts: match row.parts {
          serde_json::Value::Array(parts) => parts,
          _ => Vec::new(),
        },
        metadata: row.metadata,
      })
      .collect(),
  )
}

/// Whether a message id is taken by another chat (ids come from the client).
pub async fn message_in_other_chat(
  db: &DatabaseConnection,
  message_id: &str,
  chat_id: &str,
) -> Result<bool, DbErr> {
  Ok(
    agent_messages::Entity::find_by_id(message_id.to_string())
      .one(db)
      .await?
      .is_some_and(|m| m.chat_id != chat_id),
  )
}

pub struct SaveTurn<'a> {
  pub chat_id: &'a str,
  /// The whole conversation as it should now be stored.
  pub messages: &'a [UiMessage],
  pub model: &'a str,
  pub dicts: &'a [String],
  pub title: Option<String>,
}

/// Make the stored conversation equal `messages` (a regenerated reply replaces what followed the
/// resent message) and bump the chat.
pub async fn save_turn(db: &DatabaseConnection, turn: SaveTurn<'_>) -> Result<(), DbErr> {
  let txn = db.begin().await?;
  // Message ids come from the client: never let one overwrite another chat's row.
  let ids: Vec<String> = turn.messages.iter().map(|m| m.id.clone()).collect();
  let foreign: Vec<String> = agent_messages::Entity::find()
    .select_only()
    .column(agent_messages::Column::Id)
    .filter(agent_messages::Column::Id.is_in(ids.clone()))
    .filter(agent_messages::Column::ChatId.ne(turn.chat_id))
    .into_tuple()
    .all(&txn)
    .await?;
  let keep: Vec<&UiMessage> = turn
    .messages
    .iter()
    .filter(|m| !foreign.contains(&m.id))
    .collect();

  agent_messages::Entity::delete_many()
    .filter(agent_messages::Column::ChatId.eq(turn.chat_id))
    .filter(agent_messages::Column::Id.is_not_in(keep.iter().map(|m| m.id.clone())))
    .exec(&txn)
    .await?;
  if !keep.is_empty() {
    let rows = keep
      .iter()
      .enumerate()
      .map(|(position, m)| agent_messages::ActiveModel {
        id: Set(m.id.clone()),
        chat_id: Set(turn.chat_id.to_string()),
        role: Set(m.role.clone()),
        parts: Set(serde_json::Value::Array(m.parts.clone())),
        metadata: Set(m.metadata.clone()),
        position: Set(position as i32),
        ..Default::default()
      });
    agent_messages::Entity::insert_many(rows)
      .on_conflict(
        OnConflict::column(agent_messages::Column::Id)
          .update_columns([
            agent_messages::Column::Parts,
            agent_messages::Column::Metadata,
            agent_messages::Column::Position,
          ])
          .to_owned(),
      )
      .exec_without_returning(&txn)
      .await?;
  }

  let mut update = agent_chats::Entity::update_many()
    .col_expr(agent_chats::Column::Model, Expr::value(turn.model))
    .col_expr(agent_chats::Column::Dicts, Expr::value(turn.dicts.to_vec()))
    .col_expr(
      agent_chats::Column::UpdatedAt,
      Expr::value(Utc::now().fixed_offset()),
    );
  if let Some(title) = turn.title.filter(|t| !t.is_empty()) {
    update = update.col_expr(agent_chats::Column::Title, Expr::value(title));
  }
  update
    .filter(agent_chats::Column::Id.eq(turn.chat_id))
    .exec(&txn)
    .await?;
  txn.commit().await
}

/// Mark a reply as generating (also clears a Stop left from the last one) or as done.
pub async fn set_streaming(
  db: &impl ConnectionTrait,
  id: &str,
  streaming: bool,
) -> Result<(), DbErr> {
  let mut update = agent_chats::Entity::update_many()
    .col_expr(agent_chats::Column::Streaming, Expr::value(streaming));
  if streaming {
    update = update.col_expr(
      agent_chats::Column::StopRequestedAt,
      Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
    );
  }
  update
    .filter(agent_chats::Column::Id.eq(id))
    .exec(db)
    .await?;
  Ok(())
}

/// Ask the reply generating for this chat to stop (it may run on another instance).
pub async fn request_stop(db: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
  agent_chats::Entity::update_many()
    .col_expr(
      agent_chats::Column::StopRequestedAt,
      Expr::value(Utc::now().fixed_offset()),
    )
    .filter(agent_chats::Column::Id.eq(id))
    .filter(agent_chats::Column::Streaming.eq(true))
    .exec(db)
    .await?;
  Ok(())
}

pub async fn is_stop_requested(db: &DatabaseConnection, id: &str) -> Result<bool, DbErr> {
  Ok(
    find_chat(db, id)
      .await?
      .is_some_and(|c| c.stop_requested_at.is_some()),
  )
}

pub async fn rename_chat(
  db: &DatabaseConnection,
  id: &str,
  title: &str,
) -> Result<Option<agent_chats::Model>, DbErr> {
  agent_chats::Entity::update_many()
    .col_expr(agent_chats::Column::Title, Expr::value(title))
    .filter(agent_chats::Column::Id.eq(id))
    .exec(db)
    .await?;
  find_chat(db, id).await
}

pub async fn delete_chat(db: &DatabaseConnection, id: &str) -> Result<bool, DbErr> {
  Ok(
    agent_chats::Entity::delete_by_id(id.to_string())
      .exec(db)
      .await?
      .rows_affected
      > 0,
  )
}
