# Plan — Responsive & Chunked Handling of Very Large Imports (40,000+ designs)

## Purpose

The bulk-import flow stalls / appears unresponsive when a user imports a very large
existing library (e.g. 40,000 designs). Pressing **"Continue with {N} designs"** on the
Review page does not appear to respond, and it takes far longer to reach the next page at
40,000 files than at 8,000. This document explains exactly what happens on that button,
identifies where a genuine all-files-at-once pass occurs, and proposes options for doing
the work in smaller, responsive chunks.

> **Revision note (v2).** The first draft analysed the wrong button — the final **"Import
> Designs"** action (Step 3), whose per-file import loop is already chunked. The
> unresponsive step is actually the **"Continue with {N} designs" → precheck** transition
> on the Review page, which really does all-40k work up front. This revision corrects the
> mapping.

---

## 1. Which button, and what it actually does

The reported button is **"Continue with {N} designs"** on the *Review scanned files* page
(Step 2), `frontend/src/lib/views/ImportView.svelte:1218`. Its handler is
`runImportPrecheck` (`ImportView.svelte:496`).

Flow when the button is pressed with a large library:

1. **Step 2 renders one checkbox row per scanned file** — `{#each folder.files}` at
   `ImportView.svelte:1323`. With 40,000 files that alone is ~40,000 DOM
   `<label>`/`<input>` nodes plus per-folder grouping, so the webview main thread is
   already saturated just *displaying* the review page. This is the first reason the app
   and the Continue click feel unresponsive.
2. `importSelectedFiles` is defaulted to **all** scanned files at scan time
   (`runImportPreview`, `ImportView.svelte:705`). For a first import of a large library
   that is the full 40,000.
3. `runImportPrecheck` sets `importLoading` (label → "Running…") and `beginBusy(...)`,
   then builds the confirm wire with `buildImportConfirmWire` (`ImportView.svelte:465`),
   which iterates all selected files to derive per-folder summaries and copies the full
   40,000-path `selected_files` list.
4. It invokes `precheckImportWire` → Tauri **`precheck_bulk_import_wire`**
   (`src/routes/bulk_import.rs:1595`). That **synchronous** command:
   - resolves per-folder assignments (`resolve_bulk_import_assignments`, pure, O(folders));
   - reads first-import / hoop precheck state;
   - **stores the entire confirm wire — all 40,000 selected paths plus folder
     assignments — in an in-memory context map** keyed by a token with a 15-minute TTL
     (`store_bulk_import_context`, `bulk_import.rs:1600`).
5. Only when that invoke returns does the frontend set the context token and
   `navigateTo("#/import/step3")`. No incremental progress is shown during the wait, so
   from the user's point of view "nothing happens".

So all 40,000 items are shipped across IPC at scan time, **shipped back up** as a confirm
wire and stored in backend memory at precheck, and rendered as ~40,000 DOM rows in
between — before Step 3 can appear. That is why 8,000 files feel quick and 40,000 do not.

---

## 2. Is "something is done to all 40,000 before the next step" correct?

**Yes — for this button.** The scan → review → precheck path genuinely does all-40k work
up front:

- **All 40,000 files are enumerated as a full review DOM** (Step 2) — one checkbox per
  file — before the Continue button is even reached.
- **The full selected-file list (all 40k paths) is rebuilt, shipped over IPC, and stored
  in backend memory** during the synchronous precheck invoke the Continue button triggers.
- The precheck command is **synchronous and un-throttled**: the UI fully awaits it and
  shows no incremental progress, so during the wait "nothing happens".

