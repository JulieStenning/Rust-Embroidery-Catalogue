# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient.

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)


| Module / File Path                | Line % | Function % | Region % | Date       | Status           | Reason Accepted                                               |
| :-------------------------------- | :----- | :--------- | :------- | :--------- | :--------------- | :------------------------------------------------------------ |
| src/database/migrations.rs        | 88.37% | 100.00%    | 96.67%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/logging.rs                    | 92.47% | 90.00%     | 93.53%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem; all metrics >= 80%                             |
| src/main.rs                       | 37.84% | 42.31%     | 35.33%   | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/paths.rs                      | 84.21% | 77.55%     | 86.17%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/routes/admin.rs               | 92.63% | 79.82%     | 86.78%   | 2026-09-15 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/routes/admin.rs               | 91.82% | 77.54%     | 85.25%   | 2026-09-24 | [ACCEPTED]       | Thin glue / Tauri command wrappers; core database CRUD tested |
| src/routes/batch_operations.rs    | 77.27% | 51.79%     | 69.27%   | 2026-09-24 | [ACCEPTED]       | OS file picker dialog + live Tauri event emitter glue; core batch dispatch fully tested in services |
| src/routes/bulk_import.rs         | 84.48% | 74.09%     | 82.46%   | 2026-09-15 | [ACCEPTED]       | Thin glue and Internet tests for Gemini                       |
| src/routes/bulk_import.rs         | 83.96% | 73.44%     | 82.17%   | 2026-09-24 | [ACCEPTED]       | Tauri IPC event streaming and online Gemini Vision integration |
| src/routes/database_recovery.rs   | 92.68% | 57.14%     | 86.79%   | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/designs.rs             | 68.33% | 42.37%     | 67.16%   | 2026-09-15 | [ACCEPTED]       | Thin glue / Tauri command wrappers + event emission + OS file launching; all core DB queries and mutations fully tested |
| src/routes/designs.rs             | 68.25% | 42.91%     | 66.64%   | 2026-09-24 | [ACCEPTED]       | Thin glue / Tauri command wrappers + event emission + OS file launching; all core DB queries and mutations fully tested |
| src/routes/maintenance.rs         | 48.82% | 39.01%     | 49.92%   | 2026-09-15 | [ACCEPTED]       | Thin glue / Tauri command wrappers + OS backup I/O + native folder picker/Explorer launches; core orphan scanning & backup logic fully tested |
| src/routes/maintenance.rs         | 48.72% | 39.01%     | 49.92%   | 2026-09-24 | [ACCEPTED]       | Thin glue / Tauri command wrappers + OS backup I/O + native folder picker/Explorer launches; core orphan scanning & backup logic fully tested |
| src/routes/projects.rs            | 0.00%  | 0.00%      | 0.00%    | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/restore.rs             | 6.25%  | 13.79%     | 5.88%    | 2026-09-15 | [ACCEPTED]       | Thin glue / Tauri IPC event streaming + pool-swap gate + OS file dialog; all core restore & reconciliation logic fully tested in services::restore |
| src/routes/restore.rs             | 5.85%  | 13.79%     | 5.75%    | 2026-09-24 | [ACCEPTED]       | Thin glue / Tauri IPC event streaming + pool-swap gate + OS file dialog; all core restore & reconciliation logic fully tested in services::restore |
| src/routes/settings.rs            | 75.56% | 56.82%     | 67.59%   | 2026-09-24 | [ACCEPTED]       | Thin glue / Tauri command wrappers + OS folder picker dialog; core settings persistence & Gemini validation fully tested in services |
| src/routes/storage_migration.rs   | 8.00%  | 7.14%      | 4.67%    | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/routes/tagging_actions.rs     | 75.94% | 48.94%     | 68.12%   | 2026-09-03 | [ACCEPTED]       | Thin glue/Tauri framework limitation.OS/filesystem interfaces |
| src/services/about_documents.rs   | 92.73% | 94.12%     | 96.20%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/about_documents.rs   | 98.10% | 92.86%     | 98.81%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem; all metrics >= 90%                             |
| src/services/admin.rs             | 94.53% | 61.43%     | 81.31%   | 2026-09-24 | [ACCEPTED]       | High line coverage; DB error paths + dead branches            |
| src/services/auto_tagging.rs      | 94.94% | 84.38%     | 91.86%   | 2026-09-24 | [ACCEPTED]       | All metrics >= 80%                                            |
| src/services/backfill.rs          | 85.41% | 67.11%     | 81.36%   | 2026-09-15 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/backfill.rs          | 85.20% | 66.01%     | 81.67%   | 2026-09-24 | [ACCEPTED]       | High line/region coverage (>=81%); function gap due to async stream closures and event dispatch |
| src/services/db_health.rs         | 85.47% | 91.89%     | 80.34%   | 2026-09-15 | [ACCEPTED]       | All metrics >= 80%                                            |
| src/services/db_health.rs         | 91.15% | 90.91%     | 89.86%   | 2026-09-24 | [ACCEPTED]       | All metrics >= 89%                                            |
| src/services/design_metadata.rs   | 100.00%| 100.00%    | 100.00%  | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/services/fingerprint.rs       | 92.35% | 72.00%     | 90.94%   | 2026-09-24 | [ACCEPTED]       | High line/region coverage (>=90%); function gap due to compiler-derived struct traits; OS/Filesystem |
| src/services/gemini_client.rs     | 94.57% | 92.06%     | 94.37%   | 2026-09-24 | [ACCEPTED]       | Network client; injectable-base HTTP mock tests               |
| src/services/maintenance.rs       | 97.78% | 83.33%     | 93.75%   | 2026-09-24 | [ACCEPTED]       | Thin glue                                                     |
| src/services/projects.rs          | 96.09% | 74.36%     | 85.68%   | 2026-09-24 | [ACCEPTED]       | Derive artifacts + high line coverage                         |
| src/services/restore.rs           | 85.90% | 79.55%     | 83.42%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem; function gap due to error branches             |
| src/services/settings.rs          | 89.74% | 71.43%     | 83.94%   | 2026-09-24 | [ACCEPTED]       | Derive artifacts + high line coverage + OS/Filesystem         |
| src/services/storage_migration.rs | 72.21% | 60.71%     | 69.40%   | 2026-09-24 | [ACCEPTED]       | OS/Filesystem                                                 |
| src/services/tag_synonyms.rs      | 92.12% | 62.50%     | 83.43%   | 2026-09-24 | [ACCEPTED]       | High line/region coverage (>=83%); function gap due to compiler-derived struct traits |
| src/utils.rs                      | 90.00% | 75.00%     | 84.62%   | 2026-09-24 | [ACCEPTED]       | Dev tooling                                                   |

