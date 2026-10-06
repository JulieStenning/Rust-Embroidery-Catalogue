// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared application state managed by Tauri.

use crate::logging;
use crate::paths;
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::atomic::AtomicBool;

// ---------------------------------------------------------------------------
// Database status (exposed to frontend for the recovery flow)
// ---------------------------------------------------------------------------

/// Status of the configured database at startup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatusKind {
    /// No configured data root yet - first-run setup wizard handles it.
    Uninitialized,
    /// The configured database file exists and was opened normally.
    Connected,
    /// A configured data root exists but the database file is missing
    /// (e.g. a portable drive letter changed). The recovery view handles it.
    Missing,
    /// The database file exists but cannot be opened or is corrupted.
    Corrupted,
}

/// Detailed database status report sent to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStatus {
    pub status: DatabaseStatusKind,
    pub configured_data_root: Option<String>,
    pub database_path: Option<String>,
    pub embroidery_dir: Option<String>,
    pub data_root_missing: bool,
    pub error_message: Option<String>,
}

/// Compute the database status for the current paths/configuration.
pub fn database_status_from_paths(paths: &paths::AppPaths) -> DatabaseStatus {
    let (configured_str, data_root_missing, status) = match paths.mode {
        paths::ExecutionMode::Dev => {
            let configured = Some(paths.data_root.to_string_lossy().to_string());
            let missing = !paths.data_root.exists();
            let st = if missing || !paths.database_path.exists() {
                DatabaseStatusKind::Missing
            } else {
                DatabaseStatusKind::Connected
            };
            (configured, missing, st)
        }
        paths::ExecutionMode::Installed => {
            let configured_root = paths::read_bootstrap_data_root().ok().flatten();
            let configured = configured_root
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
            let missing = configured_root
                .as_ref()
                .map(|root| !root.exists())
                .unwrap_or(false);
            let st = match configured_root {
                None => DatabaseStatusKind::Uninitialized,
                Some(_) if missing => DatabaseStatusKind::Missing,
                Some(_) if !paths.database_path.exists() => DatabaseStatusKind::Missing,
                Some(_) => DatabaseStatusKind::Connected,
            };
            (configured, missing, st)
        }
    };

    DatabaseStatus {
        status,
        configured_data_root: configured_str,
        database_path: Some(paths.database_path.to_string_lossy().to_string()),
        embroidery_dir: Some(paths.embroidery_designs_dir.to_string_lossy().to_string()),
        data_root_missing,
        error_message: None,
    }
}

// ---------------------------------------------------------------------------

/// Holds the live SQLite pool in a way that can be swapped at runtime for a
/// database restore.
///
/// Cloning the holder is cheap (an `Arc`), and `pool()` clones the underlying
/// `SqlitePool` handle so commands keep a stable pool across `.await` points.
/// A restore takes the current pool out (closing it), replaces the database
/// file on disk, and installs a fresh pool via `replace`.
#[derive(Clone, Default)]
pub struct PoolHolder {
    inner: std::sync::Arc<std::sync::Mutex<Option<SqlitePool>>>,
}

impl PoolHolder {
    /// Wrap a freshly-created pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(Some(pool))),
        }
    }

    /// Clone the currently installed pool, or `None` if a restore has removed it.
    pub fn pool(&self) -> Option<SqlitePool> {
        self.inner
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
    }

    /// Remove and return the current pool so it can be closed before a restore
    /// swaps the underlying database file. Returns `None` if already absent.
    pub fn take(&self) -> Option<SqlitePool> {
        self.inner.lock().ok().and_then(|mut guard| guard.take())
    }

    /// Install a new pool after a restore, dropping (and thereby closing) any
    /// previous one. Safe to call even if no pool is currently installed.
    pub fn replace(&self, new: SqlitePool) {
        let previous = self
            .inner
            .lock()
            .ok()
            .and_then(|mut guard| guard.replace(new));
        // A `SqlitePool` closes itself when the last handle is dropped.
        drop(previous);
    }
}

