// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type {
  AdapterBackfillLogEntriesResponse,
  AdapterStopUnifiedBackfillResponse,
  AdapterBatchOperationsViewModelResponse,
  AdapterTaggingCandidateCountResponse,
  BrowseTaggingFolderResult,
  TaggingScopeCounts,
  RunStitchingBackfillOptions,
  BatchOperationsViewModel,
  UnifiedBackfillActionsWire,
  UnifiedBackfillRequest,
  UnifiedBackfillResult,
  UnifiedBackfillWireRequest,
} from "../types/ipc";

export async function getBatchOperationsViewModel(): Promise<AdapterBatchOperationsViewModelResponse> {
  try {
    const model = await invokeLoose<BatchOperationsViewModel>("get_batch_operations_view_model");
    return {
      source: "rust",
      model: {
        has_google_api_key: Boolean(model?.has_google_api_key),
        ai_vision_auto: Boolean(model?.ai_vision_auto),
        ai_batch_size: String(model?.ai_batch_size || ""),
        ai_delay: String(model?.ai_delay || ""),
        ai_commit_every: String(model?.ai_commit_every || ""),
        ai_workers: String(model?.ai_workers || ""),
        ai_free_tier: Boolean(model?.ai_free_tier),
        default_batch_size: Number(model?.default_batch_size ?? 100),
        default_commit_every: Number(model?.default_commit_every ?? 100),
        default_workers: Number(model?.default_workers ?? 4),
        default_delay: Number(model?.default_delay ?? 5),
        data_storage_location: String(model?.data_storage_location || ""),
      },
    };
  } catch (error) {
    return {
      source: "mock",
      model: {
        has_google_api_key: false,
        ai_vision_auto: false,
        ai_batch_size: "",
        ai_delay: "",
        ai_commit_every: "",
        ai_workers: "",
        ai_free_tier: false,
        default_batch_size: 100,
        default_commit_every: 100,
        default_workers: 4,
        default_delay: 5,
        data_storage_location: "",
      },
      error: String(error),
    };
  }
}

/**
 * Count how many designs a tagging run with the given scope `action` would
 * process. Uses the same candidate predicate as the backfill pager on the Rust
 * side, so the pre-flight estimate matches what a run actually touches.
 *
 * @param {string} action Backend scope: `tag_untagged` | `retag_all_unverified` | `retag_all`.
 * @param {Array<string>} [folderPaths] Library subfolders (absolute) to scope to; when
 *   provided, only designs under ANY of these folders are counted.
 * @param {boolean} [includeSubfolders]
 */
export async function countTaggingCandidates(
  action: string,
  folderPaths?: string[] | null,
  includeSubfolders?: boolean | null
): Promise<AdapterTaggingCandidateCountResponse> {
  try {
    const payload: Record<string, unknown> = { action: String(action) };
    const folders = Array.isArray(folderPaths)
      ? folderPaths.map((value) => String(value || "").trim()).filter(Boolean)
      : [];
    if (folders.length > 0) payload.folderPaths = folders;
    if (includeSubfolders != null) payload.includeSubfolders = Boolean(includeSubfolders);
    const result = await invokeLoose<TaggingScopeCounts>("count_tagging_candidates", payload);
    return {
      source: "rust",
      action: String(action),
      counts: {
        total_count: Number(result?.total_count ?? 0),
        unverified_count: Number(result?.unverified_count ?? 0),
        verified_count: Number(result?.verified_count ?? 0),
      },
    };
  } catch (error) {
    return {
      source: "mock",
      action: String(action),
      counts: { total_count: 0, unverified_count: 0, verified_count: 0 },
      error: String(error),
    };
  }
}

/**
 * Open a native multi-folder picker bounded to the Data Storage Location. The
 * backend opens the dialog at the library root and rejects any chosen folder
 * outside it, mirroring the import folder picker (the user can select several
 * folders at once, e.g. Ctrl/Shift-click).
 *
 * @param {string} [startDir] Optional absolute path under the root to start from.
 */
export async function browseTaggingFolder(
  startDir?: string | null
): Promise<BrowseTaggingFolderResult> {
  try {
    const result = await invokeLoose<{
      path?: string | null;
      paths?: string[];
      relative_paths?: string[];
      error?: string;
    }>("browse_tagging_folder", {
      request: {
        start_dir: startDir ? String(startDir).trim() : null,
        allow_multi: true,
      },
    });
    return {
      path: result?.path ?? null,
      paths: Array.isArray(result?.paths)
        ? result.paths.map((item) => String(item || "")).filter(Boolean)
        : [],
      relative_paths: Array.isArray(result?.relative_paths)
        ? result.relative_paths.map((item) => String(item || "")).filter(Boolean)
        : [],
      error: result?.error,
    };
  } catch (error) {
    return { path: null, paths: [], relative_paths: [], error: String(error) };
  }
}

