# Flagged Import — Corrupt Files & Failed Preview Generations

> **Status:** Updated to the converged lean approach after plan review (supersedes the earlier
> heavyweight draft that proposed a schema migration and status/reason/mtime columns).
> **Owner:** Embroidery Catalogue (Rust/Tauri v2 + Svelte/TypeScript)
> **Rule:** No code changes until this plan is approved.

## 1. Core decision — use `image_data IS NULL` as the flag, no schema change

A design whose preview could not be generated is identified by `designs.image_data IS NULL`. We do
**not** add `preview_status`, `preview_failure_reason`, `preview_failure_category`, mtime, or any other
new column. No migration.

### Why NULL is a reliable signal (verified)
- Previews are stored as **BLOBs in SQLite and rendered from the DB**, never re-read from the
  filesystem on each display. So a file **deleted after import keeps its preview**; `image_data` does
  not become NULL when the file vanishes.
- A file **missing on disk at import is never imported** (bulk import copies the source first and
  errors before creating a row), so a missing file never yields a NULL-image row.
- The app has **no in-app path that deletes an image**, and previews are generated at import.

Conclusion: `image_data IS NULL` is only ever caused by a preview that was not successfully generated
at insert time — i.e. a decode/read failure. This is the corruption flag.

### Behaviour model
| State | Meaning | User action |
|---|---|---|
| `image_data` present | Preview generated and stored | none |
| `image_data IS NULL` | Preview could not be generated (file unreadable/corrupt, or a record inserted
  without a preview attempt) | Regenerate (Tagging Actions or Design Detail) and/or locate/replace the
  file |

