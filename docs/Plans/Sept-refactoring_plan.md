# Pre-Delivery Refactoring Plan

Prepare the codebase for pre-delivery testing by clearing automated release gate failures (Clippy `-D warnings`, ESLint, Rustfmt), consolidating path operations into `src/paths.rs` per project architectural rules, and eliminating dead code and unused props.

## User Review Required

> [!IMPORTANT]
> The automated pre-release script [`run-release-checks.ps1`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/run-release-checks.ps1) executes `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `npm run lint`. Currently, all three quality gates fail. Fixing these items is required before the release pipeline can succeed.

> [!NOTE]
> Decomposing giant files like `commandAdapter.ts` (3,695 lines) into domain service files (`designsAdapter.ts`, `backupAdapter.ts`, etc.) is recommended for medium-term maintainability. However, to avoid introducing unnecessary risk right before user acceptance testing, this plan focuses on:
> 1. **Phase 1: Release Gate Fixes** (Clippy, ESLint, Rustfmt)
> 2. **Phase 2: Architectural Path Invariants & Dead Code Removal** (Unify path helpers in `paths.rs`, clean unused props & state)
> Monolith decomposition of `commandAdapter.ts` and large Svelte views can be planned as a follow-up after initial verification.

## Open Questions

None at this time. All refactorings are non-breaking, behavior-preserving code quality and architectural improvements.

---

## Proposed Changes

### Core Rust Architecture (`src/paths.rs`)

Consolidate path normalization and relative path derivation so that `src/paths.rs` remains the single source of truth for all path manipulation per `.clinerules`.

#### [MODIFY] [paths.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/paths.rs)
- Fix Clippy warning on line 415 by rewriting `if full_lower.strip_prefix(&prefix).is_none() { return None; }` to `full_lower.strip_prefix(&prefix)?;`.
- Add `pub fn relative_path_under_root(full_path: &str, root: &Path) -> String`:
  Migrate the helper from `services/backfill.rs` into `paths.rs` with clean `.eq_ignore_ascii_case()` and forward-slash normalization.
- Add `pub fn normalize_windows_explorer_target(path: &Path) -> PathBuf`:
  Migrate the Windows verbatim prefix removal (`\\?\` and `\\?\UNC\`) and slash normalization from `routes/designs.rs` so it is reusable.
- Add `pub fn normalize_path_display(path: &Path) -> String`:
  Unified display-path normalizer for Windows/Unix display paths.

---

### Rust Backend Services & Routes

Clean up redundant closures, fix Clippy warnings, and switch callers to the unified path helpers in `src/paths.rs`.

#### [MODIFY] [backfill.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/services/backfill.rs)
- Remove the local definition of `relative_path_under_root` and re-export or call `crate::paths::relative_path_under_root`.
- Replace redundant closures `.map_err(|e| AppError::invalid_input(e))` and `.map_err(|e| AppError::database(e))` with `.map_err(AppError::invalid_input)` and `.map_err(AppError::database)` (lines 1563, 1651, 1754, 1759).

#### [MODIFY] [tagging_actions.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/routes/tagging_actions.rs)
- Update `backfill::relative_path_under_root` call to `crate::paths::relative_path_under_root`.

#### [MODIFY] [maintenance.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/routes/maintenance.rs)
- Remove unnecessary `return` statement on line 1275.
- Delegate `normalize_path_string` to `crate::paths::normalize_path_display`.

#### [MODIFY] [designs.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/routes/designs.rs)
- Delegate `normalize_windows_explorer_target` to `crate::paths::normalize_windows_explorer_target`.

#### [MODIFY] [settings.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/services/settings.rs)
- Replace redundant error-mapping closures `.map_err(|e| AppError::database(e))` with `.map_err(AppError::database)` on lines 89, 158, 181.

---

### Rust Tests & Formatting

Fix Clippy warnings in test files and format files with formatting drift.

#### [MODIFY] [maintenance_tests.rs](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/routes/maintenance_tests.rs)
- Fix unnecessary `.to_string()` on `to_string_lossy()` (lines 1413, 1512, 1574).
- Fix boolean comparison `exists() == false` to `!exists()` (line 1497).

#### [FORMAT] Rustfmt
- Run `rustfmt --edition 2021` on `src/services/projects.rs`, `src/services/restore.rs`, `src/services/restore_tests.rs`, `src/services/storage_migration_tests.rs`, and `src/services/tagging.rs` to satisfy `cargo fmt --check`.

---

### Frontend Services & Views

Fix ESLint errors, clean up unused imports/variables, and eliminate vestigial props.

#### [MODIFY] [MainView.test.ts](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/__tests__/MainView.test.ts)
- Replace `(payload: any)` on line 979 with `(payload: { page?: number | string })`.

#### [MODIFY] [BrowseView.test.ts](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/views/__tests__/BrowseView.test.ts)
- Replace `(payload: any)` on line 1809 with `(payload: { page?: number | string })`.

#### [MODIFY] [TaggingActionsView.backfill.test.ts](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/views/__tests__/TaggingActionsView.backfill.test.ts)
- Change `let view = render(...)` to `const view = render(...)` on line 259 to satisfy `prefer-const`.

#### [MODIFY] [commandAdapter.ts](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/api/commandAdapter.ts)
- Remove unused `RestoreProgress` type import (line 92).
- Remove unused error parameters in catch clauses `catch (error)` -> `catch` (lines 2768, 2790, 2805).

#### [MODIFY] [BrowseView.svelte](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/views/BrowseView.svelte) & [MainView.svelte](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/MainView.svelte)
- Remove unused `detailDesignId` prop from `BrowseView.svelte` and remove `{detailDesignId}` from `<BrowseView>` in `MainView.svelte`.

#### [MODIFY] [BackupView.svelte](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/frontend/src/lib/views/BackupView.svelte)
- Remove unused state variable `let restoreError = $state("")` (line 72).

---

## Verification Plan

### Automated Tests

1. **Rust Quality & Compilation Checks:**
   ```powershell
   cargo check
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```
2. **Frontend Quality & Lint Checks:**
   ```powershell
   cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"
   npm --prefix frontend run lint
   ```
3. **Frontend Test Suite:**
   ```powershell
   npx vitest run --silent
   ```
4. **Backend Test Suite (Targeted):**
   ```powershell
   cargo test paths
   cargo test maintenance
   cargo test backfill
   ```

### Manual Verification
- Verify the app launches smoothly in development (`npm run tauri dev` or `start-rust-app.bat`).
- Confirm navigation between Browse, Tagging Actions, Backup/Restore, and Design Details works with zero console errors or regressions.