/**
 * Translate the flat view-model from the Batch Operations screen into the
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
  // Tagging runs whenever an explicit mode list is provided OR the legacy
  // run_vision toggle is on.
  const runTagging = Boolean(request.modes?.length) || Boolean(request.run_vision);
  const knownTagActions = new Set([
    "tag_untagged",
    "retag_all",
    "retag_all_unverified",
    "retag_all_vision_not_analyzed",
    "retag_all_vision_no_match",
    "retag_all_vision_analyzed",
  ]);
  const actionMode =
    request.action_mode === "tag_all"
      ? "retag_all"
      : knownTagActions.has(String(request.action_mode || ""))
        ? String(request.action_mode)
        : "tag_untagged";

  // File & Folder Rules (path_rule) always runs; Gemini Vision (ai_vision) runs when
  // its toggle is on (and is additionally gated on an API key by the backend).
  // The new workflow passes an explicit `modes` list so a Gemini-Vision-only goal is
  // expressible without the always-on path_rule fallback.
  let modes: string[];
  if (Array.isArray(request.modes) && request.modes.length) {
    modes = request.modes.slice();
  } else {
    modes = ["path_rule"];
    if (request.run_vision) modes.push("ai_vision");
  }

  const taggingWire: UnifiedBackfillActionsWire["tagging"] = {
    action: actionMode,
    modes,
    enabled: true,
  };
  if (request.merge_mode) taggingWire.merge_mode = request.merge_mode;
  if (request.exclude_verified !== undefined) {
    taggingWire.exclude_verified = Boolean(request.exclude_verified);
  }
  if (request.folder_path) taggingWire.folder_path = String(request.folder_path);
  if (Array.isArray(request.folder_paths) && request.folder_paths.length) {
    taggingWire.folder_paths = request.folder_paths
      .map((value) => String(value || "").trim())
      .filter(Boolean);
  }
  if (request.include_subfolders !== undefined) {
    taggingWire.include_subfolders = Boolean(request.include_subfolders);
  }

  return {
    actions: {
      tagging: runTagging ? taggingWire : null,
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
      image_tag_count_before: Number(result?.image_tag_count_before ?? 0),
      image_tag_count_after: Number(result?.image_tag_count_after ?? 0),
      missing_preview_count_before: Number(result?.missing_preview_count_before ?? 0),
      missing_preview_count_after: Number(result?.missing_preview_count_after ?? 0),
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

/**
 * Run a maintenance-only batch: preview generation, colour/stitch-count and
 * hoop/dimension recalculation with NO tagging. Delegates to the dedicated
 * Rust `run_maintenance_batch` command. `scope` bounds EVERY selected task to
 * either the whole catalogue ("all") or only designs missing a preview image
 * ("missing_previews").
 *
 * @param {{
 *   scope?: "all" | "missing_previews",
 *   generate_previews?: boolean,
 *   recalc_color_counts?: boolean,
 *   recalc_hoop_dimensions?: boolean,
 *   commit_every?: number,
 *   batch_size?: number,
 *   workers?: number,
 * }} [options]
 */
export async function runMaintenanceBackfill({
  scope = "all",
  generate_previews = false,
  recalc_color_counts = false,
  recalc_hoop_dimensions = false,
  commit_every = 100,
  batch_size = 100,
  workers = 4,
}: {
  scope?: "all" | "missing_previews";
  generate_previews?: boolean;
  recalc_color_counts?: boolean;
  recalc_hoop_dimensions?: boolean;
  commit_every?: number;
  batch_size?: number;
  workers?: number;
} = {}): Promise<UnifiedBackfillResult> {
  try {
    const result = await invokeLoose<UnifiedBackfillResult>("run_maintenance_batch", {
      request: {
        scope: scope === "missing_previews" ? "missing_previews" : "all",
        generate_previews: Boolean(generate_previews),
        recalc_color_counts: Boolean(recalc_color_counts),
        recalc_hoop_dimensions: Boolean(recalc_hoop_dimensions),
        commit_every: Number(commit_every),
        batch_size: Number(batch_size),
        workers: Number(workers),
      },
    });
    return {
      source: "rust",
      processed: Number(result?.processed ?? 0),
      errors: Number(result?.errors ?? 0),
      stopped: Boolean(result?.stopped),
      actions: Array.isArray(result?.actions) ? result.actions.map(String) : [],
      commit_every: Number(result?.commit_every ?? commit_every),
      batch_size: Number(result?.batch_size ?? batch_size),
      workers: Number(result?.workers ?? workers),
      missing_preview_count_before: Number(result?.missing_preview_count_before ?? 0),
      missing_preview_count_after: Number(result?.missing_preview_count_after ?? 0),
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

/**
 * Number of designs with no stored preview (`image_data IS NULL`) — the "missing
 * preview" population on the Maintenance tab's Target Scope card.
 */
export async function countMissingPreviews(): Promise<number> {
  try {
    const result = await invokeLoose<number>("count_missing_preview_designs");
    return Number(result ?? 0);
  } catch (error) {
    console.info("count_missing_preview_designs unavailable.", error);
    return 0;
  }
}
