# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient.

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)

| Module / File Path               | Line % | Function % | Region % | Date       | Status             | Reason Accepted                                       |
| :------------------------------- | :----- | :--------- | :------- | :--------- | :----------------- |:------------------------------------------------------|
| src/database/migrations.rs       | 72.73% | 100.00%    | 80.00%   | 2026-08-08 | [ACCEPTED]         | OS/Filesystem                                         |
| src/logging.rs                   | 86.89% | 75.00%     | 86.52%   | 2026-08-08 | [ACCEPTED]         |                                                       |
| src/main.rs                      | 22.55% | 18.42%     | 21.85%   | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/admin.rs              | 62.42% | 50.00%     | 60.79%   | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/bulk_import.rs        | 82.62% | 71.84%     | 79.75%   | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/designs.rs            | 51.18% | 32.39%     | 47.57%   | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/maintenance.rs        | 43.72% | 34.85%     | 45.63%   | 2026-08-08 | [ACCEPTED]         | Thin glue + OS/Filesystem                             |
| src/routes/projects.rs           | 0%     | 0%         | 0%       | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/settings.rs           | 47.27% | 31.58%     | 40.00%   | 2026-08-08 | [ACCEPTED]         | Thin glue                                             |
| src/routes/tagging_actions.rs    | 94.88% | 67.39%     | 91.67%   | 2026-08-08 | [ACCEPTED]         |                                                       |
| src/services/about_documents.rs  | 73.33% | 91.67%     | 73.47%   | 2026-08-08 | [ACCEPTED]         |                                                       |
| src/services/admin.rs            | 94.17% | 61.43%     | 82.07%   | 2026-08-08 | [ACCEPTED]         | Derive artifacts + high line coverage                 |
| src/services/backfill.rs         | 78.67% | 56.52%     | 72.30%   | 2026-08-08 | [ACCEPTED]         | OS/Filesystem                                         |
| src/services/db_health.rs        | 72.69% | 84.38%     | 68.85%   | 2026-08-08 | [ACCEPTED]         |                                                       |
| src/services/fingerprint.rs      | 93.46% | 66.67%     | 91.84%   | 2026-08-08 | [ACCEPTED]         | OS/Filesystem                                         |
| src/services/maintenance.rs      | 95.24% | 66.67%     | 89.33%   | 2026-08-08 | [ACCEPTED]         |                                                       |
| src/services/projects.rs         | 96.61% | 75.61%     | 88.11%   | 2026-08-08 | [ACCEPTED]         | Derive artifacts + high line coverage                 |
| src/services/settings.rs         | 95.90% | 76.92%     | 88.12%   | 2026-08-08 | [ACCEPTED]         | Derive artifacts + high line coverage + OS/Filesystem |
| src/utils.rs                     | 90.00% | 75.00%     | 84.62%   | 2026-08-08 | [ACCEPTED]         | Dev tooling                                           |

---

## Frontend (@Svelte Modules)

| Module / File Path                     | Line Coverage % | Function Coverage % | Branch / Region Coverage % | Date       | Status             | Reason Accepted |
| :------------------------------------- | :-------------- | :------------------ | :------------------------- | :--------- | :----------------- |:----------------|
| src/lib/views/BrowseView.svelte        | 92.45%          | 89.91%              | 71.55%                     | 2026-08-08 | [ACCEPTED]         |                 |
| src/lib/views/DesignDetailView.svelte  | 90.34%          | 83.80%              | 67.42%                     | 2026-08-08 | [ACCEPTED]         |                 |
| src/lib/views/ImportView.svelte        | 96.18%          | 94.20%              | 77.26%                     | 2026-08-08 | [ACCEPTED]         |                 |
| src/lib/views/OrphansView.svelte       | 96.09%          | 100%                | 79.54%                     | 2026-08-08 | [ACCEPTED]         |                 |
| src/lib/views/ProjectsView.svelte      | 96.76%          | 93.93%              | 69.81%                     | 2026-08-08 | [ACCEPTED]         |                 |
| src/lib/views/TagsView.svelte          | 86.66%          | 100%                | 67.34%                     | 2026-08-08 | [ACCEPTED]         |                 |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:

1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.