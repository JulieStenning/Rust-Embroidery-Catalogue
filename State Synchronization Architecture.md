# State Synchronization Architecture Plan (Revised)

## Changes from Previous Version

Based on your feedback, the following adjustments have been made:

| Feedback Point | Change Made |
|----------------|-------------|
| AI tagging is not relevant to manual detail edits | Removed all references to `ai_tagging` background tasks. Only manual user edits are tracked. |
| User may navigate to other pages before returning to Browse | Store persists across **all** route transitions. `consumePatches()` fires whenever `#/designs` is entered, regardless of intermediate pages visited. |
| User may re-edit the same design multiple times | `trackMutation()` performs a **deep merge** — subsequent edits to the same design ID overwrite/merge with prior patches, so the latest state always wins. |
| Defer hoop recalculation entirely | Removed the Background Task architecture (Phase 4). Card/filter updates for recalculation will be designed and implemented when that feature is built. |

---

## 1. Architectural Overview — Data Flow

```
┌──────────────────────────────────────────────────────────────────────┐
│                        FRONTEND (Svelte)                             │
│                                                                      │
│  ┌──────────────┐    on each save    ┌─────────────────────────┐    │
│  │ DesignDetail │ ─────────────────► │  designSessionStore.ts   │    │
│  │ View.svelte  │   trackMutation()  │  (writable Svelte store) │    │
│  └──────────────┘                    │  - modifiedIds: Set       │    │
│                                      │  - pendingPatches: Map    │    │
│         │                            └───────────┬───────────────┘    │
│         │ prev/next                               │                   │
│         │ or navigate to any page                 │ on route to       │
│         ▼                                         │ #/designs         │
│  ┌──────────────┐                                 ▼                   │
│  │  Hash Router  │                      ┌─────────────────────────┐  │
│  │  #/designs/N  │                      │  MainView.svelte         │  │
│  │  #/settings   │                      │  $effect subscriber      │  │
│  │  #/projects   │                      │  → consumePatches()      │  │
│  │  #/designs    │ ◄──── return ─────── │  → applyPatchesToBrowse()│  │
│  └──────────────┘                      │  → invalidate previews   │  │
│                                         └─────────────────────────┘  │
│                                                                      │
│  ┌──────────────┐                                                    │
│  │   Tauri       │◄──── invoke() ──── mutation commands              │
│  │   Commands    │     (update_metadata, set_rating, etc.)           │
│  └──────────────┘                                                    │
└──────────────────────────────────────────────────────────────────────┘
```

**Key insight:** The store lives at module scope (imported by both `DesignDetailView` and `MainView`). It persists across all route changes because it's a plain JavaScript object held in memory — not tied to any component lifecycle. Whether the user goes DesignDetail → Settings → Browse, or DesignDetail → Projects → Browse, the store retains all pending patches.

---

## 2. Frontend: Session Store Design

### 2.1 New File: `frontend/src/lib/stores/designSessionStore.ts`

**TypeScript interfaces (add to `frontend/src/lib/types/index.d.ts`):**

```typescript
// The shape of a mutation patch — a partial subset of BrowseDesignSummary fields
interface MutationPatch {
  designer?: string;
  source?: string;
  hoop?: string;
  projects?: string[];
  tags?: string[];
  imageTags?: string[];
  stitchingTags?: string[];
  rating?: number | null;
  is_stitched?: boolean;
  tagsChecked?: boolean;
}

interface DesignSessionState {
  modifiedIds: Set<number>;
  pendingPatches: Map<number, MutationPatch>;
}
```

**Store implementation outline:**

```typescript
// designSessionStore.ts
import { writable } from 'svelte/store';

function createDesignSessionStore() {
  const { subscribe, set, update } = writable<DesignSessionState>({
    modifiedIds: new Set(),
    pendingPatches: new Map(),
  });

  return {
    subscribe,

    /**
     * Called by DesignDetailView after every successful mutation.
     * Deep-merges the new patch into any existing patch for the same design ID.
     * Latest values always win.
     */
    trackMutation(designId: number, patch: MutationPatch) {
      update(state => {
        const nextIds = new Set(state.modifiedIds);
        nextIds.add(designId);

        const nextPatches = new Map(state.pendingPatches);
        const existing = nextPatches.get(designId) || {};
        nextPatches.set(designId, { ...existing, ...patch });

        return { modifiedIds: nextIds, pendingPatches: nextPatches };
      });
    },

    /**
     * Called by MainView when entering #/designs.
     * Returns ALL pending patches and drains the pendingPatches map.
     * modifiedIds persists for the remainder of the session (useful for
     * future features like "show recently modified").
     */
    consumePatches(): Map<number, MutationPatch> {
      let drained: Map<number, MutationPatch> = new Map();
      update(state => {
        drained = new Map(state.pendingPatches);
        return { modifiedIds: state.modifiedIds, pendingPatches: new Map() };
      });
      return drained;
    },

    /**
     * Reset the entire session.
     */
    clearSession() {
      set({ modifiedIds: new Set(), pendingPatches: new Map() });
    },
  };
}

export const designSessionStore = createDesignSessionStore();
```

