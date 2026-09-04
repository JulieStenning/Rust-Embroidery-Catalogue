# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient.

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)


| Module / File Path                | Line % | Function % | Region % | Date       | Status           | Reason Accepted                                               |
| :-------------------------------- | :----- | :--------- | :------- | :--------- | :--------------- | :------------------------------------------------------------ |
| src/database/migrations.rs        | 88.37% | 100.00%    | 96.67%   | 2026-08-27 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/logging.rs                    | 94.51% | 90.00%     | 94.89%   | 2026-08-27 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/main.rs                       | 37.50% | 42.31%     | 35.33%   | 2026-09-03 | [ACCEPTED]       | Thin glue                                                     |
| src/paths.rs                      | 79.74% | 73.17%     | 80.25%   | 2026-08-27 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/routes/admin.rs               | 92.63% | 79.82%     | 86.78%   | 2026-08-28 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/routes/bulk_import.rs         | 79.57% | 72.82%     | 77.17%   | 2026-09-03 | [ACCEPTED]       | Thin glue and Internet tests for Gemini                       |
| src/routes/database_recovery.rs   | 92.68% | 57.14%     | 86.79%   | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/designs.rs             | 69.76% | 43.51%     | 68.56%   | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/maintenance.rs         | 50.43% | 39.58%     | 51.97%   | 2026-08-28 | [ACCEPTED]       | Thin glue + OS/Filesystem                                     |
| src/routes/projects.rs            | 0%     | 0%         | 0%       | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/restore.rs             | 7.17%  | 14.29%     | 6.96%    | 2026-08-28 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/routes/settings.rs            | 76.60% | 56.82%     | 67.59%   | 2026-09-03 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces  |
| src/routes/storage_migration.rs   | 8.00%  | 7.14%      | 4.67%    | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/tagging_actions.rs     | 75.94% | 48.94%     | 68.12%   | 2026-09-03 | [ACCEPTED] | Thin glue/Tauri framework limitation.OS/filesystem interfaces                                                  |
| src/services/about_documents.rs   | 92.52% | 94.12%     | 96.20%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/admin.rs             | 94.53% | 61.43%     | 81.31%   | 2026-08-28 | [ACCEPTED]       | High line coverage; DB error paths + dead branches            |
| src/services/auto_tagging.rs      | 82.72% | 70.00%     | 75.81%   | 2026-09-03 | [ACCEPTED]       |                                                                |
| src/services/backfill.rs          | 84.88% | 65.61%     | 81.28%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/db_health.rs         | 75.43% | 84.38%     | 68.25%   | 2026-08-28 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/services/design_metadata.rs   | 98.28% | 83.33%     | 93.75%   | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/services/fingerprint.rs       | 93.06% | 70.37%     | 91.77%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/maintenance.rs       | 95.45% | 66.67%     | 89.19%   | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/services/projects.rs          | 96.00% | 74.36%     | 85.68%   | 2026-08-28 | [ACCEPTED]       | Derive artifacts + high line coverage                         |
| src/services/restore.rs           | 78.68% | 66.67%     | 74.32%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/settings.rs          | 89.60% | 65.79%     | 83.23%   | 2026-08-28 | [ACCEPTED]       | Derive artifacts + high line coverage + OS/Filesystem         |
| src/services/storage_migration.rs | 69.29% | 54.55%     | 67.05%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/storage_migration.rs | 72.02% | 58.93%     | 68.97%   | 2026-09-03 | [ACCEPTED]       | OS/Filesystem                                                  |
| src/utils.rs                      | 90.00% | 75.00%     | 84.62%   | 2026-08-28 | [ACCEPTED]       | Dev tooling                                                   |

---

## Frontend (@Svelte Modules)


| Module / File Path                    | Line Coverage % | Function Coverage % | Branch / Region Coverage % | Date       | Status     | Reason Accepted                                                                             |
| :------------------------------------ | :-------------- | :------------------ | :------------------------- | :--------- | :--------- | :------------------------------------------------------------------------------------------ |
| src/lib/views/BrowseView.svelte       | 92.45%          | 89.91%              | 71.55%                     | 2026-08-29 | [ACCEPTED] |                                                                                             |
| src/lib/views/BrowseView.svelte       | 92.48%          | 91.45%              | 73.84%                     | 2026-08-28 | [ACCEPTED] | High line/function coverage; residual branch gaps are defensive guards + template fallbacks |
| src/lib/views/DesignDetailView.svelte | 90.34%          | 83.80%              | 67.42%                     | 2026-08-29 | [ACCEPTED] |                                                                                             |
| src/lib/views/DesignDetailView.svelte | 93.41%          | 90.82%              | 80.11%                     | 2026-08-28 | [ACCEPTED] | High line/function coverage; residual branch gaps are defensive guards + stale-token checks |
| src/lib/views/ImportView.svelte       | 96.18%          | 94.20%              | 77.26%                     | 2026-08-29 | [ACCEPTED] |                                                                                             |
| src/lib/views/ImportView.svelte       | 94.1%           | 95.62%              | 78.79%                     | 2026-08-28 | [ACCEPTED] | High line/function coverage; residual branch gaps are defensive guards + fallback branches  |
| src/lib/views/ProjectsView.svelte     | 96.76%          | 93.93%              | 69.81%                     | 2026-08-29 | [ACCEPTED] |                                                                                             |
| src/lib/views/ProjectsView.svelte     | 95.88%          | 93.85%              | 85.65%                     | 2026-08-28 | [ACCEPTED] | High line/function coverage; residual branch gaps are defensive guards + stale-token checks |
| src/lib/views/TagsView.svelte         | 86.66%          | 100%                | 67.34%                     | 2026-08-29 | [ACCEPTED] |                                                                                             |
| src/lib/views/TagsView.svelte         | 99%             | 100%                | 85.71%                     | 2026-08-28 | [ACCEPTED] | High line/function coverage; residual branch gaps are SSR guards + re-entrancy guard        |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:

1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.