---

## Frontend (@Svelte Modules)


| Module / File Path                         | Line Coverage % | Function Coverage % | Branch / Region Coverage % | Date       | Status     | Reason Accepted                                                                                      |
| :----------------------------------------- | :-------------- | :------------------ | :------------------------- | :--------- | :--------- | :--------------------------------------------------------------------------------------------------- |
| src/lib/DatabaseRecoveryView.svelte        | 96.52%          | 100.00%             | 78.35%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>=96%); branch gaps in fallback error messaging & confirm dialog states |
| src/lib/views/BatchOperationsView.svelte   | 91.97%          | 92.30%              | 73.74%                     | 2026-09-15 | [ACCEPTED] | High line/function coverage (>91%); residual branch gaps in live progress & cancel modal UI states  |
| src/lib/views/BatchOperationsView.svelte   | 91.33%          | 90.19%              | 74.63%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>90%); residual branch gaps in live progress & cancel modal UI states  |
| src/lib/views/BrowseView.svelte            | 92.97%          | 92.01%              | 75.47%                     | 2026-09-24 | [ACCEPTED] | Residual branch gaps in duplicate-detection / folder filter paths                                    |
| src/lib/views/DesignDetailView.svelte     | 97.50%          | 90.72%              | 82.13%                     | 2026-09-15 | [ACCEPTED] | Retained for completeness; all metrics >= 80%                                                        |
| src/lib/views/DesignDetailView.svelte     | 97.49%          | 90.62%              | 81.79%                     | 2026-09-24 | [ACCEPTED] | Retained for completeness; all metrics >= 80%                                                        |
| src/lib/views/ImportView.svelte           | 94.89%          | 94.44%              | 76.78%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>94%); residual branch gaps in wizard navigation & modal states          |
| src/lib/views/ProjectsView.svelte         | 98.61%          | 94.20%              | 85.45%                     | 2026-09-24 | [ACCEPTED] | Retained for completeness; all metrics >= 80%                                                        |
| src/lib/views/ReferenceDataView.svelte     | 93.75%          | 94.11%              | 75.00%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>93%); container tab switcher branches; child views fully tested        |
| src/lib/views/SettingsView.svelte         | 90.50%          | 96.00%              | 80.34%                     | 2026-09-15 | [ACCEPTED] | Residual branch gaps in settings sections/toggles; all metrics >= 80%                                |
| src/lib/views/SettingsView.svelte         | 89.94%          | 90.56%              | 80.34%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (~90%); all metrics >= 80%                                               |
| src/lib/views/SystemMaintenanceView.svelte | 92.85%          | 93.75%              | 68.75%                     | 2026-09-15 | [ACCEPTED] | High line/function coverage (>92%); residual branch gaps in compaction confirmation & warning badges |
| src/lib/views/SystemMaintenanceView.svelte | 100.00%         | 100.00%             | 81.25%                     | 2026-09-24 | [ACCEPTED] | Tab navigation and busy guard states fully covered; all metrics >= 80%                               |
| src/lib/views/TagWordMatchesView.svelte    | 93.45%          | 94.28%              | 68.70%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>=93%); branch gap in search filter/toggle variants & focus callbacks   |
| src/lib/views/TaggingActionsView.svelte   | 92.92%          | 98.55%              | 76.56%                     | 2026-09-04 | [ACCEPTED] | Residual branch gaps in mode/action UI states                                                        |
| src/lib/views/TagsView.svelte             | 100.00%         | 100.00%             | 85.71%                     | 2026-09-15 | [ACCEPTED] | Retained for completeness; all metrics >= 80%                                                        |
| src/lib/views/TagsView.svelte             | 91.30%          | 87.50%              | 77.04%                     | 2026-09-24 | [ACCEPTED] | High line/function coverage (>87%); branch gaps in modal edit/delete states                          |

