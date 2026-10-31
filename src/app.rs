//! App initialisation + router composition.
//!
//! - `/api/*`: read-only JSON API ([`crate::routes`], contract in `docs/api.md`).
//! - Everything else: the built frontend from `STATIC_DIR` (SPA fallback to
//!   its shell HTML) when set; otherwise 404. In dev Vite serves the frontend
//!   and proxies `/api` here.

use crate::state::AppState;
use axum::Router;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

pub async fn init_state() -> Result<AppState, String> {
  let _ = dotenvy::from_filename(".env.local");
  let _ = dotenvy::dotenv();
  let database_url = std::env::var("PG_DATABASE_URL")
    .map_err(|_| "PG_DATABASE_URL must be set: missing environment variable".to_string())?;
  // Serverless instances each hold their own pool: keep it small on Vercel.
  let default_max = if std::env::var_os("VERCEL").is_some() {
    2
  } else {
    10
  };
  let max_connections = std::env::var("DB_MAX_CONNECTIONS")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(default_max);
  let mut opts = ConnectOptions::new(database_url);
  opts
    .max_connections(max_connections)
    .acquire_timeout(Duration::from_secs(10))
    .sqlx_logging(false);
  if let Some(schema) = std::env::var("DATABASE_SCHEMA")
    .ok()
    .filter(|s| !s.trim().is_empty())
  {
    opts.set_schema_search_path(schema);
  }
  let db = Database::connect(opts)
    .await
    .map_err(|e| format!("failed to connect to database: {e}"))?;
  Ok(AppState { db })
}

async fn api_not_found(uri: Uri) -> impl IntoResponse {
  (
    StatusCode::NOT_FOUND,
    axum::Json(serde_json::json!({ "error": format!("Not found: {}", uri.path()) })),
  )
}

pub fn router(state: AppState) -> Router {
  let api = crate::routes::router()
    .route("/api/{*rest}", axum::routing::any(api_not_found))
    .with_state(state);
  let app = match std::env::var("STATIC_DIR").ok().filter(|s| !s.is_empty()) {
    Some(dir) => {
      let shell = ["_shell.html", "index.html"]
        .iter()
        .map(|f| std::path::Path::new(&dir).join(f))
        .find(|p| p.exists())
        .unwrap_or_else(|| std::path::Path::new(&dir).join("index.html"));
      api.fallback_service(ServeDir::new(&dir).fallback(ServeFile::new(shell)))
    }
    None => api,
  };
  app.layer(CompressionLayer::new())
}
