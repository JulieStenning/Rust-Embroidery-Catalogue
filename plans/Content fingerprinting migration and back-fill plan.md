# Content Fingerprinting Migration & Back-Fill Plan

## Existing Foundation (Already Complete)

1. **Migration SQL** — `migrations/20260723000003_content_fingerprinting.up.sql` correctly adds `file_size_bytes INTEGER` and `file_hash_blake3 TEXT` columns + two indexes. The reversible `.down.sql` handles rollback.
2. **SQLx migration infrastructure** — `src/database/migrations.rs` uses `sqlx::migrate!("./migrations")` with a `_sqlx_migrations` tracking table. This automatically distinguishes new databases (full schema from scratch) from existing databases (incremental ALTER TABLE only).
3. **Rust model** — `src/database/models.rs` already has `file_size_bytes: Option<i64>` and `file_hash_blake3: Option<String>` on `Design`.
4. **`blake3 = "1.5"`** is already in `Cargo.toml`.

## What Needs to Be Built

### Step 1: Create the Content Fingerprinting Backfill Service

**File:** `src/services/fingerprint.rs` (new module)

**Design decisions:**
- Integrate as a **new optional action** inside the existing `UnifiedBackfillActions` struct (add a `fingerprinting: Option<FingerprintActionOptions>` field). This lets users trigger it manually from the Tagging Actions UI and also lets us run it silently at startup.
- Use **chunked iteration** matching the existing backfill pattern: query designs where `file_size_bytes IS NULL OR file_hash_blake3 IS NULL`, process in batches controlled by `commit_every`, and check the shared `STOP_REQUESTED` atomic flag between chunks.
- Read files with `std::fs::metadata` for file size (single system call, no content read needed), then only open the file for BLAKE3 hashing if the hash is also missing.
- **Missing file handling:** If `fs::metadata` returns `NotFound`, update the database row to set both `file_size_bytes` and `file_hash_blake3` to `-1` / `"<missing>"` sentinel values (or leave them NULL and log the path — sentinel values make the query more efficient since re-scans won't re-pick them). **Recommendation:** use `-1` for size and empty string `""` for hash to mark "checked but missing," and the selection query should filter these out: `WHERE file_size_bytes IS NULL OR (file_size_bytes = -1 AND file_hash_blake3 = '')`.
- **Pragmas:** Temporarily set `synchronous = OFF` and `journal_mode = WAL` (if not already WAL) during bulk update for speed, restoring them afterwards.

**Core function signature:**
```rust
pub async fn run_fingerprint_backfill(
    pool: &SqlitePool,
    commit_every: i64,   // e.g. 100
) -> Result<FingerprintSummary, String>
```

**Query for candidates:**
```sql
SELECT id, filepath
FROM designs
WHERE file_size_bytes IS NULL
   OR file_hash_blake3 IS NULL
   OR file_size_bytes = -1
ORDER BY id ASC
LIMIT ?
```

**Per-design processing:**
1. Call `std::fs::metadata(&filepath)` to get `len()`.
2. If missing → UPDATE `file_size_bytes = -1, file_hash_blake3 = ''`.
3. If hash is also needed → open with `BufReader`, stream through `blake3::Hasher::update_reader()` (memory-efficient for large files).
4. UPDATE with both values.

### Step 2: Register the New Module & Expose a Tauri Command

**File:** `src/services/mod.rs` — add `pub mod fingerprint;`

**File:** `src/routes/tagging_actions.rs` — add a new command:
```rust
#[tauri::command]
pub async fn run_fingerprint_backfill(
    state: State<'_, AppState>,
    commit_every: Option<i64>,
) -> Result<FingerprintSummary, String> {
    fingerprint::run_fingerprint_backfill(&state.db, commit_every.unwrap_or(100)).await
}
```

**File:** `src/main.rs` — register the command in `invoke_handler!`:
```rust
routes::tagging_actions::run_fingerprint_backfill,
```

Also add the stop command wiring so the existing `stop_unified_backfill` command resets the same `STOP_REQUESTED` flag that the fingerprint backfill checks.

### Step 3: Add Fingerprinting to the Unified Backfill Actions

**File:** `src/services/backfill.rs`

Add to `UnifiedBackfillActions`:
```rust
pub fingerprinting: Option<FingerprintActionOptions>,
```

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FingerprintActionOptions {
    pub enabled: Option<bool>,
}
```

In `run_unified_backfill()`, add a new processing block (after the color_counts block):
```rust
if let Some(fp_action) = actions.fingerprinting {
    if fp_action.enabled.unwrap_or(true) {
        actions_run.push("fingerprinting".to_string());
        let fp_summary = fingerprint::run_fingerprint_backfill(pool, commit_every).await?;
        processed += fp_summary.processed;
        errors += fp_summary.errors;
    }
}
```

### Step 4: Optional — Silent Startup Backfill

**File:** `src/main.rs`

After the existing migration block (~line 76), add:
```rust
// Trigger a lightweight backfill for orphan fingerprint data (hash + size)
// This is a fire-and-forget best-effort — errors are logged, not fatal.
let pool_clone = pool.clone();
tauri::async_runtime::spawn(async move {
    if let Err(err) = services::fingerprint::run_fingerprint_backfill(&pool_clone, 100).await {
        eprintln!("Startup fingerprint backfill error: {}", err);
    }
});
```

This spawns a background Tokio task that doesn't block startup. The UI becomes interactive immediately while hashing runs in the background.

### Step 5: Frontend Type Parity

**File:** `frontend/src/lib/types/` — add TypeScript interface:
```typescript
export interface FingerprintSummary {
  processed: number;
  errors: number;
  missing_files: number;
  stopped: boolean;
}
```

**File:** `frontend/src/lib/services/` — add the IPC wrapper in the relevant service module (e.g., `backfill.ts` or a new `fingerprint.ts`).

### Step 6: Error Logging & Missing File Audit

**File:** `src/services/fingerprint.rs`

- Log each missing file to `logs/backfill_errors.log` (reuse the existing `log_error()` helper from `backfill.rs`).
- Return `missing_files: i64` count in the summary so the frontend can surface it.
- Write to the same `logs/` directory already used by the backfill module.

---

## Idempotency Guarantees

| Scenario | Behavior |
|---|---|
| Migration re-run on already-upgraded DB | `ALTER TABLE ADD COLUMN IF NOT EXISTS` isn't supported by SQLite, but SQLx tracks applied migrations in `_sqlx_migrations`. It skips already-applied migrations — idempotent by design. |
| Backfill re-run after completion | Candidate query filters `WHERE file_size_bytes IS NULL OR file_hash_blake3 IS NULL OR file_size_bytes = -1`. Already-populated rows are excluded. |
| Backfill interrupted mid-run | Partial results are committed per-chunk. On restart, the candidate query re-selects only remaining NULL rows — picks up where it left off. |
| Missing files re-checked | Rows with `file_size_bytes = -1` won't be re-processed unless explicitly requested via a `force_recheck` option. |

## New Database vs. Existing Database

- **New databases:** Created via `migrations/20260503000000_initial.up.sql` which includes all columns from scratch. The fingerprinting columns are added in migration 003, but for a fresh DB, SQLx applies all migrations in order, so the columns exist from birth. The startup backfill query returns zero rows (no designs yet).
- **Existing databases:** Migration 003 runs the ALTER TABLE to add the two new columns. The startup backfill then finds all existing designs with NULL values and processes them.

---

## File Change Summary

| File | Action |
|---|---|
| `src/services/fingerprint.rs` | **Create** — new backfill module |
| `src/services/mod.rs` | **Edit** — add `pub mod fingerprint;` |
| `src/services/backfill.rs` | **Edit** — add `FingerprintActionOptions`, integrate into unified backfill |
| `src/routes/tagging_actions.rs` | **Edit** — add `run_fingerprint_backfill` command |
| `src/main.rs` | **Edit** — register command + optional startup spawn |
| `frontend/src/lib/types/` | **Edit** — add `FingerprintSummary` interface |
| `frontend/src/lib/services/` | **Edit** — add IPC wrapper |
