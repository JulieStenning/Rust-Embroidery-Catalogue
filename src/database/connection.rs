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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::ExecutionMode;
    use std::path::PathBuf;

    /// Helper to produce a unique temporary directory name.
    fn unique_tmp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "embroidery-connection-test-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    /// Convert a filesystem path to forward slashes for use in a `sqlite:///` URL.
    fn path_to_sqlite_abs_url(path: &std::path::Path) -> String {
        format!("sqlite:///{}", path.to_string_lossy().replace('\\', "/"))
    }

    // ─── ConnectionError::Display ────────────────────────────────────────────

    #[test]
    fn display_pool_connect_variant() {
        let err = ConnectionError::PoolConnect("something went wrong".into());
        let msg = format!("{}", err);
        assert_eq!(msg, "Pool connect: something went wrong");
    }

    #[test]
    fn display_busy_timeout_variant() {
        let err = ConnectionError::BusyTimeout("too many writers".into());
        let msg = format!("{}", err);
        assert_eq!(msg, "Busy timeout: too many writers");
    }

    // ─── establish_connection (happy path) ───────────────────────────────────

    #[tokio::test]
    async fn establish_connection_creates_pool_with_valid_app_paths() {
        let tmp = unique_tmp_dir("happy-path");
        let database_dir = tmp.join("Database");
        std::fs::create_dir_all(&database_dir).expect("create test database dir");

        let database_path = database_dir.join("test_catalogue.db");

        // Create the empty database file first so SQLite only opens it.
        // This avoids SQLite CANTOPEN issues with certain URL formats on Windows.
        std::fs::write(&database_path, []).expect("create empty db file");

        let paths = AppPaths {
            mode: ExecutionMode::Portable,
            data_root: tmp.clone(),
            embroidery_designs_dir: tmp.join("MachineEmbroideryDesigns"),
            database_dir: database_dir.clone(),
            database_path: database_path.clone(),
            thumbnail_cache_dir: tmp.join("thumbnails"),
            log_dir: tmp.join("logs"),
        };

        let pool = establish_connection(&paths)
            .await
            .expect("pool creation should succeed");

        // Verify the pool is usable by running a simple query.
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");

        assert_eq!(row.0, 1);

        pool.close().await;

        // Clean up.
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ─── establish_connection (error path) ───────────────────────────────────

    #[tokio::test]
    async fn establish_connection_returns_error_when_parent_dir_missing() {
        let tmp = unique_tmp_dir("error-path");
        // Intentionally do NOT create `tmp`, so the database_path parent is missing.
        let database_path = tmp.join("Database").join("catalogue.db");

        let paths = AppPaths {
            mode: ExecutionMode::Portable,
            data_root: tmp.clone(),
            embroidery_designs_dir: tmp.join("MachineEmbroideryDesigns"),
            database_dir: tmp.join("Database"),
            database_path,
            thumbnail_cache_dir: tmp.join("thumbnails"),
            log_dir: tmp.join("logs"),
        };

        let result = establish_connection(&paths).await;
        assert!(result.is_err(), "expected PoolConnect error, got Ok");

        match result {
            Err(ConnectionError::PoolConnect(msg)) => {
                assert!(msg.contains("Failed to connect to database"));
            }
            other => panic!("expected PoolConnect, got {:?}", other),
        }

        // No database file should have been created.
        assert!(!tmp.exists(), "tmp dir should not have been created");
    }

    // ─── establish_connection_from_env (happy path) ──────────────────────────

    #[tokio::test]
    async fn establish_connection_from_env_succeeds_with_valid_url() {
        let tmp = unique_tmp_dir("env-happy");
        let database_path = tmp.join("env_test.db");

        // Ensure parent directory exists.
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }

        // Create empty db file so SQLite can open it.
        std::fs::write(&database_path, []).expect("create empty db file");

        // Use the `sqlite:///` URI form with forward slashes — the standard
        // absolute-path URI format that SQLx understands on all platforms.
        let database_url = path_to_sqlite_abs_url(&database_path);

        // Save and override DATABASE_URL.
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", &database_url);

        let pool = establish_connection_from_env().await;

        // Verify the pool is usable.
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(row.0, 1);

        pool.close().await;

        // Restore the original variable.
        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }

        // Clean up.
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ─── establish_connection_from_env (panic path) ──────────────────────────

    #[tokio::test]
    #[should_panic(expected = "Failed to connect to database")]
    async fn establish_connection_from_env_panics_on_invalid_path() {
        let tmp = unique_tmp_dir("env-panic");
        // Do NOT create tmp — the parent directory is missing, so connect will fail.
        let database_path = tmp.join("nonexistent").join("db.db");
        let database_url = path_to_sqlite_abs_url(&database_path);

        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", &database_url);

        // This should panic.
        let _pool = establish_connection_from_env().await;

        // Restore (only reached if the panic doesn't fire).
        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }
}