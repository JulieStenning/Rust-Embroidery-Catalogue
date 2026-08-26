import { invoke } from "@tauri-apps/api/core";
import type {
  AdapterBackfillLogEntriesResponse,
  AdapterBackupViewModelResponse,
  AdapterBrowseBackupFolderResponse,
  AdapterBrowseDesignsPageResponse,
  AdapterBrowseImportFolderResponse,
  AdapterBrowseOrphanPathResponse,
  AdapterDeleteOrphansResponse,
  AdapterImportPrecheckActionResponse,
  AdapterImportPrecheckResponse,
  AdapterImportPreviewResponse,
  AdapterOrphansPageResponse,
  AdapterPersistedItemResponse,
  AdapterPersistedResponse,
  AdapterProjectDesignMutationResponse,
  AdapterProjectDetailResponse,
  AdapterProjectListResponse,
  AdapterProjectMutationResponse,
  AdapterReparseDesignResponse,
  AdapterRunBothBackupsResponse,
  AdapterSaveBackupSettingsResponse,
  AdapterScanOrphansResponse,
  AdapterStopBulkImportResponse,
  AdapterStopUnifiedBackfillResponse,
  AdapterTaggingActionsViewModelResponse,
  AdapterAppStatusResponse,
  AdapterBrowseDataRootResponse,
  AdapterCompactResponse,
  AdapterConfigureDataRootResponse,
  AdapterDbStatsResponse,
  AdapterGoogleApiKeyResponse,
  AdapterItemResponse,
  AdapterListResponse,
  AdapterMutationResponse,
  AdapterSaveSettingsResponse,
  AdapterSettingsViewModelResponse,
  AdminEntitySummary,
  AdminHoopSummary,
  AdminTagSummary,
  AppStatus,
  BackupViewModel,
  BulkImportPreview,
  BrowseImportFolderResult,
  BrowseDesignPreview,
  BrowseDesignSummaryWire,
  BrowseTagOption,
  CancelBackupResult,
  CancelRestoreResult,
  CompactResult,
  ConfigureDataRootResult,
  DatabaseBackupResult,
  DatabaseStatus,
  DatabaseValidation,
  DetectUnmatchedFilesResult,
  DetectedDataRoot,
  DbStats,
  DesignCommandResult,
  DesignDetail,
  DesignDetailWire,
  DesignsBackupResult,
  DesignImageData,
  ImportPrecheckActionResult,
  ImportPrecheckResult,
  ProjectDetailView,
  ProjectMutationResult,
  ProjectListItem,
  ProjectSummary,
  RemoveProjectDesignResult,
  ReparseDesignResultWire,
  RunStitchingBackfillOptions,
  SaveBackupSettingsRequest,
  SaveSettingsRequest,
  SearchPayload,
  SettingsViewModel,
  TaggingActionsViewModel,
  UnifiedBackfillRequest,
  UnifiedBackfillResult,
  UnifiedBackfillWireRequest,
  UpdateDesignMetadataRequest,
  StorageMigrationProgress,
  StorageMigrationSummary,
  ImportUnmatchedFilesResult,
  RestoreBothResult,
  RestoreDatabaseResult,
  RestoreDesignsResult,
  RestoreProgress,
  BrowseRestoreFileResponse,
} from "../types/ipc";
import { mapDesignDetailFromWire, mapReparseDesignFromWire } from "../types/ipc";

type LooseRecord = Record<string, unknown>;

function invokeLoose<T = LooseRecord>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const result = args === undefined ? invoke(command) : invoke(command, args);

    if (result && typeof (result as PromiseLike<T>).then === "function") {
      return new Promise<T>((resolve, reject) => {
        try {
          (result as PromiseLike<T>).then(
            (value) => resolve(value as T),
            (reason) => reject(reason)
          );
        } catch (error) {
          reject(error);
        }
      });
    }

    return Promise.resolve(result as T);
  } catch (error) {
    return Promise.reject(error);
  }
}

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
    image_tags_verified: true,
    stitching_tags_verified: true,
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
    image_tags_verified: false,
    stitching_tags_verified: false,
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
    image_tags_verified: true,
    stitching_tags_verified: true,
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

const MOCK_HOOPS = [
  { id: 1, name: "Hoop A" },
  { id: 2, name: "Hoop B" },
  { id: 3, name: "Hoop C" },
];

/** @param {LooseRecord | null | undefined} raw @param {number} index @param {{ useSeedTags?: boolean }} [options] */
function normalizeBrowseItem(
  raw: LooseRecord | null | undefined,
  index: number,
  options: { useSeedTags?: boolean } = {}
): BrowseDesignSummaryWire {
  const { useSeedTags = false } = options;
  const id = Number(raw?.id ?? index + 1);
  const filename = String(raw?.filename || raw?.name || `design-${id}.pes`);
  const seed = Math.abs(id || index + 1);
  const seededTags = [TAG_SEED[seed % TAG_SEED.length], TAG_SEED[(seed + 3) % TAG_SEED.length]];

  return {
    id,
    filename,
    filepath: String(raw?.filepath || ""),
    designer: String(raw?.designer || "Unknown"),
    source: String(raw?.source || "Unknown"),
    projects: Array.isArray(raw?.projects)
      ? raw.projects
      : Array.isArray(raw?.project_names)
        ? raw.project_names
        : typeof raw?.projects === "string"
          ? raw.projects
              .split(",")
              .map((value: string) => value.trim())
              .filter(Boolean)
          : typeof raw?.project_names === "string"
            ? raw.project_names
                .split(",")
                .map((value: string) => value.trim())
                .filter(Boolean)
            : [],
    tags:
      Array.isArray(raw?.tags) && raw.tags.length > 0
        ? raw.tags.map(String)
        : useSeedTags
          ? seededTags
          : [],
    image_tags: Array.isArray(raw?.image_tags) ? raw.image_tags.map(String) : [],
    stitching_tags: Array.isArray(raw?.stitching_tags) ? raw.stitching_tags.map(String) : [],
    hoop: raw?.hoop == null ? null : String(raw.hoop),
    rating:
      raw?.rating == null || Number.isNaN(Number(raw.rating))
        ? null
        : Math.max(0, Math.min(5, Number(raw.rating))),
    is_stitched: Boolean(raw?.is_stitched),
    image_tags_verified: Boolean(raw?.image_tags_verified ?? seed % 4 !== 0),
    stitching_tags_verified: Boolean(
      raw?.stitching_tags_verified ?? raw?.image_tags_verified ?? seed % 4 !== 0
    ),
  };
}

/**
 * Try to load designs from Rust command surface.
 * Falls back to local mock data while command migration is in progress.
 * @param {import("../types").SearchPayload} [payload]
 */
export async function getBrowseDesigns(
  payload?: SearchPayload
): Promise<AdapterBrowseDesignsPageResponse> {
  try {
    const result = await invokeLoose<LooseRecord | null | undefined>("get_designs", { payload });
    if (result && Array.isArray(result.items)) {
      return {
        source: "rust",
        page: Number(result.page ?? 1),
        page_size: Number(result.page_size ?? 50),
        total: Number(result.total ?? 0),
        total_pages: Number(result.total_pages ?? 1),
        items: result.items.map((item, index) =>
          normalizeBrowseItem(item as unknown as LooseRecord, index)
        ),
      };
    }
  } catch (error) {
    console.info("get_designs not available yet, using mock designs.", error);
  }

  return {
    source: "mock",
    page: 1,
    page_size: 50,
    total: MOCK_DESIGNS.length,
    total_pages: Math.max(1, Math.ceil(MOCK_DESIGNS.length / 50)),
    items: MOCK_DESIGNS.map((item, index) =>
      normalizeBrowseItem(item, index, { useSeedTags: true })
    ),
  };
}

/**
 * Try to load a single design detail from Rust command surface.
 * Falls back to mock data while detail command migration is in progress.
 * @param {number | string} designId
 */
