# Plan: Tauri v2 + Svelte + SQLx — Disclaimer on First Run

**Created:** 2026-05-07  
**Status:** Planning  

---

## 1. Overview

Convert the Python FastAPI + Jinja2 web application to a **Tauri v2** desktop application with a **Svelte** frontend, using **SQLx** for database access. The first deliverable is the **disclaimer-on-first-run** flow, where the user must accept a disclaimer before accessing the main application.

### Key technology decisions

| Concern                | Choice                          | Rationale                                                                          |
|------------------------|---------------------------------|------------------------------------------------------------------------------------|
| Desktop framework      | Tauri v2                        | Preferred by user for distribution; single .exe bundle on Windows                  |
| Frontend               | Svelte (bundled via Vite)       | Preferred by user; Tauri bundles frontend assets into the binary at build time     |
| Rust↔frontend IPC      | Tauri Commands + `invoke()` API | Standard Tauri pattern; no web server needed                                       |
| Database ORM           | SQLx (replacing Diesel)         | User preference; single `SqliteConnection` in Tauri State (no connection pool)     |
| Disclaimer text        | Bundled via `include_str!()`    | Frontend reads it via a Tauri command; no external HTML files                     |
| Async runtime          | Tokio                           | Required by SQLx and Tauri v2                                                      |
| CSS framework          | Tailwind CSS (CDN)              | Continue using the same CDN approach from the Python app's `base.html`             |

---

## 2. Scope

This plan covers **four interrelated workstreams** delivered as a single atomic change:

1.  **Replace Diesel with SQLx** — rewrite all existing database access code.
2.  **Add Tauri v2** — configure the desktop application framework, build system, and window.
3.  **Create Svelte frontend** — the web UI that runs inside Tauri's webview.
4.  **Wire up the disclaimer flow** — Tauri Commands backed by SQLx, called from Svelte.

---

## 3. Detailed File Plan

### 3.1 Cargo.toml (modify)

```toml
# Remove:
# - diesel (and its features)
# - dotenvy
# - vcpkg

# Add:
# - tauri = { version = "2", features = [] }
# - tauri-build = { version = "2", features = [] }  (build-dependency)
# - sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
# - tokio = { version = "1", features = ["full"] }
# - serde = { version = "1.0", features = ["derive"] }
# - serde_json = "1.0"

# Keep:
# - image, binrw, imageproc (embroidery readers + rendering)
```

Add a `[lib]` section so Tauri can find the app library crate, and a `[[bin]]` pointing to the new main entry point (which will be `src/main.rs` rewritten).

Add a `[build-dependencies]` section for `tauri-build`.

### 3.2 build.rs (create)

```rust
fn main() {
    tauri_build::build()
}
```

### 3.3 src-tauri/ directory (create)

```
src-tauri/
├── tauri.conf.json
├── capabilities/
│   └── default.json
├── icons/
│   ├── icon.png
│   ├── icon.ico
│   └── 32x32.png
```

**tauri.conf.json key settings:**
- `productName`: "Embroidery Catalogue"
- `identifier`: "com.embroidery-catalogue.app"
- `build.frontendDist`: "../frontend/dist"
- `build.devUrl`: "http://localhost:5173" (Vite dev server)
- `app.windows[0].title`: "Embroidery Catalogue"
- `app.windows[0].width`: 1280, `height`: 800
- `app.security.csp`: appropriate Content Security Policy for Tailwind CDN + Tauri IPC

**capabilities/default.json:** Grant permissions for `core:default` (which includes the invoke IPC mechanism).

### 3.4 Delete files

| File                | Reason                                          |
|---------------------|-------------------------------------------------|
| `diesel.toml`       | Diesel is being removed                         |
| `src/schema.rs`     | Diesel-generated; replaced by SQLx inline macros|
| `src/models_db.rs`  | Diesel models; replaced by SQLx `FromRow` structs|

### 3.5 src/main.rs (rewrite)

Convert from the current CLI tool into a Tauri application entry point.

```rust
// Pseudo-structure — the full implementation will:
// 1. Define an AppState struct holding:
//    - db: Mutex<SqliteConnection>
//    - disclaimer_text: String  (from include_str!("../DISCLAIMER.html"))
// 2. Register Tauri commands:
//    - check_disclaimer -> bool
//    - accept_disclaimer -> ()
//    - get_disclaimer_text -> String
// 3. On setup: load .env, establish SQLx connection, run migrations
// 4. Call tauri::Builder::default() ... .run()
```

