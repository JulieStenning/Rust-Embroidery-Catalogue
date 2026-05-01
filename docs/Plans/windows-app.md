# Windows App / Installer Web-Agent Execution Brief

## Goal

Create a **sellable Windows desktop build** of Embroidery Catalogue that installs like a normal Windows app while keeping the GitHub repository available for free public access.

The paid build should remove the main user friction points in the current delivery model:

- no browser tab or manual `localhost` navigation
- no visible command prompt window
- no dependence on the user installing Python or dependencies themselves
- no fragile fixed-port startup failures for normal users
- proper installer, shortcuts, uninstall support, and upgrade path

---

## Web-Agent Execution Brief

### Mission

Implement the **first sellable Windows desktop version** of Embroidery Catalogue using the existing FastAPI/Jinja codebase, packaged as a normal Windows app with an installer.

The target outcome is a polished consumer-facing build that:

- runs without the user installing Python manually
- opens in its own app window instead of the browser
- does not show a separate command prompt window
- avoids fixed-port conflicts during normal startup
- preserves the current free/public GitHub source distribution and USB workflow

### Required approach

The agent should use this implementation direction unless blocked by hard evidence:

1. **keep the existing web UI** in `src/main.py`, `templates/`, and `src/routes/`
2. **wrap it in a Windows desktop shell** using WebView2 / `pywebview` or an equivalent embedded browser host
3. **package the main app** with PyInstaller or an equivalent reliable Windows bundler
4. **ship it with an installer**, preferably **Inno Setup**
5. keep the installed app **localhost-only** by default (`127.0.0.1`), not exposed to the local network

### Constraints and non-goals

The agent should **not** do any of the following in the first pass:

- rewrite the UI in WinForms, WPF, or another second UI stack
- remove or break the current portable / removable-media workflow
- depend on the user manually installing Python or pip packages
- introduce DRM, activation, payment handling, or store-submission work unless explicitly requested later

### Expected outputs

At minimum, the implementation work should produce or update:

- `desktop_launcher.py` or equivalent desktop entrypoint
- desktop-mode path handling in `src/config.py`
- dynamic localhost port selection for installed mode
- a main-app packaging spec such as `EmbroideryCatalogue.spec`
- a build script such as `build_desktop.bat`
- an installer script such as `installer/EmbroideryCatalogue.iss`
- updated user-facing docs describing the installed Windows product

### Working method for the web agent

- Work in **small, verifiable steps**.
- Keep the current app working while adding the desktop product path.
- Mark checklist items in this document as completed as the work lands.
- Do **not** claim completion until the build/test/install evidence has been captured.

---

## Recommended Direction

### Preferred approach: keep the current web UI and wrap it as a desktop app

For the first commercial Windows release, **do not rewrite the UI in Windows Forms**.

Instead:

1. keep the existing FastAPI + Jinja application in `src/main.py`
2. launch the local app server in the background
3. host the UI inside an embedded desktop window using **WebView2** (for example via `pywebview`)
4. ship it with a normal Windows installer

### Why this is the best option for v1

- Reuses the current working UI in `templates/` and existing routes in `src/routes/`
- Avoids a full duplicate UI rewrite in .NET
- Solves the browser and console-window issues quickly
- Keeps the app maintainable by one codebase
- Gets to a sellable Windows product faster and with less risk

### Not recommended for the first release

A **WinForms rewrite** would mean rebuilding the application UI, navigation, forms, validation, and interaction patterns in a second technology stack. That is a much larger product rewrite and should only be considered later if there is a strong business reason for a truly native UI.

> If a future native rewrite is ever justified, evaluate **WPF** or **WinUI** rather than WinForms.

---

## Current State in This Repository

The current repo is already close to a desktop-friendly architecture, but it is still delivered as a local web app.

### Relevant files

- `src/main.py` — FastAPI application entry point
- `src/database.py` — database bootstrap via `bootstrap_database()`
- `src/config.py` — path and environment-driven configuration
- `start.bat` — current launch script; opens browser and uses fixed ports
- `setup.bat` — first-run dependency bootstrap for portable/removable-media mode
- `EmbroiderySdLauncher.spec` — PyInstaller spec for the SD-card helper only
- `build_launcher.bat` — build script for the SD launcher
- `docs/USB_DEPLOYMENT.md` — current removable-media workflow
- `docs/COMMERCIAL.md` — public/commercial distribution note

