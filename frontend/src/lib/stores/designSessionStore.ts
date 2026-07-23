import { writable, get } from "svelte/store";

// ---------------------------------------------------------------------------
// Types (mirrored in index.d.ts for wider usage, but self-contained here)
// ---------------------------------------------------------------------------

/**
 * A partial subset of BrowseDesignSummary fields that can be mutated
 * during a DesignDetails session and subsequently patched into the
 * browse card grid.
 */
export interface MutationPatch {
  designer?: string;
  source?: string;
  hoop?: string | null;
  projects?: string[];
  tags?: string[];
  imageTags?: string[];
  stitchingTags?: string[];
  rating?: number | null;
  is_stitched?: boolean;
  tagsChecked?: boolean;
}

export interface DesignSessionState {
  /** Set of design IDs modified during the current session. */
  modifiedIds: Set<number>;

  /**
   * Map of design ID → accumulated field patches.
   * Stored as plain object entries so Svelte's reactivity can track changes.
   */
  pendingPatches: Record<number, MutationPatch>;
}

// ---------------------------------------------------------------------------
// Store implementation
// ---------------------------------------------------------------------------

function createDesignSessionStore() {
  const { subscribe, set, update } = writable<DesignSessionState>({
    modifiedIds: new Set(),
    pendingPatches: {},
  });

  return {
    subscribe,

    /**
     * Track a mutation that was successfully persisted to the database.
     *
     * Called by DesignDetailView after every successful save.  Subsequent
     * edits to the same design ID deep-merge: the latest value for each
     * field wins, while fields from earlier edits (that haven't been
     * re-edited) are preserved.
     *
     * @param designId - The numeric design ID that was mutated.
     * @param patch    - The subset of fields that changed.
     */
    trackMutation(designId: number, patch: MutationPatch) {
      update((state) => {
        const nextIds = new Set(state.modifiedIds);
        nextIds.add(designId);

        const nextPatches: Record<number, MutationPatch> = {
          ...state.pendingPatches,
        };

        const existing = nextPatches[designId] ?? {};
        nextPatches[designId] = { ...existing, ...patch };

        return {
          modifiedIds: nextIds,
          pendingPatches: nextPatches,
        };
      });
    },

    /**
     * Drain all pending patches so they can be applied to the browse
     * card grid.  Returns a snapshot of the current pendingPatches and
     * resets the map to empty.
     *
     * `modifiedIds` is *not* cleared — it persists for the remainder
     * of the session (useful for future features like "show recently
     * modified").
     *
     * Called by MainView whenever the route transitions to #/designs.
     */
    consumePatches(): Record<number, MutationPatch> {
      let drained: Record<number, MutationPatch> = {};

      update((state) => {
        drained = { ...state.pendingPatches };
        return {
          modifiedIds: state.modifiedIds,
          pendingPatches: {},
        };
      });

      return drained;
    },

    /**
     * Reset the entire session store.  Intended for use at app startup
     * or when the user explicitly wants to discard the session.
     */
    clearSession() {
      set({
        modifiedIds: new Set(),
        pendingPatches: {},
      });
    },
  };
}

/** Singleton session store for tracking mutations across the DesignDetails session. */
export const designSessionStore = createDesignSessionStore();