The earlier draft's blanket claim — *"there is no all-files-up-front heavy pass"* — is
**wrong for this step**. It is only true for the *later* import execution ("Import
Designs"): that loop (`persist_bulk_import_confirm_wire`, `bulk_import.rs:1010`) really
does process one file at a time, generating each thumbnail inside a cancellable loop that
commits every batch of `commit_batch_size` (default 10). That per-file loop is not the
cause of the reported hang.

Two distinct scaling problems therefore live in the same wizard:

1. **Step 2 renders every file as a DOM node** — ~40k rows; heavy memory and a busy main
   thread that makes the whole app feel frozen and the Continue click appear unresponsive.
2. **Precheck ships and stores the entire file list synchronously** (large IPC payload,
   in-memory context copy) with no progress feedback, before Step 3 can render.

---

## 3. Options to handle huge libraries in smaller, responsive chunks

The per-file import loop is already chunked, so the fixes must target the
scan → review → precheck boundary that does all-40k work at once.

### Option A — Don't render 40,000 rows in Step 2  *(recommended, medium effort, big payoff)*
Replace the per-file checkbox grid with a **per-folder summary list** (folder, file count,
Designer/Source). Make selection folder-level ("Select / Deselect all in folder", plus
running totals) and render per-file rows **only lazily** — when a folder is expanded,
paged, or searched. This removes the ~40k-node DOM build that freezes the page and the
Continue button.

### Option B — Stop re-shipping and re-storing all file paths in precheck  *(recommended, medium effort)*
Scanning already runs on the backend (`preview_bulk_import_wire`). Keep the scan result on
the backend keyed by a scan/context token, and have the review form send only the
**per-folder assignment decisions + folder-level selection deltas** (e.g. which folders
are selected, and a compact list of files *deselected*), not the full 40k path list on
every Continue. This turns the Continue/precheck step from O(files) + multi-MB IPC into
O(folders), and removes duplicate in-memory copies of 40k paths.

### Option C — Throttle / aggregate progress events on the import run  *(low effort)*
Applies to the Step-3 "Import Designs" execution: emit ~2–4 events/second or one per `N`
files instead of two per file, so a long import keeps the UI and the Stop button
responsive.

### Option D — Raise / expose the DB commit batch size  *(low effort, small knob)*
Default 10 is very conservative; a modest bump (e.g. 50–100) trims commit overhead during
the actual import. Secondary to A and B.

### Option E — Formal resumable job with persisted checkpoints  *(highest effort)*
A job store with checkpoints. Given the per-file import loop is already safe to interrupt
mid-run (each committed batch is durable) and re-running a scan skips already-imported
files, Options A + B deliver nearly all the benefit at the scan/review boundary without a
job store.

---

## 4. Recommendation

The user's concern is correct for the **Continue → precheck** step: it does all-40k-scale
work up front and is the one that appears to hang. Fixes in priority order:

1. **Option A** — stop building a ~40,000-row review DOM (fixes the unresponsive button).
2. **Option B** — stop round-tripping and storing the full 40k path list through precheck
   (removes the big synchronous all-at-once pass before the next page).
3. **Option C** — keep the Step-3 import UI responsive via throttled progress events.

Only after A and B should commit-batch tuning (D) or a formal resumable job (E) be
considered; the per-file import loop is already chunked and cancellable.

---

## 5. Files / code referenced

- `frontend/src/lib/views/ImportView.svelte` — Step-2 review page, per-file checkbox grid
  (`ImportView.svelte:1323`), the **"Continue with {N} designs"** button
  (`ImportView.svelte:1218`), `runImportPreview` (`:705` default-selects all),
  `buildImportConfirmWire` (`:465`), `runImportPrecheck` (`:496`).
- `src/routes/bulk_import.rs` — `precheck_bulk_import_wire` (`:1595`, synchronous),
  `store_bulk_import_context` (`:1600`, keeps all paths in memory),
  `resolve_bulk_import_assignments` (`:1840`), and the later per-file import loop
  `persist_bulk_import_confirm_wire` (`:1010`, already chunked / cancellable).
- `src/services/scanning.rs` — whole-tree extension scan (name/size only, runs on backend).
- `frontend/src/lib/stores/busyStore.ts` — global UI lock (`beginBusy`/`endBusy`,
  `busyActive`) used while the Continue / import actions run.
- `docs/Plans/Disabling menu and buttons during long running actions.md` — prior work that
  added the global busy/chrome lock consumed above.

