//! Shared application state (extractable via `State<AppState>`).

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
  /// SeaORM connection (sqlx Postgres pool underneath).
  pub db: DatabaseConnection,
}
