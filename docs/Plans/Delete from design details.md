I've thoroughly analyzed all five files. Here's my complete implementation plan:

---

## Current State Summary

| Aspect | Browse Page (Modern) | Design Detail Page (Legacy) |
|---|---|---|
| **Modal** | Single unified dialog with radio buttons (catalogue-only vs. trash file) + collapsible preview list | Two-step sequence: Step 1 asks "delete file too?" → Step 2 confirms "really delete file? this cannot be undone" |
| **IPC Command** | `bulkDeleteDesigns(ids[], deleteFiles)` | `deleteDesign(id, deleteFile)` (singular) |
| **Rust Backend** | `bulk_delete_designs` (batched, emits events per design) | `delete_design` (singular, separate command) |
| **Post-Delete** | Clears selection, calls `loadBrowseItems(true)`, shows notice | Calls `navigateTo("#/designs")` — but **no refresh trigger** is communicated to MainView |

---

## Step-by-Step Implementation Plan

### Step 1 — Extract a Shared `DeleteDesignsModal.svelte` Component

**Create:** `frontend/src/lib/components/DeleteDesignsModal.svelte`

Encapsulates the unified deletion dialog currently inlined in `MainView.svelte`. Props:

| Prop | Type | Description |
|---|---|---|
| `designIds` | `number[]` | IDs to delete (length 1 for detail page) |
| `previewItems` | `Array<{id, filename, filepath, dataUrl}>` | Designs shown in collapsible preview list |
| `open` | `boolean` | Controls modal visibility |
| `onClose` | `() => void` | Cancel / backdrop-click callback |
| `onDeleted` | `(result: any) => void` | Fires after successful deletion with the bulk result |

**Internal behavior:**
- When `designIds.length === 1`, the collapsible preview drawer/summary is **not rendered at all** (auto-hidden).
- Radio buttons "Remove from catalogue only" / "Move source files to recycle bin" always present.
- Calls `bulkDeleteDesigns(designIds, deleteFile)` from `commandAdapter.js`.
- Owns its internal `busy`, `deleteFile`, and `previewOpen` state.

---

### Step 2 — Refactor `MainView.svelte` to Use the Shared Component

1. **Remove** the inline `{#if browseDeleteConfirmOpen}` block and all modal markup (~60 lines of the "Browse Delete Selected Modal").
2. **Remove** associated state variables that move into the component:
   - `browseBulkDeleteFile`
   - `browseBulkDeletePreviewOpen`
   - `browseDeleteSelectedBusy`
3. **Remove** handlers replaced by the component:
   - `confirmDeleteSelectedBrowseItems`
   - `closeBrowseDeleteConfirm`
4. **Replace** with the shared component usage.
5. **Remove** the `deleteDesign` import (no longer used).

---

### Step 3 — Replace Legacy Deletion in `DesignDetailView.svelte`

1. **Remove** the entire two-step modal sequence (`deleteModalStep === "choose"` and `"confirm-file-delete"` blocks).
2. **Remove** legacy state and 6 handler functions.
3. **Replace** `deleteDesign` import with `bulkDeleteDesigns`.
4. **Add** a new `onDesignDeleted` callback prop for parent communication.
5. **Add** a single `detailDeleteModalOpen` boolean state.
6. **Wire** the "Delete design" button to open the shared modal.
7. **Render** `<DeleteDesignsModal>` with `designIds={[detailDesignId]}` — preview drawer auto-hides since only 1 design.

---

### Step 4 — Wire Post-Deletion Refresh in `MainView.svelte`

**Add** `onDesignDeleted` callback to the `<DesignDetailView>` instance:
```svelte
<DesignDetailView
  {detailDesignId}
  {detailBrowseIds}
  {detailBrowseIndex}
  {navigateTo}
  onDesignDeleted={() => { browseNeedsRefresh = true; }}
/>
```
This ensures the existing `$effect` triggers `loadBrowseItems(true)` when redirected to `#/designs`.

---

### Step 5 — Deprecate & Remove Legacy Single-Delete IPC

| Location | Action |
|---|---|
| `commandAdapter.js` | Remove `deleteDesign()` function entirely |
| `MainView.svelte` imports | Remove `deleteDesign` |
| `src/routes/designs.rs` | Remove `#[tauri::command]` from `delete_design` |
| `src/main.rs` | Remove `routes::designs::delete_design` from `generate_handler![]` |
| | Run `cargo check` to verify |

---

### Step 6 — Verify

- Run `npx svelte-check` in `frontend/` to confirm zero type errors.
- Run `cargo check` at project root to confirm Rust compilation.

---

## Summary Checklist

| # | Step | Files |
|---|---|---|
| 1 | Create `DeleteDesignsModal.svelte` shared component | New file |
| 2 | Refactor `MainView.svelte` to use shared modal | `MainView.svelte` |
| 3 | Replace legacy deletion in `DesignDetailView.svelte` | `DesignDetailView.svelte` |
| 4 | Wire post-deletion refresh callback | `MainView.svelte` |
| 5 | Deprecate and remove legacy single-delete IPC | `commandAdapter.js`, `designs.rs`, `main.rs` |
| 6 | TypeScript + Rust verification | — |

---

Shall I proceed? Toggle to **Act Mode** and I'll implement everything.