// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose, type LooseRecord } from "./ipcClient";
import type {
  AdapterBackupViewModelResponse,
  AdapterBrowseBackupFolderResponse,
  AdapterRunBothBackupsResponse,
  AdapterSaveBackupSettingsResponse,
  BackupViewModel,
  CancelBackupResult,
  CancelRestoreResult,
  DatabaseBackupResult,
  DetectUnmatchedFilesResult,
  DesignsBackupResult,
  SaveBackupSettingsRequest,
  ImportUnmatchedFilesResult,
  RestoreBothResult,
  RestoreDatabaseResult,
  RestoreDesignsResult,
  BrowseRestoreFileResponse,
} from "../types/ipc";

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

/**
 * Open a file picker for a database backup snapshot, defaulting to the
 * configured Database backup folder and restricted to `.db` files.
 * @param {string} [startDir]
 */
export async function browseRestoreFile(startDir = ""): Promise<BrowseRestoreFileResponse> {
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
function normalizeRestoreDatabase(result: LooseRecord) {
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
  { source: string; error?: string } & DetectUnmatchedFilesResult
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
    return { source: "mock", error: String(error), checked: 0, unmatched: 0, sample: [] };
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
      flagged: Number(result?.flagged ?? 0),
      failed: Number(result?.failed ?? 0),
      failed_samples: Array.isArray(result?.failed_samples)
        ? result.failed_samples.map((p) => String(p))
        : [],
      cancelled: Boolean(result?.cancelled ?? false),
    };
  } catch {
    return {
      source: "mock",
      detected: 0,
      imported: 0,
      flagged: 0,
      failed: 0,
      failed_samples: [],
      cancelled: false,
    };
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
  } catch {
    return { source: "mock", cancel_requested: false };
  }
}
