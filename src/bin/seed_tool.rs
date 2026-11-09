// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Embroidery Catalogue Seed Database Utility
//!
//! A headless CLI tool for migrating, sanitising, and compacting the bundled
//! delivery seed database (`src-tauri/resources/EmbroideryCatalogue.db`)
//! without launching the desktop GUI application or saving licence credentials.
//!
//! ## Usage:
//! ```text
//! cargo run --bin seed-tool -- [OPTIONS]
//!
//! Options:
//!     --migrate         Run migrations, sanitise settings, compact, and verify (default action)
//!     --check           Inspect database status, check migrations and verify cleanliness without modifying
//!     --path <PATH>     Specify a custom database file path (defaults to src-tauri/resources/EmbroideryCatalogue.db)
//!     --help, -h        Print this help message
//! ```

use embroidery_catalogue::database::migrations::run_migrations;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Row, SqlitePool};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolMode {
    Migrate,
    Check,
}

struct ToolConfig {
    mode: ToolMode,
    db_path: PathBuf,
}

fn print_usage() {
    println!(
        r#"
Embroidery Catalogue - Seed Database Release Utility

USAGE:
    cargo run --bin seed-tool -- [OPTIONS]

OPTIONS:
    --migrate         Run migrations, sanitise settings, compact, and verify (default)
    --check           Read-only verification of migrations and delivery sanitisation
    --path <PATH>     Custom database file path
                      (default: src-tauri/resources/EmbroideryCatalogue.db)
    --help, -h        Display this guidance
"#
    );
}

fn parse_args() -> Result<Option<ToolConfig>, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut mode = ToolMode::Migrate;
    let mut custom_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(None);
            }
            "--migrate" => {
                mode = ToolMode::Migrate;
            }
            "--check" | "--verify" => {
                mode = ToolMode::Check;
            }
            "--path" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing argument value after '--path'".to_string());
                }
                custom_path = Some(PathBuf::from(&args[i]));
            }
            unknown => {
                return Err(format!(
                    "Unrecognised option '{}'. Run with --help for options.",
                    unknown
                ));
            }
        }
        i += 1;
    }

    let default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src-tauri")
        .join("resources")
        .join("EmbroideryCatalogue.db");

    let db_path = custom_path.unwrap_or(default_path);

    Ok(Some(ToolConfig { mode, db_path }))
}

async fn open_pool(path: &Path) -> Result<SqlitePool, String> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| {
            format!(
                "Failed to open database connection for '{}': {}",
                path.display(),
                e
            )
        })
}

fn backup_database(path: &Path) -> Result<PathBuf, String> {
    let backup_path = path.with_extension("db.bak");
    fs::copy(path, &backup_path).map_err(|e| {
        format!(
            "Failed to create backup at '{}': {}",
            backup_path.display(),
            e
        )
    })?;
    Ok(backup_path)
}

async fn count_rows(pool: &SqlitePool, table: &'static str) -> Result<i64, String> {
    let query = format!("SELECT COUNT(*) FROM {}", table);
    let row = sqlx::query(sqlx::AssertSqlSafe(query))
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Error counting rows in table '{}': {}", table, e))?;
    let count: i64 = row.get(0);
    Ok(count)
}

