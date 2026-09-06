<script>
  import { onMount, onDestroy, untrack } from "svelte";
  import { get } from "svelte/store";
  import { importSessionStore } from "../stores/importSessionStore";
  import {
    listDesigners,
    listSources,
    previewImportFromRoots,
    precheckImportWire,
    runPrecheckAction,
    requestStopBulkImport,
    browseImportFolder,
    saveImportLastBrowseFolder,
    getSettingsViewModel,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  import {
    buildImportFolderCatalog,
    selCreate,
    selFromSerialized,
    selIsSelected,
    selCount,
    selToggleFile,
    selSelectAllInFolder,
    selDeselectAllInFolder,
    selSelectAllFolders,
    selDeselectAllFolders,
  } from "../utils/importSelection.js";
  import Pagination from "../components/Pagination.svelte";

  // Step-2 file list rendering: folders larger than this start collapsed; smaller
  // folders keep today's always-visible rows. Files within a folder page at this
  // size.
  const IMPORT_FOLDER_AUTO_EXPAND_MAX = 100;
  const IMPORT_FOLDER_PAGE_SIZE = 250;

  let { currentRoute, navigateTo, onImportCompleted } = $props();

  let settingsImportLastBrowseFolder = $state("");
  let settingsLoaded = $state(false);
  let settingsLoading = $state(false);

  let importRootPath = $state("");
  /** @type {string[]} */
  let importRootPaths = $state([]);
  let importPreview = $state(/** @type {Record<string, any> | null} */ (null));
  let importPreviewSource = $state("mock");
  let importPreviewMessage = $state("");
  let importPrecheck = $state(/** @type {Record<string, any> | null} */ (null));
  let importPrecheckSource = $state("mock");
  let importPrecheckMessage = $state("Run precheck after selecting files.");
  /**
   * Folder-scoped catalog derived from the scan result. Built once per scan;
   * independent of selection so toggles never rebuild it. Each entry has
   * { folderPath, label, filePaths }.
   * @type {Array<Record<string, any>>}
   */
  let importFolderCatalog = $derived(
    Array.isArray(importPreview?.scanned_files)
      ? buildImportFolderCatalog(importPreview.scanned_files)
      : []
  );
  /**
   * Selection state. Default = every scanned file selected (no per-file list);
   * only exceptions are stored. Shape: { deselected, selectedOnly }.
   * @type {any}
   */
  let importSelection = $state(selCreate());

  // Transient Step-2 UI state (never persisted): expanded folders, per-folder
  // filename filters, and per-folder file pages.
  /** @type {Record<string, boolean>} */
  let importExpandedByPath = $state({});
  /** @type {Record<string, string>} */
  let importFolderSearchByPath = $state({});
  /** @type {Record<string, number>} */
  let importFolderPageByPath = $state({});

  let importContextToken = $state("");
  let importActionMessage = $state("");
  let importActionSource = $state("mock");
  let importActionNeedsSkipHoopsConfirm = $state(false);
  let importActionLoading = $state(false);
  let importActionInProgress = $state("");
  let importStopRequestPending = $state(false);
  let importProgressStatus = $state("");
  let importProgressToken = $state("");
  /** @type {(() => void) | null} */
  let importProgressUnlisten = null;
  let importGlobalDesignerId = $state("");
  let importGlobalSourceId = $state("");
  /** @type {Record<string, {designerId: string, sourceId: string}>} */
  let importPerFolderAssignmentByPath = $state({});
  /** @type {Array<Record<string, any>>} */
  let importDesigners = $state([]);
  /** @type {Array<Record<string, any>>} */
  let importSources = $state([]);
  let importReferenceLoading = $state(false);
  let importLoading = $state(false);
  let importBrowseLoading = $state(false);

  let importNowInProgress = $derived(
    importActionLoading && importActionInProgress === "import_now"
  );
  // Global UI lock: reflects busyState.active so secondary controls can be
  // disabled while a long-running task runs.
  let busyActive = $derived($busyState.active);
  let importRouteStep = $derived(parseImportWizardStep(currentRoute));

  // Guards the stale-context recovery path so an expired token can never loop.
  let importRecoveryInProgress = $state(false);

  /** @param {string} route */
  function parseImportWizardStep(route) {
    if (route === "#/import") return 1;
    const match = route.match(/^#\/import\/step([123])$/);
    return match ? Number(match[1]) : null;
  }

  // -------------------------------------------------------------------------
  // Session-store bridging
  // -------------------------------------------------------------------------
  // ImportView's local `$state` is destroyed whenever the route leaves "import"
  // (clicking Admin Settings, AI Tagging Guide, About or Licence from the step 3
  // "Before You Import" panel unmounts this component via MainView).  We snapshot
  // the wizard into a singleton store on every change so returning to
  // #/import/step2 or #/import/step3 restores all selections.
  // -------------------------------------------------------------------------

  /** @returns {any} */
  function buildImportSessionSnapshot() {
    return {
      rootPath: importRootPath,
      rootPaths: importRootPaths,
      preview: importPreview,
      previewSource: importPreviewSource,
      previewMessage: importPreviewMessage,
      precheck: importPrecheck,
      precheckSource: importPrecheckSource,
      precheckMessage: importPrecheckMessage,
      folderSelection: importSelection,
      contextToken: importContextToken,
      globalDesignerId: importGlobalDesignerId,
      globalSourceId: importGlobalSourceId,
      perFolderAssignmentByPath: importPerFolderAssignmentByPath,
      actionMessage: importActionMessage,
      actionSource: importActionSource,
      actionNeedsSkipHoopsConfirm: importActionNeedsSkipHoopsConfirm,
    };
  }

  /** @param {any} snapshot */
  function applyImportSessionSnapshot(snapshot) {
    if (!snapshot || typeof snapshot !== "object") return;
    importRootPath = String(snapshot.rootPath || "");
    importRootPaths = Array.isArray(snapshot.rootPaths) ? snapshot.rootPaths.slice() : [];
    importPreview = snapshot.preview || null;
    importPreviewSource = String(snapshot.previewSource || "mock");
    importPreviewMessage = String(snapshot.previewMessage || "");
    importPrecheck = snapshot.precheck || null;
    importPrecheckSource = String(snapshot.precheckSource || "mock");
    importPrecheckMessage = String(
      snapshot.precheckMessage || "Run precheck after selecting files."
    );
    importSelection = selFromSerialized(snapshot.folderSelection);
    importContextToken = String(snapshot.contextToken || "");
    importGlobalDesignerId = String(snapshot.globalDesignerId || "");
    importGlobalSourceId = String(snapshot.globalSourceId || "");
    importPerFolderAssignmentByPath =
      snapshot.perFolderAssignmentByPath && typeof snapshot.perFolderAssignmentByPath === "object"
        ? { ...snapshot.perFolderAssignmentByPath }
        : {};
    importActionMessage = String(snapshot.actionMessage || "");
    importActionSource = String(snapshot.actionSource || "mock");
    importActionNeedsSkipHoopsConfirm = Boolean(snapshot.actionNeedsSkipHoopsConfirm);
  }

  let importSessionRestored = $state(false);

  // Restore the snapshot the first time this instance mounts, then mirror every
  // subsequent state change back into the store so it survives unmount.
  $effect(() => {
    if (!importSessionRestored) {
      importSessionRestored = true;
      applyImportSessionSnapshot(get(importSessionStore));
    }
    importSessionStore.setSession(buildImportSessionSnapshot());
  });

  async function loadSettingsFromBackend() {
    if (settingsLoading || settingsLoaded) return;
    settingsLoading = true;
    try {
      const result = await getSettingsViewModel();
      const model = result.model;
      settingsImportLastBrowseFolder = String(model?.import_last_browse_folder || "").trim();
      settingsLoaded = true;
    } catch (e) {
      console.error("Could not load settings in import view", e);
    } finally {
      settingsLoading = false;
    }
  }

  async function loadImportReferenceData(force = false) {
    if (importReferenceLoading && !force) return;
    if (!force && importDesigners.length > 0 && importSources.length > 0) return;

    importReferenceLoading = true;
    try {
      const [designerResult, sourceResult] = await Promise.all([listDesigners(), listSources()]);
      importDesigners = Array.isArray(designerResult?.items) ? designerResult.items : [];
      importSources = Array.isArray(sourceResult?.items) ? sourceResult.items : [];
    } catch (error) {
      console.info("Could not load import reference data", error);
      importDesigners = [];
      importSources = [];
    } finally {
      importReferenceLoading = false;
    }
  }

  /** @param {string} folderPath */
  function getImportFolderDesigner(folderPath) {
    return String(importPerFolderAssignmentByPath?.[folderPath]?.designerId || "");
  }

  /** @param {string} folderPath */
  function getImportFolderSource(folderPath) {
    return String(importPerFolderAssignmentByPath?.[folderPath]?.sourceId || "");
  }

  /** @param {any} value */
  function normalizeNameForImportMatching(value) {
    return String(value || "")
      .toLowerCase()
      .replace(/[_\-/\\]+/g, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

  /** @param {any} value */
  function compactNameForImportMatching(value) {
    return String(value || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "");
  }

  /** @param {any} value */
  function stripWebAffixesForImportMatching(value) {
    let compact = compactNameForImportMatching(value);
    if (compact.startsWith("www")) {
      compact = compact.slice(3);
    }
    for (const suffix of ["comau", "couk", "com", "net", "org", "co", "uk"]) {
      if (compact.length > suffix.length + 2 && compact.endsWith(suffix)) {
        compact = compact.slice(0, -suffix.length);
        break;
      }
    }
    return compact;
  }

  /** @param {any} pathValue */
  function normalizeImportPathKey(pathValue) {
    return String(pathValue || "")
      .trim()
      .replace(/\\/g, "/")
      .toLowerCase();
  }

  let importPreviewResolvedAssignmentByPath = $derived(
    (() => {
      const assignments = Array.isArray(importPreview?.resolved_assignments)
        ? importPreview.resolved_assignments
        : [];
      const byPath = new Map();
      for (const assignment of assignments) {
        const folderPath = normalizeImportPathKey(assignment?.folder_path);
        if (!folderPath) continue;
        byPath.set(folderPath, assignment);
      }
      return byPath;
    })()
  );

  /** @param {any} pathValue @param {any[]} items */
  function suggestImportMatchFromPath(pathValue, items) {
    const normalizedPath = normalizeNameForImportMatching(pathValue);
    const compactPath = compactNameForImportMatching(pathValue);
    if ((!normalizedPath && !compactPath) || !Array.isArray(items) || items.length === 0) {
      return null;
    }
    const ignoredNames = new Set(["don't know", "me"]);
    const sorted = [...items]
      .filter((item) => item && typeof item === "object")
      .sort((left, right) => String(right?.name || "").length - String(left?.name || "").length);

    for (const item of sorted) {
      const rawName = String(item?.name || "").trim();
      if (!rawName) continue;
      if (ignoredNames.has(rawName.toLowerCase())) continue;

      const normalizedName = normalizeNameForImportMatching(rawName);
      const compactName = compactNameForImportMatching(rawName);
      const strippedCompactName = stripWebAffixesForImportMatching(rawName);
      if (
        (normalizedName && normalizedPath.includes(normalizedName)) ||
        (compactName && compactPath.includes(compactName)) ||
        (strippedCompactName && compactPath.includes(strippedCompactName))
      ) {
        return item;
      }
    }
    return null;
  }

  /** @param {string} folderPath */
  function getInferredImportDesigner(folderPath) {
    const resolved = importPreviewResolvedAssignmentByPath.get(normalizeImportPathKey(folderPath));
    const resolvedId = Number(resolved?.inferred_designer_id);
    if (Number.isFinite(resolvedId) && resolvedId > 0) {
      const matched = importDesigners.find((designer) => Number(designer?.id) === resolvedId);
      if (matched) return matched;
    }
    return suggestImportMatchFromPath(folderPath, importDesigners);
  }

  /** @param {string} folderPath */
  function getInferredImportSource(folderPath) {
    const resolved = importPreviewResolvedAssignmentByPath.get(normalizeImportPathKey(folderPath));
    const resolvedId = Number(resolved?.inferred_source_id);
    if (Number.isFinite(resolvedId) && resolvedId > 0) {
      const matched = importSources.find((source) => Number(source?.id) === resolvedId);
      if (matched) return matched;
    }
    return suggestImportMatchFromPath(folderPath, importSources);
  }

  /** @param {string} folderPath */
  function getImportFolderDesignerInferredLabel(folderPath) {
    const inferred = getInferredImportDesigner(folderPath);
    return inferred?.name ? `Keep inferred (${inferred.name})` : "Keep inferred";
  }

  /** @param {string} folderPath */
  function getImportFolderSourceInferredLabel(folderPath) {
    const inferred = getInferredImportSource(folderPath);
    return inferred?.name ? `Keep inferred (${inferred.name})` : "Keep inferred";
  }

  /** @param {string} fullPath */
  function getFolderPathFromFilePath(fullPath) {
    const value = String(fullPath || "").trim();
    if (!value) return "";
    const normalized = value.replace(/\\/g, "/");
    const splitIndex = normalized.lastIndexOf("/");
    if (splitIndex <= 0) return "";
    return normalized.slice(0, splitIndex);
  }

  /** @param {string} folderPath */
  function getFolderLabelFromFolderPath(folderPath) {
    const value = String(folderPath || "").trim();
    if (!value) return "Unknown folder";
    const normalized = value.replace(/\\/g, "/").replace(/\/+$/g, "");
    if (!normalized) return "Unknown folder";
    const segments = normalized.split("/").filter(Boolean);
    return segments.length > 0 ? segments[segments.length - 1] : normalized;
  }

  /** @param {string} fullPath */
  function getImportFilenameFromPath(fullPath) {
    const value = String(fullPath || "").trim();
    if (!value) return "Unknown file";
    const normalized = value.replace(/\\/g, "/");
    const segments = normalized.split("/").filter(Boolean);
    return segments.length > 0 ? segments[segments.length - 1] : normalized;
  }

  /** @param {string} folderPath @param {string} fullPath @param {boolean} checked */
  function toggleImportFile(folderPath, fullPath, checked) {
    const key = String(folderPath || "");
    const value = String(fullPath || "").trim();
    if (!key || !value) return;
    importSelection = selToggleFile(importSelection, key, value, Boolean(checked));
  }

  /** @param {string} folderPath */
  function toggleFolderExpanded(folderPath) {
    const key = String(folderPath || "");
    if (!key) return;
    const wasOpen = Boolean(importExpandedByPath[key]);
    importExpandedByPath = { ...importExpandedByPath, [key]: !wasOpen };
  }

  /** @param {string} folderPath */
  function selectAllInFolder(folderPath) {
    const key = String(folderPath || "");
    if (!key) return;
    importSelection = selSelectAllInFolder(importSelection, key);
  }

  /** @param {string} folderPath */
  function deselectAllInFolder(folderPath) {
    const key = String(folderPath || "");
    if (!key) return;
    importSelection = selDeselectAllInFolder(importSelection, key);
  }

  function selectAllImportFiles() {
    const paths = importFolderCatalog.map((folder) => folder.folderPath);
    importSelection = selSelectAllFolders(importSelection, paths);
  }

  function deselectAllImportFiles() {
    const paths = importFolderCatalog.map((folder) => folder.folderPath);
    importSelection = selDeselectAllFolders(importSelection, paths);
  }

  /** @param {string} folderPath @param {string} searchValue */
  function setFolderSearch(folderPath, searchValue) {
    const key = String(folderPath || "");
    if (!key) return;
    importFolderSearchByPath = { ...importFolderSearchByPath, [key]: String(searchValue || "") };
    // Reset to the first page whenever the filter changes.
    importFolderPageByPath = { ...importFolderPageByPath, [key]: 1 };
  }

  /** @param {string} folderPath @param {number} page */
  function setFolderPage(folderPath, page) {
    const key = String(folderPath || "");
    if (!key) return;
    const numeric = Number(page);
    importFolderPageByPath = {
      ...importFolderPageByPath,
      [key]: Number.isFinite(numeric) && numeric > 0 ? Math.floor(numeric) : 1,
    };
  }

  /** @param {string} folderPath */
  function makeFolderPageHandler(folderPath) {
    return (/** @type {number} */ page) => setFolderPage(folderPath, page);
  }

  /** Per-folder review summary: cheap counts only, no per-file objects. */
  /** @type {Array<Record<string, any>>} */
  let importFolderSummary = $derived(
    importFolderCatalog.map((record) => {
      const total = record.filePaths.length;
      const selectedCount = selCount(importSelection, record.folderPath, total);
      let kind = "partial";
      if (selectedCount <= 0) kind = "none";
      else if (selectedCount >= total) kind = "all";
      return {
        folderPath: record.folderPath,
        label: record.label,
        filePaths: record.filePaths,
        total,
        selectedCount,
        kind,
      };
    })
  );

  /** Per-folder paged file window, keyed by folder path. Depends only on the
   *  catalog and the transient search/page state — never on selection, so a
   *  checkbox toggle does not rebuild these slices. */
  /** @type {Record<string, any>} */
  let importFolderFileWindowByPath = $derived(
    (() => {
      /** @type {Record<string, any>} */
      const windows = {};
      for (const record of importFolderCatalog) {
        const folderPath = String(record.folderPath || "");
        const searchText = String(importFolderSearchByPath?.[folderPath] || "")
          .trim()
          .toLowerCase();
        const allPaths = Array.isArray(record.filePaths) ? record.filePaths : [];
        const filteredPaths = searchText
          ? allPaths.filter((/** @type {string} */ fullPath) =>
              getImportFilenameFromPath(fullPath).toLowerCase().includes(searchText)
            )
          : allPaths;
        const pageCount = Math.max(1, Math.ceil(filteredPaths.length / IMPORT_FOLDER_PAGE_SIZE));
        const rawPage = Number(importFolderPageByPath?.[folderPath] || 1);
        const page = Math.min(
          Math.max(Number.isFinite(rawPage) && rawPage > 0 ? rawPage : 1, 1),
          pageCount
        );
        const start = (page - 1) * IMPORT_FOLDER_PAGE_SIZE;
        windows[folderPath] = {
          page,
          pageCount,
          filteredCount: filteredPaths.length,
          visible: filteredPaths.slice(start, start + IMPORT_FOLDER_PAGE_SIZE),
        };
      }
      return windows;
    })()
  );

  let importTotalFileCount = $derived(
    importFolderSummary.reduce((sum, folder) => sum + folder.total, 0)
  );

  let importSelectedCount = $derived(
    importFolderSummary.reduce((sum, folder) => sum + folder.selectedCount, 0)
  );

  let importCanSelectAll = $derived(importSelectedCount < importTotalFileCount);
  let importCanDeselectAll = $derived(importSelectedCount > 0);

  /**
   * True while a scan/precheck/import action is running. Selection controls
   * (Select all / Deselect all, folder-level select/deselect) must be disabled
   * as soon as "Continue" (or the scan/import buttons) is pressed so the review
   * selection can't be changed mid-flight.
   */
  let importSelectionLocked = $derived(importLoading || importActionLoading || busyActive);

  /** Select-all is disabled during a run or when every file is already selected. */
  let importSelectAllDisabled = $derived(importSelectionLocked || !importCanSelectAll);
  /** Deselect-all is disabled during a run or when no file is selected. */
  let importDeselectAllDisabled = $derived(importSelectionLocked || !importCanDeselectAll);

  function buildImportPrecheckRequest() {
    const selectedFolders = importFolderSummary.filter((folder) => folder.selectedCount > 0);
    const perFolderAssignments = selectedFolders.map((folder) => {
      const folderPath = folder.folderPath;
      const explicitDesignerId = getImportFolderDesigner(folderPath);
      const explicitSourceId = getImportFolderSource(folderPath);
      const inferredDesigner = getInferredImportDesigner(folderPath);
      const inferredSource = getInferredImportSource(folderPath);

      return {
        folder_path: folderPath,
        designer_id: explicitDesignerId ? Number(explicitDesignerId) : null,
        source_id: explicitSourceId ? Number(explicitSourceId) : null,
        inferred_designer_id: inferredDesigner?.id ? Number(inferredDesigner.id) : null,
        inferred_source_id: inferredSource?.id ? Number(inferredSource.id) : null,
      };
    });

    // Serialise the folder-scoped selection (base + exceptions) compactly. The
    // full selected-file list is NOT sent; the backend reconstructs it from the
    // stored scan catalogue referenced by `scan_token`.
    const selectionFolderLists = (
      /** @type {Record<string, unknown> | null | undefined} */ map
    ) =>
      Object.entries(map || {}).map(([folderPath, files]) => ({
        folder_path: folderPath,
        files: Array.isArray(files) ? files.map(String) : [],
      }));

    return {
      scan_token: String(importPreview?.scan_token || ""),
      global_designer_id: importGlobalDesignerId ? Number(importGlobalDesignerId) : null,
      global_source_id: importGlobalSourceId ? Number(importGlobalSourceId) : null,
      per_folder_assignments: perFolderAssignments,
      selection: {
        deselected: selectionFolderLists(importSelection?.deselected),
        selected_only: selectionFolderLists(importSelection?.selectedOnly),
      },
      create_on_import: true,
    };
  }

  async function runImportPrecheck() {
    if (importSelectedCount === 0) {
      addToast("Select at least one file before continuing.", "error");
      return;
    }

    importLoading = true;
    importActionMessage = "";
    importActionNeedsSkipHoopsConfirm = false;
    beginBusy("Checking import selections");

    try {
      const result = await precheckImportWire(buildImportPrecheckRequest());
      importPrecheck = result.precheck || null;
      importPrecheckSource = result.source || "mock";
      importPrecheckMessage = result.message || "Precheck complete.";
      importContextToken = String(importPrecheck?.context_token || "");
      navigateTo(importPrecheck ? "#/import/step3" : "#/import/step2");
    } catch (error) {
      const message = String(error || "");
      const scanStale = /expired import scan|scan token/i.test(message);
      if (scanStale) {
        addToast("Your import scan expired. Rescanning your folders...", "info");
        await runImportPreview();
      } else {
        addToast(`Import precheck failed: ${error}`, "error");
        importPrecheck = null;
        importContextToken = "";
        navigateTo("#/import/step2");
      }
    } finally {
      importLoading = false;
      endBusy();
    }
  }

  /** @param {any} nextRoute */
  function mapServerImportRouteToHash(nextRoute) {
    const route = String(nextRoute || "").toLowerCase();
    if (route.startsWith("/designs")) return "#/designs";
    if (route.startsWith("/import")) {
      if (route.includes("step3") || route.includes("precheck") || route.includes("confirm"))
        return "#/import/step3";
      if (route.includes("step2") || route.includes("review") || route.includes("scan"))
        return "#/import/step2";
      if (route.includes("step1") || route.includes("folder")) return "#/import/step1";
      return "#/import/step1";
    }
    return null;
  }

  /**
   * Detect whether an import_now failure was caused by the backend's bulk-import
   * context token expiring (the backend stores tokens for a 15-minute TTL).  The
   * adapter degrades such failures to a mock result with no next_route and a
   * message describing the expired/unknown token.
   * @param {any} actionResult
   * @param {string} message
   */
  function isExpiredImportContextFailure(actionResult, message) {
    if (actionResult?.next_route) return false;
    const text = String(actionResult?.message || message || "");
    return /expired/i.test(text) && /context token/i.test(text);
  }

  /**
   * Recover from an expired bulk-import context token.  The user may have
   * lingered on a top-level page (Admin Settings, AI Tagging Guide, About,
   * Licence) longer than the backend's token TTL.  Re-run the precheck from the
   * preserved selections to mint a fresh token, then retry the original action.
   * @param {string} action @param {boolean} confirmSkipHoops
   */
  async function recoverExpiredImportContext(action, confirmSkipHoops = false) {
    if (importRecoveryInProgress) return;
    importRecoveryInProgress = true;
    try {
      addToast("Import context expired. Re-checking your selections before retrying...", "info");
      await runImportPrecheck();
      if (!importContextToken) {
        addToast("Could not refresh the import context. Please review and retry.", "error");
        return;
      }
      await executeImportPrecheckAction(action, confirmSkipHoops);
    } finally {
      importRecoveryInProgress = false;
    }
  }

  /** @param {string} action @param {boolean} [confirmSkipHoops] */
  async function executeImportPrecheckAction(action, confirmSkipHoops = false) {
    if (!importContextToken) {
      addToast("Missing import context token. Run precheck again.", "error");
      return;
    }

    importActionLoading = true;
    importActionInProgress = String(action || "");

    const importNowAction = action === "import_now";
    if (importNowAction) {
      importStopRequestPending = false;
      importProgressStatus = "";
      beginBusy("Importing designs");
      await startImportProgressUpdates(importContextToken);
    }

    try {
      const result = await runPrecheckAction({
        contextToken: importContextToken,
        action,
        confirmSkipHoops,
      });

      const actionResult = result.actionResult || null;
      /** @type {string} */
      const actionSource = result.source;
      /** @type {string} */
      const actionMessage = result.message;
      importActionSource = actionSource || "mock";
      importActionMessage = actionMessage || "";
      importActionNeedsSkipHoopsConfirm = Boolean(actionResult?.requires_skip_hoops_confirmation);

      if (actionResult?.consumed_context) {
        importContextToken = "";
      }

      if (action === "cancel") {
        resetImportWizard();
        return;
      }

      const hashRoute = mapServerImportRouteToHash(actionResult?.next_route);
      if (hashRoute) {
        if (hashRoute === "#/designs") {
          const persistedCount = Number(actionResult?.confirm_result?.persisted_design_count ?? 0);
          const failedCount = Number(actionResult?.confirm_result?.failed_decode_count ?? 0);
          if (persistedCount >= 1 && typeof onImportCompleted === "function") {
            onImportCompleted(persistedCount);
          }
          if (failedCount >= 1) {
            addToast(
              `${failedCount} ${failedCount === 1 ? "file could" : "files could"} not be read during import (no preview was generated). Regenerate the image under Admin → Tagging Actions, or from each design's page.`,
              "warning",
              true
            );
          }
          resetImportWizard();
        }
        navigateTo(hashRoute);
      } else if (action === "import_now") {
        if (isExpiredImportContextFailure(actionResult, actionMessage)) {
          await recoverExpiredImportContext(action, confirmSkipHoops);
        } else {
          addToast(
            actionMessage || "Import failed. Check the console for details and try again.",
            "error"
          );
        }
      }
    } catch (error) {
      addToast(`Import action failed: ${error}`, "error");
    } finally {
      if (importNowAction) {
        await stopImportProgressUpdates();
        endBusy();
      }
      importStopRequestPending = false;
      importActionInProgress = "";
      importActionLoading = false;
    }
  }

  async function requestImportStop() {
    if (!importNowInProgress || importStopRequestPending) return;

    importStopRequestPending = true;

    try {
      const result = await requestStopBulkImport();
      importActionSource = result.source || "mock";
      importActionMessage = result.message || "Stop requested.";
    } catch (error) {
      addToast(`Stop request failed: ${error}`, "error");
      importStopRequestPending = false;
    }
  }

  /** @param {string} folderPath @param {string} designerId */
  function setImportFolderDesigner(folderPath, designerId) {
    const key = String(folderPath || "").trim();
    if (!key) return;
    importPerFolderAssignmentByPath = {
      ...importPerFolderAssignmentByPath,
      [key]: {
        ...(importPerFolderAssignmentByPath?.[key] || { designerId: "", sourceId: "" }),
        designerId: String(designerId || ""),
      },
    };
  }

  /** @param {string} folderPath @param {string} sourceId */
  function setImportFolderSource(folderPath, sourceId) {
    const key = String(folderPath || "").trim();
    if (!key) return;
    importPerFolderAssignmentByPath = {
      ...importPerFolderAssignmentByPath,
      [key]: {
        ...(importPerFolderAssignmentByPath?.[key] || { designerId: "", sourceId: "" }),
        sourceId: String(sourceId || ""),
      },
    };
  }

  async function runImportPreview() {
    importLoading = true;
    importActionMessage = "";
    importActionNeedsSkipHoopsConfirm = false;
    beginBusy("Scanning import folders");

    try {
      const result = await previewImportFromRoots(getActiveImportRoots());
      importPreview = result.preview || null;
      importPreviewSource = result.source || "mock";
      importPreviewMessage = deriveImportPreviewMessage(result?.preview);
      importSelection = selCreate();
      importPerFolderAssignmentByPath = {};
      importExpandedByPath = {};
      importFolderSearchByPath = {};
      importFolderPageByPath = {};
      importPrecheck = null;
      importPrecheckSource = "mock";
      importPrecheckMessage = "Run precheck after selecting files.";
      importContextToken = "";
      navigateTo("#/import/step2");
    } catch (error) {
      addToast(`Import preview failed: ${error}`, "error");
      importPreview = null;
      importPreviewSource = "mock";
      importPreviewMessage = `Import preview failed: ${error}`;
      importSelection = selCreate();
      importExpandedByPath = {};
      importFolderSearchByPath = {};
      importFolderPageByPath = {};
      importPerFolderAssignmentByPath = {};
      importPrecheck = null;
      importContextToken = "";
      navigateTo("#/import/step1");
    } finally {
      importLoading = false;
      endBusy();
    }
  }

  /**
   * Build a user-facing explanation for an empty preview based on the
   * diagnostics returned by the Rust backend (or mock fallback).
   * @param {Record<string, any> | null | undefined} preview
   */
  function deriveImportPreviewMessage(preview) {
    if (!preview) return "";
    if (preview?.invalid_root) {
      return "Enter at least one folder path to preview import.";
    }
    if (preview?.missing_root) {
      return "The selected folder(s) could not be found on disk. Check that the path is correct and the drive is available.";
    }
    if (preview?.no_supported_files) {
      return "No supported embroidery files (JEF, PES, HUS, DST, EXP, VP3) were found in the selected folder(s).";
    }
    return "";
  }

  /** @param {number|null} index @param {string} path */
  function setImportRootPathAt(index, path) {
    const next = normalizeImportRootPath(path);
    if (!next) return;

    if (index === null || index === undefined || index < 0) {
      importRootPath = next;
      return;
    }
    importRootPaths = importRootPaths.map((value, rowIndex) => (rowIndex === index ? next : value));
  }

  async function browseImportRootPath(targetIndex = -1) {
    if (importBrowseLoading || importLoading || importActionLoading) return;

    importBrowseLoading = true;

    try {
      const currentValue =
        targetIndex === null || targetIndex === undefined || targetIndex < 0
          ? importRootPath
          : importRootPaths[targetIndex] || "";
      const currentHint = currentValue ? parentFolder(currentValue) : "";
      const persistedHint = parentFolder(settingsImportLastBrowseFolder) || "";
      const startHint = currentHint || persistedHint;
      const result = await browseImportFolder(startHint);
      const selectedPaths = Array.isArray(result?.paths)
        ? result.paths.map((value) => String(value || "").trim()).filter(Boolean)
        : [];

      if (selectedPaths.length > 0) {
        const [firstSelectedPath, ...additionalSelectedPaths] = selectedPaths;
        setImportRootPathAt(targetIndex, firstSelectedPath);
        for (const path of additionalSelectedPaths) {
          addImportRootPath(path);
        }
        await persistImportLastBrowseFolder(firstSelectedPath);
      } else {
        const selectedPath = String(result?.path || "").trim();
        if (selectedPath) {
          setImportRootPathAt(targetIndex, selectedPath);
          await persistImportLastBrowseFolder(selectedPath);
        }
      }
    } catch (error) {
      addToast(`Folder browse failed: ${error}`, "error");
    } finally {
      importBrowseLoading = false;
    }
  }

  /** @param {string} path */
  async function persistImportLastBrowseFolder(path) {
    const normalized = normalizeImportRootPath(path);
    if (!normalized) return;
    try {
      await saveImportLastBrowseFolder(normalized);
    } catch (e) {
      console.info("Could not persist last browse folder", e);
    }
  }

  /** @param {string} path */
  function parentFolder(path) {
    const p = String(path || "")
      .trim()
      .replace(/[/\\]+$/, "");
    if (!p) return "";
    const lastSep = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
    if (lastSep <= 0) return p;
    return p.slice(0, lastSep);
  }

  /** @param {string} value */
  function normalizeImportRootPath(value) {
    const trimmed = String(value || "").trim();
    if (!trimmed) return "";
    const slashNormalized = trimmed.replace(/\\/g, "/");
    const isUncPath = slashNormalized.startsWith("//");
    const compacted = isUncPath
      ? `//${slashNormalized.slice(2).replace(/\/{2,}/g, "/")}`
      : slashNormalized.replace(/\/{2,}/g, "/");
    const withoutTrailingSlash = compacted.replace(/\/+$/g, "");
    if (!withoutTrailingSlash) return compacted;
    if (/^[a-zA-Z]:$/.test(withoutTrailingSlash)) {
      return `${withoutTrailingSlash}/`;
    }
    return withoutTrailingSlash;
  }

  function getActiveImportRoots() {
    const candidateRoots = [importRootPath, ...importRootPaths]
      .map((value) => normalizeImportRootPath(value))
      .filter(Boolean);

    const uniqueRoots = [];
    const seenRoots = new Set();
    for (const root of candidateRoots) {
      const key = root.toLowerCase();
      if (seenRoots.has(key)) continue;
      seenRoots.add(key);
      uniqueRoots.push(root);
    }
    return uniqueRoots;
  }

  let importHasActiveRoots = $derived(getActiveImportRoots().length > 0);

  /** @param {string} [path] */
  function addImportRootPath(path = importRootPath) {
    const next = normalizeImportRootPath(path);
    if (!next) return;
    const existingByLower = new Set(
      importRootPaths.map((item) => String(item || "").toLowerCase())
    );
    if (!existingByLower.has(next.toLowerCase())) {
      importRootPaths = [...importRootPaths, next];
    }
  }

  /** @param {string} path */
  function removeImportRootPath(path) {
    const target = normalizeImportRootPath(path).toLowerCase();
    importRootPaths = importRootPaths.filter(
      (value) => String(value || "").toLowerCase() !== target
    );
  }

  function removePrimaryImportRootPath() {
    if (importRootPaths.length > 0) {
      importRootPath = importRootPaths[0];
      importRootPaths = importRootPaths.slice(1);
    } else {
      importRootPath = "";
    }
  }

  function addCurrentImportRootPath() {
    importRootPaths = [...importRootPaths, ""];
  }

  function resetImportWizard() {
    stopImportProgressUpdates();
    importProgressStatus = "";
    importProgressToken = "";
    importBrowseLoading = false;
    importRootPath = "";
    importRootPaths = [];
    importPreview = null;
    importPreviewSource = "mock";
    importPreviewMessage = "";
    importPrecheck = null;
    importPrecheckSource = "mock";
    importPrecheckMessage = "Run precheck after selecting files.";
    importSelection = selCreate();
    importExpandedByPath = {};
    importFolderSearchByPath = {};
    importFolderPageByPath = {};
    importContextToken = "";
    importActionMessage = "";
    importActionSource = "mock";
    importActionNeedsSkipHoopsConfirm = false;
    importStopRequestPending = false;
    importGlobalDesignerId = "";
    importGlobalSourceId = "";
    importPerFolderAssignmentByPath = {};

    importSessionStore.clear();

    navigateTo("#/import/step1");
  }

  async function stopImportProgressUpdates() {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }
    importProgressStatus = "";
    importProgressToken = "";
  }

  /** @param {string} contextToken */
  async function startImportProgressUpdates(contextToken) {
    const normalizedToken = String(contextToken || "").trim();
    if (!normalizedToken) return;

    await stopImportProgressUpdates();
    importProgressToken = normalizedToken;

    try {
      const { listen } = await import("@tauri-apps/api/event");
      importProgressUnlisten = await listen("bulk-import-progress", (event) => {
        const payload = event?.payload || {};
        const payloadToken = String(payload?.context_token ?? payload?.contextToken ?? "").trim();
        if (payloadToken && payloadToken !== importProgressToken) return;

        const stage = String(payload?.stage || "");
        const processed = Number(payload?.processed_count ?? payload?.processedCount ?? 0);
        const total = Number(payload?.total_count ?? payload?.totalCount ?? 0);
        const persisted = Number(payload?.persisted_count ?? payload?.persistedCount ?? 0);
        const committed = Number(payload?.committed_count ?? payload?.committedCount ?? persisted);
        const failed = Number(payload?.failed_count ?? payload?.failedCount ?? 0);
        const currentFile = String(payload?.current_file ?? payload?.currentFile ?? "").trim();
        const currentFilename = currentFile.replace(/\\/g, "/").split("/").pop() || currentFile;

        if (stage === "started") {
          importProgressStatus =
            total > 0
              ? `Starting import for ${total} file${total === 1 ? "" : "s"}...`
              : "Starting import...";
          return;
        }
        if (stage === "generating_images") {
          importProgressStatus =
            total > 0
              ? `${processed}/${total} processed (${committed} imported) - generating preview images...`
              : "Generating preview images...";
          return;
        }
        if ((stage === "processing_file" || stage === "processingFile") && total > 0) {
          importProgressStatus = `Processing ${Math.min(processed + 1, total)}/${total}: ${currentFilename}`;
          return;
        }
        if (stage === "batch_committed") {
          importProgressStatus =
            total > 0
              ? `${processed}/${total} processed (${committed} imported) - saving batch...`
              : `${committed} imported - saving batch...`;
          return;
        }
        if (stage === "stopped") {
          importProgressStatus =
            total > 0
              ? `Stopped after ${processed}/${total} processed (${committed} imported)`
              : `Stopped after ${committed} imported`;
          return;
        }
        if (stage === "completed") {
          const failedSuffix = failed > 0 ? `, ${failed} failed` : "";
          importProgressStatus =
            total > 0
              ? `Completed ${processed}/${total} processed (${committed} imported${failedSuffix})`
              : `Completed ${committed} imported${failedSuffix}`;
          return;
        }
        if (total > 0) {
          importProgressStatus = `${processed}/${total} processed (${committed} imported${
            failed > 0 ? `, ${failed} failed` : ""
          })`;
        } else {
          importProgressStatus = `${committed} imported${failed > 0 ? `, ${failed} failed` : ""}`;
        }
      });
    } catch (error) {
      console.info("Bulk import progress events unavailable.", error);
    }
  }

  // Reactive effect for loading steps
  $effect(() => {
    if (importRouteStep !== null) {
      untrack(() => {
        loadImportReferenceData();
        loadSettingsFromBackend();
      });
    }
  });

  onMount(() => {
    if (importRouteStep !== null) {
      loadImportReferenceData();
      loadSettingsFromBackend();
    }
  });

  onDestroy(() => {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }
  });
</script>

<section class="import-page space-y-4 font-sans">
  <h1 class="ui-page-title import-title text-2xl font-bold text-gray-800">Bulk Import</h1>

  {#if importRouteStep === 1}
    <p class="ui-help-note import-step1-intro text-sm text-gray-500">
      <br />Select one or more folders containing embroidery files. Sub-folders are included
      automatically.
      <br />Your original files are never altered or moved. Files outside your main design directory
      are safely copied into the catalogue.
      <a href="#/help?section=importing" class="text-indigo-600 hover:underline ml-1">Import help</a
      >
    </p>

    <div class="import-step1-card bg-white rounded shadow p-6 w-full space-y-4">
      <form
        id="importScanForm"
        class="space-y-4"
        onsubmit={(event) => {
          event.preventDefault();
          runImportPreview();
        }}
      >
        <div>
          <label for="import-root-path" class="ui-field-label text-sm font-semibold text-gray-700">
            <span class="block mb-1">Source Folder(s) *</span>
          </label>
          <div class="space-y-2.5">
            <div class="folder-row import-folder-row flex items-center gap-2">
              <input
                id="import-root-path"
                class="ui-text-input ui-control-text-inset import-folder-input flex-1 font-mono border rounded px-3 py-2 text-sm"
                bind:value={importRootPath}
                placeholder="Enter path to your embroidery designs folder…"
                disabled={importLoading || importActionLoading || importBrowseLoading}
                aria-label="Source folder path 1"
              />
              <button
                type="button"
                class="ui-action-button menu-button-secondary py-2"
                onclick={() => browseImportRootPath(-1)}
                disabled={importLoading || importActionLoading || importBrowseLoading || busyActive}
              >
                {importBrowseLoading ? "Browsing…" : "Browse…"}
              </button>
              <button
                type="button"
                class="ui-action-button menu-button-secondary py-2"
                onclick={removePrimaryImportRootPath}
                disabled={importLoading ||
                  importActionLoading ||
                  importBrowseLoading ||
                  busyActive ||
                  !String(importRootPath || "").trim()}
                title="Remove this folder"
              >
                Remove
              </button>
            </div>

            {#each importRootPaths as rootPath, rowIndex}
              <div
                class="folder-row import-folder-row flex items-center gap-2"
                data-index={rowIndex + 1}
              >
                <input
                  type="text"
                  class="ui-text-input ui-control-text-inset import-folder-input flex-1 font-mono border rounded px-3 py-2 text-sm bg-gray-50"
                  value={rootPath}
                  readonly
                  aria-label={`Source folder path ${rowIndex + 2}`}
                />
                <button
                  type="button"
                  class="ui-action-button menu-button-secondary py-2"
                  onclick={() => browseImportRootPath(rowIndex)}
                  disabled={importLoading || importActionLoading || importBrowseLoading}
                >
                  Browse…
                </button>
                <button
                  type="button"
                  class="ui-action-button menu-button-secondary py-2 text-red-500 border-red-200"
                  onclick={() => removeImportRootPath(rootPath)}
                  disabled={importLoading || importActionLoading || importBrowseLoading}
                  title="Remove this folder"
                >
                  Remove
                </button>
              </div>
            {/each}
          </div>

          <div class="import-step1-add-folder-shell pt-3">
            <button
              type="button"
              class="menu-button-primary ui-action-button ui-action-button-primary import-add-folder-link text-xs"
              onclick={addCurrentImportRootPath}
              disabled={importLoading ||
                importActionLoading ||
                importBrowseLoading ||
                !String(importRootPath || "").trim()}
            >
              Add another folder
            </button>
          </div>
        </div>

        <div class="ui-action-button-group import-step1-primary-actions pt-2 flex gap-2">
          <button
            class="menu-button-primary ui-action-button ui-action-button-primary"
            type="submit"
            disabled={importLoading || importBrowseLoading || !importHasActiveRoots}
          >
            {importLoading ? "Running…" : "Scan folder(s)"}
          </button>
          <button
            type="button"
            class="menu-button-secondary ui-action-button"
            onclick={resetImportWizard}
            disabled={importLoading ||
              importActionLoading ||
              importBrowseLoading ||
              !importHasActiveRoots}
          >
            Reset
          </button>
        </div>
      </form>
    </div>
  {/if}

  {#if importRouteStep === 2}
    {#if importPreview}
      <div class="ui-section-shell import-panel space-y-4">
        <div class="space-y-1">
          <p class="ui-field-label import-field-label font-bold text-gray-800 text-lg">
            Review scanned files
          </p>
          <p class="ui-help-note text-sm text-gray-500">
            {importFolderSummary.length || importPreview.folder_count || 0} folder(s) scanned - {Array.isArray(
              importPreview.scanned_files
            )
              ? importPreview.scanned_files.length
              : 0} file(s) found. Selected files will be <strong>copied into the catalogue</strong>.
            <a href="#/help?section=importing" class="text-indigo-600 hover:underline ml-1"
              >Import help</a
            >
          </p>
        </div>

        <div
          class="ui-section-shell p-4 border rounded bg-gray-50 space-y-3 import-step2-global-shell"
        >
          <p class="ui-field-label import-field-label font-semibold text-gray-800 text-sm">
            Apply to all folders (optional override)
          </p>
          <div class="grid grid-cols-2 gap-3 text-sm import-step2-global-grid">
            <label class="ui-field-label text-sm block">
              <span class="block font-medium mb-1 text-gray-700">Designer</span>
              <select
                class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                bind:value={importGlobalDesignerId}
                disabled={importReferenceLoading || importLoading || importActionLoading}
              >
                <option value="">Keep inferred (per folder)</option>
                {#each importDesigners as designer}
                  <option value={String(designer.id)}>{designer.name}</option>
                {/each}
              </select>
            </label>
            <label class="ui-field-label text-sm block">
              <span class="block font-medium mb-1 text-gray-700">Source</span>
              <select
                class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                bind:value={importGlobalSourceId}
                disabled={importReferenceLoading || importLoading || importActionLoading}
              >
                <option value="">Keep inferred (per folder)</option>
                {#each importSources as source}
                  <option value={String(source.id)}>{source.name}</option>
                {/each}
              </select>
            </label>
          </div>
        </div>

        <div class="space-y-2 import-step2-actions-shell">
          <div
            class="ui-action-button-group import-step1-primary-actions import-step2-primary-actions import-step2-inline-actions flex flex-wrap gap-2 items-center"
          >
            <button
              class="menu-button-primary ui-action-button ui-action-button-primary"
              onclick={runImportPrecheck}
              disabled={importLoading || importActionLoading || busyActive || importSelectedCount === 0}
            >
              {#if importLoading}
                Running…
              {:else if importSelectedCount > 0}
                Continue with {importSelectedCount} design{importSelectedCount === 1
                  ? ""
                  : "s"}
              {:else}
                Continue
              {/if}
            </button>
            <button
              type="button"
              class="menu-button-secondary ui-action-button"
              onclick={() => navigateTo("#/import/step1")}
              disabled={importLoading || importActionLoading || busyActive}
            >
              Cancel
            </button>
            <button
              type="button"
              class={`px-3 py-1.5 rounded border text-xs font-semibold ${importSelectAllDisabled ? "text-gray-400 bg-gray-50 cursor-not-allowed" : "bg-white hover:bg-gray-50 text-indigo-600"}`}
              onclick={selectAllImportFiles}
              disabled={importSelectAllDisabled}
            >
              Select all
            </button>
            <button
              type="button"
              class={`px-3 py-1.5 rounded border text-xs font-semibold ${importDeselectAllDisabled ? "text-gray-400 bg-gray-50 cursor-not-allowed" : "bg-white hover:bg-gray-50 text-indigo-600"}`}
              onclick={deselectAllImportFiles}
              disabled={importDeselectAllDisabled}
            >
              Deselect all
            </button>
          </div>
        </div>

        {#if importFolderSummary.length > 0}
          <div class="space-y-4">
            {#each importFolderSummary as folder (folder.folderPath)}
              {@const autoExpand = folder.total <= IMPORT_FOLDER_AUTO_EXPAND_MAX}
              {@const isExpanded = autoExpand || Boolean(importExpandedByPath[folder.folderPath])}
              {@const fileWindow =
                importFolderFileWindowByPath[folder.folderPath] || {
                  page: 1,
                  pageCount: 1,
                  filteredCount: 0,
                  visible: [],
                }}
              {@const selectionStatus =
                folder.kind === "all"
                  ? `All ${folder.total} selected`
                  : folder.kind === "none"
                    ? "None selected"
                    : `${folder.selectedCount} of ${folder.total} selected`}
              {@const folderSelectAllDisabled = importSelectionLocked || folder.kind === "all"}
              {@const folderDeselectAllDisabled = importSelectionLocked || folder.kind === "none"}
              <div
                class="ui-section-shell overflow-hidden border rounded bg-white import-step2-folder-shell shadow-sm"
                data-testid="import-folder-shell"
              >
                <div
                  class="bg-gray-50 border-b px-4 py-2.5 flex flex-wrap items-center gap-3 import-step2-folder-header"
                >
                  <div class="flex-1 min-w-0">
                    <code class="text-xs text-black font-bold import-step2-folder-label"
                      >{folder.label}</code
                    >
                    <span class="mx-2 text-xs text-gray-400" aria-hidden="true">-</span>
                    <code class="text-xs text-gray-500 break-all">{folder.folderPath}</code>
                  </div>
                  <span class="text-xs font-semibold text-indigo-700 import-step2-folder-count">
                    {selectionStatus}
                  </span>
                  {#if folder.total > IMPORT_FOLDER_AUTO_EXPAND_MAX}
                    <button
                      type="button"
                      class="text-xs px-2 py-1 rounded border font-semibold bg-white hover:bg-gray-50 text-indigo-600"
                      onclick={() => toggleFolderExpanded(folder.folderPath)}
                      disabled={importSelectionLocked}
                    >
                      {isExpanded ? "Hide files" : `Show files (${folder.total})`}
                    </button>
                  {/if}
                  <button
                    type="button"
                    aria-label={`Select all files in ${folder.label}`}
                    class={`px-2 py-1 rounded border text-xs font-semibold ${folderSelectAllDisabled ? "text-gray-400 bg-gray-50 cursor-not-allowed" : "bg-white hover:bg-gray-50 text-indigo-600"}`}
                    onclick={() => selectAllInFolder(folder.folderPath)}
                    disabled={folderSelectAllDisabled}
                  >
                    Select all
                  </button>
                  <button
                    type="button"
                    aria-label={`Deselect all files in ${folder.label}`}
                    class={`px-2 py-1 rounded border text-xs font-semibold ${folderDeselectAllDisabled ? "text-gray-400 bg-gray-50 cursor-not-allowed" : "bg-white hover:bg-gray-50 text-indigo-600"}`}
                    onclick={() => deselectAllInFolder(folder.folderPath)}
                    disabled={folderDeselectAllDisabled}
                  >
                    Deselect all
                  </button>
                </div>

                <div class="px-4 py-3 border-b bg-gray-50/50 import-step2-folder-overrides">
                  <div class="grid grid-cols-2 gap-3 text-sm">
                    <label class="ui-field-label text-sm block">
                      <span class="block font-medium mb-1 text-gray-700"
                        >Designer for this folder</span
                      >
                      <select
                        class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                        value={getImportFolderDesigner(folder.folderPath)}
                        onchange={(event) =>
                          setImportFolderDesigner(folder.folderPath, event.currentTarget.value)}
                        disabled={importReferenceLoading || importLoading || importActionLoading}
                      >
                        <option value=""
                          >{getImportFolderDesignerInferredLabel(folder.folderPath)}</option
                        >
                        {#each importDesigners as designer}
                          <option value={String(designer.id)}>{designer.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label class="ui-field-label text-sm block">
                      <span class="block font-medium mb-1 text-gray-700"
                        >Source for this folder</span
                      >
                      <select
                        class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                        value={getImportFolderSource(folder.folderPath)}
                        onchange={(event) =>
                          setImportFolderSource(folder.folderPath, event.currentTarget.value)}
                        disabled={importReferenceLoading || importLoading || importActionLoading}
                      >
                        <option value=""
                          >{getImportFolderSourceInferredLabel(folder.folderPath)}</option
                        >
                        {#each importSources as source}
                          <option value={String(source.id)}>{source.name}</option>
                        {/each}
                      </select>
                    </label>
                  </div>
                </div>

                {#if isExpanded}
                  <div class="import-step2-file-list-shell p-4">
                    {#if folder.total > IMPORT_FOLDER_AUTO_EXPAND_MAX}
                      <p class="text-xs text-gray-500 mb-2">
                        This folder contains {folder.total} files. Use Select/Deselect all to work
                        on the whole folder, or filter to find specific files.
                      </p>
                    {/if}
                    {#if folder.total > IMPORT_FOLDER_PAGE_SIZE}
                      <div class="mb-2">
                        <input
                          type="search"
                          class="ui-text-input ui-control-text-inset w-full border rounded px-3 py-1.5 text-sm"
                          placeholder="Filter files in this folder…"
                          value={importFolderSearchByPath[folder.folderPath] || ""}
                          oninput={(event) =>
                            setFolderSearch(folder.folderPath, event.currentTarget.value)}
                          aria-label={`Filter files in ${folder.label}`}
                        />
                      </div>
                    {/if}
                    {#if fileWindow.visible.length > 0}
                      <div
                        class="import-step2-file-columns grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2"
                      >
                        {#each fileWindow.visible as fullPath (fullPath)}
                          {@const isChecked = selIsSelected(importSelection, folder.folderPath, fullPath)}
                          <label
                            class="import-step2-file-item flex items-start gap-2 cursor-pointer"
                          >
                            <input
                              type="checkbox"
                              class="ui-checkbox mt-1 accent-indigo-600 rounded"
                              checked={isChecked}
                              onchange={(event) =>
                                toggleImportFile(
                                  folder.folderPath,
                                  fullPath,
                                  event.currentTarget.checked
                                )}
                              disabled={importLoading || importActionLoading}
                            />
                            <span
                              class="ui-field-label text-sm text-gray-700 break-all font-mono"
                              title={fullPath}>{getImportFilenameFromPath(fullPath)}</span
                            >
                          </label>
                        {/each}
                      </div>
                    {:else}
                      <p class="text-sm text-gray-500 italic">No files match your filter.</p>
                    {/if}
                    {#if fileWindow.pageCount > 1}
                      <Pagination
                        currentPage={fileWindow.page}
                        totalPages={fileWindow.pageCount}
                        onPageChange={makeFolderPageHandler(folder.folderPath)}
                        disabled={importLoading || importActionLoading || busyActive}
                        ariaLabel={`Files in ${folder.label}`}
                      />
                    {/if}
                  </div>
                {:else}
                  <div class="import-step2-file-list-shell p-4">
                    <p class="text-sm text-gray-500 italic">
                      Files are hidden for this large folder ({folder.total} file{folder.total ===
                      1 ? "" : "s"}). Use the controls above to select or deselect the whole folder.
                    </p>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <div
            class="border border-amber-300 bg-amber-50 text-amber-950 p-4 rounded text-sm space-y-2"
          >
            <p class="font-semibold text-amber-900">
              No supported files discovered in this preview.
            </p>
            {#if importPreviewMessage}
              <p class="text-amber-900">{importPreviewMessage}</p>
            {/if}
            <button
              type="button"
              class="menu-button-secondary ui-action-button text-xs"
              onclick={() => navigateTo("#/import/step1")}
            >
              Back to Step 1
            </button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="ui-section-shell import-panel space-y-2 border rounded p-4 bg-white text-center">
        <p class="ui-help-note italic text-gray-500">Step 2 needs a completed preview first.</p>
        <div class="pt-2">
          <button
            type="button"
            class="menu-button-secondary ui-action-button"
            onclick={() => navigateTo("#/import/step1")}>Back to Step 1</button
          >
        </div>
      </div>
    {/if}
  {/if}

  {#if importRouteStep === 3}
    {#if importPrecheck}
      <div class="ui-section-shell import-panel space-y-4">
        <p class="ui-field-label import-field-label font-bold text-gray-800 text-lg">
          Before You Import
        </p>

        <div class="border border-blue-300 bg-blue-50 text-blue-900 p-4 rounded space-y-2 text-sm">
          <p class="font-semibold text-blue-900">Note on Visual AI Tagging</p>
          <p class="ui-help-note text-blue-900">
            Initial import uses fast, offline File &amp Folder Rules to index your designs instantly. Once finished, you can run automated Visual AI tagging anytime from Tagging Actions to enrich your collection.
          </p>
        </div>

        <div class="ui-action-button-group flex flex-wrap gap-2 pt-2">
          <button
            class="menu-button-primary ui-action-button ui-action-button-primary"
            onclick={() => executeImportPrecheckAction("import_now")}
            disabled={importActionLoading || busyActive || !importContextToken}
          >
            {#if importActionLoading && importActionInProgress === "import_now"}
              {#if importProgressStatus}
                Running Import... {importProgressStatus}
              {:else}
                Running Import...
              {/if}
            {:else}
              Import Designs
            {/if}
          </button>
          <button
            class="menu-button-secondary ui-action-button"
            onclick={importNowInProgress
              ? requestImportStop
              : () => executeImportPrecheckAction("cancel")}
            disabled={importNowInProgress
              ? importStopRequestPending
              : importActionLoading || !importContextToken}
          >
            {#if importNowInProgress}
              {importStopRequestPending ? "Stopping..." : "Stop"}
            {:else}
              Cancel
            {/if}
          </button>
        </div>

        {#if importActionNeedsSkipHoopsConfirm}
          <div
            class="ui-section-shell import-folder-card border border-amber-300 bg-amber-50 text-amber-950 p-4 rounded space-y-2 text-sm mt-3"
          >
            <p class="ui-help-note text-amber-800 font-semibold">
              Hoops are not configured for a first import. Confirm to continue anyway.
            </p>
            <button
              class="menu-button-primary ui-action-button ui-action-button-primary text-xs"
              onclick={() => executeImportPrecheckAction("import_now", true)}
              disabled={importActionLoading || busyActive || !importContextToken}
            >
              Confirm import without hoop setup
            </button>
          </div>
        {/if}

        {#if importActionMessage}
          <p
            class="ui-help-note text-sm text-indigo-700 bg-indigo-50 border border-indigo-200 rounded p-3 mt-3"
          >
            {importActionMessage}
          </p>
        {/if}
      </div>
    {:else}
      <div class="ui-section-shell import-panel space-y-2 border rounded p-4 bg-white text-center">
        <p class="ui-help-note italic text-gray-500">
          Step 3 needs precheck to be completed first.
        </p>
        <div class="pt-2">
          <button
            type="button"
            class="menu-button-secondary ui-action-button"
            onclick={() => navigateTo(importPreview ? "#/import/step2" : "#/import/step1")}
            >Go to previous step</button
          >
        </div>
      </div>
    {/if}
  {/if}
</section>
