import { writable } from "svelte/store";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface TagChangeState {
  /**
   * True when the browse tag filter options need to be reloaded
   * (a tag was deleted or renamed).
   */
  tagsNeedRefresh: boolean;

  /**
   * True when the browse design cards need to be reloaded
   * (a tag was deleted, or a tag that is used by designs was renamed).
   */
  designsNeedRefresh: boolean;
}

// ---------------------------------------------------------------------------
// Store implementation
// ---------------------------------------------------------------------------

function createTagChangeStore() {
  const { subscribe, set, update } = writable<TagChangeState>({
    tagsNeedRefresh: false,
    designsNeedRefresh: false,
  });

  return {
    subscribe,

    /**
     * Flag that a tag was deleted. Deleting a tag always requires
     * a full browse refresh: the tag filter options change and any
     * cards that listed the tag need to be updated.
     */
    flagTagDeleted() {
      update(() => ({
        tagsNeedRefresh: true,
        designsNeedRefresh: true,
      }));
    },

    /**
     * Flag that a tag was renamed.
     *
     * @param hasDesigns - True when the renamed tag is used by at least
     *                     one design. In that case the card data (tag names
     *                     shown on cards) is stale, so a full design reload
     *                     is needed. Otherwise only the filter options need
     *                     refreshing.
     */
    flagTagRenamed(hasDesigns: boolean) {
      update((state) => ({
        tagsNeedRefresh: true,
        designsNeedRefresh: state.designsNeedRefresh || Boolean(hasDesigns),
      }));
    },

    /**
     * Drain the flags so the caller can react to them. Returns a snapshot
     * of the current state and resets both flags to false.
     */
    consumeFlags(): TagChangeState {
      let drained: TagChangeState = { tagsNeedRefresh: false, designsNeedRefresh: false };

      update((state) => {
        drained = { ...state };
        return { tagsNeedRefresh: false, designsNeedRefresh: false };
      });

      return drained;
    },

    /**
     * Reset the entire store to its default state.
     */
    clear() {
      set({ tagsNeedRefresh: false, designsNeedRefresh: false });
    },
  };
}

/** Singleton store for tracking tag mutations that affect the browse page. */
export const tagChangeStore = createTagChangeStore();
