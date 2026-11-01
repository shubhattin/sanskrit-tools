//! Shared application state (extractable via `State<AppState>`).

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::routes::Dictionary;

/// How long the cached dictionary list is trusted. Dictionaries only change on
/// re-import (rare, manual); between refreshes this saves 1–2 DB round trips
/// on *every* API request (search, browse, sutra/dhatu detail all read it).
const DICTIONARIES_TTL: Duration = Duration::from_secs(300);

struct DictionariesCache {
  loaded_at: Instant,
  dictionaries: Arc<Vec<Dictionary>>,
}

#[derive(Clone)]
pub struct AppState {
  /// SeaORM connection (sqlx Postgres pool underneath).
  pub db: DatabaseConnection,
  /// Shared HTTP client (Better Auth session lookups).
  pub http: reqwest::Client,
  /// Better Auth server origin (`VITE_BETTER_AUTH_URL`, no trailing slash).
  /// Shared with the frontend's auth client.
  pub better_auth_url: String,
  dictionaries: Arc<RwLock<DictionariesCache>>,
}

impl AppState {
  /// Load-once constructor: reads the dictionary list a single time.
  pub async fn new(
    db: DatabaseConnection,
    http: reqwest::Client,
    better_auth_url: String,
  ) -> Result<Self, sea_orm::DbErr> {
    let dictionaries = Arc::new(crate::routes::load_dictionaries(&db).await?);
    Ok(AppState {
      db,
      http,
      better_auth_url,
      dictionaries: Arc::new(RwLock::new(DictionariesCache {
        loaded_at: Instant::now(),
        dictionaries,
      })),
    })
  }

  /// The dictionary list, refreshed from the DB at most every
  /// [`DICTIONARIES_TTL`]. Cheap (`Arc` clone) on the hot path.
  pub async fn dictionaries(&self) -> Result<Arc<Vec<Dictionary>>, sea_orm::DbErr> {
    {
      let cache = self.dictionaries.read().await;
      if cache.loaded_at.elapsed() < DICTIONARIES_TTL {
        return Ok(cache.dictionaries.clone());
      }
    }
    let mut cache = self.dictionaries.write().await;
    // Another task may have refreshed while we waited for the write lock.
    if cache.loaded_at.elapsed() < DICTIONARIES_TTL {
      return Ok(cache.dictionaries.clone());
    }
    let fresh = Arc::new(crate::routes::load_dictionaries(&self.db).await?);
    cache.dictionaries = fresh.clone();
    cache.loaded_at = Instant::now();
    Ok(fresh)
  }
}
