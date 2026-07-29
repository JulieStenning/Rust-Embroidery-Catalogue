Here is the complete, updated architectural plan tailored specifically for your IDE coding agent. It incorporates the original path resolution, relative database paths, and licensing UI requirements, while seamlessly integrating the window suppression, file logging, and process lifecycle updates into a unified sequence.

---

# Portable SD Card Mode — Updated Architectural Specification & Implementation Plan

Below is the complete blueprint for adding Portable SD Card Mode alongside the existing Installed Mode, ensuring window suppression in release builds, file-based logging, clean process termination on app exit, and open-source licensing compliance.

---

## 1. Rust Backend Path Resolution, Logging & Lifecycle Strategy

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
    pub thumbnail_cache_dir: PathBuf,    // <data_root>/thumbnails/
    pub log_dir: PathBuf,                // <data_root>/logs/
}

```

### 1.2 Detection Algorithm (called once at startup in `main.rs`)

```text
fn resolve_app_paths() -> AppPaths:
    1. Determine the directory containing the running executable:
       Use `std::env::current_exe()` → `.parent()` (avoids reliance on working directories or external launcher scripts).
    2. Check if `<exe_dir>/data/` exists as a directory.
       - If YES → ExecutionMode::Portable
         - data_root = <exe_dir>/data/
         - embroidery_designs_dir = <data_root>/MachineEmbroideryDesigns/
         - database_path = <data_root>/Database/catalogue.db
         - log_dir = <data_root>/logs/
       - If NO  → ExecutionMode::Installed
         - On Windows: data_root = %APPDATA%/EmbroideryCatalogue/
         - On macOS:   data_root = ~/Library/Application Support/EmbroideryCatalogue/
         - On Linux:   data_root = ~/.local/share/EmbroideryCatalogue/
         - embroidery_designs_dir = <data_root>/MachineEmbroideryDesigns/
         - database_path = <data_root>/Database/catalogue.db
         - log_dir = <data_root>/logs/
    3. Create all required directories (data_root, embroidery_designs_dir, database_dir, thumbnail_cache_dir, log_dir)
       using `std::fs::create_dir_all()`.
    4. Return AppPaths.

```

### 1.3 Native Console Window Suppression & Logging Setup

* **Console Suppression:** Ensure `#![windows_subsystem = "windows"]` is set at the top of `src-tauri/src/main.rs` for release builds. This natively suppresses the terminal window on Windows when the production executable is launched directly, removing any dependency on external VBS scripts on end-user machines.
* **File Logging Pipeline:** Create `src/logging.rs` (or configure `tauri-plugin-log` / `tracing-appender`) initialized immediately after `paths::resolve_app_paths()`. All stdout/stderr and application tracing must dump to `AppPaths.log_dir` (e.g., `./data/logs/app.log` in Portable Mode) so errors can be inspected without a console window.

### 1.4 Application Exit Interception

* In `src/main.rs`, hook into Tauri’s event loop listening for `RunEvent::ExitRequested` or `WindowEvent::CloseRequested` (when the user clicks the **X** on @App.svelte).
* Ensure the event handler explicitly calls SQLite connection pool shutdowns (`SqlitePool::close()`) and sends termination signals to any running child/sidecar processes before process exit.

### 1.5 Wiring into Existing Code

* **`src/main.rs` changes:**
* Call `paths::resolve_app_paths()` before initializing logging or the database pool.
* Pass `AppPaths` into `AppState` as a new field: `pub paths: AppPaths`.
* The `database_url` in `BootstrapConfig` is derived from `AppPaths.database_path`.
* Add `AppPaths` to `.manage()` so commands can access it via `State<AppPaths>`.


* **`src/config.rs` changes:**
* Add `BootstrapConfig::from_app_paths(paths: &AppPaths)` producing `database_url = format!("sqlite:{}", paths.database_path.display())`.


* **`src/database/connection.rs` changes:**
* Return `Result<SqlitePool, AppError>` instead of panicking.
* Accept `&AppPaths` as a parameter to derive the database URL.


* **`src/routes/settings.rs` changes:**
* `derive_data_root_from_database_url()` is replaced with a direct read from `state.paths.data_root`.
* `can_configure_data_root` is `true` in Installed mode and `false` in Portable mode.
* `app_mode` is populated from `state.paths.mode`.



### 1.6 Tauri Command: `get_app_status`

Expose a Tauri command returning execution mode and path metadata so frontend views can consume status without re-fetching.

---

## 2. Database Migration & Relative Path Strategy

