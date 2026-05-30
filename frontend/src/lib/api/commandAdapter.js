import { invoke } from "@tauri-apps/api/core";

const MOCK_DESIGNS = [
  {
    id: 1,
    filename: "rose-border-01.pes",
    designer: "Mock Designer",
    source: "Mock Source",
    tags: ["Flowers", "Borders"],
    hoop: "Hoop A",
    rating: 4,
    is_stitched: false,
    tags_checked: true,
  },
  {
    id: 2,
    filename: "holiday-tree.vp3",
    designer: "Mock Studio",
    source: "Imported",
    tags: ["Christmas"],
    hoop: "Hoop B",
    rating: 3,
    is_stitched: true,
    tags_checked: false,
  },
  {
    id: 3,
    filename: "monogram-a.dst",
    designer: "Mock Designer",
    source: "Purchased",
    tags: ["Alphabets"],
    hoop: null,
    rating: null,
    is_stitched: false,
    tags_checked: true,
  },
];

const TAG_SEED = [
  "Line Outline",
  "Satin Stitch",
  "Applique",
  "Food",
  "Nautical",
  "Alphabets",
  "Floral",
  "Butterflies and Insects",
];

function normalizeBrowseItem(raw, index, options = {}) {
  const { useSeedTags = false } = options;
  const id = Number(raw?.id ?? index + 1);
  const filename = String(raw?.filename || raw?.name || `design-${id}.pes`);
  const seed = Math.abs(id || index + 1);
  const seededTags = [TAG_SEED[seed % TAG_SEED.length], TAG_SEED[(seed + 3) % TAG_SEED.length]];

  return {
    id,
    filename,
    designer: String(raw?.designer || "Unknown"),
    source: String(raw?.source || "Unknown"),
    tags:
      Array.isArray(raw?.tags) && raw.tags.length > 0
        ? raw.tags.map(String)
        : useSeedTags
          ? seededTags
          : [],
    image_tags: Array.isArray(raw?.image_tags) ? raw.image_tags.map(String) : [],
    stitching_tags: Array.isArray(raw?.stitching_tags) ? raw.stitching_tags.map(String) : [],
    hoop: raw?.hoop ?? (seed % 2 === 0 ? "Hoop A" : "Hoop B"),
    rating:
      raw?.rating == null || Number.isNaN(Number(raw.rating))
        ? seed % 5
        : Math.max(0, Math.min(5, Number(raw.rating))),
    is_stitched: Boolean(raw?.is_stitched),
    tags_checked: raw?.tags_checked ?? seed % 4 !== 0,
  };
}

/**
 * Try to load designs from Rust command surface.
 * Falls back to local mock data while command migration is in progress.
 */
export async function getBrowseDesigns() {
  try {
    const designs = await invoke("get_designs");
    if (Array.isArray(designs)) {
      return { items: designs.map((item, index) => normalizeBrowseItem(item, index)), source: "rust" };
    }
  } catch (error) {
    console.info("get_designs not available yet, using mock designs.", error);
  }

  return {
    items: MOCK_DESIGNS.map((item, index) => normalizeBrowseItem(item, index, { useSeedTags: true })),
    source: "mock",
  };
}

/**
 * Try to load a single design detail from Rust command surface.
 * Falls back to mock data while detail command migration is in progress.
 */
export async function getDesignDetail(designId) {
  const normalizedId = Number(designId);
  if (!Number.isFinite(normalizedId) || normalizedId <= 0) {
    return { item: null, source: "mock", error: `Invalid design id: ${designId}` };
  }

  try {
    // Send both key styles so the call works across mixed command bindings.
    const detail = await invoke("get_design_detail", {
      designId: normalizedId,
      design_id: normalizedId,
    });
    if (detail && typeof detail === "object") {
      return { item: detail, source: "rust" };
    }
  } catch (error) {
    console.info("get_design_detail not available yet, using mock detail.", error);

    const fallback = MOCK_DESIGNS.find((item) => item.id === normalizedId) || null;
    if (!fallback) {
      return { item: null, source: "mock", error: String(error) };
    }

    return {
      item: {
        id: fallback.id,
        filename: fallback.filename,
        filepath: `C:/mock/${fallback.filename}`,
        designer: fallback.designer,
        source: fallback.source,
        notes: "Mock detail while Rust route migration continues.",
        rating: null,
        date_added: null,
      },
      source: "mock",
      error: String(error),
    };
  }

  const fallback = MOCK_DESIGNS.find((item) => item.id === normalizedId) || null;
  if (!fallback) {
    return { item: null, source: "mock" };
  }

  return {
    item: {
      id: fallback.id,
      filename: fallback.filename,
      filepath: `C:/mock/${fallback.filename}`,
      designer: fallback.designer,
      source: fallback.source,
      notes: "Mock detail while Rust route migration continues.",
      rating: null,
      date_added: null,
    },
    source: "mock",
  };
}

