# Comprehensive Codebase Refactoring Plan

This refactoring plan is formulated directly from the rules, architectural boundaries, and quality standards defined in [`docs/policies/refactoring/REFACTORING_RULES.md`](../policies/refactoring/REFACTORING_RULES.md).

It structures the refactoring work into phased, atomic, behavior-preserving slices to ensure clean architecture and 100% compliance with release quality gates ahead of the public GitHub release.

---

## Guiding Principles & Constraints

1. **Zero Breaking Changes / Behavior Preservation (Rules 1.1 & 1.2):**
   All refactoring tasks preserve external observable behavior, user experience, database integrity, and IPC contracts.
2. **Re-export Compatibility:**
   When modularizing large files (such as `commandAdapter.ts`), `commandAdapter.ts` will re-export all sub-modules to ensure zero disruption to existing view imports.
3. **Continuous Verification (Section 5):**
   Run all automated checks (`cargo test`, `vitest`, `clippy`, `eslint`, `svelte-check`) at the conclusion of each phase.

---

## Phased Execution Roadmap

### Phase 1: Backend Rust Conformance & Test Extractions

Aligns with **Rule 3.2 (Single Source of Truth for Paths)** and **Rule 3.3 (File Length & Test Extraction Rule >500 lines)**.

- [x] **1.1 `src/readers/exp_reader.rs` test extraction:**
  - Extracted the inline `#[cfg(test)] mod tests` (538 lines) into sibling `src/readers/exp_reader_tests.rs` using `#[path = "exp_reader_tests.rs"] mod tests;`.
- [x] **1.2 `src/services/db_health.rs` test extraction:**
  - Extracted the inline `#[cfg(test)] mod tests` (575 lines) into sibling `src/services/db_health_tests.rs` using `#[path = "db_health_tests.rs"] mod tests;`.
- [x] **1.3 Path Consolidation:**
  - In `src/routes/bulk_import.rs` and `src/services/backfill.rs`, unified path normalization with `crate::paths::path_within`.
- [x] **1.4 Verification:** `cargo test` (1,404 passed), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.

---

### Phase 2: Frontend Strict Typing, ESLint Cleanups & Type Parity

Aligns with **Rule 4.1 (Strict Typing & Zero Implicit `any`)** and **Rule 4.2 (Parameter Casing & IPC Type Parity)**.

- [x] **2.1 `ImportView.svelte` Cleanup:**
  - Removed unused helper functions `getFolderPathFromFilePath` and `getFolderLabelFromFolderPath`.
- [x] **2.2 `importSelection.test.ts` ESLint Cleanup:**
  - Removed unused variable assignment `c` to eliminate the ESLint warning.
- [x] **2.3 `ProjectsView.svelte` & `OrphansView.svelte` Typing:**
  - Added typed interfaces in `src/lib/types/ipc.ts` (`OrphanDesignItem`, `OrphansPageResult`, etc.) and typed state in `ProjectsView.svelte` (`ProjectSummary[]`) and `OrphansView.svelte` (`OrphanDesignItem[]`).
- [x] **2.4 Verification:** `svelte-check` (0 errors), `npm run lint` (0 errors), `vitest run` (1,235 passed).

---

### Phase 3: Frontend Adapter Modularization

Aligns with **Rule 1.2 (Separation of Concerns)** and **Rule 4.2 (IPC Bridge & Parameter Casing)**.

- [x] **3.1 Extract Domain Adapters from `commandAdapter.ts`:**
  - `src/lib/api/ipcClient.ts` (reusable `invokeLoose` IPC bridge helper)
  - `src/lib/api/designsAdapter.ts` (browse, detail, favorite, rating, delete)
  - `src/lib/api/projectsAdapter.ts` (projects CRUD, design assignments, print layouts)
  - `src/lib/api/importAdapter.ts` (precheck, bulk import, import progress)
  - `src/lib/api/tagsAdapter.ts` (tag catalog, bulk tag assignments)
  - `src/lib/api/batchOperationsAdapter.ts` (unified backfill, tagging counts, folder scopes)
  - `src/lib/api/backupAdapter.ts` (backup create, restore, recovery)
  - `src/lib/api/orphansAdapter.ts` (orphan scanning, pagination, deletion)
  - `src/lib/api/adminAdapter.ts` (designers, sources, hoops, DB stats & compaction)
  - `src/lib/api/settingsAdapter.ts` (app settings, Gemini config, migration, app status)
- [x] **3.2 Maintain Compatibility via `commandAdapter.ts`:**
  - Re-exported all domain functions, constants, and types from `commandAdapter.ts`.
- [x] **3.3 Verification:** `npx vitest run` (53 test files, 1,235 passed), `npm run lint` (0 errors), `svelte-check` (0 errors).


---

### Phase 4: UI & Styling Consistency

