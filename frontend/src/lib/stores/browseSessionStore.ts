// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { writable } from "svelte/store";
import type {
  BrowseDesignCard,
  BrowseFilterState,
  BrowseTagOption,
  ProjectListItem,
  SearchPayload,
} from "../types/ipc";

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
  /** Filter signature for which `designIds` already holds the full filtered list. */
  designIdsLoadedSignature: string;
  /** Last browse scroll position, restored on return. */
  scrollY: number;
  /** Cached browse page cards so returning from Design Detail avoids re-querying. */
  items: BrowseDesignCard[];
  /** True once the current page snapshot has been loaded from the backend. */
  hasLoaded: boolean;
  /** Filter signature corresponding to the cached `items`. */
  lastFilterSignature: string;
  /** Last search payload corresponding to the cached `items`. */
  lastPayload: SearchPayload | null;
  /** Cached preview data URLs keyed by design id. */
  previewById: Record<number, string | null>;
  /** Cached tag filter options. */
  tagOptions: BrowseTagOption[];
  tagsLoaded: boolean;
  /** Cached project options for card dropdowns and bulk actions. */
  projects: ProjectListItem[];
  projectsLoaded: boolean;
  /** Cached reference-data filter dropdown options. */
  designerFilterOptions: string[];
  sourceFilterOptions: string[];
  hoopFilterOptions: string[];
  filterReferenceLoaded: boolean;
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
    minWidth: "",
    maxWidth: "",
    minHeight: "",
    maxHeight: "",
    sourceFilters: [],
    rating: "",
    stitched: "",
    unverifiedOnly: false,
    verificationStatus: "all",
    imageTagVerification: "all",
    stitchingTagVerification: "all",
    needsAttention: "off",
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
    designIdsLoadedSignature: "",
    scrollY: 0,
    items: [],
    hasLoaded: false,
    lastFilterSignature: "",
    lastPayload: null,
    previewById: {},
    tagOptions: [],
    tagsLoaded: false,
    projects: [],
    projectsLoaded: false,
    designerFilterOptions: [],
    sourceFilterOptions: [],
    hoopFilterOptions: [],
    filterReferenceLoaded: false,
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

    /**
     * Mark the cached browse page (and optionally reference-data dropdowns)
     * dirty so the next visit to Browse re-queries the backend while preserving
     * the user's active filters, current page, and scroll position.
     */
    invalidateCache(options?: { includeReferenceData?: boolean }) {
      const includeReferenceData = Boolean(options?.includeReferenceData);
      update((state) => ({
        ...state,
        items: [],
        hasLoaded: false,
        lastFilterSignature: "",
        lastPayload: null,
        previewById: {},
        designIdsLoadedSignature: "",
        ...(includeReferenceData
          ? {
              tagOptions: [],
              tagsLoaded: false,
              projects: [],
              projectsLoaded: false,
              designerFilterOptions: [],
              sourceFilterOptions: [],
              hoopFilterOptions: [],
              filterReferenceLoaded: false,
            }
          : {}),
      }));
    },

    /**
     * Mark cached reference-data dropdown options (tags, projects, designers,
     * sources, hoops) dirty without discarding the cached browse cards.
     */
    invalidateReferenceData() {
      update((state) => ({
        ...state,
        tagOptions: [],
        tagsLoaded: false,
        projects: [],
        projectsLoaded: false,
        designerFilterOptions: [],
        sourceFilterOptions: [],
        hoopFilterOptions: [],
        filterReferenceLoaded: false,
      }));
    },

    /** Discard the browse snapshot (used on explicit filter reset or DB restore). */
    clear() {
      set(createInitialState());
    },

    /**
     * Pre-apply the "Needs attention" filter so navigating to Browse opens directly on designs
     * whose preview could not be generated (no stored preview) or matching a given mode. Resets all other filters/page.
     */
    focusNeedsAttention(
      mode: "all" | "cannot_draw" | "exceeds_hoop" | "master_only" = "cannot_draw"
    ) {
      set({
        ...createInitialState(),
        filters: { ...defaultFilters(), needsAttention: mode },
      });
    },
  };
}

/** Singleton session store for preserving browse context across view switches. */
export const browseSessionStore = createBrowseSessionStore();