### Current pain points confirmed from the codebase

1. **Browser dependency**
   - `start.bat` launches the app and opens `http://localhost:%APP_PORT%`
   - users see the app in a browser instead of a desktop window

2. **Fixed port usage**
   - `start.bat` defaults to port `8003` for development and `8002` for portable mode
   - this creates avoidable conflict risk on user machines

3. **Visible command window**
   - Uvicorn currently runs in a console session
   - users can see or accidentally close the feedback window

4. **Dependency/setup friction**
   - `setup.bat` bootstraps pip, creates a venv, and installs from `wheels/`
   - this is appropriate for removable media, but not ideal for a commercial desktop install

5. **Main app is not yet packaged as the desktop product**
   - `EmbroiderySdLauncher.spec` packages `portable_launcher.py`, not the main application itself

---

## Product Shape

The product should be split into two delivery modes:

| Mode | Audience | Delivery | Notes |
|---|---|---|---|
| **Free public version** | technical users / contributors | GitHub repo and portable workflow | remains openly available |
| **Paid Windows version** | general users | polished Windows installer | convenience build, support, and packaging |

This is consistent with the current AGPL-based repo approach documented in `README.md` and `docs/COMMERCIAL.md`.

---

## Success Criteria

The Windows product is ready when all of the following are true:

1. the app installs on a clean Windows machine with **no Python installed**
2. the user launches it from the Start Menu or desktop shortcut
3. the UI opens inside the application window, not in a normal browser tab
4. no separate console window appears during normal use
5. the app still launches when ports `8002` and `8003` are already occupied
6. data is stored in a safe per-user location and survives upgrades
7. uninstall works cleanly
8. the existing USB / portable workflow still works for the repository build

---

## Web-Agent Working Checklist

> The implementing agent should tick these off directly in this document as work is completed and verified.

### Core build checklist

- [ ] Confirm baseline architecture and keep the existing FastAPI/Jinja UI as the v1 product UI
- [ ] Add explicit runtime-mode handling for `development`, `portable`, and `desktop`
- [ ] Refactor writable desktop data paths into `%LOCALAPPDATA%\EmbroideryCatalogue\...`
- [ ] Add a desktop launcher that starts the app in the background and embeds the UI in a desktop window
- [ ] Suppress browser auto-open and remove the visible console window for the installed build
- [ ] Implement dynamic free-port selection and readiness checks via `/health`
- [ ] Package the **main app** as a desktop executable
- [ ] Add a Windows installer with shortcuts and uninstall support
- [ ] Preserve the current USB / SD-card workflow without regression
- [ ] Update public/commercial docs for the free-vs-paid distribution split
- [ ] Add or update tests for startup, shutdown, and desktop launch behavior
- [ ] Verify install, launch, uninstall, and upgrade behavior on a clean Windows machine

### Verification evidence checklist

- [ ] `pytest -q` passes after the changes
- [ ] New or updated desktop-launcher tests pass
- [ ] Packaged executable starts on a machine without Python installed
- [ ] App still launches when ports `8002` and `8003` are already occupied
- [ ] Installed build remains bound to `127.0.0.1` only
- [ ] No external browser is required in the normal installed flow
- [ ] No command prompt window is shown during normal use
- [ ] Uninstall/reinstall and install-over-upgrade behavior have been manually verified
- [ ] Release build has been checked to ensure no live `.env` / API key secrets are bundled

---

## Implementation Plan

## Phase 1 — Decide and formalize runtime modes

### Objective
Cleanly separate development, portable, and installed desktop behavior.

### Tasks

1. Add an explicit runtime mode such as:
   - `APP_MODE=development`
   - `APP_MODE=portable`
   - `APP_MODE=desktop`