### 2.1 The Problem

When running Portable Mode from an SD card, the absolute path to `MachineEmbroideryDesigns` will change on different computers depending on the drive letter assigned by the OS (e.g., `E:\data\MachineEmbroideryDesigns\` vs `F:\data\MachineEmbroideryDesigns\`). Storing absolute paths in the database ties the catalogue to a single machine.

### 2.2 Solution: Store Paths Relative to `data_root`

* All file paths in the `designs` table (and related tables) are stored **relative to `AppPaths.data_root**`.
* On read, the backend prepends `AppPaths.data_root` to reconstruct the absolute path.
* On import/write, the backend strips `AppPaths.data_root` before persisting.

#### Example

```text
data_root        = E:\data\
absolute_path    = E:\data\MachineEmbroideryDesigns\my_design.jef
stored_path      = MachineEmbroideryDesigns\my_design.jef
reconstructed    = E:\data\ + MachineEmbroideryDesigns\my_design.jef → works on any drive

```

### 2.3 Implementation

* Add utility functions to `src/paths.rs`:
* `fn to_relative(absolute: &Path, root: &Path) -> Result<PathBuf, Error>`
* `fn to_absolute(relative: &Path, root: &Path) -> PathBuf`


* Create database migration `migrations/20260728000004_relative_paths.up.sql`:
* Add `relative_file_path TEXT` column to the `designs` table.
* Backfill by reading each row's absolute `file_path`, stripping current `data_root`, and writing to `relative_file_path`.


* Update Rust query logic in `src/routes/designs.rs`, `src/routes/bulk_import.rs`, and related services to reconstruct paths via `paths::to_absolute()` at read time.

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

### 3.2 Service Method in `src/lib/api/commandAdapter.js`

Add `getAppStatus()` wrapper calling the backend `get_app_status` Tauri command.

### 3.3 Updated Views

* **@SettingsView.svelte:**
* Display an "Execution Mode" banner at the top of the "Catalogue storage" section.
* Portable mode: display "🧳 Portable Mode — all data is stored alongside the application on your removable drive."
* Installed mode: display "💻 Installed Mode — data is stored in your system application data folder."
* Disable the "Browse…" data-root picker in Portable mode (data root is fixed to `./data/`).


* **@App.svelte:**
* Call `getAppStatus()` on startup and store in a Svelte writable store (`appStatusStore`).


* **@MainView.svelte:**
* Display a subtle status indicator showing current execution mode in the footer/status bar.


* **@DisclaimerView.svelte & @AboutView.svelte:** (See Section 4).

---

## 4. Source Code Availability & Licensing UI

### 4.1 AGPL-3.0 & GPL/LGPL Compliance Requirements

* Third-party dependencies (e.g., pyembroidery-inspired binary parsing) require clear attribution, copies/links of license texts, and source code availability instructions.

### 4.2 Changes to @DisclaimerView.svelte

Update the HTML rendered inside @DisclaimerView.svelte to include open-source attribution and source code repository links before the acceptance checkbox area.

### 4.3 Changes to @AboutView.svelte

* **Source Code Section:** Add a dedicated section with direct links to the source code repository.
* **Licence Section:** Update to clarify AGPL-3.0 compliance and provide direct navigation links to view third-party notices (@AboutDocumentView.svelte).

### 4.4 Static License Files & Tauri Configuration

* Update `disclaimer.html` and `third_party_notices.html` at project root.
* Ensure `src-tauri/tauri.conf.json` includes `LICENCE`, `third_party_notices.html`, and `templates/info/*.html` under `bundle.resources` so license documents are present in portable release builds.

---

## 5. Sequential File Execution Task List

### Phase A — Path Resolution, Logging & Lifecycle (Rust)

| # | File | Action | Summary |
| --- | --- | --- | --- |
| A1 | `src/paths.rs` | **Create** | Implement `AppPaths`, `ExecutionMode`, `resolve_app_paths()`, `to_relative()`, `to_absolute()`. |
| A2 | `src/logging.rs` | **Create** | Initialize file logging directed to `AppPaths.log_dir` (`./data/logs/`). |
| A3 | `src/main.rs` | **Modify** | Ensure `#![windows_subsystem = "windows"]` is active; invoke `resolve_app_paths()`; initialize file logger; add `paths` to `AppState`; hook `RunEvent::ExitRequested` to cleanly terminate pools/processes; register `get_app_status`. |
| A4 | `src/config.rs` | **Modify** | Add `BootstrapConfig::from_app_paths()`; update `ensure_database_dir()` to use `AppPaths.database_dir`. |
| A5 | `src/database/connection.rs` | **Modify** | Accept `&AppPaths`; convert panics to `Result`. |
| A6 | `src/routes/settings.rs` | **Modify** | Derive `data_root`/`app_mode`/`can_configure_data_root` from `AppPaths` state. |

### Phase B — Database & Relative Paths

| # | File | Action | Summary |
| --- | --- | --- | --- |
| B1 | `migrations/20260728000004_relative_paths.up.sql` | **Create** | Add `relative_file_path TEXT` column and backfill SQL logic. |
| B2 | `migrations/20260728000004_relative_paths.down.sql` | **Create** | Reverse migration script. |
| B3 | `src/models/mod.rs` | **Modify** | Update `Design` model struct with `relative_file_path`. |
| B4 | `src/routes/designs.rs` | **Modify** | Update read/write logic using `paths::to_absolute()` and `paths::to_relative()`. |
| B5 | `src/routes/bulk_import.rs` | **Modify** | Update import handlers for relative path storage. |
| B6 | `src/routes/maintenance.rs` | **Modify** | Update orphan scanning and database backups to resolve relative paths. |

### Phase C — Licensing & Static Assets

| # | File | Action | Summary |
| --- | --- | --- | --- |
| C1 | `disclaimer.html` | **Modify** | Append open-source disclosure and repository link. |
| C2 | `third_party_notices.html` | **Modify** | Audit and include full attributions for third-party libraries. |
| C3 | `src-tauri/tauri.conf.json` | **Modify** | Configure `bundle.resources` to bundle license files into release executables. |

### Phase D — Frontend Integration (Svelte)

| # | File | Action | Summary |
| --- | --- | --- | --- |
| D1 | `src/lib/types/AppStatus.ts` | **Create** | TypeScript interface for `AppStatus`. |
| D2 | `src/lib/api/commandAdapter.js` | **Modify** | Export `getAppStatus()` invoking `get_app_status`. |
| D3 | @App.svelte | **Modify** | Fetch `getAppStatus()` on mount and store in global state. |
| D4 | @SettingsView.svelte | **Modify** | Display Portable vs Installed mode banner; lock root directory selection in Portable mode. |
| D5 | @AboutView.svelte | **Modify** | Add source code repository section and update license links. |
| D6 | @DisclaimerView.svelte | **Modify** | Render updated disclaimer content containing source link disclosures. |
| D7 | @MainView.svelte | **Modify** | Add execution mode indicator in the status footer. |

### Phase E — Verification & Verification Build

| # | Action | Summary |
| --- | --- | --- |
| E1 | **Unit Testing** | Run Rust tests covering path detection (`./data/` present vs absent) and relative path conversions. |
| E2 | **Frontend Checks** | Execute `npm run check` / `svelte-check` to confirm Svelte prop types. |
| E3 | **Release Build Test** | Run `cargo tauri build` and verify that double-clicking the resulting release binary launches cleanly without a console window, writes logs to `./data/logs/`, handles relative SD card paths, and closes cleanly when clicking the **X**. |

---

## 6. Architecture Summary

| Aspect | Architectural Decision |
| --- | --- |
| **Executable Path Stability** | Detect Portable mode by checking `<exe_dir>/data/` derived via `std::env::current_exe()`. |
| **Window Suppression** | Use Rust's native `#![windows_subsystem = "windows"]` in release mode rather than external `.vbs` scripts. |
| **Uncaptured Output** | Initialize file logging (`./data/logs/app.log`) at startup as soon as paths resolve. |
| **SD Card Drive Letters** | Persist paths relative to `data_root` in SQLite; resolve dynamically at runtime. |
| **App Termination** | Intercept `RunEvent::ExitRequested` to close SQLite connection pools and stop background processes when clicking **X**. |
| **User Interface Guidance** | Explicitly disable manual data-root pickers in @SettingsView.svelte when in Portable Mode. |


## Plan: Database Delivery & Initialisation Strategy

Based on my analysis of the current codebase, here is the concrete implementation plan:

---

### 1. Rename All Hardcoded Database References (`catalogue.db` → `EmbroideryCatalogue.db`)

**Files to update:**

| File | Current value | New value |
|------|-------------|-----------|
| `src/paths.rs` ln 64 | `database_dir.join("catalogue.db")` | `database_dir.join("EmbroideryCatalogue.db")` |
| `src/paths.rs` ln 176 | test assertion: `"catalogue.db"` | `"EmbroideryCatalogue.db"` |
| `src/config.rs` ln 4 | `DEFAULT_DATABASE_URL = "sqlite:data/database/catalogue.db"` | `"sqlite:data/database/EmbroideryCatalogue.db"` |

The `catalogue.db` references in `src/routes/settings.rs` and `src/routes/maintenance.rs` are only in test assertions for `strip_sqlite_prefix()` and use example paths like `"tmp/catalogue.db"` — these are generic test names and do **not** need changing.

---

### 2. Seed Database Bundling Strategy

#### 2a. Create `src-tauri/resources/` directory and seed DB

- Create `src-tauri/resources/EmbroideryCatalogue.db` — a pre-migrated SQLite database containing the full schema and any seed data (e.g., default designers, sources, hoops, `settings` defaults).
- The `resources/` directory is the standard Tauri location for bundled assets that get embedded in the release binary.

#### 2b. Update `src-tauri/tauri.conf.json`

Add under `bundle`:
```json
"resources": [
  "resources/EmbroideryCatalogue.db"
]
```
This tells Tauri's bundler to include the seed database in the release package. At runtime, the file can be accessed via `tauri::path::BaseDirectory::Resource`.

---

### 3. Update `resolve_app_paths()` in `src/paths.rs`

The function needs two changes:

**a. Rename the database filename** (line 64):
```rust
let database_path = database_dir.join("EmbroideryCatalogue.db");
```

**b. Add seed-DB copy logic (NEW):**
After computing `database_path`, before returning `AppPaths`:
```
1. If database_path does NOT exist:
   a. Try to locate the bundled resource `resources/EmbroideryCatalogue.db`.
   b. In Tauri v2, use `tauri::path::BaseDirectory::Resource` to resolve the resource path.
      Since `resolve_app_paths()` is called BEFORE the Tauri app builder is set up,
      we use `std::env::current_exe()` to derive the exe dir, then look for the resource
      relative to the exe in dev mode, or use Tauri's resource resolution if available.
   c. Copy the bundled DB to `<data_root>/Database/EmbroideryCatalogue.db`.
   d. Log that a fresh database was seeded from the bundle.
```

**Important design decision:** Since `resolve_app_paths()` runs before Tauri's `Builder::build()`, the resource path resolution needs to work without a Tauri `AppHandle`. The standard approach in Tauri v2 is:

- In **dev mode** (`cargo tauri dev`): The resource lives at `src-tauri/resources/EmbroideryCatalogue.db`. We can detect this by looking for it relative to the exe or by using `env!("CARGO_MANIFEST_DIR")` which points to `src-tauri/`.
- In **release mode**: The resource is bundled alongside the exe. We can check `<exe_dir>/resources/EmbroideryCatalogue.db` (typical Tauri bundle layout), or use `include_bytes!()` at compile time and write the bytes out at runtime.

**Proposed approach:** Use `include_bytes!()` to embed the seed DB at compile time, then write it to the target path at runtime if missing. This avoids any path-resolution fragility and works identically in dev and release.

```rust
const SEED_DB_BYTES: &[u8] = include_bytes!("../../resources/EmbroideryCatalogue.db");

// Inside resolve_app_paths(), after creating database_dir:
if !database_path.exists() {
    if let Err(e) = std::fs::write(&database_path, SEED_DB_BYTES) {
        tracing::error!("Failed to seed EmbroideryCatalogue.db from bundle: {}", e);
    } else {
        tracing::info!("Seeded fresh EmbroideryCatalogue.db from bundled resource");
    }
}
```

Note: The `resources/` directory should be at the project root level (not inside `src-tauri/`) since `include_bytes!` is relative to the crate root (`src/`). Actually, let me reconsider — `include_bytes!` in a binary crate is relative to the file that contains the `main()` function, i.e., `src/main.rs`. So `include_bytes!("../resources/EmbroideryCatalogue.db")` from `src/paths.rs` would work if `resources/` is at the project root. But for the Tauri bundler to pick it up, `resources` should be under `src-tauri/`.

**Revised approach:** Put `EmbroideryCatalogue.db` under `src-tauri/resources/`. Then in `tauri.conf.json`, use relative paths from the `src-tauri/` directory. For `include_bytes!()` in `src/paths.rs`, the path would be `include_bytes!("../src-tauri/resources/EmbroideryCatalogue.db")`. This is slightly awkward but works. Alternatively, we can use `env!("CARGO_MANIFEST_DIR")` + `resources/EmbroideryCatalogue.db` in the copy logic at runtime rather than `include_bytes!`.

**Even simpler:** Use a runtime file-copy approach with `CARGO_MANIFEST_DIR` at build time:

In `src/paths.rs`:
```rust
fn copy_seed_database_if_missing(database_path: &Path) {
    if database_path.exists() {
        return;
    }
    // In dev mode, the seed DB lives at src-tauri/resources/
    // In release mode, Tauri bundles it next to the exe in resources/
    let seed_candidates = [
        std::env::current_exe().ok().and_then(|exe| exe.parent().map(|p| p.join("resources/EmbroideryCatalogue.db"))),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/EmbroideryCatalogue.db")),
    ];
    // ... try each candidate, copy to database_path
}
```

**Recommended final approach:** Use `include_bytes!` for reliability. Place `EmbroideryCatalogue.db` at the project root level in a `resources/` folder. The `include_bytes!` call from `src/paths.rs` resolves relative to `src/`, so: `include_bytes!("../resources/EmbroideryCatalogue.db")`. For Tauri bundling, use the `resources` bundle config pointing to the same file. This way it works identically in dev and release.

---

### 4. Frontend: SettingsView.svelte

The `data_root`, `log_folder`, and `app_mode` fields are already wired up via `get_settings_view_model()` which reads from `state.paths` (Phase A6 done). The settings view already displays `settingsDataRoot` and `settingsAppMode` in the UI.

**Additional change needed:**
- The SettingsView "Catalogue storage" section should display the **database filename** prominently. Currently it shows `data_root` but not the `.db` filename. We should add a line like:
  - `Database: <data_root>/Database/EmbroideryCatalogue.db`
- The `database_path` is already available from `get_app_status()` (but not from `get_settings_view_model()`). We can either:
  a. Add `database_path` to `SettingsViewModel` in the Rust backend, OR
  b. Call `getAppStatus()` separately in SettingsView and display `status.database_path`

The simpler approach is Option A — add `database_path: String` to `SettingsViewModel` since the settings route already has access to `state.paths.database_path`.

---

### 5. Summary of All File Changes

| # | File | Action |
|---|------|--------|
| 1 | Create `resources/EmbroideryCatalogue.db` | Pre-migrated seed database (schema + seed data) |
| 2 | `src/paths.rs` | Change `"catalogue.db"` → `"EmbroideryCatalogue.db"` (2 occurrences) |
| 3 | `src/paths.rs` | Add `include_bytes!` + copy-on-first-run logic in `resolve_app_paths()` |
| 4 | `src/config.rs` | Change `DEFAULT_DATABASE_URL` from `catalogue.db` → `EmbroideryCatalogue.db` |
| 5 | `src-tauri/tauri.conf.json` | Add `"resources": ["../resources/EmbroideryCatalogue.db"]` under `bundle` |
| 6 | `src/routes/settings.rs` | Add `database_path: String` to `SettingsViewModel` and populate from `state.paths.database_path` |
| 7 | `frontend/src/lib/views/SettingsView.svelte` | Display `settingsDatabasePath` in the Catalogue storage section |

---

### 6. `SettingsViewModel` Enhancement Detail

**Rust side** (`src/routes/settings.rs`):
```rust
pub struct SettingsViewModel {
    // ... existing fields ...
    pub database_path: String,  // NEW
}

// In get_settings_view_model():
let database_path = state.paths.database_path.to_string_lossy().to_string();

Ok(SettingsViewModel {
    // ... existing ...
    database_path,  // NEW
})
```

**Frontend side** (`SettingsView.svelte`):
```svelte
<script>
  let settingsDatabasePath = $state("");
  // ...
  function applySettingsModel(model = {}) {
    // ... existing ...
    settingsDatabasePath = String(model?.database_path || "");
  }
</script>

<!-- In the Catalogue storage section: -->
<p>Database location: {settingsDatabasePath}</p>
```

---

### 7. Seed Database Contents

The seed `EmbroideryCatalogue.db` should contain:
- Full schema from all migrations (tables: `designs`, `designers`, `sources`, `hoops`, `tags`, `design_tags`, `project_designs`, `projects`, `settings`, `disclaimer_accepted`)
- Default `settings` rows with blank values (so `UPSERT` works on first save)
- Empty `disclaimer_accepted` table (user must accept on first run)
- Optional: default hoop sizes (4x4, 5x7, 6x10 in mm)

This DB needs to be generated once (e.g., by running the app once in dev mode with an empty `data/` directory, then copying the resulting `catalogue.db` → renaming to `EmbroideryCatalogue.db`).

---

