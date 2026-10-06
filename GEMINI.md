# Gemini Rules - Embroidery Catalogue Development

You are helping build **Embroidery Catalogue**, a local, offline desktop tool for cataloguing and browsing digital embroidery designs.

---

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
  ```rust
  // Rust command signature
  fn run_stitching_backfill(
      clear_stitching_mode: Option<String>,  // snake_case in Rust
      batch_size: Option<i64>,
  )
  ```
  ```typescript
  // Correct JavaScript invoke — camelCase keys map to snake_case Rust params
  invoke("run_stitching_backfill", {
    clearStitchingMode: "all", // camelCase → maps to clear_stitching_mode
    batchSize: 100, // camelCase → maps to batch_size
  });
  ```
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

---

## 🧩 Test-Surface & Boundary-Drift Discipline

- **Two MainView test suites exist, and both must track visible changes:**
  - `frontend/src/__tests__/MainView.test.ts` — mocked-view routing shell (asserts `data-testid`, footer hrefs, router branches).
  - `frontend/src/lib/__tests__/MainView.test.ts` — integration shell that mounts the **real** child views (asserts rendered headings/text/content).
  - When changing a route, footer link, or visible heading/copy, update **both** suites. Run `npx vitest run` after visible-surface changes.
- **Moving a feature across the Rust↔frontend boundary breaks the other side's tests:**
  - When a feature migrates across the IPC boundary, grep both `src/` and `frontend/src/` for assertions/contracts referencing the old surface (slugs, commands, routes, mock keys) and update them together.
- **Static-import roots differ per tool:**
  - Vite's production build is rooted at `frontend/` and **cannot** import files outside it (`?raw`, JSON, etc.); Vitest is repo-rooted and _can_.
  - When an asset generator writes outside the frontend root (e.g. `npm run generate:licences` → repo-root `src/assets/`), mirror the files inside `frontend/src/` via a sync script/hook (`postgenerate:licences`).
  - Reconcile any example import path against where the generator actually writes, including filename spelling (e.g. `LICENSE` vs `LICENCE`).
- **Shared process-global state makes Rust tests order-dependent:**
  - Any Rust test mutating shared resources (log files, process-wide atomics, data-root paths, environment variables, or the current working directory) must be marked `#[serial]`; prefer per-test temp dirs.
  - `std::env::set_current_dir` changes process-global state. Save and restore the original cwd **before** deleting a temporary directory, and resolve repository fixtures from `env!("CARGO_MANIFEST_DIR")` rather than `current_dir()`.
  - If a test passes alone but fails in the full suite, treat it as shared-state pollution and fix the isolation, not the assertion.

---

## 🎭 End-to-End Testing (Playwright → Tauri WebView2)

**Playwright attaches over WebView2 DevTools Protocol (CDP)** (Windows only).

- **Attach, don't launch:** Spawn the exe with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>`, poll `http://127.0.0.1:<port>/json/version`, then `chromium.connectOverCDP(...)` and take `browser.contexts()[0].pages()[0]`.
- **Build the e2e exe with Tauri CLI, not plain `cargo build`:** Use `cargo tauri build --debug --no-bundle` (embeds `frontend/dist`).
- **Wait out initial navigation:** WebView2 opens on `about:blank` first. The `page` fixture must wait for the app root (e.g. `page.waitForSelector("#app", { state: "attached" })`).
- **One worker, one app:** All tests share a single SQLite DB and desktop window → `fullyParallel: false`, `workers: 1`.
- **Root-level TS configs:** `playwright.config.ts`, `vitest.config.mts`, and `tests/e2e/**` use the root `tsconfig.json` with `"types": ["node"]` and `"lib": ["ES2022", "DOM", "DOM.Iterable"]`.

---

## 🦀 Rust Coding Standards

- **Zero Panics:** Absolute ban on `unwrap()` or `expect()` in binary parsing modules. Handle all out-of-bounds, empty, or malformed files gracefully via strict error types.
- **Performance-Focused I/O:** Use buffered readers (`BufReader`) and streaming/lazy parsing logic where possible.
- **Automated Verification:** Run `cargo check` or `cargo test` after editing Rust code to ensure the borrow checker is satisfied and tests pass.
- **Native folder pickers (`rfd`):** `FileDialog::set_directory` silently falls back to system defaults on unreadable/missing paths. Validate the start directory first and pass an explicit fallback (e.g. `app_handle.path().document_dir()`).
- **Instant Database Verification (< 1 ms vs minutes on large DBs):**
  - Never execute full integrity checks (e.g. `PRAGMA quick_check(1)` or full table scans) in synchronous startup or migration verification paths. On large catalogues (e.g. 7.6 GB+), `quick_check(1)` scans every single B-tree page until an error is found, hanging startup and restore operations for minutes on slow removable media.
  - Use fast, constant-time schema verification (`PRAGMA schema_version` + `SELECT 1 FROM settings LIMIT 1` + `SELECT 1 FROM designs LIMIT 1`), which completes in < 1 ms regardless of database size.
