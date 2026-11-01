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
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub async fn init_state() -> Result<AppState, String> {
  let _ = dotenvy::from_filename(".env.local");
  let _ = dotenvy::dotenv();
  let database_url = std::env::var("PG_DATABASE_URL")
    .map_err(|_| "PG_DATABASE_URL must be set: missing environment variable".to_string())?;
  // Production goes through PgBouncer (pooler port 6432), which multiplexes
  // server connections, so per-instance pools can be generous: parallel
  // handler queries each hold a connection. Direct connections (local dev,
  // import) use the same default; override with DB_MAX_CONNECTIONS.
  let max_connections = std::env::var("DB_MAX_CONNECTIONS")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(10);
  let mut opts = ConnectOptions::new(crate::db_url::normalize(&database_url));
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
  // Caches the 29-row dictionary list (refreshed with a TTL on read).
  AppState::new(db)
    .await
    .map_err(|e| format!("failed to load dictionaries: {e}"))
}

async fn api_not_found(uri: Uri) -> impl IntoResponse {
  (
    StatusCode::NOT_FOUND,
    axum::Json(serde_json::json!({ "error": format!("Not found: {}", uri.path()) })),
  )
}

/// Composed OpenAPI-aware router (used for the OpenAPI document and serving).
pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::with_openapi(crate::openapi::ApiDoc::openapi())
    .merge(crate::routes::openapi_router())
}

pub fn router(state: AppState) -> Router {
  let api: Router<AppState> = openapi_router().into();
  let api = api
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