async fn sanitise_seed_database(pool: &SqlitePool) -> Result<(), String> {
    // 1. Remove all licence details from settings
    sqlx::query(
        "DELETE FROM settings WHERE key IN (
            'licence_email',
            'licence_key',
            'licence_tier',
            'licence_expires_at'
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to purge licence settings: {}", e))?;

    // 2. Reset onboarding / runtime state flags to pristine defaults
    sqlx::query("UPDATE settings SET value = '0' WHERE key = 'initial_setup_completed'")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to reset initial_setup_completed: {}", e))?;

    sqlx::query("UPDATE settings SET value = 'FALSE' WHERE key = 'disclaimer_accepted'")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to reset disclaimer_accepted: {}", e))?;

    sqlx::query("UPDATE settings SET value = '' WHERE key = 'import.last_browse_folder'")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to reset last_browse_folder: {}", e))?;

    Ok(())
}

async fn verify_sanitisation(pool: &SqlitePool) -> Result<(), String> {
    // Check for any licence settings
    let rows = sqlx::query("SELECT key, value FROM settings WHERE key LIKE 'licence_%'")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to query settings for licence data: {}", e))?;

    for row in rows {
        let key: String = row.get("key");
        let val: String = row.get("value");
        if !val.trim().is_empty() {
            return Err(format!(
                "Sanitisation failed: setting '{}' has value '{}'. Seed database must not contain licence data.",
                key, val
            ));
        }
    }

    // Ensure zero user designs
    let design_count = count_rows(pool, "designs").await?;
    if design_count > 0 {
        return Err(format!(
            "Sanitisation failed: found {} designs. Seed database must have 0 designs.",
            design_count
        ));
    }

    Ok(())
}

async fn verify_all_migrations_applied(pool: &SqlitePool) -> Result<(usize, i64), String> {
    let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let mut disk_versions = Vec::new();

    if migrations_dir.is_dir() {
        let entries = fs::read_dir(&migrations_dir)
            .map_err(|e| format!("Failed to read migrations directory: {}", e))?;
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.ends_with(".up.sql") {
                if let Some(prefix) = filename.split('_').next() {
                    if let Ok(version) = prefix.parse::<i64>() {
                        disk_versions.push((version, filename));
                    }
                }
            }
        }
    }

    disk_versions.sort_by_key(|(v, _)| *v);

    let applied_rows =
        sqlx::query("SELECT version, success FROM _sqlx_migrations ORDER BY version ASC")
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to query _sqlx_migrations: {}", e))?;

    let applied_versions: std::collections::HashMap<i64, bool> = applied_rows
        .into_iter()
        .map(|r| (r.get::<i64, _>("version"), r.get::<bool, _>("success")))
        .collect();

    for (version, filename) in &disk_versions {
        match applied_versions.get(version) {
            Some(true) => {}
            Some(false) => {
                return Err(format!(
                    "Migration {} ({}) failed in tracking table",
                    version, filename
                ));
            }
            None => {
                return Err(format!(
                    "Migration {} ({}) has not been applied to the seed database",
                    version, filename
                ));
            }
        }
    }

    let latest_version = disk_versions.last().map(|(v, _)| *v).unwrap_or(0);
    Ok((disk_versions.len(), latest_version))
}

