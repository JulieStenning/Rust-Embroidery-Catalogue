// Database migration helpers using SQLx
use crate::error::AppError;
use std::borrow::Cow;

use sqlx::migrate::MigrateError;
use sqlx::SqlitePool;
use tokio::time::{sleep, Duration};

/// Run all pending migrations from the `migrations/` directory.
/// SQLx will track which migrations have been applied via the `_sqlx_migrations` table.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    const MAX_ATTEMPTS: u32 = 6;
    const RETRY_DELAY_MS: u64 = 750;

    for attempt in 1..=MAX_ATTEMPTS {
        match sqlx::migrate!("./migrations").run(pool).await {
            Ok(()) => return Ok(()),
            Err(err) if is_already_exists_error(&err) => {
                // Schema already exists â€” this happens when the seed database
                // is used but the _sqlx_migrations tracking table is missing.
                // This is not an error; the schema is in place.
                tracing::info!(
                    "Migration 'already exists' â€” schema is already applied (likely a seed DB). Skipping."
                );
                return Ok(());
            }
            Err(err) if is_locked_migration_error(&err) && attempt < MAX_ATTEMPTS => {
                tracing::warn!(
                    "Database is locked while running migrations (attempt {}/{}). Retrying in {}ms...",
                    attempt,
                    MAX_ATTEMPTS,
                    RETRY_DELAY_MS
                );
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
            Err(err) => {
                return Err(AppError::database(format!(
                    "database migration failed: {err}"
                )))
            }
        }
    }

    unreachable!("Migration retry loop should always return before reaching this point")
}

fn is_locked_migration_error(err: &MigrateError) -> bool {
    match err {
        MigrateError::Execute(inner) | MigrateError::ExecuteMigration(inner, _) => {
            is_sqlite_locked_error(inner)
        }
        _ => false,
    }
}

fn is_sqlite_locked_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => {
            let code = db_err.code().unwrap_or(Cow::Borrowed(""));
            code == "5"
                || db_err
                    .message()
                    .to_ascii_lowercase()
                    .contains("database is locked")
        }
        _ => false,
    }
}

/// Check whether the error indicates the target schema object already exists.
/// This can happen when a pre-migrated seed database is used but the
/// `_sqlx_migrations` tracking table is missing or incomplete.
pub fn is_already_exists_error(err: &MigrateError) -> bool {
    match err {
        MigrateError::ExecuteMigration(inner, _) | MigrateError::Execute(inner) => match inner {
            sqlx::Error::Database(db_err) => {
                let msg = db_err.message().to_ascii_lowercase();
                msg.contains("already exists")
            }
            _ => false,
        },
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
