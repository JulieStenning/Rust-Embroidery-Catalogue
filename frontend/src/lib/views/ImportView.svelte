<script>
  import { onMount, onDestroy, untrack } from "svelte";
  import {
    listDesigners,
    listSources,
    previewImportFromRoots,
    precheckImportWire,
    runPrecheckAction,
    requestStopBulkImport,
    browseImportFolder,
    saveImportLastBrowseFolder,
    getSettingsViewModel
  } from "../api/commandAdapter.js";

  let { currentRoute, navigateTo, onImportCompleted } = $props();

  let settingsImagePreference = $state("2d");
  let settingsHelpUrl = $state("#/help");
  let settingsHasGoogleApiKey = $state(false);
  let settingsLoaded = $state(false);
  let settingsLoading = $state(false);

  let importRootPath = $state("");
  /** @type {string[]} */
  let importRootPaths = $state([]);
  let importPreview = $state(/** @type {Record<string, any> | null} */ (null));
  let importPreviewSource = $state("mock");
  let importPrecheck = $state(/** @type {Record<string, any> | null} */ (null));
  let importPrecheckSource = $state("mock");
  let importPrecheckMessage = $state("Run precheck after selecting files.");
  let importStep3ImagePreference = $state("2d");
  /** @type {string[]} */
  let importSelectedFiles = $state([]);
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
  let importError = $state("");

  let importNowInProgress = $derived(importActionLoading && importActionInProgress === "import_now");
  let importRouteStep = $derived(parseImportWizardStep(currentRoute));

  /** @param {string} route */
  function parseImportWizardStep(route) {
    if (route === "#/import") return 1;
    const match = route.match(/^#\/import\/step([123])$/);
    return match ? Number(match[1]) : null;
  }

  async function loadSettingsFromBackend() {
    if (settingsLoading || settingsLoaded) return;
    settingsLoading = true;
    try {
      const result = await getSettingsViewModel();
      const model = result?.model || {};
      settingsImagePreference = model?.image_preference === "3d" ? "3d" : "2d";
      settingsHelpUrl = String(model?.ai_tagging_help_url || "#/help");
      settingsHasGoogleApiKey = Boolean(model?.google_api_key && String(model.google_api_key).trim().length > 0);
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
    return String(value || "").toLowerCase().replace(/[^a-z0-9]+/g, "");
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
    return String(pathValue || "").trim().replace(/\\/g, "/").toLowerCase();
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

  function syncImportPerFolderAssignments() {
    const folderPaths = new Set(
      importSelectedFiles
        .map((fullPath) => getFolderPathFromFilePath(fullPath))
        .filter(Boolean)
    );
    /** @type {Record<string, {designerId: string, sourceId: string}>} */
    const next = {};
    for (const folderPath of folderPaths) {
      const previous = importPerFolderAssignmentByPath?.[folderPath] || {};
      next[folderPath] = {
        designerId: String(previous.designerId || ""),
        sourceId: String(previous.sourceId || ""),
      };
    }
    importPerFolderAssignmentByPath = next;
  }

  /** @param {string} fullPath @param {boolean} checked */
  function toggleImportFile(fullPath, checked) {
    const value = String(fullPath || "").trim();
    if (!value) return;

    const existing = new Set(importSelectedFiles);
    if (checked) {
      existing.add(value);
    } else {
      existing.delete(value);
    }
    importSelectedFiles = Array.from(existing);
    syncImportPerFolderAssignments();
  }

  function selectAllImportFiles() {
    importSelectedFiles = Array.isArray(importPreview?.scanned_files)
      ? importPreview.scanned_files
        .map((file) => String(file?.full_path || "")).filter(Boolean)
      : [];
    syncImportPerFolderAssignments();
  }

  function clearImportFileSelection() {
    importSelectedFiles = [];
    syncImportPerFolderAssignments();
  }

  let importSelectedFolderSummaries = $derived(
    (() => {
      const counts = new Map();
      for (const fullPath of importSelectedFiles) {
        const folderPath = getFolderPathFromFilePath(fullPath);
        if (!folderPath) continue;
        counts.set(folderPath, (counts.get(folderPath) || 0) + 1);
      }
      return Array.from(counts.entries())
        .map(([folderPath, selectedCount]) => ({ folderPath, selectedCount }))
        .sort((left, right) => left.folderPath.localeCompare(right.folderPath, undefined, { sensitivity: "base" }));
    })()
  );

  let importStep2FolderGroups = $derived(
    (() => {
      const scannedFiles = Array.isArray(importPreview?.scanned_files) ? importPreview.scanned_files : [];
      const selectedByPath = new Set(importSelectedFiles.map((value) => String(value || "").trim()).filter(Boolean));
      const grouped = new Map();

      for (const rawFile of scannedFiles) {
        const fullPath = String(rawFile?.full_path || "").trim();
        if (!fullPath) continue;

        const folderPath = getFolderPathFromFilePath(fullPath) || "Unknown folder";
        const file = {
          fullPath,
          filename: getImportFilenameFromPath(fullPath),
          isSelected: selectedByPath.has(fullPath),
        };

        if (!grouped.has(folderPath)) {
          grouped.set(folderPath, {
            folderPath,
            folderLabel: getFolderLabelFromFolderPath(folderPath),
            files: [],
          });
        }
        grouped.get(folderPath).files.push(file);
      }

      return Array.from(grouped.values())
        .map((group) => {
          const sortedFiles = group.files.sort(/** @param {any} left @param {any} right */ (left, right) =>
            left.filename.localeCompare(right.filename, undefined, { sensitivity: "base" })
          );
          const selectedCount = sortedFiles.filter(/** @param {any} file */ (file) => file.isSelected).length;
          return {
            ...group,
            files: sortedFiles,
            selectedCount,
          };
        })
        .sort((left, right) => left.folderPath.localeCompare(right.folderPath, undefined, { sensitivity: "base" }));
    })()
  );

  let importStep2TotalFileCount = $derived(
    importStep2FolderGroups.reduce((total, folder) => total + folder.files.length, 0)
  );

  let importStep2SelectedFileCount = $derived(
    importStep2FolderGroups.reduce((total, folder) => total + folder.selectedCount, 0)
  );

  let importStep2CanSelectAll = $derived(importStep2SelectedFileCount < importStep2TotalFileCount);
  let importStep2CanDeselectAll = $derived(importStep2SelectedFileCount > 0);

  function buildImportConfirmWire() {
    const perFolderAssignments = importSelectedFolderSummaries.map((folder) => {
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

    return {
      wire: {
        root_paths: getActiveImportRoots(),
        global_designer_id: importGlobalDesignerId ? Number(importGlobalDesignerId) : null,
        global_source_id: importGlobalSourceId ? Number(importGlobalSourceId) : null,
        per_folder_assignments: perFolderAssignments,
        selected_files: [...importSelectedFiles],
        create_on_import: true,
      },
      context_token: null,
      canonical_confirm: false,
    };
  }

  async function runImportPrecheck() {
    if (importSelectedFiles.length === 0) {
      importError = "Select at least one file before continuing.";
      return;
    }

    importLoading = true;
    importError = "";
    importActionMessage = "";
    importActionNeedsSkipHoopsConfirm = false;

    try {
      const result = await precheckImportWire(buildImportConfirmWire());
      importPrecheck = result.precheck || null;
      importPrecheckSource = result.source || "mock";
      importPrecheckMessage = result.message || "Precheck complete.";
      importStep3ImagePreference = settingsImagePreference === "3d" ? "3d" : "2d";
      importContextToken = String(importPrecheck?.context_token || "");
      navigateTo(importPrecheck ? "#/import/step3" : "#/import/step2");
    } catch (error) {
      importError = `Import precheck failed: ${error}`;
      importPrecheck = null;
      importContextToken = "";
      navigateTo("#/import/step2");
    } finally {
      importLoading = false;
    }
  }

  /** @param {any} nextRoute */
  function mapServerImportRouteToHash(nextRoute) {
    const route = String(nextRoute || "").toLowerCase();
    if (route.startsWith("/designs")) return "#/designs";
    if (route.startsWith("/import")) {
      if (route.includes("step3") || route.includes("precheck") || route.includes("confirm")) return "#/import/step3";
      if (route.includes("step2") || route.includes("review") || route.includes("scan")) return "#/import/step2";
      if (route.includes("step1") || route.includes("folder")) return "#/import/step1";
      return "#/import/step1";
    }
    return null;
  }

  /** @param {string} action @param {boolean} [confirmSkipHoops] */
  async function executeImportPrecheckAction(action, confirmSkipHoops = false) {
    if (!importContextToken) {
      importError = "Missing import context token. Run precheck again.";
      return;
    }

    importActionLoading = true;
    importActionInProgress = String(action || "");
    importError = "";

    const importNowAction = action === "import_now";
    if (importNowAction) {
      importStopRequestPending = false;
      importProgressStatus = "";
      await startImportProgressUpdates(importContextToken);
    }

    try {
      const result = await runPrecheckAction({
        contextToken: importContextToken,
        action,
        confirmSkipHoops,
        imagePreferenceOverride: importStep3ImagePreference,
      });

      const actionResult = result.actionResult || null;
      /** @type {string} */
      const actionSource = result.source;
      /** @type {string} */
      const actionMessage = result.message;
      importActionSource = actionSource || "mock";
      importActionMessage = actionMessage || "Import precheck action complete.";
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
          if (persistedCount >= 1 && typeof onImportCompleted === "function") {
            onImportCompleted(persistedCount);
          }
          resetImportWizard();
        }
        navigateTo(hashRoute);
      }
    } catch (error) {
      importError = `Import action failed: ${error}`;
    } finally {
      if (importNowAction) {
        await stopImportProgressUpdates();
      }
      importStopRequestPending = false;
      importActionInProgress = "";
      importActionLoading = false;
    }
  }

  async function requestImportStop() {
    if (!importNowInProgress || importStopRequestPending) return;

    importStopRequestPending = true;
    importError = "";

    try {
      const result = await requestStopBulkImport();
      importActionSource = result.source || "mock";
      importActionMessage = result.message || "Stop requested.";
    } catch (error) {
      importError = `Stop request failed: ${error}`;
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
    importError = "";
    importActionMessage = "";
    importActionNeedsSkipHoopsConfirm = false;

    try {
      const result = await previewImportFromRoots(getActiveImportRoots());
      importPreview = result.preview || null;
      importPreviewSource = result.source || "mock";
      importSelectedFiles = Array.isArray(importPreview?.scanned_files)
        ? importPreview.scanned_files.map((file) => String(file?.full_path || "")).filter(Boolean)
        : [];
      importPerFolderAssignmentByPath = {};
      syncImportPerFolderAssignments();
      importPrecheck = null;
      importPrecheckSource = "mock";
      importPrecheckMessage = "Run precheck after selecting files.";
      importContextToken = "";
      navigateTo("#/import/step2");
    } catch (error) {
      importError = `Import preview failed: ${error}`;
      importPreview = null;
      importPreviewSource = "mock";
      importSelectedFiles = [];
      importPerFolderAssignmentByPath = {};
      importPrecheck = null;
      importContextToken = "";
      navigateTo("#/import/step1");
    } finally {
      importLoading = false;
    }
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
    importError = "";

    try {
      const currentValue = targetIndex === null || targetIndex === undefined || targetIndex < 0
        ? importRootPath
        : importRootPaths[targetIndex] || "";
      const startHint = parentFolder(currentValue) || "";
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
      importError = `Folder browse failed: ${error}`;
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
    const p = String(path || "").trim().replace(/[/\\]+$/, "");
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

  /** @param {string} [path] */
  function addImportRootPath(path = importRootPath) {
    const next = normalizeImportRootPath(path);
    if (!next) return;
    const existingByLower = new Set(importRootPaths.map((item) => String(item || "").toLowerCase()));
    if (!existingByLower.has(next.toLowerCase())) {
      importRootPaths = [...importRootPaths, next];
    }
    importRootPath = next;
  }

  /** @param {string} path */
  function removeImportRootPath(path) {
    const target = normalizeImportRootPath(path).toLowerCase();
    importRootPaths = importRootPaths.filter((value) => String(value || "").toLowerCase() !== target);
  }

  function clearImportRootPaths() {
    importRootPath = "";
    importRootPaths = [];
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
    importPrecheck = null;
    importPrecheckSource = "mock";
    importPrecheckMessage = "Run precheck after selecting files.";
    importStep3ImagePreference = "2d";
    importSelectedFiles = [];
    importContextToken = "";
    importActionMessage = "";
    importActionSource = "mock";
    importActionNeedsSkipHoopsConfirm = false;
    importStopRequestPending = false;
    importGlobalDesignerId = "";
    importGlobalSourceId = "";
    importPerFolderAssignmentByPath = {};
    importError = "";

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
        const currentFile = String(payload?.current_file ?? payload?.currentFile ?? "").trim();
        const currentFilename = currentFile.replace(/\\/g, "/").split("/").pop() || currentFile;

        if (stage === "started") {
          importProgressStatus = total > 0 ? `Starting import for ${total} file${total === 1 ? "" : "s"}...` : "Starting import...";
          return;
        }
        if (stage === "generating_images") {
          importProgressStatus = total > 0
            ? `${processed}/${total} processed (${committed} imported) - generating preview images...`
            : "Generating preview images...";
          return;
        }
        if ((stage === "processing_file" || stage === "processingFile") && total > 0) {
          importProgressStatus = `Processing ${Math.min(processed + 1, total)}/${total}: ${currentFilename}`;
          return;
        }
        if (stage === "batch_committed") {
          importProgressStatus = total > 0
            ? `${processed}/${total} processed (${committed} imported) - saving batch...`
            : `${committed} imported - saving batch...`;
          return;
        }
        if (stage === "stopped") {
          importProgressStatus = total > 0
            ? `Stopped after ${processed}/${total} processed (${committed} imported)`
            : `Stopped after ${committed} imported`;
          return;
        }
        if (stage === "completed") {
          importProgressStatus = total > 0
            ? `Completed ${processed}/${total} processed (${committed} imported)`
            : `Completed ${committed} imported`;
          return;
        }
        if (total > 0) {
          importProgressStatus = `${processed}/${total} processed (${committed} imported)`;
        } else {
          importProgressStatus = `${committed} imported`;
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
      <br>Select one or more folders containing embroidery files. Sub-folders are included automatically.
      <br>Your original files are never altered or moved. Files outside your main design directory are safely copied into the catalogue.
      <a href="#/help?section=importing" class="text-indigo-600 hover:underline ml-1">Import help</a>
    </p>

    <div class="import-step1-card bg-white rounded shadow p-6 max-w-2xl space-y-4">
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
                disabled={importLoading || importActionLoading || importBrowseLoading}
              >
                {importBrowseLoading ? "Browsing…" : "Browse…"}
              </button>
              <button
                type="button"
                class="ui-action-button menu-button-secondary py-2"
                onclick={() => clearImportRootPaths()}
                disabled={importLoading || importActionLoading || importBrowseLoading || (!String(importRootPath || "").trim() && importRootPaths.length === 0)}
                title="Remove this folder"
              >
                Remove
              </button>
            </div>

            {#each importRootPaths as rootPath, rowIndex}
              <div class="folder-row import-folder-row flex items-center gap-2" data-index={rowIndex + 1}>
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
              disabled={importLoading || importActionLoading || importBrowseLoading || !String(importRootPath || "").trim()}
            >
              Add another folder
            </button>
          </div>
        </div>

        <div class="ui-action-button-group import-step1-primary-actions pt-2 flex gap-2">
          <button class="menu-button-primary ui-action-button ui-action-button-primary" type="submit" disabled={importLoading || importBrowseLoading}>
            {importLoading ? "Running…" : "Scan folder(s)"}
          </button>
          <button type="button" class="menu-button-secondary ui-action-button" onclick={resetImportWizard} disabled={importLoading || importActionLoading || importBrowseLoading}>
            Reset
          </button>
        </div>

        {#if importError}
          <p class="text-sm text-red-600">{importError}</p>
        {/if}

        {#if importPreview}
          <div class="grid sm:grid-cols-3 gap-3 text-sm pt-2">
            <div class="ui-section-shell import-metric-card bg-gray-50 border rounded p-3 text-center">Source: <strong>{importPreviewSource}</strong></div>
            <div class="ui-section-shell import-metric-card bg-gray-50 border rounded p-3 text-center">Discovered: <strong>{importPreview.discovered_count ?? 0}</strong></div>
            <div class="ui-section-shell import-metric-card bg-gray-50 border rounded p-3 text-center">Folders: <strong>{importPreview.folder_count ?? 0}</strong></div>
          </div>
        {/if}
      </form>
    </div>
  {/if}

  {#if importRouteStep === 2}
    {#if importPreview}
      <div class="ui-section-shell import-panel space-y-4">
        <div class="space-y-1">
          <p class="ui-field-label import-field-label font-bold text-gray-800 text-lg">Review scanned files</p>
          <p class="ui-help-note text-sm text-gray-500">
            {importStep2FolderGroups.length || importPreview.folder_count || 0} folder(s) scanned - {Array.isArray(importPreview.scanned_files) ? importPreview.scanned_files.length : 0} file(s) found.
            Selected files will be <strong>copied into the catalogue</strong>.
            <a href="#/help?section=importing" class="text-indigo-600 hover:underline ml-1">Import help</a>
          </p>
        </div>

        <div class="ui-section-shell p-4 border rounded bg-gray-50 space-y-3 import-step2-global-shell">
          <p class="ui-field-label import-field-label font-semibold text-gray-800 text-sm">Apply to all folders (optional override)</p>
          <div class="grid grid-cols-2 gap-3 text-sm import-step2-global-grid">
            <label class="ui-field-label text-sm block">
              <span class="block font-medium mb-1 text-gray-700">Designer</span>
              <select class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white" bind:value={importGlobalDesignerId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                <option value="">Keep inferred (per folder)</option>
                {#each importDesigners as designer}
                  <option value={String(designer.id)}>{designer.name}</option>
                {/each}
              </select>
            </label>
            <label class="ui-field-label text-sm block">
              <span class="block font-medium mb-1 text-gray-700">Source</span>
              <select class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white" bind:value={importGlobalSourceId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                <option value="">Keep inferred (per folder)</option>
                {#each importSources as source}
                  <option value={String(source.id)}>{source.name}</option>
                {/each}
              </select>
            </label>
          </div>
        </div>

        <div class="space-y-2 import-step2-actions-shell">
          <div class="ui-action-button-group import-step1-primary-actions import-step2-primary-actions import-step2-inline-actions flex flex-wrap gap-2 items-center">
            <button class="menu-button-primary ui-action-button ui-action-button-primary" onclick={runImportPrecheck} disabled={importLoading || importActionLoading || importSelectedFiles.length === 0}>
              {#if importSelectedFiles.length > 0}
                Continue with {importSelectedFiles.length} design{importSelectedFiles.length === 1 ? "" : "s"}
              {:else}
                Continue
              {/if}
            </button>
            <button type="button" class="menu-button-secondary ui-action-button" onclick={() => navigateTo("#/import/step1")} disabled={importLoading || importActionLoading}>
              Cancel
            </button>
            <button
              type="button"
              class={`px-3 py-1.5 rounded border text-xs font-semibold ${importStep2CanSelectAll ? "bg-white hover:bg-gray-50 text-indigo-600" : "text-gray-400 bg-gray-50 cursor-not-allowed"}`}
              onclick={selectAllImportFiles}
              disabled={importLoading || importActionLoading || !importStep2CanSelectAll}
            >
              Select all
            </button>
            <button
              type="button"
              class={`px-3 py-1.5 rounded border text-xs font-semibold ${importStep2CanDeselectAll ? "bg-white hover:bg-gray-50 text-indigo-600" : "text-gray-400 bg-gray-50 cursor-not-allowed"}`}
              onclick={clearImportFileSelection}
              disabled={importLoading || importActionLoading || !importStep2CanDeselectAll}
            >
              Deselect all
            </button>
          </div>
        </div>

        {#if importStep2FolderGroups.length > 0}
          <div class="space-y-4">
            {#each importStep2FolderGroups as folder}
              <div class="ui-section-shell overflow-hidden border rounded bg-white import-step2-folder-shell shadow-sm">
                <div class="bg-gray-50 border-b px-4 py-2.5 flex flex-wrap items-center gap-3 import-step2-folder-header">
                  <div class="flex-1 min-w-0">
                    <code class="text-xs text-black font-bold import-step2-folder-label">{folder.folderLabel}</code>
                    <span class="mx-2 text-xs text-gray-400" aria-hidden="true">-</span>
                    <code class="text-xs text-gray-500 break-all">{folder.folderPath}</code>
                  </div>
                </div>

                <div class="px-4 py-3 border-b bg-gray-50/50 import-step2-folder-overrides">
                  <div class="grid grid-cols-2 gap-3 text-sm">
                    <label class="ui-field-label text-sm block">
                      <span class="block font-medium mb-1 text-gray-700">Designer for this folder</span>
                      <select
                        class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                        value={getImportFolderDesigner(folder.folderPath)}
                        onchange={(event) => setImportFolderDesigner(folder.folderPath, event.currentTarget.value)}
                        disabled={importReferenceLoading || importLoading || importActionLoading}
                      >
                        <option value="">{getImportFolderDesignerInferredLabel(folder.folderPath)}</option>
                        {#each importDesigners as designer}
                          <option value={String(designer.id)}>{designer.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label class="ui-field-label text-sm block">
                      <span class="block font-medium mb-1 text-gray-700">Source for this folder</span>
                      <select
                        class="ui-select-input ui-control-text-inset w-full border rounded px-3 py-1.5 bg-white"
                        value={getImportFolderSource(folder.folderPath)}
                        onchange={(event) => setImportFolderSource(folder.folderPath, event.currentTarget.value)}
                        disabled={importReferenceLoading || importLoading || importActionLoading}
                      >
                        <option value="">{getImportFolderSourceInferredLabel(folder.folderPath)}</option>
                        {#each importSources as source}
                          <option value={String(source.id)}>{source.name}</option>
                        {/each}
                      </select>
                    </label>
                  </div>
                </div>

                <div class="import-step2-file-list-shell p-4">
                  <div class="import-step2-file-columns grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                    {#each folder.files as file}
                      <label class="import-step2-file-item flex items-start gap-2 cursor-pointer">
                        <input
                          type="checkbox"
                          class="ui-checkbox mt-1 accent-indigo-600 rounded"
                          checked={file.isSelected}
                          onchange={(event) => toggleImportFile(file.fullPath, event.currentTarget.checked)}
                          disabled={importLoading || importActionLoading}
                        />
                        <span class="ui-field-label text-sm text-gray-700 break-all font-mono" title={file.fullPath}>{file.filename}</span>
                      </label>
                    {/each}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-sm text-gray-600 italic">No supported files discovered in this preview.</p>
        {/if}
      </div>
    {:else}
      <div class="ui-section-shell import-panel space-y-2 border rounded p-4 bg-white text-center">
        <p class="ui-help-note italic text-gray-500">Step 2 needs a completed preview first.</p>
        <div class="pt-2">
          <button type="button" class="menu-button-secondary ui-action-button" onclick={() => navigateTo("#/import/step1")}>Back to Step 1</button>
        </div>
      </div>
    {/if}
  {/if}

  {#if importRouteStep === 3}
    {#if importPrecheck}
      <div class="ui-section-shell import-panel space-y-4">
        <p class="ui-field-label import-field-label font-bold text-gray-800 text-lg">Before You Import</p>

        {#if settingsHasGoogleApiKey}
          <div class="ui-section-shell border border-amber-300 bg-amber-50 text-amber-950 p-4 rounded space-y-2 text-sm">
            <p class="font-semibold text-amber-900">Google AI tagging is enabled for this installation</p>
            <p class="ui-help-note text-amber-900">
              Depending on your saved settings, Tier 2 and/or Tier 3 may run during this import. Gemini usage may incur cost. Free-tier limits are approximately
              <strong>15 requests per minute</strong> and <strong>1,500 requests per day</strong>.
              A historical estimate from February 2026 found that Tier 3 on 4,000 images cost about <strong>$0.33 on the paid tier</strong>; actual pricing may have changed -
              check <a href="https://ai.google.dev/pricing" target="_blank" rel="noopener" class="underline hover:text-amber-800">ai.google.dev/pricing</a>.
            </p>
          </div>
        {:else}
          <div class="ui-section-shell border border-blue-300 bg-blue-50 text-blue-950 p-4 rounded space-y-2 text-sm">
            <p class="font-semibold text-blue-900">Google AI tagging is not configured</p>
            <p class="ui-help-note text-blue-900">
              No Google API key is currently saved, so this import will use <strong>Tier 1 keyword tagging only</strong> and no Gemini calls will be made.
              If you want AI-assisted tagging, add an API key in Settings and enable the tiers you want.
            </p>
            <p class="text-xs text-blue-900 pt-1">
              <a href="#/admin/settings" class="underline font-medium">Admin Settings</a>
              · <a href={settingsHelpUrl} class="underline font-medium">AI Tagging Guide</a>
            </p>
          </div>
        {/if}

        <div class="ui-section-shell border rounded p-4 bg-white space-y-3 shadow-sm">
          <p class="font-semibold text-gray-800 text-sm">Image Preview Preference</p>
          <p class="ui-help-note text-xs text-gray-500">
            Choose how preview images are generated for this import. 2D is faster (flat render), 3D is slower but shows stitch simulation.
            Your saved setting is shown below; you can override it for this session.
          </p>
          <div class="flex flex-wrap items-center gap-4 text-sm pt-1">
            <label class="inline-flex items-center gap-2 cursor-pointer">
              <input type="radio" class="ui-radio accent-indigo-600" name="import-step3-image-preference" value="2d" bind:group={importStep3ImagePreference} disabled={importActionLoading || !importContextToken} />
              <span class="font-semibold">2D - Fast flat preview</span>
            </label>
            <label class="inline-flex items-center gap-2 cursor-pointer">
              <input type="radio" class="ui-radio accent-indigo-600" name="import-step3-image-preference" value="3d" bind:group={importStep3ImagePreference} disabled={importActionLoading || !importContextToken} />
              <span class="font-semibold">3D - Detailed stitch simulation</span>
            </label>
            <span class="text-xs text-gray-400">(Saved setting: {settingsImagePreference === "3d" ? "3D" : "2D"})</span>
          </div>
        </div>

        {#if importPrecheck.is_first_import}
          <div class="ui-section-shell import-folder-card border border-amber-300 bg-amber-50 text-amber-950 p-4 rounded space-y-2 text-sm">
            <p class="font-semibold text-amber-900">Before your first import, please check your hoops</p>
            <p class="ui-help-note text-amber-900">
              A starter set of tags is already included with the catalogue. Hoops are not, because they depend on your machine and the frames you actually own.
            </p>
            <p class="ui-help-note text-amber-900">
              If you set up your hoops now, the import process can auto-assign a hoop where the design size is known. You can also review tags, sources, and designers before importing.
            </p>
            {#if importPrecheck.needs_hoop_setup}
              <p class="ui-help-note font-semibold text-amber-900 pt-1">No hoops are defined yet for this catalogue.</p>
            {/if}
          </div>

          <p class="ui-help-note text-sm text-gray-500 italic">
            Review your hoops first, or skip them for now and the app will ask if you are really really sure before importing.
          </p>
        {:else}
          <p class="ui-help-note text-sm text-gray-500 italic">
            Consider reviewing your hoops, tags, sources, or designers before importing. Hoops usually only need special attention on the first import.
          </p>
        {/if}

        <div class="ui-action-button-group flex flex-wrap gap-2 pt-2">
          <button class="menu-button-secondary ui-action-button" onclick={() => executeImportPrecheckAction("review_hoops")} disabled={importActionLoading || !importContextToken}>Review Hoops</button>
          <button class="menu-button-secondary ui-action-button" onclick={() => executeImportPrecheckAction("review_tags")} disabled={importActionLoading || !importContextToken}>Review Tags</button>
          <button class="menu-button-secondary ui-action-button" onclick={() => executeImportPrecheckAction("review_sources")} disabled={importActionLoading || !importContextToken}>Review Sources</button>
          <button class="menu-button-secondary ui-action-button" onclick={() => executeImportPrecheckAction("review_designers")} disabled={importActionLoading || !importContextToken}>Review Designers</button>
          <button class="menu-button-primary ui-action-button ui-action-button-primary" onclick={() => executeImportPrecheckAction("import_now")} disabled={importActionLoading || !importContextToken}>
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
            onclick={importNowInProgress ? requestImportStop : () => executeImportPrecheckAction("cancel")}
            disabled={importNowInProgress ? importStopRequestPending : importActionLoading || !importContextToken}
          >
            {#if importNowInProgress}
              {importStopRequestPending ? "Stopping..." : "Stop"}
            {:else}
              Cancel
            {/if}
          </button>
        </div>

        {#if importActionNeedsSkipHoopsConfirm}
          <div class="ui-section-shell import-folder-card border border-amber-300 bg-amber-50 text-amber-950 p-4 rounded space-y-2 text-sm mt-3">
            <p class="ui-help-note text-amber-800 font-semibold">
              Hoops are not configured for a first import. Confirm to continue anyway.
            </p>
            <button class="menu-button-primary ui-action-button ui-action-button-primary text-xs" onclick={() => executeImportPrecheckAction("import_now", true)} disabled={importActionLoading || !importContextToken}>
              Confirm import without hoop setup
            </button>
          </div>
        {/if}

        {#if importActionMessage}
          <p class="ui-help-note text-sm text-indigo-700 bg-indigo-50 border border-indigo-200 rounded p-3 mt-3">{importActionMessage}</p>
        {/if}
      </div>
    {:else}
      <div class="ui-section-shell import-panel space-y-2 border rounded p-4 bg-white text-center">
        <p class="ui-help-note italic text-gray-500">Step 3 needs precheck to be completed first.</p>
        <div class="pt-2">
          <button type="button" class="menu-button-secondary ui-action-button" onclick={() => navigateTo(importPreview ? "#/import/step2" : "#/import/step1")}>Go to previous step</button>
        </div>
      </div>
    {/if}
  {/if}
</section>
