# Import Test-Spec Refresh — Implementation Plan

**Date:** 2026-09-14
**Feature:** Re-align the Import user test plans (`docs/User Test Plans/Import.md`,
`Bulk Import - Review Scanned Files.md`, `Bulk Import - Pre-Import Actions.md`) with
the shipped UI, and extend the Playwright e2e coverage to match.

---

## 1. Why this plan exists

The Import user test plans were written against an earlier design in which the import
wizard also performed **AI tagging** (Tier 2 / Tier 3), offered a **per-session 2D/3D
preview preference**, and exposed **Review Hoops / Tags / Sources / Designers** links
from step 3.

All of that has since been removed. Import now runs **File & Folder Rules only**
(offline, no API key). Visual AI tagging is a separate, later operation on the
**Batch Operations** page. The specs therefore describe controls that no longer exist,
which makes them unusable as manual test scripts (a tester cannot "pass" a checklist
item for a button that isn't there) and useless as a contract for the e2e suite.

---

## 2. Current UI contract (evidence)

All references are to `frontend/src/lib/views/ImportView.svelte` unless stated otherwise.

| Spec claim | Reality today | Location |
|---|---|---|
| Step 3 shows an AI-tagging banner (API key present, rate limits, `Tier 2 auto: on`, "Change in Settings") | Removed. Step 3 shows one static blue note: "Initial import uses fast, offline File & Folder Rules to index your designs instantly. Once finished, you can run automated Visual AI tagging anytime from Batch Operations to enrich your collection." | `:1581-1586` |
| Step 3 has a 2D/3D preview-preference radio group with "(Saved setting: 3D)" | Removed entirely (no hits anywhere in `frontend/src`) | grep |
| Step 3 has Review Hoops / Tags / Sources / Designers + "Continue with import" | Removed. Step 3 has `Import Designs` and `Cancel` only | `:1588-1619` |
| Button label "Browse..." | Actual label is `Browse…` (single U+2026 ellipsis) | `:1167`, `:1202` |
| "Reset" active by default | Disabled until at least one folder path is present | `:1240-1250` |
| "Add another folder" active by default | Disabled until the primary path is non-empty | `:1218-1228` |
| "Remove" disabled because only one required row exists | Primary `Remove` is disabled only while its input is empty; pressing it with no extra rows just clears the input. Extra rows always have an enabled `Remove` | `:1169-1181`, `:970-977` |
| Additional rows are typable | Additional rows are `readonly`; they are populated by Browse… / multi-select | `:1189-1195` |
| Non-existent path shows an inline step-1 validation error | The scan always navigates to step 2, which renders an amber "No supported files discovered in this preview." panel with a diagnostic message and **Back to Step 1** | `:783-812`, `:1540-1557` |
| Supported formats `.jef`, `.pes`, `.vp3` | `jef, pes, hus, dst, exp, vp3` (`services/scanning.rs:10`) | `:837` echoes that list |
| Leaving mid-flow prompts a dirty-flag warning | No prompt. Navigation is unconditional; wizard state survives via `importSessionStore` and is restored on return | `importSessionStore.ts`, `:110-160` |
| Step 2 dropdowns offer Keep inferred / Choose existing / **Create new** / **Leave blank** | Plain `<select>`: the inferred option plus existing Designer/Source records. No "Create new" or "Leave blank" | `:1289-1306`, `:1437-1462` |
| Step 2 "Cancel" rolls back to the review stage | Step-2 `Cancel` → `#/import/step1`; step-3 `Cancel` → full `resetImportWizard()` | `:1333`, `:696-698` |

Additional behaviour worth documenting (verified in `src/routes/bulk_import.rs`):

- Scan is **recursive** and **case-insensitively de-duplicated** within a scan
  (`services/scanning.rs:64-90`).
- The preview **filters out files already in the catalogue**, matched on the
  prospective stored `filepath`, or on the `(filename, file_size_bytes, blake3)`
  triple (`bulk_import.rs:2150-2200`). The "N file(s) found" count is therefore the
  count of *new* files, not raw files on disk.
- Routes: `#/import` and `#/import/step1` are equivalent (`parseImportWizardStep`, `:115-119`).

---

## 3. Deliverables

1. `docs/Plans/Import test spec refresh.md` — this document.
2. Rewritten `docs/User Test Plans/Import.md` (step 1).
3. Updated `docs/User Test Plans/Bulk Import - Review Scanned Files.md` (step 2).
4. Rewritten `docs/User Test Plans/Bulk Import - Pre-Import Actions.md` (step 3).
5. Stale-copy fix in `docs/User-Facing-Guidance/FIRST_IMPORT_ACTIONS.md`.
6. Harness helpers: `prepareEmptyImportSource()` / `prepareMixedImportSource()` in
   `tests/e2e/empty-root.ts`; `resetImportView()`, `folderRows()`,
   `runImportToPrecheck()` in `tests/e2e/helpers.ts`.
7. New spec `tests/e2e/import-folder-selection.spec.ts` (step 1).
8. Extended `tests/e2e/import.spec.ts` (step 2 + step 3 assertions).
9. `docs/Help for Developers/e2e testing guide.md` updated with the new spec names.

---

## 4. Test-suite design

Every step-1 test is **read-only** (no scan that mutates the catalogue), so it is safe
against the shared, single-worker data root. Because the app instance is long-lived and
`importSessionStore` is module-level state, each test begins with `resetImportView(page)`
(navigate to `#/import`; click `Reset` when enabled, otherwise clear the input).

Step-2/3 assertions are folded into the existing import run in `import.spec.ts` rather
than adding a second expensive scan+copy+DB-write cycle.

### Negative assertions (the deletion guard)

Per the project's "write the must-NOT-happen test" rule, the suite asserts the absence of
the removed UI on step 3: `Tier 2 auto`, `Change in Settings`, `2D - Fast flat preview`,
`Review Hoops`. This is what stops the obsolete banners silently reappearing.

---

## 5. Verification

1. `npx tsc --noEmit -p tsconfig.json` (root project covers `tests/e2e/**`).
2. `npm run e2e:build` (backgrounded; revert the `generate:licences` side-effect with
   `git checkout -- src/assets/licences.html frontend/src/lib/assets/licences.html`).
3. `npx playwright test tests/e2e/import-folder-selection.spec.ts`
4. `npx playwright test tests/e2e/import.spec.ts tests/e2e/import-hoop-setup.spec.ts`
5. Full `npm run e2e` as the final gate.

### Known limitation

Native folder-picker behaviour (multi-select, session-memory start folder,
overwrite-by-re-selection) cannot be driven by Playwright — it needs an OS dialog. Those
spec items are re-labelled as unit-only (`ImportView.test.ts` → *browse flows*) rather
than deleted.