export async function getDesignImageDataUrl(designId) {
  const normalizedId = Number(designId);
  if (!Number.isFinite(normalizedId) || normalizedId <= 0) {
    return { item: null, source: "mock" };
  }

  try {
    const image = await invoke("get_design_image_data_url", { designId: normalizedId });
    if (image && typeof image === "object") {
      return { item: image, source: "rust" };
    }
  } catch (error) {
    console.info("get_design_image_data_url not available yet, using mock image.", error);
  }

  return { item: null, source: "mock" };
}

export async function updateDesignMetadata(designId, request) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("update_design_metadata", {
      designId: normalizedId,
      request,
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design metadata updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not update design metadata: ${error}`,
      error: String(error),
    };
  }
}

export async function setDesignRating(designId, rating) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("set_design_rating", {
      designId: normalizedId,
      request: { rating: rating == null ? null : Number(rating) },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design rating updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not update rating: ${error}`,
      error: String(error),
    };
  }
}

export async function setDesignStitched(designId, isStitched) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("set_design_stitched", {
      designId: normalizedId,
      request: { is_stitched: Boolean(isStitched) },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design stitched state updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not update stitched state: ${error}`,
      error: String(error),
    };
  }
}

export async function setDesignTagsChecked(designId, tagsChecked) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("set_design_tags_checked", {
      designId: normalizedId,
      request: { tags_checked: Boolean(tagsChecked) },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design verification state updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not update verification state: ${error}`,
      error: String(error),
    };
  }
}

export async function setDesignTags(designId, tagIds) {
  const normalizedId = Number(designId);
  const normalizedTagIds = Array.isArray(tagIds)
    ? Array.from(new Set(tagIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)))
    : [];

  try {
    const result = await invoke("set_design_tags", {
      designId: normalizedId,
      request: { tag_ids: normalizedTagIds },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design tags updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not update tags: ${error}`,
      error: String(error),
    };
  }
}

export async function addDesignToProject(designId, projectId) {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invoke("add_design_to_project", {
      designId: normalizedId,
      request: { project_id: normalizedProjectId },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design added to project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not add design to project: ${error}`,
      error: String(error),
    };
  }
}

export async function removeDesignFromProject(designId, projectId) {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invoke("remove_design_from_project", {
      designId: normalizedId,
      projectId: normalizedProjectId,
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design removed from project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not remove design from project: ${error}`,
      error: String(error),
    };
  }
}

export async function deleteDesign(designId) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("delete_design", {
      designId: normalizedId,
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design deleted."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not delete design: ${error}`,
      error: String(error),
    };
  }
}

export async function openDesignInEditor(designId) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("open_design_in_editor", {
      designId: normalizedId,
    });
    return {
      source: "rust",
      persisted: true,
      result,
      message: String(result?.message || "Open in editor action completed."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      result: null,
      message: `Could not open in editor: ${error}`,
      error: String(error),
    };
  }
}

export async function openDesignInExplorer(designId) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("open_design_in_explorer", {
      designId: normalizedId,
    });
    return {
      source: "rust",
      persisted: true,
      result,
      message: String(result?.message || "Show in explorer action completed."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      result: null,
      message: `Could not open in explorer: ${error}`,
      error: String(error),
    };
  }
}

