# Final Plan: Logging Infrastructure, Exit Interception & Child Process Cleanup

---

## Context

This project currently uses only `println!()`/`eprintln!()` throughout the Rust backend. There is no structured logging, no file-based log output, no Windows subsystem attribute to suppress the console window in release builds, no `RunEvent` lifecycle hook for graceful shutdown, and no mechanism to signal in-flight Python child processes to terminate on exit.

This plan adds `tracing` + `tracing-appender` for file-based structured logging, hooks into Tauri's `RunEvent` for exit cleanup, adds a `shutdown_requested` flag so child processes can stop early, and ensures the release `.exe` has no visible terminal window.

The implementation **must continue to work in the development environment** (`cargo tauri dev` from a local disk), where `./data/` exists at the project root. The log directory resolution detects Portable mode by checking for `./data/` next to the executable, which also covers the development scenario where `cargo tauri dev` builds to `src-tauri/target/debug/` and `./data/` sits at the project root (the executable resolves relative to itself, so if `./data/` is present next to the exe or the exe is in a path that matches, it falls through to Installed mode which uses `%APPDATA%` — see detail below).

---

## Step 1: Add Dependencies to `Cargo.toml`

**File:** `Cargo.toml` (project root)

**Action:** Add three new dependencies under `[dependencies]`:

```toml
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

- `tracing` with `log` feature: captures any `log` crate macros from dependencies (future-proof).
- `tracing-subscriber` with `env-filter`: allows log-level control via `RUST_LOG` environment variable.
- `tracing-appender`: provides non-blocking daily-rolling file writer.

---

## Step 2: Create `src/logging.rs` — Logging Initialization Module

**File:** `src/logging.rs` (new file)

**Action:** Create a module that:

1. Resolves the log directory at startup.
2. Initialises a `tracing` subscriber with:
   - A **file layer**: daily-rolling `app.log` in the resolved log directory, using `tracing_appender::non_blocking` for async writes.
   - A **stdout layer** (debug builds only): prints to console during `cargo tauri dev`. Suppressed in release builds (no console).
3. Returns a `LogGuard` struct holding the `WorkerGuard` handles — these must be kept alive for the app's lifetime; dropping them flushes pending log writes.

**Log directory resolution logic (inline in `init_logging`):**

```
fn resolve_log_dir() -> PathBuf:
    1. Get the directory containing the running executable (std::env::current_exe() → parent).
    2. Check if <exe_dir>/data/ exists as a directory.
       - YES → Portable/SD Card mode: log_dir = <exe_dir>/data/logs/
       - NO  → Installed mode:
           - Windows: log_dir = %APPDATA%/EmbroideryCatalogue/logs/
           - macOS:   log_dir = ~/Library/Application Support/EmbroideryCatalogue/logs/
           - Linux:   log_dir = ~/.local/share/EmbroideryCatalogue/logs/
    3. Create log_dir recursively (create_dir_all).
    4. Return log_dir.
```

> **Development note:** During `cargo tauri dev`, the executable lives inside `src-tauri/target/debug/`. If `./data/` does not exist next to that debug exe path (it won't — `./data/` is at the project root), the resolution falls through to Installed mode and logs go to `%APPDATA%`. This is correct: in dev, you don't need portable logs; you have the console. To test Portable mode logging, place a `data/` directory next to the release `.exe` or set up the path so the exe sees it.

**Module structure:**

```rust
// src/logging.rs

use std::path::{Path, PathBuf};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Holds the non-blocking writer guards so they stay alive for the app's lifetime.
/// When dropped, pending log writes are flushed.
pub struct LogGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
    #[allow(dead_code)]
    _stdout_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

/// Initialise the tracing subscriber with file + optional stdout output.
/// Returns a LogGuard that must be kept in AppState.
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

/// Determine the log directory based on execution mode.
fn resolve_log_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    if exe_dir.join("data").is_dir() {
        // Portable / SD Card mode
        exe_dir.join("data").join("logs")
    } else {
        // Installed mode — use platform-specific app data directory
        let base = dirs_next_or_fallback();
        base.join("EmbroideryCatalogue").join("logs")
    }
}

