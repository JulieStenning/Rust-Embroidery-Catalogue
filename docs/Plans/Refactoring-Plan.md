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

- [ ] **5.1 Backend Suite:** `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- [ ] **5.2 Frontend Suite:** `svelte-check`, `npm run lint`, `npx vitest run`.
- [ ] **5.3 Release Checks:** `.\run-release-checks.ps1`.