### 2.2 Handling Repeated Mutations to the Same Design

Scenario: User edits Design 1 (rating: 3), navigates to Design 2, edits Design 2 (designer: "Foo"), navigates back to Design 1, edits Design 1 again (tags: ["Flowers"]).

**How `trackMutation` handles this:**

| Call | `pendingPatches` State |
|------|----------------------|
| `trackMutation(1, { rating: 3 })` | `{ 1: { rating: 3 } }` |
| `trackMutation(2, { designer: "Foo" })` | `{ 1: { rating: 3 }, 2: { designer: "Foo" } }` |
| `trackMutation(1, { tags: ["Flowers"], imageTags: ["Flowers"], stitchingTags: [] })` | `{ 1: { rating: 3, tags: [...], imageTags: [...], stitchingTags: [] }, 2: { designer: "Foo" } }` |

The spread merge `{ ...existing, ...patch }` ensures the **latest edit to each field wins**, while preserving fields from earlier edits that haven't been overwritten.

### 2.3 Handling Intermediate Navigation

The store is a module-level singleton. It does not reset when components mount/unmount. The flow for an intermediate navigation:

1. User edits Design 1 → `trackMutation(1, patch)` → store = `{ 1: { rating: 3 } }`
2. User navigates to `#/settings` → `DesignDetailView` unmounts, store **persists**
3. User navigates to `#/designs` → `MainView`'s `$effect` fires, calls `consumePatches()` → gets `{ 1: { rating: 3 } }`, applies to `browseItems`
4. Card for Design 1 now shows rating 3 without any DB re-fetch

No special handling needed — the store simply outlives component lifecycles.

---

## 3. Card Grid & Search Index Reaction

### 3.1 Patch Application in `MainView.svelte`

**New `$effect` block (replaces the `browseNeedsRefresh` pattern for mutations):**

```typescript
$effect(() => {
  if (currentRoute === "#/designs") {
    // Handle session patches first (from DesignDetail edits)
    const patches = designSessionStore.consumePatches();
    if (patches.size > 0) {
      applyPatchesToBrowse(patches);
    }

    // Full reload still needed for import/deletion (browseNeedsRefresh flag)
    if (browseNeedsRefresh) {
      untrack(() => {
        loadBrowseItems(true);
        browseNeedsRefresh = false;
      });
    }
  }
});
```

**`applyPatchesToBrowse` function:**

```typescript
function applyPatchesToBrowse(patches: Map<number, MutationPatch>) {
  for (const [id, patch] of patches) {
    const index = browseItems.findIndex(item => item.id === id);
    if (index !== -1) {
      // Create a new object so Svelte's reactivity detects the change
      browseItems[index] = { ...browseItems[index], ...patch };
    }
    // If the item isn't currently loaded (filtered out or on another page),
    // no action needed — it'll be correct when the user changes filters/page.

    // Invalidate cached preview for this card (thumbnails may change)
    if (id in browsePreviewById) {
      delete browsePreviewById[id];
    }
  }

  // browseFilteredItems, browsePageItems, browsePageRows are all $derived
  // from browseItems — they recompute automatically with zero additional code.
}
```

### 3.2 Why This Avoids Full-List Flickers

- `browseItems[index] = { ...browseItems[index], ...patch }` creates a **new object at a specific array position**. Svelte's reactivity sees the array element changed and only re-renders that card's DOM bindings.
- The `{#each}` blocks use stable keys (`rowIndex`, `item.id`), so Svelte's reconciliation algorithm correctly identifies which DOM nodes to update vs. reuse.
- `browseFilteredItems` is a `$derived` — it recalculates synchronously when `browseItems` changes. If a patch causes a design to no longer match active filters, it disappears from the derived list. This is correct UX.

