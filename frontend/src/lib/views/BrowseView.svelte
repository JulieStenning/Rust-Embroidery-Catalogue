<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { tick, untrack, onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import {
    getBrowseDesigns,
    getDesignIds,
    getBrowseDesignPreviews,
    getBrowseProjects,
    getBrowseTags,
    addDesignToProject,
    removeDesignFromProject,
    listDesigners,
    listSources,
    listHoops,
    bulkVerifyDesigns,
    bulkAddDesignsToProject,
    bulkSetTagsForDesigns,
  } from "../api/commandAdapter";
  import DeleteDesignsModal from "../components/DeleteDesignsModal.svelte";
  import FirstImportSuccessBanner from "../components/FirstImportSuccessBanner.svelte";
  import Pagination from "../components/Pagination.svelte";
  import SelectionHeader from "../components/SelectionHeader.svelte";
  import BrowseFilterPanel from "../components/browse/BrowseFilterPanel.svelte";
  import BrowseSelectionBar from "../components/browse/BrowseSelectionBar.svelte";
  import BrowseCardGrid from "../components/browse/BrowseCardGrid.svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { designSessionStore } from "../stores/designSessionStore.js";
  import { browseSessionStore } from "../stores/browseSessionStore.js";
  import { tagChangeStore } from "../stores/tagChangeStore.js";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  import { portalToBody } from "../utils/portal.js";

  /** @typedef {import("../types/ipc").BrowseDesignCard} BrowseDesignCard */
  /** @typedef {import("../types/ipc").BrowseDesignSummaryWire} BrowseDesignSummaryWire */
  /** @typedef {import("../types/ipc").BrowseTagOption} BrowseTagOption */
  /** @typedef {import("../types/ipc").ProjectListItem} ProjectListItem */
  /** @typedef {import("../types/ipc").SearchPayload} SearchPayload */
  /** @typedef {import("../types/ipc").MutationPatch} MutationPatch */
  /** @typedef {{ persisted: boolean, deleted_count: number, files_trashed: number, errors?: Array<string> }} BulkDeleteResult */
  /** @typedef {import("../types/ipc").BrowseFilterState} BrowseFilterState */
  /** @typedef {Omit<BrowseDesignSummaryWire, "projects" | "tags"> & { projects?: Array<string | { name?: string }> | string, tags?: Array<string | { description?: string }>, project_names?: Array<string> | string, folder?: string, date_added?: string | null }} BrowseCardInput */
  /** @typedef {{ persisted: boolean, updated_count?: number, updated?: number, error?: string }} BulkSetTagsResult */
  /** @typedef {{ persisted: boolean, added_count?: number, updated?: number, error?: string }} BulkAddToProjectResult */
  /** @typedef {{ persisted: boolean, verified_count?: number, updated?: number, error?: string }} BulkVerifyResult */
  /** @typedef {{ image: Array<BrowseTagOption>, stitching: Array<BrowseTagOption>, unclassified: Array<BrowseTagOption> }} TagOptionBuckets */

  let { navigateTo, browseNeedsRefresh = $bindable(false) } = $props();

  // Browse state
  /** @type {BrowseDesignCard[]} */
  let browseItems = $state([]);
  let browseLoading = $state(false);
  let browseHasLoaded = $state(false);
  /** Debounce handle for the live general-search `q` input. */
  /** @type {ReturnType<typeof setTimeout> | null} */
  let browseQTimer = null;
  // Global UI lock: reflects busyState.active so secondary controls can be
  // disabled while a long-running task runs.
  let busyActive = $derived($busyState.active);
  /** @type {ProjectListItem[]} */
  let browseProjects = $state([]);
  let browseProjectsLoaded = $state(false);
  /** @type {BrowseTagOption[]} */
  let browseTagOptions = $state([]);
  let browseTagsLoaded = $state(false);
  let browseImageTagOptions = $derived(
    (() => {
      const grouped = splitTagsByGroup(browseTagOptions);
      return [...(grouped.image || [])].sort((a, b) =>
        String(a?.description || "").localeCompare(String(b?.description || ""), undefined, {
          sensitivity: "base",
        })
      );
    })()
  );
  let browseStitchingTagOptions = $derived(
    (() => {
      const grouped = splitTagsByGroup(browseTagOptions);
      return [...(grouped.stitching || [])].sort((a, b) =>
        String(a?.description || "").localeCompare(String(b?.description || ""), undefined, {
          sensitivity: "base",
        })
      );
    })()
  );
  /** @type {TagOptionBuckets} */
  let browseGroupedTagOptions = $derived(splitTagsByGroup(browseTagOptions));
  /** @type {string[]} */
  let browseDesignerFilterOptions = $state([]);
  /** @type {string[]} */
  let browseSourceFilterOptions = $state([]);
  /** @type {string[]} */
  let browseHoopFilterOptions = $state([]);
  let browseFilterReferenceLoaded = $state(false);
  /** @type {Record<number, string | null>} */
  let browsePreviewById = $state({});
  let browsePreviewsLoading = $state(false);
  let browsePreviewRequestCounter = 0;
  let browseCurrentPage = $state(1);
  let browseTotal = $state(0);
  let browseTotalPages = $state(1);
  let browseAdditionalFiltersOpen = $state(false);
  /** @type {SvelteSet<number>} */
  let browseSelectedIds = $state(new SvelteSet());
  /** @type {HTMLDivElement | null} */
  let browseBulkBarNode = $state(null);
  let browseBulkModalOpen = $state(false);
  /** @type {Array<number | string>} */
  let browseBulkTagAddIds = $state([]);
  /** @type {Array<number | string>} */
  let browseBulkTagRemoveIds = $state([]);
  /** @type {Array<number | string>} */
  let browseBulkTagIndeterminateIds = $state([]);
  let browseBulkClearAll = $state(false);
  // Per-category uniformity of the selected designs when the bulk modal opens.
  // Uniform = every selected design shares the exact same tag set in that
  // category (Rule 2). Mixed = at least one design differs (Rule 3).
  let browseBulkImageUniform = $state(false);
  let browseBulkStitchingUniform = $state(false);
  /** @type {Record<string | number, string>} */
  let browseBulkTagGroupById = $state({});
  /** @type {number[]} */
  let browseBulkProjectSelection = $state([]);
  let browseBulkProjectDropdownOpen = $state(false);
  /** @type {Record<number, Record<number, boolean>>} */
  let browseCardProjectPendingById = $state({});
  let browseDeleteConfirmOpen = $state(false);
  const BROWSE_BULK_DELETE_MAX = 50;
  /** @type {HTMLDivElement | null} */
  let browseGridContainer = $state(null);
  let browseGridColumns = $state(5);

  const BROWSE_PAGE_ROWS = 10;
  const BROWSE_BREAKPOINT_SM = 640;
  const BROWSE_BREAKPOINT_MD = 768;
  const BROWSE_BREAKPOINT_LG = 1024;
  const BROWSE_ROW_SELECTOR_WIDTH = 28;
  /** Debounce delay (ms) before a live `q` keystroke re-queries the backend. */
  const BROWSE_Q_DEBOUNCE_MS = 250;

  /** @returns {BrowseFilterState} */
  const defaultBrowseFilters = () => ({
    q: "",
    allWords: "",
    exactPhrase: "",
    anyWords: "",
    noneWords: "",
    filename: "",
    designerFilters: /** @type {string[]} */ ([]),
    imageTagFilters: /** @type {string[]} */ ([]),
    stitchingTagFilters: /** @type {string[]} */ ([]),
    hoop: "",
    minWidth: "",
    maxWidth: "",
    minHeight: "",
    maxHeight: "",
    sourceFilters: /** @type {string[]} */ ([]),
    rating: "",
    stitched: "",
    unverifiedOnly: false,
    needsAttention: false,
    searchFilename: true,
    searchTags: true,
    searchFolder: true,
    sortBy: "name",
    sortDir: "asc",
  });

  let browseFilters = $state(defaultBrowseFilters());

  let browseFiltersAreDefault = $derived(
    browseFilters.q === "" &&
      browseFilters.allWords === "" &&
      browseFilters.exactPhrase === "" &&
      browseFilters.anyWords === "" &&
      browseFilters.noneWords === "" &&
      browseFilters.filename === "" &&
      browseFilters.designerFilters.length === 0 &&
      browseFilters.imageTagFilters.length === 0 &&
      browseFilters.stitchingTagFilters.length === 0 &&
      browseFilters.hoop === "" &&
      browseFilters.minWidth === "" &&
      browseFilters.maxWidth === "" &&
      browseFilters.minHeight === "" &&
      browseFilters.maxHeight === "" &&
      browseFilters.sourceFilters.length === 0 &&
      browseFilters.rating === "" &&
      browseFilters.stitched === "" &&
      !browseFilters.unverifiedOnly &&
      !browseFilters.needsAttention &&
      browseFilters.searchFilename &&
      browseFilters.searchTags &&
      browseFilters.searchFolder &&
      browseFilters.sortBy === "name" &&
      browseFilters.sortDir === "asc"
  );

  // --- Browse session persistence -----------------------------------------
  // BrowseView's search/filter/page state is destroyed whenever the route
  // leaves "browse" (e.g. opening a design in DesignDetailView). We restore it
  // from browseSessionStore on mount and mirror every change back so the user
  // returns to their exact search results, filters, page, and (optionally)
  // scroll position after the detail round-trip.
  /** @type {number[]} */
  let browseDesignIds = $state([]);
  let browseSessionRestored = $state(false);
  let scrollRestorePending = $state(true);

  /** @param {BrowseFilterState} f */
  function cloneBrowseFilters(f) {
    return {
      ...f,
      designerFilters: Array.isArray(f.designerFilters) ? [...f.designerFilters] : [],
      imageTagFilters: Array.isArray(f.imageTagFilters) ? [...f.imageTagFilters] : [],
      stitchingTagFilters: Array.isArray(f.stitchingTagFilters) ? [...f.stitchingTagFilters] : [],
      sourceFilters: Array.isArray(f.sourceFilters) ? [...f.sourceFilters] : [],
    };
  }

  /** Capture the current browse scroll position for restoration on return. */
  function captureBrowseScroll() {
    const y = typeof window !== "undefined" ? Number(window.scrollY || 0) : 0;
    browseSessionStore.patchSession({ scrollY: y });
  }

  /** Restore the saved scroll position once, after the first page has rendered. */
  function restoreBrowseScrollOnce() {
    if (!scrollRestorePending) return;
    scrollRestorePending = false;
    const savedY = Number(get(browseSessionStore).scrollY || 0);
    if (savedY > 0) {
      tick().then(() => {
        if (typeof window !== "undefined") {
          requestAnimationFrame(() => {
            window.scrollTo(0, savedY);
          });
        }
      });
    }
  }

  // Restore the snapshot the first time this instance mounts, then mirror every
  // subsequent state change back into the store so it survives unmount.
  $effect(() => {
    if (!browseSessionRestored) {
      browseSessionRestored = true;
      const snap = get(browseSessionStore);
      if (snap.filters) {
        browseFilters = cloneBrowseFilters(snap.filters);
        browseCurrentPage = Math.max(1, Number(snap.currentPage) || 1);
        browseTotal = Math.max(0, Number(snap.total) || 0);
        browseTotalPages = Math.max(1, Number(snap.totalPages) || 1);
      }
      if (Array.isArray(snap.designIds)) {
        browseDesignIds = [...snap.designIds];
      }
    }
    browseSessionStore.patchSession({
      filters: cloneBrowseFilters(browseFilters),
      currentPage: browseCurrentPage,
      total: browseTotal,
      totalPages: browseTotalPages,
      designIds: browseDesignIds,
    });
  });

  /** @param {string} filepath */
  function extractFolder(filepath) {
    const path = String(filepath || "")
      .trim()
      .replace(/\\/g, "/");
    if (!path) return "";
    const segments = path.split("/").filter(Boolean);
    if (segments.length <= 1) return "";
    return segments[segments.length - 2];
  }

  /**
   * @param {string} a
   * @param {string} b
   */
  function compareStrings(a, b) {
    return a.localeCompare(b);
  }

  /**
   * @param {string | { description?: string }} t
   * @returns {string}
   */
  function mapTagToString(t) {
    return typeof t === "object" && t !== null ? String(t.description || "") : String(t);
  }

  /** @param {BrowseCardInput | null | undefined} item */
  function normalizeCardItem(item) {
    if (!item || typeof item !== "object") {
      return null;
    }
    const imageTags = Array.isArray(item.image_tags)
      ? item.image_tags.map(String).sort(compareStrings)
      : [];
    const stitchingTags = Array.isArray(item.stitching_tags)
      ? item.stitching_tags.map(String).sort(compareStrings)
      : [];
    const fallbackTags = Array.isArray(item.tags) ? item.tags.map(mapTagToString) : [];
    const flatTags =
      imageTags.length > 0 || stitchingTags.length > 0
        ? Array.from(new Set([...imageTags, ...stitchingTags]))
        : fallbackTags.sort(compareStrings);

    const folder = item.folder || extractFolder(String(item.filepath || ""));
    const id = Number(item.id);
    const dateAdded = item.date_added || (id ? new Date(id * 1000).toISOString() : "");

    const projectsRaw = Array.isArray(item?.projects)
      ? item.projects
      : Array.isArray(item.project_names)
        ? item.project_names
        : typeof item?.projects === "string"
          ? item.projects.split(",")
          : typeof item.project_names === "string"
            ? item.project_names.split(",")
            : [];

    const projects = projectsRaw
      .map(
        /** @param {string | { name?: string }} project */ (project) => {
          if (typeof project === "string") {
            return project.trim();
          }
          return String(project?.name || "").trim();
        }
      )
      .filter(Boolean);

    return {
      id,
      filename: String(item.filename || ""),
      filepath: String(item.filepath || ""),
      designer: String(item.designer || ""),
      source: String(item.source || ""),
      hoop: String(item.hoop || ""),
      rating: item.rating == null ? null : Number(item.rating),
      isStitched: Boolean(item.is_stitched),
      imageTagsVerified: Boolean(item.image_tags_verified),
      stitchingTagsVerified: Boolean(item.stitching_tags_verified),
      masterFilepath: item.master_filepath ?? null,
      isMasterOnly: Boolean(item.is_master_only),
      projects,
      imageTags,
      stitchingTags,
      tags: flatTags,
      folder,
      dateAdded,
    };
  }

  /** @param {keyof BrowseFilterState} key @param {BrowseFilterState[keyof BrowseFilterState]} value */
  function updateBrowseFilter(key, value) {
    browseFilters = {
      ...browseFilters,
      [key]: value,
    };
    browseCurrentPage = 1;
    // The backend is authoritative for filtering and sorting, so every filter
    // change must re-query it. The live `q` input is debounced so results
    // appear as the user types without a DB round-trip per keystroke; clearing
    // the query resets immediately.
    if (key === "q") {
      if (browseQTimer) {
        clearTimeout(browseQTimer);
        browseQTimer = null;
      }
      if (value) {
        browseQTimer = setTimeout(() => {
          browseQTimer = null;
          loadBrowseItems(true);
        }, BROWSE_Q_DEBOUNCE_MS);
        return;
      }
      loadBrowseItems(true);
      return;
    }
    loadBrowseItems(true);
  }

  function clearBrowseFilters() {
    browseFilters = defaultBrowseFilters();
    browseCurrentPage = 1;
    // A deliberate reset should not resurrect stale browse context on remount.
    browseSessionStore.clear();
    loadBrowseItems(true);
  }

  function applyBrowseFilters() {
    if (browseQTimer) {
      clearTimeout(browseQTimer);
      browseQTimer = null;
    }
    browseCurrentPage = 1;
    loadBrowseItems(true);
  }

  /**
   * @template T
   * @param {{ items?: T[] } | null | undefined} response
   * @returns {T[]}
   */
  function getResponseItems(response) {
    const items = response?.items;
    return Array.isArray(items) ? items : [];
  }

  let browseSearchRequestId = 0;

  function cancelSearch() {
    if (browseQTimer) {
      clearTimeout(browseQTimer);
      browseQTimer = null;
    }
    browseSearchRequestId++;
    browseLoading = false;
  }

  function clearSearchInput() {
    updateBrowseFilter("q", "");
  }

  /** @param {KeyboardEvent} event */
  function handleSearchKeyDown(event) {
    if (event.key === "Escape") {
      if (browseLoading) {
        event.preventDefault();
        cancelSearch();
      } else if (browseFilters.q) {
        event.preventDefault();
        clearSearchInput();
      }
    }
  }

  async function loadBrowseItems(force = false) {
    if (browseLoading && !force) return;

    browseLoading = true;
    const currentRequestId = ++browseSearchRequestId;
    try {
      const stitchedStatus = /** @type {"all" | "yes" | "no"} */ (
        browseFilters.stitched === "yes" || browseFilters.stitched === "no"
          ? browseFilters.stitched
          : "all"
      );

      /** @type {SearchPayload} */
      const payload = {
        q: browseFilters.q,
        search_file_name: browseFilters.searchFilename,
        search_tags: browseFilters.searchTags,
        search_folder_name: browseFilters.searchFolder,
        unverified_only: browseFilters.unverifiedOnly,
        page: browseCurrentPage,
        page_size: browsePageSize,
        sort_by: browseFilters.sortBy,
        sort_dir: browseFilters.sortDir,
        additional_filters: {
          designer_filters: Array.isArray(browseFilters.designerFilters)
            ? browseFilters.designerFilters
            : [],
          image_tag_filters: Array.isArray(browseFilters.imageTagFilters)
            ? browseFilters.imageTagFilters
            : [],
          stitching_tag_filters: Array.isArray(browseFilters.stitchingTagFilters)
            ? browseFilters.stitchingTagFilters
            : [],
          source_filters: Array.isArray(browseFilters.sourceFilters)
            ? browseFilters.sourceFilters
            : [],
          hoop_size: browseFilters.hoop || null,
          min_width:
            browseFilters.minWidth.trim() !== "" && !Number.isNaN(Number(browseFilters.minWidth))
              ? Number(browseFilters.minWidth)
              : null,
          max_width:
            browseFilters.maxWidth.trim() !== "" && !Number.isNaN(Number(browseFilters.maxWidth))
              ? Number(browseFilters.maxWidth)
              : null,
          min_height:
            browseFilters.minHeight.trim() !== "" && !Number.isNaN(Number(browseFilters.minHeight))
              ? Number(browseFilters.minHeight)
              : null,
          max_height:
            browseFilters.maxHeight.trim() !== "" && !Number.isNaN(Number(browseFilters.maxHeight))
              ? Number(browseFilters.maxHeight)
              : null,
          min_rating: browseFilters.rating ? Number(browseFilters.rating) : null,
          stitched_status: stitchedStatus,
          needs_attention: browseFilters.needsAttention,
        },
      };
      const [result, fullIds] = await Promise.all([
        getBrowseDesigns(payload),
        Promise.resolve(getDesignIds(payload)).catch(() => []),
      ]);
      if (currentRequestId !== browseSearchRequestId) {
        return;
      }
      const rawItems = getResponseItems(result);
      const normalizedItems = rawItems.map(normalizeCardItem).filter((item) => item !== null);
      browseItems = /** @type {BrowseDesignCard[]} */ (normalizedItems);
      browseTotal = Math.max(0, Number(result?.total ?? 0));
      browseTotalPages = Math.max(1, Number(result?.total_pages ?? 1));
      browseCurrentPage = Math.max(1, Number(result?.page ?? browseCurrentPage));
      browseHasLoaded = true;
      // The full filtered id list drives the detail view's Prev/Next across the
      // whole result set. If the ids query is unavailable/empty, fall back to
      // the current page's ids (previous behaviour).
      browseDesignIds =
        Array.isArray(fullIds) && fullIds.length > 0
          ? fullIds.map(Number).filter((id) => Number.isFinite(id))
          : normalizedItems.map((item) => item.id).filter((id) => Number.isFinite(id));
      restoreBrowseScrollOnce();
    } catch {
      if (currentRequestId !== browseSearchRequestId) {
        return;
      }
      browseHasLoaded = true;
      browseItems = [];
      browseTotal = 0;
      browseTotalPages = 1;
    } finally {
      if (currentRequestId === browseSearchRequestId) {
        browseLoading = false;
      }
    }
  }

  async function loadBrowseTags() {
    try {
      const result = await getBrowseTags();
      browseTagOptions = getResponseItems(result);
    } catch (error) {
      browseTagOptions = [];
      console.info("Could not load browse tags list", error);
    } finally {
      browseTagsLoaded = true;
    }
  }

  async function loadBrowseProjects() {
    try {
      const result = await getBrowseProjects();
      const items = [...getResponseItems(result)];
      items.sort((a, b) =>
        (a.name || "").localeCompare(b.name || "", undefined, { sensitivity: "base" })
      );
      browseProjects = items;
      browseProjectsLoaded = true;
    } catch (error) {
      browseProjects = [];
      console.info("Could not load projects list", error);
    }
  }

  async function loadBrowseFilterReferenceData() {
    try {
      const [designerResult, sourceResult, hoopResult] = await Promise.all([
        listDesigners(),
        listSources(),
        listHoops(),
      ]);

      const designerItems = getResponseItems(designerResult);
      const sourceItems = getResponseItems(sourceResult);
      const hoopItems = getResponseItems(hoopResult);

      browseDesignerFilterOptions = Array.from(
        new Set(designerItems.map((item) => String(item?.name || "").trim()).filter(Boolean))
      ).sort((a, b) => a.localeCompare(b));

      browseSourceFilterOptions = Array.from(
        new Set(sourceItems.map((item) => String(item?.name || "").trim()).filter(Boolean))
      ).sort((a, b) => a.localeCompare(b));

      browseHoopFilterOptions = Array.from(
        new Set(hoopItems.map((item) => String(item?.name || "").trim()).filter(Boolean))
      ).sort((a, b) => a.localeCompare(b));
    } catch (error) {
      browseDesignerFilterOptions = [];
      browseSourceFilterOptions = [];
      browseHoopFilterOptions = [];
      console.info("Could not load browse filter reference data", error);
    } finally {
      browseFilterReferenceLoaded = true;
    }
  }

  /** @param {number[]} designIds */
  async function loadBrowsePreviews(designIds) {
    const ids = Array.isArray(designIds)
      ? Array.from(
          new Set(designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0))
        )
      : [];

    if (ids.length === 0) {
      browsePreviewsLoading = false;
      return;
    }

    const missingIds = ids.filter((id) => !(id in browsePreviewById));
    if (missingIds.length === 0) {
      browsePreviewsLoading = false;
      return;
    }

    const requestId = browsePreviewRequestCounter + 1;
    browsePreviewRequestCounter = requestId;

    browsePreviewsLoading = true;
    try {
      const result = await getBrowseDesignPreviews(missingIds);
      if (requestId !== browsePreviewRequestCounter) return;

      const map = { ...browsePreviewById };
      const returnedIds = new Set();
      for (const item of result.items || []) {
        if (Number.isFinite(Number(item?.id))) {
          returnedIds.add(Number(item.id));
          map[Number(item.id)] = item?.data_url || null;
        }
      }

      for (const id of missingIds) {
        if (!returnedIds.has(id) && !(id in map)) {
          map[id] = null;
        }
      }

      browsePreviewById = map;
    } catch (error) {
      console.info("Could not load browse previews", error);
      if (requestId === browsePreviewRequestCounter) {
        const nextMap = { ...browsePreviewById };
        for (const id of missingIds) {
          if (!(id in nextMap)) {
            nextMap[id] = null;
          }
        }
        browsePreviewById = nextMap;
      }
    } finally {
      if (requestId === browsePreviewRequestCounter) {
        browsePreviewsLoading = false;
      }
    }
  }

  /**
   * Apply accumulated session patches to the browse item list.
   * Patches individual card data in-place so only affected cards re-render.
   * Also invalidates cached previews for patched designs.
   * @param {Record<number, MutationPatch>} patches
   */
  function applyPatchesToBrowse(patches) {
    for (const [idStr, patch] of Object.entries(patches)) {
      const id = Number(idStr);
      const index = browseItems.findIndex((item) => item.id === id);
      if (index !== -1) {
        const { hoop, ...restPatch } = patch;
        browseItems[index] = {
          ...browseItems[index],
          ...restPatch,
          ...(hoop !== undefined ? { hoop: hoop ?? "" } : {}),
        };
      }

      // Invalidate cached preview for this card so it re-fetches if needed
      if (id in browsePreviewById) {
        const nextPreviews = { ...browsePreviewById };
        delete nextPreviews[id];
        browsePreviewById = nextPreviews;
      }
    }
  }

  // Derived Browse Computations
  // The backend is now the single source of truth for filtering, sorting, and
  // pagination — `browseItems` already holds the fully-filtered current page,
  // and `browseTotal`/`browseTotalPages` come from the backend COUNT query.
  let browsePageSize = $derived(Math.max(1, (browseGridColumns || 5) * BROWSE_PAGE_ROWS));
  let browsePageItems = $derived(browseItems);

  let browseSelectedCount = $derived(browseSelectedIds.size);
  let browseSelectionLocked = $derived(browseDeleteConfirmOpen);
  let showBrowseBulkBar = $derived(browseSelectedCount > 0);

  let totalFilteredCount = $derived(browseTotal);
  let totalCountOnPage = $derived(browsePageItems.length);
  let selectedCountOnPage = $derived(
    browsePageItems.filter((item) => browseSelectedIds.has(item.id)).length
  );
  let isAllSelectedOnPage = $derived(
    totalCountOnPage > 0 && selectedCountOnPage === totalCountOnPage
  );

  /**
   * @param {number | string} id
   * @param {boolean} checked
   */
  function toggleBrowseCardSelection(id, checked) {
    const targetId = Number(id);
    if (checked) {
      // Silently ignore if at the selection cap and not already selected
      if (browseSelectedIds.size >= BROWSE_BULK_DELETE_MAX && !browseSelectedIds.has(targetId)) {
        return;
      }
      browseSelectedIds.add(targetId);
    } else {
      browseSelectedIds.delete(targetId);
    }
  }

  /** @param {boolean} checked */
  function toggleSelectAllBrowseOnPage(checked) {
    if (checked) {
      const selected = new Set(browseSelectedIds);
      for (const item of browsePageItems) {
        if (selected.size >= BROWSE_BULK_DELETE_MAX) break;
        if (!selected.has(item.id)) {
          selected.add(item.id);
        }
      }
      browseSelectedIds = new SvelteSet(selected);
    } else {
      for (const item of browsePageItems) {
        browseSelectedIds.delete(item.id);
      }
    }
  }

  function toggleAdditionalFilters() {
    browseAdditionalFiltersOpen = !browseAdditionalFiltersOpen;
  }

  /** @param {number} width */
  function estimateBrowseColumnsFromWidth(width) {
    const normalizedWidth = Number(width) || 0;
    if (normalizedWidth >= BROWSE_BREAKPOINT_LG) {
      return 5;
    }
    if (normalizedWidth >= BROWSE_BREAKPOINT_MD) {
      return 4;
    }
    if (normalizedWidth >= BROWSE_BREAKPOINT_SM) {
      return 3;
    }
    return 2;
  }

  function refreshBrowseGridColumns() {
    if (typeof window !== "undefined") {
      browseGridColumns = estimateBrowseColumnsFromWidth(window.innerWidth || 0);
      return;
    }

    if (browseGridContainer) {
      const containerWidth = browseGridContainer.clientWidth;
      if (containerWidth && containerWidth > 0) {
        browseGridColumns = estimateBrowseColumnsFromWidth(
          Math.max(0, containerWidth + BROWSE_ROW_SELECTOR_WIDTH)
        );
        return;
      }
    }

    browseGridColumns = 2;
  }

  /** @param {keyof BrowseFilterState} key @param {string} filterValue */
  function toggleBrowseFilter(key, filterValue) {
    const raw = browseFilters[/** @type {keyof typeof browseFilters} */ (key)];
    const list = /** @type {string[]} */ (Array.isArray(raw) ? [...raw] : []);
    const val = String(filterValue || "").trim();
    if (!val) return;

    let next;
    if (list.includes(val)) {
      next = list.filter((item) => item !== val);
    } else {
      next = [...list, val];
    }

    updateBrowseFilter(key, next);
  }

  // Bulk Actions
  function openBulkTagModal() {
    if (browseSelectedIds.size === 0) return;

    const selectedDesigns = browseItems.filter((item) => browseSelectedIds.has(item.id));
    const totalSelected = selectedDesigns.length;
    const checkedIds = /** @type {Array<number | string>} */ ([]);
    const indeterminateIds = /** @type {Array<number | string>} */ ([]);

    if (totalSelected > 0 && browseTagOptions.length > 0) {
      for (const tagOption of browseTagOptions) {
        const tagId = Number(tagOption.id);
        if (!Number.isFinite(tagId)) continue;
        const desc = String(tagOption.description || "")
          .trim()
          .toLowerCase();

        let count = 0;
        for (const design of selectedDesigns) {
          if (
            Array.isArray(design.tags) &&
            design.tags.some(
              /** @param {unknown} t */ (t) =>
                String(t || "")
                  .trim()
                  .toLowerCase() === desc
            )
          ) {
            count++;
          }
        }

        if (count === totalSelected) {
          checkedIds.push(tagId);
        } else if (count > 0 && count < totalSelected) {
          indeterminateIds.push(tagId);
        }
      }
    }

    // Build tag id → group lookup so we can classify each add/remove diff by
    // category (image vs stitching) for the verification payload (Rules 2 & 3).
    const tagGroupById = /** @type {Record<string | number, string>} */ ({});
    for (const tagOption of browseTagOptions) {
      tagGroupById[Number(tagOption.id)] = String(tagOption.tag_group || "image");
    }

    // Per-category uniformity: whether every selected design shares the exact
    // same tag set within that category. Uniform ⇒ Rule 2 (mark verified);
    // mixed ⇒ Rule 3 (only mark verified when that category actually changed).
    const evenlySortedImage = (/** @type {string[]} */ tags) => [...tags].sort(compareStrings);
    const evenlySortedStitching = (/** @type {string[]} */ tags) => [...tags].sort(compareStrings);
    const firstSelected = selectedDesigns[0];
    const imageUniform =
      totalSelected > 0 &&
      selectedDesigns.every(
        (/** @type {BrowseDesignCard} */ design) =>
          evenlySortedImage(design.imageTags).join("\u0000") ===
          evenlySortedImage(firstSelected.imageTags).join("\u0000")
      );
    const stitchingUniform =
      totalSelected > 0 &&
      selectedDesigns.every(
        (/** @type {BrowseDesignCard} */ design) =>
          evenlySortedStitching(design.stitchingTags).join("\u0000") ===
          evenlySortedStitching(firstSelected.stitchingTags).join("\u0000")
      );

    browseBulkImageUniform = imageUniform;
    browseBulkStitchingUniform = stitchingUniform;
    browseBulkTagGroupById = tagGroupById;
    browseBulkTagAddIds = checkedIds;
    browseBulkTagRemoveIds = [];
    browseBulkTagIndeterminateIds = indeterminateIds;
    browseBulkClearAll = false;
    browseBulkModalOpen = true;
  }

  function closeBulkTagModal() {
    browseBulkModalOpen = false;
  }

  /**
   * Resolve the effective tri-state for a tag:
   * - "add"           → checked [✓] → tag added to all selected designs.
   * - "remove"        → unchecked [ ] → tag removed from all selected designs.
   * - "indeterminate" → mixed [-] → tag left completely untouched.
   * - "none"          → not present on any selected design
   * @param {number | string} tagId
   * @returns {"add" | "remove" | "indeterminate" | "none"}
   */
  function tagChooserState(tagId) {
    const id = Number(tagId);
    if (browseBulkTagAddIds.includes(id)) return "add";
    if (browseBulkTagRemoveIds.includes(id)) return "remove";
    if (browseBulkTagIndeterminateIds.includes(id)) return "indeterminate";
    return "none";
  }

  /**
   * Cycle a tri-state checkbox: [-] → [✓] → [ ] → [✓].
   * This derives the next state from the *current* state, so untouched
   * mixed tags are never silently dropped.
   * @param {number | string} tagId
   */
  function toggleTagChooserSelection(tagId) {
    const id = Number(tagId);
    if (!Number.isFinite(id)) return;

    const current = tagChooserState(id);

    // Remove the tag from every list first, then move it to its target list.
    browseBulkTagAddIds = browseBulkTagAddIds.filter((value) => value !== id);
    browseBulkTagRemoveIds = browseBulkTagRemoveIds.filter((value) => value !== id);
    browseBulkTagIndeterminateIds = browseBulkTagIndeterminateIds.filter((value) => value !== id);

    // [-] (mixed) or [ ] (remove) or unlisted → [✓] (add).
    if (current === "add") {
      // [✓] → [ ] (remove)
      browseBulkTagRemoveIds = [...browseBulkTagRemoveIds, id];
    } else {
      // anything else → [✓] (add)
      browseBulkTagAddIds = [...browseBulkTagAddIds, id];
    }
  }

  /**
   * Visual glyph for a tag's tri-state on the tag chooser buttons.
   * @param {number | string} tagId
   * @returns {string}
   */
  function tagChooserGlyph(tagId) {
    const state = tagChooserState(tagId);
    if (state === "add") return "✓";
    if (state === "indeterminate") return "−";
    return "";
  }

  /**
   * ARIA representation for a tag's tri-state.
   * @param {number | string} tagId
   * @returns {"true" | "false" | "mixed"}
   */
  function tagChooserAria(tagId) {
    const state = tagChooserState(tagId);
    if (state === "add") return "true";
    if (state === "indeterminate") return "mixed";
    return "false";
  }

  async function applyBulkTags() {
    if (browseSelectedIds.size === 0) return;

    const clearAll = browseBulkClearAll;
    // When 'Untagged' is active, `clear_all_tags` wipes every original tag and
    // any tags the user ticked afterwards become the replacement set, so we must
    // NOT drop browseBulkTagAddIds just because clearAll is true.
    const addIds = browseBulkTagAddIds;

    // Classify the per-category changes so we only set verification flags for
    // categories that were actually touched by this batch, per the decision
    // matrix:
    //   single design            → both categories verified
    //   multiple, uniform        → that category verified (Rule 2)
    //   multiple, mixed + change → that category verified (Rule 3)
    //   multiple, mixed, untouched → category flag left unchanged (undefined)
    const isSingle = browseSelectedIds.size === 1;
    const categoryChanged = (/** @type {Array<number | string>} */ ids) =>
      clearAll || ids.some((id) => browseBulkTagGroupById[Number(id)] !== "stitching");
    const stitchingCategoryChanged = (/** @type {Array<number | string>} */ ids) =>
      clearAll || ids.some((id) => browseBulkTagGroupById[Number(id)] === "stitching");

    // Note: unclassified tags are treated as image-category for verification
    // purposes (they are "what the design depicts").
    const imageChanged =
      categoryChanged(browseBulkTagAddIds) || categoryChanged(browseBulkTagRemoveIds);
    const stitchingChanged =
      stitchingCategoryChanged(browseBulkTagAddIds) ||
      stitchingCategoryChanged(browseBulkTagRemoveIds);

    let imageTagsVerified;
    let stitchingTagsVerified;
    if (isSingle) {
      imageTagsVerified = true;
      stitchingTagsVerified = true;
    } else {
      if (browseBulkImageUniform) {
        imageTagsVerified = true;
      } else {
        imageTagsVerified = imageChanged ? true : undefined;
      }
      if (browseBulkStitchingUniform) {
        stitchingTagsVerified = true;
      } else {
        stitchingTagsVerified = stitchingChanged ? true : undefined;
      }
    }
    if (clearAll) {
      imageTagsVerified = true;
      stitchingTagsVerified = true;
    }

    browseLoading = true;
    beginBusy("Applying bulk tags");
    try {
      const result = /** @type {BulkSetTagsResult} */ (
        await bulkSetTagsForDesigns(
          Array.from(browseSelectedIds),
          addIds,
          browseBulkTagRemoveIds,
          clearAll,
          { imageTagsVerified, stitchingTagsVerified }
        )
      );
      if (result?.persisted) {
        addToast(
          `Updated tags for ${result.updated_count ?? result.updated} design(s).`,
          "success"
        );
        closeBulkTagModal();
        await loadBrowseItems(true);
      } else {
        addToast(result?.error || "Could not bulk update tags.", "error");
        closeBulkTagModal();
      }
    } catch (e) {
      addToast(`Bulk tagging failed: ${e}`, "error");
      closeBulkTagModal();
    } finally {
      browseLoading = false;
      endBusy();
    }
  }

  async function applySharedTagChooser() {
    await applyBulkTags();
  }

  function openBulkProjectModal() {
    if (browseSelectedIds.size === 0) return;
    browseBulkProjectSelection = [];
    browseBulkProjectDropdownOpen = true;
    if (browseProjects.length === 0 && !browseProjectsLoaded) {
      loadBrowseProjects();
    }
  }

  function closeBulkProjectModal() {
    browseBulkProjectDropdownOpen = false;
  }

  /** @param {number | string} projectId @param {boolean} checked */
  function toggleBrowseBulkProjectSelection(projectId, checked) {
    const id = Number(projectId);
    if (!Number.isFinite(id)) return;
    if (checked) {
      browseBulkProjectSelection = Array.from(new Set([...browseBulkProjectSelection, id]));
    } else {
      browseBulkProjectSelection = browseBulkProjectSelection.filter((v) => v !== id);
    }
  }

  async function addSelectedToProject() {
    if (browseSelectedIds.size === 0 || browseBulkProjectSelection.length === 0) return;

    browseLoading = true;
    let totalAdded = 0;
    let anyFailed = false;
    beginBusy("Adding designs to projects");
    try {
      for (const projectId of browseBulkProjectSelection) {
        const result = /** @type {BulkAddToProjectResult} */ (
          await bulkAddDesignsToProject(projectId, Array.from(browseSelectedIds))
        );
        if (result?.persisted) {
          totalAdded += result.added_count ?? result.updated ?? 0;
        } else {
          anyFailed = true;
        }
      }
      addToast(
        anyFailed
          ? `Some projects could not be updated. ${totalAdded} design(s) added to project(s).`
          : `${totalAdded} design(s) added to project(s).`,
        anyFailed ? "warning" : "success"
      );
      closeBulkProjectModal();
      await loadBrowseItems(true);
    } catch (e) {
      addToast(`Bulk project add failed: ${e}`, "error");
    } finally {
      browseLoading = false;
      endBusy();
    }
  }

  async function runBulkVerify() {
    if (browseSelectedIds.size === 0) return;

    browseLoading = true;
    beginBusy("Verifying designs");
    try {
      const result = /** @type {BulkVerifyResult} */ (
        await bulkVerifyDesigns(Array.from(browseSelectedIds))
      );
      if (result?.persisted) {
        addToast(
          `${result.verified_count ?? result.updated} design(s) marked verified.`,
          "success"
        );
        await loadBrowseItems(true);
      } else {
        addToast(result?.error || "Could not verify designs.", "error");
      }
    } catch (e) {
      addToast(`Verification failed: ${e}`, "error");
    } finally {
      browseLoading = false;
      endBusy();
    }
  }

  function openBrowseDeleteConfirm() {
    if (browseSelectedIds.size === 0) return;
    browseDeleteConfirmOpen = true;
  }

  function closeBrowseDeleteConfirm() {
    browseDeleteConfirmOpen = false;
  }

  /** @param {BulkDeleteResult} result */
  function handleBulkDeleteResult(result) {
    if (result.persisted) {
      let notice = `${result.deleted_count} design(s) deleted from catalogue.`;
      if (result.files_trashed > 0) {
        notice += ` ${result.files_trashed} source file(s) moved to recycle bin.`;
      }
      if (result.errors && result.errors.length > 0) {
        notice += ` (${result.errors.length} file warning(s) — see console for details)`;
        console.warn("Bulk delete file warnings:", result.errors);
      }
      addToast(notice, "success");
    } else {
      addToast(result.errors?.[0] || "Bulk delete failed.", "error");
    }
    browseSelectedIds.clear();
    browseDeleteConfirmOpen = false;
    loadBrowseItems(true);
  }

  function clearBrowseSelection() {
    browseSelectedIds.clear();
  }

  /** @param {BrowseDesignCard} item @param {HTMLElement | null} summaryNode */
  function handleBrowseCardProjectDetailsToggle(item, summaryNode) {
    const detailsNode = /** @type {Element | null} */ (summaryNode?.parentNode);
    if (detailsNode && detailsNode.hasAttribute("open") && browseProjects.length === 0) {
      loadBrowseProjects();
    }
  }

  /** @param {BrowseDesignCard} item @param {number | string} projectId */
  function isBrowseCardProjectChecked(item, projectId) {
    const designId = Number(item.id);
    const prjId = Number(projectId);
    const pendingVal = browseCardProjectPendingById?.[designId]?.[prjId];
    if (pendingVal !== undefined) {
      return pendingVal;
    }
    if (!Array.isArray(item.projects)) return false;

    if (item.projects.includes(String(projectId))) {
      return true;
    }

    const targetProject = browseProjects.find((p) => Number(p.id) === prjId);
    if (targetProject && targetProject.name) {
      const targetName = String(targetProject.name).trim().toLowerCase();
      return item.projects.some((p) => String(p).trim().toLowerCase() === targetName);
    }
    return false;
  }

  /** @param {number | string} designId @param {number | string} projectId @param {boolean} checked */
  function updateBrowseCardProjectPending(designId, projectId, checked) {
    const targetDesignId = Number(designId);
    const targetProjectId = Number(projectId);
    const existing = browseCardProjectPendingById?.[targetDesignId] || {};
    browseCardProjectPendingById = {
      ...browseCardProjectPendingById,
      [targetDesignId]: {
        ...existing,
        [targetProjectId]: Boolean(checked),
      },
    };
    applyBrowseCardProjectPending(targetDesignId);
  }

  /** @param {number | string} designId */
  async function applyBrowseCardProjectPending(designId) {
    const targetDesignId = Number(designId);
    const pending = browseCardProjectPendingById?.[targetDesignId] || {};
    const projectIds = Object.keys(pending)
      .map(Number)
      .filter((id) => pending[id]);

    for (const prjId of projectIds) {
      await addDesignToProject(targetDesignId, prjId);
    }

    const removedProjectIds = Object.keys(pending)
      .map(Number)
      .filter((id) => !pending[id]);

    for (const prjId of removedProjectIds) {
      await removeDesignFromProject(targetDesignId, prjId);
    }

    browseCardProjectPendingById = {
      ...browseCardProjectPendingById,
      [targetDesignId]: {},
    };
    await loadBrowseItems(true);
  }

  function getBrowseCardProjectDropdowns() {
    if (typeof document === "undefined") return [];
    return Array.from(document.querySelectorAll(".browse-card-project-details"));
  }

  function closeBrowseCardProjectDropdowns() {
    for (const dropdown of getBrowseCardProjectDropdowns()) {
      dropdown.removeAttribute("open");
    }
  }

  /** @param {{ id: number | string }} item */
  function openDesignDetail(item) {
    const designId = Number(item.id);
    if (!Number.isFinite(designId) || designId <= 0) return;

    // The full filtered id list is already in the browse session store (kept
    // fresh by loadBrowseItems); MainView derives the detail view's Prev/Next
    // context from it. Capture scroll so it can be restored on return.
    captureBrowseScroll();
    navigateTo(`#/designs/${item.id}`);
  }

  /** @param {MouseEvent} event @param {BrowseDesignCard | { id: number | string }} item */
  function handleBrowseCardOpenDetail(event, item) {
    const anyProjectDropdownOpen = getBrowseCardProjectDropdowns().some((dropdown) =>
      dropdown.hasAttribute("open")
    );
    if (anyProjectDropdownOpen) {
      event.preventDefault();
      event.stopPropagation();
      closeBrowseCardProjectDropdowns();
      return;
    }
    openDesignDetail(item);
  }

  // Reactive effects for routing/loading

  $effect(() => {
    // Apply any accumulated session patches from DesignDetails edits
    const patches = designSessionStore.consumePatches();
    if (Object.keys(patches).length > 0) {
      untrack(() => {
        applyPatchesToBrowse(patches);
      });
    }

    // Refresh browse data affected by tag admin mutations.
    // Deleted tags (or renamed tags used by designs) require a full card
    // reload; otherwise only the tag filter options need updating.
    const tagChanges = tagChangeStore.consumeFlags();
    if (tagChanges.designsNeedRefresh) {
      untrack(() => {
        loadBrowseItems(true);
        loadBrowseTags();
      });
    } else if (tagChanges.tagsNeedRefresh) {
      untrack(() => {
        loadBrowseTags();
      });
    }

    // Full reload still needed for import/deletion flows
    if (!browseHasLoaded || browseNeedsRefresh) {
      untrack(() => {
        loadBrowseItems(true);
        browseNeedsRefresh = false;
      });
    }
  });

  $effect(() => {
    if (!browseTagsLoaded) {
      untrack(() => {
        loadBrowseTags();
      });
    }
  });

  $effect(() => {
    if (!browseFilterReferenceLoaded) {
      untrack(() => {
        loadBrowseFilterReferenceData();
      });
    }
  });

  $effect(() => {
    if (!browseProjectsLoaded) {
      untrack(() => {
        loadBrowseProjects();
      });
    }
  });

  $effect(() => {
    const ids = browsePageItems.map((item) => item.id);
    untrack(() => {
      loadBrowsePreviews(ids);
    });
  });

  $effect(() => {
    void browseTotal;
    tick().then(() => {
      untrack(() => {
        refreshBrowseGridColumns();
      });
    });
  });

  $effect(() => {
    if (browseCurrentPage > browseTotalPages) {
      browseCurrentPage = browseTotalPages;
    }
    if (browseCurrentPage < 1) {
      browseCurrentPage = 1;
    }
  });

  $effect(() => {
    const validIds = new Set(browseItems.map((item) => item.id));
    for (const id of browseSelectedIds) {
      if (!validIds.has(id)) {
        browseSelectedIds.delete(id);
      }
    }
  });

  let browsePageRows = $derived(
    (() => {
      const columns = Math.max(1, browseGridColumns || 1);
      const rows = [];
      for (let index = 0; index < browsePageItems.length; index += columns) {
        rows.push(browsePageItems.slice(index, index + columns));
      }
      return rows;
    })()
  );

  /** @param {BrowseDesignCard[]} rowItems */
  function isBrowseRowFullySelected(rowItems) {
    return rowItems.length > 0 && rowItems.every((item) => browseSelectedIds.has(item.id));
  }

  /** @param {BrowseDesignCard[]} rowItems */
  function toggleBrowseRowSelection(rowItems) {
    const allSelected = isBrowseRowFullySelected(rowItems);
    if (allSelected) {
      for (const item of rowItems) {
        browseSelectedIds.delete(item.id);
      }
    } else {
      for (const item of rowItems) {
        browseSelectedIds.add(item.id);
      }
    }
  }

  // Keep the browse scroll position in sync with the store so it survives the
  // detail round-trip and menu navigation. A passive window scroll listener
  // writes the latest position on every scroll; restoreBrowseScrollOnce()
  // (called from loadBrowseItems after the cards render) reads it back on
  // return. captureBrowseScroll() in openDesignDetail also captures the exact
  // position at click time as a final safety net.
  onMount(() => {
    if (typeof window !== "undefined") {
      window.addEventListener("scroll", captureBrowseScroll, { passive: true });
    }
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("scroll", captureBrowseScroll);
    }
    if (browseQTimer) {
      clearTimeout(browseQTimer);
      browseQTimer = null;
    }
    browseSearchRequestId++;
  });
