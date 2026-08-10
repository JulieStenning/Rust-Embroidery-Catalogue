# Build and Run Guide

How to build the Embroidery Catalogue app and run it in its two delivery modes.

---

## 1. Prerequisites

- **Rust** — install via [rustup](https://rustup.rs). Includes `cargo`.
- **Node.js + npm** — required for the frontend.
- **Tauri CLI** — one-time install: `cargo install tauri-cli --locked`
- **Frontend dependencies** — run `npm install` inside `frontend/` the first time (the dev scripts do this automatically if missing).

---

## 2. The Two Execution Modes

The app decides where to store its data at startup based on how it is run:

| Mode | Trigger | Data location |
|---|---|---|
| **Dev** | Debug build (dev scripts / `cargo tauri dev`) | `<repo>/dev_data/` |
| **Installed** | Release build | User-configured (chosen in the first-run wizard); falls back to `%APPDATA%/EmbroideryCatalogue` |

**Auto-seed behaviour:** on first launch in any mode, if no database exists at the resolved data location, the app copies the bundled seed database (`src-tauri/resources/EmbroideryCatalogue.db`) over and uses it. You never need to ship a database file separately.

---

## 3. Building

- **Release build** — run `build-rust-release.bat`. Produces MSI and NSIS installers under `target/release/bundle/`.
- **Debug build** — built automatically by the dev scripts, or run `cargo tauri build --debug --no-bundle`.
- **Clean all build artifacts** — run `cargo clean`. This wipes `target/` (debug and release). The next build recreates everything; the seed database is embedded in the binary so nothing data-bearing is lost.

---

## 4. Running in Dev Mode

In Dev mode the app reads and writes everything under `<repo>/dev_data/` so your work never touches a real user installation.

| Script | Builds? | Frontend dev server? | Use case |
|---|---|---|---|
| `start-rust-app.bat` | Yes (`cargo tauri dev`) | Yes (Vite + hot reload) | Active development |
| `start-rust-app-no-build.bat` | No | Yes (optional) | Quick restart after frontend-only changes |
| `start-rust-debug-exe.bat` | If missing | No | Run the standalone debug EXE, closest to the release experience |

- `start-rust-app-no-console.vbs` — silent wrapper that launches `start-rust-app.bat` without a console window.

---

## 5. Deploying (Installed)

A release build (`build-rust-release.bat`) produces installers in `target/release/bundle/`:

- **MSI installer** — double-click the `.msi` in `target/release/bundle/`. Installs the app to the usual Program Files location.
- **NSIS installer** — double-click the `*-setup.exe` in `target/release/bundle/`.

Both run in Installed mode. On first launch the setup wizard prompts for a data location (useful for keeping large design collections off the system drive). The choice is persisted to `%APPDATA%/EmbroideryCatalogue/config.json`, which survives reinstalls. Subsequent launches use the configured location and the wizard is not shown again unless setup is reset.

---

## 6. Resetting Data

To get a fresh database, delete the database file for the mode you are running and restart the app — it re-seeds from the bundled resource.

| Mode | Delete |
|---|---|
| Dev | `dev_data/Database/EmbroideryCatalogue.db` |
| Installed | `<configured data root>/Database/EmbroideryCatalogue.db` (or `%APPDATA%/EmbroideryCatalogue/Database/...` if no custom location was set) |