### 3.3 Scenario: Filter Active, Card Should Appear/Disappear

User has filter `min_rating = 4` active. Currently Design 5 has `rating = null` (not shown). User opens Design 5, sets rating to 5, returns to Browse.

| Step | What Happens |
|------|-------------|
| `trackMutation(5, { rating: 5 })` | Patch stored |
| Navigate to `#/designs` | `$effect` fires, `consumePatches()` returns `{ 5: { rating: 5 } }` |
| `applyPatchesToBrowse` | `browseItems[4] = { ...browseItems[4], rating: 5 }` |
| `browseFilteredItems` re-derives | Design 5 now passes `min_rating >= 4`, card appears |
| `browseTotalPages` re-derives | Page count may change if cards appear/disappear |

All of this happens within a single synchronous reactive tick. No loading spinner, no flicker.

---

## 4. Backend Event & Persistence Sync (Tauri / Rust)

### 4.1 Current State

The Rust backend already has a solid command layer. Each mutation command (`set_design_rating`, `update_design_metadata`, etc.) writes to SQLite and returns a `DesignCommandResult`. These are invoked directly by `DesignDetailView` → `commandAdapter.js` → Tauri `invoke()`.

### 4.2 Recommended Enhancement: Tauri Event Emission

Add event emission to each mutation command. This is **forward-looking** — not strictly needed for the immediate session-store pattern, but valuable for:

- Future multi-window scenarios
- Background task completions (when hoop recalculation is implemented later)
- Debugging/auditing

**Example in `designs.rs`:**

```rust
use tauri::Emitter;

#[tauri::command]
pub async fn set_design_rating(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignRatingRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_rating_with_pool(&state.db, design_id, request.rating).await?;

    // Emit event — fire-and-forget, errors are non-fatal
    let _ = state.app_handle.emit("design:mutated", serde_json::json!({
        "design_id": design_id,
        "fields": { "rating": request.rating }
    }));

    Ok(result)
}
```

**Commands to add event emission to:**

| Command | Fields in Event Payload |
|---------|------------------------|
| `update_design_metadata` | `designer`, `designer_id`, `source`, `source_id`, `hoop`, `hoop_id`, `notes` |
| `set_design_rating` | `rating` |
| `set_design_stitched` | `is_stitched` |
| `set_design_tags_checked` | `tags_checked` |
| `set_design_tags` | `tags`, `image_tags`, `stitching_tags`, `tags_checked` |
| `add_design_to_project` | `projects` |
| `remove_design_from_project` | `projects` |
| `delete_design` | `_deleted: true` (special marker) |

**Frontend listener (optional, for future use):**

```typescript
// In app initialization, optionally subscribe to backend events
import { listen } from '@tauri-apps/api/event';

listen<{ design_id: number; fields: Record<string, unknown> }>(
  'design:mutated',
  (event) => {
    designSessionStore.trackMutation(event.payload.design_id, event.payload.fields);
  }
);
```

This listener would be a **secondary sync path** — the primary path remains `DesignDetailView` calling `trackMutation()` directly after each invoke. The listener handles cases where the backend emits an event the frontend didn't directly trigger (future background tasks).

### 4.3 Rust-Side Search Index

The current Rust backend performs **no caching** — `get_designs` executes a fresh SQL query with dynamic `WHERE` clauses every time. Since all filtering is done client-side in `$derived` expressions, **no Rust-side index changes are needed for this plan**. The Rust backend only needs to correctly persist mutations and return fresh data when a full reload is requested (imports, deletions).

---

## 5. Forward Compatibility: Hoop Recalculation (Deferred)

Per your instruction, hoop recalculation card/filter updates will be designed and implemented when that feature is built. The architecture laid out here supports it naturally:

