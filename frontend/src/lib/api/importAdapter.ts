// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type {
  AdapterBrowseImportFolderResponse,
  AdapterCancelBulkImportScanResponse,
  AdapterImportPrecheckActionResponse,
  AdapterImportPrecheckResponse,
  AdapterImportPreviewResponse,
  AdapterStopBulkImportResponse,
  BulkImportPreview,
  BrowseImportFolderResult,
  ImportPrecheckActionResult,
  ImportPrecheckResult,
} from "../types/ipc";

/**
 * Try import preview using existing Rust bulk import command.
 * Falls back to a mock preview shape if command wiring is incomplete.
/**
 * Request a scan preview from one or more root folders.
 *
 * @param {string | string[]} rootPaths
 * @param {boolean} [includeLibrary=false]
 */
export async function previewImportFromRoots(
  rootPaths: string | string[],
  includeLibrary = false
): Promise<AdapterImportPreviewResponse> {
  const normalizedRoots = Array.isArray(rootPaths)
    ? rootPaths.map((rootPath) => String(rootPath || "").trim()).filter(Boolean)
    : [];

  if (normalizedRoots.length === 0 && !includeLibrary) {
    return {
      source: "mock",
      preview: {
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        scanned_files: [],
        resolved_assignments: [],
        scan_token: "",
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
        rootPath: normalizedRoots[0] || null,
        rootPaths: normalizedRoots,
        includeLibrary: Boolean(includeLibrary),
        fallbackDesignerId: null,
        fallbackSourceId: null,
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
        scan_token: String(preview?.scan_token || ""),
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
        scan_token: "",
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
 * Run import precheck (Option B). Instead of re-sending every selected file
 * path, the frontend sends a `scan_token` (minted by the preview/scan step)
 * plus folder-level selection deltas and assignments; the backend reconstructs
 * `selected_files` from its own stored scan catalogue.
 * @param {Record<string, any> | null} request
 */
export async function precheckImportWire(
  request: Record<string, unknown> | null
): Promise<AdapterImportPrecheckResponse> {
  const payload = request && typeof request === "object" ? request : null;
  if (!payload) {
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
      message: "Missing import precheck request payload.",
    };
  }

  try {
    const precheck = await invokeLoose<ImportPrecheckResult>("precheck_bulk_import_from_scan", {
      request: payload,
    });

    return {
      source: "rust",
      precheck,
      message: "Precheck loaded from Rust command.",
    };
  } catch (error) {
    console.info("precheck_bulk_import_from_scan unavailable or failed.", error);
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
 * Request cancellation for an in-flight folder scan.
 */
export async function requestCancelBulkImportScan(): Promise<AdapterCancelBulkImportScanResponse> {
  try {
    const result = await invokeLoose<{ cancel_requested?: boolean }>(
      "request_cancel_bulk_import_scan"
    );
    return {
      source: "rust",
      cancelRequested: Boolean(result?.cancel_requested),
      message: "Cancel requested for the folder scan.",
    };
  } catch (error) {
    console.info(
      "request_cancel_bulk_import_scan unavailable or failed, using mock cancel result.",
      error
    );
    return {
      source: "mock",
      cancelRequested: true,
      message: "Cancel requested (mock).",
    };
  }
}
