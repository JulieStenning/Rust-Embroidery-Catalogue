# Architecture, Safety & System Boundaries

## 🛠️ Application Architecture & Tech Stack

- **Backend:** Rust (Idiomatic, clean, explicitly typed)
- **Desktop Framework:** Tauri (v2)
- **Frontend:** Svelte (v5) / TypeScript
- **Database:** Local SQLite database for metadata, tags, and file references
- **Core Logic:** Interfacing with binary embroidery file formats (reading metadata, stitches, and properties from formats like `.jef`, `.pes`, `.hus`, `.vp3`, etc., migrating logic inspired by `pyembroidery`).
- **AI Integration (Optional):** Google Gemini API for Gemini Vision (vision analysis on rendered thumbnails) to handle automated metadata/tag suggestions (tagging relies on offline File & Folder rules and online Gemini Vision tagging).

---

## 🧭 Core Philosophy & Constraints

- **Local & Offline First:** The app must run entirely locally. Original embroidery files must **NEVER** be moved, renamed, modified, or altered. The app only reads them to extract metadata and cache generated thumbnail previews locally.
- **Performance:** Reading binary stitch files and rendering/caching previews efficiently in Rust is a critical priority. Keep the UI responsive.
- **Separation of Concerns:** Maintain a clean architectural boundary between the Rust backend and the Svelte frontend.

---

## ⚠️ Core System & Failure State Lessons

### "Missing/Invalid file" is a state with side-effects, not just a file check

**Treat "the file is missing or invalid" as a system state that disables or redirects EVERY side-effect touching that path — not merely the main file operation.**

When a configured resource (database, data root, seed asset) is absent or invalid:

1. **Audit every side effect** that touches the stale/invalid path — folder creation, log writes, cache/thumbnail generation, temp files, background tasks — and gate ALL of them on the same single recovery/invalid-state decision. (Example: the database-recovery flow correctly stopped seeding the DB when missing, but still re-created empty `Database/`, `MachineEmbroideryDesigns/` and `logs/` at the stale root via `create_dir_all` and `logging::init_logging`. Each had to be gated separately).
2. **Write the "must NOT happen" test explicitly.** Add negative assertions (e.g. "recovery mode must NOT create dirs at the stale root") alongside positive ones.
3. **Enumerate the full state space before designing the flow.** _exists-valid_, _missing_, and _exists-but-invalid/corrupt_ are distinct states with different user paths.
4. **Order writes so nothing is persisted until a valid decision is made.** For replacement of an invalid file, prefer **rename-aside** (`<file>.corrupt-<timestamp>`) over delete — it is never safe to destroy the only copy of user data.
5. **Single source of truth for the invalid state.** Compute the recovery/invalid condition once (e.g. `paths::database_recovery_mode(&AppPaths)`) and reference it everywhere — startup, logging, background init, UI gating.

---

## 🧱 Core Architectural Boundaries

### 1. Database & Queries

- Keep all SQLite queries **strictly inside the Rust backend**.
- Never expose raw SQL or database connections to the frontend. Expose data to Svelte only via high-level, intentional Tauri commands.
- Manage the SQLite connection via Tauri's native managed state (`tauri::State`). Do not spin up or open separate database instances per command.

### 2. Tauri IPC Bridge

- All backend functions exposed to the frontend must use Tauri commands returning a `Result<T, E>`.
- The error type `E` must be a custom, descriptive, and serializable enum (e.g., using `thiserror` and `serde::Serialize`) so the frontend receives explicit error strings instead of generic panics.
- **CamelCase Invoke Keys (CRITICAL):** Tauri v2 automatically maps camelCase JavaScript invoke keys to snake_case Rust command parameters. **All `invoke()` payload keys MUST use camelCase** matching the Rust parameter names. Passing snake_case keys causes Tauri to silently drop them (parameters arrive as `None` with **NO error**).
- **Adapter tests MUST assert exact camelCase keys:** Every `commandAdapter` test that mocks `invoke()` MUST assert the exact camelCase payload keys that Tauri expects to exchange with the Rust command.
- **Diagnose IPC failures first:** If a Rust command logs `Option<T>` parameters as `None` even though the frontend set them, check the invoke key casing in the adapter before anything else.
- **`AppHandle::path()` needs the `Manager` trait:** Tauri v2's `PathResolver` (used for `app_handle.path().document_dir()` etc.) comes from `use tauri::Manager;`. Prefer `app_handle.path().document_dir()` over hand-rolled env-var Documents resolution.