**Re-export existing modules** so the Tauri commands can use `disclaimer`, `settings`, and `database` modules.

### 3.6 src/database/mod.rs (modify)

Continue exporting: `pub mod connection; pub mod models; pub mod schema; pub mod migrations;`

### 3.7 src/database/connection.rs (rewrite)

```rust
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqliteConnection;
use std::env;

pub async fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnectOptions::new()
        .filename(&database_url)
        .create_if_missing(true)
        .connect()
        .await
        .expect("Failed to connect to database")
}
```

### 3.8 src/database/models.rs (rewrite)

SQLx-compatible structs using `#[derive(sqlx::FromRow)]`:

```rust
// Settings (used by disclaimer)
#[derive(sqlx::FromRow)]
pub struct Setting {
    pub key: Option<String>,
    pub value: String,
    pub description: Option<String>,
}

// (Designer, Source, Hoop, Tag, Design, Project, DesignTag, ProjectDesign
//  survive as placeholder structs for later use)
```

### 3.9 src/database/schema.rs (rewrite)

Replace Diesel `table!` macros with SQLx-compatible constant definitions (or simply delete — SQLx uses inline SQL, not a separate schema module). Minimal skeleton:

```rust
// SQLx uses inline queries; no need for a generated schema module.
// Table-name constants are provided here for code readability.
pub mod tables {
    pub const SETTINGS: &str = "settings";
    pub const DESIGNS: &str = "designs";
    // ... etc
}
```

### 3.10 src/database/migrations.rs (rewrite)

```rust
use sqlx::SqliteConnection;

pub async fn run_migrations(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(conn).await
}
```

### 3.11 src/settings.rs (rewrite)

Convert all Diesel queries to SQLx:

```rust
use sqlx::SqliteConnection;
use crate::database::models::Setting;

pub async fn load_all_settings(conn: &mut SqliteConnection) -> Result<Vec<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT key, value, description FROM settings")
        .fetch_all(conn)
        .await
}

pub async fn get_setting(conn: &mut SqliteConnection, key: &str) -> Result<Option<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT key, value, description FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(conn)
        .await
}

pub async fn update_setting(conn: &mut SqliteConnection, key: &str, value: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE settings SET value = ? WHERE key = ?")
        .bind(value)
        .bind(key)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}
```

### 3.12 src/disclaimer.rs (rewrite)

Convert Diesel queries to SQLx (now async):

```rust
use sqlx::SqliteConnection;
use crate::settings::{get_setting, update_setting};

pub async fn is_disclaimer_accepted(conn: &mut SqliteConnection) -> bool {
    match get_setting(conn, "disclaimer_accepted").await {
        Ok(Some(setting)) => setting.value == "TRUE",
        _ => false,
    }
}

pub async fn set_disclaimer_accepted(conn: &mut SqliteConnection, accepted: bool) -> bool {
    let value = if accepted { "TRUE" } else { "FALSE" };
    update_setting(conn, "disclaimer_accepted", value).await.is_ok()
}
```

### 3.13 DISCLAIMER.html (create at project root if missing)

The actual disclaimer content. The Python app reads this file at `_PROJECT_ROOT / "DISCLAIMER.html"`. For the Rust version, this file will be embedded into the binary via `include_str!("../DISCLAIMER.html")`. The content is served to the frontend through the `get_disclaimer_text` Tauri command.

### 3.14 frontend/ directory (create — Svelte + Vite)

```
frontend/
├── package.json
├── vite.config.ts
├── svelte.config.js
├── index.html
└── src/
    ├── main.js
    ├── App.svelte
    ├── lib/
    │   ├── DisclaimerView.svelte
    │   └── MainView.svelte
    └── app.css
```

**package.json** dependencies:
- `svelte` (dev)
- `@sveltejs/vite-plugin-svelte` (dev)
- `vite` (dev)
- `@tauri-apps/api` (runtime — for `invoke()`)

**vite.config.ts:** Configure server port (5173), set `build.outDir` to `dist`, ensure Tauri-compatible build settings.

