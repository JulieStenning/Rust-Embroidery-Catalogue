// Database migration helpers using SQLx
use std::borrow::Cow;

use sqlx::migrate::MigrateError;
use sqlx::SqlitePool;
use tokio::time::{sleep, Duration};

/// Run all pending migrations from the `migrations/` directory.
/// SQLx will track which migrations have been applied via the `_sqlx_migrations` table.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    const MAX_ATTEMPTS: u32 = 6;
    const RETRY_DELAY_MS: u64 = 750;

    for attempt in 1..=MAX_ATTEMPTS {
        match sqlx::migrate!("./migrations").run(pool).await {
            Ok(()) => return Ok(()),
            Err(err) if is_locked_migration_error(&err) && attempt < MAX_ATTEMPTS => {
                println!(
                    "Database is locked while running migrations (attempt {}/{}). Retrying in {}ms...",
                    attempt,
                    MAX_ATTEMPTS,
                    RETRY_DELAY_MS
                );
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
            Err(err) => return Err(err),
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