### 3. Frontend Isolation

- Svelte components must **not** call `invoke()` directly.
- Abstract all Tauri IPC calls into dedicated TypeScript service modules under `src/lib/services/` (e.g., `src/lib/services/db.ts`, `src/lib/services/parser.ts`).

### 4. Catalogue Data Root Persistence & Environment Isolation

- **The catalogue data root is persisted ONLY to the bootstrap `config.json`** via `paths::write_bootstrap_data_root` / the `set_configured_data_root` Tauri command — **never** to the SQLite settings table.
- `save_settings_view_model_inner` deliberately ignores `request.data_root` (the `let _ = request.data_root;` line). The SQLite database file lives _under_ the data root, so the DB cannot store its own location.
- After a data-root change is persisted, the app **must restart** for the running backend to relocate.
- **Dev vs Installed Mode Config Isolation (CRITICAL):**
  - In `ExecutionMode::Dev`, the active data root is strictly local to the repository/debug harness (`paths.data_root`, e.g. `dev_data/`).
  - Development mode must **NEVER** read or write the global `%APPDATA%\EmbroideryCatalogue\config.json` used by the installed release on the same machine.
  - Doing so creates cross-environment pollution: testing recovery or restore in Dev mode would resolve or overwrite the live installed catalogue path (e.g. `F:\Database`).
  - All status and persistence commands (`database_status_from_paths`, `set_configured_data_root`) must branch on `paths.mode`: `ExecutionMode::Dev` reports and isolates to `paths.data_root`, keeping developer environments completely segregated from live user installations.
