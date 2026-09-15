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

- [ ] **1.1 `src/readers/exp_reader.rs` test extraction:**
  - Extract the inline `#[cfg(test)] mod tests` (538 lines) into sibling `src/readers/exp_reader_tests.rs` using `#[path = "exp_reader_tests.rs"] mod tests;`.
- [ ] **1.2 `src/services/db_health.rs` test extraction:**
  - Extract the inline `#[cfg(test)] mod tests` (575 lines) into sibling `src/services/db_health_tests.rs` using `#[path = "db_health_tests.rs"] mod tests;`.
- [ ] **1.3 Path Consolidation:**
  - In `src/routes/bulk_import.rs`, replace manual string path manipulations (`.replace('\\', "/")`, manual prefix stripping) with methods from `src/paths.rs`.
  - In `src/services/scanning.rs`, `src/services/database_recovery.rs`, and `src/services/backfill.rs`, unify path normalization with `src/paths.rs`.
- [ ] **1.4 Verification:** Run `cargo test` and `cargo clippy --all-targets -- -D warnings`.

---

### Phase 2: Frontend Strict Typing, ESLint Cleanups & Type Parity

Aligns with **Rule 4.1 (Strict Typing & Zero Implicit `any`)** and **Rule 4.2 (Parameter Casing & IPC Type Parity)**.

- [ ] **2.1 `ImportView.svelte` Typing & Cleanup:**
  - Replace JSDoc `Record<string, any>`, `@type {any}`, and `@type {any[]}` with explicit interfaces (`BulkImportPrecheckWire`, `ImportPreview`, `ScannedItemWire`, `ImportProgressWire`, etc.) from `src/lib/types/ipc.ts`.
  - Remove unused helper functions `getFolderPathFromFilePath` and `getFolderLabelFromFolderPath`.
- [ ] **2.2 `importSelection.test.ts` ESLint Cleanup:**
  - Remove unused variable assignment `c` to eliminate the ESLint warning.
- [ ] **2.3 `ProjectsView.svelte` & `OrphansView.svelte` Typing:**
  - Replace loose JSDoc `/** @type {any[]} */` with strongly typed project/orphan interfaces.
- [ ] **2.4 Type Parity Audit:**
  - Cross-check `src/lib/types/ipc.ts` against Rust request/response models in `src/models/` and `src/routes/`.
- [ ] **2.5 Verification:** Run `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"`, `cmd /c "cd frontend && npm run lint"`, and `npx vitest run`.

---

### Phase 3: Frontend Adapter Modularization

Aligns with **Rule 1.2 (Separation of Concerns)** and **Rule 4.2 (IPC Bridge & Parameter Casing)**.

- [ ] **3.1 Extract Domain Adapters from `commandAdapter.ts`:**
  - `src/lib/api/designsAdapter.ts` (browse, detail, favorite, rating, delete)
  - `src/lib/api/importAdapter.ts` (precheck, bulk import, import progress)
  - `src/lib/api/backupAdapter.ts` (backup create, restore, recovery)
  - `src/lib/api/adminAdapter.ts` (designers, sources, hoops, tags, AI config)
  - `src/lib/api/projectsAdapter.ts` (projects CRUD, print layout)
- [ ] **3.2 Maintain Compatibility via `commandAdapter.ts`:**
  - Re-export all domain functions and types from `commandAdapter.ts`.
- [ ] **3.3 Modularize Adapter Tests:**
  - Extract domain-specific tests from `commandAdapter.test.ts` into matching test files.
- [ ] **3.4 Verification:** Run `npx vitest run` and `npm run lint`.

---

### Phase 4: UI & Styling Consistency

Aligns with **Rule 4.4 (UI & Visual Consistency)**.

- [ ] **4.1 Inline Style Audit:**
  - Review views for ad-hoc inline `style="..."` overrides and replace with standard Tailwind classes and theme tokens (`#4f46d8`, etc.).
- [ ] **4.2 Verification:** Run `npx vitest run` and visual check.

---

### Phase 5: Automated Quality Gate Verification

Aligns with **Section 5 (Pre-Commit & Verification Quality Gates)**.

- [ ] **5.1 Backend Suite:** `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- [ ] **5.2 Frontend Suite:** `svelte-check`, `npm run lint`, `npx vitest run`.
- [ ] **5.3 Release Checks:** `.\run-release-checks.ps1`.
