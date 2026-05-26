// Database connection management using SQLx
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::env;
use crate::config::DEFAULT_DATABASE_URL;

/// Create a SQLite connection pool (max 1 connection — appropriate for a local
/// single-user desktop app). The pool is `Send + Sync`, which allows it to be
/// used safely in Tauri's async command handlers.
pub async fn establish_connection() -> SqlitePool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database '{}': {}", database_url, e))
}
