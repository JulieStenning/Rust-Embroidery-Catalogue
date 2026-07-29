// Database connection management using SQLx
use crate::config::BootstrapConfig;
use crate::paths::AppPaths;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

/// Custom error type for database connection failures.
#[derive(Debug)]
pub enum ConnectionError {
    PoolConnect(String),
    BusyTimeout(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::PoolConnect(msg) => write!(f, "Pool connect: {}", msg),
            ConnectionError::BusyTimeout(msg) => write!(f, "Busy timeout: {}", msg),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Create a SQLite connection pool (max 1 connection — appropriate for a local
/// single-user desktop app). The pool is `Send + Sync`, which allows it to be
/// used safely in Tauri's async command handlers.
///
/// Accepts `&AppPaths` to derive the database URL from the resolved paths.
/// Returns `Result` rather than panicking so callers can surface errors gracefully.
pub async fn establish_connection(paths: &AppPaths) -> Result<SqlitePool, ConnectionError> {
    let bootstrap = BootstrapConfig::from_app_paths(paths);
    let database_url = bootstrap.database_url;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .map_err(|e| {
            ConnectionError::PoolConnect(format!(
                "Failed to connect to database '{}': {}",
                database_url, e
            ))
        })?;

    sqlx::query("PRAGMA busy_timeout = 30000")
        .execute(&pool)
        .await
        .map_err(|e| {
            ConnectionError::BusyTimeout(format!(
                "Failed to set SQLite busy timeout for '{}': {}",
                database_url, e
            ))
        })?;

    Ok(pool)
}

/// Legacy convenience wrapper for code paths not yet migrated to `AppPaths`.
/// Uses `BootstrapConfig::from_env()` to derive the database URL.
/// Panics on failure (to match previous behaviour during incremental migration).
pub async fn establish_connection_from_env() -> SqlitePool {
    let database_url = BootstrapConfig::from_env().database_url;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database '{}': {}", database_url, e));

    sqlx::query("PRAGMA busy_timeout = 30000")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to set SQLite busy timeout for '{}': {}",
                database_url, e
            )
        });

    pool
}
