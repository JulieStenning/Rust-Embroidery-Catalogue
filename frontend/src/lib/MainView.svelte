<script>
  import { onDestroy, tick } from "svelte";
  import {
    getBrowseDesigns,
    getBrowseDesignPreviews,
    getBrowseProjects,
    getBrowseTags,
    getDesignDetail,
    browseImportFolder,
    previewImportFromRoots,
    precheckImportWire,
    runPrecheckAction,
    bulkVerifyDesigns,
    bulkAddDesignsToProject,
    bulkSetTagsForDesigns,
    getSettingsViewModel,
    saveSettings,
    saveImportLastBrowseFolder,
    browseSettingsDataRoot,
    listDesigners,
    createDesigner,
    deleteDesigner as removeDesigner,
    listSources,
    createSource,
    deleteSource as removeSource,
    listTags,
    createTag,
    setTagGroup as updateTagGroup,
    deleteTag as removeTag,
    listHoops,
    createHoop,
    updateHoop,
    deleteHoop as removeHoop,
  } from "./api/commandAdapter.js";

  const ROUTE_PAGES = {
    "#/designs": {
      title: "Browse",
      subtitle: "Design browser placeholder",
      description: "This staged placeholder will become the catalog grid, filters, and design actions.",
      cta: "Next backend hookup: list designs command",
    },
    "#/import": {
      title: "Import",
      subtitle: "Bulk import wizard",
      description:
        "Select one or more folders, review discovered files and assignments, then run precheck and confirm import.",
      cta: "Step 1 choose roots, Step 2 review and assign, Step 3 run precheck actions and import",
    },
    "#/projects": {
      title: "Projects",
      subtitle: "Project management placeholder",
      description: "This page will host project creation, membership lists, and print-oriented views.",
      cta: "Next backend hookup: project list and project detail commands",
    },
    "#/help": {
      title: "Help",
      subtitle: "Help and guidance placeholder",
      description: "This page will surface in-app guides and troubleshooting content.",
      cta: "Next backend hookup: help document query commands",
    },
    "#/admin/designers": {
      title: "Designers",
      subtitle: "Admin designer placeholder",
      description: "This page will provide create, edit, and remove controls for designer reference data.",
      cta: "Next backend hookup: designers CRUD commands",
    },
    "#/admin/tags": {
      title: "Tags",
      subtitle: "Admin tags placeholder",
      description: "This page will provide grouped tag management and tag metadata editing.",
      cta: "Next backend hookup: tags CRUD commands",
    },
    "#/admin/sources": {
      title: "Sources",
      subtitle: "Admin sources placeholder",
      description: "This page will provide source reference management controls.",
      cta: "Next backend hookup: sources CRUD commands",
    },
    "#/admin/hoops": {
      title: "Hoops",
      subtitle: "Hoop catalog management",
      description: "Add, review, and remove machine hoop sizes used during import and design validation.",
      cta: "Maintain the hoop list your machine setup supports",
    },
    "#/admin/settings": {
      title: "Settings",
      subtitle: "Admin settings placeholder",
      description: "This page will expose app settings such as AI and import configuration.",
      cta: "Next backend hookup: settings read/write commands",
    },
    "#/admin/maintenance/backup": {
      title: "Backup",
      subtitle: "Backup and maintenance placeholder",
      description: "This page will host backup execution, destination controls, and status reporting.",
      cta: "Next backend hookup: backup execute and status commands",
    },
    "#/admin/tagging-actions": {
      title: "Tagging Actions",
      subtitle: "Batch tagging placeholder",
      description: "This page will expose tier controls and batch tagging execution options.",
      cta: "Next backend hookup: tagging action preview and run commands",
    },
    "#/admin/orphans": {
      title: "Orphans",
      subtitle: "Orphan scan results placeholder",
      description: "This page is the destination after the Orphans menu action confirms results.",
      cta: "Next backend hookup: orphan scan and cleanup commands",
    },
    "#/about": {
      title: "About",
      subtitle: "About page placeholder",
      description: "This page will present app information, disclaimers, and references.",
      cta: "Next backend hookup: about document commands",
    },
    "#/about/licence": {
      title: "Licence",
      subtitle: "Licence page placeholder",
      description: "This page will present licensing and third-party notice details.",
      cta: "Next backend hookup: license document commands",
    },
  };

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
  ];

  const ROUTE_UI_KIND = {
    "#/designs": "browse",
    "#/import": "import",
    "#/projects": "projects",
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
    "#/about/licence": "licence",
  };

  function parseDesignDetailId(route) {
    const match = route.match(/^#\/designs\/(\d+)$/);
    return match ? Number(match[1]) : null;
  }

  function resolveCurrentPage(route) {
    if (parseDesignDetailId(route) !== null) {
      return {
        title: "Design Detail",
        subtitle: "Design detail placeholder",
        description: "This staged page shows a single design summary while deeper metadata wiring is migrated.",
        cta: "Next backend hookup: richer detail and related actions",
      };
    }
    return ROUTE_PAGES[route] || null;
  }

  function resolveCurrentUiKind(route) {
    if (parseDesignDetailId(route) !== null) {
      return "design-detail";
    }
    return ROUTE_UI_KIND[route] || null;
  }

  let currentRoute = $state("");
  let currentPage = $derived(resolveCurrentPage(currentRoute));
  let currentUiKind = $derived(resolveCurrentUiKind(currentRoute));
  let detailDesignId = $derived(parseDesignDetailId(currentRoute));

  let browseItems = $state([]);
  let browseSource = $state("mock");
  let browseLoading = $state(false);
  let browseHasLoaded = $state(false);
  let browseError = $state("");
  let browseProjects = $state([]);
  let browseProjectsSource = $state("mock");
  let browseTagOptions = $state([]);
  let browseTagsSource = $state("mock");
  let browsePreviewById = $state({});
  let browsePreviewsSource = $state("mock");
  let browsePreviewsLoading = $state(false);
  let browsePreviewRequestCounter = 0;
  let browseCurrentPage = $state(1);
  let browseAdditionalFiltersOpen = $state(false);
  let browseSelectedIds = $state([]);
  let browseBulkModalOpen = $state(false);
  let browseBulkTagSelection = $state([]);
  let browseBulkProject = $state("");
  let browseActionNotice = $state("");
  let browseGridContainer = $state(null);
  let browseGridColumns = $state(1);

  const BROWSE_PAGE_ROWS = 10;
  const BROWSE_CARD_MIN_WIDTH = 170;
  const BROWSE_CARD_GAP = 16;
  const BROWSE_TAG_UNTAGGED = "__untagged__";

  const defaultBrowseFilters = () => ({
    q: "",
    allWords: "",
    exactPhrase: "",
    anyWords: "",
    noneWords: "",
    filename: "",
    designer: "",
    tagFilters: [],
    hoop: "",
    source: "",
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

  let detailItem = $state(null);
  let detailSource = $state("mock");
  let detailLoading = $state(false);
  let detailError = $state("");

  let importRootPath = $state("C:/imports");
  let importRootPaths = $state([]);
  let importStep = $state(1);
  let importPreview = $state(null);
  let importPreviewSource = $state("mock");
  let importPreviewMessage = $state("Import preview has not run yet.");
  let importPrecheck = $state(null);
  let importPrecheckSource = $state("mock");
  let importPrecheckMessage = $state("Run precheck after selecting files.");
  let importSelectedFiles = $state([]);
  let importContextToken = $state("");
  let importActionMessage = $state("");
  let importActionSource = $state("mock");
  let importActionNeedsSkipHoopsConfirm = $state(false);
  let importActionLoading = $state(false);
  let importActionInProgress = $state("");
  let importProgressStatus = $state("");
  let importProgressToken = $state("");
  let importProgressUnlisten = null;
  let importGlobalDesignerId = $state("");
  let importGlobalSourceId = $state("");
  let importPerFolderAssignmentByPath = $state({});
  let importDesigners = $state([]);
  let importSources = $state([]);
  let importReferenceLoading = $state(false);
  let importLoading = $state(false);
  let importBrowseLoading = $state(false);
  let importError = $state("");

  let settingsImagePreference = $state("2d");
  let settingsGoogleApiKey = $state("");
  let settingsApiKeyRevealed = $state(false);
  let settingsAiTier2Auto = $state(false);
  let settingsAiTier3Auto = $state(false);
  let settingsAiBatchSize = $state("");
  let settingsAiDelay = $state("");
  let settingsImportCommitBatchSize = $state("");
  let settingsImportLastBrowseFolder = $state("");
  let settingsCanConfigureDataRoot = $state(false);
  let settingsDataRoot = $state("");
  let settingsLogFolder = $state("");
  let settingsAppMode = $state("development");
  let settingsLoaded = $state(false);
  let settingsLoading = $state(false);
  let settingsSaveState = $state("idle");
  let settingsSaveMessage = $state("");
  let settingsHelpUrl = $state("#/help");
  let settingsHasGoogleApiKey = $derived(settingsGoogleApiKey.trim().length > 0);

  let backupDbDestination = $state("");
  let backupDesignsDestination = $state("");
  let backupSavedDbDestination = $state("");
  let backupSavedDesignsDestination = $state("");
  let backupStatus = $state("idle");
  let backupMessage = $state("");
  let importHasAppliedSavedRoot = $state(false);

  let adminNotice = $state("");
  let adminNoticeType = $state("info");
  let adminLoading = $state(false);
  let adminDataSource = $state("rust");

  let designers = $state([]);
  let sources = $state([]);
  let tags = $state([]);
  let hoops = $state([]);

  let newDesignerName = $state("");
  let newSourceName = $state("");
  let newTagDescription = $state("");
  let newTagGroup = $state("image");
  let newHoopName = $state("");
  let newHoopWidth = $state("");
  let newHoopHeight = $state("");
  let editingHoopId = $state(null);
  let editingHoopName = $state("");
  let editingHoopWidth = $state("");
  let editingHoopHeight = $state("");
  let pendingDeleteHoopId = $state(null);

  let imageTags = $derived(tags.filter((tag) => tag.tagGroup === "image"));
  let stitchingTags = $derived(tags.filter((tag) => tag.tagGroup === "stitching"));
  let unclassifiedTags = $derived(tags.filter((tag) => !tag.tagGroup));

  function setAdminNotice(message, type = "info") {
    adminNotice = message;
    adminNoticeType = type;
  }

  async function loadAdminDataForCurrentRoute(force = false) {
    if (adminLoading && !force) {
      return;
    }

    adminLoading = true;
    try {
      if (adminIsDesignersRoute) {
        const result = await listDesigners();
        designers = Array.isArray(result?.items) ? result.items : [];
        adminDataSource = result?.source || "mock";
      } else if (adminIsSourcesRoute) {
        const result = await listSources();
        sources = Array.isArray(result?.items) ? result.items : [];
        adminDataSource = result?.source || "mock";
      } else if (adminIsTagsRoute) {
        const result = await listTags();
        tags = Array.isArray(result?.items)
          ? result.items.map((tag) => ({
              id: Number(tag.id),
              description: String(tag.description || ""),
              tagGroup: String(tag.tag_group || ""),
            }))
          : [];
        adminDataSource = result?.source || "mock";
      } else if (adminIsHoopsRoute) {
        const result = await listHoops();
        hoops = Array.isArray(result?.items)
          ? result.items.map((hoop) => ({
              id: Number(hoop.id),
              name: String(hoop.name || ""),
              maxWidthMm: Number(hoop.max_width_mm ?? 0),
              maxHeightMm: Number(hoop.max_height_mm ?? 0),
              designCount: Number(hoop.design_count ?? 0),
            }))
          : [];
        adminDataSource = result?.source || "mock";
      }
    } finally {
      adminLoading = false;
    }
  }

  async function addDesigner(event) {
    event.preventDefault();
    const name = newDesignerName.trim();
    if (!name) {
      return;
    }

    const result = await createDesigner(name);
    if (!result?.persisted) {
      setAdminNotice(`Could not add designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newDesignerName = "";
    setAdminNotice("Designer added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function deleteDesigner(id) {
    const result = await removeDesigner(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    setAdminNotice("Designer deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function addSource(event) {
    event.preventDefault();
    const name = newSourceName.trim();
    if (!name) {
      return;
    }

    const result = await createSource(name);
    if (!result?.persisted) {
      setAdminNotice(`Could not add source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newSourceName = "";
    setAdminNotice("Source added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function deleteSource(id) {
    const result = await removeSource(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    setAdminNotice("Source deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function addTag(event) {
    event.preventDefault();
    const description = newTagDescription.trim();
    if (!description) {
      return;
    }

    const result = await createTag(description, newTagGroup);
    if (!result?.persisted) {
      setAdminNotice(`Could not add tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newTagDescription = "";
    newTagGroup = "image";
    setAdminNotice("Tag added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function setTagGroup(id, tagGroup) {
    const result = await updateTagGroup(id, tagGroup);
    if (!result?.persisted) {
      setAdminNotice(`Could not update tag group: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    setAdminNotice("Tag group updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function deleteTag(id) {
    const result = await removeTag(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    setAdminNotice("Tag deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  async function addHoop(event) {
    event.preventDefault();
    const name = newHoopName.trim();
    const width = Number(newHoopWidth);
    const height = Number(newHoopHeight);
    if (!name || Number.isNaN(width) || Number.isNaN(height) || width <= 0 || height <= 0) {
      setAdminNotice("Enter a name plus valid positive width and height.", "error");
      return;
    }

    const result = await createHoop(name, width, height);
    if (!result?.persisted) {
      setAdminNotice(`Could not add hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newHoopName = "";
    newHoopWidth = "";
    newHoopHeight = "";
    setAdminNotice("Hoop added.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  function beginEditHoop(hoop) {
    if (!hoop) {
      return;
    }

    pendingDeleteHoopId = null;
    editingHoopId = Number(hoop.id);
    editingHoopName = String(hoop.name || "");
    editingHoopWidth = String(hoop.maxWidthMm ?? "");
    editingHoopHeight = String(hoop.maxHeightMm ?? "");
  }

  function cancelEditHoop() {
    editingHoopId = null;
    editingHoopName = "";
    editingHoopWidth = "";
    editingHoopHeight = "";
  }

  async function saveHoopEdit(id) {
    const name = editingHoopName.trim();
    const width = Number(editingHoopWidth);
    const height = Number(editingHoopHeight);
    if (!name || Number.isNaN(width) || Number.isNaN(height) || width <= 0 || height <= 0) {
      setAdminNotice("Enter a name plus valid positive width and height.", "error");
      return;
    }

    const result = await updateHoop(id, name, width, height);
    if (!result?.persisted) {
      setAdminNotice(`Could not update hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditHoop();
    setAdminNotice("Hoop updated.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  function requestDeleteHoop(hoop) {
    if (!hoop) {
      return;
    }

    cancelEditHoop();
    pendingDeleteHoopId = Number(hoop.id);
    if (Number(hoop.designCount) > 0) {
      setAdminNotice(
        `Deleting '${hoop.name}' will clear the hoop assignment from ${hoop.designCount} design(s).`,
        "info"
      );
      return;
    }

    setAdminNotice(`Delete '${hoop.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteHoop() {
    pendingDeleteHoopId = null;
  }

  async function deleteHoop(id) {
    pendingDeleteHoopId = null;
    const result = await removeHoop(id);
    if (!result?.persisted) {
      setAdminNotice(`Could not delete hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditHoop();
    setAdminNotice("Hoop deleted.", "success");
    await loadAdminDataForCurrentRoute(true);
  }

  let backupHasUnsavedChanges = $derived(
    backupDbDestination.trim() !== backupSavedDbDestination.trim()
      || backupDesignsDestination.trim() !== backupSavedDesignsDestination.trim()
  );
  let backupHasDbDestination = $derived(backupSavedDbDestination.trim().length > 0);
  let backupHasDesignsDestination = $derived(backupSavedDesignsDestination.trim().length > 0);
  let backupDbSourcePath = $derived(
    settingsDataRoot ? `${settingsDataRoot}\\database\\catalogue.db` : "(not available yet)"
  );
  let backupDesignsSourcePath = $derived(
    settingsDataRoot ? `${settingsDataRoot}\\MachineEmbroideryDesigns` : "(not available yet)"
  );

  let adminIsDesignersRoute = $derived(currentRoute === "#/admin/designers");
  let adminIsTagsRoute = $derived(currentRoute === "#/admin/tags");
  let adminIsSourcesRoute = $derived(currentRoute === "#/admin/sources");
  let adminIsHoopsRoute = $derived(currentRoute === "#/admin/hoops");

  let orphanModalOpen = $state(false);
  let orphanStatus = $state("loading");
  let orphanChecked = $state(0);
  let orphanFound = $state(0);
  let orphanError = $state("");
  let orphanCanDismiss = $derived(orphanStatus !== "loading");

  let orphanInvoker = $state(null);
  let orphanModalContainer = $state(null);
  let orphanModalTitle = $state(null);
  let orphanCloseButton = $state(null);
  let orphanPrimaryButton = $state(null);

  function toggleSettingsApiKeyVisibility() {
    settingsApiKeyRevealed = !settingsApiKeyRevealed;
  }

  function applySettingsModel(model) {
    settingsImagePreference = model?.image_preference === "3d" ? "3d" : "2d";
    settingsGoogleApiKey = String(model?.google_api_key || "");
    settingsAiTier2Auto = Boolean(model?.ai_tier2_auto);
    settingsAiTier3Auto = Boolean(model?.ai_tier3_auto);
    settingsAiBatchSize = String(model?.ai_batch_size || "");
    settingsAiDelay = String(model?.ai_delay || "");
    settingsImportCommitBatchSize = String(model?.import_commit_batch_size || "");
    settingsImportLastBrowseFolder = String(model?.import_last_browse_folder || "");
    settingsCanConfigureDataRoot = Boolean(model?.can_configure_data_root);
    settingsDataRoot = String(model?.data_root || "");
    settingsLogFolder = String(model?.log_folder || "");
    settingsAppMode = String(model?.app_mode || "development");
    settingsHelpUrl = String(model?.ai_tagging_help_url || "#/help");
  }

  async function persistImportLastBrowseFolder(path) {
    const normalized = normalizeImportRootPath(path);
    if (!normalized) {
      return;
    }

    const result = await saveImportLastBrowseFolder(normalized);
    if (result?.persisted) {
      settingsImportLastBrowseFolder = normalized;
      return;
    }

    console.info("Could not persist import.last_browse_folder", result?.error || "unknown error");
  }

  async function loadSettingsFromBackend(force = false) {
    if (settingsLoading) {
      return;
    }
    if (settingsLoaded && !force) {
      return;
    }

    settingsLoading = true;
    try {
      const result = await getSettingsViewModel();
      applySettingsModel(result.model || {});
      settingsLoaded = true;
    } catch (error) {
      settingsSaveState = "error";
      settingsSaveMessage = `Could not load settings: ${error}`;
    } finally {
      settingsLoading = false;
    }
  }

  async function saveSettingsFromBackend(event) {
    event.preventDefault();
    settingsSaveState = "saving";
    settingsSaveMessage = "";

    try {
      const result = await saveSettings({
        image_preference: settingsImagePreference,
        google_api_key: settingsGoogleApiKey,
        ai_tier2_auto: settingsAiTier2Auto,
        ai_tier3_auto: settingsAiTier3Auto,
        ai_batch_size: settingsAiBatchSize,
        ai_delay: settingsAiDelay,
        import_commit_batch_size: settingsImportCommitBatchSize,
        data_root: settingsDataRoot,
      });

      if (result.saved) {
        settingsSaveState = "saved";
        settingsSaveMessage = result.message || "Settings saved successfully.";
      } else {
        settingsSaveState = "error";
        settingsSaveMessage = result.message || "Settings could not be saved.";
      }
    } catch (error) {
      settingsSaveState = "error";
      settingsSaveMessage = `Could not save settings: ${error}`;
    }
  }

  async function browseDataRootFromBackend() {
    const result = await browseSettingsDataRoot(settingsDataRoot);
    if (result.path) {
      settingsDataRoot = result.path;
      settingsSaveState = "idle";
      settingsSaveMessage = "";
      return;
    }

    if (result.error) {
      settingsSaveState = "error";
      settingsSaveMessage = result.error;
    }
  }

  function browseBackupDestinationUiOnly(kind) {
    backupStatus = "error";
    backupMessage = `Folder picker for ${kind} destination is not wired yet. Please enter the destination path manually.`;
  }

  function saveBackupDestinationsUiOnly(event) {
    event.preventDefault();

    if (!backupHasUnsavedChanges) {
      backupStatus = "error";
      backupMessage = "There are no destination changes to save.";
      return;
    }

    backupSavedDbDestination = backupDbDestination.trim();
    backupSavedDesignsDestination = backupDesignsDestination.trim();
    backupStatus = "saved";
    backupMessage = "Backup destinations saved.";
  }

  function runBackupActionUiOnly(action) {
    if (action === "database" && !backupHasDbDestination) {
      backupStatus = "error";
      backupMessage = "No database backup destination is configured. Please set one below and save destinations.";
      return;
    }

    if (action === "designs" && !backupHasDesignsDestination) {
      backupStatus = "error";
      backupMessage = "No designs backup destination is configured. Please set one below and save destinations.";
      return;
    }

    if (action === "both" && (!backupHasDbDestination || !backupHasDesignsDestination)) {
      backupStatus = "error";
      backupMessage = "Both backup destinations must be configured before you can run both backups.";
      return;
    }

    backupStatus = "saved";
    backupMessage = `Backup action "${action}" is queued in UI-only mode. Backend execution wiring is next.`;
  }

  function normalizeHash(hash) {
    if (!hash || hash === "#" || hash === "#/") {
      return "#/designs";
    }
    return hash;
  }

  function syncRouteFromHash() {
    currentRoute = normalizeHash(window.location.hash);
  }

  function linkClass(target) {
    const isActive = currentRoute === target;
    return `menu-link menu-link-primary ${isActive ? "menu-link-active" : ""}`;
  }

  function adminLinkClass(target) {
    const isActive = currentRoute === target;
    return `menu-link menu-link-admin ${isActive ? "menu-link-active" : ""}`;
  }

  async function openOrphansModal(event) {
    event.preventDefault();
    orphanInvoker = event.currentTarget ?? document.activeElement;

    orphanStatus = "loading";
    orphanChecked = 0;
    orphanFound = 0;
    orphanError = "";
    orphanModalOpen = true;

    await tick();
    orphanModalContainer?.focus();

    // Stage 1 UI-only status simulation; backend wiring comes later.
    setTimeout(async () => {
      try {
        orphanChecked = 42;
        orphanFound = 3;
        orphanStatus = "done";
      } catch (error) {
        orphanStatus = "error";
        orphanError = String(error);
      }
      await tick();
      orphanPrimaryButton?.focus() || orphanCloseButton?.focus();
    }, 800);
  }

  function closeOrphansModal() {
    if (!orphanCanDismiss) {
      return;
    }
    orphanModalOpen = false;
    orphanInvoker?.focus();
  }

  function viewOrphans() {
    navigateTo("#/admin/orphans");
    closeOrphansModal();
  }

  function navigateTo(route) {
    window.location.hash = route;
    syncRouteFromHash();
  }

  function handleModalKeydown(event) {
    if (event.key === "Tab") {
      const focusable = Array.from(
        orphanModalContainer?.querySelectorAll(
          'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
        ) || []
      ).filter((element) => !element.hasAttribute("hidden") && element.offsetParent !== null);

      if (focusable.length === 0) {
        event.preventDefault();
        orphanModalContainer?.focus();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const active = document.activeElement;

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
      return;
    }

    if (event.key === "Escape") {
      if (orphanCanDismiss) {
        closeOrphansModal();
      } else {
        event.preventDefault();
      }
    }
  }

  async function loadBrowseItems() {
    browseLoading = true;
    browseError = "";

    try {
      const result = await getBrowseDesigns();
      browseItems = Array.isArray(result.items) ? result.items : [];
      browseSource = result.source || "mock";
    } catch (error) {
      browseError = `Could not load designs: ${error}`;
      browseItems = [];
      browseSource = "mock";
    } finally {
      browseLoading = false;
      browseHasLoaded = true;
    }
  }

  async function loadBrowseProjects() {
    try {
      const result = await getBrowseProjects();
      browseProjects = Array.isArray(result.items) ? result.items : [];
      browseProjectsSource = result.source || "mock";
    } catch (error) {
      browseProjects = [];
      browseProjectsSource = "mock";
      console.info("Could not load browse projects", error);
    }
  }

  async function loadBrowseTags() {
    try {
      const result = await getBrowseTags();
      browseTagOptions = Array.isArray(result.items) ? result.items : [];
      browseTagsSource = result.source || "mock";
    } catch (error) {
      browseTagOptions = [];
      browseTagsSource = "mock";
      console.info("Could not load browse tags", error);
    }
  }

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
      if (requestId !== browsePreviewRequestCounter) {
        return;
      }

      const map = { ...browsePreviewById };
      const returnedIds = new Set();
      for (const item of result.items || []) {
        if (Number.isFinite(Number(item?.id))) {
          returnedIds.add(Number(item.id));
          map[Number(item.id)] = item?.data_url || null;
        }
      }

      // Mark any missing IDs with no returned preview as null so we don't refetch endlessly.
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

  async function runImportPreview() {
    importLoading = true;
    importError = "";
    importActionMessage = "";
    importActionNeedsSkipHoopsConfirm = false;

    try {
      const result = await previewImportFromRoots(getActiveImportRoots());
      importPreview = result.preview || null;
      importPreviewSource = result.source || "mock";
      importPreviewMessage = result.message || "Preview complete.";
      importSelectedFiles = Array.isArray(importPreview?.scanned_files)
        ? importPreview.scanned_files.map((file) => String(file?.full_path || "")).filter(Boolean)
        : [];
      importPerFolderAssignmentByPath = {};
      syncImportPerFolderAssignments();
      importPrecheck = null;
      importPrecheckSource = "mock";
      importPrecheckMessage = "Run precheck after selecting files.";
      importContextToken = "";
      importStep = importPreview && importSelectedFiles.length >= 0 ? 2 : 1;
    } catch (error) {
      importError = `Import preview failed: ${error}`;
      importPreview = null;
      importPreviewSource = "mock";
      importSelectedFiles = [];
      importPerFolderAssignmentByPath = {};
      importPrecheck = null;
      importContextToken = "";
      importStep = 1;
    } finally {
      importLoading = false;
    }
  }

  async function browseImportRootPath() {
    if (importBrowseLoading || importLoading || importActionLoading) {
      return;
    }

    importBrowseLoading = true;
    importError = "";

    try {
      const result = await browseImportFolder(importRootPath);
      const selectedPaths = Array.isArray(result?.paths)
        ? result.paths.map((value) => String(value || "").trim()).filter(Boolean)
        : [];

      if (selectedPaths.length > 0) {
        for (const path of selectedPaths) {
          addImportRootPath(path);
        }
        importRootPath = selectedPaths[0];
        await persistImportLastBrowseFolder(selectedPaths[0]);
      } else {
        const selectedPath = String(result?.path || "").trim();
        if (selectedPath) {
          addImportRootPath(selectedPath);
          importRootPath = selectedPath;
          await persistImportLastBrowseFolder(selectedPath);
        }
      }

      importPreviewMessage = result?.message || "Folder selection complete.";
    } catch (error) {
      importError = `Folder browse failed: ${error}`;
    } finally {
      importBrowseLoading = false;
    }
  }

  function normalizeImportRootPath(value) {
    const trimmed = String(value || "").trim();
    if (!trimmed) {
      return "";
    }

    const slashNormalized = trimmed.replace(/\\/g, "/");
    const isUncPath = slashNormalized.startsWith("//");
    const compacted = isUncPath
      ? `//${slashNormalized.slice(2).replace(/\/{2,}/g, "/")}`
      : slashNormalized.replace(/\/{2,}/g, "/");

    const withoutTrailingSlash = compacted.replace(/\/+$/g, "");
    if (!withoutTrailingSlash) {
      return compacted;
    }

    // Preserve drive root shape like C:/.
    if (/^[a-zA-Z]:$/.test(withoutTrailingSlash)) {
      return `${withoutTrailingSlash}/`;
    }

    return withoutTrailingSlash;
  }

  function getActiveImportRoots() {
    const roots = importRootPaths
      .map((value) => normalizeImportRootPath(value))
      .filter(Boolean);

    if (roots.length > 0) {
      return roots;
    }

    const single = normalizeImportRootPath(importRootPath);
    return single ? [single] : [];
  }

  function addImportRootPath(path = importRootPath) {
    const next = normalizeImportRootPath(path);
    if (!next) {
      return;
    }

    const existingByLower = new Set(importRootPaths.map((item) => String(item || "").toLowerCase()));
    if (!existingByLower.has(next.toLowerCase())) {
      importRootPaths = [...importRootPaths, next];
    }

    importRootPath = next;
  }

  function removeImportRootPath(path) {
    const target = normalizeImportRootPath(path).toLowerCase();
    importRootPaths = importRootPaths.filter((value) => String(value || "").toLowerCase() !== target);
  }

  function clearImportRootPaths() {
    importRootPaths = [];
  }

  async function addCurrentImportRootPath() {
    const normalized = normalizeImportRootPath(importRootPath);
    if (!normalized) {
      return;
    }

    addImportRootPath(normalized);
    await persistImportLastBrowseFolder(normalized);
  }

  function resetImportWizard() {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }

    importProgressStatus = "";
    importProgressToken = "";
    importBrowseLoading = false;
    importRootPaths = [];
    importHasAppliedSavedRoot = false;
    importStep = 1;
    importPreview = null;
    importPreviewSource = "mock";
    importPreviewMessage = "Import preview has not run yet.";
    importPrecheck = null;
    importPrecheckSource = "mock";
    importPrecheckMessage = "Run precheck after selecting files.";
    importSelectedFiles = [];
    importContextToken = "";
    importActionMessage = "";
    importActionSource = "mock";
    importActionNeedsSkipHoopsConfirm = false;
    importGlobalDesignerId = "";
    importGlobalSourceId = "";
    importPerFolderAssignmentByPath = {};
    importError = "";
  }

  async function stopImportProgressUpdates() {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }

    importProgressStatus = "";
    importProgressToken = "";
  }

  async function startImportProgressUpdates(contextToken) {
    const normalizedToken = String(contextToken || "").trim();
    if (!normalizedToken) {
      return;
    }

    await stopImportProgressUpdates();
    importProgressToken = normalizedToken;

    try {
      const { listen } = await import("@tauri-apps/api/event");
      importProgressUnlisten = await listen("bulk-import-progress", (event) => {
        const payload = event?.payload || {};
        const payloadToken = String(payload?.context_token || "").trim();
        if (payloadToken && payloadToken !== importProgressToken) {
          return;
        }

        const processed = Number(payload?.processed_count || 0);
        const total = Number(payload?.total_count || 0);
        const persisted = Number(payload?.persisted_count || 0);

        if (total > 0) {
          importProgressStatus = `${processed}/${total} processed (${persisted} imported)`;
        } else {
          importProgressStatus = `${persisted} imported`;
        }
      });
    } catch (error) {
      console.info("Bulk import progress events unavailable.", error);
    }
  }

  onDestroy(() => {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }
  });

  function getFolderPathFromFilePath(fullPath) {
    const value = String(fullPath || "").trim();
    if (!value) {
      return "";
    }

    const normalized = value.replace(/\\/g, "/");
    const splitIndex = normalized.lastIndexOf("/");
    if (splitIndex <= 0) {
      return "";
    }

    return normalized.slice(0, splitIndex);
  }

  function syncImportPerFolderAssignments() {
    const folderPaths = new Set(
      importSelectedFiles
        .map((fullPath) => getFolderPathFromFilePath(fullPath))
        .filter(Boolean)
    );

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

  function setImportFolderDesigner(folderPath, designerId) {
    const key = String(folderPath || "").trim();
    if (!key) {
      return;
    }

    importPerFolderAssignmentByPath = {
      ...importPerFolderAssignmentByPath,
      [key]: {
        ...(importPerFolderAssignmentByPath?.[key] || { designerId: "", sourceId: "" }),
        designerId: String(designerId || ""),
      },
    };
  }

  function setImportFolderSource(folderPath, sourceId) {
    const key = String(folderPath || "").trim();
    if (!key) {
      return;
    }

    importPerFolderAssignmentByPath = {
      ...importPerFolderAssignmentByPath,
      [key]: {
        ...(importPerFolderAssignmentByPath?.[key] || { designerId: "", sourceId: "" }),
        sourceId: String(sourceId || ""),
      },
    };
  }

  function getImportFolderDesigner(folderPath) {
    return String(importPerFolderAssignmentByPath?.[folderPath]?.designerId || "");
  }

  function getImportFolderSource(folderPath) {
    return String(importPerFolderAssignmentByPath?.[folderPath]?.sourceId || "");
  }

  function toggleImportFile(fullPath, checked) {
    const value = String(fullPath || "").trim();
    if (!value) {
      return;
    }

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
      ? importPreview.scanned_files.map((file) => String(file?.full_path || "")).filter(Boolean)
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
        if (!folderPath) {
          continue;
        }
        counts.set(folderPath, (counts.get(folderPath) || 0) + 1);
      }

      return Array.from(counts.entries())
        .map(([folderPath, selectedCount]) => ({ folderPath, selectedCount }))
        .sort((left, right) => left.folderPath.localeCompare(right.folderPath, undefined, { sensitivity: "base" }));
    })()
  );

  async function loadImportReferenceData(force = false) {
    if (importReferenceLoading && !force) {
      return;
    }

    if (!force && importDesigners.length > 0 && importSources.length > 0) {
      return;
    }

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

  function buildImportConfirmWire() {
    const perFolderAssignments = importSelectedFolderSummaries.map((folder) => ({
      folder_path: folder.folderPath,
      designer_id: getImportFolderDesigner(folder.folderPath)
        ? Number(getImportFolderDesigner(folder.folderPath))
        : null,
      source_id: getImportFolderSource(folder.folderPath)
        ? Number(getImportFolderSource(folder.folderPath))
        : null,
      inferred_designer_id: null,
      inferred_source_id: null,
    }));

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
      importContextToken = String(importPrecheck?.context_token || "");
      importStep = importPrecheck ? 3 : 2;
    } catch (error) {
      importError = `Import precheck failed: ${error}`;
      importPrecheck = null;
      importContextToken = "";
      importStep = 2;
    } finally {
      importLoading = false;
    }
  }

  function mapServerImportRouteToHash(nextRoute) {
    const route = String(nextRoute || "").toLowerCase();
    if (route.startsWith("/designs")) {
      return "#/designs";
    }
    if (route.startsWith("/import")) {
      return "#/import";
    }
    if (route.startsWith("/admin/tags")) {
      return "#/admin/tags";
    }
    if (route.startsWith("/admin/hoops")) {
      return "#/admin/hoops";
    }
    if (route.startsWith("/admin/sources")) {
      return "#/admin/sources";
    }
    if (route.startsWith("/admin/designers")) {
      return "#/admin/designers";
    }
    return null;
  }

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
      importProgressStatus = "";
      await startImportProgressUpdates(importContextToken);
    }

    try {
      const result = await runPrecheckAction({
        contextToken: importContextToken,
        action,
        confirmSkipHoops,
      });

      const actionResult = result.actionResult || null;
      importActionSource = result.source || "mock";
      importActionMessage = result.message || "Import precheck action complete.";
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
      importActionInProgress = "";
      importActionLoading = false;
    }
  }

  function openDesignDetail(item) {
    if (item?.id == null) {
      return;
    }
    navigateTo(`#/designs/${item.id}`);
  }

  async function loadDesignDetail(designId) {
    if (designId == null) {
      return;
    }

    detailLoading = true;
    detailError = "";

    try {
      const result = await getDesignDetail(designId);
      detailItem = result.item || null;
      detailSource = result.source || "mock";
    } catch (error) {
      detailError = `Could not load design detail: ${error}`;
      detailItem = null;
      detailSource = "mock";
    } finally {
      detailLoading = false;
    }
  }

  function tokenize(value) {
    return String(value || "")
      .toLowerCase()
      .split(/\s+/)
      .map((part) => part.trim())
      .filter(Boolean);
  }

  function wildcardToRegExp(pattern) {
    const escaped = String(pattern || "")
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*/g, ".*")
      .replace(/\?/g, ".");
    return new RegExp(`^${escaped}$`, "i");
  }

  function filenameMatchesPattern(filename, pattern) {
    const value = String(filename || "");
    const normalized = String(pattern || "").trim();
    if (!normalized) {
      return true;
    }
    if (normalized.includes("*") || normalized.includes("?")) {
      return wildcardToRegExp(normalized).test(value);
    }
    return value.toLowerCase().includes(normalized.toLowerCase());
  }

  function normalizeCardItem(item, index) {
    const id = Number(item?.id ?? index + 1);
    return {
      id,
      filename: String(item?.filename || item?.name || `design-${id}.pes`),
      designer: String(item?.designer || "Unknown"),
      source: String(item?.source || "Unknown"),
      tags: Array.isArray(item?.tags) ? item.tags.map(String) : [],
      hoop: item?.hoop ? String(item.hoop) : "",
      rating:
        item?.rating == null || Number.isNaN(Number(item.rating))
          ? null
          : Math.max(0, Math.min(5, Number(item.rating))),
      isStitched: Boolean(item?.is_stitched),
      tagsChecked: item?.tags_checked !== false,
    };
  }

  function compareBrowseItems(a, b) {
    const sign = browseFilters.sortDir === "desc" ? -1 : 1;

    if (browseFilters.sortBy === "folder") {
      const bySource = a.source.localeCompare(b.source, undefined, { sensitivity: "base" });
      if (bySource !== 0) {
        return bySource * sign;
      }
    }

    if (browseFilters.sortBy === "date_added") {
      const ad = a.id;
      const bd = b.id;
      return (ad - bd) * sign;
    }

    return a.filename.localeCompare(b.filename, undefined, { sensitivity: "base" }) * sign;
  }

  function hasActiveBrowseFilters() {
    const d = defaultBrowseFilters();
    return Object.keys(d).some((key) => {
      const left = browseFilters[key];
      const right = d[key];
      if (Array.isArray(left) && Array.isArray(right)) {
        return left.length !== right.length || left.some((v, i) => v !== right[i]);
      }
      return left !== right;
    });
  }

  function toggleAdditionalFilters() {
    browseAdditionalFiltersOpen = !browseAdditionalFiltersOpen;
  }

  function applyBrowseFilters() {
    browseCurrentPage = 1;
  }

  function resetBrowseFilters() {
    browseFilters = defaultBrowseFilters();
    browseCurrentPage = 1;
  }

  function updateBrowseFilter(key, value) {
    browseFilters = { ...browseFilters, [key]: value };
  }

  function updateBrowseTagFilter(tag, enabled) {
    const active = new Set(browseFilters.tagFilters || []);
    if (enabled) {
      active.add(tag);
    } else {
      active.delete(tag);
    }
    browseFilters = { ...browseFilters, tagFilters: Array.from(active) };
    browseCurrentPage = 1;
  }

  function estimateBrowseColumnsFromWidth(width) {
    const normalizedWidth = Number(width) || 0;
    if (normalizedWidth <= 0) {
      return 1;
    }

    return Math.max(
      1,
      Math.floor((normalizedWidth + BROWSE_CARD_GAP) / (BROWSE_CARD_MIN_WIDTH + BROWSE_CARD_GAP))
    );
  }

  function refreshBrowseGridColumns() {
    const containerWidth = browseGridContainer?.clientWidth;
    if (containerWidth && containerWidth > 0) {
      browseGridColumns = estimateBrowseColumnsFromWidth(containerWidth);
      return;
    }

    if (typeof window !== "undefined") {
      browseGridColumns = estimateBrowseColumnsFromWidth(window.innerWidth || 0);
      return;
    }

    browseGridColumns = 1;
  }

  function clearBrowseSelection() {
    browseSelectedIds = [];
  }

  function toggleBrowseSelectAllVisible(checked) {
    const visibleIds = browsePageItems.map((item) => item.id);
    if (checked) {
      browseSelectedIds = Array.from(new Set([...browseSelectedIds, ...visibleIds]));
      return;
    }
    browseSelectedIds = browseSelectedIds.filter((id) => !visibleIds.includes(id));
  }

  function toggleBrowseCardSelection(id, checked) {
    if (checked) {
      browseSelectedIds = Array.from(new Set([...browseSelectedIds, id]));
      return;
    }
    browseSelectedIds = browseSelectedIds.filter((current) => current !== id);
  }

  function openBulkTagModal() {
    if (browseSelectedIds.length === 0) {
      return;
    }
    browseBulkTagSelection = [];
    browseBulkModalOpen = true;
  }

  function closeBulkTagModal() {
    browseBulkModalOpen = false;
  }

  async function applyBulkTags() {
    if (browseSelectedIds.length === 0) {
      closeBulkTagModal();
      return;
    }

    const wantsUntagged = browseBulkTagSelection.includes(BROWSE_TAG_UNTAGGED);
    const selectedTagIds = wantsUntagged
      ? []
      : browseBulkTagSelection
          .map((value) => Number(value))
          .filter((value) => Number.isFinite(value) && value > 0);

    const selectedTagNames = selectedTagIds
      .map((tagId) => browseTagOptions.find((option) => Number(option.id) === tagId)?.description)
      .filter(Boolean);

    const result = await bulkSetTagsForDesigns(browseSelectedIds, selectedTagIds);

    browseItems = browseItems.map((item) => {
      if (!browseSelectedIds.includes(item.id)) {
        return item;
      }
      const tags = wantsUntagged ? [] : selectedTagNames;
      return {
        ...item,
        tags,
        tags_checked: true,
      };
    });

    if (result.persisted) {
      browseActionNotice = `Applied tags to ${result.updated_count} design(s) (saved in Rust backend).`;
      closeBulkTagModal();
      await loadBrowseItems();
      return;
    }

    browseActionNotice = `Applied tags to ${result.updated_count} design(s) (UI fallback only).`;
    closeBulkTagModal();
  }

  async function verifySelectedBrowseItems() {
    if (browseSelectedIds.length === 0) {
      return;
    }

    const result = await bulkVerifyDesigns(browseSelectedIds);

    browseItems = browseItems.map((item) =>
      browseSelectedIds.includes(item.id) ? { ...item, tags_checked: true } : item
    );

    if (result.persisted) {
      browseActionNotice = `Marked ${result.verified_count} design(s) as verified (saved in Rust backend).`;
      await loadBrowseItems();
      return;
    }

    browseActionNotice = `Marked ${result.verified_count} design(s) as verified (UI fallback only).`;
  }

  async function addSelectedToProject() {
    const projectId = Number(browseBulkProject);
    if (browseSelectedIds.length === 0 || !Number.isFinite(projectId) || projectId <= 0) {
      return;
    }

    const projectName =
      browseProjects.find((project) => Number(project.id) === projectId)?.name ||
      `project ${projectId}`;

    const result = await bulkAddDesignsToProject(projectId, browseSelectedIds);
    if (result.persisted) {
      browseActionNotice = `Added ${result.added_count} design(s) to ${projectName} (saved in Rust backend).`;
      return;
    }

    browseActionNotice = `Added ${result.added_count} design(s) to ${projectName} (UI fallback only).`;
  }

  function browseStars(value) {
    const score = Math.max(0, Math.min(5, Number(value || 0)));
    return "★".repeat(score) + "☆".repeat(5 - score);
  }

  let browseCardItems = $derived(browseItems.map((item, index) => normalizeCardItem(item, index)));
  let browseAvailableTags = $derived(
    Array.from(new Set(browseCardItems.flatMap((item) => item.tags))).sort((a, b) => a.localeCompare(b))
  );
  let browseAvailableDesigners = $derived(
    Array.from(new Set(browseCardItems.map((item) => item.designer))).sort((a, b) => a.localeCompare(b))
  );
  let browseAvailableSources = $derived(
    Array.from(new Set(browseCardItems.map((item) => item.source))).sort((a, b) => a.localeCompare(b))
  );
  let browseAvailableHoops = $derived(
    Array.from(new Set(browseCardItems.map((item) => item.hoop).filter(Boolean))).sort((a, b) => a.localeCompare(b))
  );

  let browseFilteredItems = $derived(
    browseCardItems
      .filter((item) => {
        const filename = item.filename.toLowerCase();
        const tagsText = item.tags.join(" ").toLowerCase();
        const sourceText = item.source.toLowerCase();
        const combinedText = [filename, tagsText, sourceText].join(" ");

        if (browseFilters.unverifiedOnly && item.tagsChecked) {
          return false;
        }

        if (browseFilters.q) {
          const q = browseFilters.q.toLowerCase().trim();
          const allowed = [
            browseFilters.searchFilename ? filename : "",
            browseFilters.searchTags ? tagsText : "",
            browseFilters.searchFolder ? sourceText : "",
          ].join(" ");
          if (!allowed.includes(q)) {
            return false;
          }
        }

        if (browseFilters.allWords) {
          const allWords = tokenize(browseFilters.allWords);
          if (!allWords.every((word) => combinedText.includes(word))) {
            return false;
          }
        }

        if (browseFilters.exactPhrase) {
          if (!combinedText.includes(String(browseFilters.exactPhrase).toLowerCase())) {
            return false;
          }
        }

        if (browseFilters.anyWords) {
          const anyWords = tokenize(browseFilters.anyWords);
          if (anyWords.length > 0 && !anyWords.some((word) => combinedText.includes(word))) {
            return false;
          }
        }

        if (browseFilters.noneWords) {
          const noneWords = tokenize(browseFilters.noneWords);
          if (noneWords.some((word) => combinedText.includes(word))) {
            return false;
          }
        }

        if (!filenameMatchesPattern(item.filename, browseFilters.filename)) {
          return false;
        }

        if (browseFilters.designer && item.designer !== browseFilters.designer) {
          return false;
        }

        if (browseFilters.hoop && item.hoop !== browseFilters.hoop) {
          return false;
        }

        if (browseFilters.source && item.source !== browseFilters.source) {
          return false;
        }

        if (browseFilters.rating !== "") {
          const requested = Number(browseFilters.rating);
          if (Number(item.rating || 0) < requested) {
            return false;
          }
        }

        if (browseFilters.stitched === "true" && !item.isStitched) {
          return false;
        }

        if (browseFilters.stitched === "false" && item.isStitched) {
          return false;
        }

        if (browseFilters.tagFilters.length > 0) {
          const wantsUntagged = browseFilters.tagFilters.includes(BROWSE_TAG_UNTAGGED);
          const explicit = browseFilters.tagFilters.filter((tag) => tag !== BROWSE_TAG_UNTAGGED);
          const hasExplicit = explicit.length === 0 || explicit.some((tag) => item.tags.includes(tag));
          const hasUntagged = wantsUntagged && item.tags.length === 0;
          if (!hasExplicit && !hasUntagged) {
            return false;
          }
        }

        return true;
      })
      .sort(compareBrowseItems)
  );

  let browsePageSize = $derived(Math.max(1, browseGridColumns * BROWSE_PAGE_ROWS));
  let browseTotal = $derived(browseFilteredItems.length);
  let browseTotalPages = $derived(Math.max(1, Math.ceil(browseTotal / browsePageSize)));

  let browsePageItems = $derived(
    browseFilteredItems.slice(
      (browseCurrentPage - 1) * browsePageSize,
      browseCurrentPage * browsePageSize
    )
  );

  let browseAllVisibleSelected = $derived(
    browsePageItems.length > 0 && browsePageItems.every((item) => browseSelectedIds.includes(item.id))
  );

  let browseSelectedCount = $derived(browseSelectedIds.length);

  let browsePageNumbers = $derived(
    (() => {
      const start = Math.max(1, browseCurrentPage - 2);
      const end = Math.min(browseTotalPages, browseCurrentPage + 2);
      const pages = [];
      for (let page = start; page <= end; page += 1) {
        pages.push(page);
      }
      return pages;
    })()
  );

  $effect(() => {
    if (currentRoute === "#/designs" && !browseHasLoaded && !browseLoading) {
      loadBrowseItems();
    }
  });

  $effect(() => {
    if (currentRoute !== "#/designs") {
      browseHasLoaded = false;
    }
  });

  $effect(() => {
    if (currentRoute === "#/import") {
      loadImportReferenceData();
      if (!settingsLoaded && !settingsLoading) {
        loadSettingsFromBackend();
      }
    }
  });

  $effect(() => {
    if (currentRoute !== "#/import") {
      importHasAppliedSavedRoot = false;
      return;
    }

    if (!settingsLoaded || importHasAppliedSavedRoot) {
      return;
    }

    const savedRoot = normalizeImportRootPath(settingsImportLastBrowseFolder);
    if (savedRoot) {
      importRootPath = savedRoot;
      if (importRootPaths.length === 0) {
        addImportRootPath(savedRoot);
      }
    }

    importHasAppliedSavedRoot = true;
  });

  $effect(() => {
    if (currentRoute !== "#/designs") {
      return;
    }

    browseTotal;
    tick().then(() => {
      refreshBrowseGridColumns();
    });
  });

  $effect(() => {
    if (currentRoute === "#/designs" && browseProjects.length === 0) {
      loadBrowseProjects();
    }
  });

  $effect(() => {
    if (currentRoute === "#/designs" && browseTagOptions.length === 0) {
      loadBrowseTags();
    }
  });

  $effect(() => {
    if (currentRoute !== "#/designs") {
      return;
    }
    const ids = browsePageItems.map((item) => item.id);
    loadBrowsePreviews(ids);
  });

  $effect(() => {
    if (detailDesignId !== null) {
      loadDesignDetail(detailDesignId);
    }
  });

  $effect(() => {
    if (currentRoute === "#/admin/settings" && !settingsLoaded && !settingsLoading) {
      loadSettingsFromBackend();
    }
  });

  $effect(() => {
    if (currentRoute === "#/admin/maintenance/backup" && !settingsLoaded && !settingsLoading) {
      loadSettingsFromBackend();
    }
  });

  $effect(() => {
    if (currentUiKind === "admin-list") {
      loadAdminDataForCurrentRoute();
    }
  });

  $effect(() => {
    if (currentUiKind !== "admin-list") {
      adminNotice = "";
      adminNoticeType = "info";
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

  syncRouteFromHash();
</script>

<svelte:window onhashchange={syncRouteFromHash} onresize={refreshBrowseGridColumns} />

<nav class="menu-shell text-white shadow">
  <div class="menu-shell-inner max-w-7xl mx-auto px-3 py-2 flex items-center gap-x-4 overflow-x-auto">
    <div class="flex items-center gap-x-4 min-w-max">
      <a href="#/designs" class="menu-brand">
        <span class="menu-brand-mark" aria-hidden="true"></span>
        <span>Embroidery Catalogue</span>
      </a>
      <a href="#/designs" class={linkClass("#/designs")}>Browse</a>
      <a href="#/import" class={linkClass("#/import")}>Import</a>
      <a href="#/projects" class={linkClass("#/projects")}>Projects</a>
      <a href="#/help" class={linkClass("#/help")}>Help</a>
    </div>

    <div class="flex items-center gap-x-3 ml-auto min-w-max">
      <span class="menu-admin-label" aria-hidden="true">Admin:</span>
      <a href="#/admin/designers" class={adminLinkClass("#/admin/designers")}>Designers</a>
      <a href="#/admin/tags" class={adminLinkClass("#/admin/tags")}>Tags</a>
      <a href="#/admin/sources" class={adminLinkClass("#/admin/sources")}>Sources</a>
      <a href="#/admin/hoops" class={adminLinkClass("#/admin/hoops")}>Hoops</a>
      <a href="#/admin/settings" class={adminLinkClass("#/admin/settings")}>Settings</a>
      <a href="#/admin/maintenance/backup" class={adminLinkClass("#/admin/maintenance/backup")}>Backup</a>
      <a href="#/admin/tagging-actions" class={adminLinkClass("#/admin/tagging-actions")}>Tagging Actions</a>
      <a
        href="#/admin/orphans"
        class={adminLinkClass("#/admin/orphans")}
        onclick={openOrphansModal}
      >
        Orphans
      </a>
    </div>
  </div>
</nav>

<main class="max-w-7xl mx-auto px-4 py-6">
  {#if currentUiKind === "browse"}
    <section class="space-y-4">
      <h1 class="text-3xl font-bold text-gray-800">Browse Designs</h1>

      <form
        class="bg-white rounded-lg shadow p-4 space-y-4 no-print"
        onsubmit={(event) => {
          event.preventDefault();
          applyBrowseFilters();
        }}
      >
        <div class="border rounded-lg bg-white p-4 space-y-2">
          <label class="block text-sm font-medium text-gray-700" for="browse-q">General search</label>
          <div class="flex flex-wrap items-center gap-3">
            <input
              id="browse-q"
              class="border rounded px-3 py-1.5 text-sm flex-1 min-w-[20rem] font-mono"
              placeholder='e.g. rose "cross stitch" -applique or *.hus'
              value={browseFilters.q}
              oninput={(event) => updateBrowseFilter("q", event.currentTarget.value)}
            />
            <label class="flex items-center gap-1.5 cursor-pointer text-sm text-gray-700 select-none whitespace-nowrap">
              <input
                type="checkbox"
                checked={browseFilters.unverifiedOnly}
                onchange={(event) => updateBrowseFilter("unverifiedOnly", event.currentTarget.checked)}
              />
              Unverified only
            </label>
          </div>
          <p class="text-xs text-gray-400 mt-0.5">
            Supports Google-like syntax:
            <code class="bg-gray-100 px-1 rounded">"exact phrase"</code>
            ·
            <code class="bg-gray-100 px-1 rounded">-exclude</code>
            ·
            <code class="bg-gray-100 px-1 rounded">word1 OR word2</code>
            ·
            <code class="bg-gray-100 px-1 rounded">*.hus</code>
            ·
            <a href="#/help" class="text-indigo-500 hover:underline">Search help</a>
          </p>
        </div>

        <details class="border rounded-lg bg-white overflow-visible relative" open={browseAdditionalFiltersOpen}>
          <summary
            class="px-4 py-3 cursor-pointer bg-gray-50 hover:bg-gray-100 font-semibold text-gray-800"
            onclick={(event) => {
              event.preventDefault();
              toggleAdditionalFilters();
            }}
          >
            Additional filters
          </summary>

          {#if browseAdditionalFiltersOpen}
            <div class="p-4 space-y-4">
              <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-4 gap-3">
                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">All words</span>
                  <input
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.allWords}
                    oninput={(event) => updateBrowseFilter("allWords", event.currentTarget.value)}
                  />
                </label>
                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Exact phrase</span>
                  <input
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.exactPhrase}
                    oninput={(event) => updateBrowseFilter("exactPhrase", event.currentTarget.value)}
                  />
                </label>
                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Any words</span>
                  <input
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.anyWords}
                    oninput={(event) => updateBrowseFilter("anyWords", event.currentTarget.value)}
                  />
                </label>
                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">None words</span>
                  <input
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.noneWords}
                    oninput={(event) => updateBrowseFilter("noneWords", event.currentTarget.value)}
                  />
                </label>
              </div>

              <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-4 gap-3 items-end">
                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Filename</span>
                  <input
                    class="border rounded px-3 py-1.5 text-sm w-full font-mono"
                    placeholder="e.g. rose* or *.jef"
                    value={browseFilters.filename}
                    oninput={(event) => updateBrowseFilter("filename", event.currentTarget.value)}
                  />
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Designer</span>
                  <select
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.designer}
                    onchange={(event) => updateBrowseFilter("designer", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    {#each browseAvailableDesigners as designer}
                      <option value={designer}>{designer}</option>
                    {/each}
                  </select>
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Hoop</span>
                  <select
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.hoop}
                    onchange={(event) => updateBrowseFilter("hoop", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    {#each browseAvailableHoops as hoop}
                      <option value={hoop}>{hoop}</option>
                    {/each}
                  </select>
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Source</span>
                  <select
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.source}
                    onchange={(event) => updateBrowseFilter("source", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    {#each browseAvailableSources as source}
                      <option value={source}>{source}</option>
                    {/each}
                  </select>
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Tags</span>
                  <div class="border rounded px-3 py-2 text-sm space-y-1 max-h-28 overflow-auto">
                    <label class="flex items-center gap-1.5">
                      <input
                        type="checkbox"
                        checked={browseFilters.tagFilters.includes(BROWSE_TAG_UNTAGGED)}
                        onchange={(event) => updateBrowseTagFilter(BROWSE_TAG_UNTAGGED, event.currentTarget.checked)}
                      />
                      Untagged
                    </label>
                    {#each browseAvailableTags as tag}
                      <label class="flex items-center gap-1.5">
                        <input
                          type="checkbox"
                          checked={browseFilters.tagFilters.includes(tag)}
                          onchange={(event) => updateBrowseTagFilter(tag, event.currentTarget.checked)}
                        />
                        <span>{tag}</span>
                      </label>
                    {/each}
                  </div>
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Min Rating</span>
                  <select
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.rating}
                    onchange={(event) => updateBrowseFilter("rating", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    <option value="1">1★</option>
                    <option value="2">2★</option>
                    <option value="3">3★</option>
                    <option value="4">4★</option>
                    <option value="5">5★</option>
                  </select>
                </label>

                <label class="text-sm text-gray-700">
                  <span class="block font-medium mb-1">Stitched</span>
                  <select
                    class="border rounded px-3 py-1.5 text-sm w-full"
                    value={browseFilters.stitched}
                    onchange={(event) => updateBrowseFilter("stitched", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    <option value="true">Yes</option>
                    <option value="false">No</option>
                  </select>
                </label>
              </div>
            </div>
          {/if}
        </details>

        <div class="border rounded-lg bg-white p-4 flex flex-wrap gap-4 items-start text-gray-700">
          <div>
            <p class="text-sm font-medium text-gray-700 mb-1">Search in</p>
            <div class="flex min-h-[38px] flex-wrap items-center gap-4">
              <label class="flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  checked={browseFilters.searchFilename}
                  onchange={(event) => updateBrowseFilter("searchFilename", event.currentTarget.checked)}
                />
                File name
              </label>
              <label class="flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  checked={browseFilters.searchTags}
                  onchange={(event) => updateBrowseFilter("searchTags", event.currentTarget.checked)}
                />
                Tags
              </label>
              <label class="flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  checked={browseFilters.searchFolder}
                  onchange={(event) => updateBrowseFilter("searchFolder", event.currentTarget.checked)}
                />
                Folder name
              </label>
            </div>
          </div>

          <label class="text-sm text-gray-700">
            <span class="block font-medium mb-1">Sort by</span>
            <select
              class="border rounded px-3 py-1.5 text-sm bg-white text-gray-800"
              value={browseFilters.sortBy}
              onchange={(event) => updateBrowseFilter("sortBy", event.currentTarget.value)}
            >
              <option value="name">Design name</option>
              <option value="folder">Folder</option>
              <option value="date_added">Date added</option>
            </select>
          </label>

          <label class="text-sm text-gray-700">
            <span class="block font-medium mb-1">Order</span>
            <select
              class="border rounded px-3 py-1.5 text-sm bg-white text-gray-800"
              value={browseFilters.sortDir}
              onchange={(event) => updateBrowseFilter("sortDir", event.currentTarget.value)}
            >
              <option value="asc">▲ Ascending</option>
              <option value="desc">▼ Descending</option>
            </select>
          </label>

          <div class="ml-auto flex gap-3 items-center self-end">
            <button type="submit" class="menu-button-primary">Search</button>
            <button
              type="button"
              class="menu-button-primary disabled:opacity-50"
              disabled={!hasActiveBrowseFilters()}
              aria-disabled={!hasActiveBrowseFilters()}
              onclick={resetBrowseFilters}
            >
              Reset
            </button>
            <button type="button" class="menu-button-secondary" onclick={loadBrowseItems}>Refresh</button>
          </div>
        </div>
      </form>

      <div class="flex items-center justify-between text-sm text-gray-600">
        <span>{browseTotal} design{browseTotal === 1 ? "" : "s"} found — page {browseCurrentPage} of {browseTotalPages}</span>
        {#if browsePageItems.length > 0}
          <label class="flex items-center gap-1 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={browseAllVisibleSelected}
              onchange={(event) => toggleBrowseSelectAllVisible(event.currentTarget.checked)}
            />
            Select all
          </label>
        {/if}
      </div>

      {#if browseLoading}
        <div class="bg-white rounded-lg shadow p-6 text-gray-700">Loading designs...</div>
      {:else if browseError}
        <div class="bg-white rounded-lg shadow p-6 text-red-600">{browseError}</div>
      {:else if browsePageItems.length === 0}
        <p class="text-gray-500">
          No designs found. Try adjusting your filters or
          <a href="#/import" class="text-indigo-600 hover:underline">import some files</a>.
        </p>
      {:else}
        <div class="browse-card-grid gap-4" bind:this={browseGridContainer}>
          {#each browsePageItems as item}
            <article class="bg-white rounded shadow hover:shadow-md overflow-hidden flex flex-col relative border border-gray-100">
              <label class="absolute top-1 left-1 z-10 cursor-pointer bg-white/90 rounded px-1">
                <span class="sr-only">Select {item.filename}</span>
                <input
                  type="checkbox"
                  checked={browseSelectedIds.includes(item.id)}
                  onchange={(event) => toggleBrowseCardSelection(item.id, event.currentTarget.checked)}
                />
              </label>

              <button class="w-full text-left" onclick={() => openDesignDetail(item)}>
                {#if browsePreviewById[item.id]}
                  <img
                    src={browsePreviewById[item.id]}
                    alt={item.filename}
                    class="w-full h-36 object-contain bg-gray-100 p-1"
                    loading="lazy"
                  />
                {:else}
                  <div class="w-full h-36 bg-gray-100 p-1 flex items-center justify-center text-xs text-gray-400">
                    {browsePreviewsLoading ? "Loading image..." : "No image"}
                  </div>
                {/if}
                <div class="p-2 flex-1 space-y-1">
                  <div class="flex items-center justify-between gap-2">
                    <p class="text-sm font-medium leading-tight break-words">{item.filename}</p>
                    {#if !item.tagsChecked}
                      <span class="font-bold text-xl leading-none self-center" style="color: #dc2626;" aria-label="Unverified">×</span>
                    {/if}
                  </div>
                  {#if item.hoop}
                    <p class="text-xs text-indigo-600">{item.hoop}</p>
                  {/if}
                  {#if item.tags.length > 0}
                    <div class="flex flex-wrap gap-1 pt-0.5">
                      {#each item.tags as tag}
                        <span class="text-[11px] leading-4 px-1.5 py-0.5 rounded bg-purple-100 text-purple-700">{tag}</span>
                      {/each}
                    </div>
                  {/if}
                  <p class="text-xs text-gray-400" aria-label={`Rating ${item.rating ?? 0} out of 5`}>
                    {browseStars(item.rating ?? 0)}
                  </p>
                </div>
              </button>
            </article>
          {/each}
        </div>
      {/if}

      {#if browseTotalPages > 1}
        <nav class="flex items-center gap-2 mt-2 flex-wrap" aria-label="Browse pagination">
          {#if browseCurrentPage > 1}
            <button
              class="px-3 py-1 rounded border text-sm hover:bg-gray-100"
              onclick={() => {
                browseCurrentPage -= 1;
              }}
            >
              ‹ Prev
            </button>
          {/if}

          {#each browsePageNumbers as pageNumber}
            <button
              class={`px-3 py-1 rounded border text-sm ${pageNumber === browseCurrentPage ? "bg-indigo-600 text-white border-indigo-600" : "hover:bg-gray-100"}`}
              onclick={() => {
                browseCurrentPage = pageNumber;
              }}
              aria-current={pageNumber === browseCurrentPage ? "page" : undefined}
            >
              {pageNumber}
            </button>
          {/each}

          {#if browseCurrentPage < browseTotalPages}
            <button
              class="px-3 py-1 rounded border text-sm hover:bg-gray-100"
              onclick={() => {
                browseCurrentPage += 1;
              }}
            >
              Next ›
            </button>
          {/if}
        </nav>
      {/if}

      {#if browseActionNotice}
        <p class="text-sm text-indigo-700 bg-indigo-50 border border-indigo-200 rounded px-3 py-2">{browseActionNotice}</p>
      {/if}

      {#if browseSelectedCount > 0}
        <div class="fixed bottom-0 left-0 right-0 bg-indigo-700 text-white shadow-lg z-50 px-4 py-3 flex flex-wrap items-center gap-3 no-print">
          <span class="font-semibold text-sm whitespace-nowrap">{browseSelectedCount} selected</span>

          <button type="button" class="bg-white text-indigo-700 font-semibold px-4 py-1.5 rounded text-sm hover:bg-indigo-100" onclick={openBulkTagModal}>
            Choose tags...
          </button>

          <button type="button" class="bg-green-500 text-white font-semibold px-4 py-1.5 rounded text-sm hover:bg-green-400" onclick={verifySelectedBrowseItems}>
            Verify selected
          </button>

          <div class="flex items-center gap-2 min-w-0">
            <select class="border border-white/40 bg-indigo-600 rounded px-2 py-1 text-sm" bind:value={browseBulkProject}>
              <option value="">Select project</option>
              {#each browseProjects as project}
                <option value={String(project.id)}>{project.name}</option>
              {/each}
            </select>
            <button type="button" class="bg-white text-indigo-700 font-semibold px-3 py-1.5 rounded text-sm hover:bg-indigo-100" onclick={addSelectedToProject}>
              Add to project
            </button>
          </div>

          <button type="button" class="bg-indigo-600 border border-indigo-400 px-3 py-1.5 rounded text-sm hover:bg-indigo-500" onclick={clearBrowseSelection}>
            Clear selection
          </button>
        </div>
      {/if}

      {#if browseBulkModalOpen}
        <div class="fixed inset-0 z-[60] flex items-center justify-center no-print" role="dialog" aria-modal="true" aria-labelledby="bulk-tag-title">
          <button type="button" class="absolute inset-0 bg-black/50" aria-label="Close tag chooser" onclick={closeBulkTagModal}></button>
          <div class="relative bg-white rounded-lg shadow-2xl w-full max-w-2xl mx-4 flex flex-col max-h-[85vh]">
            <div class="flex items-center justify-between px-5 py-4 border-b">
              <h2 id="bulk-tag-title" class="text-lg font-semibold text-gray-800">Choose tags for selected designs</h2>
              <button type="button" class="menu-button-secondary" onclick={closeBulkTagModal}>Close</button>
            </div>
            <div class="overflow-y-auto px-5 py-4 flex-1">
              <p class="text-sm text-gray-600 mb-3">{browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.</p>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                <label class="flex items-center gap-2 text-sm">
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
                  Untagged
                </label>
                {#each browseTagOptions as tagOption}
                  <label class="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={browseBulkTagSelection.includes(tagOption.id)}
                      onchange={(event) => {
                        if (event.currentTarget.checked) {
                          browseBulkTagSelection = Array.from(new Set([...browseBulkTagSelection, tagOption.id]));
                        } else {
                          browseBulkTagSelection = browseBulkTagSelection.filter((value) => value !== tagOption.id);
                        }
                      }}
                    />
                    {tagOption.description}
                  </label>
                {/each}
              </div>
            </div>
            <div class="px-5 py-3 border-t flex items-center gap-3 justify-end">
              <button type="button" class="menu-button-secondary" onclick={closeBulkTagModal}>Cancel</button>
              <button type="button" class="menu-button-primary" onclick={applyBulkTags}>Apply tags</button>
            </div>
          </div>
        </div>
      {/if}

      <p class="text-xs text-gray-500">Data source: designs {browseSource}, previews {browsePreviewsSource}, projects {browseProjectsSource}, tags {browseTagsSource}</p>
    </section>
  {:else if currentUiKind === "settings"}
    <section class="settings-page space-y-6">
      <h1 class="settings-title text-2xl font-bold text-gray-800 mb-6">Application Settings</h1>

      {#if settingsSaveState === "saved"}
        <div class="settings-alert settings-alert-success bg-green-50 border border-green-300 text-green-800 rounded px-4 py-2 text-sm">
          {settingsSaveMessage || "Settings saved successfully."}
        </div>
      {:else if settingsSaveState === "error"}
        <div class="settings-alert settings-alert-error bg-red-50 border border-red-300 text-red-800 rounded px-4 py-2 text-sm">
          {settingsSaveMessage || "Settings could not be saved."}
        </div>
      {/if}

      <div class="settings-layout max-w-3xl space-y-6">
        {#if settingsLoading && !settingsLoaded}
          <div class="settings-alert settings-alert-info bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-2 text-sm">
            Loading settings...
          </div>
        {/if}

        <form class="settings-card settings-form bg-white rounded shadow p-6 space-y-5" onsubmit={saveSettingsFromBackend}>
          <div>
            <h2 class="text-sm font-semibold text-gray-700 mb-1">Image preview preference</h2>
            <p class="text-sm text-gray-600 mb-3">
              Choose whether new imports generate <strong>2D</strong> (fast flat preview) or
              <strong>3D</strong> (detailed stitch-simulated preview) images by default.
              You can override this per import session on the precheck page.
              3D rendering is slower but produces more realistic previews.
            </p>

            <div class="space-y-2">
              <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
                <input
                  type="radio"
                  name="settings-image-preference"
                  value="2d"
                  checked={settingsImagePreference === "2d"}
                  class="text-indigo-600 focus:ring-indigo-500"
                  onchange={() => {
                    settingsImagePreference = "2d";
                  }}
                />
                <strong>2D</strong> — Fast flat preview (default, recommended for bulk imports)
              </label>

              <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
                <input
                  type="radio"
                  name="settings-image-preference"
                  value="3d"
                  checked={settingsImagePreference === "3d"}
                  class="text-indigo-600 focus:ring-indigo-500"
                  onchange={() => {
                    settingsImagePreference = "3d";
                  }}
                />
                <strong>3D</strong> — Detailed stitch-simulated preview (slower, more realistic)
              </label>
            </div>

            <p class="mt-2 text-xs text-gray-500">
              Current setting: <strong>{settingsImagePreference === "3d" ? "3D" : "2D"}</strong>.
              This affects the import pipeline only. Existing designs keep their current image type.
            </p>
          </div>

          <div>
            <h2 class="text-sm font-semibold text-gray-700 mb-1">Google Gemini API key</h2>
            <p class="text-sm text-gray-600">
              The Google API key is only required if you want your designs to be tagged automatically by Google AI.
              <a href={settingsHelpUrl} class="text-indigo-600 hover:underline">Press here for more information.</a>
            </p>
          </div>

          <div>
            <label for="settings-google-api-key" class="block text-sm font-semibold text-gray-700 mb-1">API key</label>
            <div class="flex items-center gap-2">
              <input
                id="settings-google-api-key"
                type={settingsApiKeyRevealed ? "text" : "password"}
                bind:value={settingsGoogleApiKey}
                placeholder="AIzaSy..."
                autocomplete="off"
                spellcheck="false"
                class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
              />
              <button
                type="button"
                class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
                aria-label="Show or hide API key"
                aria-pressed={settingsApiKeyRevealed}
                title={settingsApiKeyRevealed ? "Hide API key" : "Show API key"}
                onclick={toggleSettingsApiKeyVisibility}
              >
                <span aria-hidden="true" class="settings-eye-icon">{settingsApiKeyRevealed ? "🙈" : "👁"}</span>
              </button>
            </div>
            <p class="mt-2 text-xs text-gray-500">
              {#if settingsHasGoogleApiKey}
                A key is currently saved in <code>.env</code>. You can leave it as-is or replace it here.
              {:else}
                Leave this blank if you only want keyword-based tagging with no Google AI calls.
              {/if}
            </p>
          </div>

          <div class="border-t pt-4">
            <h2 class="text-sm font-semibold text-gray-700 mb-1">AI tagging during import</h2>
            <p class="text-sm text-gray-600 mb-3">
              Control whether Gemini AI tagging runs automatically when you import designs.
              Tier 1 (keyword matching) always runs and is free.
              Tiers 2 and 3 call the Google Gemini API and require an API key.
            </p>

            {#if settingsHasGoogleApiKey}
              <div class="bg-amber-50 border border-amber-300 rounded p-3 mb-3 text-sm text-amber-900">
                <strong>⚠ Cost notice:</strong> Gemini usage may incur charges on your Google account.
                Free-tier limits are approximately <strong>15 requests per minute</strong> and
                <strong>1,500 requests per day</strong>.
                A historical estimate from February 2026 found that Tier 3 on 4,000 images cost about
                <strong>$0.33 on the paid tier</strong>; actual pricing may have changed.
                Check the latest rates at
                <a href="https://ai.google.dev/pricing" class="underline" target="_blank" rel="noopener">ai.google.dev/pricing</a>.
              </div>
            {:else}
              <div class="bg-blue-50 border border-blue-200 rounded p-3 mb-3 text-sm text-blue-900">
                No API key is saved. AI tagging options below will have no effect until you add a key above.
              </div>
            {/if}

            <div class="space-y-2">
              <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
                <input type="checkbox" bind:checked={settingsAiTier2Auto} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
                Run <strong>Tier 2</strong> (Gemini text AI from filename) automatically during import
              </label>
              <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
                <input type="checkbox" bind:checked={settingsAiTier3Auto} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
                Run <strong>Tier 3</strong> (Gemini vision AI from preview image) automatically during import
              </label>
            </div>
          </div>

          <div>
            <label for="settings-ai-batch-size" class="block text-sm font-semibold text-gray-700 mb-1">
              AI tagging batch size <span class="font-normal text-gray-500">(optional)</span>
            </label>
            <input
              id="settings-ai-batch-size"
              type="number"
              min="1"
              bind:value={settingsAiBatchSize}
              placeholder="e.g. 100 — leave blank to tag all"
              class="settings-input border rounded px-3 py-2 text-sm w-48"
            />
            <p class="mt-1 text-xs text-gray-500">
              Limit AI tagging to this many newly imported designs per import run.
              Leave blank to tag all newly imported designs.
              Useful for very large imports where you want to spread Gemini calls over several runs.
            </p>
          </div>

          <div>
            <label for="settings-ai-delay" class="block text-sm font-semibold text-gray-700 mb-1">
              Delay between Gemini calls (seconds) <span class="font-normal text-gray-500">(optional)</span>
            </label>
            <input
              id="settings-ai-delay"
              type="number"
              min="0"
              step="0.5"
              bind:value={settingsAiDelay}
              placeholder="e.g. 6.0 — leave blank for default (5.0 s)"
              class="settings-input border rounded px-3 py-2 text-sm w-56"
            />
            <p class="mt-1 text-xs text-gray-500">
              Seconds to wait between API calls. Increase this if you see <em>429 Too Many Requests</em> errors.
              Default is 5.0 seconds. Also applies to batch tagging actions on the
              <a href="#/admin/tagging-actions" class="text-indigo-600 hover:underline">Tagging Actions</a> page.
            </p>
          </div>

          <div>
            <label for="settings-import-commit-batch-size" class="block text-sm font-semibold text-gray-700 mb-1">
              Import database commit batch size <span class="font-normal text-gray-500">(optional)</span>
            </label>
            <input
              id="settings-import-commit-batch-size"
              type="number"
              min="1"
              bind:value={settingsImportCommitBatchSize}
              placeholder="e.g. 10 — leave blank for default"
              class="settings-input border rounded px-3 py-2 text-sm w-56"
            />
            <p class="mt-1 text-xs text-gray-500">
              Controls how many designs are written or tag-updated before each database commit during import.
              Leave blank to use the default batch size of 10.
              Lower values reduce rollback size on failure; higher values reduce commit overhead.
            </p>
          </div>

          <div class="border-t pt-4 space-y-3">
            <h2 class="text-sm font-semibold text-gray-700 mb-1">Catalogue storage</h2>
            <p class="text-sm text-gray-600">
              Large catalogue data lives under a single home folder.
              {#if settingsCanConfigureDataRoot}
                For desktop installs you can point this to a larger drive. Changes apply after restarting the app, and any missing managed files are copied into the new location automatically.
              {:else}
                In {settingsAppMode} mode this location follows the application folder automatically.
              {/if}
            </p>

            {#if settingsCanConfigureDataRoot}
              <div>
                <label for="settings-data-root" class="block text-sm font-semibold text-gray-700 mb-1">Catalogue data location</label>
                <div class="flex items-center gap-2">
                  <input
                    id="settings-data-root"
                    type="text"
                    bind:value={settingsDataRoot}
                    placeholder="D:\\EmbroideryCatalogueData"
                    spellcheck="false"
                    class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
                  />
                  <button
                    type="button"
                    class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
                    onclick={browseDataRootFromBackend}
                  >
                    Browse…
                  </button>
                </div>
                <p class="mt-1 text-xs text-gray-500">
                  This home folder contains both the catalogue database and the managed design library.
                </p>
              </div>
            {/if}
          </div>

          <div class="flex items-center justify-between gap-3">
            <p class="text-xs text-gray-500">These settings are stored in the catalogue database for this installation.</p>
            <button type="submit" class="settings-primary-button menu-button-primary" disabled={settingsSaveState === "saving"}>
              {settingsSaveState === "saving" ? "Saving..." : "Save settings"}
            </button>
          </div>
        </form>

        <div class="settings-card settings-meta bg-white rounded shadow p-6 space-y-5">
          <div>
            <h2 class="text-sm font-semibold text-gray-700 mb-1">Storage locations</h2>
            <p class="text-sm text-gray-600">
              The catalogue database and imported embroidery files live under the catalogue data location shown below.
              Logs are stored separately so they survive data moves.
            </p>
          </div>

          <div>
            <p class="block text-sm font-semibold text-gray-700 mb-1">Catalogue data location</p>
            <code class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all">{settingsDataRoot}</code>
          </div>

          <div>
            <p class="block text-sm font-semibold text-gray-700 mb-1">Log folder</p>
            <code class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all">{settingsLogFolder}</code>
          </div>
        </div>
      </div>
    </section>
  {:else if currentUiKind === "backup"}
    <section class="backup-page space-y-4">
      <h1 class="backup-title text-2xl font-bold text-gray-800 mb-2">Backup</h1>
      <p class="text-sm text-gray-500 mb-4">
        Back up your catalogue database and embroidery design files to folders of your choice.
        The database backup saves your catalogue data, settings, tags, and projects.
        The designs backup saves the actual embroidery files.
      </p>

      <div class="backup-important mb-2 bg-amber-50 border border-amber-300 text-amber-900 rounded px-4 py-3 text-sm space-y-1">
        <p class="font-semibold">Important</p>
        <p>
          The database and embroidery designs are backed up <strong>separately</strong>.
          To make a <strong>complete backup</strong> of your catalogue, make sure you run <strong>both</strong> backups.
        </p>
      </div>

      {#if backupStatus === "saved"}
        <div class="settings-alert bg-green-50 border border-green-300 text-green-800 rounded px-4 py-2 text-sm">
          {backupMessage || "Backup destinations saved."}
        </div>
      {:else if backupStatus === "error"}
        <div class="settings-alert bg-red-50 border border-red-300 text-red-800 rounded px-4 py-2 text-sm">
          {backupMessage || "Backup action could not be completed."}
        </div>
      {/if}

      <div class="settings-layout max-w-3xl space-y-6">
        <form class="settings-card backup-card bg-white rounded shadow p-6 space-y-5" onsubmit={saveBackupDestinationsUiOnly}>
          <h2 class="text-base font-semibold text-gray-800">Backup Destinations</h2>
          <p class="text-sm text-gray-600">Set separate destination folders for the database and designs backups.</p>

          <div>
            <label for="backup-db-destination" class="block text-sm font-semibold text-gray-700 mb-1">Database backup folder</label>
            <div class="flex gap-2">
              <input
                id="backup-db-destination"
                type="text"
                bind:value={backupDbDestination}
                placeholder="e.g. C:\\Backups\\EmbroideryDB"
                spellcheck="false"
                class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
              />
              <button type="button" class="settings-secondary-button border rounded px-3 py-2 text-sm whitespace-nowrap" onclick={() => browseBackupDestinationUiOnly("database")}>
                Browse…
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500">
              This backup contains your catalogue data only - <strong>not</strong> the embroidery design files.
            </p>
          </div>

          <div>
            <label for="backup-designs-destination" class="block text-sm font-semibold text-gray-700 mb-1">Designs backup folder</label>
            <div class="flex gap-2">
              <input
                id="backup-designs-destination"
                type="text"
                bind:value={backupDesignsDestination}
                placeholder="e.g. C:\\Backups\\EmbroideryDesigns"
                spellcheck="false"
                class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
              />
              <button type="button" class="settings-secondary-button border rounded px-3 py-2 text-sm whitespace-nowrap" onclick={() => browseBackupDestinationUiOnly("designs")}>
                Browse…
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500">
              This backup contains file copies only - <strong>not</strong> the catalogue database.
            </p>
          </div>

          <div class="flex justify-end">
            <button
              type="submit"
              class="settings-primary-button menu-button-primary"
              disabled={!backupHasUnsavedChanges}
              title={!backupHasUnsavedChanges ? "No unsaved destination changes" : undefined}
            >
              Save destinations
            </button>
          </div>
        </form>

        <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
          <h2 class="text-base font-semibold text-gray-800">Database Backup</h2>
          <p class="text-sm text-gray-600">Creates a full copy of the live database file with a timestamp in the filename.</p>
          <div class="text-xs text-gray-500 space-y-0.5">
            <p>Source: <code class="settings-code inline-block border rounded px-2 py-1 font-mono">{backupDbSourcePath}</code></p>
            <p>Saved destination folder: <code class="settings-code inline-block border rounded px-2 py-1 font-mono">{backupSavedDbDestination || "(not set)"}</code></p>
          </div>
          <button
            type="button"
            class="settings-primary-button menu-button-primary"
            disabled={!backupHasDbDestination}
            title={!backupHasDbDestination ? "Set a database backup destination first" : undefined}
            onclick={() => runBackupActionUiOnly("database")}
          >
            Backup database now
          </button>
        </div>

        <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
          <h2 class="text-base font-semibold text-gray-800">Designs Backup</h2>
          <p class="text-sm text-gray-600">
            Runs an incremental mirror backup of the designs folder. Only new or changed files are copied; unchanged files are skipped.
          </p>
          <div class="text-xs text-gray-500 space-y-0.5">
            <p>Source: <code class="settings-code inline-block border rounded px-2 py-1 font-mono">{backupDesignsSourcePath}</code></p>
            <p>Saved destination folder: <code class="settings-code inline-block border rounded px-2 py-1 font-mono">{backupSavedDesignsDestination || "(not set)"}</code></p>
          </div>
          <button
            type="button"
            class="settings-primary-button menu-button-primary"
            disabled={!backupHasDesignsDestination}
            title={!backupHasDesignsDestination ? "Set a designs backup destination first" : undefined}
            onclick={() => runBackupActionUiOnly("designs")}
          >
            Run incremental backup
          </button>
        </div>

        <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
          <h2 class="text-base font-semibold text-gray-800">Backup Both</h2>
          <p class="text-sm text-gray-600">Run the database backup and the incremental designs backup in one step.</p>
          <button
            type="button"
            class="settings-primary-button menu-button-primary"
            disabled={!backupHasDbDestination || !backupHasDesignsDestination}
            title={!backupHasDbDestination || !backupHasDesignsDestination ? "Set both backup destinations first" : undefined}
            onclick={() => runBackupActionUiOnly("both")}
          >
            Run both backups
          </button>
        </div>
      </div>
    </section>
  {:else if currentPage}
    <div class="bg-white rounded-xl shadow p-6 space-y-4">
      <h1 class="text-2xl font-bold text-gray-800">{currentPage.title}</h1>
      <p class="text-sm uppercase tracking-wide text-indigo-600 font-semibold">{currentPage.subtitle}</p>
      <p class="text-gray-600">{currentPage.description}</p>

      <div class="bg-indigo-50 border border-indigo-200 rounded-lg p-4 text-sm text-indigo-800 space-y-2">
        <p class="font-semibold">Current Stage: Route-backed placeholders</p>
        <p>{currentPage.cta}</p>
        <p>Active route: <span class="font-semibold">{currentRoute}</span></p>
      </div>

      {#if currentUiKind === "design-detail"}
        <div class="space-y-3">
          <div class="flex flex-wrap gap-2">
            <button class="menu-button-secondary" onclick={() => navigateTo("#/designs")}>Back to Browse</button>
          </div>

          <div class="route-panel">
            <p class="font-semibold">Data source: {detailSource}</p>
            {#if detailLoading}
              <p>Loading design detail...</p>
            {:else if detailError}
              <p class="text-red-600">{detailError}</p>
            {:else if !detailItem}
              <p>No design found for id {detailDesignId}.</p>
            {:else}
              <div class="grid sm:grid-cols-2 gap-3 mt-3">
                <div class="route-card"><strong>Filename:</strong> {detailItem.filename || "Unknown"}</div>
                <div class="route-card"><strong>Designer:</strong> {detailItem.designer || "Unknown"}</div>
                <div class="route-card"><strong>Source:</strong> {detailItem.source || "Unknown"}</div>
                <div class="route-card"><strong>Path:</strong> {detailItem.filepath || "Unknown"}</div>
                <div class="route-card"><strong>Rating:</strong> {detailItem.rating ?? "None"}</div>
                <div class="route-card"><strong>Date added:</strong> {detailItem.date_added || "Unknown"}</div>
              </div>
              <div class="route-panel mt-3">
                <p class="font-semibold mb-1">Notes</p>
                <p class="text-sm text-gray-700">{detailItem.notes || "No notes yet."}</p>
              </div>
            {/if}
          </div>
        </div>
      {:else if currentUiKind === "import"}
        <div class="space-y-3">
          <div class="grid md:grid-cols-3 gap-3">
            <div class="route-card" class:bg-indigo-50={importStep === 1}>Step 1: Select folders</div>
            <div class="route-card" class:bg-indigo-50={importStep === 2}>Step 2: Review files</div>
            <div class="route-card" class:bg-indigo-50={importStep === 3}>Step 3: Precheck actions</div>
          </div>

          <div class="route-panel space-y-3">
            <label for="import-root-path" class="text-sm font-semibold text-gray-700">Folder path</label>
            <div class="flex flex-col sm:flex-row gap-2">
              <input
                id="import-root-path"
                class="flex-1 rounded border border-gray-300 px-3 py-2"
                bind:value={importRootPath}
                placeholder="C:/imports"
                disabled={importLoading || importActionLoading || importBrowseLoading}
              />
              <button
                class="menu-button-secondary"
                onclick={addCurrentImportRootPath}
                disabled={importLoading || importActionLoading || importBrowseLoading || !String(importRootPath || "").trim()}
              >
                Add Root
              </button>
              <button
                class="menu-button-secondary"
                onclick={browseImportRootPath}
                disabled={importLoading || importActionLoading || importBrowseLoading}
              >
                {importBrowseLoading ? "Browsing..." : "Browse"}
              </button>
              <button class="menu-button-primary" onclick={runImportPreview} disabled={importLoading || importBrowseLoading}>
                {importLoading ? "Running..." : "Run Preview"}
              </button>
              <button class="menu-button-secondary" onclick={resetImportWizard} disabled={importLoading || importActionLoading || importBrowseLoading}>
                Reset
              </button>
            </div>

            {#if importRootPaths.length > 0}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <p class="text-sm font-semibold text-gray-700">Selected roots ({importRootPaths.length})</p>
                  <button
                    class="menu-button-secondary"
                    onclick={clearImportRootPaths}
                    disabled={importLoading || importActionLoading || importBrowseLoading}
                  >
                    Clear roots
                  </button>
                </div>
                <div class="space-y-1 max-h-40 overflow-auto border border-gray-200 rounded p-2">
                  {#each importRootPaths as rootPath}
                    <div class="flex items-center justify-between gap-2 text-sm">
                      <span class="text-gray-700 break-all">{rootPath}</span>
                      <button
                        class="menu-button-secondary"
                        onclick={() => removeImportRootPath(rootPath)}
                        disabled={importLoading || importActionLoading || importBrowseLoading}
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            {#if importError}
              <p class="text-sm text-red-600">{importError}</p>
            {:else}
              <p class="text-sm text-gray-700">{importPreviewMessage}</p>
            {/if}

            {#if importPreview}
              <div class="grid sm:grid-cols-3 gap-3 text-sm">
                <div class="route-card">Source: {importPreviewSource}</div>
                <div class="route-card">Discovered: {importPreview.discovered_count ?? 0}</div>
                <div class="route-card">Folders: {importPreview.folder_count ?? 0}</div>
              </div>
            {/if}
          </div>

          {#if importPreview}
            <div class="route-panel space-y-3">
              <div class="flex flex-wrap items-center gap-2 justify-between">
                <p class="text-sm font-semibold text-gray-700">
                  Step 2: Select files ({importSelectedFiles.length} selected)
                </p>
                <div class="flex gap-2">
                  <button class="menu-button-secondary" onclick={selectAllImportFiles} disabled={importLoading || importActionLoading}>
                    Select all
                  </button>
                  <button class="menu-button-secondary" onclick={clearImportFileSelection} disabled={importLoading || importActionLoading}>
                    Clear
                  </button>
                </div>
              </div>

              <div class="grid sm:grid-cols-2 gap-2 text-sm">
                <label class="flex flex-col gap-1">
                  <span class="text-gray-600">Global designer (optional)</span>
                  <select class="rounded border border-gray-300 px-2 py-1" bind:value={importGlobalDesignerId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                    <option value="">Inferred / none</option>
                    {#each importDesigners as designer}
                      <option value={String(designer.id)}>{designer.name}</option>
                    {/each}
                  </select>
                </label>
                <label class="flex flex-col gap-1">
                  <span class="text-gray-600">Global source (optional)</span>
                  <select class="rounded border border-gray-300 px-2 py-1" bind:value={importGlobalSourceId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                    <option value="">Inferred / none</option>
                    {#each importSources as source}
                      <option value={String(source.id)}>{source.name}</option>
                    {/each}
                  </select>
                </label>
              </div>

              {#if importSelectedFolderSummaries.length > 0}
                <div class="space-y-2">
                  <p class="text-sm font-semibold text-gray-700">Per-folder overrides</p>
                  <div class="space-y-2">
                    {#each importSelectedFolderSummaries as folder}
                      <div class="route-card space-y-2">
                        <div class="text-sm text-gray-700 break-all">
                          <strong>{folder.folderPath}</strong>
                          <span class="text-gray-500"> ({folder.selectedCount} selected)</span>
                        </div>
                        <div class="grid sm:grid-cols-2 gap-2 text-sm">
                          <label class="flex flex-col gap-1">
                            <span class="text-gray-600">Designer override</span>
                            <select
                              class="rounded border border-gray-300 px-2 py-1"
                              value={getImportFolderDesigner(folder.folderPath)}
                              onchange={(event) => setImportFolderDesigner(folder.folderPath, event.currentTarget.value)}
                              disabled={importReferenceLoading || importLoading || importActionLoading}
                            >
                              <option value="">Use global / inferred</option>
                              {#each importDesigners as designer}
                                <option value={String(designer.id)}>{designer.name}</option>
                              {/each}
                            </select>
                          </label>
                          <label class="flex flex-col gap-1">
                            <span class="text-gray-600">Source override</span>
                            <select
                              class="rounded border border-gray-300 px-2 py-1"
                              value={getImportFolderSource(folder.folderPath)}
                              onchange={(event) => setImportFolderSource(folder.folderPath, event.currentTarget.value)}
                              disabled={importReferenceLoading || importLoading || importActionLoading}
                            >
                              <option value="">Use global / inferred</option>
                              {#each importSources as source}
                                <option value={String(source.id)}>{source.name}</option>
                              {/each}
                            </select>
                          </label>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if Array.isArray(importPreview.scanned_files) && importPreview.scanned_files.length > 0}
                <div class="max-h-64 overflow-auto border border-gray-200 rounded">
                  <table class="w-full text-sm">
                    <thead class="bg-gray-50 sticky top-0">
                      <tr>
                        <th class="px-3 py-2 text-left">Import</th>
                        <th class="px-3 py-2 text-left">File</th>
                        <th class="px-3 py-2 text-left">Ext</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each importPreview.scanned_files as file}
                        <tr class="border-t border-gray-100">
                          <td class="px-3 py-2">
                            <input
                              type="checkbox"
                              checked={importSelectedFiles.includes(String(file?.full_path || ""))}
                              onchange={(event) => toggleImportFile(file?.full_path, event.currentTarget.checked)}
                              disabled={importLoading || importActionLoading}
                            />
                          </td>
                          <td class="px-3 py-2 break-all">{file?.full_path || "Unknown file"}</td>
                          <td class="px-3 py-2">{file?.extension || "-"}</td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {:else}
                <p class="text-sm text-gray-600">No supported files discovered in this preview.</p>
              {/if}

              <div class="flex justify-end">
                <button class="menu-button-primary" onclick={runImportPrecheck} disabled={importLoading || importActionLoading || importSelectedFiles.length === 0}>
                  Continue to precheck
                </button>
              </div>
            </div>
          {/if}

          {#if importPrecheck}
            <div class="route-panel space-y-3">
              <p class="text-sm font-semibold text-gray-700">Step 3: Precheck actions</p>
              <p class="text-sm text-gray-700">{importPrecheckMessage}</p>
              <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-2 text-sm">
                <div class="route-card">Source: {importPrecheckSource}</div>
                <div class="route-card">First import: {importPrecheck.is_first_import ? "Yes" : "No"}</div>
                <div class="route-card">Needs hoop setup: {importPrecheck.needs_hoop_setup ? "Yes" : "No"}</div>
                <div class="route-card">Token active: {importContextToken ? "Yes" : "No"}</div>
              </div>

              <div class="flex flex-wrap gap-2">
                <button class="menu-button-secondary" onclick={() => executeImportPrecheckAction("review_hoops")} disabled={importActionLoading || !importContextToken}>Review Hoops</button>
                <button class="menu-button-secondary" onclick={() => executeImportPrecheckAction("review_tags")} disabled={importActionLoading || !importContextToken}>Review Tags</button>
                <button class="menu-button-secondary" onclick={() => executeImportPrecheckAction("review_sources")} disabled={importActionLoading || !importContextToken}>Review Sources</button>
                <button class="menu-button-secondary" onclick={() => executeImportPrecheckAction("review_designers")} disabled={importActionLoading || !importContextToken}>Review Designers</button>
                <button class="menu-button-primary" onclick={() => executeImportPrecheckAction("import_now")} disabled={importActionLoading || !importContextToken}>
                  {#if importActionLoading && importActionInProgress === "import_now"}
                    {#if importProgressStatus}
                      Running Import... {importProgressStatus}
                    {:else}
                      Running Import...
                    {/if}
                  {:else}
                    Import now
                  {/if}
                </button>
                <button class="menu-button-secondary" onclick={() => executeImportPrecheckAction("cancel")} disabled={importActionLoading || !importContextToken}>Cancel</button>
              </div>

              {#if importActionNeedsSkipHoopsConfirm}
                <div class="route-card space-y-2">
                  <p class="text-sm text-amber-800">
                    Hoops are not configured for a first import. Confirm to continue anyway.
                  </p>
                  <button class="menu-button-primary" onclick={() => executeImportPrecheckAction("import_now", true)} disabled={importActionLoading || !importContextToken}>
                    Confirm import without hoop setup
                  </button>
                </div>
              {/if}

              {#if importActionMessage}
                <p class="text-sm text-gray-700">{importActionMessage} ({importActionSource})</p>
              {/if}
            </div>
          {/if}
        </div>
      {:else if currentUiKind === "projects"}
        <div class="space-y-3">
          <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
            <div class="route-card">Project card placeholder 1</div>
            <div class="route-card">Project card placeholder 2</div>
            <div class="route-card">Project card placeholder 3</div>
          </div>
          <div class="route-panel">
            Selected project detail panel placeholder.
          </div>
        </div>
      {:else if currentUiKind === "help"}
        <div class="space-y-2">
          <div class="route-card">Getting started guide placeholder</div>
          <div class="route-card">Import workflow help placeholder</div>
          <div class="route-card">Troubleshooting links placeholder</div>
        </div>
      {:else if currentUiKind === "admin-list"}
        <section class="admin-page space-y-4">
          {#if adminNotice}
            <div
              class="admin-alert rounded px-4 py-2 text-sm border"
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
            <h1 class="admin-title text-2xl font-bold text-gray-800">Manage Designers</h1>
            <p class="text-sm text-gray-500">
              Designers are the creators or brands of embroidery designs. Use this list to keep designer names consistent.
            </p>

            <div class="admin-card bg-white rounded shadow p-4 max-w-xl">
              <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new designer</h2>
              <form class="flex gap-2" onsubmit={addDesigner}>
                <input
                  type="text"
                  bind:value={newDesignerName}
                  required
                  placeholder="New designer name..."
                  class="admin-input flex-1 border rounded px-3 py-2 text-sm"
                />
                <button type="submit" class="settings-primary-button text-sm">Add</button>
              </form>
            </div>

            <div class="admin-card bg-white rounded shadow overflow-hidden max-w-2xl">
              <table class="w-full text-sm">
                <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                  <tr>
                    <th class="px-4 py-2 text-left">Name</th>
                    <th class="px-4 py-2"></th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">
                  {#if designers.length === 0}
                    <tr>
                      <td colspan="2" class="px-4 py-3 text-gray-400">No designers yet.</td>
                    </tr>
                  {:else}
                    {#each designers as designer}
                      <tr>
                        <td class="px-4 py-2">{designer.name}</td>
                        <td class="px-4 py-2 text-right">
                          <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => deleteDesigner(designer.id)}>
                            Delete
                          </button>
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>
          {:else if adminIsTagsRoute}
            <h1 class="admin-title text-2xl font-bold text-gray-800">Manage Tags</h1>
            <p class="text-sm text-gray-500">
              Use Image tags for subject categories and Stitching tags for technique or style.
            </p>

            <div class="admin-card bg-white rounded shadow p-4 max-w-3xl">
              <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new tag</h2>
              <form class="flex flex-wrap gap-2 items-end" onsubmit={addTag}>
                <div>
                  <label for="admin-tag-description" class="block text-xs font-medium text-gray-600 mb-1">Description</label>
                  <input
                    id="admin-tag-description"
                    type="text"
                    bind:value={newTagDescription}
                    required
                    placeholder="e.g. Animals, Cross stitch..."
                    class="admin-input border rounded px-3 py-2 text-sm w-56"
                  />
                </div>
                <div>
                  <label for="admin-tag-group" class="block text-xs font-medium text-gray-600 mb-1">Group</label>
                  <select id="admin-tag-group" bind:value={newTagGroup} class="admin-input border rounded px-3 py-2 text-sm">
                    <option value="image">Image</option>
                    <option value="stitching">Stitching</option>
                  </select>
                </div>
                <button type="submit" class="settings-primary-button text-sm">Add</button>
              </form>
            </div>

            <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl">
              <div class="bg-green-50 border-b border-green-200 px-4 py-2">
                <h2 class="text-sm font-semibold text-green-800 uppercase tracking-wide">Image Tags</h2>
              </div>
              <table class="w-full text-sm">
                <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                  <tr>
                    <th class="px-4 py-2 text-left">Description</th>
                    <th class="px-4 py-2 text-left">Group</th>
                    <th class="px-4 py-2"></th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">
                  {#if imageTags.length === 0}
                    <tr><td colspan="3" class="px-4 py-3 text-gray-400">No image tags yet.</td></tr>
                  {:else}
                    {#each imageTags as tag}
                      <tr>
                        <td class="px-4 py-2">{tag.description}</td>
                        <td class="px-4 py-2">
                          <select
                            value={tag.tagGroup}
                            class="admin-input border rounded px-2 py-1 text-xs"
                            onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                          >
                            <option value="">Unclassified</option>
                            <option value="image">Image</option>
                            <option value="stitching">Stitching</option>
                          </select>
                        </td>
                        <td class="px-4 py-2 text-right">
                          <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => deleteTag(tag.id)}>Delete</button>
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>

            <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl">
              <div class="bg-blue-50 border-b border-blue-200 px-4 py-2">
                <h2 class="text-sm font-semibold text-blue-800 uppercase tracking-wide">Stitching Tags</h2>
              </div>
              <table class="w-full text-sm">
                <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                  <tr>
                    <th class="px-4 py-2 text-left">Description</th>
                    <th class="px-4 py-2 text-left">Group</th>
                    <th class="px-4 py-2"></th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">
                  {#if stitchingTags.length === 0}
                    <tr><td colspan="3" class="px-4 py-3 text-gray-400">No stitching tags yet.</td></tr>
                  {:else}
                    {#each stitchingTags as tag}
                      <tr>
                        <td class="px-4 py-2">{tag.description}</td>
                        <td class="px-4 py-2">
                          <select
                            value={tag.tagGroup}
                            class="admin-input border rounded px-2 py-1 text-xs"
                            onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                          >
                            <option value="">Unclassified</option>
                            <option value="image">Image</option>
                            <option value="stitching">Stitching</option>
                          </select>
                        </td>
                        <td class="px-4 py-2 text-right">
                          <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => deleteTag(tag.id)}>Delete</button>
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>

            {#if unclassifiedTags.length > 0}
              <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl">
                <div class="bg-amber-50 border-b border-amber-200 px-4 py-2">
                  <h2 class="text-sm font-semibold text-amber-800 uppercase tracking-wide">Unclassified Tags</h2>
                </div>
                <table class="w-full text-sm">
                  <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                    <tr>
                      <th class="px-4 py-2 text-left">Description</th>
                      <th class="px-4 py-2 text-left">Group</th>
                      <th class="px-4 py-2"></th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-100">
                    {#each unclassifiedTags as tag}
                      <tr>
                        <td class="px-4 py-2">{tag.description}</td>
                        <td class="px-4 py-2">
                          <select
                            value={tag.tagGroup}
                            class="admin-input border rounded px-2 py-1 text-xs"
                            onchange={(event) => setTagGroup(tag.id, event.currentTarget.value)}
                          >
                            <option value="">Unclassified</option>
                            <option value="image">Image</option>
                            <option value="stitching">Stitching</option>
                          </select>
                        </td>
                        <td class="px-4 py-2 text-right">
                          <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => deleteTag(tag.id)}>Delete</button>
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          {:else if adminIsSourcesRoute}
            <h1 class="admin-title text-2xl font-bold text-gray-800">Manage Sources</h1>
            <p class="text-sm text-gray-500">
              Sources describe where your designs came from, such as Purchased, Downloaded, or Gift.
            </p>

            <div class="admin-card bg-white rounded shadow p-4 max-w-xl">
              <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new source</h2>
              <form class="flex gap-2" onsubmit={addSource}>
                <input
                  type="text"
                  bind:value={newSourceName}
                  required
                  placeholder="e.g. Purchased, Downloaded..."
                  class="admin-input flex-1 border rounded px-3 py-2 text-sm"
                />
                <button type="submit" class="settings-primary-button text-sm">Add</button>
              </form>
            </div>

            <div class="admin-card bg-white rounded shadow overflow-hidden max-w-2xl">
              <table class="w-full text-sm">
                <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                  <tr>
                    <th class="px-4 py-2 text-left">Name</th>
                    <th class="px-4 py-2"></th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">
                  {#if sources.length === 0}
                    <tr>
                      <td colspan="2" class="px-4 py-3 text-gray-400">No sources yet.</td>
                    </tr>
                  {:else}
                    {#each sources as source}
                      <tr>
                        <td class="px-4 py-2">{source.name}</td>
                        <td class="px-4 py-2 text-right">
                          <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => deleteSource(source.id)}>
                            Delete
                          </button>
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>
          {:else if adminIsHoopsRoute}
            <h1 class="admin-title text-2xl font-bold text-gray-800">Manage Hoops</h1>
            <p class="text-sm text-gray-500 max-w-3xl">
              Hoop sizes depend on your machine and the frames you own. Add your own hoops below.
            </p>

            <div class="admin-card bg-white rounded shadow p-4 max-w-4xl">
              <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new hoop</h2>
              <form class="flex gap-2 items-end flex-wrap" onsubmit={addHoop}>
                <div>
                  <label for="admin-hoop-name" class="block text-xs font-medium text-gray-600 mb-1">Name</label>
                  <input
                    id="admin-hoop-name"
                    type="text"
                    bind:value={newHoopName}
                    required
                    placeholder="e.g. 5x7 hoop"
                    class="admin-input border rounded px-3 py-2 text-sm w-52"
                  />
                </div>
                <div>
                  <label for="admin-hoop-width" class="block text-xs font-medium text-gray-600 mb-1">Max Width (mm)</label>
                  <input
                    id="admin-hoop-width"
                    type="number"
                    min="0.01"
                    step="0.01"
                    bind:value={newHoopWidth}
                    required
                    class="admin-input border rounded px-3 py-2 text-sm w-36"
                  />
                </div>
                <div>
                  <label for="admin-hoop-height" class="block text-xs font-medium text-gray-600 mb-1">Max Height (mm)</label>
                  <input
                    id="admin-hoop-height"
                    type="number"
                    min="0.01"
                    step="0.01"
                    bind:value={newHoopHeight}
                    required
                    class="admin-input border rounded px-3 py-2 text-sm w-36"
                  />
                </div>
                <button type="submit" class="settings-primary-button text-sm">Add</button>
              </form>
            </div>

            <div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl">
              <table class="w-full text-sm">
                <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                  <tr>
                    <th class="px-4 py-2 text-left">Name</th>
                    <th class="px-4 py-2 text-right">Max Width (mm)</th>
                    <th class="px-4 py-2 text-right">Max Height (mm)</th>
                    <th class="px-4 py-2 text-right">Used By</th>
                    <th class="px-4 py-2"></th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">
                  {#if hoops.length === 0}
                    <tr>
                      <td colspan="5" class="px-4 py-3 text-gray-400">No hoops defined yet. Add your own machine hoops above.</td>
                    </tr>
                  {:else}
                    {#each hoops as hoop}
                      <tr>
                        <td class="px-4 py-2 font-medium">
                          {#if editingHoopId === hoop.id}
                            <input
                              type="text"
                              bind:value={editingHoopName}
                              class="admin-input border rounded px-2 py-1 text-sm w-full"
                            />
                          {:else}
                            {hoop.name}
                          {/if}
                        </td>
                        <td class="px-4 py-2 text-right">
                          {#if editingHoopId === hoop.id}
                            <input
                              type="number"
                              min="0.01"
                              step="0.01"
                              bind:value={editingHoopWidth}
                              class="admin-input border rounded px-2 py-1 text-sm w-28 text-right"
                            />
                          {:else}
                            {hoop.maxWidthMm.toFixed(2)}
                          {/if}
                        </td>
                        <td class="px-4 py-2 text-right">
                          {#if editingHoopId === hoop.id}
                            <input
                              type="number"
                              min="0.01"
                              step="0.01"
                              bind:value={editingHoopHeight}
                              class="admin-input border rounded px-2 py-1 text-sm w-28 text-right"
                            />
                          {:else}
                            {hoop.maxHeightMm.toFixed(2)}
                          {/if}
                        </td>
                        <td class="px-4 py-2 text-right text-gray-600">{hoop.designCount}</td>
                        <td class="px-4 py-2 text-right">
                          <div class="flex justify-end gap-2 flex-wrap">
                            {#if editingHoopId === hoop.id}
                              <button type="button" class="text-indigo-600 hover:text-indigo-800 text-xs" onclick={() => saveHoopEdit(hoop.id)}>
                                Save
                              </button>
                              <button type="button" class="text-gray-500 hover:text-gray-700 text-xs" onclick={cancelEditHoop}>
                                Cancel
                              </button>
                            {:else if pendingDeleteHoopId === hoop.id}
                              <button type="button" class="text-red-600 hover:text-red-800 text-xs" onclick={() => deleteHoop(hoop.id)}>
                                Confirm delete
                              </button>
                              <button type="button" class="text-gray-500 hover:text-gray-700 text-xs" onclick={cancelDeleteHoop}>
                                Cancel
                              </button>
                            {:else}
                              <button type="button" class="text-indigo-600 hover:text-indigo-800 text-xs" onclick={() => beginEditHoop(hoop)}>
                                Edit
                              </button>
                              <button type="button" class="text-red-400 hover:text-red-600 text-xs" onclick={() => requestDeleteHoop(hoop)}>
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
                              This hoop is assigned to {hoop.designCount} design(s). Deleting it will set those design hoop assignments to blank.
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
          {:else}
            <div class="route-panel">This admin screen is not yet mapped.</div>
          {/if}

          <p class="text-xs text-gray-500">
            Data source: {adminDataSource === "rust" ? "Rust persistence" : "mock fallback"}.
          </p>
        </section>
      {:else if currentUiKind === "tagging-actions"}
        <div class="space-y-3">
          <div class="grid md:grid-cols-3 gap-3">
            <div class="route-card">Tag untagged action placeholder</div>
            <div class="route-card">Tag unverified action placeholder</div>
            <div class="route-card">Retag all action placeholder</div>
          </div>
          <div class="route-panel">
            Tier selection and delay/batch options placeholder.
          </div>
        </div>
      {:else if currentUiKind === "orphans"}
        <div class="space-y-3">
          <div class="route-card">Orphan records result list placeholder</div>
          <div class="grid sm:grid-cols-2 gap-3">
            <div class="route-card">Delete selected placeholder</div>
            <div class="route-card">Delete all placeholder</div>
          </div>
        </div>
      {:else if currentUiKind === "about"}
        <div class="space-y-2">
          <div class="route-card">Application overview placeholder</div>
          <div class="route-card">Document links placeholder</div>
        </div>
      {:else if currentUiKind === "licence"}
        <div class="space-y-2">
          <div class="route-card">License text placeholder</div>
          <div class="route-card">Third-party notices placeholder</div>
        </div>
      {:else}
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3 pt-2">
          <div class="route-card">UI Shell</div>
          <div class="route-card">Navigation Contract</div>
          <div class="route-card">Backend Hookup Deferred</div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="bg-white rounded-xl shadow p-6 space-y-4">
      <h1 class="text-2xl font-bold text-gray-800">Route Not Found</h1>
      <p class="text-gray-600">
        The requested route does not exist in this stage. Use one of the known placeholders below.
      </p>

      <div class="flex flex-wrap gap-2">
        <button class="menu-button-primary" onclick={() => navigateTo("#/designs")}>Go to Browse</button>
      </div>

      <div class="border border-gray-200 rounded-lg p-4 bg-gray-50 text-sm text-gray-700">
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

<footer class="max-w-7xl mx-auto px-4 pb-6 text-xs text-gray-500">
  <div class="border-t border-gray-300 pt-4 flex flex-wrap items-center gap-x-3 gap-y-1">
    <span>Embroidery Catalogue</span>
    <span aria-hidden="true">•</span>
    <a href="#/about" class="hover:underline text-indigo-600">About</a>
    <span aria-hidden="true">•</span>
    <a href="#/about/licence" class="hover:underline text-indigo-600">Licence</a>
  </div>
</footer>

{#if orphanModalOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="orphan-modal-title"
    aria-describedby="orphan-modal-body"
    tabindex="-1"
    bind:this={orphanModalContainer}
    onclick={(event) => {
      if (event.target === event.currentTarget && orphanCanDismiss) {
        closeOrphansModal();
      }
    }}
    onkeydown={handleModalKeydown}
  >
    <div class="bg-white rounded-xl shadow-xl w-full max-w-sm p-6 space-y-4">
      <h2
        id="orphan-modal-title"
        class="text-lg font-semibold text-gray-800"
        tabindex="-1"
        bind:this={orphanModalTitle}
      >
        {orphanStatus === "loading" ? "Checking files..." : orphanStatus === "error" ? "Scan failed" : "Scan complete"}
      </h2>

      {#if orphanStatus === "loading"}
        <p id="orphan-modal-body" class="text-sm text-gray-600" aria-live="polite">⏳ Scanning database against disk, please wait...</p>
      {:else if orphanStatus === "error"}
        <p id="orphan-modal-body" class="text-sm text-red-600" aria-live="polite">Could not complete scan: {orphanError || "Unknown error"}</p>
      {:else}
        <p id="orphan-modal-body" class="text-sm text-gray-600" aria-live="polite">
          Checked {orphanChecked} file record(s). Found {orphanFound} {orphanFound === 1 ? "orphan" : "orphans"}.
        </p>
      {/if}

      <div class="flex justify-end gap-3">
        {#if orphanCanDismiss}
          <button class="menu-button-secondary" onclick={closeOrphansModal} bind:this={orphanCloseButton}>Close</button>
        {/if}
        {#if orphanStatus === "done" && orphanFound > 0}
          <button class="menu-button-primary" onclick={viewOrphans} bind:this={orphanPrimaryButton}>
            View Orphans
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
