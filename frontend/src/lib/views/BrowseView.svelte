<script>
  import { tick, untrack } from "svelte";
  import {
    getBrowseDesigns,
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
  import Pagination from "../components/Pagination.svelte";
  import SelectionHeader from "../components/SelectionHeader.svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { designSessionStore } from "../stores/designSessionStore.js";
  import { tagChangeStore } from "../stores/tagChangeStore.js";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  import { portalToBody } from "../utils/portal.js";
  import { HOOP_UNKNOWN_FILTER } from "../utils/hoopConstants.js";

  /** @typedef {import("../types/ipc").BrowseDesignCard} BrowseDesignCard */
  /** @typedef {import("../types/ipc").BrowseDesignSummaryWire} BrowseDesignSummaryWire */
  /** @typedef {import("../types/ipc").BrowseTagOption} BrowseTagOption */
  /** @typedef {import("../types/ipc").ProjectListItem} ProjectListItem */
  /** @typedef {import("../types/ipc").SearchPayload} SearchPayload */
  /** @typedef {import("../types/ipc").MutationPatch} MutationPatch */
  /** @typedef {{ persisted: boolean, deleted_count: number, files_trashed: number, errors?: Array<string> }} BulkDeleteResult */
  /** @typedef {{ q: string, allWords: string, exactPhrase: string, anyWords: string, noneWords: string, filename: string, designerFilters: Array<string>, imageTagFilters: Array<string>, stitchingTagFilters: Array<string>, hoop: string, sourceFilters: Array<string>, rating: string, stitched: string, unverifiedOnly: boolean, searchFilename: boolean, searchTags: boolean, searchFolder: boolean, sortBy: string, sortDir: string }} BrowseFilterState */
  /** @typedef {Omit<BrowseDesignSummaryWire, "projects" | "tags"> & { projects?: Array<string | { name?: string }> | string, tags?: Array<string | { description?: string }>, project_names?: Array<string> | string, folder?: string, date_added?: string | null }} BrowseCardInput */
  /** @typedef {{ persisted: boolean, updated_count?: number, updated?: number, error?: string }} BulkSetTagsResult */
  /** @typedef {{ persisted: boolean, added_count?: number, updated?: number, error?: string }} BulkAddToProjectResult */
  /** @typedef {{ persisted: boolean, verified_count?: number, updated?: number, error?: string }} BulkVerifyResult */
  /** @typedef {{ image: Array<BrowseTagOption>, stitching: Array<BrowseTagOption>, unclassified: Array<BrowseTagOption> }} TagOptionBuckets */

  let {
    navigateTo,
    browseNeedsRefresh = $bindable(false),
    detailBrowseIds = $bindable([]),
    detailBrowseIndex = $bindable(-1),
    detailDesignId,
  } = $props();

  // Browse state
  /** @type {BrowseDesignCard[]} */
  let browseItems = $state([]);
  let browseLoading = $state(false);
  let browseHasLoaded = $state(false);
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
    sourceFilters: /** @type {string[]} */ ([]),
    rating: "",
    stitched: "",
    unverifiedOnly: false,
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
      browseFilters.sourceFilters.length === 0 &&
      browseFilters.rating === "" &&
      browseFilters.stitched === "" &&
      !browseFilters.unverifiedOnly &&
      browseFilters.searchFilename &&
      browseFilters.searchTags &&
      browseFilters.searchFolder &&
      browseFilters.sortBy === "name" &&
      browseFilters.sortDir === "asc"
  );

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
    // change must re-query it. The live `q` input is deferred to form submit
    // (except when it is cleared, which resets immediately).
    if (key === "q" && value) {
      return;
    }
    loadBrowseItems(true);
  }

  function clearBrowseFilters() {
    browseFilters = defaultBrowseFilters();
    browseCurrentPage = 1;
    loadBrowseItems(true);
  }

  function applyBrowseFilters() {
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

  async function loadBrowseItems(force = false) {
    if (browseLoading && !force) return;

    browseLoading = true;
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
          min_rating: browseFilters.rating ? Number(browseFilters.rating) : null,
          stitched_status: stitchedStatus,
        },
      };
      const result = await getBrowseDesigns(payload);
      const rawItems = getResponseItems(result);
      const normalizedItems = rawItems.map(normalizeCardItem).filter((item) => item !== null);
      browseItems = /** @type {BrowseDesignCard[]} */ (normalizedItems);
      browseTotal = Math.max(0, Number(result?.total ?? 0));
      browseTotalPages = Math.max(1, Number(result?.total_pages ?? 1));
      browseCurrentPage = Math.max(1, Number(result?.page ?? browseCurrentPage));
      browseHasLoaded = true;
    } catch {
      browseHasLoaded = true;
      browseItems = [];
      browseTotal = 0;
      browseTotalPages = 1;
    } finally {
      browseLoading = false;
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
    const addIds = clearAll ? [] : browseBulkTagAddIds;

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
          `${result.updated_count ?? result.updated} design(s) tag-updated in Rust database.`,
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

    const ids = browseItems
      .map((browseItem) => Number(browseItem?.id))
      .filter((id) => Number.isFinite(id) && id > 0);

    if (ids.length > 0) {
      detailBrowseIds = ids;
      detailBrowseIndex = ids.indexOf(designId);
    } else {
      detailBrowseIds = [];
      detailBrowseIndex = -1;
    }

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
    const id = detailDesignId;
    if (id !== null && id !== undefined) {
      untrack(() => {
        openDesignDetail({ id });
      });
    } else {
      untrack(() => {
        detailBrowseIds = [];
        detailBrowseIndex = -1;
      });
    }
  });

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
</script>