**index.html:** Minimal HTML shell:
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Embroidery Catalogue</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.js"></script>
</body>
</html>
```

**src/main.js:**
```js
import App from './App.svelte';
const app = new App({ target: document.getElementById('app') });
export default app;
```

### 3.15 Svelte Components

#### App.svelte

The root component. Manages the disclaimer flow state:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import DisclaimerView from './lib/DisclaimerView.svelte';
  import MainView from './lib/MainView.svelte';

  let disclaimerAccepted = $state(false);
  let loading = $state(true);

  async function checkDisclaimer() {
    try {
      disclaimerAccepted = await invoke('check_disclaimer');
    } catch (e) {
      console.error('Failed to check disclaimer:', e);
    } finally {
      loading = false;
    }
  }

  function onDisclaimerAccepted() {
    disclaimerAccepted = true;
  }

  // Check disclaimer on mount
  $effect(() => { checkDisclaimer(); });
</script>

{#if loading}
  <div class="flex items-center justify-center h-screen">
    <p class="text-gray-500">Loading...</p>
  </div>
{:else if !disclaimerAccepted}
  <DisclaimerView onAccepted={onDisclaimerAccepted} />
{:else}
  <MainView />
{/if}
```

#### DisclaimerView.svelte

Shows the disclaimer text, requires a checkbox, and calls `invoke('accept_disclaimer')`:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';

  let { onAccepted } = $props();
  let disclaimerHtml = $state('');
  let checked = $state(false);
  let accepting = $state(false);
  let error = $state('');

  async function loadDisclaimer() {
    try {
      disclaimerHtml = await invoke('get_disclaimer_text');
    } catch (e) {
      error = 'Failed to load disclaimer text.';
    }
  }

  async function accept() {
    if (!checked || accepting) return;
    accepting = true;
    error = '';
    try {
      await invoke('accept_disclaimer');
      onAccepted();
    } catch (e) {
      error = 'Failed to save acceptance. Please try again.';
    } finally {
      accepting = false;
    }
  }

  $effect(() => { loadDisclaimer(); });
</script>