</script>

<svelte:window onresize={refreshBrowseGridColumns} />

<section class="browse-section space-y-4">
  <FirstImportSuccessBanner />
  <h1 class="ui-page-title browse-title text-2xl font-bold text-gray-800">Browse Designs</h1>
  <br />
  <BrowseFilterPanel
    {browseFilters}
    {browseLoading}
    {browseAdditionalFiltersOpen}
    {browseFiltersAreDefault}
    {browseDesignerFilterOptions}
    {browseImageTagOptions}
    {browseStitchingTagOptions}
    {browseSourceFilterOptions}
    {browseHoopFilterOptions}
    onUpdateFilter={updateBrowseFilter}
    onToggleFilter={toggleBrowseFilter}
    onToggleAdditionalFilters={toggleAdditionalFilters}
    onClearFilters={clearBrowseFilters}
    onCancelSearch={cancelSearch}
    onClearSearchInput={clearSearchInput}
    onSearchKeyDown={handleSearchKeyDown}
    onApplyFilters={applyBrowseFilters}
  />

  <SelectionHeader
    {totalFilteredCount}
    {selectedCountOnPage}
    {totalCountOnPage}
    {isAllSelectedOnPage}
    onToggleSelectAllPage={toggleSelectAllBrowseOnPage}
    busyActive={busyActive || browseLoading}
    isSearching={browseLoading}
  />

  <!-- Browse Results Grid -->
  <BrowseCardGrid
    {browseItems}
    {browseLoading}
    {browsePageRows}
    {browseGridColumns}
    {browseSelectedIds}
    {browseSelectionLocked}
    browseBulkDeleteMax={BROWSE_BULK_DELETE_MAX}
    {browsePreviewById}
    {browsePreviewsLoading}
    {browseProjects}
    bind:browseGridContainer
    onCancelSearch={cancelSearch}
    {isBrowseRowFullySelected}
    onToggleBrowseRowSelection={toggleBrowseRowSelection}
    onToggleBrowseCardSelection={toggleBrowseCardSelection}
    onHandleBrowseCardOpenDetail={handleBrowseCardOpenDetail}
    onHandleBrowseCardProjectDetailsToggle={handleBrowseCardProjectDetailsToggle}
    {isBrowseCardProjectChecked}
    onUpdateBrowseCardProjectPending={updateBrowseCardProjectPending}
  />

  <!-- Pagination -->
  <Pagination
    currentPage={browseCurrentPage}
    totalPages={browseTotalPages}
    onPageChange={(/** @type {number} */ page) => {
      browseCurrentPage = page;
      loadBrowseItems(true);
    }}
    disabled={browseLoading || busyActive}
    showFirstLast={true}
    windowSize={2}
    ariaLabel="Browse pagination"
  />
