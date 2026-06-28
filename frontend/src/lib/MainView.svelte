<script>
  import { onDestroy, tick } from "svelte";
  import {
    getBrowseDesigns,
    getBrowseDesignPreviews,
    getBrowseProjects,
    getProjectsList,
    createProject,
    getProjectDetail,
    updateProject,
    deleteProject,
    removeDesignFromProjectDetail,
    getProjectPrintView,
    getBrowseTags,
    getDesignDetail,
    getDesignImageDataUrl,
    updateDesignMetadata,
    setDesignRating,
    setDesignStitched,
    setDesignTagsChecked,
    setDesignTags,
    addDesignToProject,
    removeDesignFromProject,
    deleteDesign,
    openDesignInEditor,
    openDesignInExplorer,
    renderDesign3dPreview,
    browseImportFolder,
    previewImportFromRoots,
    precheckImportWire,
    runPrecheckAction,
    requestStopBulkImport,
    bulkVerifyDesigns,
    bulkAddDesignsToProject,
    bulkSetTagsForDesigns,
    getAboutDocuments,
    getAboutDocument,
    getSettingsViewModel,
    saveSettings,
    saveImportLastBrowseFolder,
    browseSettingsDataRoot,
    getBackupViewModel,
    saveBackupSettings,
    browseBackupFolder,
    runDatabaseBackup,
    runDesignsBackup,
    runBothBackups,
    scanOrphans,
    getOrphansPage,
    deleteOrphans as removeOrphans,
    deleteAllOrphans as removeAllOrphans,
    browseOrphanPath,
    debugOrphansScan,
    getTaggingActionsViewModel,
    runUnifiedBackfill,
    stopUnifiedBackfill,
    getBackfillLogEntries,
    runStitchingBackfill,
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
  import HelpView from "./views/HelpView.svelte";
  import AboutView from "./views/AboutView.svelte";
  import AboutDocumentView from "./views/AboutDocumentView.svelte";

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
      subtitle: "Project planning and membership",
      description: "Create and manage project groups, remove design memberships, and print project sheets.",
      cta: "Use New Project to start a sewing-session plan",
    },
    "#/help": {
      title: "Help",
      subtitle: "",
      description: "Quick guidance for using the Embroidery Catalogue.",
      cta: "Use quick-jump links to move between sections",
    },
    "#/admin/designers": {
      title: "Manage Designers",
      subtitle: "",
      description: "Designers are the creators or brands of embroidery designs. Use this list to keep designer names consistent.",
      cta: "Next backend hookup: designers CRUD commands",
    },
    "#/admin/tags": {
      title: "Tags",
      subtitle: "",
      description: "",
      cta: "Next backend hookup: tags CRUD commands",
    },
    "#/admin/sources": {
      title: "Sources",
      subtitle: "",
      description: "",
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
      subtitle: "Orphaned design records",
      description: "Find and remove database records whose files no longer exist on disk.",
      cta: "Scan first, then review and delete selected or all orphaned records",
    },
    "#/about": {
      title: "About",
      subtitle: "App information and bundled documents",
      description: "Open project documents such as privacy, security, AI tagging, and licence.",
      cta: "Choose a document to open",
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
    "#/about",
    "#/about/document/licence",
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
    "troubleshooting",
  ]);

  function parseDesignDetailId(route) {
    const match = route.match(/^#\/designs\/(\d+)$/);
    return match ? Number(match[1]) : null;
  }

  function parseDesignPrintId(route) {
    const match = route.match(/^#\/designs\/(\d+)\/print$/);
    return match ? Number(match[1]) : null;
  }

  function parseProjectDetailId(route) {
    const match = route.match(/^#\/projects\/(\d+)$/);
    return match ? Number(match[1]) : null;
  }

  function parseProjectPrintId(route) {
    const match = route.match(/^#\/projects\/(\d+)\/print$/);
    return match ? Number(match[1]) : null;
  }

  function parseAboutDocumentSlug(route) {
    if (route === "#/about/licence") {
      return "licence";
    }

    const match = route.match(/^#\/about\/document\/([a-z0-9-]+)$/);
    return match ? String(match[1]).toLowerCase() : null;
  }

  function parseImportWizardStep(route) {
    if (route === "#/import") {
      return 1;
    }

    const match = route.match(/^#\/import\/step([123])$/);
    return match ? Number(match[1]) : null;
  }

  function resolveCurrentPage(route) {
    if (parseProjectPrintId(route) !== null) {
      return {
        title: "Project Print",
        subtitle: "Printable project sheet",
        description: "Print-friendly summary of all designs in the selected project.",
        cta: "Use browser print from this screen",
      };
    }

    if (route === "#/projects/new") {
      return {
        title: "New Project",
        subtitle: "Create project",
        description: "Create a named project for planned embroidery work.",
        cta: "Name is required, description is optional",
      };
    }

    if (parseProjectDetailId(route) !== null) {
      return {
        title: "Project Detail",
        subtitle: "Manage project",
        description: "Update project details and remove design memberships.",
        cta: "Use Print Sheet for a planning handout",
      };
    }

    if (parseDesignPrintId(route) !== null) {
      return {
        title: "Design Print",
        subtitle: "Printable design summary",
        description: "Print-friendly detail view for a single design.",
        cta: "Use browser print from this screen",
      };
    }

    if (parseDesignDetailId(route) !== null) {
      return {
        title: "Design Detail",
        subtitle: "Design metadata and actions",
        description: "Review one design, edit metadata and tags, manage project membership, and run file and print actions.",
        cta: "Use the detail controls to update this design",
      };
    }

    const aboutSlug = parseAboutDocumentSlug(route);
    if (aboutSlug !== null) {
      return {
        title: "About Document",
        subtitle: "Bundled document viewer",
        description: "Read bundled project, policy, and reference documents.",
        cta: "Use Back to About to open another document",
      };
    }

    const importStep = parseImportWizardStep(route);
    if (importStep !== null) {
      return ROUTE_PAGES["#/import"];
    }

    return ROUTE_PAGES[route] || null;
  }

  function resolveCurrentUiKind(route) {
    if (parseProjectPrintId(route) !== null) {
      return "project-print";
    }

    if (route === "#/projects/new") {
      return "project-new";
    }

    if (parseProjectDetailId(route) !== null) {
      return "project-detail";
    }

    if (parseDesignPrintId(route) !== null) {
      return "design-print";
    }

    if (parseDesignDetailId(route) !== null) {
      return "design-detail";
    }

    if (parseAboutDocumentSlug(route) !== null) {
      return "about-document";
    }

    if (parseImportWizardStep(route) !== null) {
      return "import";
    }

    return ROUTE_UI_KIND[route] || null;
  }

  let currentRoute = $state("");
  let currentPage = $derived(resolveCurrentPage(currentRoute));
  let currentUiKind = $derived(resolveCurrentUiKind(currentRoute));
  let detailDesignId = $derived(parseDesignDetailId(currentRoute));
  let printDesignId = $derived(parseDesignPrintId(currentRoute));
  let projectDetailId = $derived(parseProjectDetailId(currentRoute));
  let projectPrintId = $derived(parseProjectPrintId(currentRoute));
  let aboutDocumentSlug = $derived(parseAboutDocumentSlug(currentRoute));
  let importRouteStep = $derived(parseImportWizardStep(currentRoute));

  let browseItems = $state([]);
  let browseSource = $state("mock");
  let browseLoading = $state(false);
  let browseHasLoaded = $state(false);
  let browseError = $state("");
  let browseProjects = $state([]);
  let browseProjectsSource = $state("mock");
  let browseProjectsLoading = $state(false);
  let browseProjectsLoaded = $state(false);
  let browseTagOptions = $state([]);
  let browseTagsSource = $state("mock");
  let browseDesignerFilterOptions = $state([]);
  let browseSourceFilterOptions = $state([]);
  let browseHoopFilterOptions = $state([]);
  let browseFilterReferenceLoaded = $state(false);
  let browsePreviewById = $state({});
  let browsePreviewsSource = $state("mock");
  let browsePreviewsLoading = $state(false);
  let browsePreviewRequestCounter = 0;
  let browseCurrentPage = $state(1);
  let browseAdditionalFiltersOpen = $state(false);
  let browseSelectedIds = $state([]);
  let browseBulkBarNode = $state(null);
  let browseBulkModalOverlayNode = $state(null);
  let browseBulkModalDialogNode = $state(null);
  let browseBulkModalOpen = $state(false);
  let browseBulkModalMode = $state("browse");
  let browseBulkTagSelection = $state([]);
  let browseBulkProjectSelection = $state([]);
  let browseBulkProjectDropdownOpen = $state(false);
  let browseCardProjectPendingById = $state({});
  let browseDeleteConfirmOpen = $state(false);
  let browseDeleteSelectedBusy = $state(false);
  let browseActionNotice = $state("");
  let browseGridContainer = $state(null);
  let browseGridColumns = $state(2);

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
    designerFilters: [],
    tagFilters: [],
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
  });

  let browseFilters = $state(defaultBrowseFilters());

  let detailItem = $state(null);
  let detailSource = $state("mock");
  let detailLoading = $state(false);
  let detailError = $state("");
  let detailActionMessage = $state("");
  let detailActionIsError = $state(false);
  let detailSaving = $state(false);
  let detailNotes = $state("");
  let detailDesignerId = $state("");
  let detailSourceId = $state("");
  let detailTagSelection = $state([]);
  let detailProjectToAdd = $state("");
  let detailBrowseIds = $state([]);
  let detailBrowseIndex = $state(-1);

  let projectsItems = $state([]);
  let projectsSource = $state("mock");
  let projectsLoading = $state(false);
  let projectsLoaded = $state(false);
  let projectsError = $state("");
  let projectsLoadRequestToken = 0;
  let projectsActionMessage = $state("");
  let projectsActionIsError = $state(false);

  let projectNewName = $state("");
  let projectNewDescription = $state("");
  let projectNewSaving = $state(false);

  let projectDetail = $state(null);
  let projectDetailSource = $state("mock");
  let projectDetailLoading = $state(false);
  let projectDetailError = $state("");
  let projectDetailSaving = $state(false);
  let projectDetailName = $state("");
  let projectDetailDescription = $state("");

  let projectPrint = $state(null);
  let projectPrintSource = $state("mock");
  let projectPrintLoading = $state(false);
  let projectPrintError = $state("");

  let aboutDocuments = $state([]);
  let aboutDocumentsSource = $state("mock");
  let aboutDocumentsLoading = $state(false);
  let aboutDocumentsLoaded = $state(false);
  let aboutDocumentsError = $state("");

  let aboutDocumentItem = $state(null);
  let aboutDocumentSource = $state("mock");
  let aboutDocumentLoading = $state(false);
  let aboutDocumentError = $state("");
  let aboutDocumentLoadedSlug = $state("");

  let importRootPath = $state("");
  let importRootPaths = $state([]);
  let importPreview = $state(null);
  let importPreviewSource = $state("mock");
  let importPrecheck = $state(null);
  let importPrecheckSource = $state("mock");
  let importPrecheckMessage = $state("Run precheck after selecting files.");
  let importStep3ImagePreference = $state("2d");
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
  let importNowInProgress = $derived(importActionLoading && importActionInProgress === "import_now");

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
  let backupDbSourcePath = $state("(not available yet)");
  let backupDesignsSourcePath = $state("(not available yet)");
  let backupLoaded = $state(false);
  let backupLoading = $state(false);
  let backupDatabaseRunning = $state(false);
  let backupDesignsRunning = $state(false);
  let backupStatus = $state("idle");
  let backupMessage = $state("");
  let importHasAppliedSavedRoot = $state(false);

  let taggingActionsLoaded = $state(false);
  let taggingActionsLoading = $state(false);
  let taggingRunInFlight = $state(false);
  let taggingHasGoogleApiKey = $state(false);
  let taggingTier2Default = $state(false);
  let taggingTier3Default = $state(false);
  let taggingBatchSize = $state("100");
  let taggingCommitEvery = $state("100");
  let taggingWorkers = $state("4");
  let taggingActionMode = $state("tag_untagged");
  let taggingRunTier2 = $state(false);
  let taggingRunTier3 = $state(false);
  let taggingRunStitching = $state(false);
  let taggingClearExistingStitching = $state(false);
  let taggingRunImages = $state(false);
  let taggingImageRedo = $state(false);
  let taggingUpgrade2dTo3d = $state(false);
  let taggingUsePreview3d = $state(true);
  let taggingRunColorCounts = $state(false);
  let taggingStatusType = $state("info");
  let taggingStatusMessage = $state("Configure actions, then run selected actions.");
  let taggingLastSummary = $state(null);
  let taggingLogEntries = $state([]);

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
  let adminImageTagsOpen = $state(true);
  let adminStitchingTagsOpen = $state(true);
  let adminTagsPanelStateLoaded = $state(false);

  let imageTags = $derived(tags.filter((tag) => tag.tagGroup === "image"));
  let stitchingTags = $derived(tags.filter((tag) => tag.tagGroup === "stitching"));
  let unclassifiedTags = $derived(tags.filter((tag) => !tag.tagGroup));

  function setAdminNotice(message, type = "info") {
    adminNotice = message;
    adminNoticeType = type;
  }

  function handleAdminTagPanelToggle(panel, event) {
    const isOpen = Boolean(event?.currentTarget?.open);
    if (panel === "image") {
      adminImageTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.image", isOpen ? "open" : "closed");
      }
      return;
    }

    if (panel === "stitching") {
      adminStitchingTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.stitching", isOpen ? "open" : "closed");
      }
    }
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
  let backupAnyRunning = $derived(backupDatabaseRunning || backupDesignsRunning);
  let taggingCommitValue = $derived(Math.max(1, Number.parseInt(taggingCommitEvery, 10) || 100));
  let taggingBatchValue = $derived(Math.max(1, Number.parseInt(taggingBatchSize, 10) || 100));
  let taggingWorkersValue = $derived(Math.max(1, Number.parseInt(taggingWorkers, 10) || 4));

  let adminIsDesignersRoute = $derived(currentRoute === "#/admin/designers");
  let adminIsTagsRoute = $derived(currentRoute === "#/admin/tags");
  let adminIsSourcesRoute = $derived(currentRoute === "#/admin/sources");
  let adminIsHoopsRoute = $derived(currentRoute === "#/admin/hoops");

  let orphansLoading = $state(false);
  let orphansLoaded = $state(false);
  let orphansError = $state("");
  let orphanItems = $state([]);
  let orphanPage = $state(1);
  let orphanPageSize = $state(100);
  let orphanTotal = $state(0);
  let orphanTotalPages = $state(1);
  let orphanSelectedIds = $state([]);
  let orphanActionMessage = $state("");
  let orphanActionType = $state("info");
  let orphanDebugFilter = $state("1033");
  let orphanDebugLoading = $state(false);
  let orphanDebugError = $state("");
  let orphanDebugResult = $state(null);

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

  async function loadBackupFromBackend(force = false) {
    if (backupLoading) {
      return;
    }
    if (backupLoaded && !force) {
      return;
    }

    backupLoading = true;
    try {
      const result = await getBackupViewModel();
      const model = result?.model || {};
      backupDbDestination = String(model?.db_destination || "");
      backupDesignsDestination = String(model?.designs_destination || "");
      backupSavedDbDestination = backupDbDestination;
      backupSavedDesignsDestination = backupDesignsDestination;

      const fallbackDataRoot = settingsDataRoot ? String(settingsDataRoot) : "";
      backupDbSourcePath = String(model?.db_source_path || (fallbackDataRoot ? `${fallbackDataRoot}\\database\\catalogue.db` : "(not available yet)"));
      backupDesignsSourcePath = String(
        model?.designs_source_path || (fallbackDataRoot ? `${fallbackDataRoot}\\MachineEmbroideryDesigns` : "(not available yet)")
      );

      backupLoaded = true;
    } catch (error) {
      backupStatus = "error";
      backupMessage = `Could not load backup settings: ${error}`;
    } finally {
      backupLoading = false;
    }
  }

  function applyTaggingViewModel(model) {
    taggingHasGoogleApiKey = Boolean(model?.has_google_api_key);
    taggingTier2Default = Boolean(model?.ai_tier2_auto);
    taggingTier3Default = Boolean(model?.ai_tier3_auto);

    const defaultBatch = Number(model?.default_batch_size ?? 100);
    const defaultCommit = Number(model?.default_commit_every ?? 100);
    const defaultWorkers = Number(model?.default_workers ?? 4);

    const configuredBatch = String(model?.ai_batch_size || "").trim();
    const configuredCommit = String(model?.import_commit_batch_size || "").trim();

    taggingBatchSize = configuredBatch || String(defaultBatch);
    taggingCommitEvery = configuredCommit || String(defaultCommit);
    taggingWorkers = String(defaultWorkers);
    taggingRunTier2 = taggingTier2Default && taggingHasGoogleApiKey;
    taggingRunTier3 = taggingTier3Default && taggingHasGoogleApiKey;
  }

  async function loadTaggingActionsViewModel(force = false) {
    if (taggingActionsLoading) {
      return;
    }
    if (taggingActionsLoaded && !force) {
      return;
    }

    taggingActionsLoading = true;
    try {
      const result = await getTaggingActionsViewModel();
      applyTaggingViewModel(result?.model || {});
      taggingActionsLoaded = true;
      taggingStatusType = "info";
      taggingStatusMessage = taggingHasGoogleApiKey
        ? "API key detected. Tier 2/3 options are available."
        : "No Google API key detected. Tier 2/3 calls are disabled.";
    } catch (error) {
      taggingStatusType = "error";
      taggingStatusMessage = `Could not load tagging action defaults: ${error}`;
    } finally {
      taggingActionsLoading = false;
    }
  }

  async function refreshBackfillLogEntries() {
    const result = await getBackfillLogEntries(10);
    taggingLogEntries = Array.isArray(result?.entries) ? result.entries : [];
  }

  async function runTaggingActions(event) {
    event?.preventDefault?.();
    if (taggingRunInFlight) {
      return;
    }

    const actions = {};
    actions.tagging = {
      enabled: true,
      action: taggingActionMode,
      tiers: [
        1,
        ...(taggingRunTier2 && taggingHasGoogleApiKey ? [2] : []),
        ...(taggingRunTier3 && taggingHasGoogleApiKey ? [3] : []),
      ],
    };

    if (taggingRunStitching) {
      actions.stitching = {
        enabled: true,
        clear_existing_stitching: taggingClearExistingStitching,
      };
    }

    if (taggingRunImages) {
      actions.images = {
        enabled: true,
        redo: taggingImageRedo,
        upgrade_2d_to_3d: taggingUpgrade2dTo3d && !taggingImageRedo,
        preview_3d: taggingUsePreview3d,
      };
    }

    if (taggingRunColorCounts) {
      actions.color_counts = { enabled: true };
    }

    taggingRunInFlight = true;
    taggingStatusType = "info";
    taggingStatusMessage = "Running selected actions...";

    try {
      const result = await runUnifiedBackfill({
        actions,
        batch_size: taggingBatchValue,
        commit_every: taggingCommitValue,
        workers: taggingWorkersValue,
        preview_3d: taggingUsePreview3d,
      });

      if (result?.error) {
        taggingStatusType = "error";
        taggingStatusMessage = `Backfill run failed: ${result.error}`;
      } else {
        taggingLastSummary = result;
        taggingStatusType = result.stopped ? "error" : (result.errors > 0 ? "error" : "success");
        taggingStatusMessage = `Processed ${result.processed} design(s), errors ${result.errors}, actions: ${(result.actions || []).join(", ") || "none"}.`;
      }
      await refreshBackfillLogEntries();
    } catch (error) {
      taggingStatusType = "error";
      taggingStatusMessage = `Backfill run failed: ${error}`;
    } finally {
      taggingRunInFlight = false;
    }
  }

  async function stopTaggingActionsRun() {
    const result = await stopUnifiedBackfill();
    taggingStatusType = "info";
    taggingStatusMessage = `Stop request sent: ${result?.status || "stopping"}.`;
  }

  async function runStitchingOnlyAction() {
    if (taggingRunInFlight) {
      return;
    }
    taggingRunInFlight = true;
    taggingStatusType = "info";
    taggingStatusMessage = "Running stitching-only backfill...";

    try {
      const result = await runStitchingBackfill({
        clearExistingStitching: taggingClearExistingStitching,
        batchSize: taggingBatchValue,
      });
      taggingLastSummary = result;
      taggingStatusType = result.errors > 0 ? "error" : "success";
      taggingStatusMessage = `Stitching backfill complete: processed ${result.processed}, errors ${result.errors}.`;
      await refreshBackfillLogEntries();
    } finally {
      taggingRunInFlight = false;
    }
  }

  async function browseBackupDestinationUiOnly(kind) {
    const startDir = kind === "database" ? backupDbDestination : backupDesignsDestination;
    const result = await browseBackupFolder(startDir);

    if (result.path) {
      if (kind === "database") {
        backupDbDestination = result.path;
      } else {
        backupDesignsDestination = result.path;
      }
      backupStatus = "idle";
      backupMessage = "";
      return;
    }

    if (result.error) {
      backupStatus = "error";
      backupMessage = result.error;
    }
  }

  async function saveBackupDestinationsUiOnly(event) {
    event.preventDefault();

    if (!backupHasUnsavedChanges) {
      backupStatus = "error";
      backupMessage = "There are no destination changes to save.";
      return;
    }

    const result = await saveBackupSettings({
      dbDestination: backupDbDestination,
      designsDestination: backupDesignsDestination,
    });

    if (result.saved) {
      backupSavedDbDestination = String(result.db_destination || backupDbDestination).trim();
      backupSavedDesignsDestination = String(result.designs_destination || backupDesignsDestination).trim();
      backupDbDestination = backupSavedDbDestination;
      backupDesignsDestination = backupSavedDesignsDestination;
      backupStatus = "saved";
      backupMessage = result.message || "Backup destinations saved.";
      return;
    }

    backupStatus = "error";
    backupMessage = result.message || "Could not save backup destinations.";
  }

  async function runBackupActionUiOnly(action) {
    if (backupAnyRunning) {
      return;
    }

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

    const runsDatabase = action === "database" || action === "both";
    const runsDesigns = action === "designs" || action === "both";
    if (runsDatabase) {
      backupDatabaseRunning = true;
    }
    if (runsDesigns) {
      backupDesignsRunning = true;
    }

    try {
      if (action === "database") {
        const result = await runDatabaseBackup();
        if (!result.success) {
          backupStatus = "error";
          backupMessage = result.error || "Database backup failed.";
          return;
        }

        const mb = (Number(result.size_bytes || 0) / (1024 * 1024)).toFixed(2);
        backupStatus = "saved";
        backupMessage = `Database backup created: ${result.backup_path || "(path unavailable)"} (${mb} MB).`;
        return;
      }

      if (action === "designs") {
        const result = await runDesignsBackup();
        if (!result.success) {
          backupStatus = "error";
          backupMessage = result.error || "Designs backup failed.";
          return;
        }

        backupStatus = "saved";
        backupMessage = `Designs backup complete: scanned ${result.scanned}, copied ${result.copied}, updated ${result.updated}, unchanged ${result.unchanged}, archived ${result.archived}.`;
        return;
      }

      const result = await runBothBackups();
      const dbOk = Boolean(result?.database?.success);
      const designsOk = Boolean(result?.designs?.success);

      if (dbOk && designsOk) {
        backupStatus = "saved";
        backupMessage = "Both backups completed successfully.";
        return;
      }

      const dbError = String(result?.database?.error || "").trim();
      const designsError = String(result?.designs?.error || "").trim();
      backupStatus = "error";
      backupMessage = `Backup results: database ${dbOk ? "ok" : "failed"}${dbError ? ` (${dbError})` : ""}; designs ${designsOk ? "ok" : "failed"}${designsError ? ` (${designsError})` : ""}.`;
    } finally {
      if (runsDatabase) {
        backupDatabaseRunning = false;
      }
      if (runsDesigns) {
        backupDesignsRunning = false;
      }
    }
  }

  function normalizeHash(hash) {
    if (!hash || hash === "#" || hash === "#/") {
      return "#/designs";
    }

    const hashParts = String(hash).split("#");
    const routePart = hashParts.length > 1 ? `#${hashParts[1]}` : String(hash);
    const fragmentId = hashParts.length > 2 ? hashParts.slice(2).join("#") : "";

    if (HELP_SECTION_IDS.has(routePart.startsWith("#") ? routePart.slice(1) : routePart)) {
      return "#/help";
    }

    if (routePart === "#/help" && HELP_SECTION_IDS.has(fragmentId)) {
      return "#/help";
    }

    if (routePart === "#/about/licence") {
      return "#/about/document/licence";
    }

    return routePart;
  }

  function isImportRoute(route) {
    return parseImportWizardStep(route) !== null;
  }

  function syncRouteFromHash() {
    const rawHash = window.location.hash || "";
    const nextRoute = normalizeHash(rawHash);
    const hashParts = rawHash.split("#");
    const sectionId = hashParts.length > 2 ? hashParts.slice(2).join("#") : rawHash.startsWith("#") ? rawHash.slice(1) : "";
    const scrollToHelpSection = HELP_SECTION_IDS.has(sectionId);
    const enteringProjects = currentRoute !== "#/projects" && nextRoute === "#/projects";
    currentRoute = nextRoute;

    if (scrollToHelpSection) {
      setTimeout(() => {
        document.getElementById(sectionId)?.scrollIntoView({ behavior: "smooth", block: "start" });
      }, 0);
    }

    if (enteringProjects) {
      // Force a single refresh when landing on projects without creating a reactive reload loop.
      projectsLoaded = false;
    }
  }

  function linkClass(target) {
    const isActive =
      currentRoute === target ||
      (target === "#/designs" && currentRoute.startsWith("#/designs/")) ||
      (target === "#/projects" && currentRoute.startsWith("#/projects/")) ||
      (target === "#/import" && isImportRoute(currentRoute));
    return `menu-link menu-link-primary ${isActive ? "menu-link-active" : ""}`;
  }

  function adminLinkClass(target) {
    const isActive = currentRoute === target;
    return `menu-link menu-link-admin ${isActive ? "menu-link-active" : ""}`;
  }

  async function openOrphansModal(event) {
    event.preventDefault();

    try {
      const result = await scanOrphans();
      if (result?.source !== "rust" && result?.error) {
        throw new Error(result.error);
      }
      const orphanChecked = Number(result?.checked ?? 0);
      const orphanFound = Number(result?.found ?? 0);
      orphanActionType = "info";
      orphanActionMessage = `Scan complete. Checked ${orphanChecked} file record(s). Found ${orphanFound} ${orphanFound === 1 ? "orphan" : "orphans"}.`;
    } catch (error) {
      orphanActionType = "error";
      orphanActionMessage = `Could not complete scan: ${String(error) || "Unknown error"}`;
    }

    navigateTo("#/admin/orphans");
  }

  async function loadOrphansPage(page = 1, force = false) {
    if (orphansLoading) {
      return;
    }
    if (!force && orphansLoaded && page === orphanPage) {
      return;
    }

    orphansLoading = true;
    orphansError = "";

    try {
      const result = await getOrphansPage({ page, pageSize: orphanPageSize });
      if (result?.source !== "rust" && result?.error) {
        throw new Error(result.error);
      }

      orphanItems = Array.isArray(result?.items) ? result.items : [];
      orphanPage = Math.max(1, Number(result?.page ?? page));
      orphanPageSize = Math.max(1, Number(result?.page_size ?? 100));
      orphanTotal = Math.max(0, Number(result?.total ?? 0));
      orphanTotalPages = Math.max(1, Number(result?.total_pages ?? 1));
      orphanSelectedIds = orphanItems.map((item) => Number(item.id));
      orphansLoaded = true;
    } catch (error) {
      orphansError = `Could not load orphans: ${error}`;
      orphanItems = [];
      orphanSelectedIds = [];
      orphanTotal = 0;
      orphanTotalPages = 1;
      orphansLoaded = false;
    } finally {
      orphansLoading = false;
    }
  }

  function orphanIsSelected(id) {
    return orphanSelectedIds.includes(Number(id));
  }

  function toggleOrphanSelection(id, checked) {
    const normalizedId = Number(id);
    if (!Number.isFinite(normalizedId) || normalizedId <= 0) {
      return;
    }

    if (checked) {
      orphanSelectedIds = Array.from(new Set([...orphanSelectedIds, normalizedId]));
      return;
    }

    orphanSelectedIds = orphanSelectedIds.filter((value) => Number(value) !== normalizedId);
  }

  function selectAllOrphansOnPage() {
    orphanSelectedIds = orphanItems.map((item) => Number(item.id));
  }

  function deselectAllOrphansOnPage() {
    orphanSelectedIds = [];
  }

  async function openOrphanPath(filepath) {
    const result = await browseOrphanPath(filepath);
    if (result?.ok) {
      orphanActionType = "info";
      orphanActionMessage = `Opened: ${result.opened}`;
      return;
    }

    orphanActionType = "error";
    orphanActionMessage = `Could not open folder: ${result?.error || "Unknown error"}`;
  }

  async function deleteSelectedOrphans() {
    if (orphanSelectedIds.length === 0) {
      orphanActionType = "error";
      orphanActionMessage = "Select at least one orphan record first.";
      return;
    }

    const confirmed = window.confirm(
      `Delete ${orphanSelectedIds.length} selected record(s)? This cannot be undone.`
    );
    if (!confirmed) {
      return;
    }

    const result = await removeOrphans(orphanSelectedIds);
    if (!result?.persisted) {
      orphanActionType = "error";
      orphanActionMessage = `Could not delete selected orphans: ${result?.error || "Unknown error"}`;
      return;
    }

    orphanActionType = "success";
    orphanActionMessage = `${result.deleted} record(s) deleted.`;
    await loadOrphansPage(orphanPage, true);
    if (orphanPage > orphanTotalPages) {
      await loadOrphansPage(orphanTotalPages, true);
    }
  }

  async function deleteEveryOrphan() {
    if (orphanTotal <= 0) {
      orphanActionType = "info";
      orphanActionMessage = "There are no orphan records to delete.";
      return;
    }

    const confirmed = window.confirm(
      `Delete ALL ${orphanTotal} orphaned records? This cannot be undone.`
    );
    if (!confirmed) {
      return;
    }

    const result = await removeAllOrphans();
    if (!result?.persisted) {
      orphanActionType = "error";
      orphanActionMessage = `Could not delete all orphans: ${result?.error || "Unknown error"}`;
      return;
    }

    orphanActionType = "success";
    orphanActionMessage = `${result.deleted} record(s) deleted.`;
    await loadOrphansPage(1, true);
  }

  function goToOrphanPage(page) {
    const nextPage = Math.max(1, Math.min(orphanTotalPages, Number(page) || 1));
    if (nextPage === orphanPage) {
      return;
    }
    loadOrphansPage(nextPage, true);
  }

  function orphanPaginationPages() {
    if (orphanTotalPages <= 1) {
      return [1];
    }

    const pages = [];
    const windowStart = Math.max(1, orphanPage - 3);
    const windowEnd = Math.min(orphanTotalPages, orphanPage + 3);

    if (windowStart > 1) {
      pages.push(1);
      if (windowStart > 2) {
        pages.push("...");
      }
    }

    for (let page = windowStart; page <= windowEnd; page += 1) {
      pages.push(page);
    }

    if (windowEnd < orphanTotalPages) {
      if (windowEnd < orphanTotalPages - 1) {
        pages.push("...");
      }
      pages.push(orphanTotalPages);
    }

    return pages;
  }

  function openOrphanDesign(designId) {
    const id = Number(designId);
    if (!Number.isFinite(id) || id <= 0) {
      return;
    }

    const route = `#/designs/${id}`;
    const targetUrl = `${window.location.pathname}${window.location.search}${route}`;
    const tab = window.open(targetUrl, "_blank");
    if (tab) {
      tab.focus();
      return;
    }

    // Fall back to in-app navigation if popup blocking prevents opening a tab.
    navigateTo(route);
  }

  async function runOrphanDebugScan() {
    if (orphanDebugLoading) {
      return;
    }

    orphanDebugLoading = true;
    orphanDebugError = "";

    const result = await debugOrphansScan({
      contains: orphanDebugFilter,
      limit: 300,
    });

    if (result?.source !== "rust" && result?.error) {
      orphanDebugError = `Could not run orphan debug scan: ${result.error}`;
      orphanDebugResult = null;
      orphanDebugLoading = false;
      return;
    }

    orphanDebugResult = result;
    orphanDebugLoading = false;
  }

  function navigateTo(route) {
    window.location.hash = route;
    syncRouteFromHash();
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

  async function loadBrowseProjects(force = false) {
    if (browseProjectsLoading && !force) {
      return;
    }

    browseProjectsLoading = true;

    try {
      const result = await getBrowseProjects();
      browseProjects = Array.isArray(result.items) ? result.items : [];
      browseProjectsSource = result.source || "mock";
    } catch (error) {
      browseProjects = [];
      browseProjectsSource = "mock";
      console.info("Could not load browse projects", error);
    } finally {
      browseProjectsLoading = false;
      browseProjectsLoaded = true;
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

  function setProjectsNotice(message, isError = false) {
    projectsActionMessage = String(message || "");
    projectsActionIsError = Boolean(isError);
  }

  async function loadProjects(force = false) {
    if (projectsLoading && !force) {
      return;
    }

    const requestToken = ++projectsLoadRequestToken;
    const timeoutMs = 15000;

    projectsLoading = true;
    projectsError = "";

    try {
      const timeoutResult = new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            items: [],
            source: "mock",
            error: `Could not load projects: Timed out loading projects after ${timeoutMs / 1000}s.`,
          });
        }, timeoutMs);
      });

      const result = await Promise.race([getProjectsList(), timeoutResult]);

      // Ignore stale responses if a newer load request has started.
      if (requestToken !== projectsLoadRequestToken) {
        return;
      }

      projectsItems = Array.isArray(result?.items) ? result.items : [];
      projectsSource = result?.source || "mock";
      if (result?.error) {
        projectsError = String(result.error);
      }
      projectsLoaded = true;
    } catch (error) {
      if (requestToken !== projectsLoadRequestToken) {
        return;
      }
      projectsItems = [];
      projectsSource = "mock";
      projectsError = `Could not load projects: ${error}`;
    } finally {
      if (requestToken === projectsLoadRequestToken) {
        projectsLoading = false;
      }
    }
  }

  async function submitNewProject() {
    if (projectNewSaving) {
      return;
    }

    const name = String(projectNewName || "").trim();
    if (!name) {
      setProjectsNotice("Project name is required.", true);
      return;
    }

    projectNewSaving = true;
    const result = await createProject(name, projectNewDescription);
    projectNewSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted && Number.isFinite(result.project_id) && result.project_id > 0) {
      projectNewName = "";
      projectNewDescription = "";
      await loadProjects(true);
      navigateTo(`#/projects/${result.project_id}`);
    }
  }

  async function loadProjectDetailView(projectId) {
    if (projectId == null) {
      return;
    }

    projectDetailLoading = true;
    projectDetailError = "";

    try {
      const result = await getProjectDetail(projectId);
      projectDetail = result?.item || null;
      projectDetailSource = result?.source || "mock";

      if (!projectDetail) {
        projectDetailError = result?.error || `Could not load project ${projectId}.`;
        projectDetailName = "";
        projectDetailDescription = "";
      } else {
        projectDetailName = String(projectDetail?.project?.name || "");
        projectDetailDescription = String(projectDetail?.project?.description || "");
      }
    } catch (error) {
      projectDetail = null;
      projectDetailSource = "mock";
      projectDetailError = `Could not load project detail: ${error}`;
      projectDetailName = "";
      projectDetailDescription = "";
    } finally {
      projectDetailLoading = false;
    }
  }

  async function refreshProjectDetailView() {
    if (!projectDetailId) {
      return;
    }
    await loadProjectDetailView(projectDetailId);
    await loadProjects(true);
  }

  async function saveProjectDetail() {
    if (!projectDetail?.project?.id || projectDetailSaving) {
      return;
    }

    const name = String(projectDetailName || "").trim();
    if (!name) {
      setProjectsNotice("Project name is required.", true);
      return;
    }

    projectDetailSaving = true;
    const result = await updateProject(projectDetail.project.id, name, projectDetailDescription);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshProjectDetailView();
    }
  }

  async function confirmDeleteProject() {
    if (!projectDetail?.project?.id || projectDetailSaving) {
      return;
    }

    const confirmed = window.confirm("Delete this project?");
    if (!confirmed) {
      return;
    }

    projectDetailSaving = true;
    const result = await deleteProject(projectDetail.project.id);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      await loadProjects(true);
      navigateTo("#/projects");
    }
  }

  async function removeDesignFromProjectMembership(designId) {
    if (!projectDetail?.project?.id || !designId || projectDetailSaving) {
      return;
    }

    projectDetailSaving = true;
    const result = await removeDesignFromProjectDetail(projectDetail.project.id, designId);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshProjectDetailView();
    }
  }

  async function loadProjectPrint(projectId) {
    if (projectId == null) {
      return;
    }

    projectPrintLoading = true;
    projectPrintError = "";

    try {
      const result = await getProjectPrintView(projectId);
      projectPrint = result?.item || null;
      projectPrintSource = result?.source || "mock";
      if (!projectPrint) {
        projectPrintError = result?.error || `Could not load project print view for id ${projectId}.`;
      }
    } catch (error) {
      projectPrint = null;
      projectPrintSource = "mock";
      projectPrintError = `Could not load project print view: ${error}`;
    } finally {
      projectPrintLoading = false;
    }
  }

  async function loadAboutDocuments(force = false) {
    if (aboutDocumentsLoading && !force) {
      return;
    }
    if (aboutDocumentsLoaded && !force) {
      return;
    }

    aboutDocumentsLoading = true;
    aboutDocumentsError = "";

    try {
      const result = await getAboutDocuments();
      aboutDocuments = Array.isArray(result?.items) ? result.items : [];
      aboutDocumentsSource = result?.source || "mock";
      aboutDocumentsLoaded = true;
    } catch (error) {
      aboutDocuments = [];
      aboutDocumentsSource = "mock";
      aboutDocumentsLoaded = true;
      aboutDocumentsError = `Could not load about documents: ${error}`;
    } finally {
      aboutDocumentsLoading = false;
    }
  }

  async function loadAboutDocumentView(slug, force = false) {
    const normalizedSlug = String(slug || "").trim().toLowerCase();
    if (!normalizedSlug) {
      aboutDocumentItem = null;
      aboutDocumentError = "Document not found.";
      aboutDocumentLoadedSlug = "";
      return;
    }

    if (aboutDocumentLoading && !force) {
      return;
    }

    if (aboutDocumentLoadedSlug === normalizedSlug && !force) {
      return;
    }

    aboutDocumentLoading = true;
    aboutDocumentError = "";

    try {
      const result = await getAboutDocument(normalizedSlug);
      aboutDocumentItem = result?.item || null;
      aboutDocumentSource = result?.source || "mock";
      aboutDocumentLoadedSlug = normalizedSlug;
      if (!aboutDocumentItem) {
        aboutDocumentError = String(result?.error || "Document not found.");
      }
    } catch (error) {
      aboutDocumentItem = null;
      aboutDocumentSource = "mock";
      aboutDocumentLoadedSlug = normalizedSlug;
      aboutDocumentError = `Could not load document: ${error}`;
    } finally {
      aboutDocumentLoading = false;
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

  function setImportRootPathAt(index, path) {
    const next = normalizeImportRootPath(path);
    if (!next) {
      return;
    }

    if (index === null || index === undefined || index < 0) {
      importRootPath = next;
      return;
    }

    importRootPaths = importRootPaths.map((value, rowIndex) => (rowIndex === index ? next : value));
  }

  async function browseImportRootPath(targetIndex = -1) {
    if (importBrowseLoading || importLoading || importActionLoading) {
      return;
    }

    importBrowseLoading = true;
    importError = "";

    try {
      const currentValue = targetIndex === null || targetIndex === undefined || targetIndex < 0
        ? importRootPath
        : importRootPaths[targetIndex] || "";
      const startHint = parentFolder(currentValue)
        || parentFolder(settingsImportLastBrowseFolder)
        || "";
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

  function parentFolder(path) {
    const p = String(path || "").trim().replace(/[\/\\]+$/, "");
    if (!p) return "";
    const lastSep = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
    if (lastSep <= 0) return p;
    return p.slice(0, lastSep);
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
    const candidateRoots = [importRootPath, ...importRootPaths]
      .map((value) => normalizeImportRootPath(value))
      .filter(Boolean);

    const uniqueRoots = [];
    const seenRoots = new Set();
    for (const root of candidateRoots) {
      const key = root.toLowerCase();
      if (seenRoots.has(key)) {
        continue;
      }
      seenRoots.add(key);
      uniqueRoots.push(root);
    }

    return uniqueRoots;
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
    importRootPath = "";
    importRootPaths = [];
  }

  async function addCurrentImportRootPath() {
    importRootPaths = [...importRootPaths, ""];
  }

  function resetImportWizard() {
    if (importProgressUnlisten) {
      importProgressUnlisten();
      importProgressUnlisten = null;
    }

    importProgressStatus = "";
    importProgressToken = "";
    importBrowseLoading = false;
    importRootPath = "";
    importRootPaths = [];
    importHasAppliedSavedRoot = false;
    importPreview = null;
    importPreviewSource = "mock";
    importPrecheck = null;
    importPrecheckSource = "mock";
    importPrecheckMessage = "Run precheck after selecting files.";
    importStep3ImagePreference = settingsImagePreference === "3d" ? "3d" : "2d";
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

    if (isImportRoute(currentRoute)) {
      navigateTo("#/import/step1");
    }
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
        const payloadToken = String(payload?.context_token ?? payload?.contextToken ?? "").trim();
        if (payloadToken && payloadToken !== importProgressToken) {
          return;
        }

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

  function getFolderLabelFromFolderPath(folderPath) {
    const value = String(folderPath || "").trim();
    if (!value) {
      return "Unknown folder";
    }

    const normalized = value.replace(/\\/g, "/").replace(/\/+$/g, "");
    if (!normalized) {
      return "Unknown folder";
    }

    const segments = normalized.split("/").filter(Boolean);
    return segments.length > 0 ? segments[segments.length - 1] : normalized;
  }

  function getImportFilenameFromPath(fullPath) {
    const value = String(fullPath || "").trim();
    if (!value) {
      return "Unknown file";
    }

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

  function normalizeNameForImportMatching(value) {
    return String(value || "")
      .toLowerCase()
      .replace(/[_\-/\\]+/g, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

  function compactNameForImportMatching(value) {
    return String(value || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "");
  }

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
        if (!folderPath) {
          continue;
        }
        byPath.set(folderPath, assignment);
      }
      return byPath;
    })()
  );

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
      if (!rawName) {
        continue;
      }

      if (ignoredNames.has(rawName.toLowerCase())) {
        continue;
      }

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

  function getInferredImportDesigner(folderPath) {
    const resolved = importPreviewResolvedAssignmentByPath.get(normalizeImportPathKey(folderPath));
    const resolvedId = Number(resolved?.inferred_designer_id);
    if (Number.isFinite(resolvedId) && resolvedId > 0) {
      const matched = importDesigners.find((designer) => Number(designer?.id) === resolvedId);
      if (matched) {
        return matched;
      }
    }

    return suggestImportMatchFromPath(folderPath, importDesigners);
  }

  function getInferredImportSource(folderPath) {
    const resolved = importPreviewResolvedAssignmentByPath.get(normalizeImportPathKey(folderPath));
    const resolvedId = Number(resolved?.inferred_source_id);
    if (Number.isFinite(resolvedId) && resolvedId > 0) {
      const matched = importSources.find((source) => Number(source?.id) === resolvedId);
      if (matched) {
        return matched;
      }
    }

    return suggestImportMatchFromPath(folderPath, importSources);
  }

  function getImportFolderDesignerInferredLabel(folderPath) {
    const inferred = getInferredImportDesigner(folderPath);
    return inferred?.name ? `Keep inferred (${inferred.name})` : "Keep inferred";
  }

  function getImportFolderSourceInferredLabel(folderPath) {
    const inferred = getInferredImportSource(folderPath);
    return inferred?.name ? `Keep inferred (${inferred.name})` : "Keep inferred";
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

  let importStep2FolderGroups = $derived(
    (() => {
      const scannedFiles = Array.isArray(importPreview?.scanned_files) ? importPreview.scanned_files : [];
      const selectedByPath = new Set(importSelectedFiles.map((value) => String(value || "").trim()).filter(Boolean));
      const grouped = new Map();

      for (const rawFile of scannedFiles) {
        const fullPath = String(rawFile?.full_path || "").trim();
        if (!fullPath) {
          continue;
        }

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
          const sortedFiles = group.files.sort((left, right) =>
            left.filename.localeCompare(right.filename, undefined, { sensitivity: "base" })
          );
          const selectedCount = sortedFiles.filter((file) => file.isSelected).length;
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

  function mapServerImportRouteToHash(nextRoute) {
    const route = String(nextRoute || "").toLowerCase();
    if (route.startsWith("/designs")) {
      return "#/designs";
    }
    if (route.startsWith("/import")) {
      if (route.includes("step3") || route.includes("precheck") || route.includes("confirm")) {
        return "#/import/step3";
      }
      if (route.includes("step2") || route.includes("review") || route.includes("scan")) {
        return "#/import/step2";
      }
      if (route.includes("step1") || route.includes("folder")) {
        return "#/import/step1";
      }
      if (route.includes("precheck") || route.includes("confirm")) {
        return "#/import/step3";
      }
      if (route.includes("review") || route.includes("scan")) {
        return "#/import/step2";
      }
      return "#/import/step1";
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
      importStopRequestPending = false;
      importActionInProgress = "";
      importActionLoading = false;
    }
  }

  async function requestImportStop() {
    if (!importNowInProgress || importStopRequestPending) {
      return;
    }

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

  function openDesignDetail(item) {
    if (item?.id == null) {
      return;
    }
    navigateTo(`#/designs/${item.id}`);
  }

  function loadDetailBrowseContext(designId) {
    try {
      const raw = window.sessionStorage.getItem("browse_ids");
      const parsed = JSON.parse(raw || "[]");
      const ids = Array.isArray(parsed)
        ? parsed.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
        : [];
      detailBrowseIds = ids;
      detailBrowseIndex = ids.indexOf(Number(designId));
    } catch (error) {
      detailBrowseIds = [];
      detailBrowseIndex = -1;
    }
  }

  function goToPreviousDetail() {
    if (detailBrowseIndex <= 0) {
      return;
    }
    const prevId = detailBrowseIds[detailBrowseIndex - 1];
    if (Number.isFinite(prevId)) {
      navigateTo(`#/designs/${prevId}`);
    }
  }

  function goToNextDetail() {
    if (detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1) {
      return;
    }
    const nextId = detailBrowseIds[detailBrowseIndex + 1];
    if (Number.isFinite(nextId)) {
      navigateTo(`#/designs/${nextId}`);
    }
  }

  function splitDetailTagsByGroup(tags) {
    const allTags = Array.isArray(tags) ? tags : [];
    return {
      image: allTags.filter((tag) => String(tag?.tag_group || "") === "image"),
      stitching: allTags.filter((tag) => String(tag?.tag_group || "") === "stitching"),
      unclassified: allTags.filter((tag) => {
        const group = String(tag?.tag_group || "");
        return group !== "image" && group !== "stitching";
      }),
    };
  }

  function splitTagsByGroup(tags) {
    const allTags = Array.isArray(tags) ? tags : [];
    return {
      image: allTags.filter((tag) => String(tag?.tag_group || "") === "image"),
      stitching: allTags.filter((tag) => String(tag?.tag_group || "") === "stitching"),
      unclassified: allTags.filter((tag) => {
        const group = String(tag?.tag_group || "");
        return group !== "image" && group !== "stitching";
      }),
    };
  }

  function ratingToStars(rating) {
    const numeric = Number(rating);
    if (!Number.isFinite(numeric) || numeric <= 0) {
      return "";
    }
    const clamped = Math.min(5, Math.max(0, numeric));
    return `${"★".repeat(clamped)}${"☆".repeat(5 - clamped)}`;
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
      if (!detailItem && result?.error) {
        detailError = `Could not load design detail from Rust backend: ${result.error}`;
      }
      detailNotes = String(detailItem?.notes || "");
      detailDesignerId = detailItem?.designer_id == null ? "" : String(detailItem.designer_id);
      detailSourceId = detailItem?.source_id == null ? "" : String(detailItem.source_id);
      detailTagSelection = Array.isArray(detailItem?.tags)
        ? detailItem.tags.map((tag) => Number(tag?.id)).filter((id) => Number.isFinite(id))
        : [];
      detailProjectToAdd = Array.isArray(detailItem?.available_projects) && detailItem.available_projects.length > 0
        ? String(detailItem.available_projects[0].id)
        : "";
      loadDetailBrowseContext(designId);
    } catch (error) {
      detailError = `Could not load design detail: ${error}`;
      detailItem = null;
      detailSource = "mock";
      detailProjectToAdd = "";
      detailBrowseIds = [];
      detailBrowseIndex = -1;
    } finally {
      detailLoading = false;
    }
  }

  async function refreshDetailAfterAction() {
    if (detailDesignId == null && printDesignId == null) {
      return;
    }
    const id = detailDesignId ?? printDesignId;
    await loadDesignDetail(id);
  }

  function setDetailActionNotice(message, isError = false) {
    detailActionMessage = message;
    detailActionIsError = isError;
  }

  async function saveDetailMetadata() {
    if (!detailItem?.id || detailSaving) {
      return;
    }

    detailSaving = true;
    const result = await updateDesignMetadata(detailItem.id, {
      notes: detailNotes,
      designer_id: detailDesignerId ? Number(detailDesignerId) : null,
      source_id: detailSourceId ? Number(detailSourceId) : null,
    });
    detailSaving = false;

    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function submitDetailRating(rating) {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await setDesignRating(detailItem.id, rating);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailStitched() {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await setDesignStitched(detailItem.id, !Boolean(detailItem?.is_stitched));
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailTagsChecked() {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await setDesignTagsChecked(detailItem.id, !Boolean(detailItem?.tags_checked));
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function saveDetailTags() {
    if (!detailItem?.id || detailSaving) {
      return false;
    }
    detailSaving = true;
    const result = await setDesignTags(detailItem.id, detailTagSelection);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
      return true;
    }

    return false;
  }

  async function addDetailToProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await addDesignToProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function addSelectedDetailProject() {
    if (!detailProjectToAdd) {
      return;
    }
    await addDetailToProject(Number(detailProjectToAdd));
  }

  async function removeDetailFromProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await removeDesignFromProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function launchDetailInEditor() {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await openDesignInEditor(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted || !result?.result?.success);
  }

  async function launchDetailInExplorer() {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await openDesignInExplorer(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted || !result?.result?.success);
  }

  async function renderDetail3dPreview() {
    if (!detailItem?.id || detailSaving) {
      return;
    }
    detailSaving = true;
    const result = await renderDesign3dPreview(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      const refreshedImage = await getDesignImageDataUrl(detailItem.id);
      if (refreshedImage?.item?.data_url) {
        detailItem = {
          ...detailItem,
          image_data_url: refreshedImage.item.data_url,
          image_type: refreshedImage.item.image_type || detailItem.image_type,
        };
      }
      await refreshDetailAfterAction();
    }
  }

  async function deleteDetailDesign() {
    if (!detailItem?.id || detailSaving) {
      return;
    }

    const confirmed = window.confirm("Delete this design from the catalogue?");
    if (!confirmed) {
      return;
    }

    detailSaving = true;
    const result = await deleteDesign(detailItem.id);
    detailSaving = false;

    if (result.persisted) {
      navigateTo("#/designs");
      return;
    }

    setDetailActionNotice(result.message, true);
  }

  function openDetailPrintView() {
    if (!detailItem?.id) {
      return;
    }
    navigateTo(`#/designs/${detailItem.id}/print`);
  }

  function printCurrentView() {
    window.print();
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
    const imageTags = Array.isArray(item?.image_tags) ? item.image_tags.map(String).sort((left, right) => left.localeCompare(right)) : [];
    const stitchingTags = Array.isArray(item?.stitching_tags) ? item.stitching_tags.map(String).sort((left, right) => left.localeCompare(right)) : [];
    const fallbackTags = Array.isArray(item?.tags) ? item.tags.map(String) : [];
    const allTags =
      imageTags.length > 0 || stitchingTags.length > 0
        ? Array.from(new Set([...imageTags, ...stitchingTags]))
        : fallbackTags.sort((left, right) => left.localeCompare(right));
    const projectsRaw = Array.isArray(item?.projects)
      ? item.projects
      : Array.isArray(item?.project_names)
        ? item.project_names
        : typeof item?.projects === "string"
          ? item.projects.split(",")
          : typeof item?.project_names === "string"
            ? item.project_names.split(",")
            : [];

    const projects = projectsRaw
      .map((project) => {
        if (typeof project === "string") {
          return project.trim();
        }
        return String(project?.name || "").trim();
      })
      .filter(Boolean);

    return {
      id,
      filename: String(item?.filename || item?.name || `design-${id}.pes`),
      designer: String(item?.designer || "Unknown"),
      source: String(item?.source || "Unknown"),
      tags: allTags,
      imageTags,
      stitchingTags,
      projects,
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

  function updateBrowseDesignerFilter(designer, enabled) {
    const active = new Set(browseFilters.designerFilters || []);
    if (enabled) {
      active.add(designer);
    } else {
      active.delete(designer);
    }
    browseFilters = { ...browseFilters, designerFilters: Array.from(active) };
    browseCurrentPage = 1;
  }

  function updateBrowseSourceFilter(source, enabled) {
    const active = new Set(browseFilters.sourceFilters || []);
    if (enabled) {
      active.add(source);
    } else {
      active.delete(source);
    }
    browseFilters = { ...browseFilters, sourceFilters: Array.from(active) };
    browseCurrentPage = 1;
  }

  function getBrowseTagDropdowns() {
    if (typeof document === "undefined") {
      return [];
    }

    return [
      document.getElementById("browseDesignerDropdown"),
      document.getElementById("browseSourceDropdown"),
      document.getElementById("browseImageTagsDropdown"),
      document.getElementById("browseStitchingTagsDropdown"),
    ].filter(Boolean);
  }

  function getBrowseCardProjectDropdowns() {
    if (typeof document === "undefined") {
      return [];
    }

    return Array.from(document.querySelectorAll(".browse-card-project-details"));
  }

  function closeBrowseTagDropdowns(exceptNode = null) {
    for (const dropdown of getBrowseTagDropdowns()) {
      if (exceptNode && dropdown === exceptNode) {
        continue;
      }

      if (dropdown.hasAttribute("open")) {
        dropdown.removeAttribute("open");
      }
    }
  }

  function closeBrowseCardProjectDropdowns(exceptNode = null) {
    for (const dropdown of getBrowseCardProjectDropdowns()) {
      if (exceptNode && dropdown === exceptNode) {
        continue;
      }

      if (dropdown.hasAttribute("open")) {
        dropdown.removeAttribute("open");
      }
    }
  }

  function summarizeBrowseTagFilters(options, includeUntagged = false) {
    const selected = browseFilters.tagFilters || [];
    const explicit = options.filter((tag) => selected.includes(tag));
    const wantsUntagged = includeUntagged && selected.includes(BROWSE_TAG_UNTAGGED);
    if (!wantsUntagged && explicit.length === 0) {
      return "Any";
    }
    const parts = [];
    if (wantsUntagged) {
      parts.push("Untagged");
    }
    parts.push(...explicit);
    return parts.join(" · ");
  }

  function summarizeBrowseMultiFilters(options, selected) {
    const explicit = options.filter((value) => (selected || []).includes(value));
    if (explicit.length === 0) {
      return "Any";
    }
    return explicit.join(" · ");
  }

  function normalizeBrowseTagGroup(tag) {
    return String(tag?.tag_group ?? tag?.tagGroup ?? "")
      .trim()
      .toLowerCase();
  }

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

    const containerWidth = browseGridContainer?.clientWidth;
    if (containerWidth && containerWidth > 0) {
      browseGridColumns = estimateBrowseColumnsFromWidth(
        Math.max(0, containerWidth + BROWSE_ROW_SELECTOR_WIDTH)
      );
      return;
    }

    browseGridColumns = 2;
  }

  function clearBrowseSelection() {
    browseSelectedIds = [];
  }

  function openBrowseDeleteConfirm() {
    if (browseSelectedIds.length === 0 || browseDeleteSelectedBusy) {
      return;
    }

    browseDeleteConfirmOpen = true;
  }

  function closeBrowseDeleteConfirm(event) {
    event?.preventDefault?.();
    event?.stopPropagation?.();
    event?.stopImmediatePropagation?.();

    if (browseDeleteSelectedBusy) {
      return;
    }

    browseDeleteConfirmOpen = false;
  }

  async function confirmDeleteSelectedBrowseItems() {
    if (browseDeleteSelectedBusy) {
      return;
    }

    const selectedIds = Array.from(
      new Set(
        browseSelectedIds
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
      )
    );

    if (selectedIds.length === 0) {
      browseDeleteConfirmOpen = false;
      return;
    }

    browseDeleteSelectedBusy = true;

    let deletedCount = 0;
    const failedIds = [];
    let firstFailure = "";

    for (const designId of selectedIds) {
      const result = await deleteDesign(designId);
      if (result?.persisted) {
        deletedCount += 1;
      } else {
        failedIds.push(designId);
        if (!firstFailure) {
          firstFailure = String(result?.error || result?.message || "Unknown error");
        }
      }
    }

    browseDeleteSelectedBusy = false;
    browseDeleteConfirmOpen = false;

    if (failedIds.length === 0) {
      browseActionNotice = `Deleted ${deletedCount} design(s) from the database. Files on disk were not deleted.`;
      clearBrowseSelection();
      await loadBrowseItems();
      return;
    }

    browseSelectedIds = failedIds;
    await loadBrowseItems();

    if (deletedCount > 0) {
      browseActionNotice = `Deleted ${deletedCount} of ${selectedIds.length} selected design(s). ${failedIds.length} could not be deleted.${firstFailure ? ` First failure: ${firstFailure}` : ""}`;
      return;
    }

    browseActionNotice = `Could not delete selected designs.${firstFailure ? ` Reason: ${firstFailure}` : ""}`;
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

  function syncBrowseSelectionFromDom() {
    if (typeof document === "undefined") {
      return;
    }

    const ids = Array.from(document.querySelectorAll(".browse-design-checkbox:checked"))
      .map((node) => Number(node.getAttribute("data-design-id")))
      .filter((id) => Number.isFinite(id) && id > 0);

    browseSelectedIds = Array.from(new Set(ids));
  }

  function browseRowSelectionState(rowItems) {
    const rowIds = rowItems.map((item) => item.id);
    const selectedCount = rowIds.filter((id) => browseSelectedIds.includes(id)).length;
    return {
      all: rowIds.length > 0 && selectedCount === rowIds.length,
      partial: selectedCount > 0 && selectedCount < rowIds.length,
    };
  }

  function toggleBrowseRowSelection(rowItems, checked) {
    const rowIds = rowItems.map((item) => item.id);
    if (checked) {
      browseSelectedIds = Array.from(new Set([...browseSelectedIds, ...rowIds]));
      return;
    }

    browseSelectedIds = browseSelectedIds.filter((id) => !rowIds.includes(id));
  }

  function setIndeterminate(node, value) {
    node.indeterminate = Boolean(value);
    return {
      update(nextValue) {
        node.indeterminate = Boolean(nextValue);
      },
    };
  }

  function portalToBody(node) {
    if (typeof document === "undefined") {
      return {};
    }

    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("browse-bulk-bar-portal");

    if (parent) {
      parent.insertBefore(marker, node);
    }

    host.appendChild(node);

    return {
      destroy() {
        if (node.parentNode === host) {
          host.removeChild(node);
        }

        if (marker.parentNode) {
          marker.parentNode.removeChild(marker);
        }
      },
    };
  }

  function openBulkTagModal() {
    if (browseSelectedIds.length === 0) {
      return;
    }
    browseBulkModalMode = "browse";
    browseBulkTagSelection = computeBrowseTagChooserInitialSelection();
    browseBulkTagSelection = browseBulkTagSelection.filter((value) => value !== BROWSE_TAG_UNTAGGED);
    browseBulkModalOpen = true;
  }

  function computeBrowseTagChooserInitialSelection() {
    const selectedIds = new Set(
      browseSelectedIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
    );
    if (selectedIds.size === 0) {
      return [];
    }

    const tagNameToId = new Map(
      (Array.isArray(browseTagOptions) ? browseTagOptions : [])
        .map((option) => [String(option?.description || "").trim().toLowerCase(), Number(option?.id)])
        .filter(([name, id]) => name && Number.isFinite(id) && id > 0)
    );

    const names = new Set();
    const selectedItems = browseCardItems.filter((item) => selectedIds.has(Number(item.id)));
    for (const item of selectedItems) {
      for (const tagName of [...(item.imageTags || []), ...(item.stitchingTags || []), ...(item.tags || [])]) {
        const normalized = String(tagName || "").trim().toLowerCase();
        if (normalized) {
          names.add(normalized);
        }
      }
    }

    const tagIds = [];
    for (const name of names) {
      const id = tagNameToId.get(name);
      if (Number.isFinite(id)) {
        tagIds.push(id);
      }
    }

    return Array.from(new Set(tagIds));
  }

  function openDetailTagModal() {
    if (!detailItem?.id || detailSaving) {
      return;
    }

    browseBulkModalMode = "detail";
    browseBulkModalOpen = true;
  }

  function dismissBulkTagModal(reason = "dismissed") {
    browseBulkModalOpen = false;
  }

  function closeBulkTagModal(event) {
    event?.preventDefault?.();
    event?.stopPropagation?.();
    event?.stopImmediatePropagation?.();
    const currentTarget = event?.currentTarget;
    const target = event?.target;
    const reason = event?.type
      ? `${event.type}${currentTarget?.tagName ? ` from ${String(currentTarget.tagName).toLowerCase()}` : ""}${target?.tagName ? ` targeting ${String(target.tagName).toLowerCase()}` : ""}`
      : "dismissed";
    dismissBulkTagModal(reason);
  }

  $effect(() => {
    return undefined;
  });

  $effect(() => {
    if (typeof document === "undefined") {
      return undefined;
    }

    const onDocumentClick = (event) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }

      const dropdowns = getBrowseTagDropdowns();
      const clickedInside = dropdowns.some((dropdown) => dropdown.contains(target));
      if (!clickedInside) {
        closeBrowseTagDropdowns();
      }

      const projectDropdowns = getBrowseCardProjectDropdowns();
      const clickedInsideProjectDropdown = projectDropdowns.some((dropdown) => dropdown.contains(target));
      if (!clickedInsideProjectDropdown) {
        closeBrowseCardProjectDropdowns();
      }

      const bulkProjectDropdown = document.querySelector(".browse-bulk-project-dropdown");
      if (bulkProjectDropdown instanceof Node && !bulkProjectDropdown.contains(target)) {
        browseBulkProjectDropdownOpen = false;
      }
    };

    const onDetailsToggle = (event) => {
      const target = event.target;
      if (!(target instanceof HTMLElement)) {
        return;
      }

      if (
        (
          target.id === "browseDesignerDropdown" ||
          target.id === "browseSourceDropdown" ||
          target.id === "browseImageTagsDropdown" ||
          target.id === "browseStitchingTagsDropdown"
        ) &&
        target.hasAttribute("open")
      ) {
        closeBrowseTagDropdowns(target);
      }

      if (target.classList.contains("browse-card-project-details") && target.hasAttribute("open")) {
        closeBrowseCardProjectDropdowns(target);
      }
    };

    document.addEventListener("click", onDocumentClick);
    document.addEventListener("toggle", onDetailsToggle, true);

    return () => {
      document.removeEventListener("click", onDocumentClick);
      document.removeEventListener("toggle", onDetailsToggle, true);
    };
  });

  async function applyBulkTags() {
    if (browseSelectedIds.length === 0) {
      closeBulkTagModal();
      return false;
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
      return true;
    }

    browseActionNotice = "Could not save tags in Rust backend. No database changes were committed.";
    closeBulkTagModal();
    return false;
  }

  async function applySharedTagChooser() {
    if (browseBulkModalMode === "detail") {
      const saved = await saveDetailTags();
      if (saved) {
        closeBulkTagModal();
      }
      return;
    }

    await applyBulkTags();
  }

  function tagChooserSelectionIncludes(tagId) {
    const id = Number(tagId);
    if (!Number.isFinite(id)) {
      return false;
    }

    if (browseBulkModalMode === "detail") {
      return detailTagSelection.includes(id);
    }

    return browseBulkTagSelection.includes(id);
  }

  function toggleTagChooserSelection(tagId, checked) {
    const id = Number(tagId);
    if (!Number.isFinite(id)) {
      return;
    }

    if (browseBulkModalMode === "detail") {
      if (checked) {
        detailTagSelection = Array.from(new Set([...detailTagSelection, id]));
      } else {
        detailTagSelection = detailTagSelection.filter((value) => value !== id);
      }
      return;
    }

    if (checked) {
      browseBulkTagSelection = Array.from(new Set([...browseBulkTagSelection, id]));
    } else {
      browseBulkTagSelection = browseBulkTagSelection.filter((value) => value !== id);
    }
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
    const projectIds = (Array.isArray(browseBulkProjectSelection) ? browseBulkProjectSelection : [])
      .map((projectId) => Number(projectId))
      .filter((projectId) => Number.isFinite(projectId) && projectId > 0);

    if (browseSelectedIds.length === 0 || projectIds.length === 0) {
      return;
    }

    let addedCount = 0;
    let firstFailure = "";

    for (const projectId of projectIds) {
      const result = await bulkAddDesignsToProject(projectId, browseSelectedIds);
      if (result?.persisted) {
        addedCount += Number(result?.added_count || 0);
      } else if (!firstFailure) {
        firstFailure = String(result?.error || result?.message || "Unknown error");
      }
    }

    if (addedCount > 0) {
      browseActionNotice = `Added ${addedCount} design(s) across ${projectIds.length} project(s) (saved in Rust backend).`;
      browseBulkProjectDropdownOpen = false;
      await loadBrowseItems();
      return;
    }

    const reason = firstFailure ? ` Reason: ${firstFailure}` : "";
    browseActionNotice = `Could not add selected designs to selected projects.${reason}`;
  }

  function toggleBrowseBulkProjectSelection(projectId, enabled) {
    const normalizedProjectId = Number(projectId);
    if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
      return;
    }

    const active = new Set(
      (Array.isArray(browseBulkProjectSelection) ? browseBulkProjectSelection : [])
        .map((value) => Number(value))
        .filter((value) => Number.isFinite(value) && value > 0)
    );

    if (enabled) {
      active.add(normalizedProjectId);
    } else {
      active.delete(normalizedProjectId);
    }

    browseBulkProjectSelection = Array.from(active);
  }

  function summarizeBrowseBulkProjectSelection() {
    const selected = new Set(
      (Array.isArray(browseBulkProjectSelection) ? browseBulkProjectSelection : [])
        .map((value) => Number(value))
        .filter((value) => Number.isFinite(value) && value > 0)
    );

    if (selected.size === 0) {
      return "Select project";
    }

    const names = browseProjects
      .filter((project) => selected.has(Number(project.id)))
      .map((project) => String(project.name || "").trim())
      .filter(Boolean);

    if (names.length === 0) {
      return `${selected.size} selected`;
    }

    if (names.length <= 2) {
      return names.join(", ");
    }

    return `${names.slice(0, 2).join(", ")} +${names.length - 2}`;
  }

  function toggleBrowseBulkProjectDropdown(event) {
    event?.preventDefault?.();
    event?.stopPropagation?.();
    browseBulkProjectDropdownOpen = !browseBulkProjectDropdownOpen;
  }

  function getBrowseProjectIdsForCard(item) {
    const currentNames = new Set((Array.isArray(item?.projects) ? item.projects : []).map((value) => String(value || "").trim().toLowerCase()));
    return browseProjects
      .filter((project) => currentNames.has(String(project?.name || "").trim().toLowerCase()))
      .map((project) => Number(project.id))
      .filter((projectId) => Number.isFinite(projectId) && projectId > 0);
  }

  function findBrowseCardItemById(designId) {
    const normalizedDesignId = Number(designId);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0) {
      return null;
    }

    return browseCardItems.find((item) => Number(item?.id) === normalizedDesignId) || null;
  }

  function getBrowseCardPendingProjectIds(designId, fallbackItem = null) {
    const key = String(designId);
    if (Array.isArray(browseCardProjectPendingById?.[key])) {
      return browseCardProjectPendingById[key].map((value) => Number(value)).filter((value) => Number.isFinite(value) && value > 0);
    }

    const item = fallbackItem || findBrowseCardItemById(designId);
    return item ? getBrowseProjectIdsForCard(item) : [];
  }

  function setBrowseCardPendingProjectIds(designId, projectIds) {
    const normalizedDesignId = Number(designId);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0) {
      return;
    }

    const key = String(normalizedDesignId);
    const normalizedIds = Array.from(
      new Set((Array.isArray(projectIds) ? projectIds : [])
        .map((projectId) => Number(projectId))
        .filter((projectId) => Number.isFinite(projectId) && projectId > 0))
    );

    browseCardProjectPendingById = {
      ...browseCardProjectPendingById,
      [key]: normalizedIds,
    };
  }

  function primeBrowseCardProjectPendingFromItem(item) {
    const normalizedDesignId = Number(item?.id);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0) {
      return;
    }

    const key = String(normalizedDesignId);
    if (Array.isArray(browseCardProjectPendingById?.[key])) {
      return;
    }

    setBrowseCardPendingProjectIds(normalizedDesignId, getBrowseProjectIdsForCard(item));
  }

  function isBrowseCardProjectChecked(item, projectId) {
    return getBrowseCardPendingProjectIds(item?.id, item).includes(Number(projectId));
  }

  function updateBrowseCardProjectPending(designId, projectId, enabled) {
    const normalizedDesignId = Number(designId);
    const normalizedProjectId = Number(projectId);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0 || !Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
      return;
    }

    const active = new Set(getBrowseCardPendingProjectIds(normalizedDesignId));
    if (enabled) {
      active.add(normalizedProjectId);
    } else {
      active.delete(normalizedProjectId);
    }

    setBrowseCardPendingProjectIds(normalizedDesignId, Array.from(active));
  }

  async function applyBrowseCardProjectPending(designId) {
    const normalizedDesignId = Number(designId);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0) {
      return;
    }

    const item = findBrowseCardItemById(normalizedDesignId);
    if (!item) {
      return;
    }

    const existingProjectIds = new Set(getBrowseProjectIdsForCard(item));
    const desiredProjectIds = new Set(getBrowseCardPendingProjectIds(normalizedDesignId, item));

    const projectIdsToAdd = Array.from(desiredProjectIds).filter((projectId) => !existingProjectIds.has(projectId));
    const projectIdsToRemove = Array.from(existingProjectIds).filter((projectId) => !desiredProjectIds.has(projectId));

    if (projectIdsToAdd.length === 0 && projectIdsToRemove.length === 0) {
      return;
    }

    const key = String(normalizedDesignId);
    let addedCount = 0;
    let removedCount = 0;
    let firstFailure = "";

    for (const projectId of projectIdsToAdd) {
      const result = await addDesignToProject(normalizedDesignId, projectId);
      if (result?.persisted) {
        addedCount += 1;
      } else if (!firstFailure) {
        firstFailure = String(result?.error || result?.message || "Unknown error");
      }
    }

    for (const projectId of projectIdsToRemove) {
      const result = await removeDesignFromProject(normalizedDesignId, projectId);
      if (result?.persisted) {
        removedCount += 1;
      } else if (!firstFailure) {
        firstFailure = String(result?.error || result?.message || "Unknown error");
      }
    }

    browseCardProjectPendingById = {
      ...browseCardProjectPendingById,
      [key]: [],
    };

    if (addedCount > 0 || removedCount > 0) {
      const actions = [];
      if (addedCount > 0) {
        actions.push(`added to ${addedCount} ${addedCount === 1 ? "project" : "projects"}`);
      }
      if (removedCount > 0) {
        actions.push(`removed from ${removedCount} ${removedCount === 1 ? "project" : "projects"}`);
      }
      browseActionNotice = `Design ${actions.join(" and ")} (saved in Rust backend).`;
      await loadBrowseItems();
      return;
    }

    const reason = firstFailure ? ` Reason: ${firstFailure}` : "";
    browseActionNotice = `Could not add design to selected projects.${reason}`;
  }

  function handleBrowseCardProjectDetailsToggle(item, node) {
    if (!node) {
      return;
    }

    const normalizedDesignId = Number(item?.id);
    if (!Number.isFinite(normalizedDesignId) || normalizedDesignId <= 0) {
      return;
    }

    if (node.hasAttribute("open")) {
      primeBrowseCardProjectPendingFromItem(item);
      closeBrowseCardProjectDropdowns(node);
      return;
    }

    void applyBrowseCardProjectPending(normalizedDesignId);
  }

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

  function browseStars(value) {
    const score = Math.max(0, Math.min(5, Number(value || 0)));
    return "★".repeat(score) + "☆".repeat(5 - score);
  }

  let browseCardItems = $derived(browseItems.map((item, index) => normalizeCardItem(item, index)));
  let browseAvailableImageTags = $derived(
    Array.from(
      new Set(
        (Array.isArray(browseTagOptions) ? browseTagOptions : [])
          .filter((tag) => normalizeBrowseTagGroup(tag) === "image")
          .map((tag) => String(tag?.description || "").trim())
          .filter(Boolean)
      )
    ).sort((a, b) => a.localeCompare(b))
  );
  let browseAvailableStitchingTags = $derived(
    Array.from(
      new Set(
        (Array.isArray(browseTagOptions) ? browseTagOptions : [])
          .filter((tag) => normalizeBrowseTagGroup(tag) === "stitching")
          .map((tag) => String(tag?.description || "").trim())
          .filter(Boolean)
      )
    ).sort((a, b) => a.localeCompare(b))
  );
  let browseDesignerSelectOptions = $derived(browseDesignerFilterOptions);
  let browseSourceSelectOptions = $derived(browseSourceFilterOptions);
  let browseAvailableHoops = $derived(browseHoopFilterOptions);

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

        if (
          (browseFilters.designerFilters || []).length > 0 &&
          !(browseFilters.designerFilters || []).includes(item.designer)
        ) {
          return false;
        }

        if (browseFilters.hoop && item.hoop !== browseFilters.hoop) {
          return false;
        }

        if (
          (browseFilters.sourceFilters || []).length > 0 &&
          !(browseFilters.sourceFilters || []).includes(item.source)
        ) {
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

  let browseAllVisibleSelected = $derived(
    browsePageItems.length > 0 && browsePageItems.every((item) => browseSelectedIds.includes(item.id))
  );

  let browseSelectedCount = $derived(browseSelectedIds.length);
  let showBrowseBulkBar = $derived(currentUiKind === "browse" && browseSelectedCount > 0);

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
    if (isImportRoute(currentRoute)) {
      loadImportReferenceData();
      if (!settingsLoaded && !settingsLoading) {
        loadSettingsFromBackend();
      }
    }
  });

  $effect(() => {
    if (!isImportRoute(currentRoute)) {
      importHasAppliedSavedRoot = false;
      return;
    }

    if (!settingsLoaded || importHasAppliedSavedRoot) {
      return;
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
    if (currentRoute === "#/designs" && !browseProjectsLoaded && !browseProjectsLoading) {
      loadBrowseProjects();
    }
  });

  $effect(() => {
    if (currentRoute !== "#/designs") {
      browseProjectsLoaded = false;
    }
  });

  $effect(() => {
    if ((currentRoute === "#/projects" || currentRoute === "#/projects/new") && !projectsLoaded && !projectsLoading) {
      loadProjects();
    }
  });

  $effect(() => {
    if (!projectsLoading) {
      return;
    }

    const activeToken = projectsLoadRequestToken;
    const watchdogHandle = setTimeout(() => {
      if (projectsLoading && projectsLoadRequestToken === activeToken) {
        projectsLoading = false;
        projectsLoaded = true;
        if (!projectsError) {
          projectsError = "Could not load projects: Timed out loading projects after 15s.";
        }
      }
    }, 16000);

    return () => clearTimeout(watchdogHandle);
  });

  $effect(() => {
    if (projectDetailId !== null) {
      loadProjectDetailView(projectDetailId);
    }
  });

  $effect(() => {
    if (projectPrintId !== null) {
      loadProjectPrint(projectPrintId);
    }
  });

  $effect(() => {
    if (currentUiKind !== "project-detail") {
      projectDetail = null;
      projectDetailError = "";
      projectDetailName = "";
      projectDetailDescription = "";
    }
  });

  $effect(() => {
    if (currentUiKind !== "project-print") {
      projectPrint = null;
      projectPrintError = "";
    }
  });

  $effect(() => {
    if (currentRoute === "#/designs" && browseTagOptions.length === 0) {
      loadBrowseTags();
    }
  });

  $effect(() => {
    if (currentRoute === "#/designs" && !browseFilterReferenceLoaded) {
      loadBrowseFilterReferenceData();
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
    const id = detailDesignId ?? printDesignId;
    if (id !== null) {
      loadDesignDetail(id);
      return;
    }

    detailActionMessage = "";
    detailActionIsError = false;
    detailProjectToAdd = "";
    detailBrowseIds = [];
    detailBrowseIndex = -1;
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
    if (currentRoute === "#/admin/maintenance/backup" && !backupLoaded && !backupLoading) {
      loadBackupFromBackend();
    }
  });

  $effect(() => {
    if (currentRoute === "#/admin/tagging-actions" && !taggingActionsLoaded && !taggingActionsLoading) {
      loadTaggingActionsViewModel();
      refreshBackfillLogEntries();
    }
  });

  $effect(() => {
    if (currentRoute === "#/admin/orphans" && !orphansLoaded && !orphansLoading) {
      loadOrphansPage(1);
    }
  });

  $effect(() => {
    if (currentUiKind === "about") {
      loadAboutDocuments();
    }
  });

  $effect(() => {
    const slug = aboutDocumentSlug;
    if (slug !== null) {
      loadAboutDocumentView(slug);
      return;
    }

    aboutDocumentItem = null;
    aboutDocumentError = "";
    aboutDocumentLoadedSlug = "";
  });

  $effect(() => {
    if (currentUiKind === "admin-list") {
      loadAdminDataForCurrentRoute();
    }
  });

  $effect(() => {
    if (currentRoute !== "#/admin/tags" || adminTagsPanelStateLoaded || typeof window === "undefined") {
      return;
    }

    const imageSavedState = window.localStorage.getItem("admin.tags.collapsible.image");
    const stitchingSavedState = window.localStorage.getItem("admin.tags.collapsible.stitching");
    if (imageSavedState === "open" || imageSavedState === "closed") {
      adminImageTagsOpen = imageSavedState === "open";
    }
    if (stitchingSavedState === "open" || stitchingSavedState === "closed") {
      adminStitchingTagsOpen = stitchingSavedState === "open";
    }

    adminTagsPanelStateLoaded = true;
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

  $effect(() => {
    if (currentRoute !== "#/designs" || typeof document === "undefined") {
      return;
    }

    const onChange = (event) => {
      const target = event.target;
      if (!(target instanceof HTMLElement)) {
        return;
      }

      if (
        target.classList.contains("browse-design-checkbox") ||
        target.classList.contains("browse-row-checkbox") ||
        target.id === "browse-select-all-visible"
      ) {
        setTimeout(syncBrowseSelectionFromDom, 0);
      }
    };

    document.addEventListener("change", onChange);
    setTimeout(syncBrowseSelectionFromDom, 0);

    return () => {
      document.removeEventListener("change", onChange);
    };
  });

  $effect(() => {
    return undefined;
  });

  syncRouteFromHash();
</script>

<svelte:window onhashchange={syncRouteFromHash} onresize={refreshBrowseGridColumns} />

<nav class="menu-shell text-white shadow">
  <div class="menu-shell-inner max-w-7xl mx-auto">
    <div class="menu-primary-group">
      <a href="#/designs" class="menu-brand">
        <span aria-hidden="true">🧵</span>
        <span>Embroidery Catalogue</span>
      </a>
      <a href="#/designs" class={linkClass("#/designs")}>Browse</a>
      <a href="#/import" class={linkClass("#/import")}>Import</a>
      <a href="#/projects" class={linkClass("#/projects")}>Projects</a>
      <a href="#/help" class={linkClass("#/help")}>Help</a>
    </div>

    <div class="menu-admin-group">
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
    <section class="browse-section space-y-3">
      <h1 class="ui-page-title browse-title">Browse Designs</h1>

      <form
        class="browse-search-shell space-y-3 no-print"
        onsubmit={(event) => {
          event.preventDefault();
          applyBrowseFilters();
        }}
      >
        <div class="ui-section-shell browse-general-search space-y-1.5">
          <label class="ui-section-label browse-general-search-label block" for="browse-q">General search</label>
          <div class="browse-general-search-row flex items-center gap-2">
            <input
              id="browse-q"
              class="ui-text-input ui-control-text-inset browse-general-input text-sm flex-1 min-w-[20rem] font-mono"
              placeholder='e.g. rose "cross stitch" -applique or *.hus'
              value={browseFilters.q}
              oninput={(event) => updateBrowseFilter("q", event.currentTarget.value)}
            />
            <label class="ui-field-label browse-unverified-label flex items-center gap-1.5 cursor-pointer select-none whitespace-nowrap">
              <input
                type="checkbox"
                class="ui-checkbox browse-unverified-checkbox"
                checked={browseFilters.unverifiedOnly}
                onchange={(event) => updateBrowseFilter("unverifiedOnly", event.currentTarget.checked)}
              />
              Unverified only
            </label>
          </div>
          <p class="ui-help-note browse-general-help mt-0.5">
            Supports Google-like syntax: "exact phrase" · -exclude · word1 OR word2 · *.hus ·
            <a href="#search" class="ui-app-link">Search help</a>
          </p>
        </div>

        <details class="ui-section-shell browse-additional-filters overflow-visible relative" open={browseAdditionalFiltersOpen}>
          <summary
            class="ui-section-label browse-additional-summary cursor-pointer"
            onclick={(event) => {
              event.preventDefault();
              toggleAdditionalFilters();
            }}
          >
            Additional filters
          </summary>

          {#if browseAdditionalFiltersOpen}
            <div class="p-4 space-y-4">
              <div class="browse-two-col-grid max-w-3xl">
                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">All of these words</span>
                  <input
                    class="ui-text-input ui-control-text-inset text-sm w-full"
                    value={browseFilters.allWords}
                    oninput={(event) => updateBrowseFilter("allWords", event.currentTarget.value)}
                  />
                </label>
                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">This exact phrase</span>
                  <input
                    class="ui-text-input ui-control-text-inset text-sm w-full"
                    value={browseFilters.exactPhrase}
                    oninput={(event) => updateBrowseFilter("exactPhrase", event.currentTarget.value)}
                  />
                </label>
                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Any of these words</span>
                  <input
                    class="ui-text-input ui-control-text-inset text-sm w-full"
                    value={browseFilters.anyWords}
                    oninput={(event) => updateBrowseFilter("anyWords", event.currentTarget.value)}
                  />
                </label>
                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">None of these words</span>
                  <input
                    class="ui-text-input ui-control-text-inset text-sm w-full"
                    value={browseFilters.noneWords}
                    oninput={(event) => updateBrowseFilter("noneWords", event.currentTarget.value)}
                  />
                </label>
                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Filename</span>
                  <input
                    class="ui-text-input ui-control-text-inset text-sm w-full font-mono"
                    placeholder="e.g. rose* or *.jef"
                    value={browseFilters.filename}
                    oninput={(event) => updateBrowseFilter("filename", event.currentTarget.value)}
                  />
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Designer</span>
                  <details class="ui-multi-dropdown text-sm" id="browseDesignerDropdown">
                    <summary class="ui-text-input ui-control-text-inset ui-multi-dropdown-summary list-none cursor-pointer">
                      <span class="ui-multi-dropdown-summary-text">
                        {summarizeBrowseMultiFilters(browseDesignerSelectOptions, browseFilters.designerFilters)}
                      </span>
                      <span class="ui-control-caret" aria-hidden="true"></span>
                    </summary>
                    <div class="ui-checkbox-list-shell ui-multi-dropdown-panel px-3 py-2 space-y-1 max-h-56 overflow-auto">
                      {#if browseDesignerSelectOptions.length === 0}
                        <p class="ui-help-note">No designers yet.</p>
                      {:else}
                        {#each browseDesignerSelectOptions as designer}
                          <label class="ui-field-label flex items-center gap-1.5">
                            <input
                              type="checkbox"
                              class="ui-checkbox"
                              checked={browseFilters.designerFilters.includes(designer)}
                              onchange={(event) => updateBrowseDesignerFilter(designer, event.currentTarget.checked)}
                            />
                            <span>{designer}</span>
                          </label>
                        {/each}
                      {/if}
                    </div>
                  </details>
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Hoop</span>
                  <select
                    class="ui-select-input ui-control-text-inset text-sm w-full"
                    value={browseFilters.hoop}
                    onchange={(event) => updateBrowseFilter("hoop", event.currentTarget.value)}
                  >
                    <option value="">Any</option>
                    {#each browseAvailableHoops as hoop}
                      <option value={hoop}>{hoop}</option>
                    {/each}
                  </select>
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Source</span>
                  <details class="ui-multi-dropdown text-sm" id="browseSourceDropdown">
                    <summary class="ui-text-input ui-control-text-inset ui-multi-dropdown-summary list-none cursor-pointer">
                      <span class="ui-multi-dropdown-summary-text">
                        {summarizeBrowseMultiFilters(browseSourceSelectOptions, browseFilters.sourceFilters)}
                      </span>
                      <span class="ui-control-caret" aria-hidden="true"></span>
                    </summary>
                    <div class="ui-checkbox-list-shell ui-multi-dropdown-panel px-3 py-2 space-y-1 max-h-56 overflow-auto">
                      {#if browseSourceSelectOptions.length === 0}
                        <p class="ui-help-note">No sources yet.</p>
                      {:else}
                        {#each browseSourceSelectOptions as source}
                          <label class="ui-field-label flex items-center gap-1.5">
                            <input
                              type="checkbox"
                              class="ui-checkbox"
                              checked={browseFilters.sourceFilters.includes(source)}
                              onchange={(event) => updateBrowseSourceFilter(source, event.currentTarget.checked)}
                            />
                            <span>{source}</span>
                          </label>
                        {/each}
                      {/if}
                    </div>
                  </details>
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Image tags</span>
                  <details class="ui-multi-dropdown text-sm" id="browseImageTagsDropdown">
                    <summary class="ui-text-input ui-control-text-inset ui-multi-dropdown-summary list-none cursor-pointer">
                      <span class="ui-multi-dropdown-summary-text">
                        {summarizeBrowseTagFilters(browseAvailableImageTags, true)}
                      </span>
                      <span class="ui-control-caret" aria-hidden="true"></span>
                    </summary>
                    <div class="ui-checkbox-list-shell ui-multi-dropdown-panel px-3 py-2 space-y-1 max-h-56 overflow-auto">
                      <label class="ui-field-label flex items-center gap-1.5">
                        <input
                          type="checkbox"
                          class="ui-checkbox"
                          checked={browseFilters.tagFilters.includes(BROWSE_TAG_UNTAGGED)}
                          onchange={(event) => updateBrowseTagFilter(BROWSE_TAG_UNTAGGED, event.currentTarget.checked)}
                        />
                        Untagged
                      </label>
                      {#each browseAvailableImageTags as tag}
                        <label class="ui-field-label flex items-center gap-1.5">
                          <input
                            type="checkbox"
                            class="ui-checkbox"
                            checked={browseFilters.tagFilters.includes(tag)}
                            onchange={(event) => updateBrowseTagFilter(tag, event.currentTarget.checked)}
                          />
                          <span>{tag}</span>
                        </label>
                      {/each}
                    </div>
                  </details>
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Stitching tags</span>
                  <details class="ui-multi-dropdown text-sm" id="browseStitchingTagsDropdown">
                    <summary class="ui-text-input ui-control-text-inset ui-multi-dropdown-summary list-none cursor-pointer">
                      <span class="ui-multi-dropdown-summary-text">
                        {summarizeBrowseTagFilters(browseAvailableStitchingTags, false)}
                      </span>
                      <span class="ui-control-caret" aria-hidden="true"></span>
                    </summary>
                    <div class="ui-checkbox-list-shell ui-multi-dropdown-panel px-3 py-2 space-y-1 max-h-56 overflow-auto">
                      {#if browseAvailableStitchingTags.length === 0}
                        <p class="ui-help-note">No stitching tags yet.</p>
                      {:else}
                        {#each browseAvailableStitchingTags as tag}
                          <label class="ui-field-label flex items-center gap-1.5">
                            <input
                              type="checkbox"
                              class="ui-checkbox"
                              checked={browseFilters.tagFilters.includes(tag)}
                              onchange={(event) => updateBrowseTagFilter(tag, event.currentTarget.checked)}
                            />
                            <span>{tag}</span>
                          </label>
                        {/each}
                      {/if}
                    </div>
                  </details>
                </label>

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Min Rating</span>
                  <select
                    class="ui-select-input ui-control-text-inset text-sm w-full"
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

                <label class="ui-field-label text-sm">
                  <span class="block font-medium mb-1">Stitched</span>
                  <select
                    class="ui-select-input ui-control-text-inset text-sm w-full"
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

        <div class="ui-section-shell p-3 flex flex-wrap gap-4 items-end">
          <fieldset class="ui-field-label text-sm mr-8">
            <legend class="block font-medium px-1">Search in</legend>
            <div class="flex min-h-[2rem] flex-wrap items-center gap-3">
              <label class="ui-field-label flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  class="ui-checkbox"
                  checked={browseFilters.searchFilename}
                  onchange={(event) => updateBrowseFilter("searchFilename", event.currentTarget.checked)}
                />
                File name
              </label>
              <label class="ui-field-label flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  class="ui-checkbox"
                  checked={browseFilters.searchTags}
                  onchange={(event) => updateBrowseFilter("searchTags", event.currentTarget.checked)}
                />
                Tags
              </label>
              <label class="ui-field-label flex items-center gap-1.5 text-sm">
                <input
                  type="checkbox"
                  class="ui-checkbox"
                  checked={browseFilters.searchFolder}
                  onchange={(event) => updateBrowseFilter("searchFolder", event.currentTarget.checked)}
                />
                Folder name
              </label>
            </div>
          </fieldset>

          <div class="flex items-end gap-6 pl-10">
            <fieldset class="ui-field-label text-sm">
              <legend class="block font-medium mb-1">Sort by</legend>
              <div class="flex min-h-[2rem] flex-wrap items-center gap-3">
                <label class="ui-field-label flex items-center gap-1.5 text-sm">
                  <input
                    type="radio"
                    class="ui-radio"
                    name="browse-sort-by"
                    checked={browseFilters.sortBy === "name"}
                    onchange={() => updateBrowseFilter("sortBy", "name")}
                  />
                  Design name
                </label>
                <label class="ui-field-label flex items-center gap-1.5 text-sm">
                  <input
                    type="radio"
                    class="ui-radio"
                    name="browse-sort-by"
                    checked={browseFilters.sortBy === "folder"}
                    onchange={() => updateBrowseFilter("sortBy", "folder")}
                  />
                  Folder
                </label>
                <label class="ui-field-label flex items-center gap-1.5 text-sm">
                  <input
                    type="radio"
                    class="ui-radio"
                    name="browse-sort-by"
                    checked={browseFilters.sortBy === "date_added"}
                    onchange={() => updateBrowseFilter("sortBy", "date_added")}
                  />
                  Date added
                </label>
              </div>
            </fieldset>

            <fieldset class="ui-field-label text-sm">
              <legend class="block font-medium mb-1">Order</legend>
              <div class="flex min-h-[2rem] flex-wrap items-center gap-3">
                <label class="ui-field-label flex items-center gap-1.5 text-sm">
                  <input
                    type="radio"
                    class="ui-radio"
                    name="browse-sort-dir"
                    checked={browseFilters.sortDir === "asc"}
                    onchange={() => updateBrowseFilter("sortDir", "asc")}
                  />
                  Ascending
                </label>
                <label class="ui-field-label flex items-center gap-1.5 text-sm">
                  <input
                    type="radio"
                    class="ui-radio"
                    name="browse-sort-dir"
                    checked={browseFilters.sortDir === "desc"}
                    onchange={() => updateBrowseFilter("sortDir", "desc")}
                  />
                  Descending
                </label>
              </div>
            </fieldset>
          </div>

          <div class="ml-auto flex gap-3 items-center self-end">
            <button type="submit" class="menu-button-primary ui-action-button ui-action-button-primary browse-search-submit-button">Search</button>
            <button
              type="button"
              class="menu-button-primary ui-action-button ui-action-button-primary browse-search-reset-button"
              disabled={!hasActiveBrowseFilters()}
              aria-disabled={!hasActiveBrowseFilters()}
              onclick={resetBrowseFilters}
            >
              Reset
            </button>
          </div>
        </div>
      </form>

      <div class="flex items-center justify-between text-sm text-gray-600">
        <span>{browseTotal} design{browseTotal === 1 ? "" : "s"} found — page {browseCurrentPage} of {browseTotalPages}</span>
        {#if browsePageItems.length > 0}
          <label class="ui-field-label flex items-center gap-1 cursor-pointer select-none">
            <input
              type="checkbox"
              id="browse-select-all-visible"
              checked={browseAllVisibleSelected}
              oninput={() => toggleBrowseSelectAllVisible(!browseAllVisibleSelected)}
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
        <div class="browse-grid-rows" bind:this={browseGridContainer}>
          {#each browsePageRows as rowItems, rowIndex}
            {@const rowSelection = browseRowSelectionState(rowItems)}
            <div class="browse-grid-row">
              <label class="browse-row-selector no-print" title={`Select row ${rowIndex + 1}`}>
                <span class="sr-only">Select row {rowIndex + 1}</span>
                <input
                  type="checkbox"
                  class="browse-row-checkbox rounded accent-indigo-500"
                  checked={rowSelection.all}
                  use:setIndeterminate={rowSelection.partial}
                  oninput={() => toggleBrowseRowSelection(rowItems, !rowSelection.all)}
                />
              </label>

              <div class="browse-card-grid" style={`--browse-grid-columns: ${Math.max(2, browseGridColumns || 2)};`}>
                {#each rowItems as item}
                  <article class="browse-design-card bg-white rounded shadow hover:shadow-md overflow-hidden flex flex-col relative">
                    <label class="absolute top-1 left-1 z-10 cursor-pointer bg-white/90 rounded px-1">
                      <span class="sr-only">Select {item.filename}</span>
                      <input
                        type="checkbox"
                        class="browse-design-checkbox"
                        data-design-id={item.id}
                        checked={browseSelectedIds.includes(item.id)}
                        oninput={() => toggleBrowseCardSelection(item.id, !browseSelectedIds.includes(item.id))}
                      />
                    </label>

                    <button class="browse-card-link w-full text-left" onclick={(event) => handleBrowseCardOpenDetail(event, item)}>
                      {#if browsePreviewById[item.id]}
                        <div class="browse-card-image-frame bg-gray-100 p-1">
                          <img
                            src={browsePreviewById[item.id]}
                            alt={item.filename}
                            class="browse-card-image"
                            loading="lazy"
                          />
                        </div>
                      {:else}
                        <div class="browse-card-image-frame bg-gray-100 p-1 flex items-center justify-center text-xs text-gray-400">
                          {browsePreviewsLoading ? "Loading image..." : "No image"}
                        </div>
                      {/if}
                      <div class="browse-card-meta p-4">
                        <div class="browse-card-title-row flex items-start justify-between gap-1">
                          <p class="browse-card-title ui-field-label text-xs font-medium truncate flex-1" title={item.filename}>{item.filename}</p>
                          <span
                            class={`browse-card-verified text-xs leading-none ${item.tagsChecked ? "browse-card-verified-ok" : "browse-card-verified-missing"}`}
                            title={item.tagsChecked ? "Verified" : "Not verified"}
                            aria-label={item.tagsChecked ? "Verified" : "Not verified"}
                          >
                            {item.tagsChecked ? "✓" : "x"}
                          </span>
                        </div>
                        <p class="browse-card-hoop ui-field-label text-xs text-indigo-600">{item.hoop || "Hoop unknown"}</p>
                        {#if item.projects.length > 0}
                          <p class="browse-card-projects ui-field-label text-xs text-gray-500" title={item.projects.join(", ")}>
                            {item.projects.join(", ")}
                          </p>
                        {/if}
                        {#if item.imageTags.length > 0 || item.stitchingTags.length > 0 || item.tags.length > 0}
                          <p class="browse-card-tags ui-field-label text-xs" title={item.tags.join(", ")}>
                            {item.tags.join(", ")}
                          </p>
                        {:else}
                          <p class="browse-card-tags ui-field-label text-xs text-gray-300 italic">No tags</p>
                        {/if}
                        <p class="browse-card-rating text-xs text-gray-400" aria-label={`Rating ${item.rating ?? 0} out of 5`}>
                          {browseStars(item.rating ?? 0)}
                        </p>
                      </div>
                    </button>

                    {#if browseProjects.length > 0}
                      <details
                        class="browse-card-project-details px-4 py-2 no-print"
                        ontoggle={(event) => handleBrowseCardProjectDetailsToggle(item, event.currentTarget)}
                      >
                        <summary class="browse-card-project-summary ui-field-label text-xs text-gray-400 cursor-pointer hover:text-indigo-600 select-none">
                          + Add to project
                        </summary>
                        <div class="ui-checkbox-list-shell mt-1 max-h-40 overflow-auto px-2 py-1 space-y-1">
                          {#each browseProjects as project}
                            <label class="ui-field-label flex items-center gap-1.5 text-xs">
                              <input
                                type="checkbox"
                                class="ui-checkbox"
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
              </div>
            </div>
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

    </section>
  {:else if currentUiKind === "settings"}
    <section class="settings-page space-y-6">
      <h1 class="ui-page-title settings-title mb-6">Application Settings</h1>

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
      <h1 class="ui-page-title backup-title mb-2">Backup</h1>
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
            disabled={!backupHasDbDestination || backupAnyRunning}
            title={!backupHasDbDestination ? "Set a database backup destination first" : undefined}
            onclick={() => runBackupActionUiOnly("database")}
          >
            {backupDatabaseRunning ? "Backing up database..." : "Backup database now"}
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
            disabled={!backupHasDesignsDestination || backupAnyRunning}
            title={!backupHasDesignsDestination ? "Set a designs backup destination first" : undefined}
            onclick={() => runBackupActionUiOnly("designs")}
          >
            {backupDesignsRunning ? "Running incremental backup..." : "Run incremental backup"}
          </button>
        </div>

        <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
          <h2 class="text-base font-semibold text-gray-800">Backup Both</h2>
          <p class="text-sm text-gray-600">Run the database backup and the incremental designs backup in one step.</p>
          <button
            type="button"
            class="settings-primary-button menu-button-primary"
            disabled={!backupHasDbDestination || !backupHasDesignsDestination || backupAnyRunning}
            title={!backupHasDbDestination || !backupHasDesignsDestination ? "Set both backup destinations first" : undefined}
            onclick={() => runBackupActionUiOnly("both")}
          >
            {backupAnyRunning ? "Backup in progress..." : "Run both backups"}
          </button>
        </div>
      </div>
    </section>
  {:else if currentPage}
    <div class={`bg-white rounded-xl shadow p-6 space-y-4 ${currentUiKind === "projects-list" || currentUiKind === "project-new" || currentUiKind === "project-detail" || currentUiKind === "project-print" || currentUiKind === "about" || currentUiKind === "about-document" ? "bg-transparent rounded-none shadow-none p-0" : ""}`}>
      {#if currentUiKind !== "import" && currentUiKind !== "projects-list" && currentUiKind !== "project-new" && currentUiKind !== "project-detail" && currentUiKind !== "project-print" && currentUiKind !== "about" && currentUiKind !== "about-document" && !adminIsTagsRoute && !adminIsSourcesRoute && !adminIsHoopsRoute}
        <h1 class="ui-page-title">{currentPage.title}</h1>
        {#if currentPage.subtitle}
          <p class="text-sm uppercase tracking-wide text-indigo-600 font-semibold">{currentPage.subtitle}</p>
        {/if}
        {#if currentPage.description}
          <p class="text-gray-600">{currentPage.description}</p>
        {/if}
      {/if}

      {#if currentUiKind === "design-detail"}
        <div class="space-y-4">
          <div class="flex flex-wrap gap-2">
            <button class="menu-button-secondary" onclick={() => navigateTo("#/designs")}>Back to Browse</button>
            <button class="menu-button-secondary" onclick={openDetailPrintView} disabled={!detailItem}>Print View</button>
            <button class="menu-button-secondary" onclick={goToPreviousDetail} disabled={detailBrowseIndex <= 0}>‹ Prev</button>
            <button class="menu-button-secondary" onclick={goToNextDetail} disabled={detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1}>Next ›</button>
            {#if detailBrowseIndex >= 0 && detailBrowseIds.length > 0}
              <span class="text-sm text-gray-500 self-center">{detailBrowseIndex + 1} / {detailBrowseIds.length}</span>
            {/if}
          </div>

          {#if detailActionMessage}
            <div class={`rounded border px-3 py-2 text-sm ${detailActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
              {detailActionMessage}
            </div>
          {/if}

          <div class="route-panel">
            <p class="font-semibold">Data source: {detailSource}</p>
            {#if detailLoading}
              <p>Loading design detail...</p>
            {:else if detailError}
              <p class="text-red-600">{detailError}</p>
            {:else if !detailItem}
              <p>No design found for id {detailDesignId}.</p>
            {:else}
              <div class="grid lg:grid-cols-2 gap-4 mt-3">
                <div class="space-y-3">
                  <div class="route-card">
                    <strong>Filename:</strong> {detailItem.filename || "Unknown"}
                  </div>
                  <div class="route-card break-all">
                    <strong>Path:</strong> {detailItem.filepath || "Unknown"}
                  </div>
                  {#if detailItem.image_data_url}
                    <img
                      src={detailItem.image_data_url}
                      alt={detailItem.filename || "Design preview"}
                      class="w-full rounded border border-gray-200 bg-white p-2 max-h-[24rem] object-contain"
                    />
                  {:else}
                    <div class="route-card text-gray-500">No preview image saved yet.</div>
                  {/if}

                  <div class="flex flex-wrap gap-2">
                    <button class="menu-button-secondary" onclick={launchDetailInEditor} disabled={detailSaving}>Open in Editor</button>
                    <button class="menu-button-secondary" onclick={launchDetailInExplorer} disabled={detailSaving}>Show in Explorer</button>
                    <button class="menu-button-primary" onclick={renderDetail3dPreview} disabled={detailSaving}>
                      {detailItem.image_data_url ? (detailItem.image_type === "3d" ? "✓ 3D Preview" : "Render 3D Preview") : "Generate 3D Preview"}
                    </button>
                  </div>
                </div>

                <div class="space-y-3">
                  <div class="grid sm:grid-cols-2 gap-3">
                    <div class="route-card"><strong>Designer:</strong> {detailItem.designer || "Unknown"}</div>
                    <div class="route-card"><strong>Source:</strong> {detailItem.source || "Unknown"}</div>
                    <div class="route-card"><strong>Hoop:</strong> {detailItem.hoop || "Unknown"}</div>
                    <div class="route-card"><strong>Date added:</strong> {detailItem.date_added || "Unknown"}</div>
                    <div class="route-card"><strong>Dimensions:</strong> {detailItem.width_mm ?? "?"} x {detailItem.height_mm ?? "?"} mm</div>
                    <div class="route-card"><strong>Stitches:</strong> {detailItem.stitch_count ?? "?"}</div>
                      <div class="route-card"><strong>Colours:</strong> {detailItem.color_count ?? "?"}</div>
                      <div class="route-card"><strong>Colour changes:</strong> {detailItem.color_change_count ?? "?"}</div>
                  </div>

                    {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                      <div class="flex flex-wrap gap-2 items-center">
                        {#each detailItem.tags as tag}
                          <span class={`text-xs px-2 py-0.5 rounded-full ${tag.tag_group === "stitching" ? "bg-blue-100 text-blue-700" : tag.tag_group === "image" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-700"}`}>
                            {tag.description}
                          </span>
                        {/each}
                      </div>
                    {/if}

                  <div class="route-panel space-y-2">
                    <p class="font-semibold">Rating and stitched status</p>
                    <div class="flex flex-wrap gap-2 items-center">
                      <span class="text-sm text-gray-600">Current rating: {detailItem.rating ?? "None"}</span>
                      {#each [1, 2, 3, 4, 5] as score}
                        <button
                          class="menu-button-secondary"
                          onclick={() => submitDetailRating(score)}
                          disabled={detailSaving}
                        >
                          {score}★
                        </button>
                      {/each}
                      {#if detailItem.rating}
                        <button class="menu-button-secondary" onclick={() => submitDetailRating(null)} disabled={detailSaving}>Clear rating</button>
                      {/if}
                    </div>
                    <div class="flex flex-wrap gap-2 items-center">
                      <button class="menu-button-secondary" onclick={toggleDetailStitched} disabled={detailSaving}>
                        {detailItem.is_stitched ? "✓ Mark as Not Stitched" : "Mark as Stitched"}
                      </button>
                      {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                        <button class="menu-button-secondary" onclick={toggleDetailTagsChecked} disabled={detailSaving}>
                          {detailItem.tags_checked ? "✓ Verified" : "⚠ Verify"}
                        </button>
                      {/if}
                    </div>
                  </div>

                  <form
                    class="route-panel space-y-2"
                    onsubmit={(event) => {
                      event.preventDefault();
                      saveDetailMetadata();
                    }}
                  >
                    <p class="font-semibold">Metadata</p>
                    <label class="block text-sm text-gray-700">
                      <span class="block mb-1">Notes</span>
                      <textarea class="w-full border rounded px-2 py-1" rows="4" bind:value={detailNotes}></textarea>
                    </label>
                    <div class="grid sm:grid-cols-2 gap-2">
                      <label class="text-sm text-gray-700">
                        <span class="block mb-1">Designer</span>
                        <select class="w-full border rounded px-2 py-1" bind:value={detailDesignerId}>
                          <option value="">None</option>
                          {#each detailItem.designers || [] as designer}
                            <option value={String(designer.id)}>{designer.name}</option>
                          {/each}
                        </select>
                      </label>
                      <label class="text-sm text-gray-700">
                        <span class="block mb-1">Source</span>
                        <select class="w-full border rounded px-2 py-1" bind:value={detailSourceId}>
                          <option value="">None</option>
                          {#each detailItem.sources || [] as source}
                            <option value={String(source.id)}>{source.name}</option>
                          {/each}
                        </select>
                      </label>
                    </div>
                    <button type="submit" class="menu-button-primary" disabled={detailSaving}>Save metadata</button>
                  </form>

                  <div class="route-panel space-y-2">
                    <p class="font-semibold">Projects</p>
                    {#if Array.isArray(detailItem.projects) && detailItem.projects.length > 0}
                      <div class="space-y-1">
                        {#each detailItem.projects as project}
                          <div class="flex items-center justify-between border rounded px-2 py-1">
                            <span>{project.name}</span>
                            <button class="menu-button-secondary" onclick={() => removeDetailFromProject(project.id)} disabled={detailSaving}>Remove</button>
                          </div>
                        {/each}
                      </div>
                    {:else}
                      <p class="text-sm text-gray-500">Not assigned to any project.</p>
                    {/if}

                    {#if Array.isArray(detailItem.available_projects) && detailItem.available_projects.length > 0}
                      <div class="flex flex-col sm:flex-row gap-2">
                        <select class="w-full sm:flex-1 border rounded px-2 py-1" bind:value={detailProjectToAdd} disabled={detailSaving}>
                          {#each detailItem.available_projects as project}
                            <option value={String(project.id)}>{project.name}</option>
                          {/each}
                        </select>
                        <button class="menu-button-primary" onclick={addSelectedDetailProject} disabled={detailSaving || !detailProjectToAdd}>
                          Add to Project
                        </button>
                      </div>
                    {/if}
                  </div>
                  <details class="route-panel space-y-2" open>
                    <summary class="font-semibold cursor-pointer">
                      Tags
                      {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                        <span class="ml-2 text-xs text-gray-500">({detailItem.tags.length} assigned: {detailItem.tags.map((tag) => tag.description).join(", ")})</span>
                      {:else}
                        <span class="ml-2 text-xs text-gray-400">(none assigned)</span>
                      {/if}
                      <span class={`ml-2 text-xs ${detailItem.tags_checked ? "text-green-600" : "text-orange-500"}`}>
                        {detailItem.tags_checked ? "✓ Tags checked" : "⚠ Tags not verified"}
                      </span>
                    </summary>
                    <div class="space-y-3 pt-2">
                      <div class="flex items-center gap-2">
                        <button class="menu-button-primary" onclick={openDetailTagModal} disabled={detailSaving}>Choose tags...</button>
                        <button class="menu-button-secondary" onclick={saveDetailTags} disabled={detailSaving}>Save tags</button>
                        <span class="text-xs text-gray-500">Saving tags marks this design as verified. Choose tags opens the same tag chooser used in Browse.</span>
                      </div>
                    </div>
                  </details>

                  <div class="flex justify-end">
                    <button class="menu-button-secondary" onclick={deleteDetailDesign} disabled={detailSaving}>Delete design</button>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      {:else if currentUiKind === "design-print"}
        <div class="space-y-3">
          <div class="flex flex-wrap gap-2 no-print">
            <button class="menu-button-secondary" onclick={() => navigateTo(`#/designs/${printDesignId}`)}>Back to Detail</button>
            <button class="menu-button-primary" onclick={printCurrentView}>Print</button>
          </div>

          <div class="route-panel print:p-0 print:shadow-none print:border-none">
            {#if detailLoading}
              <p>Loading printable design detail...</p>
            {:else if detailError}
              <p class="text-red-600">{detailError}</p>
            {:else if !detailItem}
              <p>No design found for id {printDesignId}.</p>
            {:else}
              <div class="space-y-3">
                <h2 class="text-xl font-semibold">{detailItem.filename}</h2>
                {#if detailItem.image_data_url}
                  <img src={detailItem.image_data_url} alt={detailItem.filename} class="w-full max-h-[32rem] object-contain border rounded p-2" />
                {/if}
                <div class="grid sm:grid-cols-2 gap-2 text-sm">
                  <div><strong>File:</strong> {detailItem.filepath || "Unknown"}</div>
                  <div><strong>Designer:</strong> {detailItem.designer || "Unknown"}</div>
                  <div><strong>Source:</strong> {detailItem.source || "Unknown"}</div>
                  <div><strong>Hoop:</strong> {detailItem.hoop || "Unknown"}</div>
                  <div><strong>Dimensions:</strong> {detailItem.width_mm ?? "?"} x {detailItem.height_mm ?? "?"} mm</div>
                  <div><strong>Stitches:</strong> {detailItem.stitch_count ?? "?"}</div>
                  <div><strong>Colours:</strong> {detailItem.color_count ?? "?"}</div>
                  <div><strong>Colour changes:</strong> {detailItem.color_change_count ?? "?"}</div>
                  <div><strong>Added:</strong> {detailItem.date_added || "Unknown"}</div>
                </div>
                {#if detailItem.rating}
                  <div><strong>Rating:</strong> <span class="text-yellow-500">{ratingToStars(detailItem.rating)}</span></div>
                {/if}
                {#if detailItem.is_stitched}
                  <div><strong>Stitched:</strong> Yes</div>
                {/if}
                {#if detailItem.notes}
                  <div>
                    <p class="font-semibold">Notes</p>
                    <p class="text-sm text-gray-700 whitespace-pre-wrap">{detailItem.notes}</p>
                  </div>
                {/if}
                {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                  <div>
                    <p class="font-semibold">Tags</p>
                    <p class="text-sm text-gray-700">{detailItem.tags.map((tag) => tag.description).join(", ")}</p>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      {:else if currentUiKind === "import"}
        <section class="import-page space-y-4">
          <h1 class="ui-page-title import-title">Bulk Import</h1>
          {#if importRouteStep === 1}
            <p class="ui-help-note import-step1-intro">
              Select one or more folders containing embroidery files. Sub-folders are scanned automatically.
              Selected files are <strong>copied into the catalogue</strong>, and each source folder name is preserved inside managed storage.
              <a href="#/help#importing" class="ui-app-link ml-1">Import help</a>
            </p>
          {/if}

          {#if importRouteStep === 1}
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
              <label for="import-root-path" class="ui-field-label text-sm">
                <span class="block font-medium mb-1">Source Folder(s) *</span>
              </label>
              <div class="space-y-2.5">
                <div class="folder-row import-folder-row flex items-center" data-index="0">
                  <input
                    id="import-root-path"
                    class="ui-text-input ui-control-text-inset import-folder-input flex-1 font-mono"
                    bind:value={importRootPath}
                    placeholder="Enter path to your embroidery designs folder…"
                    disabled={importLoading || importActionLoading || importBrowseLoading}
                    aria-label="Source folder path 1"
                  />
                  <button
                    type="button"
                    class="ui-action-button"
                    onclick={() => browseImportRootPath(-1)}
                    disabled={importLoading || importActionLoading || importBrowseLoading}
                  >
                    {importBrowseLoading ? "Browsing…" : "Browse…"}
                  </button>
                  <button
                    type="button"
                    class="ui-action-button"
                    onclick={() => clearImportRootPaths()}
                    disabled={importLoading || importActionLoading || importBrowseLoading || (!String(importRootPath || "").trim() && importRootPaths.length === 0)}
                    title="Remove this folder"
                  >
                    Remove
                  </button>
                </div>

                {#each importRootPaths as rootPath, rowIndex}
                  <div class="folder-row import-folder-row flex items-center" data-index={rowIndex + 1}>
                    <input
                      type="text"
                      class="ui-text-input ui-control-text-inset import-folder-input flex-1 font-mono"
                      value={rootPath}
                      readonly
                      aria-label={`Source folder path ${rowIndex + 2}`}
                    />
                    <button
                      type="button"
                      class="ui-action-button"
                      onclick={() => browseImportRootPath(rowIndex)}
                      disabled={importLoading || importActionLoading || importBrowseLoading}
                    >
                      Browse…
                    </button>
                    <button
                      type="button"
                      class="ui-action-button"
                      onclick={() => removeImportRootPath(rootPath)}
                      disabled={importLoading || importActionLoading || importBrowseLoading}
                      title="Remove this folder"
                    >
                      Remove
                    </button>
                  </div>
                {/each}
              </div>

              <div class="import-step1-add-folder-shell">
                <button
                  type="button"
                  class="menu-button-primary ui-action-button ui-action-button-primary import-add-folder-link"
                  onclick={addCurrentImportRootPath}
                  disabled={importLoading || importActionLoading || importBrowseLoading || !String(importRootPath || "").trim()}
                >
                  Add another folder
                </button>
              </div>

            </div>

            <div class="ui-action-button-group import-step1-primary-actions">
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
              <div class="grid sm:grid-cols-3 gap-3 text-sm">
                <div class="ui-section-shell import-metric-card">Source: {importPreviewSource}</div>
                <div class="ui-section-shell import-metric-card">Discovered: {importPreview.discovered_count ?? 0}</div>
                <div class="ui-section-shell import-metric-card">Folders: {importPreview.folder_count ?? 0}</div>
              </div>
            {/if}
            </form>
          </div>
          {/if}

          {#if importRouteStep === 2}
            {#if importPreview}
            <div class="ui-section-shell import-panel space-y-4">
              <div class="space-y-1">
                <p class="ui-field-label import-field-label">Review scanned files</p>
                <p class="ui-help-note">
                  {importStep2FolderGroups.length || importPreview.folder_count || 0} folder(s) scanned - {Array.isArray(importPreview.scanned_files) ? importPreview.scanned_files.length : 0} file(s) found.
                  Selected files will be <strong>copied into the catalogue</strong>.
                  <a href="#/help#importing" class="ui-app-link ml-1">Import help</a>
                </p>
              </div>

              <div class="ui-section-shell p-3 space-y-3 import-step2-global-shell">
                <p class="ui-field-label import-field-label">Apply to all folders (optional override)</p>
                <div class="grid grid-cols-2 gap-2 text-sm import-step2-global-grid">
                  <label class="ui-field-label text-sm">
                    <span class="block font-medium mb-1">Designer</span>
                    <select class="ui-select-input ui-control-text-inset" bind:value={importGlobalDesignerId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                      <option value="">Keep inferred (per folder)</option>
                      {#each importDesigners as designer}
                        <option value={String(designer.id)}>{designer.name}</option>
                      {/each}
                    </select>
                  </label>
                  <label class="ui-field-label text-sm">
                    <span class="block font-medium mb-1">Source</span>
                    <select class="ui-select-input ui-control-text-inset" bind:value={importGlobalSourceId} disabled={importReferenceLoading || importLoading || importActionLoading}>
                      <option value="">Keep inferred (per folder)</option>
                      {#each importSources as source}
                        <option value={String(source.id)}>{source.name}</option>
                      {/each}
                    </select>
                  </label>
                </div>
              </div>

              <div class="space-y-2 import-step2-actions-shell">
                <div class="ui-action-button-group import-step1-primary-actions import-step2-primary-actions import-step2-inline-actions">
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
                    class={`import-step2-link-button import-step2-link-button-first ${importStep2CanSelectAll ? "import-step2-link-button-active" : "import-step2-link-button-inactive"}`}
                    onclick={selectAllImportFiles}
                    disabled={importLoading || importActionLoading || !importStep2CanSelectAll}
                  >
                    Select all
                  </button>
                  <button
                    type="button"
                    class={`import-step2-link-button ${importStep2CanDeselectAll ? "import-step2-link-button-active" : "import-step2-link-button-inactive"}`}
                    onclick={clearImportFileSelection}
                    disabled={importLoading || importActionLoading || !importStep2CanDeselectAll}
                  >
                    Deselect all
                  </button>
                </div>
              </div>

              {#if importStep2FolderGroups.length > 0}
                <div class="space-y-3">
                  {#each importStep2FolderGroups as folder}
                    <div class="ui-section-shell overflow-hidden import-step2-folder-shell">
                      <div class="bg-gray-50 border-b px-4 py-3 flex flex-wrap items-center gap-3 import-step2-folder-header">
                        <div class="flex-1 min-w-0">
                          <code class="text-xs text-black font-bold import-step2-folder-label">{folder.folderLabel}</code>
                          <span class="mx-2 text-xs text-gray-400" aria-hidden="true">-</span>
                          <code class="text-xs text-gray-500 break-all">{folder.folderPath}</code>
                        </div>
                      </div>

                      <div class="px-4 py-3 border-b bg-gray-50/50 import-step2-folder-overrides">
                        <div class="grid grid-cols-2 gap-2 text-sm">
                          <label class="ui-field-label text-sm">
                            <span class="block font-medium mb-1">Designer for this folder</span>
                            <select
                              class="ui-select-input ui-control-text-inset"
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
                          <label class="ui-field-label text-sm">
                            <span class="block font-medium mb-1">Source for this folder</span>
                            <select
                              class="ui-select-input ui-control-text-inset"
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

                      <div class="import-step2-file-list-shell">
                        <div class="import-step2-file-columns">
                          {#each folder.files as file}
                            <label class="import-step2-file-item">
                              <input
                                type="checkbox"
                                class="ui-checkbox"
                                checked={file.isSelected}
                                onchange={(event) => toggleImportFile(file.fullPath, event.currentTarget.checked)}
                                disabled={importLoading || importActionLoading}
                              />
                              <span class="ui-field-label text-sm import-step2-filename" title={file.fullPath}>{file.filename}</span>
                            </label>
                          {/each}
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              {:else}
                <p class="text-sm text-gray-600">No supported files discovered in this preview.</p>
              {/if}
            </div>
            {:else}
            <div class="ui-section-shell import-panel space-y-2">
              <p class="ui-help-note">Step 2 needs a completed preview first.</p>
              <div>
                <button type="button" class="menu-button-secondary ui-action-button" onclick={() => navigateTo("#/import/step1")}>Back to Step 1</button>
              </div>
            </div>
            {/if}
          {/if}

          {#if importRouteStep === 3}
            {#if importPrecheck}
            <div class="ui-section-shell import-panel space-y-4">
              <p class="ui-field-label import-field-label">Before You Import</p>

              {#if settingsHasGoogleApiKey}
                <div class="ui-section-shell border-amber-300 bg-amber-50 text-amber-950 space-y-2">
                  <p class="font-medium text-amber-900">Google AI tagging is enabled for this installation</p>
                  <p class="ui-help-note text-amber-900">
                    Depending on your saved settings, Tier 2 and/or Tier 3 may run during this import. Gemini usage may incur cost. Free-tier limits are approximately
                    <strong>15 requests per minute</strong> and <strong>1,500 requests per day</strong>.
                    A historical estimate from February 2026 found that Tier 3 on 4,000 images cost about <strong>$0.33 on the paid tier</strong>; actual pricing may have changed -
                    check <a href="https://ai.google.dev/pricing" target="_blank" rel="noopener" class="underline">ai.google.dev/pricing</a>.
                  </p>
                  <p class="text-xs text-amber-900">
                    Tier 2 auto: <strong>{settingsAiTier2Auto ? "on" : "off"}</strong>
                    · Tier 3 auto: <strong>{settingsAiTier3Auto ? "on" : "off"}</strong>
                    · AI batch limit: <strong>{settingsAiBatchSize || "10"} designs</strong>
                    · DB commit batch: <strong>{settingsImportCommitBatchSize || "10"} designs</strong>
                    · <a href="#/admin/settings" class="underline">Change in Settings</a>
                  </p>
                </div>
              {:else}
                <div class="ui-section-shell border-blue-300 bg-blue-50 text-blue-950 space-y-2">
                  <p class="font-medium text-blue-900">Google AI tagging is not configured</p>
                  <p class="ui-help-note text-blue-900">
                    No Google API key is currently saved, so this import will use <strong>Tier 1 keyword tagging only</strong> and no Gemini calls will be made.
                    If you want AI-assisted tagging, add an API key in Settings and enable the tiers you want.
                  </p>
                  <p class="text-xs text-blue-900">
                    <a href="#/admin/settings" class="underline">Admin Settings</a>
                    · <a href={settingsHelpUrl} class="underline">AI Tagging Guide</a>
                  </p>
                </div>
              {/if}

              <div class="ui-section-shell space-y-2">
                <p class="font-medium text-gray-900">Image Preview Preference</p>
                <p class="ui-help-note">
                  Choose how preview images are generated for this import. 2D is faster (flat render), 3D is slower but shows stitch simulation.
                  Your saved setting is shown below; you can override it for this session.
                </p>
                <div class="flex flex-wrap items-center gap-4 text-sm">
                  <label class="inline-flex items-center gap-2">
                    <input type="radio" class="ui-radio" name="import-step3-image-preference" value="2d" bind:group={importStep3ImagePreference} disabled={importActionLoading || !importContextToken} />
                    <span class="font-medium">2D - Fast flat preview</span>
                  </label>
                  <label class="inline-flex items-center gap-2">
                    <input type="radio" class="ui-radio" name="import-step3-image-preference" value="3d" bind:group={importStep3ImagePreference} disabled={importActionLoading || !importContextToken} />
                    <span class="font-medium">3D - Detailed stitch simulation</span>
                  </label>
                  <span class="text-xs text-gray-500">(Saved setting: {settingsImagePreference === "3d" ? "3D" : "2D"})</span>
                </div>
              </div>

              {#if importPrecheck.is_first_import}
                <div class="ui-section-shell import-folder-card space-y-2 border-amber-300 bg-amber-50 text-amber-950">
                  <p class="font-medium text-amber-900">Before your first import, please check your hoops</p>
                  <p class="ui-help-note text-amber-900">
                    A starter set of tags is already included with the catalogue. Hoops are not, because they depend on your machine and the frames you actually own.
                  </p>
                  <p class="ui-help-note text-amber-900">
                    If you set up your hoops now, the import process can auto-assign a hoop where the design size is known. You can also review tags, sources, and designers before importing.
                  </p>
                  {#if importPrecheck.needs_hoop_setup}
                    <p class="ui-help-note font-medium text-amber-900">No hoops are defined yet for this catalogue.</p>
                  {/if}
                </div>

                <p class="ui-help-note">
                  Review your hoops first, or skip them for now and the app will ask if you are really really sure before importing.
                </p>
              {:else}
                <p class="ui-help-note">
                  Consider reviewing your hoops, tags, sources, or designers before importing. Hoops usually only need special attention on the first import.
                </p>
              {/if}

              <div class="ui-action-button-group">
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
                    {importPrecheck.is_first_import ? "No, import now anyway" : "No, import now"}
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
                <div class="ui-section-shell import-folder-card space-y-2">
                  <p class="ui-help-note text-amber-800">
                    Hoops are not configured for a first import. Confirm to continue anyway.
                  </p>
                  <button class="menu-button-primary ui-action-button ui-action-button-primary" onclick={() => executeImportPrecheckAction("import_now", true)} disabled={importActionLoading || !importContextToken}>
                    Confirm import without hoop setup
                  </button>
                </div>
              {/if}

              {#if importActionMessage}
                <p class="ui-help-note">{importActionMessage}</p>
              {/if}
            </div>
            {:else}
            <div class="ui-section-shell import-panel space-y-2">
              <p class="ui-help-note">Step 3 needs precheck to be completed first.</p>
              <div>
                <button type="button" class="menu-button-secondary ui-action-button" onclick={() => navigateTo(importPreview ? "#/import/step2" : "#/import/step1")}>Go to previous step</button>
              </div>
            </div>
            {/if}
          {/if}
        </section>
      {:else if currentUiKind === "projects-list"}
        <section class="projects-page space-y-4">
          <div class="projects-header flex items-center justify-between gap-3">
            <h1 class="ui-page-title projects-title">Projects</h1>
            <button class="menu-button-primary" onclick={() => navigateTo("#/projects/new")}>+ New Project</button>
          </div>

          <p class="projects-intro text-sm text-gray-500">
            Group designs for a planned embroidery task - for example a seasonal set or a quilt block series.
            <a href="#importing" class="text-indigo-600 hover:underline">Learn more</a>
          </p>

          {#if projectsActionMessage}
            <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
              {projectsActionMessage}
            </div>
          {/if}

          <div class="projects-shell">
            {#if projectsLoading}
              <p>Loading projects...</p>
            {:else if projectsError}
              <p class="text-red-600">{projectsError}</p>
            {:else if !projectsItems || projectsItems.length === 0}
              <p class="projects-empty text-gray-500">
                No projects yet.
                <button class="text-indigo-600 hover:underline" onclick={() => navigateTo("#/projects/new")}>Create one</button>.
              </p>
            {:else}
              <div class="projects-grid">
                {#each projectsItems as project}
                  <a
                    class="projects-tile text-left block"
                    href={`#/projects/${project.id}`}
                    aria-label={`Open project ${project.name}`}
                  >
                    <div class="projects-tile-top flex items-start justify-between gap-3">
                      <h2 class="projects-tile-title font-semibold text-gray-800">{project.name}</h2>
                      <span class="projects-count-badge">{Number(project.design_count || 0)} design{Number(project.design_count || 0) === 1 ? "" : "s"}</span>
                    </div>
                    {#if project.description}
                      <p class="projects-tile-description text-sm text-gray-500 mt-1">{project.description}</p>
                    {/if}
                    {#if project.date_created}
                      <p class="projects-tile-meta text-xs text-gray-400 mt-2">Created {project.date_created}</p>
                    {/if}
                  </a>
                {/each}
              </div>
            {/if}
          </div>
        </section>
      {:else if currentUiKind === "project-new"}
        <section class="projects-page space-y-4">
          <div>
            <button class="projects-back-link text-indigo-600 text-sm hover:underline" onclick={() => navigateTo("#/projects")}>← Projects</button>
          </div>

          <div class="projects-form-card space-y-3">
            <h2 class="projects-subtitle text-2xl font-bold text-gray-800">New Project</h2>
            <p class="projects-intro text-sm text-gray-500">
              Projects let you group designs for a planned embroidery task.
              <a href="#importing" class="text-indigo-600 hover:underline">Help</a>
            </p>

            {#if projectsActionMessage}
              <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
                {projectsActionMessage}
              </div>
            {/if}

            <form
              class="space-y-3"
              onsubmit={(event) => {
                event.preventDefault();
                submitNewProject();
              }}
            >
              <label class="projects-label block text-sm text-gray-700">
                <span class="block font-medium mb-1">Name *</span>
                <input
                  type="text"
                  class="projects-input w-full border rounded px-3 py-1.5 text-sm"
                  bind:value={projectNewName}
                  required
                  placeholder="e.g. Christmas Stockings 2024"
                />
              </label>
              <label class="projects-label block text-sm text-gray-700">
                <span class="block font-medium mb-1">Description</span>
                <textarea
                  rows="3"
                  class="projects-input projects-textarea w-full border rounded px-3 py-1.5 text-sm"
                  bind:value={projectNewDescription}
                  placeholder="Optional notes, goals, or deadline"
                ></textarea>
              </label>
              <button type="submit" class="menu-button-primary" disabled={projectNewSaving}>
                {projectNewSaving ? "Creating..." : "Create Project"}
              </button>
            </form>
          </div>
        </section>
      {:else if currentUiKind === "project-detail"}
        <section class="projects-page space-y-4">
          <div class="projects-detail-top flex items-center justify-between gap-3 no-print">
            <button class="projects-back-link text-indigo-600 text-sm hover:underline" onclick={() => navigateTo("#/projects")}>← Projects</button>
            <div class="flex flex-wrap gap-3">
              {#if projectDetail?.project?.id}
                <button class="projects-action-link text-sm text-gray-600 hover:underline" onclick={() => navigateTo(`#/projects/${projectDetail.project.id}/print`)}>Print Sheet</button>
              {/if}
              <button class="projects-danger-link text-sm text-red-500 hover:underline" onclick={confirmDeleteProject} disabled={projectDetailSaving || !projectDetail?.project?.id}>Delete Project</button>
            </div>
          </div>

          {#if projectsActionMessage}
            <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
              {projectsActionMessage}
            </div>
          {/if}

          <div class="projects-form-card">
            {#if projectDetailLoading}
              <p>Loading project...</p>
            {:else if projectDetailError}
              <p class="text-red-600">{projectDetailError}</p>
            {:else if !projectDetail?.project}
              <p>Project not found.</p>
            {:else}
              <form
                class="space-y-3"
                onsubmit={(event) => {
                  event.preventDefault();
                  saveProjectDetail();
                }}
              >
                <input
                  type="text"
                  class="projects-title-input text-2xl font-bold border-b w-full focus:outline-none py-1"
                  bind:value={projectDetailName}
                  required
                />
                <textarea
                  rows="2"
                  class="projects-input projects-textarea w-full border rounded px-2 py-1 text-sm focus:outline-none"
                  bind:value={projectDetailDescription}
                  placeholder="Description..."
                ></textarea>
                <button type="submit" class="menu-button-primary" disabled={projectDetailSaving}>
                  {projectDetailSaving ? "Saving..." : "Save"}
                </button>
              </form>
            {/if}
          </div>

          {#if projectDetail?.project}
            <div class="space-y-3">
              <h2 class="text-lg font-semibold">Designs ({Array.isArray(projectDetail?.designs) ? projectDetail.designs.length : 0})</h2>
              {#if Array.isArray(projectDetail.designs) && projectDetail.designs.length > 0}
                <div class="projects-design-grid grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mb-6">
                  {#each projectDetail.designs as design}
                    <div class="projects-design-card bg-white rounded shadow overflow-hidden flex flex-col">
                      <a class="projects-design-link text-left block" href={`#/designs/${design.id}`} aria-label={`Open design ${design.filename}`}>
                        {#if design.image_data_url}
                          <img src={design.image_data_url} alt={design.filename} class="projects-design-image" loading="lazy" />
                        {:else if design.has_image}
                          <div class="projects-design-preview w-full h-32 bg-gray-100 flex items-center justify-center text-gray-700 text-xs">Image unavailable</div>
                        {:else}
                          <div class="projects-design-preview-empty w-full h-32 bg-gray-200 flex items-center justify-center text-gray-400 text-xs">No image</div>
                        {/if}
                      </a>
                      <div class="projects-design-meta p-2 flex-1 flex flex-col">
                        <a class="projects-design-title-link text-xs font-medium text-gray-800 truncate" href={`#/designs/${design.id}`}>
                          {design.filename}
                        </a>
                        {#if design.designer_name}
                          <p class="text-xs text-gray-500">{design.designer_name}</p>
                        {/if}
                        <button class="text-xs text-red-400 hover:underline mt-auto pt-1 text-left" onclick={() => removeDesignFromProjectMembership(design.id)} disabled={projectDetailSaving}>Remove</button>
                      </div>
                    </div>
                  {/each}
                </div>
              {:else}
                <p class="text-gray-500 mb-6">No designs in this project yet.</p>
              {/if}
            </div>
          {/if}
        </section>
      {:else if currentUiKind === "project-print"}
        <section class="projects-page space-y-3 projects-print-page">
          <div class="flex flex-wrap gap-2 no-print">
            <button class="menu-button-secondary" onclick={() => navigateTo(`#/projects/${projectPrintId}`)}>Back to Project</button>
            <button class="menu-button-primary" onclick={printCurrentView}>Print</button>
          </div>

          <div class="projects-print-shell print:p-0 print:shadow-none print:border-none">
            {#if projectPrintLoading}
              <p>Loading printable project sheet...</p>
            {:else if projectPrintError}
              <p class="text-red-600">{projectPrintError}</p>
            {:else if !projectPrint?.project}
              <p>Project not found.</p>
            {:else}
              <div class="space-y-4">
                <h2 class="text-2xl font-semibold">{projectPrint.project.name}</h2>
                {#if projectPrint.project.description}
                  <p class="text-sm text-gray-600">{projectPrint.project.description}</p>
                {/if}

                <div class="space-y-3">
                  {#if Array.isArray(projectPrint.designs) && projectPrint.designs.length > 0}
                    {#each projectPrint.designs as design}
                      <div class="projects-print-card border border-gray-200 rounded p-3 flex gap-4 print:break-inside-avoid">
                        {#if design.image_data_url}
                          <img src={design.image_data_url} alt={design.filename} class="projects-print-image w-40 h-40 object-contain bg-gray-100" />
                        {:else}
                          <div class="projects-print-image projects-design-preview-empty w-40 h-40 bg-gray-200 flex items-center justify-center text-gray-400 text-xs">No image</div>
                        {/if}
                        <div class="text-sm space-y-1 flex-1">
                          <h3 class="font-semibold text-base">{design.filename}</h3>
                          {#if design.width_mm != null && design.height_mm != null}
                            <p><strong>Size:</strong> {design.width_mm} x {design.height_mm} mm</p>
                          {/if}
                          {#if design.hoop}
                            <p><strong>Hoop:</strong> {design.hoop}</p>
                          {/if}
                          {#if design.stitch_count != null}
                            <p><strong>Stitches:</strong> {design.stitch_count}</p>
                          {/if}
                          {#if design.color_count != null}
                            <p><strong>Colours:</strong> {design.color_count}</p>
                          {/if}
                          {#if design.color_change_count != null}
                            <p><strong>Colour changes:</strong> {design.color_change_count}</p>
                          {/if}
                          {#if design.designer_name}
                            <p><strong>Designer:</strong> {design.designer_name}</p>
                          {/if}
                          {#if design.rating}
                            <p><strong>Rating:</strong> <span class="text-yellow-600">{ratingToStars(design.rating)}</span></p>
                          {/if}
                          {#if design.is_stitched}
                            <p><strong>Stitched:</strong> Yes</p>
                          {/if}
                          {#if design.notes}
                            <p class="italic text-gray-700">{design.notes}</p>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  {:else}
                    <p class="text-gray-500">No designs in this project yet.</p>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        </section>
      {:else if currentUiKind === "help"}
        <HelpView />
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
            <h1 class="ui-page-title admin-title">Manage Tags</h1>
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

            <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl" open={adminImageTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("image", event)}>
              <summary class="bg-green-50 border-b border-green-200 px-4 py-2 flex items-center gap-2 cursor-pointer">
                <svg class={`h-4 w-4 text-green-700 transition-transform duration-200 ${adminImageTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                  <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
                </svg>
                <h2 class="text-sm font-semibold text-green-800 uppercase tracking-wide">Image Tags</h2>
              </summary>
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
            </details>

            <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl" open={adminStitchingTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("stitching", event)}>
              <summary class="bg-blue-50 border-b border-blue-200 px-4 py-2 flex items-center gap-2 cursor-pointer">
                <svg class={`h-4 w-4 text-blue-700 transition-transform duration-200 ${adminStitchingTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                  <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
                </svg>
                <h2 class="text-sm font-semibold text-blue-800 uppercase tracking-wide">Stitching Tags</h2>
              </summary>
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
            </details>

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
            <h1 class="ui-page-title admin-title">Manage Sources</h1>
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
            <h1 class="ui-page-title admin-title">Manage Hoops</h1>
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
                    min="1"
                    step="1"
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
                    min="1"
                    step="1"
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
                              min="1"
                              step="1"
                              bind:value={editingHoopWidth}
                              class="admin-input border rounded px-2 py-1 text-sm w-28 text-right"
                            />
                          {:else}
                            {hoop.maxWidthMm.toFixed(0)}
                          {/if}
                        </td>
                        <td class="px-4 py-2 text-right">
                          {#if editingHoopId === hoop.id}
                            <input
                              type="number"
                              min="1"
                              step="1"
                              bind:value={editingHoopHeight}
                              class="admin-input border rounded px-2 py-1 text-sm w-28 text-right"
                            />
                          {:else}
                            {hoop.maxHeightMm.toFixed(0)}
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

        </section>
      {:else if currentUiKind === "tagging-actions"}
        <div class="space-y-4">
          <form class="settings-card settings-form bg-white rounded shadow p-6 space-y-5" onsubmit={runTaggingActions}>
            <div class="grid md:grid-cols-2 gap-4">
              <label class="settings-input">
                <span>Tagging mode</span>
                <select bind:value={taggingActionMode} class="border rounded px-2 py-1 text-sm">
                  <option value="tag_untagged">Tag only untagged</option>
                  <option value="retag_all_unverified">Tag untagged and unverified</option>
                  <option value="retag_all">Re-tag ALL (including verified)</option>
                </select>
              </label>

              <div class="settings-input">
                <span>Tiers</span>
                <div class="flex flex-wrap gap-3 mt-1 text-sm">
                  <label class="inline-flex items-center gap-2">
                    <input type="checkbox" checked disabled />
                    <span>Tier 1</span>
                  </label>
                  <label class="inline-flex items-center gap-2">
                    <input type="checkbox" bind:checked={taggingRunTier2} disabled={!taggingHasGoogleApiKey} />
                    <span>Tier 2</span>
                  </label>
                  <label class="inline-flex items-center gap-2">
                    <input type="checkbox" bind:checked={taggingRunTier3} disabled={!taggingHasGoogleApiKey} />
                    <span>Tier 3</span>
                  </label>
                </div>
              </div>
            </div>

            <div class="grid md:grid-cols-3 gap-4">
              <label class="settings-input">
                <span>Batch size</span>
                <input type="number" min="1" step="1" bind:value={taggingBatchSize} class="border rounded px-2 py-1 text-sm" />
              </label>
              <label class="settings-input">
                <span>Commit every</span>
                <input type="number" min="1" step="1" bind:value={taggingCommitEvery} class="border rounded px-2 py-1 text-sm" />
              </label>
              <label class="settings-input">
                <span>Workers</span>
                <input type="number" min="1" step="1" bind:value={taggingWorkers} class="border rounded px-2 py-1 text-sm" />
              </label>
            </div>

            <div class="grid md:grid-cols-3 gap-4 border border-gray-200 rounded p-3 bg-gray-50">
              <label class="inline-flex items-center gap-2 text-sm">
                <input type="checkbox" bind:checked={taggingRunStitching} />
                <span>Include stitching action</span>
              </label>
              <label class="inline-flex items-center gap-2 text-sm">
                <input type="checkbox" bind:checked={taggingRunImages} />
                <span>Include images action</span>
              </label>
              <label class="inline-flex items-center gap-2 text-sm">
                <input type="checkbox" bind:checked={taggingRunColorCounts} />
                <span>Include threads and colours action</span>
              </label>

              <label class="inline-flex items-center gap-2 text-sm">
                <input type="checkbox" bind:checked={taggingClearExistingStitching} disabled={!taggingRunStitching} />
                <span>Clear existing stitching tags first</span>
              </label>
              <label class="inline-flex items-center gap-2 text-sm">
                <input type="checkbox" bind:checked={taggingImageRedo} disabled={!taggingRunImages} />
                <span>Re-process all images</span>
              </label>
              <label class="inline-flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  bind:checked={taggingUpgrade2dTo3d}
                  disabled={!taggingRunImages || taggingImageRedo}
                />
                <span>Upgrade 2D images to 3D</span>
              </label>
              <label class="inline-flex items-center gap-2 text-sm md:col-span-3">
                <input type="checkbox" bind:checked={taggingUsePreview3d} disabled={!taggingRunImages} />
                <span>Use 3D preview generation for image action</span>
              </label>
            </div>

            <div class="flex flex-wrap gap-2">
              <button type="submit" class="menu-button-primary" disabled={taggingRunInFlight}>Run selected actions</button>
              <button type="button" class="menu-button-secondary" onclick={stopTaggingActionsRun} disabled={!taggingRunInFlight}>
                Stop running
              </button>
              <button type="button" class="menu-button-secondary" onclick={runStitchingOnlyAction} disabled={taggingRunInFlight}>
                Run stitching-only
              </button>
              <button type="button" class="menu-button-secondary" onclick={refreshBackfillLogEntries} disabled={taggingRunInFlight}>
                Refresh log
              </button>
            </div>
          </form>

          <div
            class="border rounded px-3 py-2 text-sm"
            class:bg-green-50={taggingStatusType === "success"}
            class:border-green-300={taggingStatusType === "success"}
            class:text-green-800={taggingStatusType === "success"}
            class:bg-red-50={taggingStatusType === "error"}
            class:border-red-300={taggingStatusType === "error"}
            class:text-red-800={taggingStatusType === "error"}
            class:bg-blue-50={taggingStatusType !== "success" && taggingStatusType !== "error"}
            class:border-blue-200={taggingStatusType !== "success" && taggingStatusType !== "error"}
            class:text-blue-800={taggingStatusType !== "success" && taggingStatusType !== "error"}
          >
            {taggingStatusMessage}
          </div>

          {#if taggingLastSummary}
            <div class="route-panel text-sm">
              <p><strong>Last run:</strong> processed {taggingLastSummary.processed}, errors {taggingLastSummary.errors}, stopped {taggingLastSummary.stopped ? "yes" : "no"}.</p>
              <p><strong>Actions:</strong> {(taggingLastSummary.actions || []).join(", ") || "none"}</p>
            </div>
          {/if}

          <div class="route-panel text-sm">
            <p class="font-semibold mb-2">Backfill log (latest)</p>
            {#if taggingLogEntries.length === 0}
              <p class="text-gray-600">No log entries available.</p>
            {:else}
              <ul class="space-y-1">
                {#each taggingLogEntries as entry}
                  <li><strong>{entry.level}:</strong> {entry.message}</li>
                {/each}
              </ul>
            {/if}
          </div>

          {#if !taggingHasGoogleApiKey}
            <div class="route-panel text-sm text-blue-800 bg-blue-50 border border-blue-200">
              No API key configured. Tier 2 and Tier 3 are disabled; Tier 1 keyword tagging still works.
            </div>
          {/if}
        </div>
      {:else if currentUiKind === "orphans"}
        <div class="space-y-4">
          <p class="text-sm text-gray-500 max-w-3xl">
            These designs exist in the database but their files were not found on disk.
            Deleting a record removes it from the database only.
          </p>

          {#if orphanActionMessage}
            <div
              class="border rounded px-3 py-2 text-sm"
              class:bg-green-50={orphanActionType === "success"}
              class:border-green-300={orphanActionType === "success"}
              class:text-green-800={orphanActionType === "success"}
              class:bg-red-50={orphanActionType === "error"}
              class:border-red-300={orphanActionType === "error"}
              class:text-red-800={orphanActionType === "error"}
              class:bg-blue-50={orphanActionType === "info"}
              class:border-blue-200={orphanActionType === "info"}
              class:text-blue-800={orphanActionType === "info"}
            >
              {orphanActionMessage}
            </div>
          {/if}

          {#if orphansError}
            <div class="border border-red-300 bg-red-50 text-red-700 rounded px-3 py-2 text-sm">{orphansError}</div>
          {/if}

          <div class="flex flex-wrap items-center gap-3 text-sm">
            <span class="text-gray-700 font-medium">
              {orphanTotal} orphaned record(s) total, page {orphanPage} of {orphanTotalPages}, showing {orphanItems.length}
            </span>
            <div class="ml-auto flex gap-2">
              <button type="button" class="menu-button-secondary" onclick={selectAllOrphansOnPage} disabled={orphansLoading || orphanItems.length === 0}>
                Select all on page
              </button>
              <button type="button" class="menu-button-secondary" onclick={deselectAllOrphansOnPage} disabled={orphansLoading || orphanItems.length === 0}>
                Deselect all
              </button>
              <button type="button" class="menu-button-secondary" onclick={() => loadOrphansPage(orphanPage, true)} disabled={orphansLoading}>
                Refresh
              </button>
              <button type="button" class="menu-button-primary" onclick={deleteSelectedOrphans} disabled={orphansLoading || orphanSelectedIds.length === 0}>
                Delete selected
              </button>
              <button type="button" class="menu-button-primary" onclick={deleteEveryOrphan} disabled={orphansLoading || orphanTotal === 0}>
                Delete all
              </button>
            </div>
          </div>

          <div class="border border-gray-200 rounded p-3 bg-gray-50 space-y-2">
            <p class="text-xs font-semibold text-gray-700">Orphan debug scan</p>
            <div class="flex flex-wrap items-end gap-2 text-sm">
              <label class="flex flex-col gap-1 min-w-[20rem]">
                <span class="text-xs text-gray-600">Filter filepath contains</span>
                <input
                  type="text"
                  class="border rounded px-2 py-1 text-sm"
                  bind:value={orphanDebugFilter}
                  placeholder="Amazing Designs - 1033 Crestswer"
                />
              </label>
              <button type="button" class="menu-button-secondary" onclick={runOrphanDebugScan} disabled={orphanDebugLoading}>
                {orphanDebugLoading ? "Running..." : "Run debug scan"}
              </button>
            </div>

            {#if orphanDebugError}
              <p class="text-xs text-red-700">{orphanDebugError}</p>
            {/if}

            {#if orphanDebugResult}
              <p class="text-xs text-gray-700">
                Base: {orphanDebugResult.base_path} | Checked: {orphanDebugResult.checked} | Found: {orphanDebugResult.found} | Samples: {orphanDebugResult.samples.length}
              </p>
              {#if orphanDebugResult.checked === 0 && orphanDebugFilter.trim().length > 0}
                <p class="text-xs text-amber-700">
                  No rows matched this filter. Try a broader substring such as just the numeric set id (for example: 1033).
                </p>
              {/if}
              <div class="bg-white rounded border border-gray-200 overflow-x-auto max-h-56">
                <table class="w-full text-xs">
                  <thead class="bg-gray-100 text-gray-600 uppercase">
                    <tr>
                      <th class="px-2 py-1 text-left">Id</th>
                      <th class="px-2 py-1 text-left">Exists</th>
                      <th class="px-2 py-1 text-left">Filename</th>
                      <th class="px-2 py-1 text-left">Stored filepath</th>
                      <th class="px-2 py-1 text-left">Resolved path</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-100">
                    {#each orphanDebugResult.samples as sample}
                      <tr>
                        <td class="px-2 py-1">{sample.id}</td>
                        <td class="px-2 py-1 {sample.exists ? 'text-green-700' : 'text-red-700'}">{sample.exists ? "yes" : "no"}</td>
                        <td class="px-2 py-1">{sample.filename}</td>
                        <td class="px-2 py-1 font-mono">{sample.filepath}</td>
                        <td class="px-2 py-1 font-mono">{sample.resolved_path}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>

          <div class="bg-white rounded shadow overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-gray-50 text-gray-600 uppercase text-xs">
                <tr>
                  <th class="px-3 py-2 w-8"></th>
                  <th class="px-4 py-2 text-left">Filename</th>
                  <th class="px-4 py-2 text-left">Path (relative) - open nearest existing folder</th>
                  <th class="px-4 py-2 text-left">Designer</th>
                  <th class="px-4 py-2 text-left">Date Added</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-100">
                {#if orphansLoading}
                  <tr>
                    <td colspan="5" class="px-4 py-4 text-gray-500">Loading orphan records...</td>
                  </tr>
                {:else if orphanItems.length === 0}
                  <tr>
                    <td colspan="5" class="px-4 py-4 text-gray-500">No orphaned records found.</td>
                  </tr>
                {:else}
                  {#each orphanItems as item}
                    <tr class="hover:bg-gray-50">
                      <td class="px-3 py-2 text-center">
                        <input
                          type="checkbox"
                          checked={orphanIsSelected(item.id)}
                          onchange={(event) => toggleOrphanSelection(item.id, event.currentTarget.checked)}
                        />
                      </td>
                      <td class="px-4 py-2 font-mono text-xs">
                        <button type="button" class="text-indigo-600 hover:underline" onclick={() => openOrphanDesign(item.id)}>
                          {item.filename}
                        </button>
                      </td>
                      <td class="px-4 py-2 font-mono text-xs">
                        <button
                          type="button"
                          class="text-indigo-600 hover:underline"
                          onclick={() => openOrphanPath(item.filepath)}
                        >
                          {item.filepath}
                        </button>
                      </td>
                      <td class="px-4 py-2">{item.designer || "-"}</td>
                      <td class="px-4 py-2 text-gray-500">{item.date_added || "-"}</td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>

          {#if orphanTotalPages > 1}
            <div class="flex flex-wrap items-center justify-center gap-2 text-sm">
              <button type="button" class="menu-button-secondary" onclick={() => goToOrphanPage(1)} disabled={orphansLoading || orphanPage <= 1}>
                &lt;&lt; First
              </button>
              <button type="button" class="menu-button-secondary" onclick={() => goToOrphanPage(orphanPage - 1)} disabled={orphansLoading || orphanPage <= 1}>
                &lt; Prev
              </button>

              {#each orphanPaginationPages() as pageToken}
                {#if pageToken === "..."}
                  <span class="px-1 text-gray-400">...</span>
                {:else if pageToken === orphanPage}
                  <span class="px-3 py-1 border rounded bg-indigo-600 text-white">{pageToken}</span>
                {:else}
                  <button
                    type="button"
                    class="menu-button-secondary"
                    onclick={() => goToOrphanPage(pageToken)}
                    disabled={orphansLoading}
                  >
                    {pageToken}
                  </button>
                {/if}
              {/each}

              <button type="button" class="menu-button-secondary" onclick={() => goToOrphanPage(orphanPage + 1)} disabled={orphansLoading || orphanPage >= orphanTotalPages}>
                Next &gt;
              </button>
              <button type="button" class="menu-button-secondary" onclick={() => goToOrphanPage(orphanTotalPages)} disabled={orphansLoading || orphanPage >= orphanTotalPages}>
                Last &gt;&gt;
              </button>
            </div>
          {/if}
        </div>
      {:else if currentUiKind === "about"}
        <AboutView
          documents={aboutDocuments}
          loading={aboutDocumentsLoading}
          error={aboutDocumentsError}
        />
      {:else if currentUiKind === "about-document"}
        <AboutDocumentView
          documentItem={aboutDocumentItem}
          loading={aboutDocumentLoading}
          error={aboutDocumentError}
        />
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
      <h1 class="ui-page-title">Route Not Found</h1>
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

{#if browseBulkModalOpen}
  {@const tagOptionsForChooser = browseBulkModalMode === "detail" ? (Array.isArray(detailItem?.all_tags) ? detailItem.all_tags : []) : browseTagOptions}
  {@const groupedTagOptions = splitTagsByGroup(tagOptionsForChooser)}
  <div
    bind:this={browseBulkModalOverlayNode}
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
      onmousedown={closeBulkTagModal}
      onclick={closeBulkTagModal}
    ></button>
    <div
      bind:this={browseBulkModalDialogNode}
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;"
    >
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
        <h2 id="bulk-tag-title" class="text-lg font-semibold" style="margin:0;">
          {browseBulkModalMode === "detail" ? "Choose tags for this design" : "Choose tags for selected designs"}
        </h2>
      </div>
      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p class="text-sm" style="margin:0 0 0.75rem 0;">
          {#if browseBulkModalMode === "detail"}
            {detailItem?.filename || "Current design"}
          {:else}
            {browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.
          {/if}
        </p>

        {#if browseBulkModalMode !== "detail"}
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
        {/if}

        <div class="tag-chooser-sections">
          {#if groupedTagOptions.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image">Image tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.image as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.stitching.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-stitching">Stitching tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.stitching as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-unclassified">Unclassified tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.unclassified as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
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
        <button type="button" class="menu-button-secondary" onmousedown={closeBulkTagModal} onclick={closeBulkTagModal}>Cancel</button>
        <button type="button" class="menu-button-primary" onclick={applySharedTagChooser} disabled={browseBulkModalMode === "detail" && detailSaving}>
          {browseBulkModalMode === "detail" ? "Save tags" : "Apply tags"}
        </button>
      </div>
    </div>
  </div>
{/if}

<div
  bind:this={browseBulkBarNode}
  use:portalToBody
  class={`browse-bulk-bar ui-section-shell no-print ${showBrowseBulkBar ? "" : "hidden"}`}
  style={showBrowseBulkBar
    ? "position:fixed;left:0;right:0;bottom:0;top:auto;display:flex;z-index:2147483000;"
    : "position:fixed;left:0;right:0;bottom:0;top:auto;display:none;z-index:2147483000;"}
>
    <span class="ui-field-label browse-bulk-count">{browseSelectedCount} selected</span>

    <button type="button" class="menu-button-primary ui-action-button ui-action-button-primary browse-bulk-button" onclick={openBulkTagModal}>
      Choose tags...
    </button>

    <button type="button" class="menu-button-primary ui-action-button browse-bulk-button" onclick={verifySelectedBrowseItems}>
      Verify selected
    </button>

    <div class="browse-bulk-project-group">
      <details class="ui-multi-dropdown browse-bulk-project-dropdown" open={browseBulkProjectDropdownOpen}>
        <summary
          class="ui-select-input ui-control-text-inset ui-multi-dropdown-summary list-none cursor-pointer browse-bulk-project-select"
          onclick={toggleBrowseBulkProjectDropdown}
        >
          <span class="ui-multi-dropdown-summary-text">{summarizeBrowseBulkProjectSelection()}</span>
          <span class="ui-control-caret" aria-hidden="true"></span>
        </summary>
        <div class="ui-checkbox-list-shell ui-multi-dropdown-panel browse-bulk-project-panel px-3 py-2 space-y-1 max-h-56 overflow-auto">
          {#if browseProjects.length === 0}
            <p class="ui-help-note">No projects yet.</p>
          {:else}
            {#each browseProjects as project}
              <label class="ui-field-label flex items-center gap-1.5 text-xs">
                <input
                  type="checkbox"
                  class="ui-checkbox"
                  checked={browseBulkProjectSelection.includes(Number(project.id))}
                  onchange={(event) => toggleBrowseBulkProjectSelection(project.id, event.currentTarget.checked)}
                />
                <span>{project.name}</span>
              </label>
            {/each}
          {/if}
        </div>
      </details>
      <button type="button" class="menu-button-primary ui-action-button ui-action-button-primary browse-bulk-button" onclick={addSelectedToProject}>
        Add to project
      </button>
    </div>

      <button
        type="button"
        class="menu-button-secondary ui-action-button browse-bulk-button"
        onclick={openBrowseDeleteConfirm}
        disabled={browseDeleteSelectedBusy}
      >
        Delete selected
      </button>

    <button type="button" class="menu-button-primary ui-action-button ui-action-button-primary browse-bulk-button" onclick={clearBrowseSelection}>
      Clear selection
    </button>
</div>

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
        onmousedown={closeBrowseDeleteConfirm}
        onclick={closeBrowseDeleteConfirm}
      ></button>

      <div
        class="tag-chooser-dialog"
        style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
      >
        <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
          <h2 id="browse-delete-selected-title" class="text-lg font-semibold" style="margin:0;">
            Delete selected designs?
          </h2>
        </div>

        <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
          <p class="text-sm" style="margin:0 0 0.75rem 0;">
            {browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected.
          </p>
          <p class="text-sm" style="margin:0;">
            The design(s) will be deleted from the database, but the file(s) will remain on your computer. Do you really want to do this?
          </p>
        </div>

        <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
          <button
            type="button"
            class="menu-button-secondary"
            onmousedown={closeBrowseDeleteConfirm}
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
    <a href="#/about" class="hover:underline text-indigo-600">About</a>
    <span aria-hidden="true">•</span>
    <a href="#/about/document/licence" class="hover:underline text-indigo-600">Licence</a>
  </div>
</footer>

