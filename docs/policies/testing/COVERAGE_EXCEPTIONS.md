# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient. 

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)

| Module / File Path | Approx. Coverage | Date Accepted | Reason / Rationale |
| :--- | :--- | :--- | :--- |
| src/services/projects.rs | 31.40% | 2026-08-08 | [PENDING REVIEW] |
| src/services/db_health.rs | 42.59% | 2026-08-08 | [PENDING REVIEW] |
| src/main.rs | 56.20% | 2026-08-08 | [PENDING REVIEW] |
| src/readers/embroidery_reader.rs | 62.50% | 2026-08-08 | [PENDING REVIEW] |
| src/routes/designs.rs | 67.56% | 2026-08-08 | [PENDING REVIEW] |
| src/services/about_documents.rs | 73.47% | 2026-08-08 | [PENDING REVIEW] |
| src/routes/maintenance.rs | 74.27% | 2026-08-08 | [PENDING REVIEW] |
| src/readers/vp3_reader.rs | 74.32% | 2026-08-08 | [PENDING REVIEW] |
| src/error.rs | 75.38% | 2026-08-08 | [PENDING REVIEW] |

---

## Frontend (@Svelte Modules)

Use `@` path references for agent context resolution.

| Component Reference | Approx. Coverage | Date Accepted | Reason / Rationale |
| :--- | :--- | :--- | :--- |
| @DesignDetailView.svelte | ~40% | 2026-07-27 | Visual canvas & thumbnail rendering; tested via manual UX pass |
| @ImportTestHarness.svelte | 0% | 2026-07-27 | Dev-only test harness; excluded from production coverage targets |
| @DesignPrintView.svelte | ~20% | 2026-07-27 | Browser print API integration; DOM snapshot tests sufficient |
| @TechnicalDataGrid.svelte | ~60% | 2026-07-27 | Simple display grid for metadata; state management tested separately |
| @BackupView.svelte | ~45% | 2026-07-27 | Native file dialog wrappers handled by Tauri integration |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:
1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.