- **Rendering & Parsing Performance Discipline:**
  - Avoid nested $O(r^2 \cdot N)$ coordinate loops when rendering stitch files. Use vectorised, single-pass polygon/capsule rasterizers.
  - Performance-critical parsing and rendering pipelines must be benchmarked using Criterion harnesses in `benches/` to detect regressions.
- **Production Module Size Limit (< 500 lines):**
  - Every production Rust module must remain strictly under **500 lines**.
  - Complex modules must be decomposed into focused domain submodules (e.g. `src/routes/designs/` with `browse.rs`, `details.rs`, `metadata.rs`, `tags.rs`, `deletion.rs`, `launch.rs`, `preview.rs`, `types.rs`, `mod.rs`).

### 💾 Canonical `designs.filepath` format (single source of truth)

- **Invariant:** every row in `designs.filepath` is a _canonical relative path from the designs library root_ (`<data_root>/MachineEmbroideryDesigns` = `AppPaths.embroidery_designs_dir`).
- **Format rules:** forward slashes (`/`) only; **no leading slash**; **no absolute path or Windows drive letter**; preserves exact case. A root design is a bare filename (`rose.pes`); subfolders are `Flowers/rose.pes`.
- `MachineEmbroideryDesigns` must NEVER be stored as a path prefix in `filepath`.
- **Single source of truth:** all path logic goes through helpers in `src/paths.rs` (`canonical_design_rel`, `design_rel_from_full`, `resolve_design_filepath`).

### 📏 Test File Separation Rule

Any Rust source file whose total line count exceeds **500 lines** (production + embedded test code) MUST have its `#[cfg(test)]` module(s) extracted into a separate sibling file `<basename>_tests.rs` with `#[path = "<basename>_tests.rs"] mod tests;`.

---

## 🎨 Frontend & TypeScript Coding Standards

### 1. Native Svelte 5 TypeScript & Type Parity

- All new and refactored Svelte components must use `<script lang="ts">` with typed runes (`$state`, `$derived`, `$props<{ ... }>()`).
- Maintain strict type parity across the IPC bridge. If a Rust `struct` is returned by a Tauri command, create a matching TypeScript `interface` in `src/lib/types/` (and mirror `IpcError` in `src/lib/types/errors.ts`).

### 2. Component Size Limit (< 500 lines)

- Svelte components and views must remain under **500 lines**.
- Decompose complex views into cohesive subcomponents (e.g., `BrowseFilterPanel.svelte`, `BrowseSelectionBar.svelte`, `BrowseCardGrid.svelte` under `frontend/src/lib/components/browse/`).

### 3. Strict Typing & No Implicit Any

- Every single function/method/arrow parameter must be explicitly typed:
  - **No Implicit Any (TS7006):** Never leave arrow functions or inner closure parameters untyped (e.g. `const rank = (/** @type {string} */ name) => { ... }`).
  - **Empty Collection State (TS7005):** Always provide explicit JSDoc annotations when initializing empty array/object state with `$state([])` or `$state({})` (e.g. `/** @type {string[]} */ let items = $state([]);` or `let items = $state(/** @type {string[]} */ ([]));`), otherwise TypeScript infers `any[]`.
- **Guard nullable values before narrowing calls:** Short-circuit `null` first when passing into APIs requiring non-null types:
  ```typescript
  const uiKind = resolveCurrentUiKind(route); // string | null
  // ✅ uiKind !== null && UTILITY_KINDS.has(uiKind)
  ```

### 4. Lint & Type Verification

- Execute `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` after modifying frontend files to ensure zero type errors.
- Do **not** run `npm run check` from the repo root (the script lives in `frontend/package.json`).

### 5. Route-Level State Persistence

- `MainView.svelte` conditionally mounts one view per `currentUiKind`. Unmounting destroys local `$state`. Lift multi-step wizard state into module stores (e.g. `src/lib/stores/importSessionStore.ts`).
- Use context-aware back navigation via `previousRoute` in `MainView.svelte`.

### 6. Ambient module declarations (.d.ts)

- A `.d.ts` file with top-level `import`/`export` becomes a module augmentation and stops applying globally. Keep wildcard files pure scripts and use `/// <reference types="..." />`.

### 7. Visual Consistency & Canonical UI Theme Spec