---

## Frontend End-to-End Integration Coverage (Playwright CDP)

The following table reflects live V8 Chrome DevTools Protocol (CDP) coverage collected across running desktop app sessions via `npm run e2e:coverage` (report generated in `coverage/e2e/html/index.html`).

| Svelte View Module                         | Line Coverage % | Function Coverage % | Branch Coverage % | Statement Coverage % | Date       | Status           | Reason Accepted / Integration Notes                                                                                           |
| :----------------------------------------- | :-------------- | :------------------ | :---------------- | :------------------- | :--------- | :--------------- | :---------------------------------------------------------------------------------------------------------------------------- |
| src/lib/MainView.svelte                    | 93.75%          | 89.83%              | 89.83%            | 93.65%               | 2026-09-24 | [ACCEPTED] | Core router shell, footer links, header title, navigation history roundtrips; all metrics >= 80%                              |
| src/lib/InitialSetupView.svelte            | 25.71%          | 35.00%              | 20.00%            | 26.08%               | 2026-09-24 | [ACCEPTED] | First-run setup wizard bypassed in E2E by test fixtures; fully tested in unit tests (93.4% Line, 92.6% Function)              |
| src/lib/DatabaseRecoveryView.svelte        | 0.00%           | 0.00%               | 0.00%             | 0.00%                | 2026-09-24 | [ACCEPTED] | Disaster recovery screen only mounts on corrupted DB startup; fully tested in unit tests (96.5% Line, 100% Function)          |
| src/lib/views/AboutDocumentView.svelte     | 53.96%          | 39.53%              | 41.17%            | 48.27%               | 2026-09-24 | [ACCEPTED] | Static document renderer for bundled licence/notice text; visual layout covered in unit tests                                 |
| src/lib/views/AboutView.svelte             | 0.00%           | 0.00%               | 100.00%           | 0.00%                | 2026-09-24 | [ACCEPTED] | Static informational view with external website links and version display; verified in unit tests                             |
| src/lib/views/AdminDesignersView.svelte    | 97.14%          | 100.00%             | 66.66%            | 95.50%               | 2026-09-24 | [ACCEPTED] | High line/function coverage (97.1% Line, 100% Function); remaining branch gaps are defensive whitespace/duplicate validation |
| src/lib/views/AdminHoopsView.svelte        | 97.59%          | 100.00%             | 74.60%            | 96.26%               | 2026-09-24 | [ACCEPTED] | High line/function coverage (97.6% Line, 100% Function); remaining branch gaps are custom dimension boundary clamp guards    |
| src/lib/views/AdminSourcesView.svelte      | 97.10%          | 100.00%             | 66.66%            | 95.40%               | 2026-09-24 | [ACCEPTED] | High line/function coverage (97.1% Line, 100% Function); residual branch gaps are empty search query fallbacks                |
| src/lib/views/BackupView.svelte            | 61.53%          | 88.09%              | 48.28%            | 63.24%               | 2026-09-24 | [ACCEPTED] | Core backup generation verified (88.1% Func); gaps are native file dialog aborts & restore prompts (covered in unit tests)    |
| src/lib/views/BatchOperationsView.svelte   | 74.00%          | 69.23%              | 52.68%            | 71.64%               | 2026-09-24 | [ACCEPTED] | Batch task triggers & live progress verified; stream cancellations & error rollbacks covered in unit tests (91.3% Line)       |
| src/lib/views/BrowseView.svelte            | 79.73%          | 83.83%              | 61.90%            | 79.31%               | 2026-09-24 | [ACCEPTED] | Strong line/func coverage (~80% Line, 83.8% Func); primary catalog browsing, tag/designer filters, and multi-select verified  |
| src/lib/views/DesignDetailView.svelte     | 86.59%          | 91.85%              | 67.37%            | 86.02%               | 2026-09-24 | [ACCEPTED] | High line/func coverage (86.6% Line, 91.9% Func); metadata viewer, thread table, and tag chips verified; gaps are null fields |
| src/lib/views/DesignPrintView.svelte       | 91.17%          | 84.61%              | 55.93%            | 90.38%               | 2026-09-24 | [ACCEPTED] | High line/func coverage (91.2% Line, 84.6% Func); visual print worksheet styling; qualifies under Exclusion Guideline 2       |
| src/lib/views/HelpView.svelte              | 100.00%         | 100.00%             | 100.00%           | 100.00%              | 2026-09-24 | [ACCEPTED] | In-app help documentation and keyboard shortcut guide; all metrics 100%                                                       |
| src/lib/views/ImportView.svelte           | 67.17%          | 68.61%              | 53.96%            | 66.50%               | 2026-09-24 | [ACCEPTED] | Multi-step import workflow & deduplication verified; folder dialog errors & malformed files covered in unit tests (94.9% Line)|
| src/lib/views/OrphansView.svelte           | 82.35%          | 83.33%              | 60.97%            | 80.00%               | 2026-09-24 | [ACCEPTED] | High line/func coverage (>82% Line/Func); orphan detection & reconciliation verified; gaps are empty state & modal aborts     |
| src/lib/views/ProjectsView.svelte         | 89.93%          | 90.43%              | 71.72%            | 86.95%               | 2026-09-24 | [ACCEPTED] | High line/func coverage (89.9% Line, 90.4% Func); project CRUD & export workflows verified; gaps are modal aborts             |
| src/lib/views/ReferenceDataView.svelte     | 100.00%         | 100.00%             | 84.21%            | 100.00%              | 2026-09-24 | [ACCEPTED] | Reference data container tab switching and sub-view navigation; all metrics >= 84%                                            |
| src/lib/views/SettingsView.svelte         | 55.70%          | 61.03%              | 48.94%            | 54.34%               | 2026-09-24 | [ACCEPTED] | Settings navigation, theme toggle, and path inspection verified; relocation side-effects & vacuum covered in unit tests (~90%)|
| src/lib/views/SystemMaintenanceView.svelte | 100.00%         | 100.00%             | 84.61%            | 100.00%              | 2026-09-24 | [ACCEPTED] | System maintenance container tab switching (Settings / Backup / Orphans) and busy state guard; all metrics >= 84%             |
| src/lib/views/TagWordMatchesView.svelte    | 34.00%          | 24.44%              | 21.55%            | 33.33%               | 2026-09-24 | [ACCEPTED] | View navigation verified in E2E; full synonym rule matrix & inline chip editing covered in Vitest unit tests (93.45% Line)    |
| src/lib/views/TagsView.svelte             | 87.27%          | 93.33%              | 67.27%            | 88.88%               | 2026-09-24 | [ACCEPTED] | High line/func coverage (87.3% Line, 93.3% Func); tag CRUD & category filters verified; gaps are duplicate name/modal aborts |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:

1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.