export async function getDesignDetail(
  designId: number | string
): Promise<AdapterItemResponse<DesignDetail>> {
  const normalizedId = Number(designId);
  if (!Number.isFinite(normalizedId) || normalizedId <= 0) {
    return { item: null, source: "mock", error: `Invalid design id: ${designId}` };
  }

  let invokeError = null;
  try {
    const detail = await invokeLoose<DesignDetailWire | null>("get_design_detail", {
      designId: normalizedId,
    });
    if (detail && typeof detail === "object") {
      return { item: mapDesignDetailFromWire(detail), source: "rust" };
    }
  } catch (error) {
    invokeError = error;
  }

  try {
    const detail = await invokeLoose<DesignDetailWire | null>("get_design_detail", {
      designId: normalizedId,
    });
    if (detail && typeof detail === "object") {
      return { item: mapDesignDetailFromWire(detail), source: "rust" };
    }
  } catch (error) {
    invokeError = invokeError || error;
  }

  if (invokeError) {
    console.info("get_design_detail not available yet, using mock detail.", invokeError);

    const fallback = MOCK_DESIGNS.find((item) => item.id === normalizedId) || null;
    if (!fallback) {
      return { item: null, source: "mock", error: String(invokeError) };
    }

    const matchedHoop = MOCK_HOOPS.find((hoop) => hoop.name === fallback.hoop) || null;

    return {
      item: mapDesignDetailFromWire({
        id: fallback.id,
        filename: fallback.filename,
        filepath: `C:/mock/${fallback.filename}`,
        image_type: null,
        image_data_url: null,
        width_mm: null,
        height_mm: null,
        stitch_count: null,
        color_count: null,
        color_change_count: null,
        designer: fallback.designer,
        designer_id: null,
        source: fallback.source,
        source_id: null,
        hoop: fallback.hoop,
        hoop_id: matchedHoop ? matchedHoop.id : null,
        is_stitched: Boolean(fallback.is_stitched),
        image_tags_verified: Boolean(fallback.image_tags_verified),
        stitching_tags_verified: Boolean(fallback.stitching_tags_verified),
        hoops: MOCK_HOOPS,
        notes: "Mock detail while Rust route migration continues.",
        rating: null,
        tagging_tier: null,
        date_added: null,
        tags: [],
        projects: [],
        available_projects: [],
        all_tags: [],
        designers: [],
        sources: [],
      }),
      source: "mock",
      error: String(invokeError),
    };
  }

  const fallback = MOCK_DESIGNS.find((item) => item.id === normalizedId) || null;
  if (!fallback) {
    return { item: null, source: "mock" };
  }

  const matchedHoop = MOCK_HOOPS.find((hoop) => hoop.name === fallback.hoop) || null;

  return {
    item: mapDesignDetailFromWire({
      id: fallback.id,
      filename: fallback.filename,
      filepath: `C:/mock/${fallback.filename}`,
      image_type: null,
      image_data_url: null,
      width_mm: null,
      height_mm: null,
      stitch_count: null,
      color_count: null,
      color_change_count: null,
      designer: fallback.designer,
      designer_id: null,
      source: fallback.source,
      source_id: null,
      hoop: fallback.hoop,
      hoop_id: matchedHoop ? matchedHoop.id : null,
      is_stitched: Boolean(fallback.is_stitched),
      image_tags_verified: Boolean(fallback.image_tags_verified),
      stitching_tags_verified: Boolean(fallback.stitching_tags_verified),
      hoops: MOCK_HOOPS,
      notes: "Mock detail while Rust route migration continues.",
      rating: null,
      tagging_tier: null,
      date_added: null,
      tags: [],
      projects: [],
      available_projects: [],
      all_tags: [],
      designers: [],
      sources: [],
    }),
    source: "mock",
  };
}

/**
 * @param {number | string} designId
 */
export async function getDesignImageDataUrl(
  designId: number | string
): Promise<AdapterItemResponse<DesignImageData>> {
  const normalizedId = Number(designId);
  if (!Number.isFinite(normalizedId) || normalizedId <= 0) {
    return { item: null, source: "mock" };
  }

  try {
    const image = await invokeLoose<DesignImageData | null>("get_design_image_data_url", {
      designId: normalizedId,
    });
    if (image && typeof image === "object") {
      return { item: image, source: "rust" };
    }
  } catch (error) {
    console.info("get_design_image_data_url not available yet, using mock image.", error);
  }

  return { item: null, source: "mock" };
}

/**
 * @param {number | string} designId
 * @param {Record<string, any>} request
 */
