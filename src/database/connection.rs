// Database connection management using SQLx
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use crate::config::BootstrapConfig;

/// Create a SQLite connection pool (max 1 connection — appropriate for a local
/// single-user desktop app). The pool is `Send + Sync`, which allows it to be
/// used safely in Tauri's async command handlers.
pub async fn establish_connection() -> SqlitePool {
    let database_url = BootstrapConfig::from_env().database_url;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database '{}': {}", database_url, e));

    sqlx::query("PRAGMA busy_timeout = 30000")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to set SQLite busy timeout for '{}': {}", database_url, e));

    pool
}
