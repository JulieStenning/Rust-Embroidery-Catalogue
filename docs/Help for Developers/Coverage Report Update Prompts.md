Run `cargo llvm-cov --summary-only` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md for rust modules:
1. Identify all Rust modules with Line, Function, or Region coverage below 80%.
2. Update the Backend table structure to include distinct columns for Line %, Function %, and Region %.
3. For any newly appearing low-coverage module NOT currently listed, add a row with:
   - Module / File Path
   - Line Coverage %
   - Function Coverage %
   - Region Coverage %
   - Current Date (YYYY-MM-DD) — use the machine's actual current date, never a remembered/stale one.
   - "[PENDING REVIEW]" in the Status Column.
4. For modules already listed, use HISTORY-PRESERVING updates:
   - Compare the new measurement against the module's existing row(s).
   - If an existing (historical) row is better than the new measurement in ANY one metric, KEEP that better row and INSERT a new row directly below it with the latest metrics (Line %, Function %, Region %), today's date (YYYY-MM-DD), and "[PENDING REVIEW]" in Status, so I can compare progress. Never delete a row that is better than the latest measurement on any metric.
   - Only when the new measurement is equal-or-better than the existing row in ALL metrics, overwrite that row in place with the latest metrics and today's date (no extra row is needed).
5. Order rows by module including file path. When a module has multiple rows, keep them contiguous with the newest directly below its previous row.
6. Retention: once a module is listed, KEEP it in the table (do NOT remove it) even when its coverage reaches or rises above 80% in every metric, so the log remains a complete picture. New modules are only ADDED when they first fall below 80% on any metric; healthy-but-unlisted modules are not added.
7. Do NOT attempt to write, modify, or generate unit tests for any files. Do not write or edit source code files—only update COVERAGE_EXCEPTIONS.md.

-------
Run `npx vitest run --coverage` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the Svelte frontend codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md for svelte modules:
1. Do NOT rely on the istanbul terminal "text" table: it truncates long file paths and its "Lines" value cannot be recomputed from the statement data. Emit a machine-readable report and read the per-file percentages from it, e.g.:
   `npx vitest run --coverage --coverage.reporter=json-summary --coverage.reportsDirectory=<tmp>` → parse `<tmp>/coverage-summary.json`; map `lines.pct` → Line %, `functions.pct` → Function %, `branches.pct` → Branch %. If the run exceeds your command window (Windows), run it as a detached/redirected background process and poll, or ask the user to run it; clean up `<tmp>` afterwards.
2. Identify all Svelte view modules (the `.svelte` views under frontend/src/lib/views/, plus lib-root MainView.svelte and InitialSetupView.svelte; exclude __tests__/__mocks__/test-harness files and non-view files such as components, stores, utils, services, types) that have ANY of Line %, Function %, or Branch % below 80%.
3. Apply rules 3-7 from the "Instructions for updating COVERAGE_EXCEPTIONS.md for rust modules" section above to the Frontend table (which keeps distinct Line %, Function %, and Branch/Region % columns), including the history-preserving update and module-retention rules.

------
Prompt to update tests to increase coverage
@ModuleName has a NumberHere% function/line/region coverage. Can it be improved? Use information in @/.clinerules for information on how to write the tests. Explain your reasons if the coverage should be under 100%. If you changed a test, update @/docs\policies\testing\COVERAGE_EXCEPTIONS.md with the new coverage at the end of the task. 