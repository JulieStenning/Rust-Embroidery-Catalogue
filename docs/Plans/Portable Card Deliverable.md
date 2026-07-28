# Portable SD Card Mode — Architectural Specification & Implementation Plan

Below is the complete step-by-step blueprint for adding Portable Mode alongside the existing Installed Mode, with open-source licensing compliance baked into the UI.

---

## 1. Rust Backend Path Resolution Strategy

### 1.1 New Module: `src/paths.rs`

A dedicated path-resolution module that centralises all filesystem layout decisions. This module exports a single struct `AppPaths` and an enum `ExecutionMode`.

#### ExecutionMode enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ExecutionMode {
    /// Running from an SD card / USB stick — ./data/ exists next to the exe.
    Portable,
    /// Standard OS install — data lives under %APPDATA% (or platform equivalent).
    Installed,
}
```

#### AppPaths struct
```rust
#[derive(Debug, Clone, Serialize)]
pub struct AppPaths {
    pub mode: ExecutionMode,
    pub data_root: PathBuf,              // e.g. D:\data\  or  C:\Users\...\AppData\Roaming\EmbroideryCatalogue\
    pub embroidery_designs_dir: PathBuf, // <data_root>/MachineEmbroideryDesigns/
    pub database_dir: PathBuf,           // <data_root>/Database/
    pub database_path: PathBuf,          // <data_root>/Database/catalogue.db
    pub thumbnail_cache_dir: PathBuf,    // <data_root>/thumbnails/ (or similar)
    pub log_dir: PathBuf,                // <data_root>/logs/
}
```

### 1.2 Detection Algorithm (called once at startup in `main.rs`)

```text
fn resolve_app_paths() -> AppPaths:
    1. Determine the directory containing the running executable.
       In Tauri v2, use `std::env::current_exe()` → `.parent()`.
    2. Check if `<exe_dir>/data/` exists as a directory.
       - If YES → ExecutionMode::Portable
         - data_root = <exe_dir>/data/
         - embroidery_designs_dir = <data_root>/MachineEmbroideryDesigns/
         - database_path = <data_root>/Database/catalogue.db
       - If NO  → ExecutionMode::Installed
         - On Windows: data_root = %APPDATA%/EmbroideryCatalogue/
           (resolved via `dirs_next::data_dir()` or `known_folder` API)
         - On macOS:   data_root = ~/Library/Application Support/EmbroideryCatalogue/
         - On Linux:   data_root = ~/.local/share/EmbroideryCatalogue/
         - embroidery_designs_dir = <data_root>/MachineEmbroideryDesigns/
         - database_path = <data_root>/Database/catalogue.db
    3. Create all required directories (data_root, embroidery_designs_dir, database_dir, thumbnail_cache_dir, log_dir)
       using `std::fs::create_dir_all()`.
    4. Return AppPaths.