/// Coordinates long-running background tasks, cancellation tokens, and task state.
#[derive(Default)]
pub struct TaskCoordinator {
    pub backfill_running: AtomicBool,
    pub backfill_stop_requested: AtomicBool,
    pub bulk_import_stop_requested: AtomicBool,
    pub backup_cancel_requested: AtomicBool,
    pub restore_cancel_requested: AtomicBool,
}

/// Shared application state managed by Tauri.
/// The pool is held in a `PoolHolder` so a restore can close and replace it;
/// commands obtain a cheap clone via `AppState::db_pool`.
pub struct AppState {
    /// Connection pool for the SQLite database (swappable at runtime).
    pub db: PoolHolder,
    /// Status of the configured database (Connected / Missing / Uninitialized).
    pub database_status: DatabaseStatus,
    /// Resolved application paths (Portable vs Installed mode).
    pub paths: paths::AppPaths,
    /// Log guard — kept alive so log writes are flushed on app exit.
    pub log_guard: logging::LogGuard,
    /// Flag signalled when the app is shutting down; background tasks can check it.
    pub shutdown_requested: AtomicBool,
    /// Atomic guard preventing overlapping incremental-vacuum maintenance runs.
    pub maintenance_running: AtomicBool,
    /// True while a catalogue storage migration is in progress.
    pub migration_running: AtomicBool,
    /// Cooperative cancellation flag observed by the running migration loop.
    pub migration_cancel_requested: std::sync::Arc<AtomicBool>,
    /// True while a database restore is closing/swapping the live pool, so other
    /// commands fail fast instead of acquiring a closed pool.
    pub restore_in_progress: AtomicBool,
    /// Coordinator for background tasks and cooperative cancellation flags.
    pub tasks: TaskCoordinator,
}

impl AppState {
    /// Clone the current live database pool for use by a command. Fails fast
    /// while a database restore is swapping the pool so no command touches a
    /// closed pool.
    pub fn db_pool(&self) -> Result<SqlitePool, String> {
        if self
            .restore_in_progress
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err("The database is being restored; please retry shortly.".to_string());
        }
        self.db
            .pool()
            .ok_or_else(|| "The database pool is unavailable.".to_string())
    }
}

// ---------------------------------------------------------------------------
// AppStatus (exposed to frontend)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub execution_mode: String,
    pub data_root: String,
    pub embroidery_dir: String,
    pub database_path: String,
    /// True when a previously-configured data root is no longer present on disk
    /// (e.g. a portable drive letter changed). The frontend offers a recovery
    /// dialog to reselect the location.
    pub data_root_missing: bool,
    /// True when a configured data root exists but the database file is missing.
    pub database_missing: bool,
}

/// Pure function to construct an `AppStatus` from `AppPaths`.
/// Extracted for testability — this does not depend on Tauri state.
pub fn app_status_from_paths(paths: &paths::AppPaths) -> AppStatus {
    let mode_str = match paths.mode {
        paths::ExecutionMode::Dev => "dev".to_string(),
        paths::ExecutionMode::Installed => "installed".to_string(),
    };

    // Only Installed mode can have a configured-then-missing root; Dev mode
    // always resolves to the project dev_data folder.
    let data_root_missing = matches!(paths.mode, paths::ExecutionMode::Installed)
        && paths::configured_data_root_missing()
            .ok()
            .flatten()
            .unwrap_or(false);

    // A database is "missing" when a data root is configured but the derived
    // DB file does not exist. This is the recovery-flow condition.
    let has_configured_root = matches!(paths.mode, paths::ExecutionMode::Installed)
        && paths::read_bootstrap_data_root().ok().flatten().is_some();
    let database_missing = has_configured_root && !paths.database_path.exists();

    AppStatus {
        execution_mode: mode_str,
        data_root: paths.data_root.to_string_lossy().to_string(),
        embroidery_dir: paths.embroidery_designs_dir.to_string_lossy().to_string(),
        database_path: paths.database_path.to_string_lossy().to_string(),
        data_root_missing,
        database_missing,
    }
}
