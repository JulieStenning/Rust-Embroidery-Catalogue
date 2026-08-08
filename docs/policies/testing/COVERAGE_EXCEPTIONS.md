# Test Coverage Exceptions Log

This file tracks modules, frontend components, and backend logic where test coverage is lower than standard thresholds, but has been reviewed and accepted as sufficient.

**Rule for AI Agents:** Do not attempt to add or generate unit tests for files or functions listed here unless explicitly instructed.

---

## Backend (Rust / Tauri)


| Module / File Path               | Function Coverage | Line Coverage | Region Coverage | Date Accepted | Reason / Rationale                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| :------------------------------- | :---------------- | :------------ | :-------------- | :------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| src/services/db_health.rs        | 80.00%            | ~65%          | ~65%            | 2026-08-08    | [ACCEPTED]                                                                                                                                                                      | src/main.rs                      | ~57%              | ~65%          | ~65%            | 2026-08-08    | [ACCEPTED] 
| src/routes/designs.rs            | ~75%              | ~75%          | ~75%            | 2026-08-08    | [ACCEPTED] 
| src/services/about_documents.rs  | 91.67%            | 73.33%        | 73.47%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/routes/maintenance.rs        | ~65%              | ~75%          | ~78%            | 2026-08-08    | [ACCEPTED] 
| src/readers/vp3_reader.rs        | 100%              | 75.21%        | 74.32%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/error.rs                     | 80.00%            | 79.25%        | 75.38%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/utils.rs                     | 75.00%            | 90.00%        | 84.62%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/logging.rs                   | 75.00%            | 86.89%        | 86.52%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/services/backfill.rs         | 79.42%            | 89.87%        | 87.47%          | 2026-08-08    | [ACCEPTED] 
| src/services/maintenance.rs      | 66.67%            | 95.24%        | 89.33%          | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                                                                                                                                                                                                                                                                    
| src/routes/tagging_actions.rs    | 67.39%            | 94.88%        | 91.67%          | 2026-08-08    | [ACCEPTED] 
| src/routes/admin.rs              | 78.81%            | 94.25%        | 93.91%          | 2026-08-08    | [ACCEPTED] 
| src/services/fingerprint.rs      | 68.92%            | 94.66%        | 94.00%          | 2026-08-08    | [ACCEPTED] 
| src/routes/projects.rs           | 0%                | 0%            | 0%              | 2026-08-08    | [ACCEPTED]                                                                                                                                                                                                              |

---

## Frontend (@Svelte Modules)

| Module / File Path                     | Line Coverage % | Function Coverage % | Branch / Region Coverage % | Date       | Reason / Rationale |
| :------------------------------------- | :-------------- | :------------------ | :------------------------- | :--------- | :----------------- |
| src/lib/views/BrowseView.svelte        | 92.45%          | 89.91%              | 71.55%                     | 2026-08-08 | [ACCEPTED]         |
| src/lib/views/DesignDetailView.svelte  | 90.34%          | 83.80%              | 67.42%                     | 2026-08-08 | [ACCEPTED]         |
| src/lib/views/ImportView.svelte        | 96.18%          | 94.20%              | 77.26%                     | 2026-08-08 | [ACCEPTED]         |
| src/lib/views/OrphansView.svelte       | 96.09%          | 100%                | 79.54%                     | 2026-08-08 | [ACCEPTED]         |
| src/lib/views/ProjectsView.svelte      | 96.76%          | 93.93%              | 69.81%                     | 2026-08-08 | [ACCEPTED]         |
| src/lib/views/TagsView.svelte          | 86.66%          | 100%                | 67.34%                     | 2026-08-08 | [ACCEPTED]         |

---

## Exclusion Guidelines

A file or function qualifies for coverage exception if it falls into one of these categories:

1. **Thin Glue Code:** Tauri `#[tauri::command]` handlers that simply pass parameters to Rust service functions.
2. **Visual Layout / Print Views:** Components dedicated primarily to print styling or complex CSS layout.
3. **Dev Tooling:** Test harnesses, inspector overlays, or debugging views (e.g., @Inspector.svelte, @ImportTestHarness.svelte).
4. **Third-Party / OS Interfaces:** File dialogs, SQLite migration execution, or external binary reader bindings.