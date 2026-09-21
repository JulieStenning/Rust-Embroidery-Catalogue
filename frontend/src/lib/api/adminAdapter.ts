// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type {
  AdapterPersistedItemResponse,
  AdapterPersistedResponse,
  AdapterCompactResponse,
  AdapterDbStatsResponse,
  AdapterListResponse,
  AdminEntitySummary,
  AdminHoopSummary,
  CompactResult,
  DbStats,
} from "../types/ipc";

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
      { id: 1, name: "The Rose Studio", design_count: 0 },
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