<svelte:window onresize={refreshBrowseGridColumns} />

<section class="browse-section space-y-4">
  <h1 class="ui-page-title browse-title text-2xl font-bold text-gray-800">Browse Designs</h1>
  <br />
  <form
    class="browse-search-shell space-y-3 no-print bg-white rounded shadow p-4 border"
    onsubmit={(event) => {
      event.preventDefault();
      applyBrowseFilters();
    }}
  >
    <div class="ui-section-shell browse-general-search space-y-1.5">
      <label
        class="ui-section-label browse-general-search-label block text-xs font-semibold text-gray-600 uppercase"
        for="browse-q">General search</label
      >
      <p></p>
      <div class="browse-general-search-row flex items-center gap-2">
        <input
          id="browse-q"
          class="ui-text-input ui-control-text-inset browse-general-input text-sm flex-1 min-w-[20rem] font-mono border rounded px-3 py-2"
          placeholder="e.g. rose &quot;cross stitch&quot; -applique or *.hus"
          value={browseFilters.q}
          oninput={(event) => updateBrowseFilter("q", event.currentTarget.value)}
        />
        <label
          class="ui-field-label browse-unverified-label flex items-center gap-1.5 cursor-pointer select-none text-sm text-gray-700 whitespace-nowrap"
        >
          <input
            type="checkbox"
            class="ui-checkbox browse-unverified-checkbox accent-indigo-600 rounded"
            checked={browseFilters.unverifiedOnly}
            onchange={(event) => updateBrowseFilter("unverifiedOnly", event.currentTarget.checked)}
          />
          Unverified only
        </label>
      </div>
      <div
        class="browse-search-in-row flex flex-wrap items-center gap-4 text-xs text-gray-700 my-1.5 py-1.5 px-3 bg-gray-50 rounded border border-gray-200"
      >
        <span class="font-semibold text-gray-600 uppercase text-[11px] tracking-wide"
          >Search in:</span
        >
        <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
          <input
            id="search-filename-checkbox"
            type="checkbox"
            class="ui-checkbox accent-indigo-600 rounded cursor-pointer"
            checked={browseFilters.searchFilename}
            onchange={(event) => updateBrowseFilter("searchFilename", event.currentTarget.checked)}
          />
          <span>File name</span>
        </label>
        <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
          <input
            id="search-folder-checkbox"
            type="checkbox"
            class="ui-checkbox accent-indigo-600 rounded cursor-pointer"
            checked={browseFilters.searchFolder}
            onchange={(event) => updateBrowseFilter("searchFolder", event.currentTarget.checked)}
          />
          <span>Folder name</span>
        </label>
        <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
          <input
            id="search-tags-checkbox"
            type="checkbox"
            class="ui-checkbox accent-indigo-600 rounded cursor-pointer"
            checked={browseFilters.searchTags}
            onchange={(event) => updateBrowseFilter("searchTags", event.currentTarget.checked)}
          />
          <span>Tags</span>
        </label>
      </div>
      <p class="ui-help-note browse-general-help text-xs text-gray-500 mt-0.5">
        Supports Google-like syntax: "exact phrase" · -exclude · word1 OR word2 · *.hus ·
        <a href="#/help?section=search" class="text-indigo-600 hover:underline">Search help</a>
      </p>
    </div>

    <details
      class="ui-section-shell browse-additional-filters overflow-visible relative"
      open={browseAdditionalFiltersOpen}
    >
      <summary
        class="ui-section-label browse-additional-summary cursor-pointer text-xs font-semibold text-gray-600 uppercase select-none list-none flex items-center gap-1"
        onclick={(event) => {
          event.preventDefault();
          toggleAdditionalFilters();
        }}
      >
        <span>{browseAdditionalFiltersOpen ? "▼" : "▶"}</span>
        <span>Additional Filters</span>
      </summary>
      <div class="grid sm:grid-cols-2 md:grid-cols-4 gap-4 pt-3 border-t mt-2 px-4">
        <!-- Designers Filter -->
        <div class="space-y-1">
          <span class="block text-xs font-semibold text-gray-700">Designer</span>
          <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
            {#each browseDesignerFilterOptions as opt}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  checked={browseFilters.designerFilters.includes(opt)}
                  onchange={() => toggleBrowseFilter("designerFilters", opt)}
                  class="accent-indigo-600 rounded"
                />
                <span>{opt}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Image Tags Filter -->
        <div class="space-y-1">
          <span class="block text-xs font-semibold text-gray-700">Image tags</span>
          <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
            {#each browseImageTagOptions as opt}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  checked={browseFilters.imageTagFilters.includes(opt.description)}
                  onchange={() => toggleBrowseFilter("imageTagFilters", opt.description)}
                  class="accent-indigo-600 rounded"
                />
                <span>{opt.description}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Stitching Tags Filter -->
        <div class="space-y-1">
          <span class="block text-xs font-semibold text-gray-700">Stitching tags</span>
          <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
            {#each browseStitchingTagOptions as opt}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  checked={browseFilters.stitchingTagFilters.includes(opt.description)}
                  onchange={() => toggleBrowseFilter("stitchingTagFilters", opt.description)}
                  class="accent-indigo-600 rounded"
                />
                <span>{opt.description}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Sources Filter -->
        <div class="space-y-1">
          <span class="block text-xs font-semibold text-gray-700">Source</span>
          <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
            {#each browseSourceFilterOptions as opt}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  checked={browseFilters.sourceFilters.includes(opt)}
                  onchange={() => toggleBrowseFilter("sourceFilters", opt)}
                  class="accent-indigo-600 rounded"
                />
                <span>{opt}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Other Properties -->
        <div class="space-y-2.5 text-xs">
          <label class="block">
            <span class="block font-semibold text-gray-700 mb-1">Hoop size</span>
            <select
              class="border rounded px-2.5 py-1.5 w-full bg-white text-xs"
              value={browseFilters.hoop}
              onchange={(e) => updateBrowseFilter("hoop", e.currentTarget.value)}
            >
              <option value="">Any hoop</option>
              {#each browseHoopFilterOptions as opt}
                <option value={opt}>{opt}</option>
              {/each}
              <option value={HOOP_UNKNOWN_FILTER}>Hoop unknown</option>
            </select>
          </label>

          <div class="grid grid-cols-2 gap-2">
            <label class="block">
              <span class="block font-semibold text-gray-700 mb-1">Minimum rating</span>
              <select
                class="border rounded px-2.5 py-1.5 w-full bg-white text-xs"
                value={browseFilters.rating}
                onchange={(e) => updateBrowseFilter("rating", e.currentTarget.value)}
              >
                <option value="">Any</option>
                {#each [1, 2, 3, 4, 5] as score}
                  <option value={String(score)}>{score}★</option>
                {/each}
              </select>
            </label>
            <label class="block">
              <span class="block font-semibold text-gray-700 mb-1">Stitched</span>
              <select
                class="border rounded px-2.5 py-1.5 w-full bg-white text-xs"
                value={browseFilters.stitched}
                onchange={(e) => updateBrowseFilter("stitched", e.currentTarget.value)}
              >
                <option value="">Any</option>
                <option value="yes">Stitched</option>
                <option value="no">Not Stitched</option>
              </select>
            </label>
          </div>
        </div>
      </div>
    </details>

    <!-- Sorting and Columns -->
    <div
      class="flex flex-wrap items-center justify-between gap-3 pt-2 pb-4 text-xs border-t text-gray-600 px-4"
    >
      <div class="flex flex-wrap items-center gap-3">
        <label class="flex items-center gap-1.5 font-medium">
          Sort by:
          <select
            class="border rounded px-2 py-1 bg-white text-xs"
            value={browseFilters.sortBy}
            onchange={(e) => updateBrowseFilter("sortBy", e.currentTarget.value)}
          >
            <option value="name">Name</option>
            <option value="rating">Rating</option>
            <option value="stitched">Stitched</option>
            <option value="folder">Folder</option>
            <option value="date_added">Date Added</option>
          </select>
        </label>
        <label class="flex items-center gap-1.5 font-medium">
          Direction:
          <select
            class="border rounded px-2 py-1 bg-white text-xs"
            value={browseFilters.sortDir}
            onchange={(e) => updateBrowseFilter("sortDir", e.currentTarget.value)}
          >
            <option value="asc">Ascending</option>
            <option value="desc">Descending</option>
          </select>
        </label>
        <button
          type="button"
          class="text-indigo-600 hover:underline disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={clearBrowseFilters}
          disabled={browseFiltersAreDefault}>Reset filters</button
        >
      </div>
    </div>
  </form>

  <SelectionHeader
    {totalFilteredCount}
    {selectedCountOnPage}
    {totalCountOnPage}
    {isAllSelectedOnPage}
    onToggleSelectAllPage={toggleSelectAllBrowseOnPage}
    busyActive={busyActive || browseLoading}
  />

  <!-- Browse Results Grid -->
  <div bind:this={browseGridContainer} class="browse-grid-rows flex flex-col gap-5">
    {#if browseLoading && browseItems.length === 0}
      <p class="text-center py-12 text-gray-500 font-medium">Loading designs...</p>
    {:else if browseItems.length === 0}
      <p class="text-center py-12 text-gray-500 font-medium">No designs match your filters.</p>
    {:else}
      {#each browsePageRows as rowItems, rowIndex (rowIndex)}
        <div
          class="browse-grid-row grid gap-4"
          style={`grid-template-columns: 2rem repeat(${browseGridColumns}, minmax(0, 1fr));`}
        >
          <!-- Row selector checkbox -->
          <label
            class="browse-row-selector flex items-center justify-center bg-indigo-50 rounded cursor-pointer select-none"
            title={`Select row ${rowIndex + 1}`}
          >
            <span class="sr-only">Select row {rowIndex + 1}</span>
            <input
              type="checkbox"
              class="browse-row-checkbox rounded accent-indigo-500"
              checked={isBrowseRowFullySelected(rowItems)}
              onchange={() => toggleBrowseRowSelection(rowItems)}
            />
          </label>

          {#each rowItems as item (item.id)}
            <article
              class="browse-card border rounded-lg bg-white overflow-hidden shadow-sm flex flex-col hover:shadow transition relative"
              data-id={item.id}
            >
              <!-- Selection checkbox -->
              <label class="absolute top-2.5 left-2.5 z-10 cursor-pointer select-none">
                <input
                  type="checkbox"
                  class="browse-design-checkbox rounded accent-indigo-650"
                  checked={browseSelectedIds.has(item.id)}
                  oninput={() =>
                    toggleBrowseCardSelection(item.id, !browseSelectedIds.has(item.id))}
                  disabled={browseSelectionLocked ||
                    (browseSelectedIds.size >= BROWSE_BULK_DELETE_MAX &&
                      !browseSelectedIds.has(item.id))}
                />
              </label>

              <button
                class="browse-card-link w-full text-left flex flex-col flex-1"
                onclick={(event) => handleBrowseCardOpenDetail(event, item)}
              >
                {#if browsePreviewById[item.id]}
                  <div
                    class="browse-card-image-frame bg-gray-50 p-2 flex items-center justify-center h-48 border-b"
                  >
                    <img
                      src={browsePreviewById[item.id]}
                      alt={item.filename}
                      class="browse-card-image max-h-full object-contain"
                      loading="lazy"
                    />
                  </div>
                {:else}
                  <div
                    class="browse-card-image-frame bg-gray-50 p-2 flex items-center justify-center h-48 border-b text-xs text-gray-400 font-medium italic"
                  >
                    {browsePreviewsLoading ? "Loading image..." : "No preview image"}
                  </div>
                {/if}
                <div class="browse-card-meta p-4 flex-1 flex flex-col justify-between">
                  <div>
                    <div class="browse-card-title-row flex items-start justify-between gap-1.5">
                      <p
                        class="browse-card-title text-sm font-semibold text-gray-800 truncate flex-1"
                        title={item.filename}
                      >
                        {item.filename}
                      </p>
                      {#if item.imageTagsVerified && item.stitchingTagsVerified}
                        <span
                          class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-green-500"
                          title="Verified"
                          aria-label="Verified"
                        >
                          ✓
                        </span>
                      {:else if item.imageTagsVerified}
                        <span
                          class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-amber-400"
                          title="Image Verified, Stitching Unverified"
                          aria-label="Image Verified, Stitching Unverified"
                        >
                          ◐
                        </span>
                      {:else if item.stitchingTagsVerified}
                        <span
                          class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-amber-400"
                          title="Stitching Verified, Image Unverified"
                          aria-label="Stitching Verified, Image Unverified"
                        >
                          ◑
                        </span>
                      {/if}
                    </div>
                    <p class="browse-card-hoop text-xs font-semibold text-indigo-600 mt-1">
                      {item.hoop || "Hoop unknown"}
                    </p>
                    {#if item.projects.length > 0}
                      <p
                        class="browse-card-projects text-[11px] text-gray-500 mt-1 truncate"
                        title={item.projects.join(", ")}
                      >
                        {item.projects.join(", ")}
                      </p>
                    {/if}
                  </div>
                  <div class="pt-2">
                    {#if item.tags.length > 0}
                      <p
                        class="browse-card-tags text-[11px] text-gray-500 truncate"
                        title={item.tags.join(", ")}
                      >
                        {item.tags.join(", ")}
                      </p>
                    {:else}
                      <p class="browse-card-tags text-[11px] text-gray-300 italic">No tags</p>
                    {/if}
                    <p
                      class="browse-card-rating text-xs mt-1"
                      aria-label={item.rating != null && item.rating > 0
                        ? `Rating ${item.rating} out of 5`
                        : "Not rated"}
                    >
                      {#if item.rating != null && item.rating > 0}
                        <span class="text-amber-600">★</span>
                        <span class="text-gray-700 font-bold ml-0.5">{item.rating}</span>
                      {:else}
                        <span class="text-gray-400">☆ —</span>
                      {/if}
                    </p>
                  </div>
                </div>
              </button>

              <details
                class="browse-card-project-details px-4 py-2 bg-gray-50 border-t no-print"
                ontoggle={(event) =>
                  handleBrowseCardProjectDetailsToggle(item, event.currentTarget)}
              >
                <summary
                  class="browse-card-project-summary text-xs font-semibold text-gray-500 cursor-pointer hover:text-indigo-600 select-none"
                >
                  + Add to project
                </summary>
                <div
                  class="ui-checkbox-list-shell mt-1.5 max-h-36 overflow-auto px-2 py-1.5 border rounded bg-white space-y-1"
                >
                  {#each browseProjects as project}
                    <label
                      class="ui-field-label flex items-center gap-1.5 text-xs text-gray-700 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        class="ui-checkbox accent-indigo-650 rounded"
                        checked={isBrowseCardProjectChecked(item, project.id)}
                        onchange={(event) =>
                          updateBrowseCardProjectPending(
                            item.id,
                            project.id,
                            event.currentTarget.checked
                          )}
                      />
                      <span>{project.name}</span>
                    </label>
                  {:else}
                    <p class="text-[11px] text-gray-500 italic px-1 py-0.5">
                      No projects found. Create one first.
                    </p>
                  {/each}
                </div>
              </details>
            </article>
          {/each}
        </div>
      {/each}
    {/if}
  </div>

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
{#if showBrowseBulkBar}
  <div
    bind:this={browseBulkBarNode}
    use:portalToBody
    class="browse-bulk-bar ui-section-shell no-print fixed bottom-0 left-0 right-0 bg-white border-t p-4 shadow-lg flex flex-wrap items-center justify-between gap-4 z-40"
  >
    <div class="flex items-center gap-3 text-sm text-gray-700">
      <span class="font-semibold"
        >{browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected</span
      >
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs"
        onclick={openBulkTagModal}
      >
        Choose tags
      </button>

      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs"
        onclick={runBulkVerify}
      >
        Verify tags
      </button>

      <details class="relative" open={browseBulkProjectDropdownOpen} style="display:inline-block;">
        <summary
          class="menu-button-secondary ui-action-button text-xs cursor-pointer select-none list-none"
          onclick={(event) => {
            event.preventDefault();
            if (browseBulkProjectDropdownOpen) {
              closeBulkProjectModal();
            } else {
              openBulkProjectModal();
            }
          }}
        >
          Add to project…
        </summary>
        <div
          class="absolute bottom-full mb-2 right-0 bg-white border rounded shadow-lg p-3 max-h-48 overflow-auto min-w-[12rem] space-y-1.5 z-50"
        >
          {#if browseProjects.length === 0}
            <p class="text-xs text-gray-500 italic">No projects found. Create one first.</p>
          {:else}
            {#each browseProjects as project}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  class="ui-checkbox accent-indigo-650 rounded"
                  checked={browseBulkProjectSelection.includes(Number(project.id))}
                  onchange={(event) =>
                    toggleBrowseBulkProjectSelection(project.id, event.currentTarget.checked)}
                />
                <span>{project.name}</span>
              </label>
            {/each}
          {/if}
          <div class="pt-2 border-t flex justify-end">
            <button
              type="button"
              class="menu-button-primary text-[10px] py-1 px-2.5"
              onclick={addSelectedToProject}
              disabled={browseBulkProjectSelection.length === 0}
            >
              Apply
            </button>
          </div>
        </div>
      </details>

      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs text-red-500 border-red-200"
        onclick={openBrowseDeleteConfirm}
      >
        Delete selected
      </button>

      <button
        type="button"
        class="menu-button-primary ui-action-button ui-action-button-primary text-xs"
        onclick={clearBrowseSelection}
      >
        Clear selection
      </button>
    </div>
  </div>
{/if}

<!-- Browse Bulk Tag Modal -->
{#if browseBulkModalOpen}
  {@const groupedTagOptions = browseGroupedTagOptions}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="bulk-tag-title"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Close tag chooser"
      onclick={closeBulkTagModal}
    ></button>
    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
    >
      <div
        class="tag-chooser-header"
        style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;"
      >
        <h2 id="bulk-tag-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Choose tags for selected designs
        </h2>
      </div>
      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p class="text-xs text-gray-500 font-semibold" style="margin:0 0 0.75rem 0;">
          {browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.
        </p>

        <div class="tag-chooser-section" style="margin-bottom:0.75rem;">
          <label class="tag-chooser-option" style="font-weight:600;">
            <input
              type="checkbox"
              checked={browseBulkClearAll}
              disabled={browseBulkTagAddIds.length > 0 ||
                browseBulkTagRemoveIds.length > 0 ||
                browseBulkTagIndeterminateIds.length > 0}
              onchange={(event) => {
                browseBulkClearAll = event.currentTarget.checked;
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
                    disabled={browseBulkClearAll}
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
                    disabled={browseBulkClearAll}
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
                    disabled={browseBulkClearAll}
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
      <div
        class="tag-chooser-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;"
      >
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