```

### 1.3 Wiring into Existing Code

- **`src/main.rs` changes:**
  - Call `paths::resolve_app_paths()` before any database initialization.
  - Pass `AppPaths` into `AppState` as a new field: `pub paths: AppPaths`.
  - The `database_url` in `BootstrapConfig` is now derived from `AppPaths.database_path` rather than an env-var default. (Keep the `DATABASE_URL` env-var override as a fallback for development, but the resolved path takes priority.)
  - Add `AppPaths` to the `.manage()` call so commands can access it via `State<AppPaths>`.

- **`src/config.rs` changes:**
  - `BootstrapConfig::from_env()` gains a new method `from_app_paths(paths: &AppPaths)` that produces a `BootstrapConfig` with `database_url = format!("sqlite:{}", paths.database_path.display())`.
  - The existing `DATABASE_URL` env-var / `.env` logic is retained as a development convenience but the resolved `AppPaths.database_path` is the canonical source.

- **`src/database/connection.rs` changes:**
  - Remove `panic!()` calls; return `Result<SqlitePool, AppError>` instead. (This satisfies the "Zero Panics" rule.)
  - Accept `&AppPaths` as a parameter to derive the database URL.

- **`src/routes/settings.rs` changes:**
  - `derive_data_root_from_database_url()` is replaced with a direct read from `state.paths.data_root`.
  - `can_configure_data_root` becomes `true` only in Installed mode; `false` in Portable mode (since portable data sticks with the exe).
  - `app_mode` is populated from `state.paths.mode` (serialised as `"portable"` or `"installed"`).

### 1.4 New Tauri Command: `get_app_status`

```rust
#[tauri::command]
fn get_app_status(paths: State<'_, AppPaths>) -> AppStatus {
    AppStatus {
        execution_mode: paths.mode,      // "portable" | "installed"
        data_root: paths.data_root.to_string_lossy().to_string(),
        embroidery_dir: paths.embroidery_designs_dir.to_string_lossy().to_string(),
        database_path: paths.database_path.to_string_lossy().to_string(),
    }
}
```

Expose this in `invoke_handler` so the frontend can display mode/path information anywhere.

---

## 2. Database Migration & Relative Path Strategy

### 2.1 The Problem

When running Portable Mode from an SD card, the absolute path to `MachineEmbroideryDesigns` will differ on different computers (e.g., `E:\data\MachineEmbroideryDesigns\` vs `F:\data\MachineEmbroideryDesigns\`). Storing absolute paths in the database ties the catalogue to one machine.

### 2.2 Solution: Store Paths Relative to `data_root`

- All file paths on the `designs` table (and any other table referencing files) are stored **relative to `AppPaths.data_root`**.
- On read, the backend prepends `AppPaths.data_root` to reconstruct the absolute path.
- On import/write, the backend strips `AppPaths.data_root` before persisting.

#### Example
```
data_root        = E:\data\
absolute_path    = E:\data\MachineEmbroideryDesigns\my_design.jef
stored_path      = MachineEmbroideryDesigns\my_design.jef
reconstructed    = E:\data\ + MachineEmbroideryDesigns\my_design.jef → works on any drive
```

### 2.3 Implementation

- Add two utility functions to `src/paths.rs`:
  - `fn to_relative(absolute: &Path, root: &Path) -> Result<PathBuf, Error>`
  - `fn to_absolute(relative: &Path, root: &Path) -> PathBuf`
- Create a **database migration** (e.g., `migrations/20260728000004_relative_paths.up.sql`) that:
  - Adds a column `relative_file_path TEXT` to the `designs` table (nullable, then populated via backfill).
  - The backfill reads each row's absolute `file_path`, strips the current `data_root`, and writes it to `relative_file_path`.
  - Once backfilled, the old `file_path` column is dropped (or renamed to `file_path_absolute_deprecated`).
- Update all Rust query code (in `src/routes/designs.rs`, `src/routes/bulk_import.rs`, etc.) to reconstruct paths via `paths::to_absolute()` at read time.
- Add a new column `designs_root_relative` to the `settings` table (or compute it at startup) that records the `data_root` used when the catalogue was created. If the current `data_root` differs, warn the user but still try to resolve paths using the new root — this handles the SD-card-plugged-into-another-PC scenario.

### 2.4 Path Migration Workflow

```text
1. On startup after migration:
   - Read all designs with NULL relative_file_path.
   - For each, compute relative path by stripping current data_root from file_path.
   - Update relative_file_path column.
   - This is idempotent; safe to re-run.
2. After backfill:
   - All read paths use: to_absolute(relative_file_path, data_root).
   - All newly imported designs store the relative path from the start.
```

---

## 3. Svelte Frontend Adjustments

### 3.1 New TypeScript Interface in `src/lib/types/AppStatus.ts`

```ts
export interface AppStatus {
  execution_mode: "portable" | "installed";
  data_root: string;
  embroidery_dir: string;
  database_path: string;
}
```

### 3.2 New Service Method in `src/lib/api/commandAdapter.js`

```js
export async function getAppStatus() {
  return invoke("get_app_status");
}
```

### 3.3 Updated Views

#### @SettingsView.svelte
- Display a new **"Execution Mode"** banner at the top of the "Catalogue storage" section:
  - Portable mode: show an info card: "🧳 Portable Mode — all data is stored alongside the application on your removable drive."
  - Installed mode: show: "💻 Installed Mode — data is stored in your system application data folder."
- The `data_root` display already exists; ensure it reflects the correct value from `get_app_status()`.
- In Portable mode, disable the "Browse…" data-root picker (data root is fixed to `./data/`).

#### @App.svelte
- Optionally call `getAppStatus()` on startup and store the result in a Svelte writable store (`appStatusStore`) so any child component can access it without re-fetching.

#### @MainView.svelte
- Add a subtle status bar or footer indicator showing execution mode (e.g., a small USB icon with "Portable" tooltip). This is a passive indicator, not a blocking UI.

#### @DisclaimerView.svelte (Licensing — see Section 4)

#### @AboutView.svelte (Licensing — see Section 4)

---

## 4. Source Code Availability & Licensing UI

### 4.1 Requirements from AGPL-3.0 & GPL/LGPL Dependencies

- The AGPL-3.0 license requires that users who interact with the software remotely must be able to access the Corresponding Source. For a desktop app, the equivalent obligation is that users receive (or are clearly told where to get) the full source code.
- Third-party dependencies (pyembroidery-inspired parsers, etc.) are GPL/LGPL-licensed. Their licenses require attribution and a copy of (or link to) the license text and source code.

### 4.2 Changes to @DisclaimerView.svelte

The disclaimer already loads HTML from the backend (`get_disclaimer_text` which reads `DISCLAIMER.html`). Update this HTML to include:

- A prominent paragraph at the end:
  > **Open Source:** This application is free software licensed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). The full source code is available at: [https://github.com/juliestenning/rust-embroidery-catalogue](https://github.com/juliestenning/rust-embroidery-catalogue)
- A link to the Third-Party Notices page within the app (`#/about/document/third-party-notices`).

