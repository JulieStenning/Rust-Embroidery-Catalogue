# Bulk Deletion Workflow — Implementation Plan

**Date:** 2026-07-26
**Feature:** Multi-select batch deletion with unified confirmation modal, OS recycle bin integration, and collapsible preview drawer.

---

## 1. Current State Summary

| Aspect | What Exists |
|---|---|
| **Selection** | `browseSelectedIds: SvelteSet<number>` — already used for bulk tags, verify, project add |
| **Single delete** | Rust `delete_design(designId, deleteFile)` — when `deleteFile=true`, uses `trash::delete()` (safe OS recycle bin). Frontend `deleteDesign()` adapter in `commandAdapter.js` wraps this |
| **Bulk delete (current)** | `confirmDeleteSelectedBrowseItems()` in `MainView.svelte` — exists but is a **buggy placeholder**: it incorrectly calls `bulkVerifyDesigns`, then loops single `deleteDesign(id, false)`. DB only — no file toggle, no batch efficiency |
| **Modal (current)** | Simple two-button modal (`browseDeleteConfirmOpen`) with count only, no file action toggle, no preview list |
| **Selection cap** | None — selections can grow unbounded |
| **`trash` crate** | Already a dependency in `Cargo.toml` (version 3), already used in `delete_design_with_pool()` |

---

## 2. Key Design Decisions

1. **Single bulk Rust command** rather than N `invoke()` calls — one DB transaction, one IPC round-trip, much faster for up to 50 items.
2. **Trash, not hard delete** — The existing `trash` crate is already integrated. File deletions go to the OS recycle bin, never permanent.
3. **Selection cap at 50** — Enforced in both frontend (checkbox guard) and backend (command rejects >50). No warning — once 50 is reached, further checkbox clicks are silently ignored. Once "Delete selected" is pressed and the modal opens, all checkboxes become **disabled** so the selection cannot be modified while the confirmation modal is visible.
4. **Per-file errors don't abort the batch** — If one file can't be trashed, the DB deletion still proceeds for all designs. Individual file errors are collected and reported separately.
5. **Preview drawer default closed** — Uses `<details>` element so users can optionally verify their batch without overwhelming the modal by default.
6. **Reuse existing `design:mutated` event emission** — Same pattern as the single `delete_design` command.
7. **Single IPC call** — The entire batch deletion is one `invoke()` call, not a loop.

---

## 3. Files to Modify

| File | Changes |
|---|---|
| `src/routes/designs.rs` | Add `BulkDeleteDesignsRequest`, `BulkDeleteDesignsResult`, `bulk_delete_designs` command, and inner function |
| `src/main.rs` (command registration) | Register the new `bulk_delete_designs` Tauri command |
| `frontend/src/lib/api/commandAdapter.js` | Add `bulkDeleteDesigns()` adapter function |
| `frontend/src/lib/MainView.svelte` | Add state variables; selection cap guard; selection lock; replace delete modal; replace delete handler |

---

## 4. Rust Backend Details

### Data Types

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct BulkDeleteDesignsRequest {
    pub design_ids: Vec<i64>,
    pub delete_files: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkDeleteDesignsResult {
    pub requested_count: usize,
    pub deleted_count: usize,
    pub files_trashed: usize,
    pub errors: Vec<String>,
}
```

### Logic Flow
1. Sanity check: reject if >50 or empty.
2. Begin transaction.
3. Fetch filepaths: `SELECT id, filepath FROM designs WHERE id IN (...)`.
4. Batch delete: `DELETE FROM designs WHERE id IN (...)`.
5. If `delete_files`: for each filepath, resolve and `trash::delete()`. Collect errors.
6. Commit transaction.
7. Emit `design:mutated` events.
8. Return counts and errors.

---

## 5. Frontend Changes

### New State
- `browseBulkDeleteFile` (boolean toggle for file trash)
- `browseBulkDeletePreviewOpen` (boolean for preview drawer)
- `BROWSE_BULK_DELETE_MAX = 50` (constant)
- `browseSelectionLocked` (derived: modal open or busy)

### Selection Cap
- `toggleBrowseCardSelection`: silently ignore if `>= 50` and not already selected.
- `toggleSelectAllBrowseOnPage`: cap at 50.
- Checkboxes disabled while locked or at cap.

### Delete Confirmation Modal
- Radio toggle: "Keep files" vs "Move to recycle bin"
- Warning text (conditional)
- Collapsible preview drawer with thumbnails
- Cancel / "Delete N designs" (red) buttons
- Busy state disables all

### Handler
- Calls `bulkDeleteDesigns(ids, browseBulkDeleteFile)`
- Shows success notice with counts
- Clears selection, reloads browse, resets modal state

---

## 6. Acceptance Criteria

- [X] Selecting up to 50 designs works; beyond 50 is silently ignored.
- [x] Modal shows correct count.
- [x] File toggle defaults to "keep files".
- [x] Preview drawer lists all selected with thumbnails.
- [x] "Keep files" deletes DB rows only.
- [x] "Move to recycle bin" also trashes source files.
- [X] After deletion, browse reloads, selection cleared, notice shown.
- [?] Per-file errors logged but don't abort batch.
- [?] Rust backend rejects >50 IDs.
- [x] `cargo check` passes.
- [x] `npm run check` passes. (cd frontend/ first)