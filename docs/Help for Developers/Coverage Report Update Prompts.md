Run `cargo llvm-cov --summary-only` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md for rust modules:

1. Identify all Rust modules with Line, Function, or Region coverage below 80%.
2. Update the Backend table structure to include distinct columns for Line %, Function %, and Region %.
3. For any newly appearing module NOT currently listed, add a row with:
   - Module / File Path
   - Line Coverage %
   - Function Coverage %
   - Region Coverage %
   - Current Date (YYYY-MM-DD) — use the machine's actual current date, never a remembered/stale one.
   - "[PENDING REVIEW]" in the Status Column.
4. For modules already listed, use HISTORY-PRESERVING updates:
   - Compare the new measurement against the module's existing row(s).
   - If an existing (historical) row is better than the new measurement in ANY one metric, KEEP that better row and INSERT a new row directly below it with the latest metrics (Line %, Function %, Region %), today's date (YYYY-MM-DD), and "[PENDING REVIEW]" in Status, so I can compare progress. Never delete a row that is better than the latest measurement on any metric.
   - Only when the new measurement is equal-or-better than the existing row in ALL metrics, overwrite that row in place with the latest metrics and today's date (no extra row is needed). Set the status to [ACCEPTED] and write a reason for the acceptance.
5. Order rows by module including file path. When a module has multiple rows, keep them contiguous with the newest directly below its previous row.
6. Do NOT attempt to write, modify, or generate unit tests for any files. Do not write or edit source code files—only update COVERAGE_EXCEPTIONS.md.

---

Run `npx vitest run --coverage` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the Svelte frontend codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md for svelte modules:

1. Do NOT rely on the istanbul terminal "text" table: it truncates long file paths and its "Lines" value cannot be recomputed from the statement data. Emit a machine-readable report and read the per-file percentages from it, e.g.:
   `npx vitest run --coverage --coverage.reporter=json-summary --coverage.reportsDirectory=<tmp>` → parse `<tmp>/coverage-summary.json`; map `lines.pct` → Line %, `functions.pct` → Function %, `branches.pct` → Branch %. If the run exceeds your command window (Windows), run it as a detached/redirected background process and poll, or ask the user to run it; clean up `<tmp>` afterwards.
2. Identify all Svelte view modules (the `.svelte` views under frontend/src/lib/views/, plus lib-root MainView.svelte and InitialSetupView.svelte; exclude **tests**/**mocks**/test-harness files and non-view files such as components, stores, utils, services, types).
3. Apply rules 3-6 from the "Instructions for updating COVERAGE_EXCEPTIONS.md for rust modules" section above to the Frontend table (which keeps distinct Line %, Function %, and Branch/Region % columns), including the history-preserving update.

---

Run `npm run e2e:coverage` (or `npm run e2e`) and update COVERAGE_EXCEPTIONS.md to reflect the current status of the Playwright E2E integration test suite.

Instructions for updating COVERAGE_EXCEPTIONS.md for Playwright E2E coverage:

1. **Prerequisites & Build:**
   - Ensure the Tauri debug application binary is freshly compiled before running E2E tests:
     `npm run e2e:build` (runs `cargo tauri build --debug --no-bundle`).
2. **Execution & Coverage Collection:**
   - Run the full E2E test suite with coverage collection enabled:
     `npm run e2e:coverage` (or `npx playwright test`).
   - Playwright automatically attaches to the running desktop app over WebView2 Chrome DevTools Protocol (CDP) and records V8 byte-level JavaScript coverage via `monocart-coverage-reports`.
3. **Extracting E2E View Coverage Metrics:**
   - Read the per-view module percentages from the generated HTML reports:
     - Root views (`MainView.svelte`, `InitialSetupView.svelte`, `DatabaseRecoveryView.svelte`): inspect `coverage/e2e/html/lib/index.html`.
     - Sub-views (under `frontend/src/lib/views/`): inspect `coverage/e2e/html/lib/views/index.html`.
   - Map columns: `Lines %` → Line Coverage %, `Functions %` → Function Coverage %, `Branches %` → Branch Coverage %, `Statements %` → Statement Coverage %.
4. **Updating the E2E Section in `COVERAGE_EXCEPTIONS.md`:**
   - Identify all Svelte view modules with Line, Function, or Branch coverage below 80% (along with modules retained for completeness).
   - Locate the `## Frontend End-to-End Integration Coverage (Playwright CDP)` table.
   - For all Svelte view modules, update or insert rows with:
     - Svelte View Module (`src/lib/...`)
     - Line Coverage %
     - Function Coverage %
     - Branch Coverage %
     - Statement Coverage %
     - Current Date (YYYY-MM-DD) — use the machine's actual current date.
     - Status: `[ACCEPTED]` or `[PENDING REVIEW]`.
     - Reason Accepted / Integration Notes (briefly describing the primary user journeys and integration surfaces exercised by the E2E specs).
   - Apply history-preserving rules (rules 3-5 from the Rust instructions) to maintain comparison rows whenever prior measurements were higher in any metric.
5. **Substantiating Unit Exceptions:**
   - Cross-reference the E2E integration coverage with the unit-level `## Frontend (@Svelte Modules)` table in `COVERAGE_EXCEPTIONS.md` to substantiate acceptance reasons for views whose residual branch gaps (e.g., live IPC progress event streams, modal dialogues, routing transitions) are thoroughly validated by live end-to-end integration tests.
6. **Formatting & Cleanup:**
   - Run `npx prettier --write docs/policies/testing/COVERAGE_EXCEPTIONS.md` to ensure table alignment and markdown compliance.

---

Prompt to update tests to increase coverage:
@ModuleName has a NumberHere% function/line/region coverage. Can it be improved? Use information in @/.clinerules for information on how to write the tests. Explain your reasons if the coverage should be under 100%. If you changed a test, update @/docs/policies/testing/COVERAGE_EXCEPTIONS.md with the new coverage at the end of the task.

For Antigravity:
You can see the coverage details for @ModuleName in [COVERAGE_EXCEPTIONS.md](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/policies/testing/COVERAGE_EXCEPTIONS.md). Can we improve the coverage? I don't want to add tests for the sake of it. If we can increase the coverage, please write a plan. The plan should include updating [COVERAGE_EXCEPTIONS.md](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/policies/testing/COVERAGE_EXCEPTIONS.md) using the rules in [Coverage Report Update Prompts.md](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/Help%20for%20Developers/Coverage%20Report%20Update%20Prompts.md).

If we cannot improve the coverage, please update [COVERAGE_EXCEPTIONS.md](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/policies/testing/COVERAGE_EXCEPTIONS.md) to say [ACCEPTED] and add the reason why.

