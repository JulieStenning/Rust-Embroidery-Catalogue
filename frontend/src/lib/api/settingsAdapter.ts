import { invokeLoose } from "./ipcClient";
import type {
  AdapterAppStatusResponse,
  AdapterBrowseDataRootResponse,
  AdapterConfigureDataRootResponse,
  AdapterGoogleApiKeyResponse,
  AdapterSaveSettingsResponse,
  AdapterSettingsViewModelResponse,
  AppStatus,
  ConfigureDataRootResult,
  DatabaseStatus,
  DatabaseValidation,
  DetectedDataRoot,
  GeminiModelTestResult,
  SaveSettingsRequest,
  SettingsViewModel,
  StorageMigrationProgress,
  StorageMigrationSummary,
} from "../types/ipc";

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
        title: "AI Tagging & Batch Operations Guide",
        description:
          "How to run Visual AI tagging from Batch Operations, set up a Google API key, and understand usage costs.",
        filename: "docs/User-Facing-Guidance/BATCH_OPERATIONS_BACKFILL.md",
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
      ai_batch_size: "",
      ai_delay: "",
      ai_gemini_model: "",
      ai_commit_every: "",
      ai_workers: "",
      ai_free_tier: false,
      import_last_browse_folder: "",
      can_configure_data_root: false,
      data_root: "",
      library_root: "",
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
 * List Gemini models available for the given API key (populates the Settings
 * model dropdown).
 * @param {string} apiKey
 */
export async function listGeminiModels(
  apiKey: string
): Promise<{ source: "rust" | "mock"; models: string[]; error?: string }> {
  try {
    const result = await invokeLoose<{ models?: string[] } | string[]>("list_gemini_models", {
      apiKey,
    });
    const models = Array.isArray(result)
      ? result.map(String)
      : Array.isArray(result?.models)
        ? result.models.map(String)
        : [];
    return { source: "rust", models };
  } catch (error) {
    console.info("list_gemini_models failed.", error);
    return { source: "mock", models: [], error: String(error) };
  }
}

/**
 * Validate a Gemini model against the given API key (Settings "Test model"
 * button).
 * @param {string} apiKey
 * @param {string} model
 */
export async function testGeminiModel(
  apiKey: string,
  model: string
): Promise<GeminiModelTestResult> {
  try {
    const result = await invokeLoose<GeminiModelTestResult>("test_gemini_model", {
      apiKey,
      model,
    });
    return { ok: Boolean(result?.ok), message: String(result?.message || "") };
  } catch (error) {
    console.info("test_gemini_model failed.", error);
    return { ok: false, message: String(error) };
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
