# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient.

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)


| Module / File Path                | Line % | Function % | Region % | Date       | Status           | Reason Accepted                                               |
| :-------------------------------- | :----- | :--------- | :------- | :--------- | :--------------- | :------------------------------------------------------------ |
| src/database/migrations.rs        | 88.37% | 100.00%    | 96.67%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/logging.rs                    | 94.51% | 90.00%     | 94.89%   | 2026-08-27 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/logging.rs                    | 92.47% | 90.00%     | 93.53%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem; all metrics >= 80%                             |
| src/main.rs                       | 37.84% | 42.31%     | 35.33%   | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/paths.rs                      | 84.21% | 77.55%     | 86.17%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/routes/admin.rs               | 92.63% | 79.82%     | 86.78%   | 2026-09-15 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/routes/batch_operations.rs    | 60.53% | 41.07%     | 54.73%   | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/routes/bulk_import.rs         | 84.48% | 74.09%     | 82.46%   | 2026-09-15 | [ACCEPTED]       | Thin glue and Internet tests for Gemini                       |
| src/routes/database_recovery.rs   | 92.68% | 57.14%     | 86.79%   | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/designs.rs             | 69.76% | 43.51%     | 68.56%   | 2026-08-28 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/designs.rs             | 68.33% | 42.37%     | 67.16%   | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/routes/maintenance.rs         | 50.43% | 39.58%     | 51.97%   | 2026-08-28 | [ACCEPTED]       | Thin glue + OS/Filesystem                                     |
| src/routes/maintenance.rs         | 48.82% | 39.01%     | 48.82%   | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/routes/projects.rs            | 0.00%  | 0.00%      | 0.00%    | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/restore.rs             | 7.17%  | 14.29%     | 6.96%    | 2026-08-28 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/routes/restore.rs             | 6.25%  | 13.79%     | 5.88%    | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/routes/settings.rs            | 76.60% | 56.82%     | 67.59%   | 2026-09-03 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces  |
| src/routes/settings.rs            | 75.56% | 56.82%     | 67.59%   | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/routes/storage_migration.rs   | 8.00%  | 7.14%      | 4.67%    | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/tagging_actions.rs     | 75.94% | 48.94%     | 68.12%   | 2026-09-03 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/services/about_documents.rs   | 92.73% | 94.12%     | 96.20%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/admin.rs             | 94.53% | 61.43%     | 81.31%   | 2026-09-15 | [ACCEPTED]       | High line coverage; DB error paths + dead branches            |
| src/services/auto_tagging.rs      | 94.74% | 84.38%     | 91.78%   | 2026-09-15 | [ACCEPTED]       | All metrics >= 80%                                            |
| src/services/backfill.rs          | 85.41% | 67.11%     | 81.36%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/db_health.rs         | 85.47% | 91.89%     | 80.34%   | 2026-09-15 | [ACCEPTED]       | All metrics >= 80%                                            |
| src/services/design_metadata.rs   | 100.00%| 100.00%    | 100.00%  | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/services/fingerprint.rs       | 93.06% | 70.37%     | 91.77%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/fingerprint.rs       | 92.35% | 72.00%     | 90.94%   | 2026-09-15 | [PENDING REVIEW] |                                                               |
| src/services/gemini_client.rs     | 94.59% | 92.42%     | 94.07%   | 2026-09-04 | [ACCEPTED]       | Network client; injectable-base HTTP mock tests               |
| src/services/gemini_client.rs     | 94.54% | 92.06%     | 94.35%   | 2026-09-15 | [ACCEPTED]       | Network client; injectable-base HTTP mock tests               |
| src/services/maintenance.rs       | 97.78% | 83.33%     | 93.75%   | 2026-09-15 | [ACCEPTED]       | Thin glue                                                     |
| src/services/projects.rs          | 96.09% | 74.36%     | 85.68%   | 2026-09-15 | [ACCEPTED]       | Derive artifacts + high line coverage                         |
| src/services/restore.rs           | 80.67% | 77.78%     | 78.59%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/settings.rs          | 89.74% | 71.43%     | 83.94%   | 2026-09-15 | [ACCEPTED]       | Derive artifacts + high line coverage + OS/Filesystem         |
| src/services/storage_migration.rs | 69.29% | 54.55%     | 67.05%   | 2026-08-28 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/storage_migration.rs | 72.02% | 58.93%     | 68.97%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/utils.rs                      | 90.00% | 75.00%     | 84.62%   | 2026-09-15 | [ACCEPTED]       | Dev tooling                                                   |

---

## Frontend (@Svelte Modules)


| Module / File Path                         | Line Coverage % | Function Coverage % | Branch / Region Coverage % | Date       | Status           | Reason Accepted                                                                             |
| :----------------------------------------- | :-------------- | :------------------ | :------------------------- | :--------- | :--------------- | :------------------------------------------------------------------------------------------ |
| src/lib/views/BatchOperationsView.svelte   | 91.97%          | 92.30%              | 73.74%                     | 2026-09-15 | [PENDING REVIEW] |                                                                                             |
| src/lib/views/BrowseView.svelte            | 92.96%          | 91.94%              | 75.47%                     | 2026-09-15 | [ACCEPTED]       | Residual branch gaps in duplicate-detection / folder filter paths                           |
| src/lib/views/DesignDetailView.svelte     | 97.49%          | 90.81%              | 80.37%                     | 2026-09-04 | [ACCEPTED]       | Retained for completeness; branch >= 80%                                                    |
| src/lib/views/DesignDetailView.svelte     | 97.50%          | 90.72%              | 82.13%                     | 2026-09-15 | [ACCEPTED]       | Retained for completeness; all metrics >= 80%                                               |
| src/lib/views/ImportView.svelte           | 95.93%          | 95.65%              | 78.85%                     | 2026-09-04 | [ACCEPTED]       | Residual branch gaps in import wizard route/modal states                                    |
| src/lib/views/ImportView.svelte           | 93.24%          | 93.29%              | 75.59%                     | 2026-09-15 | [PENDING REVIEW] |                                                                                             |
| src/lib/views/ProjectsView.svelte         | 98.57%          | 93.84%              | 85.64%                     | 2026-09-04 | [ACCEPTED]       | Retained for completeness; all metrics >= 80%                                               |
| src/lib/views/ProjectsView.svelte         | 98.61%          | 94.20%              | 85.45%                     | 2026-09-15 | [ACCEPTED]       | Retained for completeness; all metrics >= 80%                                               |
| src/lib/views/ReferenceDataView.svelte     | 93.33%          | 94.11%              | 72.22%                     | 2026-09-15 | [PENDING REVIEW] |                                                                                             |
| src/lib/views/SettingsView.svelte         | 90.50%          | 96.00%              | 80.34%                     | 2026-09-15 | [ACCEPTED]       | Residual branch gaps in settings sections/toggles; all metrics >= 80%                       |
| src/lib/views/SystemMaintenanceView.svelte | 92.85%          | 93.75%              | 68.75%                     | 2026-09-15 | [PENDING REVIEW] |                                                                                             |
| src/lib/views/TaggingActionsView.svelte   | 92.92%          | 98.55%              | 76.56%                     | 2026-09-04 | [ACCEPTED]       | Residual branch gaps in mode/action UI states                                               |
| src/lib/views/TagsView.svelte             | 100.00%         | 100.00%             | 85.71%                     | 2026-09-15 | [ACCEPTED]       | Retained for completeness; all metrics >= 80%                                               |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:

1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.