### 4.3 Changes to @AboutView.svelte

The About page already shows:
- An AGPL-3.0 licence section with a link to the licence text.

Add the following new sections or augment existing ones:

1. **"Source Code" section** (new, between "What this app is" and "Where data is stored"):
   ```html
   <div>
     <p class="ui-section-label font-semibold text-gray-850">Source Code</p>
     <p class="text-gray-600">
       The full source code for Embroidery Catalogue is publicly available on GitHub at:
       <a href="https://github.com/juliestenning/rust-embroidery-catalogue"
          class="text-indigo-600 hover:underline font-medium break-all"
          target="_blank" rel="noopener">
         github.com/juliestenning/rust-embroidery-catalogue
       </a>
     </p>
   </div>
   ```

2. **"Licence" section** (update existing):
   - Add a sentence: "This repository is licensed under **AGPL-3.0-or-later**. You can view the full licence text and access the complete source code as required by Section 13 of the AGPL."
   - The link to `#/about/document/licence` already exists.

3. **Ensure the "Third-Party Notices" document is always present** in the documents list. It already is (from `src/routes/about.rs` line 58-61), but verify the `THIRD_PARTY_NOTICES.html` file at the project root exists and contains proper attribution for all GPL/LGPL dependencies.

### 4.4 New/Updated Static Files

- **`disclaimer.html`** at project root: Add the open-source notice paragraph at the bottom (above the acceptance checkbox area).
- **`third_party_notices.html`** at project root: Ensure it lists:
  - pyembroidery (GPL/LGPL — name the specific license)
  - SQLx (MIT/Apache-2.0 — verify)
  - Tauri (MIT/Apache-2.0)
  - Svelte (MIT)
  - Tailwind CSS (MIT)
  - Any other bundled/static-linked dependencies with their full license texts or links to them.
- **`LICENCE`** file: Already present with AGPL-3.0 text. No changes needed but verify it's bundled in the Tauri `resources` config so it ships with the portable build.

### 4.5 Tauri `tauri.conf.json` Bundle Config Update

Ensure the portable bundle includes the licence files:
```json
"bundle": {
  "resources": {
    "LICENCE": "LICENCE",
    "third_party_notices.html": "third_party_notices.html",
    "templates/info/*.html": "templates/info/"
  }
}
```

---

## 5. Sequential File Execution Task List

Below is the ordered list of files to create or modify, grouped by phase.

### Phase A — Path Resolution Foundation (Rust)

| # | File | Action | Summary |
|---|------|--------|---------|
| A1 | `src/paths.rs` | **Create** | New module: `AppPaths`, `ExecutionMode`, `resolve_app_paths()`, `to_relative()`, `to_absolute()` |
| A2 | `src/main.rs` | **Modify** | Call `resolve_app_paths()` at startup; add `paths: AppPaths` to `AppState`; register `get_app_status` command; pass `AppPaths` to `BootstrapConfig` |
| A3 | `src/config.rs` | **Modify** | Add `BootstrapConfig::from_app_paths()`; keep `from_env()` as fallback; update `ensure_database_dir()` to use `AppPaths.database_dir` |
| A4 | `src/database/connection.rs` | **Modify** | Accept `&AppPaths`; remove `panic!()` calls → return `Result` |
| A5 | `src/routes/settings.rs` | **Modify** | Read `data_root`/`app_mode`/`can_configure_data_root` from `AppPaths` state; add `get_app_status` command (or in `paths.rs` as a separate command module) |