2. Refactor `src/config.py` so that:
   - **development** keeps current repo-local behavior
   - **portable** keeps current self-relative `data/` behavior for USB/removable media
   - **desktop** stores writable data under:
     - `%LOCALAPPDATA%\EmbroideryCatalogue\database\`
     - `%LOCALAPPDATA%\EmbroideryCatalogue\designs\`
     - `%LOCALAPPDATA%\EmbroideryCatalogue\logs\`

3. Ensure all user-modifiable files are stored outside `Program Files`.

### Deliverables

- updated `src/config.py`
- clear runtime-mode rules documented in comments and docs

---

## Phase 2 — Create a desktop launcher for the main app

### Objective
Launch the existing FastAPI app as a background service inside a desktop shell.

### Tasks

1. Add a new entrypoint such as `desktop_launcher.py`.
2. In the launcher:
   - set `APP_MODE=desktop`
   - suppress browser auto-open using `EMBROIDERY_DISABLE_EXTERNAL_OPEN=1`
   - call `bootstrap_database()` from `src/database.py`
   - start Uvicorn programmatically in a background thread or process
   - wait for `GET /health` before showing the UI window
   - shut the server down cleanly when the desktop app closes

3. Replace terminal logging with file logging:
   - `%LOCALAPPDATA%\EmbroideryCatalogue\logs\app.log`

4. Wrap the UI in a desktop window using **WebView2** via `pywebview` or equivalent.

### Deliverables

- new `desktop_launcher.py`
- app opens in an embedded window instead of the system browser
- no visible console window during normal use

---

## Phase 3 — Remove fixed-port conflicts for installed mode

### Objective
Make startup reliable on end-user machines.

### Tasks

1. Stop depending on fixed ports for the installed desktop build.
2. Allocate a free `127.0.0.1` port dynamically at runtime.
3. Keep the server bound to **`127.0.0.1` only** for the installed app; do not expose `0.0.0.0` or LAN access by default.
4. Pass the actual chosen URL into the embedded desktop window.
5. Keep `8002` and `8003` only for the existing `start.bat` development/portable workflows.
6. Add clear fallback behavior if startup fails:
   - log the reason
   - show a user-friendly error dialog
   - optionally open the external browser only as a last-resort fallback mode

### Deliverables

- free-port selection helper
- reliable startup even when typical ports are already occupied

---

## Phase 4 — Package the main application

### Objective
Turn the main app into a distributable Windows desktop executable.

### Tasks

1. Add a new PyInstaller spec for the actual product, for example:
   - `EmbroideryCatalogue.spec`

2. Configure the build with:
   - `console=False`
   - bundled templates, static assets, icons, and Alembic files
   - bundled licence and notice files:
     - `LICENSE`
     - `THIRD_PARTY_NOTICES.md`

3. Prefer a **one-folder build first** for reliability and easier diagnosis.
4. Add a build script such as `build_desktop.bat`.
5. Keep `EmbroiderySdLauncher.spec` for the removable-media helper separately.

### Deliverables

- `EmbroideryCatalogue.spec`
- `build_desktop.bat`
- working packaged desktop app built from the repo

---

## Phase 5 — Create the Windows installer

### Objective
Ship the packaged desktop app as a polished Windows installer.

### Recommended installer technology

**Inno Setup**

This is a good fit for a single Windows desktop product because it is simpler to maintain than WiX for this use case and supports shortcuts, uninstall, per-machine install, and installer scripting.

### Tasks

1. Add an installer script such as:
   - `installer/EmbroideryCatalogue.iss`

2. Installer responsibilities:
   - install to `%ProgramFiles%\Embroidery Catalogue\`
   - create Start Menu shortcut
   - optionally create desktop shortcut
   - register uninstall entry
   - preserve user data under `%LOCALAPPDATA%` by default
   - optionally check for or bootstrap WebView2 if needed

3. Include product branding:
   - app icon
   - installer icon
   - product name and version metadata

4. Support **install-over-upgrade** for v1 so users can update by simply running the newer installer over the existing installation.
5. Defer auto-update to a later phase unless it becomes a release requirement.
6. If available, code-sign the executable and installer to reduce SmartScreen warnings.

### Deliverables

- Inno Setup script
- installable `.exe` installer artifact
- uninstaller working correctly

---

## Phase 6 — Keep the portable / removable-media workflow intact

### Objective
Do not break the existing USB deployment capability while adding the sellable desktop build.

### Tasks

1. Leave `start.bat` and `setup.bat` available for repository/portable use.
2. Preserve `docs/USB_DEPLOYMENT.md` as a separate workflow.
3. Ensure `populate_sdcard.bat` and `portable_launcher.py` continue to work independently of the installer build.
4. Avoid mixing the desktop product’s installed-data rules into the portable mode rules.

### Deliverables

- no regressions in the removable-media workflow
- desktop and portable modes can coexist cleanly

---

## Phase 7 — Documentation, licensing, and commercial packaging clarity

### Objective
Make the public/free vs paid/installer split clear and compliant.

### Tasks

1. Update:
   - `README.md`
   - `docs/GETTING_STARTED.md`
   - `docs/TROUBLESHOOTING.md`
   - `docs/COMMERCIAL.md`

2. Document:
   - how the free repo build is used
   - how the paid Windows installer is used
   - where user data is stored
   - offline behavior and first-run behavior
   - upgrade and uninstall process
   - Windows firewall / startup troubleshooting for end users

3. Ensure release packaging does **not** accidentally include a live `.env`, `GOOGLE_API_KEY`, or other user/developer secrets.
4. Review bundled runtime and wheel contents so the shipped installer includes only what is required.
5. Include licence and notices in the installer and shipped files.
6. Provide a source-code link for the shipped version to stay aligned with the AGPL distribution model.

### Deliverables

- updated docs for end users
- commercial distribution wording aligned with the actual product

---

## Phase 8 — Verification and release hardening

### Objective
Verify the installer build behaves like a consumer desktop app.

### Tasks

1. Add and/or update tests for:
   - launcher startup
   - free-port selection
   - `/health` readiness
   - graceful shutdown

2. Keep the existing tests for current launch behavior working, especially:
   - `tests/test_root_scripts.py`
   - `tests/test_portable_launcher.py`

3. Smoke-test on a clean Windows VM with:
   - no Python installed
   - ports `8002` and `8003` already occupied
   - no repo checkout present
   - offline first run
   - uninstall/reinstall and upgrade scenarios

4. Create a release checklist for every paid build.

### Deliverables

- verified installer workflow
- release checklist
- repeatable build and test steps

---

## Suggested File Additions / Changes

### New files likely needed

- `desktop_launcher.py`
- `EmbroideryCatalogue.spec`
- `build_desktop.bat`
- `installer/EmbroideryCatalogue.iss`
- optional helper module for port selection / startup readiness checks
- app icon resources

### Existing files likely to change

- `src/config.py`
- `src/main.py` (only if small launch-mode hooks are needed)
- `README.md`
- `docs/GETTING_STARTED.md`
- `docs/TROUBLESHOOTING.md`
- `docs/COMMERCIAL.md`

---

## Technical Notes for the Implementing Agent

### Keep for v1

- FastAPI backend and route structure
- Jinja templates and current HTML UI
- SQLite + Alembic
- existing portable/USB workflow

### Change for the paid Windows product

- replace browser-first startup with desktop-shell startup
- remove visible console window
- auto-select a free port at runtime
- package dependencies and runtime into the installed build
- add installer, shortcuts, and uninstall support

### Do not attempt in the first pass

- full WinForms or WPF rewrite
- Microsoft Store submission
- DRM / serial-key activation system
- auto-update service unless clearly needed for release one

---

## Acceptance Checklist

The work is complete only when the following can be demonstrated on a clean Windows machine:

- [ ] install succeeds without Python being preinstalled
- [ ] launching the app does not show a command window
- [ ] the UI opens in the app window, not an external browser tab
- [ ] the app starts successfully even when `8002` and `8003` are already in use
- [ ] database and imported files are stored in a safe writable per-user location
- [ ] uninstall works cleanly
- [ ] reinstall/upgrade preserves user data unless explicitly removed
- [ ] the repo’s existing portable workflow still works

---

## Final Recommendation

**Build the commercial version as a packaged Windows desktop shell around the existing FastAPI/Jinja app, and ship it with an Inno Setup installer.**

This is the fastest, lowest-risk way to solve the current user issues and create a sellable Windows product without throwing away the working codebase.
