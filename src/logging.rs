// Structured logging via tracing + tracing-appender.
// Provides daily-rolling file output for release builds and
// dual file+stdout output for debug builds (cargo tauri dev).

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
pub fn init_logging(log_dir: &Path) -> LogGuard {
    std::fs::create_dir_all(log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "app.log");
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(false);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(debug_assertions)]
    {
        let (non_blocking_stdout, stdout_guard) =
            tracing_appender::non_blocking(std::io::stdout());
        let stdout_layer = fmt::layer()
            .with_writer(non_blocking_stdout)
            .with_target(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();

        LogGuard {
            _file_guard: file_guard,
            _stdout_guard: Some(stdout_guard),
        }
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .init();

        LogGuard {
            _file_guard: file_guard,
            _stdout_guard: None,
        }
    }
}