### Phase B — Database & Relative Paths

| # | File | Action | Summary |
|---|------|--------|---------|
| B1 | `migrations/20260728000004_relative_paths.up.sql` | **Create** | Add `relative_file_path TEXT` column; backfill SQL for existing rows |
| B2 | `migrations/20260728000004_relative_paths.down.sql` | **Create** | Reverse migration (restore `file_path` from `relative_file_path` if needed) |
| B3 | `src/models/mod.rs` | **Modify** | Add `relative_file_path` field to `Design` struct; update SQLx query macros |
| B4 | `src/routes/designs.rs` | **Modify** | Read: reconstruct absolute path via `paths::to_absolute()`; Write: strip root → store relative |
| B5 | `src/routes/bulk_import.rs` | **Modify** | Same read/write path adjustment |
| B6 | `src/routes/maintenance.rs` | **Modify** | Orphan scanning, backup paths — update to use relative paths |
| B7 | `src/services/` (any file reading designs) | **Modify** | Ensure all design file access goes through path resolution |

### Phase C — Licensing & UI Compliance

| # | File | Action | Summary |
|---|------|--------|---------|
| C1 | `disclaimer.html` | **Modify** | Add open-source notice & source code URL paragraph |
| C2 | `third_party_notices.html` | **Modify** | Audit and add complete dependency license attributions |
| C3 | `frontend/src/lib/DisclaimerView.svelte` | **Modify** | Ensure new disclaimer HTML renders correctly (no changes to logic needed if HTML is the only change) |
| C4 | `frontend/src/lib/views/AboutView.svelte` | **Modify** | Add "Source Code" section with GitHub link; enhance "Licence" section |
| C5 | `src-tauri/tauri.conf.json` | **Modify** | Add `resources` to `bundle` so licence files are included in the binary |

### Phase D — Frontend Status Display

| # | File | Action | Summary |
|---|------|--------|---------|
| D1 | `frontend/src/lib/types/AppStatus.ts` | **Create** | TypeScript interface for `AppStatus` |
| D2 | `frontend/src/lib/api/commandAdapter.js` | **Modify** | Add `getAppStatus()` service function |
| D3 | `frontend/src/lib/views/SettingsView.svelte` | **Modify** | Display execution mode banner; disable data-root picker in Portable mode; show correct paths |
| D4 | `frontend/src/lib/MainView.svelte` | **Modify** | Optional: add subtle mode indicator in footer/status bar |
| D5 | `frontend/src/App.svelte` | **Modify** | Optional: call `getAppStatus()` on mount and store in a Svelte store |

### Phase E — Verification

| # | File | Action | Summary |
|---|------|--------|---------|
| E1 | `src/paths.rs` | **Test** | Unit tests: `resolve_app_paths` with temporary dirs simulating `./data/` present and absent |
| E2 | `src/routes/settings.rs` | **Test** | Add/update tests for `data_root` derivation from `AppPaths` |
| E3 | — | **CLI** | Run `cargo check && cargo test` after each Rust file change |
| E4 | — | **CLI** | Run `npm run check` (or `npx svelte-check`) after frontend changes |

---

## 6. Summary of Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| Store file paths **relative to `data_root`** in SQLite | Enables SD-card portability across machines with different drive letters |
| Detect Portable mode by checking `./data/` next to the **executable**, not CWD | The executable location is stable; CWD can change when launched from shortcuts |
| `AppState` holds `AppPaths` as a managed state field | All Tauri commands automatically get access; no need for global statics |
| `can_configure_data_root` is `true` only in Installed mode | Portable mode's data root must stay fixed to `./data/` for the SD-card metaphor to work |
| Licensing info lives in both @DisclaimerView.svelte and @AboutView.svelte | Disclaimer is shown once before first use; About is permanent reference |
| Bundled licence files via `tauri.conf.json` resources | Ships with the portable binary so users always have access offline |

---

This plan covers all five deliverables requested: path resolution, database relative-path strategy, frontend UI adjustments, open-source compliance, and a sequential file checklist. Ready for review before implementation.