</section>

<!-- Bulk Actions Bar (Sticky Bottom) -->
<BrowseSelectionBar
  {showBrowseBulkBar}
  {browseSelectedCount}
  {browseBulkProjectDropdownOpen}
  {browseProjects}
  {browseBulkProjectSelection}
  bind:browseBulkBarNode
  onOpenBulkTagModal={openBulkTagModal}
  onRunBulkVerify={runBulkVerify}
  onOpenBulkProjectModal={openBulkProjectModal}
  onCloseBulkProjectModal={closeBulkProjectModal}
  onToggleBulkProjectSelection={toggleBrowseBulkProjectSelection}
  onAddSelectedToProject={addSelectedToProject}
  onOpenBrowseDeleteConfirm={openBrowseDeleteConfirm}
  onClearBrowseSelection={clearBrowseSelection}
/>

<!-- Browse Bulk Tag Modal -->
{#if browseBulkModalOpen}
  {@const groupedTagOptions = browseGroupedTagOptions}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="bulk-tag-title"
  >
    <button
      type="button"
      class="tag-chooser-backdrop"
      aria-label="Close tag chooser"
      onclick={closeBulkTagModal}
    ></button>
    <div class="tag-chooser-dialog">
      <div class="tag-chooser-header">
        <h2 id="bulk-tag-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Choose tags for selected designs
        </h2>
      </div>
      <div class="tag-chooser-body">
        <p class="text-xs text-gray-500 font-semibold" style="margin:0 0 0.75rem 0;">
          {browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.
        </p>

        <div class="tag-chooser-section" style="margin-bottom:0.75rem;">
          <label class="tag-chooser-option" style="font-weight:600;">
            <input
              type="checkbox"
              checked={browseBulkClearAll}
              onchange={(event) => {
                browseBulkClearAll = event.currentTarget.checked;
                // Untagged = replace mode: wipe every on-screen tag selection so
                // the design will end up empty. Any tag ticked afterwards becomes
                // the new (replacement) tag set, applied only when 'Apply tags'
                // is pressed. Cancel never touches the database.
                browseBulkTagAddIds = [];
                browseBulkTagRemoveIds = [];
                browseBulkTagIndeterminateIds = [];
              }}
            />
            <span>Untagged (clear all tags)</span>
          </label>
        </div>

        <div class="tag-chooser-sections">
          {#if groupedTagOptions.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image font-semibold">
                Image tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.image as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    onclick={() => toggleTagChooserSelection(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.stitching.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-stitching font-semibold"
              >
                Stitching tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.stitching as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    onclick={() => toggleTagChooserSelection(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-unclassified font-semibold"
              >
                Unclassified tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.unclassified as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    onclick={() => toggleTagChooserSelection(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}
        </div>
      </div>
      <div class="tag-chooser-footer">
        <button type="button" class="menu-button-secondary" onclick={closeBulkTagModal}
          >Cancel</button
        >
        <button type="button" class="menu-button-primary" onclick={applySharedTagChooser}>
          Apply tags
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Shared Delete Modal -->
<DeleteDesignsModal
  designIds={Array.from(browseSelectedIds)}
  previewItems={browseItems
    .filter((item) => browseSelectedIds.has(item.id))
    .map((item) => ({
      id: item.id,
      filename: item.filename,
      filepath: item.filepath,
      dataUrl: browsePreviewById[item.id] ?? null,
    }))}
  open={browseDeleteConfirmOpen}
  onClose={closeBrowseDeleteConfirm}
  onDeleted={handleBulkDeleteResult}
/>
