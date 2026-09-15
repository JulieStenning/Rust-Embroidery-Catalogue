import { invokeLoose } from "./ipcClient";
import type {
  AdapterBrowseOrphanPathResponse,
  AdapterDeleteOrphansResponse,
  AdapterOrphansPageResponse,
  AdapterScanOrphansResponse,
} from "../types/ipc";

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