- All UI components, surfaces, typography, forms, tables, and modal dialogs must strictly comply with the canonical [Look and Feel & UI Theme Implementation Specification](docs/Specs/look-and-feel-implementation-spec.md).
- **Theme Parity & Token Rules:** Always use CSS custom property tokens (`--surface-*`, `--text-*`, `--border-*`). Never use un-gated `@media (prefers-color-scheme: dark)` overrides without `:root:not([data-theme="light"])`, and avoid hardcoded raw opacity classes (like `bg-gray-50/50`) on cards.
- **Reference Gold Standards:** Model new pages and components on **Browse Designs**, **Choose Tags**, and **Design Details**.
- Primary action buttons use the app's purple/indigo + white look (`settings-primary-button menu-button-primary` classes). Do not override with ad-hoc colors.

---

## 🧪 Svelte 5 View Testing Rules

- **Flush synchronous reactivity with `tick()`, not polling:**
  ```typescript
  import { tick } from "svelte";
  await fireEvent.change(select, { target: { value: "some-value" } });
  await tick();
  const cards = screen.getAllByRole("article");
  ```
- **Scope `getByText` with `within()`** when text appears in multiple DOM regions.
- **Direct mock assertions over `waitFor`** for synchronous side-effects.
- **Choose fixture data that proves state changes:** Default order should differ from expected sorted/filtered order.
- **Mock components should expose `data-testid` and `data-*` attributes.**
- **Route-driven props branching templates must use `$derived`, not `const`.**
- **`vi.mock` hoisting:** Use `vi.hoisted()` for fixture objects passed into `vi.mock()`.
- **Module-scope re-evaluations:** Use `vi.resetModules()` + `vi.doMock()` and dynamically re-import both the test harness (`render`, `tick`) and component together to avoid dual Svelte instance issues.

---

## 🖥️ Command Execution & Formatting Gotchas

- **Test execution commands (`npm test` vs `frontend/` directory):**
  - **Frontend tests (`npm test` / `npx vitest run`):** Must ALWAYS be run from the **repo root**. The root `package.json` defines `"test": "vitest run"`. Running `npm test` from inside `frontend/` fails because `frontend/package.json` does NOT define a `test` script.
  - **Specific Vitest test file:** `npx vitest run <path_from_root>` (e.g. `npx vitest run frontend/src/lib/views/__tests__/ImportView.test.ts`).
  - **Vitest config loss in `frontend/`:** Running Vitest from inside `frontend/` also drops `vitest.config.mts` and causes `document is not defined`.
  - **Frontend type-checking (`svelte-check`):** The `check` script lives only in `frontend/package.json`. Run from root via `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"`. Do NOT run `npm run check` from the repo root.
  - **Backend Rust tests:** Run from repo root via `cargo test` (or `cargo test <module_or_test_name>`).
  - **Full quality check:** Run from repo root via `npm run check:all`.
- **Formatting specific Rust files:** Run `rustfmt --edition 2021 <files...>`. Never use `cargo fmt -- <files>` as it reformats the whole crate.
- **Formatting config — `.editorconfig`, `.prettierrc` and `.gitattributes` are separate mechanisms:**
  - `.gitattributes` is authoritative for checkout line endings: `*.bat` / `*.cmd` are CRLF; the four generated licence manifests and frontend TypeScript/Svelte/JavaScript under `frontend/src/` are LF. The frontend rules are extension-scoped so PNG/ICO assets remain binary.
  - `.editorconfig` guides editors. rustfmt does **not** read it. Prettier does, but `frontend/.prettierrc` overrides it only under `frontend/`; root files use `.editorconfig`, where `max_line_length` must remain 80.
  - Both `npm --prefix frontend run format:check` and root `npx prettier --check .` must pass on Windows and Linux. The former 161-file CRLF failure was fixed by LF attributes; widespread line-ending warnings now indicate attribute/normalization drift, not an accepted baseline.
  - Keep root and frontend formatter dependencies aligned. Both currently use Prettier 3.9.6 and `prettier-plugin-svelte` 4.1.1; mismatched plugin versions produced mutually incompatible Svelte `<textarea>` formatting. Update both manifests/lockfiles and run both formatting gates after a formatter change.
  - `.gitattributes` has no brace expansion (gitignore-style wildmatch); `.editorconfig` does.
  - CI runs root Prettier and targets `[master, main]`: `master` is the current default and `main` is a future-rename safeguard. Verify workflow filters against `origin`, never an assumed branch name.
