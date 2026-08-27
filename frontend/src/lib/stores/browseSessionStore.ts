import { writable } from "svelte/store";
import type { BrowseFilterState } from "../types/ipc";

// ---------------------------------------------------------------------------
// Browse session state
// ---------------------------------------------------------------------------
// The browse view (search query, tag/hoop/source filters, sort order, current
// page, and the full filtered design-id list) lives in component-local
// `$state` inside BrowseView.svelte. That state is destroyed whenever the
// route leaves "browse" — e.g. when the user opens a design in
// DesignDetailView. This store holds a serializable snapshot so the user
// returns to their exact search results, filters, and page after the detail
// round-trip.
//
// BrowseView keeps its local `$state` as the live source of truth while
// mounted and mirrors every change into this store (via an effect); on
// remount it restores from the store. clear() resets the snapshot.
// ---------------------------------------------------------------------------

export interface BrowseSessionState {
  filters: BrowseFilterState;
  currentPage: number;
  total: number;
  totalPages: number;
  /** Ordered full filtered design-id list used for Next/Prev in the detail view. */
  designIds: number[];
  /** Last browse scroll position, restored on return. */
  scrollY: number;
}

function defaultFilters(): BrowseFilterState {
  return {
    q: "",
    allWords: "",
    exactPhrase: "",
    anyWords: "",
    noneWords: "",
    filename: "",
    designerFilters: [],
    imageTagFilters: [],
    stitchingTagFilters: [],
    hoop: "",
    sourceFilters: [],
    rating: "",
    stitched: "",
    unverifiedOnly: false,
    searchFilename: true,
    searchTags: true,
    searchFolder: true,
    sortBy: "name",
    sortDir: "asc",
  };
}

function createInitialState(): BrowseSessionState {
  return {
    filters: defaultFilters(),
    currentPage: 1,
    total: 0,
    totalPages: 1,
    designIds: [],
    scrollY: 0,
  };
}

function createBrowseSessionStore() {
  const { subscribe, set, update } = writable<BrowseSessionState>(createInitialState());

  return {
    subscribe,

    /**
     * Replace the entire browse snapshot. Used by BrowseView's mirror effect
     * so navigating away and back preserves the browse context.
     * @param {BrowseSessionState} state
     */
    setSession(state: BrowseSessionState) {
      set(state);
    },

    /**
     * Patch a subset of the browse snapshot. Convenient for targeted writes
     * (e.g. capturing the scroll position without re-supplying the full object).
     * @param {Partial<BrowseSessionState>} patch
     */
    patchSession(patch: Partial<BrowseSessionState>) {
      update((state) => ({ ...state, ...patch }));
    },

    /**
     * Remove a single design id from the full filtered list. Used when a design
     * is deleted from the detail view so Prev/Next and the position counter skip
     * the now-deleted design while the user remains in the detail view.
     * @param {number} designId
     */
    removeDesignId(designId: number) {
      update((state) =>
        state.designIds.includes(designId)
          ? { ...state, designIds: state.designIds.filter((id) => id !== designId) }
          : state
      );
    },

    /** Discard the browse snapshot (used on explicit filter reset). */
    clear() {
      set(createInitialState());
    },
  };
}

/** Singleton session store for preserving browse context across view switches. */
export const browseSessionStore = createBrowseSessionStore();
