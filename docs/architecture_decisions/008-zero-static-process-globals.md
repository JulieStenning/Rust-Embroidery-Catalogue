# Architecture Decision Record 008: Zero Static Process Globals and Managed State Lifecycle

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-07

---

## Context and Problem Statement

Early iterations of background services (such as bulk folder scanning, AI vision backfills, and stitching tag detection) used file-level static singletons, e.g.:
- `static OnceLock<Mutex<BulkImportContextStore>>`
- `static AtomicBool BULK_IMPORT_STOP_REQUESTED`
- `static Mutex<Option<SqlitePool>> BULK_IMPORT_DB_POOL`

This pattern introduced severe architectural vulnerabilities:
1. **Test Pollution & Concurrency Hazards:** Parallel unit/integration tests shared static state, causing flaky tests unless annotated with `#[serial]`.
2. **Dangling Background Tasks:** Closing the app or swapping databases while a background task was active caused memory leaks or writes to invalid database handles.
3. **Windows File Lock Violations:** Database restore operations could not cleanly close active database connections held in static singletons.

---

## Decision Drivers

* **Encapsulated Lifecycle:** All state, cancellation tokens, database pool holders, and background coordinators must be owned by the application lifecycle container.
* **Safe Parallel Testing:** Tests must construct isolated state instances without cross-test leakage.
* **Deterministic Database Hot-Swapping:** Database restore and migration routines must be able to cleanly shut down and swap connection pools without leaving orphaned file handles open.

---

## Considered Options

1. **Continue Using Global Statics with `#[serial]` Tests:** Require serial test execution and manual resets between tests.
2. **Arc-wrapped Standalone Coordinators:** Pass ad-hoc coordinator references through each Tauri command.
3. **Tauri Managed `AppState` Container (Chosen):** Encapsulate all coordinators, tokens, and pool holders inside a unified `AppState` struct managed by Tauri's dependency injection (`tauri::State<AppState>`).

---

## Decision Outcome

**Option 3 (Tauri Managed `AppState` Container)** was chosen.

### Architectural Invariants:
1. **Absolute Ban on Static Globals:** No `static Mutex<...>`, `static OnceLock<...>`, or `static AtomicBool` in production route handlers or services.
2. **`AppState` Encapsulation:**
   - Background tasks are managed by `TaskCoordinator` and `BulkImportCoordinator` inside `AppState`.
   - SQLite connection pool is held within a swappable `PoolHolder` in `AppState`.
3. **Dependency Injection:** Every Tauri command requiring state receives `app_state: tauri::State<AppState>`.
4. **Safe Pool Teardown:** Restoring a database invokes `app_state.pool_holder.close_and_swap(...)`, closing all pool connections to allow file rename operations without Windows file-locking errors.