async fn compact_database(pool: &SqlitePool) -> Result<(), String> {
    // Checkpoint any WAL records into the main database
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);")
        .execute(pool)
        .await
        .map_err(|e| format!("WAL checkpoint failed: {}", e))?;

    // Vacuum defragments all b-trees and shrinks the SQLite file
    sqlx::query("VACUUM;")
        .execute(pool)
        .await
        .map_err(|e| format!("VACUUM failed: {}", e))?;

    // Run query optimiser index statistics
    sqlx::query("PRAGMA optimize;")
        .execute(pool)
        .await
        .map_err(|e| format!("PRAGMA optimize failed: {}", e))?;

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let config = match parse_args() {
        Ok(Some(cfg)) => cfg,
        Ok(None) => return ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("\nError: {}", err);
            return ExitCode::FAILURE;
        }
    };

    println!("============================================================");
    println!("     EMBROIDERY CATALOGUE - SEED DATABASE UTILITY");
    println!("============================================================");
    println!("Target database: {}", config.db_path.display());
    println!("Mode:            {:?}", config.mode);
    println!("------------------------------------------------------------");

    if !config.db_path.exists() {
        eprintln!(
            "Error: Database file does not exist at '{}'",
            config.db_path.display()
        );
        return ExitCode::FAILURE;
    }

    let initial_size = fs::metadata(&config.db_path).map(|m| m.len()).unwrap_or(0);

    match config.mode {
        ToolMode::Check => {
            println!("Opening database in read-only verification mode...");
            let pool = match open_pool(&config.db_path).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            match verify_all_migrations_applied(&pool).await {
                Ok((count, latest)) => {
                    println!(
                        "[PASS] All {} migrations applied. Latest version: {}",
                        count, latest
                    );
                }
                Err(e) => {
                    eprintln!("[FAIL] Migration check: {}", e);
                    return ExitCode::FAILURE;
                }
            }

            match verify_sanitisation(&pool).await {
                Ok(()) => {
                    println!(
                        "[PASS] Seed database is completely sanitised (no licence or user data)."
                    );
                }
                Err(e) => {
                    eprintln!("[FAIL] Sanitisation check: {}", e);
                    return ExitCode::FAILURE;
                }
            }

            let tags_count = count_rows(&pool, "tags").await.unwrap_or(0);
            let synonyms_count = count_rows(&pool, "tag_synonyms").await.unwrap_or(0);
            println!("Starter tags:     {} tags", tags_count);
            println!("Starter synonyms: {} synonyms", synonyms_count);
            println!("File size:        {:.1} KB", initial_size as f64 / 1024.0);
            println!("============================================================");
            println!("Check complete: Database is ready for release delivery.");
            ExitCode::SUCCESS
        }
        ToolMode::Migrate => {
            println!("Creating pre-migration backup...");
            match backup_database(&config.db_path) {
                Ok(bak) => println!("Backup created:  {}", bak.display()),
                Err(e) => {
                    eprintln!("Aborting: {}", e);
                    return ExitCode::FAILURE;
                }
            }

            println!("Connecting and applying migrations...");
            let pool = match open_pool(&config.db_path).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = run_migrations(&pool).await {
                eprintln!("[FAIL] Migration failed: {}", e);
                return ExitCode::FAILURE;
            }
            println!("[PASS] Migrations executed successfully.");

            println!("Sanitising delivery settings (licence and onboarding flags)...");
            if let Err(e) = sanitise_seed_database(&pool).await {
                eprintln!("[FAIL] Sanitisation failed: {}", e);
                return ExitCode::FAILURE;
            }

            if let Err(e) = verify_sanitisation(&pool).await {
                eprintln!("[FAIL] Sanitisation verification failed: {}", e);
                return ExitCode::FAILURE;
            }
            println!("[PASS] Verified: Zero licence keys, zero emails, zero designs.");

            let (migration_count, latest_version) = match verify_all_migrations_applied(&pool).await
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("[FAIL] Verification of applied migrations failed: {}", e);
                    return ExitCode::FAILURE;
                }
            };
            println!(
                "[PASS] All {} migrations up to {} verified in tracking table.",
                migration_count, latest_version
            );

            println!("Compacting and optimising database pages (VACUUM & PRAGMA optimize)...");
            if let Err(e) = compact_database(&pool).await {
                eprintln!("[FAIL] Compaction failed: {}", e);
                return ExitCode::FAILURE;
            }

            let tags_count = count_rows(&pool, "tags").await.unwrap_or(0);
            let synonyms_count = count_rows(&pool, "tag_synonyms").await.unwrap_or(0);

            // Close connection pool cleanly before checking final file size
            pool.close().await;

            // Clean up temporary WAL/SHM files if created
            let wal_path = config.db_path.with_extension("db-wal");
            let shm_path = config.db_path.with_extension("db-shm");
            let _ = fs::remove_file(wal_path);
            let _ = fs::remove_file(shm_path);

            let final_size = fs::metadata(&config.db_path).map(|m| m.len()).unwrap_or(0);

            println!("------------------------------------------------------------");
            println!("Starter tags:     {} tags", tags_count);
            println!("Starter synonyms: {} synonyms", synonyms_count);
            println!("Original size:    {:.1} KB", initial_size as f64 / 1024.0);
            println!("Compacted size:   {:.1} KB", final_size as f64 / 1024.0);
            println!("============================================================");
            println!("SUCCESS: Seed database updated and ready for release delivery.");
            ExitCode::SUCCESS
        }
    }
}
