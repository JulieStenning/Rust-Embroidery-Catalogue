# Refactoring Rules and Guidelines

This document outlines the mandatory rules, architectural boundaries, quality standards, and verification processes required when refactoring code in the **Embroidery Catalogue** repository.

---

## 1. Core Refactoring Principles

### 1.1 Strict Behavior Preservation
- Refactoring must **never** alter external observable behavior, user experience, database integrity, or IPC contracts unless explicitly defined in an approved specification.
- If a behavioral bug or missing feature is uncovered during refactoring, it must be addressed in a **separate, dedicated task/PR**, not bundled into the refactor.

### 1.2 Separation of Concerns & Atomic Slices
- Do not mix structural refactoring with:
  - New features or UI overhauls.
  - Dependency upgrades.
  - Configuration or build system overhauls.
- Break large refactors into small, test-backed, reviewable phases.

### 1.3 Pre-Refactor Safety Net
- Never begin modifying code without verifying that existing tests pass:
  ```bash
  cargo test
  npx vitest run
  ```
- If an area lacks adequate test coverage, write regression tests that capture the current behavior **before** refactoring the implementation.

---

## 2. Public Release, Licensing & Security Constraints

As this codebase will be made publicly available on GitHub to comply with third-party open-source library licenses:

### 2.1 Zero Secrets & Personal Data
- **Never commit credentials or private configurations:** Ensure no API keys, private tokens, personal absolute file paths, `.env` files, or proprietary user data are checked in.
- **Dynamic environment variables:** Dev-only settings (e.g. `GOOGLE_API_KEY`, `EMBROIDERY_DATA_ROOT`) must remain strictly ignored in release mode.

### 2.2 License & Attribution Compliance
- Maintain all third-party license notices, headers, and attribution manifests generated via `cargo-about` and `npm run generate:licences`.
- Do not remove or bypass license checks defined in `deny.toml` or `LICENCE`.

---

## 3. Backend (Rust) Refactoring Rules

### 3.1 Zero Panics in Production Code
- Absolute ban on `unwrap()`, `expect()`, or `panic!()` in production paths (especially binary parsers, database access, command routes, and data migration).
- Always propagate errors using `Result<T, AppError>` with explicit `thiserror` variants.

### 3.2 Single Source of Truth for Paths
- **All path normalization, resolution, and relative path derivation must use [`src/paths.rs`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/src/paths.rs).**
- Never create ad-hoc path manipulation or string stripping functions in services or routes.
- **Canonical relative path format for designs:**
  - Forward slashes only (`/`).
  - No leading slash.
  - No drive letters or absolute prefixes.
  - Case-preserving.
  - Never prefix with `MachineEmbroideryDesigns`.

### 3.3 File Length & Test Extraction Rule (>500 lines)
- Any Rust source file whose total line count exceeds **500 lines** must have its `#[cfg(test)]` module extracted into a sibling `_tests.rs` file.
- Use the `#[path]` module declaration:
  ```rust
  #[cfg(test)]
  #[path = "<basename>_tests.rs"]
  mod tests;
  ```
- Verify that `use super::*;` retains full access to private items and that `cargo test <module>` passes.

### 3.4 Database & Query Boundaries
- Keep all SQLite queries **strictly inside the Rust backend**; never expose SQL queries or direct DB handles to the frontend.
- Use Tauri's managed state (`tauri::State<DatabasePool>`). Do not spin up redundant connection pools.

### 3.5 Test Isolation & Shared State
- Any test that modifies shared process-wide state (such as logger configurations, global atomics, or environment variables) must:
  - Be marked with `#[serial]` (from the `serial_test` crate), or
  - Use isolated temporary directories (`tempfile::tempdir()`).

---

## 4. Frontend (Svelte 5 / TypeScript) Refactoring Rules

### 4.1 Strict Typing & Zero Implicit `any`
- Every parameter, return type, and store state must be explicitly typed.
- Maintain exact 1:1 type parity between Rust data structs and TypeScript interfaces in `src/lib/types/`.

### 4.2 IPC Bridge & Parameter Casing (Critical)
- Svelte components must **never** call Tauri's `invoke()` directly; all IPC must route through adapter modules under `src/lib/services/` or `src/lib/api/`.
- **CamelCase Payload Keys:** Tauri v2 automatically maps camelCase JS keys to snake_case Rust parameters. All `invoke()` payload objects **MUST** use camelCase keys.

### 4.3 Svelte 5 Rune State & Navigation
- Use `$derived` for route-driven or computed properties that must react to navigation changes.
- Multi-step wizard state (e.g. bulk import) that must survive cross-view navigation must be stored in module-level stores rather than local component state.

### 4.4 UI & Visual Consistency
- Use established color tokens and styling classes (e.g., indigo/purple `#4f46d8` for primary buttons).
- Avoid hardcoded inline CSS overrides (`style="..."`) that drift from design standards.

---

## 5. Pre-Commit & Verification Quality Gates

Before committing any refactored code or creating a PR, all automated quality gates must be executed and pass:

### 5.1 Rust Verification
```powershell
cargo check
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### 5.2 Frontend Verification
```powershell
cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"
npm run lint
npx vitest run
```

### 5.3 Automated Release Gate
Run the complete automated release check script from the repository root:
```powershell
.\run-release-checks.ps1
```

---

## 6. Commit Standards

- Write commit messages in the **past tense** describing the exact refactor performed:
  - *Example:* `refactored(backend): consolidated path resolution into paths.rs and eliminated clippy warnings`
  - *Example:* `refactored(frontend): extracted TaggingActionsView subcomponents and typed invoke payloads`
