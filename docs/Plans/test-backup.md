# Test Plan: Backup & Restore — Unmatched-Files Reconciliation

Manual + automated test plan for the reconciliation feature set:

1. **Inline preview/metadata on import** — the restore "Import unmatched file(s)" path now
   generates a preview image + `image_type` + dimensions + counts + recommended hoop, and inserts a
   **flagged** row (NULL preview) when a file cannot be decoded instead of skipping it.
2. **Restore progress card** — scope-aware, correct per-operation labels, dismissible terminal card
   (no more false "Database restored" on a designs-only sync).
3. **Standalone "Find unmatched design files"** action in Backup & Restore (no full restore needed).
4. **Shared component** — the same scan → prompt → import flow is reused by Batch Operations.

---

## Part 1 — Automated checks (run first)

From the repo root:

```
cmd /c "npx vitest run frontend/src/lib/components/__tests__/UnmatchedFilesReconciler.test.ts"
cmd /c "npx vitest run frontend/src/lib/views/__tests__/BackupView.test.ts"
cmd /c "npx vitest run frontend/src/lib/views/__tests__/BatchOperationsView.maintenance.test.ts"
cmd /c "npx vitest run BatchOperationsView"
cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"
```

Full frontend sweep (may take >30s):

```
cmd /c "npx vitest run"
```

Rust side (changed earlier in this feature set):

```
cargo test
```

Expected: all suites pass; `svelte-check` reports **0 errors, 0 warnings**.

---

## Part 2 — Manual UI testing

### Setup

1. Launch the dev app — dev mode resolves data to `<repo>/dev_data/` and auto-seeds the database on
   first run:
   - `start-rust-app-no-build.bat` (if a debug build already exists), or
   - `start-rust-app.bat` (build + run).
2. The design library the scan reads is **`dev_data/MachineEmbroideryDesigns/`**.
3. Create a file to simulate an "unmatched" design (copy a real fixture in):

   ```
   copy "tests\Test Designs\Bean.pes" "dev_data\MachineEmbroideryDesigns\ZZ-unmatched.pes"
   ```

   Use a supported extension: `pes`, `dst`, `exp`, `jef`, `hus`, `vp3`.

### Scenario A — Standalone scan finds & imports (Backup & Restore)

1. Navigate **Admin → System → Backup & Restore → Restore** tab.
2. Scroll to the **Find unmatched design files** card → click **Scan for unmatched files**.
3. Expect the **Unmatched files found** card: *"N design file(s) on disk have no record in the
   catalogue (scanned M)…"* with the sample path list.
4. Click **Import N file(s)**.
5. Expect a success toast *"Imported N unmatched file(s)."* and the prompt to close.
6. Open **Browse** and search `ZZ-unmatched` → the design appears **with a thumbnail preview**
   (proves preview + metadata are generated on import).
7. Confirm the prompt is gone and the **Restore in progress** card is not stuck.

### Scenario B — Scan finds nothing

1. Reopen the reconciler and click **Scan for unmatched files** again (after A imported the file).
2. Expect an info toast: *"No unmatched design files found (checked N)."* and **no** prompt card.

### Scenario C — Flagged (undecodable) file

1. Create a garbage file with a supported extension:

   ```
   echo not an embroidery file > dev_data\MachineEmbroideryDesigns\ZZ-broken.pes
   ```

2. **Scan** → the prompt shows it → **Import**.
3. Expect a **warning** toast: *"1 need attention (preview could not be generated) — regenerate in
   Batch Operations."*
4. In **Browse**, the design shows the no-preview / "could not be read / needs attention" treatment
   and appears under the **Needs attention** filter — i.e. it was inserted as a **flagged** row
   (NULL preview), not silently dropped.

### Scenario D — Batch Operations entry point (shared component)

1. Navigate **Admin → Batch Operations**.
2. Click the **Maintenance & File Processing** tab.
3. At the bottom the same **Find unmatched design files** card appears → **Scan** → the same
   prompt/Import/Dismiss. (Copy another `*.pes` into the library first so there is something to find.)
4. Import and verify in Browse, as in Scenario A.

### Scenario E — "Restore Both" regression

1. Backup & Restore → **Restore** tab → choose a DB backup file and ensure a designs backup folder
   is set.
2. Click **Restore Both** and confirm.
3. Expect the progress card to show the correct per-step labels and the **Unmatched files found**
   prompt to appear afterwards if extras exist.
4. Spot-check the standalone buttons:
   - **Restore Database Now** → card ends **"Database restore complete"**.
   - **Sync designs from backup** → card ends **"Design sync complete"** (never "Database restored").

### Scenario F — Shared-store behaviour (the extraction)

1. In Backup & Restore, **Scan** so the prompt is showing.
2. Navigate away (e.g. to Browse) and back → the prompt should persist (module-level store).
3. Batch Operations shows it too (same store). Click **Dismiss** → it clears in both views.

### Scenario G — Busy gating

1. While a scan/import is running, confirm the **Scan** button is disabled and the import/dismiss
   controls are gated by the busy lock.

### Scenario H — Cancel

1. **Import cancel:** start an import with many unmatched files; while "Importing…", click **Cancel**
   in the prompt. Expect the button to switch to "Cancelling…", the run to stop shortly after, and a
   warning toast *"Import cancelled — N file(s) imported."* (rows imported before the stop are kept —
   cancel is "stop the rest", not rollback).
2. **Restore cancel:** start **Sync designs from backup**; while the progress card shows the designs
   phase, click **Cancel** on the card. Expect "Cancelling…", the sync to stop, and the terminal card
   to read **"Design sync cancelled"**.
3. Confirm Cancel is **disabled during the database swap** (tooltip "This step can't be interrupted")
   and **not shown** on the unmatched-import progress card (the prompt owns that one).
4. After cancelling, start another run and confirm the button is not stuck showing "Cancelling…".

---

## Part 3 — Cleanup / reset between runs

- Click **Dismiss** to clear the prompt, or restart the app (the shared store resets on reload).
- Delete test files: `del dev_data\MachineEmbroideryDesigns\ZZ-*.pes`.
- Note: Scenarios A/C actually **add catalogue rows**. For a clean catalogue, delete those designs in
  **Browse** (or restore a DB snapshot); otherwise a re-scan simply finds nothing for them (they are
  matched now).

---

## Part 4 — Where to look if something fails

- App log (dev): `dev_data/logs/` — search for:
  - `[restore] unmatched-file import finished`
  - `preview generation failed`
  - `hoop recommendation failed`
- Rust commands involved (registered in `src/main.rs`):
  - `detect_design_files_absent_from_database`
  - `import_unmatched_design_files`
- Key implementation files:
  - `src/services/restore.rs` (`import_single_design`, `import_unmatched_design_files`,
    `perform_designs_restore`, `RestoreProgress`)
  - `src/services/design_metadata.rs` (`parse_design_file_lenient`)
  - `src/routes/restore.rs` (progress emitters)
  - `frontend/src/lib/components/UnmatchedFilesReconciler.svelte`
  - `frontend/src/lib/stores/unmatchedFilesStore.ts`
  - `frontend/src/lib/views/BackupView.svelte`, `BatchOperationsView.svelte`