export async function updateDesignMetadata(
  designId: number | string,
  request: UpdateDesignMetadataRequest
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose<DesignCommandResult>("update_design_metadata", {
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

/**
 * @param {number | string} designId
 * @param {number | null} rating
 */
export async function setDesignRating(
  designId: number | string,
  rating: number | null
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose<DesignCommandResult>("set_design_rating", {
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

/**
 * @param {number | string} designId
 * @param {boolean} isStitched
 */
export async function setDesignStitched(
  designId: number | string,
  isStitched: boolean
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose<DesignCommandResult>("set_design_stitched", {
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

/**
 * Update the image and/or stitching verification flags independently.
 * Each `null` / absent field is left untouched in SQLite.
 * @param {number | string} designId
 * @param {{ imageTagsVerified?: boolean | null, stitchingTagsVerified?: boolean | null }} patch
 */
export async function setDesignVerification(
  designId: number | string,
  patch: { imageTagsVerified?: boolean | null; stitchingTagsVerified?: boolean | null }
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose<DesignCommandResult>("set_design_verification", {
      designId: normalizedId,
      request: {
        image_tags_verified: patch.imageTagsVerified ?? null,
        stitching_tags_verified: patch.stitchingTagsVerified ?? null,
      },
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

/**
 * Replace a single design's full tag set.
 *
 * The optional verification object lets callers explicitly drive the
 * image/stitching verification flags. Absent (`null`) fields tell the backend
 * to leave that flag untouched, so an unaffected category never has its prior
 * verified status cleared.
 *
 * @param {number | string} designId
 * @param {Array<number | string>} tagIds
 * @param {{ imageTagsVerified?: boolean | null, stitchingTagsVerified?: boolean | null }} [verification]
 */
export async function setDesignTags(
  designId: number | string,
  tagIds: Array<number | string>,
  verification?: { imageTagsVerified?: boolean | null; stitchingTagsVerified?: boolean | null }
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedTagIds = Array.isArray(tagIds)
    ? Array.from(
        new Set(tagIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0))
      )
    : [];

  try {
    const result = await invokeLoose<DesignCommandResult>("set_design_tags", {
      designId: normalizedId,
      request: {
        tag_ids: normalizedTagIds,
        image_tags_verified: verification?.imageTagsVerified ?? null,
        stitching_tags_verified: verification?.stitchingTagsVerified ?? null,
      },
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

/**
 * @param {number | string} designId
 * @param {number | string} tagId
 */
export async function removeDesignTag(
  designId: number | string,
  tagId: number | string
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedTagId = Number(tagId);

  try {
    const result = await invokeLoose<DesignCommandResult>("remove_design_tag", {
      designId: normalizedId,
      tagId: normalizedTagId,
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Tag removed from design."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not remove tag: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} designId
 * @param {number | string} projectId
 */
export async function addDesignToProject(
  designId: number | string,
  projectId: number | string
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<DesignCommandResult>("add_design_to_project", {
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

/**
 * @param {number | string} designId
 * @param {number | string} projectId
 */
export async function removeDesignFromProject(
  designId: number | string,
  projectId: number | string
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<DesignCommandResult>("remove_design_from_project", {
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

/**
 * Bulk delete designs from the catalogue.
 * When deleteFiles is true, source design files are moved to the OS trash/recycle bin.
 *
 * @param {Array<number | string>} designIds - Design IDs to delete (max 50).
 * @param {boolean} [deleteFiles=false] - Whether to also move source files to the recycle bin.
 * @returns {Promise<{
 *   source: string,
 *   persisted: boolean,
 *   deleted_count: number,
 *   files_trashed: number,
 *   errors: string[]
 * }>}
 */
export async function bulkDeleteDesigns(designIds: Array<number | string>, deleteFiles = false) {
  const ids = Array.isArray(designIds)
    ? Array.from(
        new Set(designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0))
      )
    : [];

  if (ids.length === 0) {
    return {
      source: "mock",
      persisted: false,
      deleted_count: 0,
      files_trashed: 0,
      errors: [],
    };
  }

  try {
    const result = await invokeLoose("bulk_delete_designs", {
      request: {
        design_ids: ids,
        delete_files: Boolean(deleteFiles),
      },
    });
    return {
      source: "rust",
      persisted: true,
      deleted_count: Number(result?.deleted_count ?? 0),
      files_trashed: Number(result?.files_trashed ?? 0),
      errors: Array.isArray(result?.errors) ? result.errors.map(String) : [],
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      deleted_count: 0,
      files_trashed: 0,
      errors: [String(error)],
    };
  }
}

/**
 * @param {number | string} designId
 */
export async function openDesignInEditor(designId: number | string) {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose("open_design_in_editor", {
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

/**
 * @param {number | string} designId
 */
export async function openDesignInExplorer(designId: number | string) {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose("open_design_in_explorer", {
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

/**
 * Generate a 2D or 3D preview for a design.
 * @param {number | string} designId
 * @param {boolean} [preview3d=true] - Whether to generate a 3D preview (true) or 2D (false).
 */
export async function renderDesign3dPreview(designId: number | string, preview3d = true) {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose("render_design_3d_preview", {
      designId: normalizedId,
      request: { preview_3d: Boolean(preview3d) },
    });
    return {
      source: "rust",
      persisted: true,
      result,
      message: String(result?.message || "Preview rendered."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      result: null,
      message: `Could not render preview: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Re-read the design file from disk and recalculate its technical metadata
 * (dimensions, stitch count, colour counts, recommended hoop).
 *
 * @param {number | string} designId
 */
export async function reparseDesignFile(
  designId: number | string
): Promise<AdapterReparseDesignResponse> {
  const normalizedId = Number(designId);

  try {
    const result = await invokeLoose<ReparseDesignResultWire>("reparse_design_file", {
      designId: normalizedId,
    });
    return {
      source: "rust",
      persisted: true,
      result: result && typeof result === "object" ? mapReparseDesignFromWire(result) : null,
      message: String(result?.message || "Design metadata recalculated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      result: null,
      message: `Could not recalculate metadata: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Try import preview using existing Rust bulk import command.
 * Falls back to a mock preview shape if command wiring is incomplete.
 * @param {string | string[]} rootPaths
 */
export async function previewImportFromRoots(
  rootPaths: string | string[]
): Promise<AdapterImportPreviewResponse> {
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
        scanned_files: [],
        resolved_assignments: [],
        missing_root: false,
        no_supported_files: false,
        invalid_root: true,
      },
      message: "Enter at least one folder path to preview import.",
    };
  }

  try {
    const preview = await invokeLoose<Partial<BulkImportPreview>>("preview_bulk_import", {
      request: {
        root_path: normalizedRoots[0],
        root_paths: normalizedRoots,
        fallback_designer_id: null,
        fallback_source_id: null,
      },
    });

    return {
      source: "rust",
      preview: {
        discovered_count: Number(preview?.discovered_count ?? 0),
        selected_count: Number(preview?.selected_count ?? 0),
        folder_count: Number(preview?.folder_count ?? normalizedRoots.length),
        scanned_files: Array.isArray(preview?.scanned_files) ? preview.scanned_files : [],
        resolved_assignments: Array.isArray(preview?.resolved_assignments)
          ? preview.resolved_assignments
          : [],
        missing_root: Boolean(preview?.missing_root),
        no_supported_files: Boolean(preview?.no_supported_files),
        invalid_root: Boolean(preview?.invalid_root),
      },
      message: "Preview loaded from Rust command.",
    };
  } catch (error) {
    console.info("preview_bulk_import unavailable or failed, using mock preview.", error);
    return {
      source: "mock",
      preview: {
        discovered_count: 0,
        selected_count: 0,
        folder_count: normalizedRoots.length,
        scanned_files: [],
        resolved_assignments: [],
        missing_root: false,
        no_supported_files: true,
        invalid_root: false,
      },
      message: `Rust preview command failed: ${error}`,
    };
  }
}

/**
 * @param {string} rootPath
 */
export async function previewImportFromRoot(
  rootPath: string
): Promise<AdapterImportPreviewResponse> {
  const normalizedRoot = String(rootPath || "").trim();
  return previewImportFromRoots(normalizedRoot ? [normalizedRoot] : []);
}

/**
 * Open native folder picker for import root selection.
 * @param {string} [startDir]
 */
export async function browseImportFolder(
  startDir = ""
): Promise<AdapterBrowseImportFolderResponse> {
  try {
    const result = await invokeLoose<BrowseImportFolderResult>("browse_import_folder", {
      request: {
        start_dir: String(startDir || "").trim() || null,
        allow_multi: true,
      },
    });

    return {
      source: "rust",
      path: String(result?.path || ""),
      paths: Array.isArray(result?.paths)
        ? result.paths.map((item) => String(item || "")).filter(Boolean)
        : [],
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
 * @param {Record<string, any> | null} confirmWire
 */
export async function precheckImportWire(
  confirmWire: Record<string, unknown> | null
): Promise<AdapterImportPrecheckResponse> {
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
    const precheck = await invokeLoose<ImportPrecheckResult>("precheck_bulk_import_wire", {
      confirmWire: wire,
    });

    return {
      source: "rust",
      precheck,
      message: "Precheck loaded from Rust command.",
    };
  } catch (error) {
    console.info("precheck_bulk_import_wire unavailable or failed.", error);
    throw new Error(`Precheck failed: ${error}`);
  }
}

/**
 * Execute Step 3 precheck action in Rust backend.
 * @param {object} options
 * @param {string} options.contextToken
 * @param {string} options.action
 * @param {boolean} [options.confirmSkipHoops]
 */
export async function runPrecheckAction({
  contextToken,
  action,
  confirmSkipHoops = false,
}: {
  contextToken: string;
  action: string;
  confirmSkipHoops?: boolean;
}): Promise<AdapterImportPrecheckActionResponse> {
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
    const actionResult = await invokeLoose<ImportPrecheckActionResult>(
      "precheck_bulk_import_action_wire",
      {
        request: {
          context_token: normalizedToken,
          action: normalizedAction,
          confirm_skip_hoops: Boolean(confirmSkipHoops),
        },
      }
    );

    return {
      source: "rust",
      actionResult,
      message: "",
    };
  } catch (error) {
    console.info(
      "precheck_bulk_import_action_wire unavailable or failed, using mock action result.",
      error
    );
    const isCancel = normalizedAction === "cancel";

    return {
      source: "mock",
      actionResult: {
        action: normalizedAction,
        context_token_present: !isCancel,
        consumed_context: isCancel,
        requires_skip_hoops_confirmation: false,
        next_route: isCancel ? "/import/" : null,
        confirm_result: null,
      },
      message: `Import action failed: ${error}`,
    };
  }
}

/**
 * Request stop for the currently running bulk import.
 */
export async function requestStopBulkImport(): Promise<AdapterStopBulkImportResponse> {
  try {
    const result = await invokeLoose<{ stop_requested?: boolean }>("request_stop_bulk_import");
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
 * @param {Array<number | string>} designIds
 */
export async function bulkVerifyDesigns(designIds: Array<number | string>) {
  const normalizedIds =
    designIds && typeof designIds[Symbol.iterator] === "function"
      ? Array.from(designIds)
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
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
    const result = await invokeLoose("bulk_verify_designs", { designIds: normalizedIds });
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

export async function getBrowseProjects(): Promise<AdapterListResponse<ProjectListItem>> {
  try {
    const projects = await invokeLoose<ProjectListItem[]>("get_projects_for_browse");
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

export async function getProjectsList(
  requestTimeoutMs = 15000
): Promise<AdapterProjectListResponse> {
  const REQUEST_TIMEOUT_MS = requestTimeoutMs;

  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => {
      reject(new Error(`Timed out loading projects after ${REQUEST_TIMEOUT_MS / 1000}s.`));
    }, REQUEST_TIMEOUT_MS);
  });

  try {
    const projects = await Promise.race([
      invokeLoose<ProjectSummary[]>("get_projects_list"),
      timeoutPromise,
    ]);
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

/**
 * @param {string} name
 * @param {string} description
 */
export async function createProject(
  name: string,
  description: string
): Promise<AdapterProjectMutationResponse> {
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invokeLoose<ProjectMutationResult>("create_project", { request: payload });
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

/**
 * @param {number | string} projectId
 */
export async function getProjectDetail(
  projectId: number | string
): Promise<AdapterProjectDetailResponse> {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const detail = await invokeLoose<ProjectDetailView>("get_project_detail", {
      projectId: normalizedProjectId,
    });
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

/**
 * @param {number | string} projectId
 * @param {string} name
 * @param {string} description
 */
export async function updateProject(
  projectId: number | string,
  name: string,
  description: string
): Promise<AdapterProjectMutationResponse> {
  const normalizedProjectId = Number(projectId);
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invokeLoose<ProjectMutationResult>("update_project", {
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

/**
 * @param {number | string} projectId
 */
export async function deleteProject(
  projectId: number | string
): Promise<AdapterProjectMutationResponse> {
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<ProjectMutationResult>("delete_project", {
      projectId: normalizedProjectId,
    });
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

/**
 * @param {number | string} projectId
 * @param {number | string} designId
 */
export async function removeDesignFromProjectDetail(
  projectId: number | string,
  designId: number | string
): Promise<AdapterProjectDesignMutationResponse> {
  const normalizedProjectId = Number(projectId);
  const normalizedDesignId = Number(designId);

  try {
    const result = await invokeLoose<RemoveProjectDesignResult>(
      "remove_design_from_project_detail",
      {
        projectId: normalizedProjectId,
        designId: normalizedDesignId,
      }
    );
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

/**
 * @param {number | string} projectId
 */
export async function getProjectPrintView(
  projectId: number | string
): Promise<AdapterProjectDetailResponse> {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const view = await invokeLoose<ProjectDetailView>("get_project_print_view", {
      projectId: normalizedProjectId,
    });
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
 * @param {number | string} projectId
 * @param {Array<number | string>} designIds
 */
export async function bulkAddDesignsToProject(
  projectId: number | string,
  designIds: Array<number | string>
) {
  const normalizedProjectId = Number(projectId);
  const normalizedIds =
    designIds && typeof designIds[Symbol.iterator] === "function"
      ? Array.from(designIds)
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
      : [];

  if (
    !Number.isFinite(normalizedProjectId) ||
    normalizedProjectId <= 0 ||
    normalizedIds.length === 0
  ) {
    return {
      source: "mock",
      project_id: normalizedProjectId,
      requested_count: normalizedIds.length,
      added_count: 0,
      persisted: false,
    };
  }

  try {
    const result = await invokeLoose("bulk_add_designs_to_project", {
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
export async function getBrowseTags(): Promise<AdapterListResponse<BrowseTagOption>> {
  try {
    const tags = await invokeLoose<BrowseTagOption[]>("get_tags_for_browse");
    if (Array.isArray(tags)) {
      return {
        items: tags.map((tag) => ({
          id: Number(tag?.id),
          description: String(tag?.description || ""),
          tag_group: tag?.tag_group == null ? null : String(tag.tag_group),
          is_system: tag?.is_system == null ? false : Boolean(tag.is_system),
        })),
        source: "rust",
      };
    }
    return {
      items: [],
      source: "rust",
      error: "get_tags_for_browse returned an unexpected payload.",
    };
  } catch (error) {
    return { items: [], source: "mock", error: String(error) };
  }
}

/**
 * Apply an explicit add/remove tag diff across a batch of designs.
 *
 * Tags left untouched (indeterminate / mixed in the UI) are simply excluded
 * from both lists, so the backend never touches them. This prevents the
 * previous blanket "replace all tags" behaviour from accidentally removing
 * tags that existed on only some selected designs.
 *
 * @param {Array<number | string>} designIds
 * @param {Array<number | string>} tagsToAdd - Tags to add to ALL selected designs.
 * @param {Array<number | string>} [tagsToRemove=[]] - Tags to remove from ALL selected designs.
 * @param {boolean} [clearAllTags=false] - Clear all tags from the selected designs first.
 * @param {{ imageTagsVerified?: boolean | null, stitchingTagsVerified?: boolean | null }} [verification]
 *   Optional per-category verification overrides. `null` / absent means "leave
 *   that category's existing verification flag untouched" — the backend will
 *   NOT clear a prior verified status for a category with no change.
 */
export async function bulkSetTagsForDesigns(
  designIds: Array<number | string>,
  tagsToAdd: Array<number | string>,
  tagsToRemove: Array<number | string> = [],
  clearAllTags = false,
  verification?: { imageTagsVerified?: boolean | null; stitchingTagsVerified?: boolean | null }
) {
  const normalizedDesignIds =
    designIds && typeof designIds[Symbol.iterator] === "function"
      ? Array.from(designIds)
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
      : [];
  const normalizedAddIds =
    tagsToAdd && typeof tagsToAdd[Symbol.iterator] === "function"
      ? Array.from(
          new Set(
            Array.from(tagsToAdd)
              .map((id) => Number(id))
              .filter((id) => Number.isFinite(id) && id > 0)
          )
        )
      : [];
  const normalizedRemoveIds =
    tagsToRemove && typeof tagsToRemove[Symbol.iterator] === "function"
      ? Array.from(
          new Set(
            Array.from(tagsToRemove)
              .map((id) => Number(id))
              .filter((id) => Number.isFinite(id) && id > 0)
          )
        )
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
    const result = await invokeLoose("bulk_set_tags_for_designs", {
      designIds: normalizedDesignIds,
      request: {
        tags_to_add: normalizedAddIds,
        tags_to_remove: normalizedRemoveIds,
        clear_all_tags: Boolean(clearAllTags),
        image_tags_verified: verification?.imageTagsVerified ?? null,
        stitching_tags_verified: verification?.stitchingTagsVerified ?? null,
      },
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
 * @param {Array<number | string>} designIds
 */
export async function getBrowseDesignPreviews(
  designIds: Array<number | string>
): Promise<AdapterListResponse<BrowseDesignPreview>> {
  const normalizedIds = Array.isArray(designIds)
    ? Array.from(
        new Set(designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0))
      )
    : [];

  if (normalizedIds.length === 0) {
    return { items: [], source: "mock" };
  }

  try {
    const previews = await invokeLoose<BrowseDesignPreview[]>("get_design_previews_for_browse", {
      designIds: normalizedIds,
    });
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
    const docs = await invokeLoose("get_about_documents");
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
        description:
          "Explains what data is stored locally and what optional AI features may send externally.",
        filename: "templates/info/PRIVACY.html",
        available: false,
      },
      {
        slug: "security",
        title: "Security",
        description: "Guidance on secrets, API keys, portable deployments, and safe usage.",
        filename: "templates/info/security.html",
        available: false,
      },
      {
        slug: "ai-tagging",
        title: "AI Tagging Guide",
        description:
          "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
        filename: "templates/info/ai_tagging.html",
        available: false,
      },
      {
        slug: "data-storage",
        title: "Data Storage & External Drives Guide",
        description:
          "How Embroidery Catalogue stores your designs and database, and how to choose external storage.",
        filename: "docs/User-Facing-Guidance/DATA_STORAGE_GUIDE.md",
        available: false,
      },
    ],
  };
}

/**
 * @param {string} slug
 */
export async function getAboutDocument(slug: string) {
  const normalizedSlug = String(slug || "")
    .trim()
    .toLowerCase();
  if (!normalizedSlug) {
    return { item: null, source: "mock", error: "Document not found." };
  }

  try {
    const item = await invokeLoose("get_about_document", { slug: normalizedSlug });
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
export async function getSettingsViewModel(): Promise<AdapterSettingsViewModelResponse> {
  try {
    const model = await invokeLoose<SettingsViewModel>("get_settings_view_model");
    if (model && typeof model === "object") {
      return { model, source: "rust" };
    }
  } catch (error) {
    console.info("get_settings_view_model unavailable, using local fallback.", error);
  }

  return {
    source: "mock",
    model: {
      preview_3d_profile: "balanced",
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
      database_path: "",
      log_folder: "",
      app_mode: "development",
      ai_tagging_help_url: "#/help",
      db_idle_check_interval_secs: "1800",
    },
  };
}

/**
 * Save settings via Rust backend.
 * @param {Record<string, any>} request
 */
export async function saveSettings(
  request: SaveSettingsRequest
): Promise<AdapterSaveSettingsResponse> {
  try {
    const result = await invokeLoose<{ saved: boolean; message: string }>(
      "save_settings_view_model",
      { request }
    );
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
 * @param {string} path
 */
export async function saveImportLastBrowseFolder(path: string) {
  try {
    const result = await invokeLoose("save_import_last_browse_folder", {
      path: String(path || ""),
    });
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
 * @param {string} startDir
 */
export async function browseSettingsDataRoot(
  startDir: string
): Promise<AdapterBrowseDataRootResponse> {
  try {
    const result = await invokeLoose<{ path?: string | null; error?: string | null }>(
      "browse_settings_data_root",
      { startDir: startDir }
    );
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

/**
 * Fetch the database recovery status (Uninitialized / Connected / Missing).
 * The app shell blocks the main UI until "missing" is resolved.
 */
export async function getDatabaseStatus(): Promise<{
  source: string;
  status: DatabaseStatus | null;
  error?: string;
}> {
  try {
    const status = await invokeLoose<DatabaseStatus>("get_database_status");
    return {
      source: "rust",
      status: status && typeof status === "object" ? status : null,
    };
  } catch (error) {
    console.info("get_database_status failed.", error);
    return { source: "mock", status: null, error: String(error) };
  }
}

/**
 * Scan other drive letters for the same relative catalogue path (e.g. D: moved to E:).
 */
export async function detectRelocatedDataRoot(configuredDataRoot: string): Promise<{
  source: string;
  detected: DetectedDataRoot | null;
  error?: string;
}> {
  try {
    const result = await invokeLoose<DetectedDataRoot>("detect_relocated_data_root", {
      configuredDataRoot: String(configuredDataRoot || ""),
    });
    return {
      source: "rust",
      detected: result && typeof result === "object" ? result : null,
    };
  } catch (error) {
    console.info("detect_relocated_data_root failed.", error);
    return { source: "mock", detected: null, error: String(error) };
  }
}

/**
 * Validate a candidate data root contains a real catalogue database.
 */
export async function validateDatabasePath(candidateDataRoot: string): Promise<{
  source: string;
  validation: DatabaseValidation | null;
  error?: string;
}> {
  try {
    const result = await invokeLoose<DatabaseValidation>("validate_database_path", {
      candidateDataRoot: String(candidateDataRoot || ""),
    });
    return {
      source: "rust",
      validation: result && typeof result === "object" ? result : null,
    };
  } catch (error) {
    console.info("validate_database_path failed.", error);
    return { source: "mock", validation: null, error: String(error) };
  }
}

/**
 * Create a fresh empty catalogue at `dataRoot` (guarded). Builds the standard
 * layout (MachineEmbroideryDesigns, logs, Database) and writes the seed DB.
 * `overwrite` is only true after the user explicitly confirms.
 */
export async function seedDatabaseToDataRoot(
  dataRoot: string,
  overwrite = false
): Promise<{
  source: string;
  persisted: boolean;
  error?: string;
}> {
  try {
    await invokeLoose("seed_database_to_data_root", {
      dataRoot: String(dataRoot || ""),
      overwrite: Boolean(overwrite),
    });
    return { source: "rust", persisted: true };
  } catch (error) {
    console.info("seed_database_to_data_root failed.", error);
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Fetch the currently configured Google API key (optional, for AI tagging).
 *
 * @returns {Promise<AdapterGoogleApiKeyResponse>}
 */
export async function getGoogleApiKey(): Promise<AdapterGoogleApiKeyResponse> {
  try {
    const result = await invokeLoose<string | null>("get_google_api_key");
    return {
      source: "rust",
      key: result ? String(result) : "",
    };
  } catch (error) {
    console.info("get_google_api_key failed.", error);
    return { source: "mock", key: "", error: String(error) };
  }
}

/**
 * Persist the user's Google API key via the Rust `.env` writer.
 * Pass an empty string to clear the stored key.
 *
 * @param {string} apiKey
 * @returns {Promise<{source: string; persisted: boolean; error?: string}>}
 */
export async function setGoogleApiKey(apiKey: string): Promise<{
  source: string;
  persisted: boolean;
  error?: string;
}> {
  const normalized = String(apiKey || "").trim();
  try {
    await invokeLoose("set_google_api_key", { apiKey: normalized });
    return { source: "rust", persisted: true };
  } catch (error) {
    console.info("set_google_api_key failed.", error);
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function getTaggingActionsViewModel(): Promise<AdapterTaggingActionsViewModelResponse> {
  try {
    const model = await invokeLoose<TaggingActionsViewModel>("get_tagging_actions_view_model");
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

/**
 * Translate the flat view-model from the Tagging Actions screen into the
 * nested `actions` descriptor the Rust `backfill::UnifiedBackfillRequest`
 * expects. Tagging, image generation and colour counts are independent
 * activities: each section is only included when its checkbox was enabled, so
 * an "image generation only" run never triggers tagging and vice versa.
 *
 * @param {UnifiedBackfillRequest} request
 */
function buildUnifiedBackfillWireRequest(
  request: UnifiedBackfillRequest
): UnifiedBackfillWireRequest {
  const runTagging = Boolean(request.run_tier2) || Boolean(request.run_tier3);
  const actionMode = request.action_mode === "tag_all" ? "retag_all" : "tag_untagged";

  const tiers = [1];
  if (request.run_tier2) tiers.push(2);
  if (request.run_tier3) tiers.push(3);

  return {
    actions: {
      tagging: runTagging ? { action: actionMode, tiers, enabled: true } : null,
      stitching: null,
      images: request.run_images ? { enabled: true, redo: Boolean(request.image_redo) } : null,
      color_counts: request.run_color_counts ? { enabled: true } : null,
      hoop_dimensions: request.run_hoop_dimensions ? { enabled: true } : null,
      fingerprinting: null,
    },
    batch_size: Number(request.batch_size ?? 100),
    commit_every: Number(request.commit_every ?? 100),
    workers: Number(request.workers ?? 4),
  };
}

/**
 * Run the unified backfill. The view-model is translated to the nested wire
 * shape expected by the Rust `run_unified_backfill` Tauri command.
 *
 * @param {UnifiedBackfillRequest} request
 */
export async function runUnifiedBackfill(
  request: UnifiedBackfillRequest
): Promise<UnifiedBackfillResult> {
  try {
    const wireRequest = buildUnifiedBackfillWireRequest(request);
    const result = await invokeLoose<UnifiedBackfillResult>("run_unified_backfill", {
      request: wireRequest,
    });
    return {
      source: "rust",
      processed: Number(result?.processed ?? 0),
      errors: Number(result?.errors ?? 0),
      stopped: Boolean(result?.stopped),
      actions: Array.isArray(result?.actions) ? result.actions.map(String) : [],
      commit_every: Number(result?.commit_every ?? 100),
      batch_size: Number(result?.batch_size ?? 100),
      workers: Number(result?.workers ?? 4),
      stitching_tag_count_before: Number(result?.stitching_tag_count_before ?? 0),
      stitching_tag_count_after: Number(result?.stitching_tag_count_after ?? 0),
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

export async function stopUnifiedBackfill(): Promise<AdapterStopUnifiedBackfillResponse> {
  try {
    const result = await invokeLoose<{ status?: string }>("stop_unified_backfill");
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

/**
 * @param {number} [limit]
 */
export async function getBackfillLogEntries(
  limit = 20
): Promise<AdapterBackfillLogEntriesResponse> {
  try {
    const entries = await invokeLoose<Array<{ level?: string; message?: string }>>(
      "get_backfill_log_entries",
      { limit: Number(limit) }
    );
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

/**
 * @param {RunStitchingBackfillOptions} [options]
 */
export async function runStitchingBackfill({
  clear_stitching_mode = "none",
  batch_size = 100,
}: RunStitchingBackfillOptions = {}): Promise<UnifiedBackfillResult> {
  try {
    const result = await invokeLoose<UnifiedBackfillResult>("run_stitching_backfill", {
      clearStitchingMode: String(clear_stitching_mode),
      batchSize: Number(batch_size),
    });
    return {
      source: "rust",
      processed: Number(result?.processed ?? 0),
      errors: Number(result?.errors ?? 0),
      stopped: Boolean(result?.stopped),
      actions: Array.isArray(result?.actions) ? result.actions.map(String) : [],
      stitching_tag_count_before: Number(result?.stitching_tag_count_before ?? 0),
      stitching_tag_count_after: Number(result?.stitching_tag_count_after ?? 0),
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

export async function getBackupViewModel(): Promise<AdapterBackupViewModelResponse> {
  try {
    const model = await invokeLoose<BackupViewModel>("get_backup_view_model");
    return {
      source: "rust",
      model: {
        db_destination: String(model?.db_destination || ""),
        designs_destination: String(model?.designs_destination || ""),
        db_source_path: String(model?.db_source_path || ""),
        designs_source_path: String(model?.designs_source_path || ""),
        db_last_backup_at: String(model?.db_last_backup_at || ""),
        designs_last_backup_at: String(model?.designs_last_backup_at || ""),
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
        db_last_backup_at: "",
        designs_last_backup_at: "",
      },
      error: String(error),
    };
  }
}

/**
 * @param {{ dbDestination: string, designsDestination: string }} options
 */
export async function saveBackupSettings({
  dbDestination,
  designsDestination,
}: SaveBackupSettingsRequest): Promise<AdapterSaveBackupSettingsResponse> {
  try {
    const result = await invokeLoose<{
      saved?: boolean;
      message?: string;
      db_destination?: string;
      designs_destination?: string;
    }>("save_backup_settings", {
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

/**
 * @param {string} [startDir]
 */
export async function browseBackupFolder(
  startDir = ""
): Promise<AdapterBrowseBackupFolderResponse> {
  try {
    const result = await invokeLoose<{ path?: string | null; error?: string | null }>(
      "browse_backup_folder",
      {
        startDir: String(startDir || "") || null,
      }
    );

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

export async function runDatabaseBackup(): Promise<{ source: string } & DatabaseBackupResult> {
  try {
    const result = await invokeLoose<DatabaseBackupResult>("run_database_backup");
    return {
      source: "rust",
      success: Boolean(result?.success),
      backup_path: result?.backup_path ? String(result.backup_path) : "",
      size_bytes: Number(result?.size_bytes ?? 0),
      completed_at: String(result?.completed_at || ""),
      error: result?.error ? String(result.error) : "",
      cancelled: Boolean(result?.cancelled),
    };
  } catch (error) {
    return {
      source: "mock",
      success: false,
      backup_path: "",
      size_bytes: 0,
      completed_at: "",
      error: String(error),
      cancelled: false,
    };
  }
}

/**
 * Raise the cooperative backup cancellation flag on the Rust backend.
 *
 * The running backup command observes the flag at its next safe boundary:
 * a partially created database backup file is removed, while already-copied
 * design files are left in the destination folder.
 */
export async function requestCancelBackup(): Promise<
  { source: string } & CancelBackupResult & { error?: string }
> {
  try {
    const result = await invokeLoose<CancelBackupResult>("request_cancel_backup");
    return {
      source: "rust",
      cancel_requested: Boolean(result?.cancel_requested),
    };
  } catch (error) {
    return {
      source: "mock",
      cancel_requested: false,
      error: String(error),
    };
  }
}

export async function runDesignsBackup(): Promise<{ source: string } & DesignsBackupResult> {
  try {
    const result = await invokeLoose<DesignsBackupResult>("run_designs_backup");
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
      cancelled: Boolean(result?.cancelled),
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
      cancelled: false,
    };
  }
}

export async function runBothBackups(): Promise<AdapterRunBothBackupsResponse> {
  try {
    const result = await invokeLoose<{
      database?: DatabaseBackupResult | null;
      designs?: DesignsBackupResult | null;
    }>("run_both_backups");
    const database = result?.database
      ? { ...result.database, cancelled: Boolean(result.database.cancelled) }
      : null;
    const designs = result?.designs
      ? { ...result.designs, cancelled: Boolean(result.designs.cancelled) }
      : null;
    return {
      source: "rust",
      database,
      designs,
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

export async function scanOrphans(): Promise<AdapterScanOrphansResponse> {
  try {
    const result = await invokeLoose<{ checked?: number; found?: number }>("scan_orphans");
    return {
      source: "rust",
      checked: Number(result?.checked ?? 0),
      found: Number(result?.found ?? 0),
    };
  } catch (error) {
    return {
      source: "mock",
      checked: 0,
      found: 0,
      error: String(error),
    };
  }
}

/**
 * Open a file picker for a database backup snapshot, defaulting to the
 * configured Database backup folder and restricted to `.db` files.
 * @param {string} [startDir]
 */
export async function browseRestoreFile(
  startDir = ""
): Promise<BrowseRestoreFileResponse> {
  try {
    const result = await invokeLoose<{ path?: string | null; error?: string | null }>(
      "browse_restore_file",
      {
        startDir: String(startDir || "") || null,
      }
    );
    return {
      source: "rust",
      path: result?.path ? String(result.path) : null,
      error: result?.error ? String(result.error) : null,
    };
  } catch (error) {
    return {
      source: "mock",
      path: null,
      error: `File picker unavailable: ${error}`,
    };
  }
}

/**
 * Normalize a raw Rust database-restore outcome into the adapter shape.
 * @param {{[k:string]: unknown}} result
 */
function normalizeRestoreDatabase(result) {
  return {
    success: Boolean(result?.success),
    restored_path: String(result?.restored_path || ""),
    rollback_copy_path: result?.rollback_copy_path ? String(result.rollback_copy_path) : null,
    design_count: Number(result?.design_count ?? 0),
    schema_version_hint:
      result?.schema_version_hint === null || result?.schema_version_hint === undefined
        ? null
        : Number(result.schema_version_hint),
    previous_schema_version_hint:
      result?.previous_schema_version_hint === null ||
      result?.previous_schema_version_hint === undefined
        ? null
        : Number(result.previous_schema_version_hint),
    rolled_back: Boolean(result?.rolled_back),
    error: result?.error ? String(result.error) : null,
  };
}

/**
 * Swap the live database for a user-selected backup snapshot, with automatic
 * rollback on verification failure. Invoke keys are camelCase (`dbFile`).
 * @param {string} dbFile
 */
export async function restoreDatabase(
  dbFile: string
): Promise<{ source: string } & RestoreDatabaseResult> {
  try {
    const result = await invokeLoose<LooseRecord>("restore_database", {
      request: { db_file: String(dbFile || "") },
    });
    return { source: "rust", ...normalizeRestoreDatabase(result) };
  } catch (error) {
    return {
      source: "mock",
      success: false,
      restored_path: "",
      rollback_copy_path: null,
      design_count: 0,
      schema_version_hint: null,
      previous_schema_version_hint: null,
      rolled_back: false,
      error: String(error),
    };
  }
}

/**
 * Incremental mirror restore of design files from a backup folder.
 * @param {{ designsSourceDir?: string }} [opts]
 */
export async function restoreDesignsIncremental(
  opts: { designsSourceDir?: string } = {}
): Promise<{ source: string } & RestoreDesignsResult> {
  try {
    const result = await invokeLoose<LooseRecord>("restore_designs_incremental", {
      request: { designs_source_dir: String(opts?.designsSourceDir || "") || null },
    });
    return {
      source: "rust",
      success: Boolean(result?.success),
      scanned: Number(result?.scanned ?? 0),
      copied: Number(result?.copied ?? 0),
      updated: Number(result?.updated ?? 0),
      skipped: Number(result?.skipped ?? 0),
      total_bytes_copied: Number(result?.total_bytes_copied ?? 0),
      error: result?.error ? String(result.error) : null,
    };
  } catch (error) {
    return {
      source: "mock",
      success: false,
      scanned: 0,
      copied: 0,
      updated: 0,
      skipped: 0,
      total_bytes_copied: 0,
      error: String(error),
    };
  }
}

/**
 * Restore the database then sync design files, then reconcile unmatched files.
 * @param {string} dbFile
 * @param {{ designsSourceDir?: string }} [opts]
 */
export async function restoreBoth(
  dbFile: string,
  opts: { designsSourceDir?: string } = {}
): Promise<{ source: string } & RestoreBothResult> {
  try {
    const result = await invokeLoose<{
      database?: Record<string, unknown>;
      designs?: Record<string, unknown>;
      unmatched?: Record<string, unknown>;
    }>("restore_both", {
      request: {
        db_file: String(dbFile || ""),
        designs_source_dir: String(opts?.designsSourceDir || "") || null,
      },
    });
    return {
      source: "rust",
      database: result?.database ? normalizeRestoreDatabase(result.database) : null,
      designs: result?.designs
        ? {
            success: Boolean(result.designs?.success),
            scanned: Number(result.designs?.scanned ?? 0),
            copied: Number(result.designs?.copied ?? 0),
            updated: Number(result.designs?.updated ?? 0),
            skipped: Number(result.designs?.skipped ?? 0),
            total_bytes_copied: Number(result.designs?.total_bytes_copied ?? 0),
            error: result.designs?.error ? String(result.designs.error) : null,
          }
        : null,
      unmatched: result?.unmatched
        ? {
            checked: Number(result.unmatched?.checked ?? 0),
            unmatched: Number(result.unmatched?.unmatched ?? 0),
            sample: Array.isArray(result.unmatched?.sample)
              ? result.unmatched.sample.map((p) => String(p))
              : [],
          }
        : null,
    };
  } catch (error) {
    return {
      source: "mock",
      database: null,
      designs: null,
      unmatched: null,
      error: String(error),
    };
  }
}

/**
 * Post-restore reconciliation scan: files on disk absent from the database.
 */
export async function detectDesignFilesAbsentFromDatabase(): Promise<
  { source: string } & DetectUnmatchedFilesResult
> {
  try {
    const result = await invokeLoose<LooseRecord>("detect_design_files_absent_from_database");
    return {
      source: "rust",
      checked: Number(result?.checked ?? 0),
      unmatched: Number(result?.unmatched ?? 0),
      sample: Array.isArray(result?.sample) ? result.sample.map((p) => String(p)) : [],
    };
  } catch (error) {
    return { source: "mock", checked: 0, unmatched: 0, sample: [] };
  }
}

/**
 * Batch import of unmatched design files as new catalogue records.
 */
export async function importUnmatchedDesignFiles(): Promise<
  { source: string } & ImportUnmatchedFilesResult
> {
  try {
    const result = await invokeLoose<LooseRecord>("import_unmatched_design_files");
    return {
      source: "rust",
      detected: Number(result?.detected ?? 0),
      imported: Number(result?.imported ?? 0),
      failed: Number(result?.failed ?? 0),
      failed_samples: Array.isArray(result?.failed_samples)
        ? result.failed_samples.map((p) => String(p))
        : [],
    };
  } catch (error) {
    return { source: "mock", detected: 0, imported: 0, failed: 0, failed_samples: [] };
  }
}

/**
 * Raise the cooperative restore cancellation flag.
 */
export async function requestCancelRestore(): Promise<{ source: string } & CancelRestoreResult> {
  try {
    const result = await invokeLoose<LooseRecord>("request_cancel_restore");
    return {
      source: "rust",
      cancel_requested: Boolean(result?.cancel_requested),
    };
  } catch (error) {
    return { source: "mock", cancel_requested: false };
  }
}

/**
 * Fetch the catalogue database storage metrics from the Rust backend
 * (file size on disk, page/freelist counts, recoverable freelist size).
 */
export async function getDbStats(): Promise<AdapterDbStatsResponse> {
  try {
    const result = await invokeLoose<DbStats>("get_db_stats");
    return {
      source: "rust",
      stats: {
        file_size_bytes: Number(result?.file_size_bytes ?? 0),
        page_count: Number(result?.page_count ?? 0),
        freelist_count: Number(result?.freelist_count ?? 0),
        page_size: Number(result?.page_size ?? 0),
        free_ratio: Number(result?.free_ratio ?? 0),
        reclaimable_bytes: Number(result?.reclaimable_bytes ?? 0),
      },
    };
  } catch (error) {
    console.info("get_db_stats unavailable, using zero stats.", error);
    return {
      source: "mock",
      stats: {
        file_size_bytes: 0,
        page_count: 0,
        freelist_count: 0,
        page_size: 0,
        free_ratio: 0,
        reclaimable_bytes: 0,
      },
      error: String(error),
    };
  }
}

/**
 * Manually compact & optimise the catalogue database (full VACUUM +
 * PRAGMA optimize). Errors are returned in the adapter envelope.
 */
export async function compactDatabase(): Promise<AdapterCompactResponse> {
  try {
    const result = await invokeLoose<CompactResult>("compact_database");
    return {
      source: "rust",
      result: {
        file_size_before: Number(result?.file_size_before ?? 0),
        file_size_after: Number(result?.file_size_after ?? 0),
        pages_reclaimed: Number(result?.pages_reclaimed ?? 0),
        duration_ms: Number(result?.duration_ms ?? 0),
      },
      message: "Database compacted successfully.",
    };
  } catch (error) {
    return {
      source: "mock",
      result: null,
      message: String(error),
      error: String(error),
    };
  }
}

/**
 * @param {{ page?: number, pageSize?: number }} [options]
 */
export async function getOrphansPage({
  page = 1,
  pageSize = 100,
}: { page?: number; pageSize?: number } = {}): Promise<AdapterOrphansPageResponse> {
  const normalizedPage = Number.isFinite(Number(page)) && Number(page) > 0 ? Number(page) : 1;
  const normalizedPageSize =
    Number.isFinite(Number(pageSize)) && Number(pageSize) > 0 ? Number(pageSize) : 1;

  try {
    const result = await invokeLoose<AdapterOrphansPageResponse>("get_orphans_page", {
      request: {
        page: normalizedPage,
        page_size: normalizedPageSize,
      },
    });

    return {
      source: "rust",
      page: Number(result?.page ?? normalizedPage),
      page_size: Number(result?.page_size ?? normalizedPageSize),
      total: Number(result?.total ?? 0),
      total_pages: Number(result?.total_pages ?? 1),
      items: Array.isArray(result?.items)
        ? result.items.map((item) => ({
            id: Number(item?.id),
            filename: String(item?.filename || ""),
            filepath: String(item?.filepath || ""),
            designer: String(item?.designer || ""),
            date_added: item?.date_added == null ? null : String(item.date_added),
          }))
        : [],
    };
  } catch (error) {
    return {
      source: "mock",
      page: normalizedPage,
      page_size: normalizedPageSize,
      total: 0,
      total_pages: 1,
      items: [],
      error: String(error),
    };
  }
}

/**
 * @param {Array<number | string>} designIds
 */
export async function deleteOrphans(
  designIds: Array<number | string>
): Promise<AdapterDeleteOrphansResponse> {
  const ids = Array.isArray(designIds)
    ? designIds.map((id) => Number(id)).filter((id) => Number.isFinite(id) && id > 0)
    : [];

  try {
    const result = await invokeLoose<{ deleted?: number }>("delete_orphans", {
      request: {
        design_ids: ids,
      },
    });

    return {
      source: "rust",
      persisted: true,
      deleted: Number(result?.deleted ?? 0),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      deleted: 0,
      error: String(error),
    };
  }
}

export async function deleteAllOrphans(): Promise<AdapterDeleteOrphansResponse> {
  try {
    const result = await invokeLoose<{ deleted?: number }>("delete_all_orphans");
    return {
      source: "rust",
      persisted: true,
      deleted: Number(result?.deleted ?? 0),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      deleted: 0,
      error: String(error),
    };
  }
}

/**
 * @param {string} filepath
 */
export async function browseOrphanPath(filepath: string): Promise<AdapterBrowseOrphanPathResponse> {
  try {
    const result = await invokeLoose<{ ok?: boolean; opened?: string }>("browse_orphan_path", {
      filepath: String(filepath || ""),
    });

    return {
      source: "rust",
      ok: Boolean(result?.ok),
      opened: String(result?.opened || ""),
    };
  } catch (error) {
    return {
      source: "mock",
      ok: false,
      opened: "",
      error: String(error),
    };
  }
}

export async function listDesigners(): Promise<AdapterListResponse<AdminEntitySummary>> {
  try {
    const items = await invokeLoose<AdminEntitySummary[]>("list_designers");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          name: String(item?.name || ""),
          design_count: Number(item?.design_count ?? 0),
        })),
      };
    }
  } catch (error) {
    console.info("list_designers unavailable, using mock designers.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, name: "Amazing Designs", design_count: 0 },
      { id: 2, name: "Urban Threads", design_count: 0 },
      { id: 3, name: "Mock Studio", design_count: 0 },
    ],
  };
}

/**
 * @param {string} name
 */
export async function createDesigner(
  name: string
): Promise<AdapterPersistedItemResponse<AdminEntitySummary>> {
  try {
    const item = await invokeLoose<AdminEntitySummary>("create_designer", { request: { name } });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} designerId
 * @param {string} name
 */
export async function updateDesigner(
  designerId: number | string,
  name: string
): Promise<AdapterPersistedItemResponse<AdminEntitySummary>> {
  try {
    const item = await invokeLoose<AdminEntitySummary>("update_designer", {
      request: {
        designer_id: Number(designerId),
        name,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} designerId
 */
export async function deleteDesigner(
  designerId: number | string
): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_designer", { designerId: Number(designerId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function listSources(): Promise<AdapterListResponse<AdminEntitySummary>> {
  try {
    const items = await invokeLoose<AdminEntitySummary[]>("list_sources");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          name: String(item?.name || ""),
          design_count: Number(item?.design_count ?? 0),
        })),
      };
    }
  } catch (error) {
    console.info("list_sources unavailable, using mock sources.", error);
  }

  return {
    source: "mock",
    items: [
      { id: 1, name: "Purchased", design_count: 0 },
      { id: 2, name: "Downloaded", design_count: 2 },
      { id: 3, name: "Gift", design_count: 0 },
    ],
  };
}

/**
 * @param {string} name
 */
export async function createSource(
  name: string
): Promise<AdapterPersistedItemResponse<AdminEntitySummary>> {
  try {
    const item = await invokeLoose<AdminEntitySummary>("create_source", { request: { name } });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} sourceId
 * @param {string} name
 */
export async function updateSource(
  sourceId: number | string,
  name: string
): Promise<AdapterPersistedItemResponse<AdminEntitySummary>> {
  try {
    const item = await invokeLoose<AdminEntitySummary>("update_source", {
      request: {
        source_id: Number(sourceId),
        name,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        name: String(item?.name || ""),
        design_count: Number(item?.design_count ?? 0),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} sourceId
 */
export async function deleteSource(sourceId: number | string): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_source", { sourceId: Number(sourceId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Check whether the user has completed or skipped the initial setup wizard.
 */
export async function checkInitialSetup(): Promise<boolean> {
  try {
    const result = await invokeLoose<boolean>("check_initial_setup");
    return Boolean(result);
  } catch (error) {
    console.info("check_initial_setup failed.", error);
    return true; // Default to true on error — do not block the app.
  }
}

/**
 * Persist that the user has completed or skipped the initial setup wizard.
 */
export async function completeInitialSetup(): Promise<void> {
  try {
    await invokeLoose<void>("complete_initial_setup");
  } catch (error) {
    console.error("complete_initial_setup failed:", error);
  }
}

export async function listTags(): Promise<AdapterListResponse<AdminTagSummary>> {
  try {
    const items = await invokeLoose<AdminTagSummary[]>("list_tags");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          description: String(item?.description || ""),
          tag_group: item?.tag_group == null ? "" : String(item.tag_group),
          design_count: Number(item?.design_count ?? 0),
          is_system: Boolean(item?.is_system ?? false),
        })),
      };
    }
    return {
      source: "rust",
      items: [],
      error: "list_tags returned an unexpected payload.",
    };
  } catch (error) {
    return {
      source: "mock",
      items: [],
      error: String(error),
    };
  }
}

/**
 * @param {string} description
 * @param {string | null} tagGroup
 */
export async function createTag(
  description: string,
  tagGroup: string | null
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("create_tag", {
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
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 * @param {string | null} tagGroup
 */
export async function setTagGroup(
  tagId: number | string,
  tagGroup: string | null
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("set_tag_group", {
      request: { tag_id: Number(tagId), tag_group: tagGroup },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 * @param {string} description
 */
export async function updateTag(
  tagId: number | string,
  description: string
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("update_tag", {
      request: {
        tag_id: Number(tagId),
        description,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 */
export async function deleteTag(tagId: number | string): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_tag", { tagId: Number(tagId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

export async function listHoops(): Promise<AdapterListResponse<AdminHoopSummary>> {
  try {
    const items = await invokeLoose<AdminHoopSummary[]>("list_hoops");
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

/**
 * @param {string} name
 * @param {number} maxWidthMm
 * @param {number} maxHeightMm
 */
export async function createHoop(
  name: string,
  maxWidthMm: number,
  maxHeightMm: number
): Promise<AdapterPersistedItemResponse<AdminHoopSummary>> {
  try {
    const item = await invokeLoose<AdminHoopSummary>("create_hoop", {
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

/**
 * @param {number | string} hoopId
 * @param {string} name
 * @param {number} maxWidthMm
 * @param {number} maxHeightMm
 */
export async function updateHoop(
  hoopId: number | string,
  name: string,
  maxWidthMm: number,
  maxHeightMm: number
): Promise<AdapterPersistedItemResponse<AdminHoopSummary>> {
  try {
    const item = await invokeLoose<AdminHoopSummary>("update_hoop", {
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

/**
 * @param {number | string} hoopId
 */
export async function deleteHoop(hoopId: number | string): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_hoop", { hoopId: Number(hoopId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Fetch the current execution mode and path metadata from the Rust backend.
 * Used to determine Portable vs Installed mode on startup.
 *
 * @returns {Promise<{
 *   source: string,
 *   status: import("../types/AppStatus").AppStatus | null,
 *   error?: string
 * }>}
 */
export async function getAppStatus(): Promise<AdapterAppStatusResponse> {
  try {
    const status = await invokeLoose<AppStatus>("get_app_status");
    if (status && typeof status === "object") {
      const mode = String(status.execution_mode || "");
      return {
        source: "rust",
        status: {
          execution_mode: mode === "dev" ? "dev" : "installed",
          data_root: String(status.data_root || ""),
          embroidery_dir: String(status.embroidery_dir || ""),
          database_path: String(status.database_path || ""),
          data_root_missing: Boolean(status.data_root_missing),
          database_missing: Boolean(status.database_missing),
        },
      };
    }
  } catch (error) {
    console.info("get_app_status unavailable, returning null.", error);
  }

  return {
    source: "mock",
    status: null,
    error: "get_app_status command not available.",
  };
}

/**
 * Fetch the persisted, user-configured data root for Installed mode.
 *
 * Returns `null` on first run (no config yet) so the setup wizard knows to
 * prompt for a data location. In Portable/Dev mode this also returns `null`
 * (there is no config to read).
 *
 * @returns {Promise<{ source: string, path: string | null, error?: string }>}
 */
export async function getConfiguredDataRoot(): Promise<{
  source: string;
  path: string | null;
  error?: string;
}> {
  try {
    const path = await invokeLoose<string | null>("get_configured_data_root");
    return {
      source: "rust",
      path: path ? String(path) : null,
    };
  } catch (error) {
    console.info("get_configured_data_root unavailable, returning null.", error);
    return {
      source: "mock",
      path: null,
      error: String(error),
    };
  }
}

/**
 * Persist the user's chosen data root for Installed mode.
 *
 * Writes the tiny `config.json` under the platform app-data dir so the choice
 * survives reinstalls. The invoke key `dataRoot` maps to the Rust `data_root`.
 *
 * @param {string} dataRoot - Absolute path to the desired data root.
 * @returns {Promise<{ source: string, persisted: boolean, error?: string }>}
 */
export async function setConfiguredDataRoot(dataRoot: string): Promise<{
  source: string;
  persisted: boolean;
  error?: string;
}> {
  const normalized = String(dataRoot || "").trim();
  if (!normalized) {
    return { source: "mock", persisted: false, error: "Data root cannot be empty." };
  }
  try {
    await invokeLoose("set_configured_data_root", { dataRoot: normalized });
    return { source: "rust", persisted: true };
  } catch (error) {
    console.info("set_configured_data_root failed.", error);
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Persist the user-chosen data root for Installed mode and initialize catalogue storage.
 *
 * If an existing database is detected at the chosen location, it is preserved
 * without overwriting. Otherwise a fresh seed database is copied.
 *
 * @param {string} dataRoot - Absolute path to the desired data root.
 * @returns {Promise<AdapterConfigureDataRootResponse>}
 */
export async function configureFreshDataRoot(
  dataRoot: string
): Promise<AdapterConfigureDataRootResponse> {
  const normalized = String(dataRoot || "").trim();
  if (!normalized) {
    return { source: "mock", persisted: false, error: "Data root cannot be empty." };
  }
  try {
    const result = await invokeLoose<ConfigureDataRootResult>("configure_fresh_data_root", {
      dataRoot: normalized,
    });
    if (result && typeof result === "object") {
      return {
        source: "rust",
        persisted: true,
        data_root: String(result.data_root || normalized),
        existing_database_detected: Boolean(result.existing_database_detected),
        database_path: String(result.database_path || ""),
      };
    }
    return { source: "rust", persisted: true };
  } catch (error) {
    console.info("configure_fresh_data_root failed.", error);
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Open a native folder picker to choose the data root for Installed mode.
 *
 * @param {string} [startDir] - Optional starting directory for the picker.
 * @returns {Promise<{ source: string, path: string | null, error?: string }>}
 */
export async function browseDataRootFolder(startDir = ""): Promise<{
  source: string;
  path: string | null;
  error?: string;
}> {
  try {
    const path = await invokeLoose<string | null>("browse_data_root_folder", {
      startDir: String(startDir || "") || null,
    });
    return {
      source: "rust",
      path: path ? String(path) : null,
    };
  } catch (error) {
    console.info("browse_data_root_folder failed.", error);
    return { source: "mock", path: null, error: String(error) };
  }
}

/**
 * Ask the Rust backend to restart the application process.
 *
 * This is used after the initial-setup wizard relocates the data root so the
 * new location takes effect immediately. The backend spawns a fresh copy of
 * the executable (with the same args) and returns once it is launched.
 *
 * @returns {Promise<{ source: string, restarted: boolean, error?: string }>}
 */
export async function restartApplication(): Promise<{
  source: string;
  restarted: boolean;
  error?: string;
}> {
  try {
    const result = await invokeLoose<boolean>("restart_application");
    return {
      source: "rust",
      restarted: Boolean(result),
    };
  } catch (error) {
    console.info("restart_application failed.", error);
    return { source: "mock", restarted: false, error: String(error) };
  }
}

/**
 * Start migrating the active catalogue to a newly selected data root.
 *
 * Streams progress events on `catalogue-storage-migration-progress`. The Rust
 * command force-moves any pre-existing non-empty target aside before copying.
 * The invoke keys are camelCase (`targetDir`, `force`) → Rust `target_dir`,
 * `force`.
 *
 * @param {string} targetDir - Absolute path to the new data root.
 * @returns {Promise<{ source: string, summary: StorageMigrationSummary | null, error?: string }>}
 */
export async function startCatalogueStorageMigration(targetDir: string): Promise<{
  source: string;
  summary: StorageMigrationSummary | null;
  error?: string;
}> {
  const normalized = String(targetDir || "").trim();
  if (!normalized) {
    return { source: "mock", summary: null, error: "Data root cannot be empty." };
  }
  try {
    const summary = await invokeLoose<StorageMigrationSummary>(
      "start_catalogue_storage_migration",
      { targetDir: normalized, force: true }
    );
    return {
      source: "rust",
      summary: summary && typeof summary === "object" ? summary : null,
    };
  } catch (error) {
    console.info("start_catalogue_storage_migration failed.", error);
    return { source: "mock", summary: null, error: String(error) };
  }
}

/**
 * Request cancellation of a running catalogue storage migration (cooperative).
 *
 * @returns {Promise<{ source: string, cancelled: boolean, error?: string }>}
 */
export async function cancelCatalogueStorageMigration(): Promise<{
  source: string;
  cancelled: boolean;
  error?: string;
}> {
  try {
    await invokeLoose<void>("cancel_catalogue_storage_migration");
    return { source: "rust", cancelled: true };
  } catch (error) {
    console.info("cancel_catalogue_storage_migration failed.", error);
    return { source: "mock", cancelled: false, error: String(error) };
  }
}

/**
 * Subscribe to `catalogue-storage-migration-progress` events from Rust.
 *
 * @param {(progress: StorageMigrationProgress) => void} callback
 * @returns {Promise<() => void>} An unlisten function.
 */
export async function listenCatalogueStorageMigrationProgress(
  callback: (progress: StorageMigrationProgress) => void
): Promise<() => void> {
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<StorageMigrationProgress>(
    "catalogue-storage-migration-progress",
    (event) => callback(event.payload)
  );
  return unlisten;
}