- When recalculation is implemented, the Rust backend will emit `"design:mutated"` with updated hoop fields.
- The frontend listener (or the store's `trackMutation` method) will apply the patch to `browseItems`.
- The card grid and filters will react identically to how they react to manual edits.
- No architectural changes to the session store or patch engine will be needed.

---

## 6. Implementation Roadmap

### Phase 1: Session Store & Patch Engine

| Step | File(s) | Description |
|------|---------|-------------|
| 1.1 | `frontend/src/lib/stores/designSessionStore.ts` (new) | Create the writable store with `trackMutation()`, `consumePatches()`, `clearSession()`. |
| 1.2 | `frontend/src/lib/types/index.d.ts` | Add `MutationPatch` and `DesignSessionState` TypeScript interfaces. |
| 1.3 | `frontend/src/lib/MainView.svelte` | Add `applyPatchesToBrowse()` function. Add `$effect` that checks the session store whenever route becomes `#/designs`. Handle preview cache invalidation for patched IDs. |

### Phase 2: DesignDetailView Integration

| Step | File(s) | Description |
|------|---------|-------------|
| 2.1 | `frontend/src/lib/views/DesignDetailView.svelte` | Import `designSessionStore`. After each successful mutation invoke, call `trackMutation()` with the changed fields. Compute the patch from the local `$state` values that were just saved. |
| 2.2 | `frontend/src/lib/views/DesignDetailView.svelte` | Review `refreshDetailAfterAction()` usage. Reduce unnecessary reloads — the detail view already has the correct local state after a successful save. Keep reload only where server-computed fields change (e.g., `setDesignTags` auto-sets `tags_checked` to `true`). |

### Phase 3: Rust Backend Event Emission

| Step | File(s) | Description |
|------|---------|-------------|
| 3.1 | `src/routes/designs.rs` | Add `use tauri::Emitter;`. In each mutation command, emit `"design:mutated"` event after successful DB write. |
| 3.2 | `src/main.rs` | Verify `AppState` provides access to `AppHandle` for event emission (required by `state.app_handle.emit()`). |

### Phase 4: Testing & Polish

| Step | Description |
|------|-------------|
| 4.1 | Manual test: Edit Design 1 (rating, tags, designer) → Next to Design 2 → edit → Back to Browse → verify both cards reflect changes. |
| 4.2 | Manual test: Edit Design 1 → navigate to Settings → navigate to Browse → verify card reflects changes (intermediate navigation). |
| 4.3 | Manual test: Edit Design 1 (rating: 3) → edit Design 2 → re-edit Design 1 (rating: 5) → Back to Browse → verify Design 1 shows rating 5 (latest wins). |
| 4.4 | Manual test: Set filter (e.g., rating ≥ 4) → go to detail → set rating to 5 → Back to Browse → card appears without manual refresh. |
| 4.5 | Manual test: Set filter → go to detail → clear rating → Back to Browse → card disappears from filtered view (but still exists in full `browseItems`). |
| 4.6 | Verify `browseNeedsRefresh` still works correctly for import and deletion flows (full reload path unchanged). |
| 4.7 | Unit test: `designSessionStore.trackMutation()` deep merge behavior with repeated edits to the same ID. |

---

## 7. Summary of Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Module-level writable store (not component `$state`)** | Must survive route changes across component mount/unmount cycles. A module singleton is the simplest pattern for cross-component state. |
| **`consumePatches()` drain-on-read pattern** | Patches are only applied when Browse is entered, not continuously. This avoids unnecessary reactive work while the user is still editing in the detail view or on other pages. |
| **Deep merge on repeated edits** | `{ ...existing, ...patch }` ensures the latest edit to each field wins, while preserving fields from earlier edits that haven't been re-edited. |
| **Client-side filtering via `$derived`** | All data (max 500 items) is already in memory. Re-running filter predicates on patch is O(n) and instantaneous. No backend round-trip needed. |
| **Patch, don't reload** | A full `loadBrowseItems(true)` causes a loading spinner, flickers the card grid, and loses scroll position. Patching individual array elements is invisible. |
| **Tauri events as secondary path** | The primary sync path is `DesignDetailView` → `trackMutation()`. Tauri events are wired for forward compatibility (background tasks, multi-window) but not required for the immediate implementation. |
| **Hoop recalculation deferred** | The architecture (session store + patch engine + Tauri events) fully supports it. Implementation will be trivial when the feature is built — just call `trackMutation()` or emit an event. |

---

## 8. File Structure After Implementation

```
frontend/src/lib/
├── stores/
│   └── designSessionStore.ts        ← NEW
├── MainView.svelte                   ← MODIFIED: add applyPatchesToBrowse + $effect
├── views/
│   └── DesignDetailView.svelte       ← MODIFIED: call trackMutation after saves
├── api/
│   └── commandAdapter.js             ← UNCHANGED
├── types/
│   └── index.d.ts                    ← MODIFIED: add MutationPatch interface
└── utils/
    └── tagHelpers.js                 ← UNCHANGED

src/
├── routes/
│   └── designs.rs                    ← MODIFIED: emit "design:mutated" events
└── main.rs                           ← POSSIBLY MODIFIED: AppHandle accessibility
```