Decision noted: route-level `restore_both` ordering tests stay out of scope. Section 4.1 and 6.3 move into the **non-automatable** list.

Here is the **finalized plan**, ready for implementation once you toggle to Act mode.

---

## Final Plan: Unit Tests for "Restore Workflow.md"

### Deliverable 1 — New/updated test files

**Rust — `src/services/restore_tests.rs` (extend, service-level only)**

| Test | Covers user-test section |
|---|---|
| `perform_database_restore_rolls_back_on_corrupt_file` | 7.2–7.5, 7.7 |
| `perform_database_restore_retains_named_rollback_copy` | 2.5, 2.6 |
| `perform_database_restore_reports_schema_version_hints` | 8.1–8.3 |
| `perform_designs_restore_errors_on_missing_source` | 6.1 |
| `import_unmatched_design_files_imports_real_design` (uses `tests/Test Designs/Bean.pes`) | 5.6, 5.7 |
| *(optional)* `perform_designs_restore_honours_cancel_flag` | — |

**Frontend — `BackupView.test.ts` (extend `restore` block)**

| Test | Covers |
|---|---|
| cancel file picker → no error/no crash | 1.4 |
| picked path displayed in readonly input | 1.5 |
| schema-version-changed banner | 8.4 |
| rolled-back banner + error toast | 7.6 |
| invalid designs path → error toast | 6.2 |

**Frontend — new files**

| File | Covers |
|---|---|
| `frontend/src/lib/components/__tests__/RestoreProgressPanel.test.ts` | 2.3, 3.3, 4.5 |
| `frontend/src/lib/services/__tests__/restoreEvents.test.ts` (mirrors `dbMaintenanceEvents.test.ts`) | 2.3 |
| `frontend/src/lib/stores/__tests__/restoreProgressStore.test.ts` | — (store reset/initial) |

### Deliverable 2 — Automatable list
All items except the three below (consolidated): Section 1.1/1.2/1.4/1.5, 2.2–2.6, 3.2–3.6, 4.2–4.5, 5.3–5.7, 6.1–6.2, 7.2–7.7, 8.1–8.4.

### Deliverable 3 — Non-automatable list

| # | Item | Reason |
|---|---|---|
| Setup | Working Backup Location / valid DB snapshot | Real external filesystem provisioning |
| 1.3 | `.db` file-extension filter in picker | Native `rfd` dialog filter not observable in unit tests (validation logic already covered) |
| 2.6 | Full `@MainView.svelte` browse render after restore | Cross-view integration beyond unit scope (DB count asserted in Rust) |
| **4.1** | DB-before-designs ordering in `restore_both` | Route command needs stubbed `AppState` — out of scope per your decision |
| **6.3** | Restore Both with invalid designs path (exact ordering) | Same route-level limitation |
| 4.2 | "Seamless transition" (visual) | Subjective/visual |

### Discrepancies to resolve during implementation
1. Unmatched banner uses inline `data-testid="unmatched-files-prompt"`, not `@Notice.svelte` — tests target the `data-testid`.
2. Button label is "Import N file(s)", not "Import Unmatched Files" — tests use `/Import \d+ file/`.
3. Designs toast omits "scanned" (Section 3.6) — I'll assert the *current* "`X copied, Y skipped`" message unless you want the implementation changed.

### Verification (final gate)
- `cargo check` → `cargo test restore` → full `cargo test`
- Repo-root `npx vitest run` for the affected suites
- `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"`
- `rustfmt --edition 2021 src/services/restore_tests.rs`

---

When you're ready, **toggle to Act mode** and I'll implement the tests exactly as planned.