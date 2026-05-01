# Unified Backfill Actions Plan

## Overview
Redesign the tagging actions page to allow users to select any combination of backfill operations (stitching types, images, thread/colour counts) to run in a single batch, processing 100 designs at a time with commit, while preserving per-operation options. Add error logging to a file with a user-accessible link.

---

## Steps

### Phase 1: Discovery & Design
1. Review and document current per-action options:
   - Stitching types: unverified only, clear existing stitching tags.
   - Images: missing images only, option to redo all.
   - Thread/colour counts: missing counts only, option to redo all.
2. Design a unified UI for multi-selecting actions and configuring their options.
3. Specify error log file location and user access method (e.g., download link).

### Phase 2: Backend Refactor
4. Refactor backend to:
   - Accept multiple actions and their options in a single request.
   - Read each design file only once per batch.
   - Run selected actions in sequence for each design, marking each as processed for successful actions.
   - Log errors (with design filename, action, and error message) to a persistent log file.
   - Commit every 100 designs.
5. Ensure the single-action features (from the UI) still work, but do not maintain separate legacy scripts.

### Phase 3: Frontend Update
6. Update tagging actions page UI:
   - Allow multi-select of backfill actions.
   - Show/hide per-action options.
   - Display progress and errors.
   - Provide a link to download/view the error log file.

### Phase 4: Verification & Testing
7. Add/extend tests to cover:
   - All combinations of actions and options.
   - Batch commit logic.
   - Error logging and log file access.
8. Manual verification of UI, DB state, and error log.

---

## Relevant Files
- `src/routes/` — Tagging actions endpoints (to be unified)
- `src/services/auto_tagging.py` — Stitching backfill logic
- `src/services/preview.py` — Image rendering logic
- `src/services/settings_service.py` — Design base path logic
- `backfill_stitching_tags.py`, `backfill_images.py`, `backfill_color_counts.py` — Reference for batching, error handling, and per-action options
- `templates/` or `static/` — Tagging actions page UI
- `tests/` — Test cases for backfill logic and UI

---

## Verification
1. Run all combinations of backfill actions and options; verify correct DB updates.
2. Confirm only 100 designs are processed per commit, even for combined actions.
3. Check that design files are not redundantly read.
4. Validate UI allows all combinations and reflects progress/errors.
5. Download/view error log and confirm contents.
6. Ensure single-action features (from the UI) still work.

---

## Decisions
- Batch size: fixed at 100.
- If a design fails for one action but succeeds for others, mark as processed for successful actions.
- No new backfill actions planned.
- Error log file to be accessible via UI.
- Legacy scripts do not need to be maintained; only the single-action features in the UI must remain functional.

---

## Further Considerations
1. Error log file location: Recommend `logs/backfill_errors.log` (rotated/cleared on new run).
2. Error log format: CSV or plain text with timestamp, design, action, error.
3. UI: Should indicate if errors occurred and prompt user to check/download the log.
