// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

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

    configure_pragmas(&pool, &database_url).await?;

    Ok(pool)
}

/// Apply the standard SQLite PRAGMA configuration on a freshly opened
/// connection:
///  - `busy_timeout = 30000` — wait up to 30s for a busy database.
///  - `foreign_keys = ON` — SQLite defaults FK enforcement off per connection;
///    the app relies on `ON DELETE CASCADE` for `design_tags`/`project_designs`.
///  - `auto_vacuum = INCREMENTAL` — enables freelist page reclamation via
///    `PRAGMA incremental_vacuum(N)` (see `services::compaction`).
///
/// NOTE: `PRAGMA auto_vacuum` only takes effect for a *freshly created*
/// database file. On an existing database the mode can only be changed by
/// running a one-off full `VACUUM` after setting the PRAGMA.
///
/// The application never creates a brand-new database at runtime — it
/// always opens a copy of the shipped seed DB (`src-tauri/resources/`),
/// which has already been converted to incremental auto-vacuum mode (see
/// `scripts/convert_auto_vacuum.py`). The same one-off conversion was
/// applied to the developer DB at `Data/Database/`. The startup PRAGMA
/// below is therefore a harmless no-op on those pre-converted files; it is
/// retained so the connection configuration is explicit and defensive.
async fn configure_pragmas(
    pool: &sqlx::SqlitePool,
    database_url: &str,
) -> Result<(), ConnectionError> {
    sqlx::query("PRAGMA busy_timeout = 30000")
        .execute(pool)
        .await
        .map_err(|e| {
            ConnectionError::BusyTimeout(format!(
                "Failed to set SQLite busy timeout for '{}': {}",
                database_url, e
            ))
        })?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(|e| {
            ConnectionError::BusyTimeout(format!(
                "Failed to enable foreign keys for '{}': {}",
                database_url, e
            ))
        })?;

    sqlx::query("PRAGMA auto_vacuum = INCREMENTAL")
        .execute(pool)
        .await
        .map_err(|e| {
            ConnectionError::BusyTimeout(format!(
                "Failed to set auto_vacuum = INCREMENTAL for '{}': {}",
                database_url, e
            ))
        })?;

    match read_auto_vacuum_mode(pool).await {
        Ok(mode) => {
            tracing::info!(
                "SQLite PRAGMA configuration complete — database={}, auto_vacuum={}, foreign_keys=ON, busy_timeout=30000",
                database_url, mode
            );
        }
        Err(e) => {
            tracing::warn!(
                "Configured SQLite PRAGMAs but could not read auto_vacuum mode for '{}': {}",
                database_url,
                e
            );
        }
    }

    Ok(())
}

/// Read the current `auto_vacuum` mode (0 = NONE, 1 = FULL, 2 = INCREMENTAL)
/// using the `pragma_auto_vacuum` table-valued function so it is fetchable
/// under SQLx.
async fn read_auto_vacuum_mode(pool: &sqlx::SqlitePool) -> Result<i64, ConnectionError> {
    let (mode,): (i64,) = sqlx::query_as("SELECT auto_vacuum FROM pragma_auto_vacuum")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            ConnectionError::BusyTimeout(format!("Failed to read auto_vacuum mode: {}", e))
        })?;
    Ok(mode)
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
            mode: ExecutionMode::Installed,
            data_root: tmp.clone(),
            embroidery_designs_dir: tmp.join("MachineEmbroideryDesigns"),
            database_dir: database_dir.clone(),
            database_path: database_path.clone(),
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
            mode: ExecutionMode::Installed,
            data_root: tmp.clone(),
            embroidery_designs_dir: tmp.join("MachineEmbroideryDesigns"),
            database_dir: tmp.join("Database"),
            database_path,
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
}