- **Licence manifest generation is now a no-op unless dependencies changed:** `cargo tauri build` runs `npm run generate:licences`; `scripts/sync-licences.mjs` normalises generated artifacts to LF and `.gitattributes` pins all four manifests to LF. Do not automatically revert them. A real diff now represents a dependency/licence refresh and must be reviewed and committed deliberately.
- **Project licence is GPL-3.0-or-later (never MIT, never AGPL):**
  - Root `LICENSE` is a **byte-verbatim** copy of the GNU GPL-3.0 text — never prepend, append or edit anything in it. All attributions live in root `NOTICE`. Both are mirrored into `frontend/src/` by `scripts/sync-licences.mjs`; edit the root file and rerun the script.
  - `about.toml` `accepted[]` and `deny.toml` must keep `"GPL-3.0-or-later"`.
  - Every `.rs`/`.svelte`/`.ts`/`.js` file carries the GPL-3.0-or-later SPDX header. Re-run `pwsh ./scripts/add-spdx-headers.ps1` after adding source files.
  - `bundle.resources` in root `tauri.conf.json` ships `LICENSE` and `NOTICE`. The root config is authoritative.
  - `bundle.licenseFile` deliberately points at `src-tauri/disclaimer.rtf`, not `LICENSE`: the installer shows the risk disclaimer, while GPL metadata and the full licence/notices live in the in-app About / Licence pages. Do not redirect it to `LICENSE`.
  - `DISCLAIMER.html` and `src-tauri/disclaimer.rtf` are maintained twice by hand and must stay section-for-section equivalent. The differing `licenseFile` strings in the two config locations resolve to the same RTF because they are relative to different directories.
  - The unused `src-tauri/tauri.conf.json` knowingly omits root `LICENSE`/`NOTICE` resources and licence generation. Do not “sync” it with forbidden `../LICENSE`-style paths.
  - UI prose noun = British **licence**, verb = US **license**, proper names/metadata = US. Existing route/test/CSS identifiers remain unchanged.
- **Never `git add -A` / `git add .` / `git commit -a` — stage by explicit path:** this workspace is often edited by more than one agent at once, so unrelated WIP can appear mid-task. Check `git diff --numstat` first (and again right before each commit), stage an explicit list, then confirm via `git diff --cached --name-only` that the index holds only your files. Leave files carrying another agent's edits unstaged; shared headers ride along with their commit.
- **Commit message tense:** Write commit messages in the **past tense** (e.g. `refactored(frontend): ...`, not `refactor(frontend): ...`).

---

## ⏱️ Check & Test Execution Benchmarks (Execution Times & Cadence)

To avoid premature or redundant polling during task execution, use the measured benchmark timings below (measured on Windows with SSD; allow ~20s variance for cold cache / background load):

| Tier / Check | Command | Measured Duration | Purpose & Cadence |
|---|---|---|---|
| **Rust Fast Format** | `cargo fmt --check` | ~3 s | Instant formatting check. |
| **Rust Fast Compile** | `cargo check` | ~3 s | Instant borrow / type validation during edits. |
| **Frontend Lint** | `npm run lint:frontend` | ~13 s | ESLint syntax & rule validation. |
| **Repo Format Check** | `npm run format:check` | ~16 s | Prettier check across frontend & root docs. |
| **Frontend Type-check** | `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` | ~17 s | Native Svelte 5 / TypeScript type-check. |
| **Rust Tests** | `cargo test` | ~17 s | Comprehensive backend unit & integration tests. |
| **Rust Docs** | `cargo doc --no-deps` | ~18 s | Verifies rustdoc links & documentation. |
| **Rust Clippy** | `cargo clippy --all-targets -- -D warnings` | ~26–35 s | Strict linter verification. |
| **Rust Aggregate** | `npm run check:rust` | ~48 s | Full Rust suite (fmt, clippy, doc, test). |
| **Vitest Test Suite** | `npm test` (`npx vitest run`) | ~55 s | Full frontend unit & integration test suite. |
| **Frontend Aggregate** | `npm run check:frontend` | ~98 s (~1.5 min) | Runs vitest + format:check + lint:frontend. |
| **Full Quality Check** | `npm run check:all` | ~118 s (~2 min) | Full repo verification (frontend + rust). |
| **E2E Debug Build** | `npm run e2e:build` (`cargo tauri build --debug --no-bundle`) | ~265 s (~4.5 min) | Frontend build + debug Tauri executable compile. |
| **Playwright E2E Tests** | `npm run e2e` (`npx playwright test`) | ~658 s (~11 min) | Complete desktop WebView2 UI & workflow tests. |

### 🧭 Cadence & Check Frequency Guidelines

1. **Do not poll running checks:** Background tasks are reactive and automatically notify when complete. Never poll `manage_task(action="status")` in a loop.
2. **Right-size verification during development:**
   - For rapid Rust edits, use `cargo check` (~3 s) instead of running full test or clippy suites on every minor change.
   - For rapid frontend edits, use `svelte-check` (~17 s) to catch TypeScript / rune issues.
   - Run full suites (`npm run check:all` ~2 min, `npm run e2e` ~11 min) only at major milestones or before final delivery, rather than after every individual file edit.
3. **Set appropriate async timeouts:** When executing longer commands, set `WaitMsBeforeAsync` or background expectations aligned with the table above rather than timing out prematurely.

