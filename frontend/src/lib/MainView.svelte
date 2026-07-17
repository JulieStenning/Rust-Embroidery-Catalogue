<script>
  import { onDestroy, tick, untrack } from "svelte";
  import {
    getBrowseDesigns,
    getBrowseDesignPreviews,
    getBrowseProjects,
    getBrowseTags,
    deleteDesign,
    addDesignToProject,
    removeDesignFromProject,
    listDesigners,
    createDesigner,
    updateDesigner,
    deleteDesigner as removeDesigner,
    listSources,
    createSource,
    updateSource,
    deleteSource as removeSource,
    listTags,
    createTag,
    setTagGroup as updateTagGroup,
    deleteTag as removeTag,
    listHoops,
    createHoop,
    updateHoop,
    deleteHoop as removeHoop,
    bulkVerifyDesigns,
    bulkAddDesignsToProject,
    bulkSetTagsForDesigns
  } from "./api/commandAdapter.js";
  import HelpView from "./views/HelpView.svelte";
  import AboutView from "./views/AboutView.svelte";
  import AboutDocumentView from "./views/AboutDocumentView.svelte";
  import SettingsView from "./views/SettingsView.svelte";
  import BackupView from "./views/BackupView.svelte";
  import TaggingActionsView from "./views/TaggingActionsView.svelte";
  import OrphansView from "./views/OrphansView.svelte";
  import ProjectsView from "./views/ProjectsView.svelte";
  import DesignDetailView from "./views/DesignDetailView.svelte";
  import DesignPrintView from "./views/DesignPrintView.svelte";
  import ImportView from "./views/ImportView.svelte";

  import Pagination from "./components/Pagination.svelte";
  import { splitTagsByGroup } from "./utils/tagHelpers.js";

  const ORDERED_ROUTE_HINTS = [
    "#/designs",
    "#/import",
    "#/projects",
    "#/help",
    "#/admin/designers",
    "#/admin/tags",
    "#/admin/sources",
    "#/admin/hoops",
    "#/admin/settings",
    "#/admin/maintenance/backup",
    "#/admin/tagging-actions",
    "#/admin/orphans",
    "#/about",
  ];

  const ROUTE_UI_KIND = {
    "#/designs": "browse",
    "#/import": "import",
    "#/projects": "projects-list",
    "#/help": "help",
    "#/admin/designers": "admin-list",
    "#/admin/tags": "admin-list",
    "#/admin/sources": "admin-list",
    "#/admin/hoops": "admin-list",
    "#/admin/settings": "settings",
    "#/admin/maintenance/backup": "backup",
    "#/admin/tagging-actions": "tagging-actions",
    "#/admin/orphans": "orphans",
    "#/about": "about",
  };

  const HELP_SECTION_IDS = new Set([
    "search",
    "importing",
    "ai-tagging",
    "tagging-actions",
    "projects",
    "maintenance",
    "troubleshooting"
  ]);

  /** @param {string} route */
  function parseDesignDetailId(route) {
    const match = route.match(/^#\/designs\/(\d+)$/);
    return match ? Number(match[1]) : null;
  }

  /** @param {string} route */
  function parseDesignPrintId(route) {
    const match = route.match(/^#\/designs\/(\d+)\/print$/);
    return match ? Number(match[1]) : null;
  }

  /** @param {string} route */
  function parseProjectDetailId(route) {
    const match = route.match(/^#\/projects\/(\d+)$/);
    return match ? Number(match[1]) : null;
  }

  /** @param {string} route */
  function parseProjectPrintId(route) {
    const match = route.match(/^#\/projects\/(\d+)\/print$/);
    return match ? Number(match[1]) : null;
  }

  /** @param {string} route */
  function parseAboutDocumentSlug(route) {
    if (route === "#/about/licence") return "licence";
    const match = route.match(/^#\/about\/document\/([a-z0-9-]+)$/);
    return match ? String(match[1]).toLowerCase() : null;
  }

  /** @param {string} route */
  function parseImportWizardStep(route) {
    if (route === "#/import") return 1;
    const match = route.match(/^#\/import\/step([123])$/);
    return match ? Number(match[1]) : null;
  }

  /** @param {string} route */
  function resolveCurrentUiKind(route) {
    if (parseProjectPrintId(route) !== null) return "project-print";
    if (route === "#/projects/new") return "project-new";
    if (parseProjectDetailId(route) !== null) return "project-detail";
    if (parseDesignPrintId(route) !== null) return "design-print";
    if (parseDesignDetailId(route) !== null) return "design-detail";
    if (parseAboutDocumentSlug(route) !== null) return "about-document";
    if (parseImportWizardStep(route) !== null) return "import";
    return ROUTE_UI_KIND[/** @type {keyof typeof ROUTE_UI_KIND} */ (route)] || null;
  }

  let currentRoute = $state("");
  let previousRoute = $state("");
  let currentUiKind = $derived(resolveCurrentUiKind(currentRoute));
  let detailDesignId = $derived(parseDesignDetailId(currentRoute));
  let printDesignId = $derived(parseDesignPrintId(currentRoute));
  let projectDetailId = $derived(parseProjectDetailId(currentRoute));
  let projectPrintId = $derived(parseProjectPrintId(currentRoute));
  let aboutDocumentSlug = $derived(parseAboutDocumentSlug(currentRoute));

  // Browse state
  /** @type {any[]} */
  let browseItems = $state([]);
  let browseSource = $state("mock");
  let browseLoading = $state(false);
  let browseHasLoaded = $state(false);
  let browseError = $state("");
  /** @type {any[]} */
  let browseProjects = $state([]);
  let browseProjectsSource = $state("mock");
  let browseProjectsLoaded = $state(false);
  /** @type {any[]} */
  let browseTagOptions = $state([]);
  let browseTagsSource = $state("mock");
  let browsePreviewsSource = $state("mock");
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
  let browseAdditionalFiltersOpen = $state(false);
  /** @type {number[]} */
  let browseSelectedIds = $state([]);
  /** @type {HTMLDivElement | null} */
  let browseBulkBarNode = $state(null);
  /** @type {HTMLDivElement | null} */
  let browseBulkModalOverlayNode = $state(null);
  /** @type {HTMLDivElement | null} */
  let browseBulkModalDialogNode = $state(null);
  let browseBulkModalOpen = $state(false);
  let browseBulkModalMode = $state("browse");
  /** @type {Array<number | string>} */
  let browseBulkTagSelection = $state([]);
  /** @type {number[]} */
  let browseBulkProjectSelection = $state([]);
  let browseBulkProjectDropdownOpen = $state(false);
  /** @type {Record<number, Record<number, boolean>>} */
  let browseCardProjectPendingById = $state({});
  let browseDeleteConfirmOpen = $state(false);
  let browseDeleteSelectedBusy = $state(false);
  let browseActionNotice = $state("");
  /** @type {HTMLDivElement | null} */
  let browseGridContainer = $state(null);
  let browseGridColumns = $state(5);

  const BROWSE_PAGE_ROWS = 10;
  const BROWSE_BREAKPOINT_SM = 640;
  const BROWSE_BREAKPOINT_MD = 768;
  const BROWSE_BREAKPOINT_LG = 1024;
  const BROWSE_ROW_SELECTOR_WIDTH = 28;
  const BROWSE_TAG_UNTAGGED = "__untagged__";

  const defaultBrowseFilters = () => ({
    q: "",
    allWords: "",
    exactPhrase: "",
    anyWords: "",
    noneWords: "",
    filename: "",
    designerFilters: /** @type {string[]} */ ([]),
    tagFilters: /** @type {string[]} */ ([]),
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

  let browseNeedsRefresh = $state(false);

  /** @param {number} importedCount */
  function handleImportCompleted(importedCount) {
    if (importedCount >= 1) {
      browseNeedsRefresh = true;
    }
  }

  // Detail navigation browse context
  /** @type {number[]} */
  let detailBrowseIds = $state([]);
  let detailBrowseIndex = $state(-1);

  // Admin CRUD state
  /** @type {any[]} */
  let designers = $state([]);
  let newDesignerName = $state("");
  /** @type {number | null} */
  let editingDesignerId = $state(null);
  let editingDesignerName = $state("");
  /** @type {number | null} */
  let pendingDeleteDesignerId = $state(null);

  /** @type {any[]} */
  let sources = $state([]);
  let newSourceName = $state("");
  /** @type {number | null} */
  let editingSourceId = $state(null);
  let editingSourceName = $state("");
  /** @type {number | null} */
  let pendingDeleteSourceId = $state(null);

  /** @type {any[]} */
  let hoops = $state([]);
  let newHoopName = $state("");
  let newHoopWidth = $state(0);
  let newHoopHeight = $state(0);
  /** @type {number | null} */
  let editingHoopId = $state(null);
  let editingHoopName = $state("");
  let editingHoopWidth = $state(0);
  let editingHoopHeight = $state(0);
  /** @type {number | null} */
  let pendingDeleteHoopId = $state(null);

  /** @type {any[]} */
  let imageTags = $state([]);
  /** @type {any[]} */
  let stitchingTags = $state([]);
  /** @type {any[]} */
  let unclassifiedTags = $state([]);
  let newTagDescription = $state("");
  let newTagGroup = $state("image");
  let adminImageTagsOpen = $state(true);
  let adminStitchingTagsOpen = $state(true);
  let adminTagsPanelStateLoaded = $state(false);

  let adminNotice = $state("");
  let adminNoticeType = $state("info");
  let adminLoading = $state(false);

  let canAddDesigner = $derived(newDesignerName.trim().length > 0);
  let canClearDesignerForm = $derived(newDesignerName.length > 0);
  let canAddSource = $derived(newSourceName.trim().length > 0);
  let canClearSourceForm = $derived(newSourceName.length > 0);
  let canAddHoop = $derived(newHoopName.trim().length > 0 && newHoopWidth > 0 && newHoopHeight > 0);
  let canClearHoopForm = $derived(newHoopName.length > 0 || newHoopWidth > 0 || newHoopHeight > 0);

  let adminIsDesignersRoute = $derived(currentRoute === "#/admin/designers");
  let adminIsTagsRoute = $derived(currentRoute === "#/admin/tags");
  let adminIsSourcesRoute = $derived(currentRoute === "#/admin/sources");
  let adminIsHoopsRoute = $derived(currentRoute === "#/admin/hoops");

  /** @param {string} message @param {string} [type] */
  function setAdminNotice(message, type = "info") {
    adminNotice = message;
    adminNoticeType = type;
  }

  /** @param {string} hashString */
  function normalizeHash(hashString) {
    const raw = String(hashString || "").trim();
    if (!raw.startsWith("#")) {
      return "#/designs";
    }

    const questionIndex = raw.indexOf("?");
    const path = questionIndex !== -1 ? raw.slice(0, questionIndex) : raw;
    const pathLower = path.toLowerCase();

    for (const hint of ORDERED_ROUTE_HINTS) {
      if (pathLower === hint.toLowerCase()) {
        return hint;
      }
    }

    if (parseDesignDetailId(path) !== null) {
      return path;
    }
    if (parseDesignPrintId(path) !== null) {
      return path;
    }
    if (parseProjectDetailId(path) !== null) {
      return path;
    }
    if (parseProjectPrintId(path) !== null) {
      return path;
    }
    if (parseAboutDocumentSlug(path) !== null) {
      return path;
    }
    if (parseImportWizardStep(path) !== null) {
      return path;
    }

    return "#/designs";
  }

  function syncRouteFromHash() {
    const newHash = window.location.hash || "#/designs";
    const nextRoute = normalizeHash(newHash);
    if (nextRoute !== currentRoute) {
      previousRoute = currentRoute;
      currentRoute = nextRoute;
    }

    const questionIndex = newHash.indexOf("?");
    if (nextRoute === "#/help" && questionIndex !== -1) {
      const queryParams = new URLSearchParams(newHash.slice(questionIndex));
      const section = queryParams.get("section");
      if (section && HELP_SECTION_IDS.has(section)) {
        tick().then(() => {
          setTimeout(() => {
            const el = document.getElementById(section);
            if (el) {
              el.scrollIntoView({ behavior: "smooth", block: "start" });
            }
          }, 150);
        });
      }
    }
  }

  /** @param {string} target */
  function navigateTo(target) {
    window.location.hash = target;
  }

  /** @param {string} target */
  function linkClass(target) {
    const isActive = currentRoute === target || (target === "#/import" && currentUiKind === "import");
    return `menu-link ${isActive ? "menu-link-active" : ""}`;
  }

  /** @param {string} target */
  function adminLinkClass(target) {
    const isActive = currentRoute === target;
    return `menu-link menu-link-admin ${isActive ? "menu-link-active" : ""}`;
  }

  // Browse Search Parser
  /** @param {string} queryString */
  function parseQueryWithOr(queryString) {
    const query = String(queryString || "").trim();
    if (!query) {
      return [];
    }

    const orParts = query.split(/\bOR\b/);
    const groups = [];

    for (const part of orParts) {
      const terms = [];
      const regex = /"([^"]+)"|(-?\S+)/g;
      let match;

      while ((match = regex.exec(part)) !== null) {
        if (match[1]) {
          terms.push({ text: match[1], phrase: true, exclude: false });
        } else if (match[2]) {
          const rawTerm = match[2];
          const exclude = rawTerm.startsWith("-");
          const text = exclude ? rawTerm.slice(1) : rawTerm;
          if (text) {
            terms.push({ text, phrase: false, exclude });
          }
        }
      }

      if (terms.length > 0) {
        groups.push(terms);
      }
    }

    return groups;
  }

  /** @param {string} term */
  function isWildcardPattern(term) {
    return term.includes("*") || term.includes("?");
  }

  /** @param {string} fieldValue @param {string} term */
  function matchesTerm(fieldValue, term) {
    const val = String(fieldValue || "").toLowerCase();
    const pat = String(term || "").toLowerCase();

    if (isWildcardPattern(pat)) {
      const escaped = pat.replace(/[-/\\^$*+?.()|[\]{}]/g, (char) => {
        if (char === "*") return ".*";
        if (char === "?") return ".";
        return "\\" + char;
      });
      const regex = new RegExp("^" + escaped + "$");
      return regex.test(val);
    }

    return val.includes(pat);
  }

  /** @param {string} filepath */
  function extractFolder(filepath) {
    const path = String(filepath || "").trim().replace(/\\/g, "/");
    if (!path) return "";
    const segments = path.split("/").filter(Boolean);
    if (segments.length <= 1) return "";
    return segments[segments.length - 2];
  }

  /** @param {any} item */
  function normalizeCardItem(item) {
    if (!item || typeof item !== "object") {
      return null;
    }
    const tags = Array.isArray(item.tags) ? item.tags : [];
    const imageTags = tags.filter(/** @param {any} t */ (t) => t.tag_group === "image").map(/** @param {any} t */ (t) => t.description);
    const stitchingTags = tags.filter(/** @param {any} t */ (t) => t.tag_group === "stitching").map(/** @param {any} t */ (t) => t.description);
    const flatTags = tags.map(/** @param {any} t */ (t) => t.description);

    const folder = item.folder || extractFolder(item.filepath);
    const id = Number(item.id);
    const dateAdded = item.date_added || (id ? new Date(id * 1000).toISOString() : "");

    return {
      id,
      filename: String(item.filename || ""),
      filepath: String(item.filepath || ""),
      designer: String(item.designer || ""),
      source: String(item.source || ""),
      hoop: String(item.hoop || ""),
      rating: item.rating == null ? null : Number(item.rating),
      is_stitched: Boolean(item.is_stitched),
      tagsChecked: Boolean(item.tags_checked),
      projects: Array.isArray(item.projects) ? item.projects.map(/** @param {any} p */ (p) => String(p.name || "")) : [],
      imageTags,
      stitchingTags,
      tags: flatTags,
      folder,
      dateAdded,
    };
  }

  /** @param {any} left @param {any} right @param {string} sortBy @param {string} sortDir */
  function compareBrowseItems(left, right, sortBy, sortDir) {
    const directionMultiplier = sortDir === "desc" ? -1 : 1;

    if (sortBy === "rating") {
      const scoreLeft = left.rating ?? -1;
      const scoreRight = right.rating ?? -1;
      if (scoreLeft !== scoreRight) {
        return (scoreLeft - scoreRight) * directionMultiplier;
      }
    }

    if (sortBy === "stitched") {
      const stitchedLeft = left.is_stitched ? 1 : 0;
      const stitchedRight = right.is_stitched ? 1 : 0;
      if (stitchedLeft !== stitchedRight) {
        return (stitchedLeft - stitchedRight) * directionMultiplier;
      }
    }

    if (sortBy === "folder") {
      const folderLeft = left.folder;
      const folderRight = right.folder;
      const comp = folderLeft.localeCompare(folderRight, undefined, { sensitivity: "base" });
      if (comp !== 0) {
        return comp * directionMultiplier;
      }
    }

    if (sortBy === "date_added") {
      const dateLeft = left.dateAdded;
      const dateRight = right.dateAdded;
      const comp = dateLeft.localeCompare(dateRight);
      if (comp !== 0) {
        return comp * directionMultiplier;
      }
    }

    // Default sorting (name/filename)
    const nameLeft = left.filename;
    const nameRight = right.filename;
    return nameLeft.localeCompare(nameRight, undefined, { sensitivity: "base" }) * directionMultiplier;
  }

  /** @param {string} key @param {any} value */
  function updateBrowseFilter(key, value) {
    browseFilters = {
      ...browseFilters,
      [key]: value,
    };
    browseCurrentPage = 1;
  }

  function clearBrowseFilters() {
    browseFilters = defaultBrowseFilters();
    browseCurrentPage = 1;
  }

  function applyBrowseFilters() {
    browseCurrentPage = 1;
  }

  async function loadBrowseItems(force = false) {
    if (browseLoading && !force) return;

    browseLoading = true;
    browseError = "";
    try {
      const result = await getBrowseDesigns();
      const rawItems = Array.isArray(result?.items) ? result.items : [];
      browseItems = rawItems.map(normalizeCardItem).filter(Boolean);
      browseSource = result?.source || "mock";
      browseHasLoaded = true;
    } catch (error) {
      browseItems = [];
      browseSource = "mock";
      browseError = `Could not load designs: ${error}`;
    } finally {
      browseLoading = false;
    }
  }

  async function loadBrowseTags() {
    try {
      const result = await getBrowseTags();
      browseTagOptions = Array.isArray(result?.items) ? result.items : [];
      browseTagsSource = result?.source || "mock";
    } catch (error) {
      browseTagOptions = [];
      browseTagsSource = "mock";
      console.info("Could not load browse tags list", error);
    }
  }

  async function loadBrowseProjects() {
    try {
      const result = await getBrowseProjects();
      browseProjects = Array.isArray(result?.items) ? result.items : [];
      browseProjectsSource = result?.source || "mock";
      browseProjectsLoaded = true;
    } catch (error) {
      browseProjects = [];
      browseProjectsSource = "mock";
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

      browseDesignerFilterOptions = Array.from(
        new Set(
          (Array.isArray(designerResult?.items) ? designerResult.items : [])
            .map((item) => String(item?.name || "").trim())
            .filter(Boolean)
        )
      ).sort((a, b) => a.localeCompare(b));

      browseSourceFilterOptions = Array.from(
        new Set(
          (Array.isArray(sourceResult?.items) ? sourceResult.items : [])
            .map((item) => String(item?.name || "").trim())
            .filter(Boolean)
        )
      ).sort((a, b) => a.localeCompare(b));

      browseHoopFilterOptions = Array.from(
        new Set(
          (Array.isArray(hoopResult?.items) ? hoopResult.items : [])
            .map((item) => String(item?.name || "").trim())
            .filter(Boolean)
        )
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
      ? Array.from(new Set(designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)))
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
      browsePreviewsSource = result.source || "mock";
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
        browsePreviewsSource = "mock";
      }
    } finally {
      if (requestId === browsePreviewRequestCounter) {
        browsePreviewsLoading = false;
      }
    }
  }

  // Derived Browse Computations
  let browseFilteredItems = $derived(
    (() => {
      let filtered = [...browseItems];

      // general search parsing
      const queryGroups = parseQueryWithOr(browseFilters.q);
      if (queryGroups.length > 0) {
        filtered = filtered.filter((item) => {
          return queryGroups.some((group) => {
            return group.every((term) => {
              const activeFields = /** @type {string[]} */ ([]);
              if (browseFilters.searchFilename) activeFields.push(item.filename);
              if (browseFilters.searchTags) activeFields.push(...item.tags);
              if (browseFilters.searchFolder) activeFields.push(item.folder);

              const termMatchesAnyField = activeFields.some((field) => matchesTerm(field, term.text));
              return term.exclude ? !termMatchesAnyField : termMatchesAnyField;
            });
          });
        });
      }

      // specific filters
      /** @type {string[]} */
      const designerFilters = Array.isArray(browseFilters.designerFilters) ? browseFilters.designerFilters : [];
      if (designerFilters.length > 0) {
        const activeDesigners = new Set(designerFilters.map((d) => String(d).toLowerCase().trim()));
        filtered = filtered.filter((item) => activeDesigners.has(String(item.designer).toLowerCase().trim()));
      }

      /** @type {string[]} */
      const tagFilters = Array.isArray(browseFilters.tagFilters) ? browseFilters.tagFilters : [];
      if (tagFilters.length > 0) {
        const activeTags = new Set(tagFilters.map((t) => String(t).toLowerCase().trim()));
        filtered = filtered.filter((item) => {
          const itemTags = Array.isArray(item.tags) ? item.tags : [];
          return itemTags.some(/** @param {any} tag */ (tag) => activeTags.has(String(tag).toLowerCase().trim()));
        });
      }

      /** @type {string[]} */
      const sourceFilters = Array.isArray(browseFilters.sourceFilters) ? browseFilters.sourceFilters : [];
      if (sourceFilters.length > 0) {
        const activeSources = new Set(sourceFilters.map((s) => String(s).toLowerCase().trim()));
        filtered = filtered.filter((item) => activeSources.has(String(item.source).toLowerCase().trim()));
      }

      const hoopVal = String(browseFilters.hoop || "").trim();
      if (hoopVal) {
        filtered = filtered.filter((item) => String(item.hoop || "").toLowerCase().trim() === hoopVal.toLowerCase());
      }

      const ratingVal = String(browseFilters.rating || "").trim();
      if (ratingVal) {
        const score = Number(ratingVal);
        filtered = filtered.filter((item) => item.rating === score);
      }

      const stitchedVal = String(browseFilters.stitched || "").trim();
      if (stitchedVal === "yes") {
        filtered = filtered.filter((item) => item.is_stitched);
      } else if (stitchedVal === "no") {
        filtered = filtered.filter((item) => !item.is_stitched);
      }

      if (browseFilters.unverifiedOnly) {
        filtered = filtered.filter((item) => !item.tagsChecked);
      }

      // Sorting
      return filtered.sort((left, right) =>
        compareBrowseItems(left, right, browseFilters.sortBy, browseFilters.sortDir)
      );
    })()
  );

  let browsePageSize = $derived(Math.max(1, (browseGridColumns || 5) * BROWSE_PAGE_ROWS));
  let browseTotalPages = $derived(Math.max(1, Math.ceil(browseFilteredItems.length / browsePageSize)));

  let browsePageItems = $derived(
    (() => {
      const startIndex = (browseCurrentPage - 1) * browsePageSize;
      return browseFilteredItems.slice(startIndex, startIndex + browsePageSize);
    })()
  );

  let browseSelectedCount = $derived(browseSelectedIds.length);
  let showBrowseBulkBar = $derived(currentUiKind === "browse" && browseSelectedCount > 0);

  /** @param {any} id @param {any} checked */
  function toggleBrowseCardSelection(id, checked) {
    const targetId = Number(id);
    if (checked) {
      browseSelectedIds = Array.from(new Set([...browseSelectedIds, targetId]));
    } else {
      browseSelectedIds = browseSelectedIds.filter((item) => item !== targetId);
    }
  }

  function checkAllBrowseOnPage() {
    const pageIds = browsePageItems.map((item) => item.id);
    browseSelectedIds = Array.from(new Set([...browseSelectedIds, ...pageIds]));
  }

  function checkNoneBrowse() {
    browseSelectedIds = [];
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

  /** @param {string} key @param {string} filterValue */
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
    if (browseSelectedIds.length === 0) return;
    browseBulkModalMode = "browse";
    browseBulkTagSelection = [];
    browseBulkModalOpen = true;
  }

  function closeBulkTagModal() {
    browseBulkModalOpen = false;
  }

  /** @param {number | string} tagId */
  function tagChooserSelectionIncludes(tagId) {
    return browseBulkTagSelection.includes(Number(tagId));
  }

  /** @param {any} tagId @param {any} checked */
  function toggleTagChooserSelection(tagId, checked) {
    const id = Number(tagId);
    if (!Number.isFinite(id)) return;
    if (checked) {
      browseBulkTagSelection = Array.from(new Set([...browseBulkTagSelection, id]));
    } else {
      browseBulkTagSelection = browseBulkTagSelection.filter((value) => value !== id);
    }
  }

  async function applyBulkTags() {
    if (browseSelectedIds.length === 0) return;

    const tagIds = [...browseBulkTagSelection];
    const clearAll = tagIds.includes(BROWSE_TAG_UNTAGGED);
    const finalTags = clearAll ? [] : tagIds;

    browseLoading = true;
    try {
      const result = /** @type {any} */ (await bulkSetTagsForDesigns(browseSelectedIds, finalTags));
      if (result?.persisted) {
        browseActionNotice = `${result.updated_count ?? result.updated} design(s) tag-updated in Rust database.`;
        closeBulkTagModal();
        await loadBrowseItems(true);
      } else {
        browseActionNotice = result?.error || "Could not bulk update tags.";
        closeBulkTagModal();
      }
    } catch (e) {
      browseActionNotice = `Bulk tagging failed: ${e}`;
      closeBulkTagModal();
    } finally {
      browseLoading = false;
    }
  }

  async function applySharedTagChooser() {
    await applyBulkTags();
  }

  function openBulkProjectModal() {
    if (browseSelectedIds.length === 0) return;
    browseBulkProjectSelection = [];
    browseBulkProjectDropdownOpen = true;
  }

  function closeBulkProjectModal() {
    browseBulkProjectDropdownOpen = false;
  }

  /** @param {any} projectId @param {any} checked */
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
    if (browseSelectedIds.length === 0 || browseBulkProjectSelection.length === 0) return;

    browseLoading = true;
    try {
      const result = /** @type {any} */ (await bulkAddDesignsToProject(browseSelectedIds, browseBulkProjectSelection));
      if (result?.persisted) {
        browseActionNotice = `${result.added_count ?? result.updated} design(s) added to project(s).`;
        closeBulkProjectModal();
        await loadBrowseItems(true);
      } else {
        browseActionNotice = result?.error || "Could not bulk add designs to projects.";
      }
    } catch (e) {
      browseActionNotice = `Bulk project add failed: ${e}`;
    } finally {
      browseLoading = false;
    }
  }

  async function runBulkVerify() {
    if (browseSelectedIds.length === 0) return;

    browseLoading = true;
    try {
      const result = /** @type {any} */ (await bulkVerifyDesigns(browseSelectedIds));
      if (result?.persisted) {
        browseActionNotice = `${result.verified_count ?? result.updated} design(s) marked verified.`;
        await loadBrowseItems(true);
      } else {
        browseActionNotice = result?.error || "Could not verify designs.";
      }
    } catch (e) {
      browseActionNotice = `Verification failed: ${e}`;
    } finally {
      browseLoading = false;
    }
  }

  function openBrowseDeleteConfirm() {
    if (browseSelectedIds.length === 0) return;
    browseDeleteConfirmOpen = true;
  }

  function closeBrowseDeleteConfirm() {
    browseDeleteConfirmOpen = false;
  }

  async function confirmDeleteSelectedBrowseItems() {
    if (browseSelectedIds.length === 0) return;

    browseDeleteSelectedBusy = true;
    try {
      const result = await bulkVerifyDesigns(browseSelectedIds); // Placeholder API or bulk delete
      // Since bulkDelete isn't standard, delete each one or call bulk delete if exists
      // The plan states "Delete selected designs from database only"
      let successCount = 0;
      for (const designId of browseSelectedIds) {
        const delRes = await deleteDesign(designId, false);
        if (delRes?.persisted) {
          successCount++;
        }
      }
      browseActionNotice = `${successCount} design(s) deleted from database.`;
      browseSelectedIds = [];
      closeBrowseDeleteConfirm();
      await loadBrowseItems(true);
    } catch (e) {
      browseActionNotice = `Delete failed: ${e}`;
    } finally {
      browseDeleteSelectedBusy = false;
    }
  }

  function clearBrowseSelection() {
    browseSelectedIds = [];
  }

  /** @param {any} item @param {any} summaryNode */
  function handleBrowseCardProjectDetailsToggle(item, summaryNode) {
    const detailsNode = summaryNode?.parentNode;
    if (detailsNode && detailsNode.hasAttribute("open") && browseProjects.length === 0) {
      loadBrowseProjects();
    }
  }

  /** @param {any} item @param {any} projectId */
  function isBrowseCardProjectChecked(item, projectId) {
    const designId = Number(item.id);
    const prjId = Number(projectId);
    const pendingVal = browseCardProjectPendingById?.[designId]?.[prjId];
    if (pendingVal !== undefined) {
      return pendingVal;
    }
    return Array.isArray(item.projects) && item.projects.includes(String(projectId));
  }

  /** @param {any} designId @param {any} projectId @param {any} checked */
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

  /** @param {any} designId */
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

  /** @param {any} rating */
  function browseStars(rating) {
    const numeric = Number(rating);
    if (!Number.isFinite(numeric) || numeric <= 0) return "";
    const clamped = Math.min(5, Math.max(0, numeric));
    return `${"★".repeat(clamped)}${"☆".repeat(5 - clamped)}`;
  }

  /** @param {any} item */
  function openDesignDetail(item) {
    const designId = Number(item.id);
    if (!Number.isFinite(designId) || designId <= 0) return;

    const ids = browseFilteredItems
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

  /** @param {any} event @param {any} item */
  function handleBrowseCardOpenDetail(event, item) {
    const anyProjectDropdownOpen = getBrowseCardProjectDropdowns().some((dropdown) => dropdown.hasAttribute("open"));
    if (anyProjectDropdownOpen) {
      event?.preventDefault?.();
      event?.stopPropagation?.();
      closeBrowseCardProjectDropdowns();
      return;
    }
    openDesignDetail(item);
  }

  // Admin Data Loading & Mutation Handlers
  async function loadAdminDataForCurrentRoute(force = false) {
    if (adminLoading && !force) return;

    adminLoading = true;
    try {
      if (adminIsDesignersRoute) {
        const result = await listDesigners();
        designers = Array.isArray(result?.items)
          ? result.items.map(/** @param {any} d */ (d) => ({ id: Number(d.id), name: String(d.name || ""), designCount: Number(d.design_count || 0) }))
          : [];
      } else if (adminIsTagsRoute) {
        const result = await listTags();
        const rawTags = Array.isArray(result?.items) ? result.items : [];
        const mappedTags = rawTags.map((t) => ({ id: Number(t.id), description: String(t.description || ""), tagGroup: String(t.tag_group || "") }));

        const groups = splitTagsByGroup(mappedTags);
        imageTags = groups.image;
        stitchingTags = groups.stitching;
        unclassifiedTags = groups.unclassified;
        adminTagsPanelStateLoaded = true;
      } else if (adminIsSourcesRoute) {
        const result = await listSources();
        sources = Array.isArray(result?.items)
          ? result.items.map((s) => ({ id: Number(s.id), name: String(s.name || ""), designCount: Number(s.design_count || 0) }))
          : [];
      } else if (adminIsHoopsRoute) {
        const result = await listHoops();
        hoops = Array.isArray(result?.items)
          ? result.items.map((h) => ({ id: Number(h.id), name: String(h.name || ""), maxWidthMm: Number(h.max_width_mm || 0), maxHeightMm: Number(h.max_height_mm || 0), designCount: Number(h.design_count || 0) }))
          : [];
      }
    } catch (e) {
      setAdminNotice(`Failed to load admin data: ${e}`, "error");
    } finally {
      adminLoading = false;
    }
  }

  /** @param {any} event */
  async function addDesigner(event) {
    event.preventDefault();
    const name = newDesignerName.trim();
    if (!name) return;

    const result = await createDesigner(name);
    if (!result?.persisted) {
      setAdminNotice(`Could not add designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newDesignerName = "";
    setAdminNotice("Designer added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} designer */
  function beginEditDesigner(designer) {
    if (!designer) return;
    pendingDeleteDesignerId = null;
    editingDesignerId = Number(designer.id);
    editingDesignerName = String(designer.name || "");
  }

  function cancelEditDesigner() {
    editingDesignerId = null;
    editingDesignerName = "";
  }

  /** @param {number} id */
  async function saveDesignerEdit(id) {
    const name = editingDesignerName.trim();
    if (!name) {
      setAdminNotice("Enter a designer name.", "error");
      return;
    }

    const result = await updateDesigner(id, name);
    if (!result?.persisted) {
      setAdminNotice(`Could not update designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditDesigner();
    setAdminNotice("Designer updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} designer */
  function requestDeleteDesigner(designer) {
    if (!designer) return;
    cancelEditDesigner();
    pendingDeleteDesignerId = Number(designer.id);
    if (Number(designer.designCount) > 0) {
      setAdminNotice(`Deleting '${designer.name}' will clear assignment from ${designer.designCount} design(s).`, "info");
      return;
    }
    setAdminNotice(`Delete '${designer.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteDesigner() {
    pendingDeleteDesignerId = null;
    setAdminNotice("");
  }

  /** @param {number} id */
  async function deleteDesigner(id) {
    const result = await removeDesigner(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteDesignerId = null;
    setAdminNotice("Designer deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  function clearNewDesignerForm() {
    newDesignerName = "";
  }

  /** @param {any} event */
  async function addSource(event) {
    event.preventDefault();
    const name = newSourceName.trim();
    if (!name) return;

    const result = await createSource(name);
    if (!result?.persisted) {
      setAdminNotice(`Could not add source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newSourceName = "";
    setAdminNotice("Source added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} source */
  function beginEditSource(source) {
    if (!source) return;
    pendingDeleteSourceId = null;
    editingSourceId = Number(source.id);
    editingSourceName = String(source.name || "");
  }

  function cancelEditSource() {
    editingSourceId = null;
    editingSourceName = "";
  }

  /** @param {number} id */
  async function saveSourceEdit(id) {
    const name = editingSourceName.trim();
    if (!name) {
      setAdminNotice("Enter a source name.", "error");
      return;
    }

    const result = await updateSource(id, name);
    if (!result?.persisted) {
      setAdminNotice(`Could not update source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditSource();
    setAdminNotice("Source updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} source */
  function requestDeleteSource(source) {
    if (!source) return;
    cancelEditSource();
    pendingDeleteSourceId = Number(source.id);
    if (Number(source.designCount) > 0) {
      setAdminNotice(`Deleting '${source.name}' will clear assignment from ${source.designCount} design(s).`, "info");
      return;
    }
    setAdminNotice(`Delete '${source.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteSource() {
    pendingDeleteSourceId = null;
    setAdminNotice("");
  }

  /** @param {number} id */
  async function deleteSource(id) {
    const result = await removeSource(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete source: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteSourceId = null;
    setAdminNotice("Source deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  function clearNewSourceForm() {
    newSourceName = "";
  }

  /** @param {any} event */
  async function addHoop(event) {
    event.preventDefault();
    const name = newHoopName.trim();
    const w = Number(newHoopWidth);
    const h = Number(newHoopHeight);
    if (!name || w <= 0 || h <= 0) return;

    const result = await createHoop(name, w, h);
    if (!result?.persisted) {
      setAdminNotice(`Could not add hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newHoopName = "";
    newHoopWidth = 0;
    newHoopHeight = 0;
    setAdminNotice("Hoop added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} hoop */
  function beginEditHoop(hoop) {
    if (!hoop) return;
    pendingDeleteHoopId = null;
    editingHoopId = Number(hoop.id);
    editingHoopName = String(hoop.name || "");
    editingHoopWidth = Number(hoop.maxWidthMm || 0);
    editingHoopHeight = Number(hoop.maxHeightMm || 0);
  }

  function cancelEditHoop() {
    editingHoopId = null;
    editingHoopName = "";
    editingHoopWidth = 0;
    editingHoopHeight = 0;
  }

  /** @param {number} id */
  async function saveHoopEdit(id) {
    const name = editingHoopName.trim();
    const w = Number(editingHoopWidth);
    const h = Number(editingHoopHeight);
    if (!name || w <= 0 || h <= 0) {
      setAdminNotice("Enter hoop details.", "error");
      return;
    }

    const result = await updateHoop(id, name, w, h);
    if (!result?.persisted) {
      setAdminNotice(`Could not update hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditHoop();
    setAdminNotice("Hoop updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {any} hoop */
  function requestDeleteHoop(hoop) {
    if (!hoop) return;
    cancelEditHoop();
    pendingDeleteHoopId = Number(hoop.id);
    if (Number(hoop.designCount) > 0) {
      setAdminNotice(`Deleting '${hoop.name}' will clear assignment from ${hoop.designCount} design(s).`, "info");
      return;
    }
    setAdminNotice(`Delete '${hoop.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteHoop() {
    pendingDeleteHoopId = null;
    setAdminNotice("");
  }

  /** @param {number} id */
  async function deleteHoop(id) {
    const result = await removeHoop(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteHoopId = null;
    setAdminNotice("Hoop deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  function clearNewHoopForm() {
    newHoopName = "";
    newHoopWidth = 0;
    newHoopHeight = 0;
  }

  /** @param {any} event */
  async function addTag(event) {
    event.preventDefault();
    const desc = newTagDescription.trim();
    if (!desc) return;

    const result = await createTag(desc, newTagGroup);
    if (!result?.persisted) {
      setAdminNotice(`Could not add tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newTagDescription = "";
    setAdminNotice("Tag added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {number} id */
  async function deleteTag(id) {
    const result = await removeTag(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    setAdminNotice("Tag deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {number} id @param {string | null} group */
  async function setTagGroup(id, group) {
    const result = await updateTagGroup(id, group || null);
    if (!result?.persisted) {
      setAdminNotice(`Could not set tag group: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    setAdminNotice("Tag updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  /** @param {string} panel @param {any} event */
  function handleAdminTagPanelToggle(panel, event) {
    const isOpen = Boolean(event?.currentTarget?.open);
    if (panel === "image") {
      adminImageTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.image", isOpen ? "open" : "closed");
      }
    }
    if (panel === "stitching") {
      adminStitchingTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.stitching", isOpen ? "open" : "closed");
      }
    }
  }

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("main-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
      },
    };
  }

  // Reactive effects for routing/loading
  $effect(() => {
    if (currentRoute === "#/designs" && (!browseHasLoaded || browseNeedsRefresh)) {
      untrack(() => {
        loadBrowseItems(true);
        browseNeedsRefresh = false;
      });
    }
  });

  $effect(() => {
    if (currentRoute === "#/designs" && browseTagOptions.length === 0) {
      untrack(() => {
        loadBrowseTags();
      });
    }
  });

  $effect(() => {
    if (currentRoute === "#/designs" && !browseFilterReferenceLoaded) {
      untrack(() => {
        loadBrowseFilterReferenceData();
      });
    }
  });

  $effect(() => {
    if (currentRoute !== "#/designs") return;
    const ids = browsePageItems.map((item) => item.id);
    untrack(() => {
      loadBrowsePreviews(ids);
    });
  });

  $effect(() => {
    if (currentRoute === "#/designs") {
      browseFilteredItems.length;
      tick().then(() => {
        untrack(() => {
          refreshBrowseGridColumns();
        });
      });
    }
  });

  $effect(() => {
    const id = detailDesignId ?? printDesignId;
    if (id !== null) {
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
    if (currentUiKind === "admin-list") {
      currentRoute;
      untrack(() => {
        loadAdminDataForCurrentRoute();
      });
    }
  });

  $effect(() => {
    if (currentRoute !== "#/admin/tags" || adminTagsPanelStateLoaded || typeof window === "undefined") return;

    const imageSavedState = window.localStorage.getItem("admin.tags.collapsible.image");
    const stitchingSavedState = window.localStorage.getItem("admin.tags.collapsible.stitching");
    if (imageSavedState === "open" || imageSavedState === "closed") {
      adminImageTagsOpen = imageSavedState === "open";
    }
    if (stitchingSavedState === "open" || stitchingSavedState === "closed") {
      adminStitchingTagsOpen = stitchingSavedState === "open";
    }
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
    const validIds = new Set(browseCardItems.map((item) => item.id));
    const next = browseSelectedIds.filter((id) => validIds.has(id));
    if (next.length !== browseSelectedIds.length || next.some((id, index) => id !== browseSelectedIds[index])) {
      browseSelectedIds = next;
    }
  });

  // Watch for page notices to clear
  let lastRoute = "";
  $effect(() => {
    const route = currentRoute;
    if (route !== lastRoute) {
      if (route === "#/designs") {
        browseActionNotice = "";
      }
      if (route.startsWith("#/admin/")) {
        adminNotice = "";
        adminNoticeType = "info";
      }
      lastRoute = route;
    }
  });

  let browseCardItems = $derived(browsePageItems);

  syncRouteFromHash();
</script>

<svelte:window onhashchange={syncRouteFromHash} onresize={refreshBrowseGridColumns} />

<nav class="menu-shell text-white shadow font-sans">
  <div class="menu-shell-inner max-w-7xl mx-auto flex items-center justify-between px-4 py-3">
    <div class="menu-primary-group flex items-center gap-4">
      <a href="#/designs" class="menu-brand flex items-center gap-1.5 font-bold text-lg text-white">
        <span aria-hidden="true">🧵</span>
        <span>Embroidery Catalogue</span>
      </a>
      <a href="#/designs" class={linkClass("#/designs")}>Browse</a>
      <a href="#/import" class={linkClass("#/import")}>Import</a>
      <a href="#/projects" class={linkClass("#/projects")}>Projects</a>
      <a href="#/help" class={linkClass("#/help")}>Help</a>
    </div>

    <div class="menu-admin-group flex items-center gap-3 text-xs text-indigo-200">
      <span class="menu-admin-label opacity-70" aria-hidden="true">Admin:</span>
      <a href="#/admin/designers" class={adminLinkClass("#/admin/designers")}>Designers</a>
      <a href="#/admin/tags" class={adminLinkClass("#/admin/tags")}>Tags</a>
      <a href="#/admin/sources" class={adminLinkClass("#/admin/sources")}>Sources</a>
      <a href="#/admin/hoops" class={adminLinkClass("#/admin/hoops")}>Hoops</a>
      <a href="#/admin/settings" class={adminLinkClass("#/admin/settings")}>Settings</a>
      <a href="#/admin/maintenance/backup" class={adminLinkClass("#/admin/maintenance/backup")}>Backup</a>
      <a href="#/admin/tagging-actions" class={adminLinkClass("#/admin/tagging-actions")}>Tagging Actions</a>
      <a href="#/admin/orphans" class={adminLinkClass("#/admin/orphans")}>Orphans</a>
    </div>
  </div>
</nav>

<main class="max-w-7xl mx-auto px-4 py-6 font-sans">
  {#if currentUiKind === "browse"}
    <section class="browse-section space-y-4">
      <h1 class="ui-page-title browse-title text-2xl font-bold text-gray-800">Browse Designs</h1>

      <form
        class="browse-search-shell space-y-3 no-print bg-white rounded shadow p-4 border"
        onsubmit={(event) => {
          event.preventDefault();
          applyBrowseFilters();
        }}
      >
        <div class="ui-section-shell browse-general-search space-y-1.5">
          <label class="ui-section-label browse-general-search-label block text-xs font-semibold text-gray-600 uppercase" for="browse-q">General search</label>
          <div class="browse-general-search-row flex items-center gap-2">
            <input
              id="browse-q"
              class="ui-text-input ui-control-text-inset browse-general-input text-sm flex-1 min-w-[20rem] font-mono border rounded px-3 py-2"
              placeholder='e.g. rose "cross stitch" -applique or *.hus'
              value={browseFilters.q}
              oninput={(event) => updateBrowseFilter("q", event.currentTarget.value)}
            />
            <label class="ui-field-label browse-unverified-label flex items-center gap-1.5 cursor-pointer select-none text-sm text-gray-700 whitespace-nowrap">
              <input
                type="checkbox"
                class="ui-checkbox browse-unverified-checkbox accent-indigo-600 rounded"
                checked={browseFilters.unverifiedOnly}
                onchange={(event) => updateBrowseFilter("unverifiedOnly", event.currentTarget.checked)}
              />
              Unverified only
            </label>
          </div>
          <p class="ui-help-note browse-general-help text-xs text-gray-500 mt-0.5">
            Supports Google-like syntax: "exact phrase" · -exclude · word1 OR word2 · *.hus ·
            <a href="#/help?section=search" class="text-indigo-600 hover:underline">Search help</a>
          </p>
        </div>

        <details class="ui-section-shell browse-additional-filters overflow-visible relative" open={browseAdditionalFiltersOpen}>
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
          <div class="grid sm:grid-cols-2 md:grid-cols-4 gap-4 pt-3 border-t mt-2">
            <!-- Designers Filter -->
            <div class="space-y-1">
              <span class="block text-xs font-semibold text-gray-700">Designer</span>
              <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
                {#each browseDesignerFilterOptions as opt}
                  <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                    <input type="checkbox" checked={browseFilters.designerFilters.includes(opt)} onchange={() => toggleBrowseFilter("designerFilters", opt)} class="accent-indigo-600 rounded" />
                    <span>{opt}</span>
                  </label>
                {/each}
              </div>
            </div>

            <!-- Tags Filter -->
            <div class="space-y-1">
              <span class="block text-xs font-semibold text-gray-700">Tag</span>
              <div class="border rounded bg-white max-h-36 overflow-auto p-1.5 space-y-1">
                {#each browseTagOptions as opt}
                  <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                    <input type="checkbox" checked={browseFilters.tagFilters.includes(opt.description)} onchange={() => toggleBrowseFilter("tagFilters", opt.description)} class="accent-indigo-600 rounded" />
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
                    <input type="checkbox" checked={browseFilters.sourceFilters.includes(opt)} onchange={() => toggleBrowseFilter("sourceFilters", opt)} class="accent-indigo-600 rounded" />
                    <span>{opt}</span>
                  </label>
                {/each}
              </div>
            </div>

            <!-- Other Properties -->
            <div class="space-y-2.5 text-xs">
              <label class="block">
                <span class="block font-semibold text-gray-700 mb-1">Hoop size</span>
                <select class="border rounded px-2.5 py-1.5 w-full bg-white text-xs" value={browseFilters.hoop} onchange={(e) => updateBrowseFilter("hoop", e.currentTarget.value)}>
                  <option value="">Any hoop</option>
                  {#each browseHoopFilterOptions as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
              </label>

              <div class="grid grid-cols-2 gap-2">
                <label class="block">
                  <span class="block font-semibold text-gray-700 mb-1">Rating</span>
                  <select class="border rounded px-2.5 py-1.5 w-full bg-white text-xs" value={browseFilters.rating} onchange={(e) => updateBrowseFilter("rating", e.currentTarget.value)}>
                    <option value="">Any</option>
                    {#each [1, 2, 3, 4, 5] as score}
                      <option value={String(score)}>{score}★</option>
                    {/each}
                  </select>
                </label>
                <label class="block">
                  <span class="block font-semibold text-gray-700 mb-1">Stitched</span>
                  <select class="border rounded px-2.5 py-1.5 w-full bg-white text-xs" value={browseFilters.stitched} onchange={(e) => updateBrowseFilter("stitched", e.currentTarget.value)}>
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
        <div class="flex flex-wrap items-center justify-between gap-3 pt-2 text-xs border-t text-gray-600">
          <div class="flex flex-wrap items-center gap-3">
            <label class="flex items-center gap-1.5 font-medium">
              Sort by:
              <select class="border rounded px-2 py-1 bg-white text-xs" value={browseFilters.sortBy} onchange={(e) => updateBrowseFilter("sortBy", e.currentTarget.value)}>
                <option value="name">Name</option>
                <option value="rating">Rating</option>
                <option value="stitched">Stitched</option>
                <option value="folder">Folder</option>
                <option value="date_added">Date Added</option>
              </select>
            </label>
            <label class="flex items-center gap-1.5 font-medium">
              Direction:
              <select class="border rounded px-2 py-1 bg-white text-xs" value={browseFilters.sortDir} onchange={(e) => updateBrowseFilter("sortDir", e.currentTarget.value)}>
                <option value="asc">Ascending</option>
                <option value="desc">Descending</option>
              </select>
            </label>
            <button type="button" class="text-indigo-600 hover:underline" onclick={clearBrowseFilters}>Reset filters</button>
          </div>
          <div>
            <span>Found <strong>{browseFilteredItems.length}</strong> design(s).</span>
          </div>
        </div>
      </form>

      <!-- Browse Results Grid -->
      <div
        bind:this={browseGridContainer}
        class="browse-grid-container grid gap-4"
        style={`grid-template-columns: repeat(${browseGridColumns}, minmax(0, 1fr));`}
      >
        {#if browseLoading && browseItems.length === 0}
          <p class="text-center py-12 text-gray-500 font-medium col-span-full">Loading designs...</p>
        {:else if browseFilteredItems.length === 0}
          <p class="text-center py-12 text-gray-500 font-medium col-span-full">No designs match your filters.</p>
        {:else}
          {#each browseCardItems as item (item.id)}
            <article class="browse-card border rounded-lg bg-white overflow-hidden shadow-sm flex flex-col hover:shadow transition relative" data-id={item.id}>
              <!-- Selection checkbox -->
              <label class="absolute top-2.5 left-2.5 z-10 bg-white/95 rounded p-1 shadow-sm border cursor-pointer select-none">
                <input
                  type="checkbox"
                  class="rounded accent-indigo-650"
                  checked={browseSelectedIds.includes(item.id)}
                  oninput={() => toggleBrowseCardSelection(item.id, !browseSelectedIds.includes(item.id))}
                />
              </label>

              <button class="browse-card-link w-full text-left flex flex-col flex-1" onclick={(event) => handleBrowseCardOpenDetail(event, item)}>
                {#if browsePreviewById[item.id]}
                  <div class="browse-card-image-frame bg-gray-50 p-2 flex items-center justify-center h-48 border-b">
                    <img
                      src={browsePreviewById[item.id]}
                      alt={item.filename}
                      class="browse-card-image max-h-full object-contain"
                      loading="lazy"
                    />
                  </div>
                {:else}
                  <div class="browse-card-image-frame bg-gray-50 p-2 flex items-center justify-center h-48 border-b text-xs text-gray-400 font-medium italic">
                    {browsePreviewsLoading ? "Loading image..." : "No preview image"}
                  </div>
                {/if}
                <div class="browse-card-meta p-4 flex-1 flex flex-col justify-between">
                  <div>
                    <div class="browse-card-title-row flex items-start justify-between gap-1.5">
                      <p class="browse-card-title text-sm font-semibold text-gray-800 truncate flex-1" title={item.filename}>{item.filename}</p>
                      {#if item.tagsChecked}
                        <span
                          class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-green-500"
                          title="Verified"
                          aria-label="Verified"
                        >
                          ✓
                        </span>
                      {/if}
                    </div>
                    <p class="browse-card-hoop text-xs font-semibold text-indigo-600 mt-1">{item.hoop || "Hoop unknown"}</p>
                    {#if item.projects.length > 0}
                      <p class="browse-card-projects text-[11px] text-gray-500 mt-1 truncate" title={item.projects.join(", ")}>
                        📁 {item.projects.join(", ")}
                      </p>
                    {/if}
                  </div>
                  <div class="pt-2">
                    {#if item.tags.length > 0}
                      <p class="browse-card-tags text-[11px] text-gray-500 truncate" title={item.tags.join(", ")}>
                        {item.tags.join(", ")}
                      </p>
                    {:else}
                      <p class="browse-card-tags text-[11px] text-gray-300 italic">No tags</p>
                    {/if}
                    <p class="browse-card-rating text-xs text-yellow-500 font-bold mt-1" aria-label={`Rating ${item.rating ?? 0} out of 5`}>
                      {browseStars(item.rating ?? 0)}
                    </p>
                  </div>
                </div>
              </button>

              {#if browseProjects.length > 0}
                <details
                  class="browse-card-project-details px-4 py-2 bg-gray-50 border-t no-print"
                  ontoggle={(event) => handleBrowseCardProjectDetailsToggle(item, event.currentTarget)}
                >
                  <summary class="browse-card-project-summary text-xs font-semibold text-gray-500 cursor-pointer hover:text-indigo-600 select-none">
                    + Add to project
                  </summary>
                  <div class="ui-checkbox-list-shell mt-1.5 max-h-36 overflow-auto px-2 py-1.5 border rounded bg-white space-y-1">
                    {#each browseProjects as project}
                      <label class="ui-field-label flex items-center gap-1.5 text-xs text-gray-700 cursor-pointer">
                        <input
                          type="checkbox"
                          class="ui-checkbox accent-indigo-650 rounded"
                          checked={isBrowseCardProjectChecked(item, project.id)}
                          onchange={(event) => updateBrowseCardProjectPending(item.id, project.id, event.currentTarget.checked)}
                        />
                        <span>{project.name}</span>
                      </label>
                    {/each}
                  </div>
                </details>
              {/if}
            </article>
          {/each}
        {/if}
      </div>

      <!-- Pagination -->
      <Pagination
        currentPage={browseCurrentPage}
        totalPages={browseTotalPages}
        onPageChange={(/** @type {number} */ page) => { browseCurrentPage = page; }}
        disabled={browseLoading}
        showFirstLast={true}
        windowSize={2}
        ariaLabel="Browse pagination"
      />

      {#if browseActionNotice}
        <p class="text-sm text-indigo-700 bg-indigo-50 border border-indigo-200 rounded px-3 py-2 shadow-sm font-medium">{browseActionNotice}</p>
      {/if}
    </section>
  {:else if currentUiKind === "settings"}
    <SettingsView />
  {:else if currentUiKind === "backup"}
    <BackupView />
  {:else if currentUiKind === "tagging-actions"}
    <TaggingActionsView />
  {:else if currentUiKind === "orphans"}
    <OrphansView />
  {:else if currentUiKind === "projects-list" || currentUiKind === "project-new" || currentUiKind === "project-detail" || currentUiKind === "project-print"}
    <ProjectsView
      {currentUiKind}
      {projectDetailId}
      {projectPrintId}
      {navigateTo}
    />
  {:else if currentUiKind === "design-detail"}
    <DesignDetailView
      {detailDesignId}
      {detailBrowseIds}
      {detailBrowseIndex}
      {navigateTo}
    />
  {:else if currentUiKind === "design-print"}
    <DesignPrintView
      {printDesignId}
      {navigateTo}
    />
  {:else if currentUiKind === "import"}
    <ImportView
      {currentRoute}
      {navigateTo}
      onImportCompleted={handleImportCompleted}
    />
  {:else if currentUiKind === "about"}
    <AboutView />
  {:else if currentUiKind === "about-document"}
    <AboutDocumentView
      slug={aboutDocumentSlug}
    />
  {:else if currentUiKind === "help"}
    <HelpView />
  {:else if currentUiKind === "admin-list"}
    <section class="admin-page space-y-4">
      {#if adminNotice}
        <div
          class="admin-alert rounded px-4 py-2 text-sm border font-medium shadow-sm"
          class:bg-green-50={adminNoticeType === "success"}
          class:border-green-300={adminNoticeType === "success"}
          class:text-green-800={adminNoticeType === "success"}
          class:bg-red-50={adminNoticeType === "error"}
          class:border-red-300={adminNoticeType === "error"}
          class:text-red-800={adminNoticeType === "error"}
          class:bg-blue-50={adminNoticeType !== "success" && adminNoticeType !== "error"}
          class:border-blue-200={adminNoticeType !== "success" && adminNoticeType !== "error"}
          class:text-blue-800={adminNoticeType !== "success" && adminNoticeType !== "error"}
        >
          {adminNotice}
        </div>
      {/if}

      {#if adminIsDesignersRoute}
        <div class="space-y-1 font-sans">
          <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800">Manage Designers</h1>
          <p class="text-gray-600 text-sm">Designers are the creators or brands of embroidery designs.</p>
        </div>

        <div class="admin-card bg-white rounded shadow p-5 max-w-xl border mt-2">
          <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new designer</h2>
          <form class="flex gap-2" onsubmit={addDesigner}>
            <input
              type="text"
              bind:value={newDesignerName}
              required
              placeholder="New designer name..."
              class="admin-input flex-1 border rounded px-3 py-2 text-sm"
            />
            <button type="submit" class="menu-button-primary text-sm" disabled={!canAddDesigner}>Add</button>
            <button type="button" class="menu-button-secondary text-sm" onclick={clearNewDesignerForm} disabled={!canClearDesignerForm}>Clear</button>
          </form>
        </div>

        <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
          <table class="w-full text-sm text-left">
            <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
              <tr>
                <th class="px-4 py-3">Name</th>
                <th class="px-4 py-3 text-right">Used by</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              {#if designers.length === 0}
                <tr>
                  <td colspan="3" class="px-4 py-3 text-gray-400 italic">No designers yet.</td>
                </tr>
              {:else}
                {#each designers as designer}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">
                      {#if editingDesignerId === designer.id}
                        <input
                          type="text"
                          bind:value={editingDesignerName}
                          class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                        />
                      {:else}
                        {designer.name}
                      {/if}
                    </td>
                    <td class="px-4 py-2 text-right text-gray-600 font-mono">{designer.designCount}</td>
                    <td class="px-4 py-2 text-right">
                      <div class="flex justify-end gap-2.5 flex-wrap">
                        {#if editingDesignerId === designer.id}
                          <button type="button" class="text-indigo-650 hover:underline text-xs font-semibold" onclick={() => saveDesignerEdit(designer.id)}>
                            Save
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelEditDesigner}>
                            Cancel
                          </button>
                        {:else if pendingDeleteDesignerId === designer.id}
                          <button type="button" class="text-red-600 hover:underline text-xs font-bold" onclick={() => deleteDesigner(designer.id)}>
                            Confirm delete
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelDeleteDesigner}>
                            Cancel
                          </button>
                        {:else}
                          <button type="button" class="text-indigo-655 hover:underline text-xs font-semibold" onclick={() => beginEditDesigner(designer)}>
                            Edit
                          </button>
                          <button type="button" class="text-red-400 hover:underline text-xs font-semibold" onclick={() => requestDeleteDesigner(designer)}>
                            Delete
                          </button>
                        {/if}
                      </div>
                    </td>
                  </tr>
                  {#if pendingDeleteDesignerId === designer.id}
                    <tr class="bg-amber-50">
                      <td colspan="3" class="px-4 py-2 text-xs text-amber-800">
                        {#if designer.designCount > 0}
                          This designer is currently used by {designer.designCount} design(s). If you delete it, those designs will no longer have a designer assigned.
                        {:else}
                          Confirm deletion for this designer.
                        {/if}
                      </td>
                    </tr>
                  {/if}
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      {:else if adminIsTagsRoute}
        <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800">Manage Tags</h1>
        <p class="text-sm text-gray-500">
          Use Image tags for subject categories and Stitching tags for technique or style.
        </p>

        <div class="admin-card bg-white rounded shadow p-5 max-w-3xl border">
          <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new tag</h2>
          <form class="flex flex-wrap gap-3 items-end" onsubmit={addTag}>
            <div>
              <label for="admin-tag-description" class="block text-xs font-semibold text-gray-650 mb-1">Description</label>
              <input
                id="admin-tag-description"
                type="text"
                bind:value={newTagDescription}
                required
                placeholder="e.g. Animals, Cross stitch..."
                class="admin-input border rounded px-3 py-2 text-sm w-56 font-sans"
              />
            </div>
            <div>
              <label for="admin-tag-group" class="block text-xs font-semibold text-gray-650 mb-1">Group</label>
              <select id="admin-tag-group" bind:value={newTagGroup} class="admin-input border rounded px-3 py-2 text-sm bg-white font-sans">
                <option value="image">Image</option>
                <option value="stitching">Stitching</option>
              </select>
            </div>
            <button type="submit" class="menu-button-primary text-sm py-2">Add</button>
          </form>
        </div>

        <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border" open={adminImageTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("image", event)}>
          <summary class="bg-green-50 border-b border-green-200 px-4 py-2.5 flex items-center gap-2 cursor-pointer select-none">
            <svg class={`h-4 w-4 text-green-700 transition-transform duration-200 ${adminImageTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
            </svg>
            <h2 class="text-sm font-bold text-green-800 tracking-wide">Image Tags</h2>
          </summary>
          <table class="w-full text-sm text-left">
            <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
              <tr>
                <th class="px-4 py-2.5">Description</th>
                <th class="px-4 py-2.5">Group</th>
                <th class="px-4 py-2.5"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              {#if imageTags.length === 0}
                <tr><td colspan="3" class="px-4 py-3 text-gray-400 italic">No image tags yet.</td></tr>
              {:else}
                {#each imageTags as tag}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">{tag.description}</td>
                    <td class="px-4 py-2">
                      <select
                        value={tag.tagGroup}
                        class="admin-input border rounded px-2.5 py-1 text-xs bg-white"
                        onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                      >
                        <option value="">Unclassified</option>
                        <option value="image">Image</option>
                        <option value="stitching">Stitching</option>
                      </select>
                    </td>
                    <td class="px-4 py-2 text-right">
                      <button type="button" class="text-red-400 hover:text-red-650 hover:underline text-xs font-semibold" onclick={() => deleteTag(tag.id)}>Delete</button>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </details>

        <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border" open={adminStitchingTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("stitching", event)}>
          <summary class="bg-blue-50 border-b border-blue-200 px-4 py-2.5 flex items-center gap-2 cursor-pointer select-none">
            <svg class={`h-4 w-4 text-blue-700 transition-transform duration-200 ${adminStitchingTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
            </svg>
            <h2 class="text-sm font-bold text-blue-800 tracking-wide">Stitching Tags</h2>
          </summary>
          <table class="w-full text-sm text-left">
            <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
              <tr>
                <th class="px-4 py-2.5">Description</th>
                <th class="px-4 py-2.5">Group</th>
                <th class="px-4 py-2.5"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              {#if stitchingTags.length === 0}
                <tr><td colspan="3" class="px-4 py-3 text-gray-400 italic">No stitching tags yet.</td></tr>
              {:else}
                {#each stitchingTags as tag}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">{tag.description}</td>
                    <td class="px-4 py-2">
                      <select
                        value={tag.tagGroup}
                        class="admin-input border rounded px-2.5 py-1 text-xs bg-white"
                        onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                      >
                        <option value="">Unclassified</option>
                        <option value="image">Image</option>
                        <option value="stitching">Stitching</option>
                      </select>
                    </td>
                    <td class="px-4 py-2 text-right">
                      <button type="button" class="text-red-400 hover:text-red-650 hover:underline text-xs font-semibold" onclick={() => deleteTag(tag.id)}>Delete</button>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </details>

        {#if unclassifiedTags.length > 0}
          <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
            <div class="bg-amber-50 border-b border-amber-200 px-4 py-2.5">
              <h2 class="text-sm font-bold text-amber-800 tracking-wide">Unclassified Tags</h2>
            </div>
            <table class="w-full text-sm text-left">
              <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
                <tr>
                  <th class="px-4 py-2.5">Description</th>
                  <th class="px-4 py-2.5">Group</th>
                  <th class="px-4 py-2.5"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-100">
                {#each unclassifiedTags as tag}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">{tag.description}</td>
                    <td class="px-4 py-2">
                      <select
                        value={tag.tagGroup}
                        class="admin-input border rounded px-2.5 py-1 text-xs bg-white"
                        onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                      >
                        <option value="">Unclassified</option>
                        <option value="image">Image</option>
                        <option value="stitching">Stitching</option>
                      </select>
                    </td>
                    <td class="px-4 py-2 text-right">
                      <button type="button" class="text-red-400 hover:text-red-655 hover:underline text-xs font-semibold" onclick={() => deleteTag(tag.id)}>Delete</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else if adminIsSourcesRoute}
        <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800 font-sans">Manage Sources</h1>
        <p class="text-sm text-gray-500">
          Sources describe where your designs came from, such as Purchased, Downloaded, or Gift.
        </p>

        <div class="admin-card bg-white rounded shadow p-5 max-w-xl border mt-2">
          <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new source</h2>
          <form class="flex gap-2" onsubmit={addSource}>
            <input
              type="text"
              bind:value={newSourceName}
              required
              placeholder="e.g. Purchased, Downloaded..."
              class="admin-input flex-1 border rounded px-3 py-2 text-sm"
            />
            <button type="submit" class="menu-button-primary text-sm" disabled={!canAddSource}>Add</button>
            <button type="button" class="menu-button-secondary text-sm" onclick={clearNewSourceForm} disabled={!canClearSourceForm}>Clear</button>
          </form>
        </div>

        <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
          <table class="w-full text-sm text-left">
            <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
              <tr>
                <th class="px-4 py-3">Name</th>
                <th class="px-4 py-3 text-right">Used by</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              {#if sources.length === 0}
                <tr>
                  <td colspan="3" class="px-4 py-3 text-gray-400 italic">No sources yet.</td>
                </tr>
              {:else}
                {#each sources as source}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">
                      {#if editingSourceId === source.id}
                        <input
                          type="text"
                          bind:value={editingSourceName}
                          class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                        />
                      {:else}
                        {source.name}
                      {/if}
                    </td>
                    <td class="px-4 py-2 text-right text-gray-600 font-mono">{source.designCount}</td>
                    <td class="px-4 py-2 text-right">
                      <div class="flex justify-end gap-2.5 flex-wrap">
                        {#if editingSourceId === source.id}
                          <button type="button" class="text-indigo-650 hover:underline text-xs font-semibold" onclick={() => saveSourceEdit(source.id)}>
                            Save
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelEditSource}>
                            Cancel
                          </button>
                        {:else if pendingDeleteSourceId === source.id}
                          <button type="button" class="text-red-600 hover:underline text-xs font-bold" onclick={() => deleteSource(source.id)}>
                            Confirm delete
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelDeleteSource}>
                            Cancel
                          </button>
                        {:else}
                          <button type="button" class="text-indigo-655 hover:underline text-xs font-semibold" onclick={() => beginEditSource(source)}>
                            Edit
                          </button>
                          <button type="button" class="text-red-400 hover:underline text-xs font-semibold" onclick={() => requestDeleteSource(source)}>
                            Delete
                          </button>
                        {/if}
                      </div>
                    </td>
                  </tr>
                  {#if pendingDeleteSourceId === source.id}
                    <tr class="bg-amber-50">
                      <td colspan="3" class="px-4 py-2 text-xs text-amber-800">
                        {#if source.designCount > 0}
                          This source is currently used by {source.designCount} design(s). If you delete it, those designs will no longer have a source assigned.
                        {:else}
                          Confirm deletion for this source.
                        {/if}
                      </td>
                    </tr>
                  {/if}
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      {:else if adminIsHoopsRoute}
        <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800 font-sans">Manage Hoops</h1>
        <p class="text-sm text-gray-500">
          Hoop sizes depend on your machine and the frames you own. Add your own hoops below.
        </p>

        <div class="admin-card bg-white rounded shadow p-5 max-w-4xl border mt-2">
          <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new hoop</h2>
          <form class="flex gap-3 items-end flex-wrap" onsubmit={addHoop}>
            <div>
              <label for="admin-hoop-name" class="block text-xs font-semibold text-gray-650 mb-1">Name</label>
              <input
                id="admin-hoop-name"
                type="text"
                bind:value={newHoopName}
                required
                placeholder="e.g. 5x7 hoop"
                class="admin-input border rounded px-3 py-2 text-sm w-52 font-sans"
              />
            </div>
            <div>
              <label for="admin-hoop-width" class="block text-xs font-semibold text-gray-650 mb-1">Max Width (mm)</label>
              <input
                id="admin-hoop-width"
                type="number"
                min="1"
                step="1"
                bind:value={newHoopWidth}
                required
                class="admin-input border rounded px-3 py-2 text-sm w-36 font-sans text-right"
              />
            </div>
            <div>
              <label for="admin-hoop-height" class="block text-xs font-semibold text-gray-650 mb-1">Max Height (mm)</label>
              <input
                id="admin-hoop-height"
                type="number"
                min="1"
                step="1"
                bind:value={newHoopHeight}
                required
                class="admin-input border rounded px-3 py-2 text-sm w-36 font-sans text-right"
              />
            </div>
            <button type="submit" class="menu-button-primary text-sm py-2" disabled={!canAddHoop}>Add</button>
            <button type="button" class="menu-button-secondary text-sm py-2" onclick={clearNewHoopForm} disabled={!canClearHoopForm}>Clear</button>
          </form>
        </div>

        <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
          <table class="w-full text-sm text-left">
            <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
              <tr>
                <th class="px-4 py-3">Name</th>
                <th class="px-4 py-3 text-right">Max width (mm)</th>
                <th class="px-4 py-3 text-right">Max height (mm)</th>
                <th class="px-4 py-3 text-right">Used by</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              {#if hoops.length === 0}
                <tr>
                  <td colspan="5" class="px-4 py-3 text-gray-400 italic">No hoops defined yet. Add your own machine hoops above.</td>
                </tr>
              {:else}
                {#each hoops as hoop}
                  <tr class="hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">
                      {#if editingHoopId === hoop.id}
                        <input
                          type="text"
                          bind:value={editingHoopName}
                          class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                        />
                      {:else}
                        {hoop.name}
                      {/if}
                    </td>
                    <td class="px-4 py-2 text-right font-mono">
                      {#if editingHoopId === hoop.id}
                        <input
                          type="number"
                          min="1"
                          step="1"
                          bind:value={editingHoopWidth}
                          class="admin-input border rounded px-2 py-1 text-sm w-28 text-right font-mono"
                        />
                      {:else}
                        {hoop.maxWidthMm.toFixed(0)}
                      {/if}
                    </td>
                    <td class="px-4 py-2 text-right font-mono">
                      {#if editingHoopId === hoop.id}
                        <input
                          type="number"
                          min="1"
                          step="1"
                          bind:value={editingHoopHeight}
                          class="admin-input border rounded px-2 py-1 text-sm w-28 text-right font-mono"
                        />
                      {:else}
                        {hoop.maxHeightMm.toFixed(0)}
                      {/if}
                    </td>
                    <td class="px-4 py-2 text-right text-gray-600 font-mono">{hoop.designCount}</td>
                    <td class="px-4 py-2 text-right">
                      <div class="flex justify-end gap-2.5 flex-wrap">
                        {#if editingHoopId === hoop.id}
                          <button type="button" class="text-indigo-650 hover:underline text-xs font-semibold" onclick={() => saveHoopEdit(hoop.id)}>
                            Save
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelEditHoop}>
                            Cancel
                          </button>
                        {:else if pendingDeleteHoopId === hoop.id}
                          <button type="button" class="text-red-600 hover:underline text-xs font-bold" onclick={() => deleteHoop(hoop.id)}>
                            Confirm delete
                          </button>
                          <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelDeleteHoop}>
                            Cancel
                          </button>
                        {:else}
                          <button type="button" class="text-indigo-655 hover:underline text-xs font-semibold" onclick={() => beginEditHoop(hoop)}>
                            Edit
                          </button>
                          <button type="button" class="text-red-400 hover:underline text-xs font-semibold" onclick={() => requestDeleteHoop(hoop)}>
                            Delete
                          </button>
                        {/if}
                      </div>
                    </td>
                  </tr>
                  {#if pendingDeleteHoopId === hoop.id}
                    <tr class="bg-amber-50">
                      <td colspan="5" class="px-4 py-2 text-xs text-amber-800">
                        {#if hoop.designCount > 0}
                          This hoop is currently used by {hoop.designCount} design(s). If you delete it, those designs will no longer have a hoop assigned.
                        {:else}
                          Confirm deletion for this hoop.
                        {/if}
                      </td>
                    </tr>
                  {/if}
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  {:else}
    <div class="bg-white rounded-xl shadow p-6 space-y-4 border">
      <h1 class="ui-page-title text-2xl font-bold text-gray-800">Route Not Found</h1>
      <p class="text-gray-600">
        The requested route does not exist. Use one of the known placeholders below.
      </p>

      <div class="flex flex-wrap gap-2 pt-2">
        <button class="menu-button-primary" onclick={() => navigateTo("#/designs")}>Go to Browse</button>
      </div>

      <div class="border border-gray-200 rounded-lg p-4 bg-gray-50 text-sm text-gray-700 shadow-inner">
        <p class="font-semibold mb-2">Known routes</p>
        <ul class="space-y-1">
          {#each ORDERED_ROUTE_HINTS as route}
            <li>{route}</li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}
</main>

<!-- Bulk Actions Bar (Sticky Bottom) -->
{#if showBrowseBulkBar}
  <div
    bind:this={browseBulkBarNode}
    use:portalToBody
    class="browse-bulk-bar ui-section-shell no-print fixed bottom-0 left-0 right-0 bg-white border-t p-4 shadow-lg flex flex-wrap items-center justify-between gap-4 z-40"
  >
    <div class="flex items-center gap-3 text-sm text-gray-700">
      <span class="font-semibold">{browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected</span>
      <button type="button" class="text-indigo-650 hover:underline" onclick={checkAllBrowseOnPage}>Select all on page</button>
      <span class="text-gray-300">|</span>
      <button type="button" class="text-indigo-650 hover:underline" onclick={checkNoneBrowse}>Deselect all</button>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <button type="button" class="menu-button-secondary ui-action-button text-xs" onclick={openBulkTagModal}>
        Choose tags
      </button>

      <button type="button" class="menu-button-secondary ui-action-button text-xs" onclick={runBulkVerify}>
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
        <div class="absolute bottom-full mb-2 right-0 bg-white border rounded shadow-lg p-3 max-h-48 overflow-auto min-w-[12rem] space-y-1.5 z-50">
          {#if browseProjects.length === 0}
            <p class="text-xs text-gray-500 italic">No projects found. Create one first.</p>
          {:else}
            {#each browseProjects as project}
              <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
                <input
                  type="checkbox"
                  class="ui-checkbox accent-indigo-650 rounded"
                  checked={browseBulkProjectSelection.includes(Number(project.id))}
                  onchange={(event) => toggleBrowseBulkProjectSelection(project.id, event.currentTarget.checked)}
                />
                <span>{project.name}</span>
              </label>
            {/each}
          {/if}
          <div class="pt-2 border-t flex justify-end">
            <button type="button" class="menu-button-primary text-[10px] py-1 px-2.5" onclick={addSelectedToProject} disabled={browseBulkProjectSelection.length === 0}>
              Apply
            </button>
          </div>
        </div>
      </details>

      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs text-red-500 border-red-200"
        onclick={openBrowseDeleteConfirm}
        disabled={browseDeleteSelectedBusy}
      >
        Delete selected
      </button>

      <button type="button" class="menu-button-primary ui-action-button ui-action-button-primary text-xs" onclick={clearBrowseSelection}>
        Clear selection
      </button>
    </div>
  </div>
{/if}

<!-- Browse Bulk Tag Modal -->
{#if browseBulkModalOpen}
  {@const tagOptionsForChooser = browseTagOptions}
  {@const groupedTagOptions = /** @type {{ image: any[], stitching: any[], unclassified: any[] }} */ (splitTagsByGroup(/** @type {any} */ (tagOptionsForChooser)))}
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
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
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
              checked={browseBulkTagSelection.includes(BROWSE_TAG_UNTAGGED)}
              onchange={(event) => {
                if (event.currentTarget.checked) {
                  browseBulkTagSelection = Array.from(new Set([...browseBulkTagSelection, BROWSE_TAG_UNTAGGED]));
                } else {
                  browseBulkTagSelection = browseBulkTagSelection.filter((value) => value !== BROWSE_TAG_UNTAGGED);
                }
              }}
            />
            <span>Untagged (clear all tags)</span>
          </label>
        </div>

        <div class="tag-chooser-sections">
          {#if groupedTagOptions.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image font-semibold">Image tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.image as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                      disabled={browseBulkTagSelection.includes(BROWSE_TAG_UNTAGGED)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.stitching.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-stitching font-semibold">Stitching tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.stitching as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                      disabled={browseBulkTagSelection.includes(BROWSE_TAG_UNTAGGED)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-unclassified font-semibold">Unclassified tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.unclassified as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                      disabled={browseBulkTagSelection.includes(BROWSE_TAG_UNTAGGED)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}
        </div>
      </div>
      <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
        <button type="button" class="menu-button-secondary" onclick={closeBulkTagModal}>Cancel</button>
        <button type="button" class="menu-button-primary" onclick={applySharedTagChooser}>
          Apply tags
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Browse Delete Selected Modal -->
{#if browseDeleteConfirmOpen}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="browse-delete-selected-title"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Close delete selected confirmation"
      onclick={closeBrowseDeleteConfirm}
    ></button>

    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
    >
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
        <h2 id="browse-delete-selected-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Delete selected designs?
        </h2>
      </div>

      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p class="text-xs text-gray-500 font-semibold" style="margin:0 0 0.75rem 0;">
          {browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.
        </p>
        <p class="text-sm text-gray-700 font-sans" style="margin:0;">
          The design(s) will be deleted from the database, but the file(s) will remain on your computer. Do you really want to do this?
        </p>
      </div>

      <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={closeBrowseDeleteConfirm}
          disabled={browseDeleteSelectedBusy}
        >
          No
        </button>
        <button
          type="button"
          class="menu-button-primary"
          onclick={confirmDeleteSelectedBrowseItems}
          disabled={browseDeleteSelectedBusy}
        >
          {browseDeleteSelectedBusy ? "Deleting..." : "Yes"}
        </button>
      </div>
    </div>
  </div>
{/if}

<footer class="max-w-7xl mx-auto px-4 pb-6 text-xs text-gray-500">
  <div class="border-t border-gray-300 pt-4 flex flex-wrap items-center gap-x-3 gap-y-1">
    <span>Embroidery Catalogue</span>
    <span aria-hidden="true">•</span>
    <a href="#/about" class="hover:underline text-indigo-650 font-medium">About</a>
    <span aria-hidden="true">•</span>
    <a href="#/about/document/licence" class="hover:underline text-indigo-650 font-medium">Licence</a>
  </div>
</footer>
