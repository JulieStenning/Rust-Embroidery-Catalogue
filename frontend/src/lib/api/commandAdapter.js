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

function normalizeBrowseItem(raw, index) {
  const id = Number(raw?.id ?? index + 1);
  const filename = String(raw?.filename || raw?.name || `design-${id}.pes`);
  const seed = Math.abs(id || index + 1);
  const seededTags = [TAG_SEED[seed % TAG_SEED.length], TAG_SEED[(seed + 3) % TAG_SEED.length]];

  return {
    id,
    filename,
    designer: String(raw?.designer || "Unknown"),
    source: String(raw?.source || "Unknown"),
    tags: Array.isArray(raw?.tags) && raw.tags.length > 0 ? raw.tags : seededTags,
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
      return { items: designs.map(normalizeBrowseItem), source: "rust" };
    }
  } catch (error) {
    console.info("get_designs not available yet, using mock designs.", error);
  }

  return { items: MOCK_DESIGNS.map(normalizeBrowseItem), source: "mock" };
}

/**
 * Try to load a single design detail from Rust command surface.
 * Falls back to mock data while detail command migration is in progress.
 */
export async function getDesignDetail(designId) {
  try {
    const detail = await invoke("get_design_detail", { designId });
    if (detail && typeof detail === "object") {
      return { item: detail, source: "rust" };
    }
  } catch (error) {
    console.info("get_design_detail not available yet, using mock detail.", error);
  }

  const fallback = MOCK_DESIGNS.find((item) => item.id === designId) || null;
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

/**
 * Try import preview using existing Rust bulk import command.
 * Falls back to a mock preview shape if command wiring is incomplete.
 */
export async function previewImportFromRoot(rootPath) {
  const normalizedRoot = (rootPath || "").trim();
  if (!normalizedRoot) {
    return {
      source: "mock",
      preview: {
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        resolved_assignments: [],
      },
      message: "Enter a folder path to preview import.",
    };
  }

  try {
    const preview = await invoke("preview_bulk_import", {
      request: {
        root_path: normalizedRoot,
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
    const result = await invoke("bulk_add_designs_to_project", {
      projectId: normalizedProjectId,
      designIds: normalizedIds,
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
      added_count: normalizedIds.length,
      persisted: false,
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
