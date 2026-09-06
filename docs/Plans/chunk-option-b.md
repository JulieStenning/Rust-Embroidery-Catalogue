# Plan — Option B: Stop re-shipping / re-storing the full file list at precheck

## 1. Purpose & scope

Option A made Step-2 scale (no 40k DOM rows, O(1) selection). The remaining all-at-once
cost is the **Continue → precheck** leg: the frontend re-serialises every selected path back
to Rust (`selected_files`) and the backend stores that full list in the in-memory context
map before Step 3 appears.

**Goal:** the scan already ran on the backend. Keep it server-side behind a token, and have
Continue send only the (small) folder-level selection deltas + assignment decisions. The
backend reconstructs the concrete `selected_files` from its own scan.

**In scope:** server-side scan store + `scan_token` (minted by preview); a new compact
precheck command; preview returns the token; frontend sends compact deltas; Rust + frontend
tests incl. a negative "must not send the full list" assertion; folder-grouping parity.

**Out of scope:** stopping the 40k download at scan time (needs server-side review paging -
separate "Option B2"); changes to import execution or legacy full-wire commands.

## 2. Current flow (code-grounded)

- `preview_bulk_import_wire_with_pool` (`bulk_import.rs:2000`) scans, filters DB-existing,
  infers per-folder assignments, returns all `scanned_files`; nothing retained server-side.
- `precheck_bulk_import_wire` (`bulk_import.rs:1595`) receives the full `BulkImportConfirmWire`
  with `selected_files` and stores `confirm_wire.clone()` as context.
- `ImportNow` -> `do_confirm_bulk_import_wire_internal` -> `persist_bulk_import_confirm_wire`
  iterates `wire.selected_files`.

## 3. Target flow

1. Preview mints `scan_token`, stores the filtered scan catalog + resolved assignments, and
   returns the token with the normal preview.
2. Continue sends a compact request (no `selected_files`): `scan_token`, global ids,
   `per_folder_assignments`, `selection` (folder base + exceptions), `create_on_import`.
3. Backend reconstructs `selected_files` from stored catalog + spec, drops assignments for
   folders with zero selected files, builds the canonical `BulkImportConfirmWire`, stores the
   normal context, returns the context token.
4. ImportNow / persist unchanged.

## 4. Key design decision - folder identity parity

Selection is keyed by slash-normalised folder path; exceptions are exact full paths. Backend
must derive the same folder per file as the frontend (`trim` -> `\`->`/` -> prefix up to last
`/`; no slash => "Unknown folder"). Add one Rust helper + a parity unit test over
representative raw forms. Exceptions match by exact full-path equality; grouping only decides
the per-folder default.

## 5. Wire / types (Rust)

New request types in `bulk_import.rs`:
- `BulkImportPrecheckFromScanRequest { scan_token, global_designer_id, global_source_id,
  per_folder_assignments: Vec<FolderAssignmentWire>, selection: BulkImportSelectionWire,
  create_on_import }`
- `BulkImportSelectionWire { deselected: Vec<FolderFilesWire>, selected_only: Vec<FolderFilesWire> }`
- `FolderFilesWire { folder_path: String, files: Vec<String> }`

`BulkImportPreview` gains `scan_token: String`.

New scan store mirroring `BULK_IMPORT_CONTEXT_STORE`: TTL ~60 min, prune-on-insert.

## 6. Commands & main.rs

- New command `precheck_bulk_import_from_scan(request) -> BulkImportPrecheckResult`; register
  in `src/main.rs`.
- Reconstruction helper `materialize_selected_from_scan(catalog, selection)` (Rust twin of
  `selMaterializeSelectedPaths`) + drop-empty-assignment logic.
- Keep legacy `precheck_bulk_import_wire`, debug/direct-execute commands untouched.

## 7. Frontend

- `types/ipc.ts`: add `scanToken` to preview; add request/selection folder types.
- `commandAdapter.ts`: `precheckImportWire` calls `precheck_bulk_import_from_scan` with the
  compact request; stop sending `selected_files`.
- `ImportView.svelte`: store `scan_token`; build compact request from `importSelection`;
  rescan on stale-scan-token error.
- Session store unchanged (preview carries the token).

## 8. Tests

Rust: scan-store mint/lookup/prune; preview token; reconstruction for default-all / none /
mixed / Unknown folder; drop zero-selected assignments; unknown-token error; small persist
e2e; folder-parity helper.
Frontend: assert compact `selection` in precheck payload; negative test that no full
`selected_files` list is sent; adapter command/payload updates.

## 9. Phases & verification

1. Backend store/token/parity/command + Rust tests -> `cargo test bulk_import` + `cargo check`.
2. IPC types + adapter + ImportView -> `npx svelte-check` + vitest on affected files.
3. Rewrite affected tests incl. negative assertion; run both MainView shells + ImportView.
4. Manual large-library test.

## 10. Risks

- Scan staleness (catalog snapshots at scan time). Mitigate: TTL + rescan on missing token.
- Extra backend catalog copy (~40k paths) until TTL.
- Folder-parity is the main correctness risk (helper + tests).
- Back-compat: legacy full-wire path preserved.
