//! Shared application state (extractable via `State<AppState>`).

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
}
