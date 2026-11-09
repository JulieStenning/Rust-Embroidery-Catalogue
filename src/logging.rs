// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

// Structured logging via tracing + tracing-appender.
// Provides daily-rolling file output for release builds and
// dual file+stdout output for debug builds (cargo tauri dev).

use crate::error::AppError;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Holds the non-blocking writer guards so they stay alive for the app's lifetime.
/// When dropped, pending log writes are flushed to disk.
pub struct LogGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
    #[allow(dead_code)]
    _stdout_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

/// Initialise the tracing subscriber with file + optional stdout output.
/// Returns a LogGuard that must be kept in AppState so it lives for the app's lifetime.
///
/// NOTE: The caller is responsible for ensuring `log_dir` exists and points to the
/// correct application log directory (e.g. via `paths::resolve_app_paths().log_dir`).
pub fn init_logging(log_dir: &Path) -> Result<LogGuard, AppError> {
    std::fs::create_dir_all(log_dir).map_err(|err| {
        AppError::io(format!(
            "failed to create log dir {}: {err}",
            log_dir.display()
        ))
    })?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .build(log_dir)
        .map_err(|err| {
            AppError::io(format!(
                "failed to create rolling file appender in {}: {err}",
                log_dir.display()
            ))
        })?;
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(false);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(debug_assertions)]
    {
        let (non_blocking_stdout, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
        let stdout_layer = fmt::layer()
            .with_writer(non_blocking_stdout)
            .with_target(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(stdout_layer)
            .try_init()
            .map_err(|err| {
                AppError::invalid_input(format!("failed to initialize tracing subscriber: {err}"))
            })?;

        Ok(LogGuard {
            _file_guard: file_guard,
            _stdout_guard: Some(stdout_guard),
        })
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .try_init()
            .map_err(|err| {
                AppError::invalid_input(format!("failed to initialize tracing subscriber: {err}"))
            })?;

        Ok(LogGuard {
            _file_guard: file_guard,
            _stdout_guard: None,
        })
    }
}

/// Default log retention duration (14 days).
pub const LOG_RETENTION_DURATION: std::time::Duration = std::time::Duration::from_secs(14 * 86_400);

/// Log retention in days.
pub const LOG_RETENTION_DAYS: u64 = 14;

/// Prune log files inside `log_dir` that are older than `max_age_duration`.
///
/// Daily application logs (`app.*.log`) and backfill logs (`backfill_*.log`)
/// are evaluated. The age is determined from the file's last modified timestamp.
/// If an old file cannot be removed (e.g. locked by a running process or
/// permission denied), a warning is logged and pruning continues for other files.
///
/// Returns the number of files successfully deleted.
pub fn prune_old_log_files(
    log_dir: &Path,
    max_age_duration: std::time::Duration,
) -> Result<usize, AppError> {
    if !log_dir.is_dir() {
        return Ok(0);
    }

    let entries = std::fs::read_dir(log_dir).map_err(|err| {
        AppError::io(format!(
            "failed to read log directory {}: {err}",
            log_dir.display()
        ))
    })?;

    let now = std::time::SystemTime::now();
    let mut pruned_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let is_target_log = (file_name.starts_with("app.") || file_name.starts_with("backfill_"))
            && file_name.ends_with(".log");
        if !is_target_log {
            continue;
        }

        let is_expired = match entry.metadata().and_then(|m| m.modified()) {
            Ok(modified) => match now.duration_since(modified) {
                Ok(age) => age >= max_age_duration,
                Err(_) => false,
            },
            Err(_) => false,
        };

        if is_expired {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    pruned_count += 1;
                    tracing::info!("Pruned old log file: {}", path.display());
                }
                Err(err) => {
                    tracing::warn!("Could not remove old log file {}: {err}", path.display());
                }
            }
        }
    }

    Ok(pruned_count)
}

