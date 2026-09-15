import { invokeLoose, type LooseRecord } from "./ipcClient";
import type {
  AdapterBrowseDesignsPageResponse,
  AdapterReparseDesignResponse,
  AdapterItemResponse,
  AdapterListResponse,
  AdapterMutationResponse,
  BrowseDesignPreview,
  BrowseDesignSummaryWire,
  DesignCommandResult,
  DesignDetail,
  DesignDetailWire,
  DesignImageData,
  ReparseDesignResultWire,
  SearchPayload,
  UpdateDesignMetadataRequest,
} from "../types/ipc";
import { mapDesignDetailFromWire, mapReparseDesignFromWire } from "../types/ipc";

export const MOCK_DESIGNS = [
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
    tags: ["Words and Letters"],
    hoop: null,
    rating: null,
    is_stitched: false,
    image_tags_verified: true,
    stitching_tags_verified: true,
  },
];

export const TAG_SEED = [
  "Line Outline",
  "Satin Stitch",
  "Applique",
  "Food",
  "Nautical",
  "Words and Letters",
  "Floral",
  "Butterflies and Insects",
];

export const MOCK_HOOPS = [
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
 * Fetch the full ordered list of design IDs matching the current browse
 * filters/sort (no pagination). This drives the detail view's Prev/Next
 * navigation across the whole filtered result set rather than just the current
 * page. Falls back to an empty list when the command surface is unavailable;
 * BrowseView then falls back to the current page's IDs.
 * @param {SearchPayload} [payload]
 */
export async function getDesignIds(payload?: SearchPayload): Promise<number[]> {
  try {
    const result = await invokeLoose<{ ids?: Array<number | string> } | null | undefined>(
      "get_design_ids",
      { payload }
    );
    if (result && Array.isArray(result.ids)) {
      return result.ids.map((id) => Number(id)).filter((id) => Number.isFinite(id));
    }
  } catch (error) {
    console.info("get_design_ids not available yet.", error);
  }
  return [];
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
