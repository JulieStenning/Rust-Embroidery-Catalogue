// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

// Database migration helpers using SQLx
// Recompiled with 20261101000001_reclassify_alphabet_glyphs
use crate::error::AppError;
use sha2::{Digest, Sha384};
use std::borrow::Cow;
use std::collections::HashMap;

use sqlx::migrate::MigrateError;
use sqlx::SqlitePool;
use tokio::time::{sleep, Duration};

/// Reconciles stored migration checksums in `_sqlx_migrations` if they were recorded
/// using Windows CRLF line endings instead of canonical LF line endings.
///
/// When a repository or database is migrated across platforms or environments with different
/// git `core.autocrlf` configurations, SQLx compares raw byte hashes. If a migration's
/// stored checksum matches the SHA-384 of its CRLF-normalised SQL, this function safely
/// updates the tracking table to the canonical compiled checksum so that `sqlx::migrate!`
/// recognises the migration as identical and valid.
pub async fn reconcile_crlf_migration_checksums(pool: &SqlitePool) -> Result<(), AppError> {
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to check _sqlx_migrations table: {e}")))?;

    if !table_exists {
        return Ok(());
    }

    let migrator = sqlx::migrate!("./migrations");

    // Fetch existing applied migration records
    let applied: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations")
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::database(format!("failed to fetch _sqlx_migrations records: {e}"))
            })?;

    let applied_map: HashMap<i64, Vec<u8>> = applied.into_iter().collect();

    for migration in migrator.iter() {
        if let Some(stored_checksum) = applied_map.get(&migration.version) {
            if stored_checksum.as_slice() != migration.checksum.as_ref() {
                // Compute the CRLF SHA-384 of the migration SQL
                let lf_sql = migration.sql.as_ref().replace("\r\n", "\n");
                let crlf_sql = lf_sql.replace('\n', "\r\n");
                let mut hasher = Sha384::new();
                hasher.update(crlf_sql.as_bytes());
                let crlf_checksum = hasher.finalize();

                if stored_checksum.as_slice() == &crlf_checksum[..] {
                    tracing::info!(
                        version = migration.version,
                        "Reconciling migration CRLF checksum to canonical LF checksum in _sqlx_migrations"
                    );
                    sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
                        .bind(migration.checksum.as_ref())
                        .bind(migration.version)
                        .execute(pool)
                        .await
                        .map_err(|e| {
                            AppError::database(format!(
                                "failed to update reconciled checksum for migration {}: {e}",
                                migration.version
                            ))
                        })?;
                }
            }
        }
    }

    Ok(())
}

/// Run all pending migrations from the `migrations/` directory.
/// SQLx will track which migrations have been applied via the `_sqlx_migrations` table.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    const MAX_ATTEMPTS: u32 = 6;
    const RETRY_DELAY_MS: u64 = 750;

    for attempt in 1..=MAX_ATTEMPTS {
        if let Err(e) = reconcile_crlf_migration_checksums(pool).await {
            tracing::warn!("Migration checksum reconciliation skipped or failed: {e}");
        }

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