impl LogGuard {
    /// Create a dummy `LogGuard` for use in tests where a real logging
    /// setup is not needed. Connects to `std::io::sink()` to avoid
    /// writing actual log files.
    #[cfg(test)]
    pub(crate) fn dummy_for_test() -> Self {
        let (_, file_guard) = tracing_appender::non_blocking(std::io::sink());
        LogGuard {
            _file_guard: file_guard,
            _stdout_guard: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper to create a unique temp directory for each test run.
    fn test_log_dir() -> PathBuf {
        std::env::temp_dir().join("embroidery_logging_test")
    }

    /// Test that init_logging creates the directory and writes at least one log file.
    ///
    /// NOTE: Only one test can call init_logging per process because
    /// tracing_subscriber::registry().try_init() is a global-once operation.
    /// A second call returns an error (it does not panic), which this test
    /// asserts below, so all assertions are combined into this single test.
    #[test]
    fn test_init_logging_creates_directory_and_log_files() {
        let log_dir = test_log_dir();

        // Clean any leftover state from a previous failed run.
        let _ = std::fs::remove_dir_all(&log_dir);
        assert!(
            !log_dir.exists(),
            "Precondition: directory should not exist yet"
        );

        // Act – initialise logging.
        let guard = init_logging(&log_dir).unwrap();

        // Assert the directory was created.
        assert!(log_dir.exists(), "Log directory should exist after init");
        assert!(log_dir.is_dir(), "Log directory should be a directory");

        // Assert at least one file entry exists inside (the rolled log file).
        let entries: Vec<_> = std::fs::read_dir(&log_dir)
            .expect("Should be able to read log directory")
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty(), "At least one log file should exist");

        // The rolled file must be named `app.<yyyy-mm-dd>.log` so it sorts by
        // date and opens with the OS default `.log` handler.
        for entry in &entries {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            assert!(
                name.starts_with("app.") && name.ends_with(".log"),
                "log file must be named app.<yyyy-mm-dd>.log, got: {name}"
            );
        }

        // Drop the guard so pending writes are flushed.
        drop(guard);

        // Second initialisation: the global tracing subscriber is already set,
        // so try_init() returns Err and init_logging maps it to
        // AppError::invalid_input (it does not panic).
        let second = init_logging(&log_dir);
        match second {
            Err(AppError::InvalidInput { message }) => assert!(
                message.contains("failed to initialize tracing subscriber"),
                "unexpected invalid-input message: {message}"
            ),
            _ => panic!("second init_logging should fail with InvalidInput"),
        }

        // Clean up after ourselves.
        let _ = std::fs::remove_dir_all(&log_dir);
    }

    /// Test that init_logging returns an AppError::Io when the log directory
    /// cannot be created (here, because a regular file occupies the parent path).
    /// This fails before any global subscriber is touched, so it is independent
    /// of the single-init constraint above.
    #[test]
    fn test_init_logging_returns_io_error_when_dir_cannot_be_created() {
        // A regular file in the way of the directory path forces create_dir_all
        // to fail.
        let base = std::env::temp_dir().join("embroidery_logging_test_blocker");
        let _ = std::fs::remove_file(&base);
        std::fs::write(&base, []).expect("create blocker file");
        let log_dir = base.join("subdir");

        let result = init_logging(&log_dir);
        match result {
            Err(AppError::Io { message }) => assert!(
                message.contains("failed to create log dir"),
                "unexpected io message: {message}"
            ),
            _ => panic!("expected AppError::Io when log dir cannot be created"),
        }

        let _ = std::fs::remove_file(&base);
    }

    #[test]
    fn test_prune_old_log_files() {
        let log_dir = std::env::temp_dir().join("embroidery_logging_prune_test");
        let _ = std::fs::remove_dir_all(&log_dir);
        std::fs::create_dir_all(&log_dir).expect("create test log dir");

        let fresh_file = log_dir.join("app.2026-11-08.log");
        let old_file = log_dir.join("app.2026-10-01.log");
        let fresh_backfill = log_dir.join("backfill_info.log");
        let old_backfill = log_dir.join("backfill_errors.log");
        let unrelated_file = log_dir.join("notes.txt");

        std::fs::write(&fresh_file, b"fresh log").expect("write fresh");
        std::fs::write(&old_file, b"old log").expect("write old");
        std::fs::write(&fresh_backfill, b"fresh backfill").expect("write fresh backfill");
        std::fs::write(&old_backfill, b"old backfill").expect("write old backfill");
        std::fs::write(&unrelated_file, b"keep me").expect("write unrelated");

        // Set mtime of old_file and old_backfill to 20 days ago
        let now = std::time::SystemTime::now();
        let twenty_days_ago = now - std::time::Duration::from_secs(20 * 86_400);
        let file = std::fs::File::options()
            .write(true)
            .open(&old_file)
            .expect("open old");
        file.set_modified(twenty_days_ago).expect("set old mtime");

        let bf_file = std::fs::File::options()
            .write(true)
            .open(&old_backfill)
            .expect("open old backfill");
        bf_file
            .set_modified(twenty_days_ago)
            .expect("set old backfill mtime");

        // Set mtime of unrelated_file to 20 days ago as well (should not be pruned because not app.*.log or backfill_*.log)
        let un_file = std::fs::File::options()
            .write(true)
            .open(&unrelated_file)
            .expect("open unrelated");
        un_file
            .set_modified(twenty_days_ago)
            .expect("set unrelated mtime");

        let pruned = prune_old_log_files(&log_dir, std::time::Duration::from_secs(14 * 86_400))
            .expect("prune should succeed");
        assert_eq!(
            pruned, 2,
            "should have pruned exactly 2 old log files (1 daily log + 1 backfill log)"
        );

        assert!(fresh_file.exists(), "fresh log file must still exist");
        assert!(!old_file.exists(), "old log file must be pruned");
        assert!(
            fresh_backfill.exists(),
            "fresh backfill log must still exist"
        );
        assert!(!old_backfill.exists(), "old backfill log must be pruned");
        assert!(unrelated_file.exists(), "unrelated file must not be pruned");

        let _ = std::fs::remove_dir_all(&log_dir);
    }
}