/// Get the platform-specific user data directory without adding a new dependency.
fn dirs_next_or_fallback() -> PathBuf {
    // Use `dirs_next` if available, otherwise fall back to home dir
    // We'll implement this inline using std::env so no extra crate is needed.
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support")
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share")
    }
}
```

---

## Step 3: Modify `src/main.rs` — Windows Subsystem, Logging Init, Event Loop & AppState

**File:** `src/main.rs`

**Action (3a):** Add the Windows subsystem attribute at the very top of the file (line 1):

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

This must be the very first line, before any `pub mod` declarations. It tells the linker: release builds = no console; debug builds = console window (needed for `cargo tauri dev`).

**Action (3b):** Add the `pub mod logging;` declaration alongside the other module declarations (around line 8):

```rust
pub mod logging; // <-- ADD THIS
```

**Action (3c):** Add `LogGuard` and `shutdown_requested` to `AppState`:

```rust
use crate::logging::LogGuard;
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub db: SqlitePool,
    pub disclaimer_text: String,
    pub log_guard: LogGuard,                        // NEW
    pub shutdown_requested: AtomicBool,              // NEW
}
```

**Action (3d):** Replace the `fn main()` body. The new flow is:

```rust
fn main() {
    // ── Logging (must be first — before any print/tracing calls) ────────
    let log_dir = logging::resolve_log_dir_hidden(); // exposed from logging.rs
    let log_guard = logging::init_logging(&log_dir);
    tracing::info!("Embroidery Catalogue starting — log_dir={}", log_dir.display());

    // Load .env file if present (best-effort; not required in production)
    load_dotenv();

    // Resolve bootstrap configuration from process environment.
    let bootstrap_config = config::BootstrapConfig::from_env();
    tracing::info!("Parsed bootstrap configuration: {:#?}", bootstrap_config);

    // Ensure the database directory exists before trying to connect
    config::ensure_database_dir(&bootstrap_config.database_url);

    // Run async setup using Tauri's built-in Tokio runtime
    let (pool, disclaimer_text) = tauri::async_runtime::block_on(async {
        let pool = database::connection::establish_connection().await;

        database::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run database migrations");

        let disclaimer_text = include_str!("../disclaimer.html").to_string();
        (pool, disclaimer_text)
    });

    let app_state = AppState {
        db: pool.clone(),
        disclaimer_text,
        log_guard,
        shutdown_requested: AtomicBool::new(false),
    };

    // Launch a lightweight background backfill for orphan fingerprint data
    let fp_pool = app_state.db.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) =
            services::fingerprint::run_fingerprint_backfill(&fp_pool, 100).await
        {
            tracing::error!("Startup fingerprint backfill error: {}", err);
        }
    });

    routes::bulk_import::initialize_bulk_import_db_pool(app_state.db.clone());
    let startup_reset = routes::bulk_import::reset_bulk_import_context_store_for_startup();
    tracing::info!(
        "Bulk import context startup reset: cleared={}, active={}, resets={}, at_ms={}",
        startup_reset.cleared_context_count,
        startup_reset.active_context_count,
        startup_reset.reset_count,
        startup_reset.reset_at_millis
    );

    // ── Build & run with lifecycle event hook ──────────────────────────
    let app = tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            routes::bulk_import::initialize_bulk_import_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... all existing commands unchanged ...
        ])
        .build(tauri::generate_context!())
        .expect("Error while building the Embroidery Catalogue application");

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { code, .. } => {
                tracing::info!(
                    "Exit requested (code: {:?}) — signalling shutdown and flushing logs...",
                    code
                );
                // Signal all background tasks to stop.
                let state = app_handle.state::<AppState>();
                state.shutdown_requested.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            tauri::RunEvent::Exit => {
                tracing::info!("Embroidery Catalogue exiting.");
                // AppState (including LogGuard) is dropped here,
                // which flushes pending log writes to disk.
            }
            _ => {}
        }
    });
}
```

> **Important:** The `invoke_handler` macro call — the full list of all ~82 commands must be preserved exactly as-is. Only the surrounding structure changes (`.run()` → `.build().run()`).

**Action (3e):** Remove `println!("Parsed bootstrap configuration: {:#?}", bootstrap_config)`. Replaced by `tracing::info!`.

**Action (3f):** Remove `eprintln!("Startup fingerprint backfill error: {}", ...)`. Replaced by `tracing::error!`.

**Action (3g):** Remove `println!("Bulk import context startup reset: ...")`. Replaced by `tracing::info!`.

---

## Step 4: Expose `resolve_log_dir()` from `logging.rs`

**File:** `src/logging.rs`

**Action:** The `resolve_log_dir()` function is currently private to `logging.rs`. Add a public wrapper or make it `pub(crate)` so `main.rs` can call it to get the directory before initialising logging:

In `logging.rs`, change:
```rust
fn resolve_log_dir() -> PathBuf {
```
to:
```rust
pub(crate) fn resolve_log_dir() -> PathBuf {
```

---

## Step 5: Convert Remaining `println!`/`eprintln!` Calls to `tracing!` Macros

All print statements across the Rust codebase (excluding test-only output) are converted to the appropriate `tracing` level macro. The mapping:

| Current macro | Tracing equivalent | When to use |
|---------------|-------------------|-------------|
| `println!("...")` — informational | `tracing::info!("...")` | Startup messages, progress, completion summaries |
| `println!("...")` — debug/timing | `tracing::debug!("...")` | Fine-grained timing, per-file messages |
| `eprintln!("...")` — errors | `tracing::error!("...")` | Failures, unrecoverable conditions |
| `eprintln!("...")` — warnings | `tracing::warn!("...")` | Recoverable issues, unexpected but non-fatal |

### Files and specific conversions:

**5a. `src/config.rs`** (line 46):
```rust
// BEFORE
println!("Debug bootstrap configuration: {:#?}", config);
// AFTER
tracing::debug!("Debug bootstrap configuration: {:#?}", config);
```

**5b. `src/database/migrations.rs`** (lines ~18-19):
```rust
// BEFORE
println!("Database is locked while running migrations (attempt {}/{}). Retrying in {}ms...", ...);
// AFTER
tracing::warn!("Database is locked while running migrations (attempt {}/{}). Retrying in {}ms...", ...);
```

**5c. `src/services/image_generation.rs`** — multiple calls:
- `println!("[TIMING] Python batch timed out after {}ms with {}/{} results", ...)` → `tracing::warn!`
- `eprintln!("Skipping strict python/native parity assertions because python adapter is unavailable: {}", ...)` → `tracing::warn!`
- `eprintln!("Skipping complex VP3 parity assertions because python adapter is unavailable: {}", ...)` → `tracing::warn!`
- `eprintln!("Skipping user VP3 regression fixture test because file is not present: {}", ...)` → `tracing::warn!`

**5d. `src/services/backfill.rs`** — custom `log_info!`:
Replace the `log_info(format!(...))` call with `tracing::info!(...)`.

**5e. `src/routes/bulk_import.rs`** — multiple calls:
- `println!("Import file '{}' content-identical to existing '{}' — reusing stored path", ...)` → `tracing::debug!`
- `println!("Import collision: '{}' exists with different content — auto-renaming to '{}'", ...)` → `tracing::info!`
- `println!("Copied external file '{}' to managed directory '{}'", ...)` → `tracing::debug!`
- `println!("Failed to emit bulk import progress event: {error}")` → `tracing::error!`
- `println!("[TIMING] Python batch done: {}ms for {} file(s)", ...)` → `tracing::debug!`
- `println!("[TIMING] file={} backend={} image_gen={}ms{}", ...)` → `tracing::debug!`
- `println!("Image generation adapter error for '{}': {}", ...)` → `tracing::error!`
- `println!("Bulk import committed chunk [{}-{}]: ...", ...)` → `tracing::debug!`
- `println!("[TIMING] Bulk import complete: total={}ms | ...", ...)` → `tracing::info!`
- `println!("Bulk import DB pool not initialized; skipping persistence step.")` → `tracing::warn!`
- `println!("Preview dedup: excluded_by_path={} excluded_by_triple={} imported={}", ...)` → `tracing::info!`

**5f. `src/routes/maintenance.rs`** — multiple calls:
- `eprintln!("[backup] Could not create destination folder '{}': {}", ...)` → `tracing::error!`
- `eprintln!("[backup] Could not copy '{}' to '{}': {}", ...)` → `tracing::error!`
- `eprintln!("[backup] Could not create archive folder '{}': {}", ...)` → `tracing::error!`
- `eprintln!("[backup] Could not archive '{}' to '{}': {}", ...)` → `tracing::error!`
- `eprintln!("[backup] Could not clean up empty directories under '{}': {}", ...)` → `tracing::error!`

**5g. `src/readers/hus_reader.rs`** (line ~44):
```rust
// BEFORE
eprintln!("Warning: Encountered unknown stitch command byte: {cmd:#X} at index {i}");
// AFTER
tracing::warn!("Encountered unknown stitch command byte: {cmd:#X} at index {i}");
```

**5h. `src/readers/vp3_reader.rs`** — multiple calls:
- `eprintln!("Skipping VP3 diagnostics because fixture is missing: {}", ...)` → `tracing::warn!`
- `eprintln!("VP3 diagnostics 220306: stitches={}, max_len={:.2}, ...", ...)` → `tracing::debug!`
- `eprintln!("VP3 diagnostics test-less-220306: ...", ...)` → `tracing::debug!`
- `eprintln!("VP3 diagnostics 01Peacock: stitches={}, max_len={:.2}, ...", ...)` → `tracing::debug!`

**5i. `src/readers/jef_reader.rs`** (line ~33):
```rust
// BEFORE — per-stitch debug print (REMOVE ENTIRELY)
println!("stitch {}: ({}, {}) type: {:?}", ...);
// AFTER — deleted. No replacement.
```
This is noisy per-stitch output. Remove the line entirely.

---

## Step 6: Add Shutdown Flag Check in Child Process Code

**File:** `src/services/image_generation.rs`

**Action:** In the function that spawns Python (`run_python_batch` or similar, where `Command::new(&python_executable).spawn()` is called):

Before the `spawn()` call, add a check:

```rust
// Check if app is shutting down before spawning a new Python process
// (the shutdown flag is passed via the closure context from the caller)
if shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
    return; // or return an empty result
}
```

However, this function currently doesn't receive the shutdown flag. The cleanest approach for Phase 1:

1. Add a `shutdown_requested: Arc<AtomicBool>` field to the function signature (or use a global static).
2. In `main.rs` when spawning the backfill tasks, clone the `AtomicBool` (it supports `Clone`) and pass it into the async block or closure.
3. In `bulk_import.rs`, the `spawn_blocking` closure can check the flag before spawning Python.

**Minimal change for `services/image_generation.rs`:**

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub fn run_python_batch(
    file_paths: &[PathBuf],
    python_executable: &Path,
    script_path: &Path,
    shutdown_flag: &AtomicBool,  // NEW parameter
) -> HashMap<String, ImageGenerationResult> {
    if shutdown_flag.load(Ordering::SeqCst) {
        return HashMap::new(); // Early exit: app is closing
    }

    // ... existing spawn logic ...

    // Optionally check again after spawning, before waiting:
    if shutdown_flag.load(Ordering::SeqCst) {
        let _ = child.kill();
    }

    // ... rest of function ...
}
```

The callers in `bulk_import.rs` and any other spawn site must be updated to pass `&shutdown_requested`.

---

## Step 7: Verify Compilation

**Action:** Run `cargo check` from the project root. This validates that:

- All new dependencies resolve.
- `src/logging.rs` compiles without errors.
- `src/main.rs` borrow checker is satisfied with the new `AppState` fields and the `app.run()` closure.
- All `tracing::` macro invocations across all files are syntactically correct.
- The `resolve_log_dir()` function compiles on all target platforms (`cfg` blocks).

**No `cargo tauri build` is run** — the user will verify the release build.

---

## Complete File Change Summary

| # | File | Action |
|---|------|--------|
| 1 | `Cargo.toml` | **Modify** — add `tracing`, `tracing-subscriber`, `tracing-appender` |
| 2 | `src/logging.rs` | **Create** — `init_logging()`, `resolve_log_dir()`, `LogGuard` struct |
| 3 | `src/main.rs` | **Modify** — add `windows_subsystem` attr, add `pub mod logging`, add `LogGuard` + `AtomicBool` to `AppState`, switch to `build().run()` with event loop, convert `println!` → `tracing!` |
| 4 | `src/config.rs` | **Modify** — `println!` → `tracing::debug!` |
| 5 | `src/database/migrations.rs` | **Modify** — `println!` → `tracing::warn!` |
| 6 | `src/services/image_generation.rs` | **Modify** — convert `eprintln!`/`println!` → `tracing!`, add `shutdown_flag` parameter to Python spawn function |
| 7 | `src/services/backfill.rs` | **Modify** — `log_info!` → `tracing::info!` |
| 8 | `src/routes/bulk_import.rs` | **Modify** — convert `println!`/`eprintln!` → `tracing!`, pass shutdown flag to image generation |
| 9 | `src/routes/maintenance.rs` | **Modify** — `eprintln!` → `tracing::error!` |
| 10 | `src/readers/hus_reader.rs` | **Modify** — `eprintln!` → `tracing::warn!` |
| 11 | `src/readers/vp3_reader.rs` | **Modify** — `eprintln!` → `tracing::warn!` / `tracing::debug!` |
| 12 | `src/readers/jef_reader.rs` | **Modify** — remove per-stitch `println!` entirely |

**No frontend files are changed. No database migrations are needed. No Tauri configuration changes are needed.**

---

## Verification Checklist (Post-Implementation)

- [ ] `cargo check` passes with zero errors.
- [ ] User builds with `cargo tauri build` — `.exe` launches with no visible terminal window.
- [ ] After navigating the app and closing normally, a log file exists at the resolved log directory (e.g. `%APPDATA%/EmbroideryCatalogue/logs/app.YYYY-MM-DD`).
- [ ] Log file contains: startup message, bootstrap config dump, backfill results, and exit message.
- [ ] Running via `cargo tauri dev` still works — logs appear in the terminal (stdout layer active in debug builds).