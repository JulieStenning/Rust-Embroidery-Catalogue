# Testing Discipline, Commands & Execution Benchmarks

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

| Tier / Check             | Command                                                             | Measured Duration | Purpose & Cadence                                |
| ------------------------ | ------------------------------------------------------------------- | ----------------- | ------------------------------------------------ |
| **Rust Fast Format**     | `cargo fmt --check`                                                 | ~3 s              | Instant formatting check.                        |
| **Rust Fast Compile**    | `cargo check`                                                       | ~3 s              | Instant borrow / type validation during edits.   |
| **Frontend Lint**        | `npm run lint:frontend`                                             | ~13 s             | ESLint syntax & rule validation.                 |
| **Repo Format Check**    | `npm run format:check`                                              | ~16 s             | Prettier check across frontend & root docs.      |
| **Frontend Type-check**  | `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` | ~17 s             | Native Svelte 5 / TypeScript type-check.         |
| **Rust Tests**           | `cargo test`                                                        | ~17 s             | Comprehensive backend unit & integration tests.  |
| **Rust Docs**            | `cargo doc --no-deps`                                               | ~18 s             | Verifies rustdoc links & documentation.          |
| **Rust Clippy**          | `cargo clippy --all-targets -- -D warnings`                         | ~26–35 s          | Strict linter verification.                      |
| **Rust Aggregate**       | `npm run check:rust`                                                | ~48 s             | Full Rust suite (fmt, clippy, doc, test).        |
| **Vitest Test Suite**    | `npm test` (`npx vitest run`)                                       | ~55 s             | Full frontend unit & integration test suite.     |
| **Frontend Aggregate**   | `npm run check:frontend`                                            | ~98 s (~1.5 min)  | Runs vitest + format:check + lint:frontend.      |
| **Full Quality Check**   | `npm run check:all`                                                 | ~118 s (~2 min)   | Full repo verification (frontend + rust).        |
| **E2E Debug Build**      | `npm run e2e:build` (`cargo tauri build --debug --no-bundle`)       | ~265 s (~4.5 min) | Frontend build + debug Tauri executable compile. |
| **Playwright E2E Tests** | `npm run e2e` (`npx playwright test`)                               | ~658 s (~11 min)  | Complete desktop WebView2 UI & workflow tests.   |

### 🧭 Cadence & Check Frequency Guidelines

1. **Do not poll running checks:** Background tasks are reactive and automatically notify when complete. Never poll `manage_task(action="status")` in a loop.
2. **Right-size verification during development:**
   - For rapid Rust edits, use `cargo check` (~3 s) instead of running full test or clippy suites on every minor change.
   - For rapid frontend edits, use `svelte-check` (~17 s) to catch TypeScript / rune issues.
   - Run full suites (`npm run check:all` ~2 min, `npm run e2e` ~11 min) only at major milestones or before final delivery, rather than after every individual file edit.
3. **Set appropriate async timeouts:** When executing longer commands, set `WaitMsBeforeAsync` or background expectations aligned with the table above rather than timing out prematurely.