Aligns with **Rule 4.4 (UI & Visual Consistency)**.

- [x] **4.1 Inline Style Audit & Button / Modal Standardisation:**
  - Added `.menu-button-danger` and `.ui-action-button-danger` design tokens to `src/app.css` and replaced all ad-hoc `style="background-color:#dc2626;border-color:#dc2626;"` overrides across `BackupView.svelte`, `UnmatchedFilesReconciler.svelte`, `RestoreProgressPanel.svelte`, `DeleteDesignsModal.svelte`, `ConfirmDeleteProjectModal.svelte`, and `CancelBackupModal.svelte`.
  - Added clean `.modal-overlay`, `.modal-backdrop`, `.modal-dialog`, `.modal-header`, `.modal-body`, `.modal-footer` utility classes to `src/app.css` and standardized all dialog wrappers.
- [x] **4.2 Verification:** `npx vitest run` (53 test files, 1,235 tests passed), `svelte-check` (0 errors), `npm run lint` (0 errors).

---

### Phase 5: Automated Quality Gate Verification

Aligns with **Section 5 (Pre-Commit & Verification Quality Gates)**.

- [x] **5.1 Backend Suite:** `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1,404 passed).
- [x] **5.2 Frontend Suite:** `svelte-check` (0 errors, 0 warnings), `npm run lint` (0 errors), `npx vitest run` (53 test files, 1,235 passed).
- [x] **5.3 Release Checks:** `.\run-release-checks.ps1` & `.\verify-release-logs.ps1` (All 9 quality gates passed: Rust Check, Rust Clippy, Rust Tests, Prettier Results, Rust Formatting, Frontend Unit Tests, Playwright E2E Tests [197 passed], Svelte Type Check, and Tauri Packaging with MSI & NSIS installers).

---

## 6. Lessons Learned & Efficiency Guidelines for Future Refactoring

The following practical insights and patterns were discovered during this refactoring exercise and should be applied in future refactoring workflows:

### 6.1 Tauri E2E Test Binary Caching
- **Context:** Playwright tests run against the built debug desktop binary (`target/debug/embroidery-catalogue.exe`) over WebView2 CDP. Tauri compiles and bundles the built frontend assets (`frontend/dist/`) directly inside the binary.
- **Rule for Future Work:** When changing frontend templates, CSS, or TypeScript files, **always run `npm run e2e:build` before running individual Playwright tests (`npx playwright test <spec>`)**. Running tests without a rebuild executes against the previous binary bundle, making frontend fixes appear not to take effect.

### 6.2 Modal Dialogs & CSS Flexbox Constraints
- **Context:** When extracting inline modal styling into CSS utility classes, large content grids (such as tag selection lists with 60+ items) can expand beyond the viewport if flex shrinking rules are omitted.
- **Rule for Future Work:**
  - Every `.modal-dialog` and `.tag-chooser-dialog` must include `overflow: hidden;`, `display: flex; flex-direction: column;`, and `max-height: 88vh;`.
  - Every scrollable body (`.modal-body`, `.tag-chooser-body`) must include `min-height: 0;` alongside `overflow-y: auto; flex: 1;`. Without `min-height: 0;`, CSS flex items default to `min-height: auto`, which prevents shrinking and forces the modal container to grow taller than the window, pushing headers/controls to negative coordinates outside the viewport.

### 6.3 Svelte 5 Runes Lifecycle & Avoiding Dual Data Loads
- **Context:** In Svelte 5 runes mode, `$effect` runs on initial mount as well as whenever reactive prop dependencies change.
- **Rule for Future Work:** Do **not** pair `$effect` and `onMount` to trigger the same asynchronous loader (`loadDesignDetail`). Having both hooks fires two concurrent in-flight requests on mount, creating a race condition where the second slower response can wipe out user/test input (e.g. resetting form selections) while the component is being interacted with.

### 6.4 Scripted AST/Regex File Splitting
- **Context:** Splitting large monolithic adapter files (`commandAdapter.ts` at 3,818 lines) into 9 domain adapters was executed safely via an automated script (`scripts/split-command-adapter.mjs`).
- **Rule for Future Work:** For files exceeding 1,000 lines, use scripted AST extraction coupled with a backward-compatible barrel re-export file. This avoids manual copy-paste errors, keeps git diffs clean, and eliminates the need to update dozens of importing components.

### 6.5 Incremental Quality Gate Validation
- **Context:** Catching Svelte type errors (e.g., JSDoc casts in `.svelte` files) and Prettier format discrepancies early avoids long debugging cycles at the final release build stage.
- **Rule for Future Work:** Run `npx svelte-check --tsconfig frontend/jsconfig.json` and `npx prettier --check frontend/src` at the end of **each individual phase**, rather than deferring to the final release verification.