### Known gap (deferred, out of scope for this change)
Any import path that inserts a design record must also generate the preview. Today the restore
reconciliation import (`import_single_design`, reached from the Backup/Restore "Import unmatched
file(s)" prompt) inserts a **successfully-parsed** file with `image_data NULL` because it never runs
preview generation. Fixing that path (generate the image on import) is **separate future work** the
user will handle later.

**For the purposes of this change we assume the image is regenerated** (the design already exists and
previews can be produced for it through Tagging Actions / Design Detail). This means the NULL = corrupt
flag is treated as valid for catalogues maintained through the normal import path.

## 2. Failure semantics (per import path)
- **Main bulk import:** a decode/read failure during preview generation does **not** skip or abort —
  the design row is still created with `image_data NULL` (plus NULL dims/counts). This retention is the
  desired flagged-import behaviour. The row stays in the catalogue and the user is told how to
  regenerate it.
- **Restore reconciliation import:** a read/parse failure currently returns an error and the file is
  skipped (no row). Out of scope for this change (see deferred gap above).

## 3. Backend changes (minimal)

### 3.1 Import result tallies (`src/routes/bulk_import.rs`)
Today a failed preview still counts as `persisted`, so the summary cannot tell the truth. Add a small
count (no schema):
- Count failures where `image_generation` returned no image (decode/read failure) inside
  `persist_bulk_import_confirm_wire`.
- Surface it on the **confirm result wire** the frontend reads (`confirm_result.failed_decode_count`)
  and on the **bulk-import-progress event** (`BulkImportProgressEvent.failed_count`).
- The row is still inserted with `image_data NULL` (unchanged) so the record is never dropped.

### 3.2 Tagging Actions regeneration (already works — no change required)
The "images" action in `run_unified_backfill` targets designs with no preview (`image_data IS NULL`,
`select_image_candidates` non-redo predicate), so corrupt/flagged rows are already candidates for
batch regeneration. On success it stores the preview; on failure it increments `errors` and logs.
No status column or per-design persistence is added.

## 4. Frontend changes

### 4.1 Browse design card — replace the "No Preview Image" message
When a design card has no preview (`image_data IS NULL`), replace the current "No Preview Image"
placeholder with:

> Preview could not be generated — the file may be corrupt or unreadable

- Keep the distinct card visual (so it is not confused with a loading/generating state).
- Clicking the card still opens Design Detail for triage.
- Optional triage aid: a "Needs attention" quick filter that selects designs with
  `image_data IS NULL` in Browse.

### 4.2 ImportView completion summary
On completion, when `failed_decode_count > 0`, show:
> Imported M designs; N could not be read (no preview was generated).
> These can be regenerated under **Admin → Tagging Actions**.

Include a direct CTA to navigate to Browse filtered to the affected items (and/or to Tagging Actions).

### 4.3 DesignDetail
- Show a non-blocking banner when there is no preview: "Preview could not be generated — the file may
  be corrupt or unreadable."
- Offer per-design **Regenerate Preview** (existing `renderDesign3dPreview`) and **Open folder**
  (existing `openDesignInExplorer`).
- Note: a missing file does not clear an existing preview (image is a DB BLOB); Regenerate on a design
  whose file is gone will fail with a "file not found" style error, which is distinct from a corrupt
  decode.

## 5. Out of scope / removed from earlier draft
- No schema migration and no new `designs` columns (status, failure reason/category, mtime removed).
- No structured `StitchParseError` enum or reader dispatcher layer (was only needed to feed the
  removed reason/category columns).
- No auto-regeneration on file edit (edited files are regenerated explicitly by the user).
- Fixing preview generation in the restore reconciliation import path is deferred future work.

## 6. Impacted files
| Area | File(s) | Change |
|---|---|---|
| Rust | `src/routes/bulk_import.rs` | count failed previews; add `failed_decode_count` to confirm result + `failed_count` to progress event |
| Types | `frontend/src/lib/types/ipc.ts`, `types/index.d.ts` | add `failed_decode_count` / `failed_count` fields (type parity) |
| Adapter | `frontend/src/lib/api/commandAdapter.ts` + tests | expose new fields; assert camelCase keys |
| View | `frontend/src/lib/views/BrowseView.svelte` | informative no-preview card message; optional "Needs attention" filter (`image_data IS NULL`) |
| View | `frontend/src/lib/views/ImportView.svelte` | completion summary + CTA when `failed_decode_count > 0` |
| View | `frontend/src/lib/views/DesignDetailView.svelte` | no-preview banner + Regenerate / Open folder |
| Tests | both `MainView.test.ts` suites + `BrowseView/ImportView/DesignDetailView` tests + `ImportTestHarness.svelte` | assert new copy/summary/CTA paths |

## 7. Risks & edge cases
- **Deferred restore reconciliation:** its rows are healthy files with NULL image until regenerated; if
  it ever runs before previews are added there, those rows will show the "could not be read" message
  even though the file is fine. Accepted for this change (will be fixed with the deferred import-preview
  work).
- **Transient read/lock during bulk import** can also leave a NULL image even if the file later reads
  fine; the remedy (Regenerate) is identical, so the message stays truthful about the outcome
  ("could not be read / could not be generated"), not about the cause.
- **No loop risk:** Tagging Actions only regenerates candidates that currently have no preview, so a
  healthy file is fixed in one run and stops being a candidate; a still-corrupt file is retried only
  when the user runs regeneration again.
- **Missing-file vs corrupt:** a deleted file keeps its preview; only actions that re-read the file
  (Regenerate, stitching) surface a "file not found" error. NULL image never means "file missing".

## 8. Verification / testing
- **Rust:** extend `bulk_import` tests to assert that a decode/read failure still inserts the row with
  `image_data IS NULL` and that `failed_decode_count`/`failed_count` are reported. Keep affected tests
  `#[serial]` with temp dirs where they touch shared resources.
- **Frontend (Vitest from repo root):**
  - `BrowseView` — no-preview card shows the new guidance copy.
  - `ImportView` (via `ImportTestHarness`) — completion with `failed_decode_count>0` shows summary +
    CTA.
  - `DesignDetailView` — no-preview banner + Regenerate clears it on success.
  - Both `MainView` suites if routing/copy changes; full `npx vitest run`.
  - `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` for type parity.

## 9. Build order
1. `bulk_import.rs` failure counts + confirm-result/progress fields (+Rust tests).
2. `types/ipc.ts` / `index.d.ts` / `commandAdapter` + adapter tests.
3. BrowseView no-preview card message (and optional "Needs attention" filter).
4. ImportView completion summary + CTA.
5. DesignDetail no-preview banner + Regenerate / Open folder.
6. Full Rust + full Vitest + `svelte-check` gate.