export async function renderDesign3dPreview(designId) {
  const normalizedId = Number(designId);

  try {
    const result = await invoke("render_design_3d_preview", {
      designId: normalizedId,
    });
    return {
      source: "rust",
      persisted: true,
      result,
      message: String(result?.message || "3D preview rendered."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      result: null,
      message: `Could not render 3D preview: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Try import preview using existing Rust bulk import command.
 * Falls back to a mock preview shape if command wiring is incomplete.
 */
export async function previewImportFromRoots(rootPaths) {
  const normalizedRoots = Array.isArray(rootPaths)
    ? rootPaths.map((rootPath) => String(rootPath || "").trim()).filter(Boolean)
    : [];

  if (normalizedRoots.length === 0) {
    return {
      source: "mock",
      preview: {
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        resolved_assignments: [],
      },
      message: "Enter at least one folder path to preview import.",
    };
  }

  try {
    const preview = await invoke("preview_bulk_import", {
      request: {
        root_path: normalizedRoots[0],
        root_paths: normalizedRoots,
        fallback_designer_id: null,
        fallback_source_id: null,
      },
    });

    return {
      source: "rust",
      preview,
      message: "Preview loaded from Rust command.",
    };
  } catch (error) {
    console.info("preview_bulk_import unavailable or failed, using mock preview.", error);
    return {
      source: "mock",
      preview: {
        discovered_count: 3,
        selected_count: 0,
        folder_count: 1,
        resolved_assignments: [],
      },
      message: "Rust preview command not fully wired yet, showing mock counts.",
    };
  }
}

export async function previewImportFromRoot(rootPath) {
  const normalizedRoot = String(rootPath || "").trim();
  return previewImportFromRoots(normalizedRoot ? [normalizedRoot] : []);
}

/**
 * Open native folder picker for import root selection.
 */
export async function browseImportFolder(startDir = "") {
  try {
    const result = await invoke("browse_import_folder", {
      request: {
        start_dir: String(startDir || "").trim() || null,
        allow_multi: true,
      },
    });

    return {
      source: "rust",
      path: String(result?.path || ""),
      paths: Array.isArray(result?.paths) ? result.paths.map((item) => String(item || "")).filter(Boolean) : [],
      message: result?.path ? "Folder selected." : "Folder selection cancelled.",
    };
  } catch (error) {
    console.info("browse_import_folder unavailable or failed, using mock folder browse.", error);
    return {
      source: "mock",
      path: String(startDir || ""),
      paths: [],
      message: "Native folder picker not available in this mode.",
    };
  }
}

/**
 * Run import precheck and persist tokenized import context in Rust backend.
 */
export async function precheckImportWire(confirmWire) {
  const wire = confirmWire && typeof confirmWire === "object" ? confirmWire : null;
  if (!wire) {
    return {
      source: "mock",
      precheck: {
        context_token: "",
        context_token_present: false,
        ready_for_confirm: false,
        is_first_import: false,
        needs_hoop_setup: false,
        root_path_count: 0,
        selected_file_count: 0,
        resolved_assignments: [],
      },
      message: "Missing confirm wire payload.",
    };
  }

  try {
    const precheck = await invoke("precheck_bulk_import_wire", {
      confirmWire: wire,
    });

    return {
      source: "rust",
      precheck,
      message: "Precheck loaded from Rust command.",
    };
  } catch (error) {
    console.info("precheck_bulk_import_wire unavailable or failed, using mock precheck.", error);
    return {
      source: "mock",
      precheck: {
        context_token: "mock-import-token",
        context_token_present: true,
        ready_for_confirm: true,
        is_first_import: false,
        needs_hoop_setup: false,
        root_path_count: Number(wire?.wire?.root_paths?.length ?? 0),
        selected_file_count: Number(wire?.wire?.selected_files?.length ?? 0),
        resolved_assignments: Array.isArray(wire?.wire?.per_folder_assignments)
          ? wire.wire.per_folder_assignments
          : [],
      },
      message: "Rust precheck command not fully wired yet, using mock precheck.",
    };
  }
}

/**
 * Execute Step 3 precheck action in Rust backend.
 */
export async function runPrecheckAction({ contextToken, action, confirmSkipHoops = false }) {
  const normalizedToken = String(contextToken || "").trim();
  const normalizedAction = String(action || "").trim();

  if (!normalizedToken || !normalizedAction) {
    return {
      source: "mock",
      actionResult: {
        action: normalizedAction || "",
        context_token_present: false,
        consumed_context: false,
        requires_skip_hoops_confirmation: false,
        next_route: null,
        confirm_result: null,
      },
      message: "Missing precheck action payload.",
    };
  }

  try {
    const actionResult = await invoke("precheck_bulk_import_action_wire", {
      request: {
        context_token: normalizedToken,
        action: normalizedAction,
        confirm_skip_hoops: Boolean(confirmSkipHoops),
      },
    });

    return {
      source: "rust",
      actionResult,
      message: "Precheck action loaded from Rust command.",
    };
  } catch (error) {
    console.info("precheck_bulk_import_action_wire unavailable or failed, using mock action result.", error);
    const isImportNow = normalizedAction === "import_now";
    const isCancel = normalizedAction === "cancel";

    return {
      source: "mock",
      actionResult: {
        action: normalizedAction,
        context_token_present: !isCancel,
        consumed_context: isImportNow || isCancel,
        requires_skip_hoops_confirmation: false,
        next_route: isImportNow ? "/designs/" : isCancel ? "/import/" : null,
        confirm_result: null,
      },
      message: "Rust precheck action command not fully wired yet, using mock result.",
    };
  }
}

/**
 * Request stop for the currently running bulk import.
 */
export async function requestStopBulkImport() {
  try {
    const result = await invoke("request_stop_bulk_import");
    return {
      source: "rust",
      stopRequested: Boolean(result?.stop_requested),
      message: "Stop requested for the running import.",
    };
  } catch (error) {
    console.info("request_stop_bulk_import unavailable or failed, using mock stop result.", error);
    return {
      source: "mock",
      stopRequested: true,
      message: "Stop requested (mock).",
    };
  }
}

/**
 * Mark selected designs as verified in Rust backend.
 * Falls back to local-only behavior while route wiring is in progress.
 */
export async function bulkVerifyDesigns(designIds) {
  const normalizedIds = Array.isArray(designIds)
    ? designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
    : [];

  if (normalizedIds.length === 0) {
    return {
      source: "mock",
      requested_count: 0,
      verified_count: 0,
      persisted: false,
    };
  }

  try {
    const result = await invoke("bulk_verify_designs", { designIds: normalizedIds });
    return {
      source: "rust",
      requested_count: Number(result?.requested_count ?? normalizedIds.length),
      verified_count: Number(result?.verified_count ?? 0),
      persisted: true,
    };
  } catch (error) {
    console.info("bulk_verify_designs unavailable or failed, using local verify fallback.", error);
    return {
      source: "mock",
      requested_count: normalizedIds.length,
      verified_count: normalizedIds.length,
      persisted: false,
    };
  }
}

/**
 * Load project choices for browse bulk actions.
 * Falls back to mock project options while migration is in progress.
 */
export async function getBrowseProjects() {
  try {
    const projects = await invoke("get_projects_for_browse");
    if (Array.isArray(projects)) {
      return { items: projects, source: "rust" };
    }
  } catch (error) {
    console.info("get_projects_for_browse unavailable, using mock projects.", error);
  }

  return {
    items: [
      { id: 1, name: "Project A" },
      { id: 2, name: "Project B" },
      { id: 3, name: "Project C" },
    ],
    source: "mock",
  };
}

export async function getProjectsList() {
  const REQUEST_TIMEOUT_MS = 15000;

  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => {
      reject(new Error(`Timed out loading projects after ${REQUEST_TIMEOUT_MS / 1000}s.`));
    }, REQUEST_TIMEOUT_MS);
  });

  try {
    const projects = await Promise.race([invoke("get_projects_list"), timeoutPromise]);
    if (Array.isArray(projects)) {
      return { items: projects, source: "rust" };
    }
  } catch (error) {
    console.info("get_projects_list unavailable or timed out, using empty fallback.", error);
    return {
      items: [],
      source: "mock",
      error: `Could not load projects: ${String(error)}`,
    };
  }

  return { items: [], source: "mock" };
}

export async function createProject(name, description) {
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invoke("create_project", { request: payload });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || 0),
      message: String(result?.message || "Project created."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: 0,
      message: `Could not create project: ${error}`,
      error: String(error),
    };
  }
}

export async function getProjectDetail(projectId) {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const detail = await invoke("get_project_detail", { projectId: normalizedProjectId });
    if (detail && typeof detail === "object") {
      return { item: detail, source: "rust" };
    }
  } catch (error) {
    return {
      item: null,
      source: "mock",
      error: `Could not load project detail: ${error}`,
    };
  }

  return { item: null, source: "mock", error: "Project detail was empty." };
}

export async function updateProject(projectId, name, description) {
  const normalizedProjectId = Number(projectId);
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invoke("update_project", {
      projectId: normalizedProjectId,
      request: payload,
    });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      message: String(result?.message || "Project updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      message: `Could not update project: ${error}`,
      error: String(error),
    };
  }
}

export async function deleteProject(projectId) {
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invoke("delete_project", { projectId: normalizedProjectId });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      message: String(result?.message || "Project deleted."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      message: `Could not delete project: ${error}`,
      error: String(error),
    };
  }
}

export async function removeDesignFromProjectDetail(projectId, designId) {
  const normalizedProjectId = Number(projectId);
  const normalizedDesignId = Number(designId);

  try {
    const result = await invoke("remove_design_from_project_detail", {
      projectId: normalizedProjectId,
      designId: normalizedDesignId,
    });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      design_id: Number(result?.design_id || normalizedDesignId),
      message: String(result?.message || "Design removed from project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      design_id: normalizedDesignId,
      message: `Could not remove design from project: ${error}`,
      error: String(error),
    };
  }
}

export async function getProjectPrintView(projectId) {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const view = await invoke("get_project_print_view", { projectId: normalizedProjectId });
    if (view && typeof view === "object") {
      return { item: view, source: "rust" };
    }
  } catch (error) {
    return {
      item: null,
      source: "mock",
      error: `Could not load project print view: ${error}`,
    };
  }

  return { item: null, source: "mock", error: "Project print view was empty." };
}

/**
 * Add selected designs to a project in Rust backend.
 * Falls back to local-only behavior while route wiring is in progress.
 */
export async function bulkAddDesignsToProject(projectId, designIds) {
  const normalizedProjectId = Number(projectId);
  const normalizedIds = Array.isArray(designIds)
    ? designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
    : [];

  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0 || normalizedIds.length === 0) {
    return {
      source: "mock",
      project_id: normalizedProjectId,
      requested_count: normalizedIds.length,
      added_count: 0,
      persisted: false,
    };
  }

  try {
    // Send both key styles so this works across mixed command bindings.
    const result = await invoke("bulk_add_designs_to_project", {
      projectId: normalizedProjectId,
      designIds: normalizedIds,
      project_id: normalizedProjectId,
      design_ids: normalizedIds,
    });
    return {
      source: "rust",
      project_id: Number(result?.project_id ?? normalizedProjectId),
      requested_count: Number(result?.requested_count ?? normalizedIds.length),
      added_count: Number(result?.added_count ?? 0),
      persisted: true,
    };
  } catch (error) {
    console.info("bulk_add_designs_to_project unavailable or failed, using local fallback.", error);
    return {
      source: "mock",
      project_id: normalizedProjectId,
      requested_count: normalizedIds.length,
      added_count: 0,
      persisted: false,
      error: String(error),
    };
  }
}

/**
 * Load tag options for browse filters and bulk-tag modal.
 * Falls back to tag names derived from mock designs while migration is in progress.
 */
export async function getBrowseTags() {
  try {
    const tags = await invoke("get_tags_for_browse");
    if (Array.isArray(tags)) {
      return {
        items: tags.map((tag) => ({
          id: Number(tag?.id),
          description: String(tag?.description || ""),
          tag_group: tag?.tag_group == null ? null : String(tag.tag_group),
        })),
        source: "rust",
      };
    }
  } catch (error) {
    console.info("get_tags_for_browse unavailable, using mock tag options.", error);
  }

  const fromMock = Array.from(new Set(MOCK_DESIGNS.flatMap((item) => item.tags || []))).map(
    (description, index) => ({
      id: index + 1,
      description,
      tag_group: null,
    })
  );

  return { items: fromMock, source: "mock" };
}

/**
 * Replace tag assignments for selected designs in Rust backend.
 * Falls back to local-only behavior while route wiring is in progress.
 */
export async function bulkSetTagsForDesigns(designIds, tagIds) {
  const normalizedDesignIds = Array.isArray(designIds)
    ? designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
    : [];
  const normalizedTagIds = Array.isArray(tagIds)
    ? Array.from(new Set(tagIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)))
    : [];

  if (normalizedDesignIds.length === 0) {
    return {
      source: "mock",
      requested_count: 0,
      updated_count: 0,
      persisted: false,
    };
  }

  try {
    const result = await invoke("bulk_set_tags_for_designs", {
      designIds: normalizedDesignIds,
      tagIds: normalizedTagIds,
    });
    return {
      source: "rust",
      requested_count: Number(result?.requested_count ?? normalizedDesignIds.length),
      updated_count: Number(result?.updated_count ?? 0),
      persisted: true,
    };
  } catch (error) {
    console.info("bulk_set_tags_for_designs unavailable or failed, using local fallback.", error);
    return {
      source: "mock",
      requested_count: normalizedDesignIds.length,
      updated_count: normalizedDesignIds.length,
      persisted: false,
    };
  }
}

/**
 * Fetch page-scoped preview image data URLs for browse cards.
 * Falls back to empty previews if unavailable.
 */
export async function getBrowseDesignPreviews(designIds) {
  const normalizedIds = Array.isArray(designIds)
    ? Array.from(new Set(designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)))
    : [];

  if (normalizedIds.length === 0) {
    return { items: [], source: "mock" };
  }

  try {
    const previews = await invoke("get_design_previews_for_browse", { designIds: normalizedIds });
    if (Array.isArray(previews)) {
      return {
        items: previews.map((item) => ({
          id: Number(item?.id),
          data_url: item?.data_url ? String(item.data_url) : null,
        })),
        source: "rust",
      };
    }
  } catch (error) {
    console.info("get_design_previews_for_browse unavailable, using empty previews.", error);
  }

  return {
    items: normalizedIds.map((id) => ({ id, data_url: null })),
    source: "mock",
  };
}

export async function getAboutDocuments() {
  try {
    const docs = await invoke("get_about_documents");
    if (Array.isArray(docs)) {
      return {
        source: "rust",
        items: docs.map((doc) => ({
          slug: String(doc?.slug || ""),
          title: String(doc?.title || ""),
          description: String(doc?.description || ""),
          filename: String(doc?.filename || ""),
          available: Boolean(doc?.available),
        })),
      };
    }
  } catch (error) {
    console.info("get_about_documents unavailable, using mock fallback.", error);
  }

  return {
    source: "mock",
    items: [
      {
        slug: "disclaimer",
        title: "Disclaimer",
        description: "Important use-at-your-own-risk and limitation-of-liability information.",
        filename: "DISCLAIMER.html",
        available: false,
      },
      {
        slug: "privacy",
        title: "Privacy",
        description: "Explains what data is stored locally and what optional AI features may send externally.",
        filename: "PRIVACY.md",
        available: false,
      },
      {
        slug: "security",
        title: "Security",
        description: "Guidance on secrets, API keys, portable deployments, and safe usage.",
        filename: "SECURITY.md",
        available: false,
      },
      {
        slug: "ai-tagging",
        title: "AI Tagging Guide",
        description: "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
        filename: "docs/User-Facing-Guidance/AI_TAGGING.md",
        available: false,
      },
      {
        slug: "third-party-notices",
        title: "Third-Party Notices",
        description: "Licensing and attribution information for bundled and dependency software.",
        filename: "THIRD_PARTY_NOTICES.md",
        available: false,
      },
      {
        slug: "licence",
        title: "Licence",
        description: "The licence terms for the Embroidery Catalogue project itself.",
        filename: "LICENCE",
        available: false,
      },
    ],
  };
}

export async function getAboutDocument(slug) {
  const normalizedSlug = String(slug || "").trim().toLowerCase();
  if (!normalizedSlug) {
    return { item: null, source: "mock", error: "Document not found." };
  }

  try {
    const item = await invoke("get_about_document", { slug: normalizedSlug });
    if (item && typeof item === "object") {
      return {
        source: "rust",
        item: {
          slug: String(item?.slug || normalizedSlug),
          title: String(item?.title || ""),
          description: String(item?.description || ""),
          filename: String(item?.filename || ""),
          document_text: String(item?.document_text || ""),
        },
      };
    }
  } catch (error) {
    return {
      source: "mock",
      item: null,
      error: String(error),
    };
  }

  return {
    source: "mock",
    item: null,
    error: "Document not found.",
  };
}

/**
 * Load settings from Rust backend.
 */
export async function getSettingsViewModel() {
  try {
    const model = await invoke("get_settings_view_model");
    if (model && typeof model === "object") {
      return { model, source: "rust" };
    }
  } catch (error) {
    console.info("get_settings_view_model unavailable, using local fallback.", error);
  }

  return {
    source: "mock",
    model: {
      image_preference: "2d",
      google_api_key: "",
      has_google_api_key: false,
      ai_tier2_auto: false,
      ai_tier3_auto: false,
      ai_batch_size: "",
      ai_delay: "",
      import_commit_batch_size: "",
      import_last_browse_folder: "",
      can_configure_data_root: false,
      data_root: "",
      log_folder: "",
      app_mode: "development",
      ai_tagging_help_url: "#/help",
    },
  };
}

/**
 * Save settings via Rust backend.
 */
export async function saveSettings(request) {
  try {
    const result = await invoke("save_settings_view_model", { request });
    return {
      source: "rust",
      saved: Boolean(result?.saved),
      message: String(result?.message || "Settings saved successfully."),
      persisted: true,
    };
  } catch (error) {
    console.info("save_settings_view_model failed.", error);
    return {
      source: "mock",
      saved: false,
      message: `Could not save settings: ${error}`,
      persisted: false,
    };
  }
}

export async function saveImportLastBrowseFolder(path) {
  try {
    const result = await invoke("save_import_last_browse_folder", { path: String(path || "") });
    return {
      source: "rust",
      saved: Boolean(result?.saved),
      path: String(result?.path || ""),
      persisted: true,
    };
  } catch (error) {
    console.info("save_import_last_browse_folder failed.", error);
    return {
      source: "mock",
      saved: false,
      path: String(path || ""),
      persisted: false,
      error: String(error),
    };
  }
}

/**
 * Open settings data-root folder picker when available.
 */
export async function browseSettingsDataRoot(startDir) {
  try {
    const result = await invoke("browse_settings_data_root", { startDir });
    return {
      source: "rust",
      path: result?.path ? String(result.path) : null,
      error: result?.error ? String(result.error) : null,
    };
  } catch (error) {
    console.info("browse_settings_data_root failed.", error);
    return {
      source: "mock",
      path: null,
      error: `Folder picker unavailable: ${error}`,
    };
  }
}

export async function getTaggingActionsViewModel() {
  try {
    const model = await invoke("get_tagging_actions_view_model");
    return {
      source: "rust",
      model: {
        has_google_api_key: Boolean(model?.has_google_api_key),
        ai_tier2_auto: Boolean(model?.ai_tier2_auto),
        ai_tier3_auto: Boolean(model?.ai_tier3_auto),
        ai_batch_size: String(model?.ai_batch_size || ""),
        ai_delay: String(model?.ai_delay || ""),
        import_commit_batch_size: String(model?.import_commit_batch_size || ""),
        default_batch_size: Number(model?.default_batch_size ?? 100),
        default_commit_every: Number(model?.default_commit_every ?? 100),
        default_workers: Number(model?.default_workers ?? 4),
      },
    };
  } catch (error) {
    return {
      source: "mock",
      model: {
        has_google_api_key: false,
        ai_tier2_auto: false,
        ai_tier3_auto: false,
        ai_batch_size: "",
        ai_delay: "",
        import_commit_batch_size: "",
        default_batch_size: 100,
        default_commit_every: 100,
        default_workers: 4,
      },
      error: String(error),
    };
  }
}

export async function runUnifiedBackfill(request) {
  try {
    const result = await invoke("run_unified_backfill", { request });
    return {
      source: "rust",
      processed: Number(result?.processed ?? 0),
      errors: Number(result?.errors ?? 0),
      stopped: Boolean(result?.stopped),
      actions: Array.isArray(result?.actions) ? result.actions.map(String) : [],
      commit_every: Number(result?.commit_every ?? 100),
      batch_size: Number(result?.batch_size ?? 100),
      workers: Number(result?.workers ?? 4),
    };
  } catch (error) {
    return {
      source: "mock",
      processed: 0,
      errors: 1,
      stopped: false,
      actions: [],
      error: String(error),
    };
  }
}

export async function stopUnifiedBackfill() {
  try {
    const result = await invoke("stop_unified_backfill");
    return {
      source: "rust",
      status: String(result?.status || "stopping"),
    };
  } catch (error) {
    return {
      source: "mock",
      status: "stopping",
      error: String(error),
    };
  }
}

export async function getBackfillLogEntries(limit = 20) {
  try {
    const entries = await invoke("get_backfill_log_entries", { limit: Number(limit) });
    if (Array.isArray(entries)) {
      return {
        source: "rust",
        entries: entries.map((entry) => ({
          level: String(entry?.level || "info"),
          message: String(entry?.message || ""),
        })),
      };
    }
  } catch (error) {
    return {
      source: "mock",
      entries: [{ level: "error", message: String(error) }],
    };
  }

  return {
    source: "mock",
    entries: [],
  };
}

export async function runStitchingBackfill({ clearExistingStitching = false, batchSize = 100 } = {}) {
  try {
    const result = await invoke("run_stitching_backfill", {
      clearExistingStitching: Boolean(clearExistingStitching),
      batchSize: Number(batchSize),
    });
    return {
      source: "rust",
      processed: Number(result?.processed ?? 0),
      errors: Number(result?.errors ?? 0),
      stopped: Boolean(result?.stopped),
      actions: Array.isArray(result?.actions) ? result.actions.map(String) : [],
    };
  } catch (error) {
    return {
      source: "mock",
      processed: 0,
      errors: 1,
      stopped: false,
      actions: ["stitching"],
      error: String(error),
    };
  }
}

export async function getBackupViewModel() {
  try {
    const model = await invoke("get_backup_view_model");
    return {
      source: "rust",
      model: {
        db_destination: String(model?.db_destination || ""),
        designs_destination: String(model?.designs_destination || ""),
        db_source_path: String(model?.db_source_path || ""),
        designs_source_path: String(model?.designs_source_path || ""),
      },
    };
  } catch (error) {
    console.info("get_backup_view_model unavailable, using local fallback.", error);
    return {
      source: "mock",
      model: {
        db_destination: "",
        designs_destination: "",
        db_source_path: "",
        designs_source_path: "",
      },
      error: String(error),
    };
  }
}

export async function saveBackupSettings({ dbDestination, designsDestination }) {
  try {
    const result = await invoke("save_backup_settings", {
      request: {
        db_destination: String(dbDestination || ""),
        designs_destination: String(designsDestination || ""),
      },
    });

    return {
      source: "rust",
      persisted: Boolean(result?.saved),
      saved: Boolean(result?.saved),
      message: String(result?.message || "Backup destinations saved."),
      db_destination: String(result?.db_destination || ""),
      designs_destination: String(result?.designs_destination || ""),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      saved: false,
      message: `Could not save backup destinations: ${error}`,
      error: String(error),
    };
  }
}

export async function browseBackupFolder(startDir = "") {
  try {
    const result = await invoke("browse_backup_folder", {
      startDir: String(startDir || "") || null,
    });

    return {
      source: "rust",
      path: result?.path ? String(result.path) : null,
      error: result?.error ? String(result.error) : null,
    };
  } catch (error) {
    return {
      source: "mock",
      path: null,
      error: `Folder picker unavailable: ${error}`,
    };
  }
}

export async function runDatabaseBackup() {
  try {
    const result = await invoke("run_database_backup");
    return {
      source: "rust",
      success: Boolean(result?.success),
      backup_path: result?.backup_path ? String(result.backup_path) : "",
      size_bytes: Number(result?.size_bytes ?? 0),
      completed_at: String(result?.completed_at || ""),
      error: result?.error ? String(result.error) : "",
    };
  } catch (error) {
    return {
      source: "mock",
      success: false,
      backup_path: "",
      size_bytes: 0,
      completed_at: "",
      error: String(error),
    };
  }
}

export async function runDesignsBackup() {
  try {
    const result = await invoke("run_designs_backup");
    return {
      source: "rust",
      success: Boolean(result?.success),
      scanned: Number(result?.scanned ?? 0),
      copied: Number(result?.copied ?? 0),
      updated: Number(result?.updated ?? 0),
      unchanged: Number(result?.unchanged ?? 0),
      archived: Number(result?.archived ?? 0),
      total_bytes_copied: Number(result?.total_bytes_copied ?? 0),
      completed_at: String(result?.completed_at || ""),
      error: result?.error ? String(result.error) : "",
    };
  } catch (error) {
    return {
      source: "mock",
      success: false,
      scanned: 0,
      copied: 0,
      updated: 0,
      unchanged: 0,
      archived: 0,
      total_bytes_copied: 0,
      completed_at: "",
      error: String(error),
    };
  }
}

export async function runBothBackups() {
  try {
    const result = await invoke("run_both_backups");
    return {
      source: "rust",
      database: result?.database || null,
      designs: result?.designs || null,
    };
  } catch (error) {
    return {
      source: "mock",
      database: null,
      designs: null,
      error: String(error),
    };
  }
}

export async function listDesigners() {
  try {
    const items = await invoke("list_designers");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({ id: Number(item?.id), name: String(item?.name || "") })),
      };
    }
  } catch (error) {
    console.info("list_designers unavailable, using mock designers.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, name: "Amazing Designs" },
      { id: 2, name: "Urban Threads" },
      { id: 3, name: "Mock Studio" },
    ],
  };
}

export async function createDesigner(name) {
  try {
    const item = await invoke("create_designer", { request: { name } });
    return {
      source: "rust",
      persisted: true,
      item: { id: Number(item?.id), name: String(item?.name || "") },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function deleteDesigner(designerId) {
  try {
    await invoke("delete_designer", { designerId: Number(designerId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function listSources() {
  try {
    const items = await invoke("list_sources");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({ id: Number(item?.id), name: String(item?.name || "") })),
      };
    }
  } catch (error) {
    console.info("list_sources unavailable, using mock sources.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, name: "Purchased" },
      { id: 2, name: "Downloaded" },
      { id: 3, name: "Gift" },
    ],
  };
}

export async function createSource(name) {
  try {
    const item = await invoke("create_source", { request: { name } });
    return {
      source: "rust",
      persisted: true,
      item: { id: Number(item?.id), name: String(item?.name || "") },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function deleteSource(sourceId) {
  try {
    await invoke("delete_source", { sourceId: Number(sourceId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function listTags() {
  try {
    const items = await invoke("list_tags");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          description: String(item?.description || ""),
          tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        })),
      };
    }
  } catch (error) {
    console.info("list_tags unavailable, using mock tags.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, description: "Animals", tag_group: "image" },
      { id: 2, description: "Flowers", tag_group: "image" },
      { id: 3, description: "Cross Stitch", tag_group: "stitching" },
      { id: 4, description: "Holiday", tag_group: "" },
    ],
  };
}

export async function createTag(description, tagGroup) {
  try {
    const item = await invoke("create_tag", {
      request: {
        description,
        tag_group: tagGroup,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function setTagGroup(tagId, tagGroup) {
  try {
    const item = await invoke("set_tag_group", {
      request: { tag_id: Number(tagId), tag_group: tagGroup },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function deleteTag(tagId) {
  try {
    await invoke("delete_tag", { tagId: Number(tagId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function listHoops() {
  try {
    const items = await invoke("list_hoops");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          name: String(item?.name || ""),
          max_width_mm: Number(item?.max_width_mm ?? 0),
          max_height_mm: Number(item?.max_height_mm ?? 0),
          design_count: Number(item?.design_count ?? 0),
        })),
      };
    }
  } catch (error) {
    console.info("list_hoops unavailable, using mock hoops.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, name: "4x4 hoop", max_width_mm: 100, max_height_mm: 100, design_count: 0 },
      { id: 2, name: "5x7 hoop", max_width_mm: 130, max_height_mm: 180, design_count: 2 },
      { id: 3, name: "6x10 hoop", max_width_mm: 160, max_height_mm: 260, design_count: 0 },
    ],
  };
}

export async function createHoop(name, maxWidthMm, maxHeightMm) {
  try {
    const item = await invoke("create_hoop", {
      request: {
        name,
        max_width_mm: Number(maxWidthMm),
        max_height_mm: Number(maxHeightMm),
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        max_width_mm: Number(item?.max_width_mm ?? 0),
        max_height_mm: Number(item?.max_height_mm ?? 0),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function updateHoop(hoopId, name, maxWidthMm, maxHeightMm) {
  try {
    const item = await invoke("update_hoop", {
      request: {
        hoop_id: Number(hoopId),
        name,
        max_width_mm: Number(maxWidthMm),
        max_height_mm: Number(maxHeightMm),
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        max_width_mm: Number(item?.max_width_mm ?? 0),
        max_height_mm: Number(item?.max_height_mm ?? 0),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function deleteHoop(hoopId) {
  try {
    await invoke("delete_hoop", { hoopId: Number(hoopId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}