<div class="max-w-4xl mx-auto py-6 px-4 space-y-4">
  <div class="bg-amber-50 border border-amber-300 text-amber-900 rounded-lg px-4 py-3">
    Before using the app, please review and accept this disclaimer.
    You will only be asked once for this installation.
  </div>

  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h1 class="text-2xl font-bold text-gray-800">Disclaimer</h1>

    <div class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 space-y-4">
      {@html disclaimerHtml}
    </div>

    {#if error}
      <div class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm">
        {error}
      </div>
    {/if}

    <label class="flex items-start gap-3 text-sm text-gray-700">
      <input type="checkbox" bind:checked={checked} class="mt-1" />
      <span>I have read and accept the disclaimer above.</span>
    </label>

    <button
      onclick={accept}
      disabled={!checked || accepting}
      class="bg-indigo-600 text-white px-4 py-2 rounded text-sm hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {accepting ? 'Accepting...' : 'Accept and continue'}
    </button>
  </div>
</div>
```

#### MainView.svelte

Placeholder main view (to be expanded in future tasks):

```svelte
<div class="max-w-7xl mx-auto px-4 py-6">
  <h1 class="text-2xl font-bold text-gray-800">🧵 Embroidery Catalogue</h1>
  <p class="text-gray-600 mt-2">Welcome! The disclaimer has been accepted.</p>
</div>
```

---

## 4. Tauri Commands (in src/main.rs)

| Command                   | Returns       | Description                                                                      |
|---------------------------|---------------|----------------------------------------------------------------------------------|
| `check_disclaimer`        | `bool`        | Calls `disclaimer::is_disclaimer_accepted()` to check the `disclaimer_accepted` setting. |
| `accept_disclaimer`       | `()`          | Calls `disclaimer::set_disclaimer_accepted(true)` to persist acceptance.         |
| `get_disclaimer_text`     | `String`      | Returns the bundled `DISCLAIMER.html` content (from `AppState::disclaimer_text`).|

All commands take `tauri::State<'_, AppState>` to access the database connection and disclaimer text.

The `AppState` struct:
```rust
pub struct AppState {
    pub db: std::sync::Mutex<SqliteConnection>,
    pub disclaimer_text: String,
}
```

On `setup`, the app:
1. Loads `.env` via `dotenvy` (or manual parsing) to set `DATABASE_URL`
2. Establishes the SQLite connection
3. Runs any pending migrations
4. Wraps connection in `Mutex` and stores in Tauri managed state

---

## 5. Tauri Command Implementations

```rust
#[tauri::command]
async fn check_disclaimer(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(disclaimer::is_disclaimer_accepted(&mut conn).await)
}

#[tauri::command]
async fn accept_disclaimer(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    disclaimer::set_disclaimer_accepted(&mut conn, true).await;
    Ok(())
}

#[tauri::command]
fn get_disclaimer_text(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.disclaimer_text.clone())
}
```

---

## 6. Migration Strategy

The existing `migrations/2026-05-03-000000_initial/up.sql` is plain SQL and is fully compatible with SQLx. SQLx's `sqlx::migrate!("./migrations")` macro reads the same directory structure. **No migration file changes are needed.**

The migration:
- Creates all tables (`designers`, `sources`, `hoops`, `tags`, `designs`, `projects`, `settings`, `design_tags`, `project_designs`)
- Inserts default settings (including `disclaimer_accepted = 'FALSE'`)
- Inserts default tags (stitching types and image categories)

---

## 7. CSS / Styling Approach

Continue using **Tailwind CSS via CDN** (`<script src="https://cdn.tailwindcss.com"></script>`) as the Python app does in `templates/base.html`. This avoids a build-time CSS pipeline and keeps the initial setup simple.

The dark-mode CSS currently in `base.html` can be ported to a `<style>` block in `index.html` or a separate `app.css` file in a future task.

---

## 8. What Stays Unchanged

| File / Module                                   | Notes                                                        |
|-------------------------------------------------|--------------------------------------------------------------|
| `src/models/mod.rs` (EmbPattern, Stitch, etc.)  | Core domain models — no DB dependency, no changes needed     |
| `src/readers/` (all embroidery format readers)  | No DB dependency, no changes needed                          |
| `src/png_writer.rs`                             | Rendering — no DB dependency, no changes needed              |
| `src/templating.rs`                             | No longer needed; can be removed or left as placeholder      |
| `src/utils.rs`                                  | Kept as-is for now                                           |
| `migrations/`                                   | SQL compatible with SQLx, no changes needed                  |
| `static/`, `templates/`                         | Not used by the Rust+Tauri app; can be removed in a follow-up |

---

## 9. Build & Run Commands

```bash
# Development (with hot-reload)
cd frontend && npm install
cd ..
cargo tauri dev

# Production build (Windows .exe + installer)
cargo tauri build
```

---

## 10. Implementation Order

1.  Update `Cargo.toml` — add Tauri + SQLx deps, remove Diesel
2.  Install SQLx CLI (`cargo install sqlx-cli`)
3.  Create `build.rs` with `tauri_build::build()`
4.  Rewrite `src/database/connection.rs` for SQLx
5.  Rewrite `src/database/models.rs` for SQLx
6.  Rewrite `src/database/schema.rs` (or skeleton)
7.  Rewrite `src/database/migrations.rs` with `sqlx::migrate!()`
8.  Rewrite `src/settings.rs` for SQLx (async)
9.  Rewrite `src/disclaimer.rs` for SQLx (async)
10. Delete `diesel.toml`, `src/schema.rs`, `src/models_db.rs`
11. Ensure `DISCLAIMER.html` exists at project root
12. Create `src-tauri/tauri.conf.json`, `capabilities/default.json`, icon placeholders
13. Rewrite `src/main.rs` — Tauri builder, AppState, commands, setup hook
14. Scaffold Svelte frontend (`frontend/`):
    - `package.json`, `vite.config.ts`, `svelte.config.js`
    - `index.html`
    - `src/main.js`, `src/App.svelte`
    - `src/lib/DisclaimerView.svelte`, `src/lib/MainView.svelte`
15. Run `cargo tauri dev` to test the disclaimer flow end-to-end

---

## 11. Verification Checklist

- [ ] `cargo check` passes (no compilation errors)
- [ ] `cargo tauri dev` launches the app window
- [ ] First run: Disclaimer view is shown
- [ ] Disclaimer text is loaded and displayed correctly
- [ ] Checkbox is required before "Accept" button becomes active
- [ ] Clicking "Accept" persists `disclaimer_accepted = TRUE` in the `settings` table
- [ ] After acceptance, MainView is shown
- [ ] Subsequent runs: Disclaimer is skipped (MainView shown immediately)
- [ ] SQLite database file is created at the correct location
- [ ] All migrations run successfully (tables, default settings, default tags)
- [ ] `cargo tauri build` produces a working Windows executable