- **Test / e2e data-root override (`EMBROIDERY_DATA_ROOT`) — debug builds only:** An absolute `EMBROIDERY_DATA_ROOT` redirects the Dev-mode data root for Playwright harnesses. It is honoured **only** inside `resolve_paths_from_exe_dir` and **only** under `cfg!(debug_assertions)`.
- **Post-Commit Cleanup in Storage Migration:** After the bootstrap `config.json` is written, migration has succeeded. Post-migration tidy-up (renaming old data root) is best-effort. Filesystem roots (e.g. `F:\`) have no parent directory (`path.parent().is_none()`); fall back to writing a `storage location moved.txt` marker rather than failing the migration.
- **Expired Backend Context Tokens (TTL):** Backend-minted tokens (such as the 15-minute `context_token` from bulk import precheck) can expire while the user explores other pages. The frontend must detect the expired/unknown token response and automatically re-run precheck from preserved selections to mint a fresh token before retrying.

### 5. Tagging & Verification Separation (Image Tags vs Stitching Tags)

- **Semantic vs Technical Isolation:**
  - **Image Tags (Tab 1 - Tagging & Categorisation):** Semantic subject-matter tags generated via offline file/folder rules (`path_rule`) or online Gemini Vision (`ai_vision`). Governed strictly by `designs.image_tags_verified`. Image tagging passes must **never** read, write, clear, or modify `stitching_tags_verified` or tags in the `stitching` group.
  - **Stitching Tags (Tab 2 - Maintenance & File Processing):** Technical density/fill tags generated via deterministic binary stitch parsing (`stitch_identifier`). Governed strictly by `designs.stitching_tags_verified`. Stitching operations must **never** read, write, clear, or modify `image_tags_verified` or tags in the image tag pool.
- **Verification Independence:** `image_tags_verified` and `stitching_tags_verified` are strictly separate flags in SQLite and the domain model. Verifying or resetting one category must never cross-pollute or clear the other.
- **Verification Invariant on Tag Changes:** If any image tags are added to or removed from a design (e.g. automated tagging passes, single tag removal), `image_tags_verified` MUST be set to `0` (unverified). Similarly, if any stitching tags are added to or removed from a design (e.g. stitching detection/backfill, stitching tag clear, single tag removal), `stitching_tags_verified` MUST be set to `0` (unverified).
- **UI Responsibility Boundary:**
  - Tab 1 ("Tagging & Categorisation") is strictly for Image / Subject Tagging (Steps 1–3) with a callout link to Tab 2.
  - Tab 2 ("Maintenance & File Processing") is the home for all offline technical binary calculations: Stitching tag detection, preview image generation, thread colour/stitch counts recalculation, and hoop dimension recalculation.

### 6. Zero Static Process Globals & Managed State

- Absolute ban on standalone `static Mutex<...>`, `static OnceLock<...>`, and `static AtomicBool` in routes and services.
- All background task managers, cancellation tokens, coordinators (`TaskCoordinator`), and session context caches must be encapsulated within `AppState` and injected via Tauri's managed state (`tauri::State<AppState>`).
- This guarantees clean lifecycle management, allows parallel test execution without cross-test state leakage, and prevents dangling background tasks during app shutdown or database hot-swapping.

### 7. Structured IPC Error Enum (`IpcError`)

- All backend functions exposed across the Tauri IPC bridge must return `Result<T, IpcError>` using the centralized `IpcError` enum in `src/error.rs` (which derives `thiserror::Error` and `serde::Serialize`), rather than converting errors to raw strings via `.map_err(|e| e.to_string())`.
- `IpcError` in `src/error.rs` must be mirrored in TypeScript under `frontend/src/lib/types/errors.ts`.

### 8. Master Design vs Stitch File Pairing & Lifecycle (ADR 006)

- **Relational Pairing:** Stitch files (`.PES`, `.JEF`, `.VP3`, `.DST`, etc.) are the primary rendered entity in the `designs` table. Working master formats (`.ART`, `.EMB`, `.JAN`, `.BE`, etc.) are linked relationally via `master_design_file_id`.
- **Asynchronous Stem Matching:** When importing a stitch design or master format whose counterpart already exists in the catalogue, the system automatically detects and links them by relative path stem without creating duplicate designs.
- **Independent Launch Actions:** Backend provides dedicated IPC launch commands for opening the editable master design in desktop digitizing suites (`open_master_design_file`) versus inspecting the stitch file (`open_design_file`).

### 9. Library Drift Synchronization Boundaries (ADR 007)

- **Two-Way Drift Separation:**
  - **Disk $\rightarrow$ DB (Unmatched Files):** Physical files on disk missing from SQLite are reconciled via `@UnmatchedFilesReconciler.svelte` located in **Bulk Import** (onboarding flow) and **Orphaned Files** (maintenance flow).
  - **DB $\rightarrow$ Disk (Orphaned Records):** SQLite records pointing to missing disk files are reconciled via `@OrphansView.svelte` for safe metadata removal.
- **Disaster Recovery Isolation:** Drift reconciliation must **never** be mixed into Database Recovery or Backup Restore workflows.

### 10. Cancellation vs. Stopping Semantics (ADR 011)

- **Cancel (Atomic Rollback):**
  - Reserved strictly for operations where in-flight or uncommitted work is **completely rolled back or discarded**, leaving zero partial state in SQLite or on disk.
  - Examples: Modal dismissal without saving (@DeleteDesignsModal.svelte, @QuickAddEntityModal.svelte), setup wizard abort, storage migration cancellation (`cancel_catalogue_storage_migration`), database backup cancellation (`request_cancel_backup` cleans up partial archive).
- **Stop (Progressive Halt):**
  - Reserved strictly for **halting iterative or batch background processes mid-flight** where work completed so far is **strictly committed and retained**.
  - Examples: Bulk importing designs (@ImportView.svelte), unified backfill / stitch detection (@BatchOperationsView.svelte), unmatched files reconciliation (@UnmatchedFilesReconciler.svelte), design file syncing / restore (@RestoreProgressPanel.svelte).
  - Progressive button states must display `"Stop"` -> `"Stopping..."` while disabling the button until the background worker yields.
  - Backend flags and commands must follow `*_stop_requested` and `request_stop_*`.
