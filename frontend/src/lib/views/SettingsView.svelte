<script>
  import { onMount } from "svelte";
  import {
    getSettingsViewModel,
    saveSettings,
    listGeminiModels,
    testGeminiModel,
    browseSettingsDataRoot,
    restartApplication,
    startCatalogueStorageMigration,
    cancelCatalogueStorageMigration,
    listenCatalogueStorageMigrationProgress,
    getDbStats,
    compactDatabase,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  /** @typedef {import("../types/ipc").StorageMigrationProgress} StorageMigrationProgress */

  /** @typedef {import("../types/ipc").SettingsViewModel} SettingsViewModel */
  /** @typedef {import("../types/ipc").SaveSettingsRequest} SaveSettingsRequest */
  /** @typedef {import("../types/ipc").DbStats} DbStats */
  /** @typedef {import("../types/ipc").CompactResult} CompactResult */

  let settingsLoading = $state(false);
  let settingsLoaded = $state(false);
  let settingsSaveState = $state("idle"); // "idle" | "saving" | "saved" | "error"

  let settingsGoogleApiKey = $state("");
  let settingsApiKeyRevealed = $state(false);
  let settingsAiBatchSize = $state("");
  let settingsAiDelay = $state("");
  let settingsAiGeminiModel = $state("");
  let settingsAiCommitEvery = $state("");
  let settingsAiWorkers = $state("");
  let settingsAiFreeTier = $state(false);
  let settingsGeminiModels = $state([]);
  let settingsModelsLoading = $state(false);
  let settingsModelTesting = $state(false);
  let settingsModelTestMessage = $state("");
  let settingsImportCommitBatchSize = $state("");
  let settingsDbIdleCheckIntervalSecs = $state("1800");
  let dbStats = $state(/** @type {DbStats | null} */ (null));
  let isCompacting = $state(false);

  let settingsCanConfigureDataRoot = $state(false);
  let settingsDataRoot = $state("");
  let settingsLibraryRoot = $state("");
  let settingsDatabasePath = $state("");
  let settingsLogFolder = $state("");
  let settingsAppMode = $state("development");
  let settingsHelpUrl = $state("#/help");
  let showRestartConfirm = $state(false);
  let restarting = $state(false);

  let migrating = $state(false);
  let migrationProgress = $state(/** @type {StorageMigrationProgress | null} */ (null));
  let migrationError = $state("");
  /** @type {(() => void) | null} */
  let unlistenMigration = null;

  // Global UI lock: reflects busyState.active so secondary controls can be
  // disabled while a long-running task runs.
  let busyActive = $derived($busyState.active);

  let settingsHasGoogleApiKey = $derived(settingsGoogleApiKey.trim().length > 0);

  // Effective defaults for blank fields: free-tier keys use a conservative
  // workers/delay pair so runs stay under the ~15 requests/minute limit.
  let settingsDefaultWorkers = $derived(settingsAiFreeTier ? 2 : 4);
  let settingsDefaultDelay = $derived(settingsAiFreeTier ? "10" : "0");

  function toggleSettingsApiKeyVisibility() {
    settingsApiKeyRevealed = !settingsApiKeyRevealed;
  }

  /** @param {Partial<SettingsViewModel>} [model] */
  function applySettingsModel(model = {}) {
    settingsGoogleApiKey = String(model?.google_api_key || "");
    settingsAiBatchSize = String(model?.ai_batch_size || "");
    settingsAiDelay = String(model?.ai_delay || "");
    settingsAiGeminiModel = String(model?.ai_gemini_model || "");
    settingsAiCommitEvery = String(model?.ai_commit_every || "");
    settingsAiWorkers = String(model?.ai_workers || "");
    settingsAiFreeTier = Boolean(model?.ai_free_tier);
    settingsImportCommitBatchSize = String(model?.import_commit_batch_size || "");
    settingsDbIdleCheckIntervalSecs = String(model?.db_idle_check_interval_secs || "1800");
    settingsCanConfigureDataRoot = Boolean(model?.can_configure_data_root);
    settingsDataRoot = String(model?.data_root || "");
    settingsLibraryRoot = String(model?.library_root || model?.data_root || "");
    settingsDatabasePath = String(model?.database_path || "");
    settingsLogFolder = String(model?.log_folder || "");
    settingsAppMode = String(model?.app_mode || "development");
    settingsHelpUrl = String(model?.ai_tagging_help_url || "#/help");
  }

  /** Load available Gemini models for the current API key into the dropdown. */
  async function loadGeminiModels() {
    if (!settingsHasGoogleApiKey) {
      settingsGeminiModels = [];
      return;
    }
    settingsModelsLoading = true;
    try {
      const result = await listGeminiModels(settingsGoogleApiKey.trim());
      settingsGeminiModels = sortModelsFlashFirst(Array.isArray(result?.models) ? result.models : []);
      if (result?.error) {
        addToast(`Could not list Gemini models: ${result.error}`, "error");
      }
    } catch (error) {
      addToast(`Could not list Gemini models: ${error}`, "error");
    } finally {
      settingsModelsLoading = false;
    }
  }

  /**
   * Sort the available Gemini models so flash models come first (preferred
   * `gemini-flash*` aliases, then other `*-flash`, then the rest), matching the
   * backend's auto-selection preference. Flash is recommended for tagging: it is
   * the fastest and cheapest tier for the tiny text/vision prompts used here.
   * @param {string[]} models
   * @returns {string[]}
   */
  function sortModelsFlashFirst(models) {
    const rank = (name) => {
      const lower = String(name || "").toLowerCase();
      if (lower.startsWith("gemini-flash")) return 0;
      if (lower.includes("flash")) return 1;
      return 2;
    };
    return [...models].sort(
      (a, b) => rank(a) - rank(b) || String(a).localeCompare(String(b))
    );
  }

  /** Validate the currently selected Gemini model against the API key. */
  async function testSelectedModel() {
    const model = settingsAiGeminiModel.trim();
    if (!model) {
      addToast("Enter or pick a Gemini model to test.", "warning");
      return;
    }
    settingsModelTesting = true;
    settingsModelTestMessage = "";
    try {
      const result = await testGeminiModel(settingsGoogleApiKey.trim(), model);
      settingsModelTestMessage = result.message;
      addToast(result.message, result.ok ? "success" : "error");
    } catch (error) {
      addToast(`Could not test Gemini model: ${error}`, "error");
    } finally {
      settingsModelTesting = false;
    }
  }

  // Refresh the model dropdown whenever a key is present (and when it changes).
  $effect(() => {
    if (settingsHasGoogleApiKey) {
      loadGeminiModels();
    } else {
      settingsGeminiModels = [];
      settingsModelTestMessage = "";
    }
  });

  /** @param {number} bytes */
  function formatBytes(bytes) {
    if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }
    return `${value.toFixed(value >= 100 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  }

  async function loadDbStats() {
    try {
      const result = await getDbStats();
      if (result.stats) {
        dbStats = result.stats;
      }
    } catch (error) {
      console.info("Could not load database stats.", error);
    }
  }

  async function runManualCompaction() {
    if (isCompacting) return;
    isCompacting = true;
    beginBusy("Compacting database");
    try {
      const result = await compactDatabase();
      if (result.result) {
        const { file_size_before, file_size_after, pages_reclaimed } = result.result;
        const before = formatBytes(file_size_before);
        const after = formatBytes(file_size_after);
        addToast(
          `Database compacted — ${pages_reclaimed.toLocaleString()} pages reclaimed (${before} → ${after})`,
          "success"
        );
        await loadDbStats();
      } else {
        addToast(result.error || "Could not compact database.", "error");
      }
    } catch (error) {
      addToast(`Could not compact database: ${error}`, "error");
    } finally {
      isCompacting = false;
      endBusy();
    }
  }

  async function loadSettingsFromBackend(force = false) {
    if (settingsLoading) return;
    if (settingsLoaded && !force) return;

    settingsLoading = true;
    try {
      const result = await getSettingsViewModel();
      applySettingsModel(result.model);
      settingsLoaded = true;
    } catch (error) {
      addToast(`Could not load settings: ${error}`, "error");
    } finally {
      settingsLoading = false;
    }
  }

  /**
   * Normalise a number-input bound value to a string for the Rust IPC layer.
   *
   * Svelte coerces `bind:value` on `<input type="number">` to a number (or
   * `null` when emptied), even though the field state is a string. The Rust
   * `SaveSettingsRequest` expects these as strings, so convert defensively:
   *   - null / undefined / "" -> "" (leave blank)
   *   - 0 / 6 / 0.5 -> "0" / "6" / "0.5"
   * @param {unknown} value
   * @returns {string}
   */
  function settingsNumericToString(value) {
    if (value === null || value === undefined || value === "") return "";
    return String(value);
  }

  /** @param {Event} event */
  async function saveSettingsFromBackend(event) {
    event.preventDefault();
    settingsSaveState = "saving";

    try {
      /** @type {SaveSettingsRequest} */
      const request = {
        google_api_key: settingsGoogleApiKey,
        ai_batch_size: settingsNumericToString(settingsAiBatchSize),
        ai_delay: settingsNumericToString(settingsAiDelay),
        ai_gemini_model: settingsAiGeminiModel,
        ai_commit_every: settingsNumericToString(settingsAiCommitEvery),
        ai_workers: settingsNumericToString(settingsAiWorkers),
        ai_free_tier: settingsAiFreeTier,
        import_commit_batch_size: settingsNumericToString(settingsImportCommitBatchSize),
        data_root: settingsDataRoot,
        db_idle_check_interval_secs: settingsNumericToString(settingsDbIdleCheckIntervalSecs),
      };

      const result = await saveSettings(request);

      if (result.saved) {
        settingsSaveState = "saved";
        addToast(result.message || "Settings saved successfully.", "success");
      } else {
        settingsSaveState = "error";
        addToast(result.message || "Settings could not be saved.", "error");
      }
    } catch (error) {
      settingsSaveState = "error";
      addToast(`Could not save settings: ${error}`, "error");
    }
  }

  /** @param {StorageMigrationProgress} progress */
  function onMigrationProgress(progress) {
    migrationProgress = progress;
    if (progress.current_phase === "cancelled") {
      migrating = false;
      endBusy();
      addToast("Catalogue migration cancelled.", "info");
    } else if (progress.current_phase === "error") {
      migrating = false;
      endBusy();
      migrationError = progress.error || progress.status_message || "Migration failed.";
      addToast(migrationError, "error");
    } else if (progress.current_phase === "completed") {
      migrating = false;
      endBusy();
      addToast("Catalogue storage migrated successfully.", "success");
    }
  }

  /**
   * Reduce a user-picked catalogue folder to the internal storage data root.
   *
   * The persisted data root is the parent folder that contains `Database/`,
   * `MachineEmbroideryDesigns/` and `logs/`. The Settings field now lets the user
   * point directly at the `MachineEmbroideryDesigns` design library, so when the
   * picked folder's last segment is that library folder we reduce it to its
   * parent — otherwise the migration would double-nest the layout.
   * @param {string} picked
   * @returns {string}
   */
  function normalizePickedDataRoot(picked) {
    const raw = String(picked || "").trim();
    if (!raw) return "";
    const norm = raw.replace(/\\/g, "/").replace(/\/+$/, "");
    const lower = norm.toLowerCase();
    const marker = "/machineembroiderydesigns";
    if (lower === marker || lower.endsWith(marker)) {
      const idx = lower.lastIndexOf(marker);
      const parent = norm.slice(0, idx).replace(/\/+$/, "");
      // Preserve the platform separators used by the rest of the UI.
      return parent ? parent.replace(/\//g, "\\") : "";
    }
    return raw;
  }

  /** @param {string} dataRoot */
  function libraryRootForDataRoot(dataRoot) {
    const trimmed = String(dataRoot || "").trim().replace(/[\\/]+$/, "");
    if (!trimmed) return "";
    const sep = trimmed.includes("/") ? "/" : "\\";
    return `${trimmed}${sep}MachineEmbroideryDesigns`;
  }

  async function browseDataRootFromBackend() {
    const result = await browseSettingsDataRoot(settingsLibraryRoot || settingsDataRoot);
    if (!result.path) {
      if (result.error) {
        addToast(result.error, "error");
      }
      return;
    }

    // Normalise the picked folder to the internal data root so the catalogue
    // layout (Database/, MachineEmbroideryDesigns/, logs/) is not double-nested.
    const newDataRoot = normalizePickedDataRoot(result.path);

    // Begin the full catalogue migration to the freshly picked target.
    migrating = true;
    migrationError = "";
    migrationProgress = null;
    beginBusy("Migrating catalogue storage");
    try {
      unlistenMigration = await listenCatalogueStorageMigrationProgress(onMigrationProgress);
    } catch (error) {
      console.info("Could not subscribe to migration progress.", error);
      unlistenMigration = null;
    }

    const started = await startCatalogueStorageMigration(newDataRoot);
    if (started.error) {
      migrating = false;
      endBusy();
      migrationError = started.error;
      addToast(`Could not start catalogue migration: ${started.error}`, "error");
      await stopListeningToMigration();
      return;
    }
    if (started.summary) {
      settingsDataRoot = newDataRoot;
      settingsLibraryRoot = libraryRootForDataRoot(newDataRoot);
      settingsSaveState = "idle";
      showRestartConfirm = true;
      migrating = false;
      endBusy();
      await closeMigrationModal();
    }
  }

  /** Close/dismiss the migration modal and tear down any progress listener. */
  async function closeMigrationModal() {
    migrating = false;
    migrationError = "";
    migrationProgress = null;
    endBusy();
    await stopListeningToMigration();
  }

  /** @return {Promise<void>} */
  async function stopListeningToMigration() {
    if (typeof unlistenMigration === "function") {
      const fn = unlistenMigration;
      unlistenMigration = null;
      try {
        await fn();
      } catch (error) {
        console.info("Could not unlisten migration progress.", error);
      }
    }
  }

  async function handleCancelMigration() {
    const result = await cancelCatalogueStorageMigration();
    if (result.error) {
      addToast(`Could not cancel migration: ${result.error}`, "error");
    }
  }

  /** Launch the application restart after the user confirms. */
  async function handleRestart() {
    if (restarting) return;
    restarting = true;
    try {
      const res = await restartApplication();
      if (!res.restarted) {
        addToast(
          `Could not restart the application: ${res.error || "unknown error"}. Please close and reopen it manually so your new data location takes effect.`,
          "error"
        );
        restarting = false;
        showRestartConfirm = false;
        return;
      }
      // On success the process is relaunching; the window will close shortly.
    } catch (error) {
      addToast(`Failed to restart the application: ${error}`, "error");
      restarting = false;
      showRestartConfirm = false;
    }
  }

  onMount(() => {
    loadSettingsFromBackend();
    loadDbStats();
  });
</script>

<section class="settings-page space-y-6">
  <h1 class="ui-page-title settings-title mb-6">Application Settings</h1>

  <div class="settings-layout max-w-3xl space-y-6">
    {#if settingsLoading && !settingsLoaded}
      <div
        class="settings-alert settings-alert-info bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-2 text-sm"
      >
        Loading settings...
      </div>
    {/if}

    <form
      class="settings-card settings-form bg-white rounded shadow p-6 space-y-5"
      onsubmit={saveSettingsFromBackend}
    >
      <div>
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Google Gemini API key</h2>
        <p class="text-sm text-gray-600">
          The Google API key is only required if you want your designs to be tagged automatically by
          Google AI.
          <a href={settingsHelpUrl} class="text-indigo-600 hover:underline"
            >Press here for more information.</a
          >
        </p>
      </div>

      <div>
        <label for="settings-google-api-key" class="block text-sm font-semibold text-gray-700 mb-1"
          >API key</label
        >
        <div class="flex items-center gap-2">
          <input
            id="settings-google-api-key"
            type={settingsApiKeyRevealed ? "text" : "password"}
            bind:value={settingsGoogleApiKey}
            placeholder="AIzaSy..."
            autocomplete="off"
            spellcheck="false"
            class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
          />
          <button
            type="button"
            class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
            aria-label="Show or hide API key"
            aria-pressed={settingsApiKeyRevealed}
            title={settingsApiKeyRevealed ? "Hide API key" : "Show API key"}
            onclick={toggleSettingsApiKeyVisibility}
          >
            <span aria-hidden="true" class="settings-eye-icon"
              >{settingsApiKeyRevealed ? "🙈" : "👁"}</span
            >
          </button>
        </div>
        <p class="mt-2 text-xs text-gray-500">
          {#if settingsHasGoogleApiKey}
            A key is currently saved in <code>.env</code>. You can leave it as-is or replace it
            here.
          {:else}
            Leave this blank if you only want keyword-based tagging with no Google AI calls.
          {/if}
        </p>
      </div>

      <div class="border-t pt-4">
        <label class="flex flex-col gap-1 text-sm text-gray-700 cursor-pointer">
          <span class="flex items-center gap-2">
            <input
              type="checkbox"
              bind:checked={settingsAiFreeTier}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            My Google API key is on the <strong>free tier</strong>
          </span>
          <span class="text-xs text-gray-500"
            >Tick this only if your key is on the free tier - it has stricter rate limits.</span
          >
        </label>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div>
          <label
            for="settings-ai-batch-size"
            class="block text-sm font-semibold text-gray-700 mb-1"
          >
            AI tagging batch size <span class="font-normal text-gray-500">(optional)</span>
          </label>
          <input
            id="settings-ai-batch-size"
            type="number"
            min="1"
            bind:value={settingsAiBatchSize}
            placeholder="e.g. 100"
            class="settings-input border rounded px-3 py-2 text-sm w-full"
          />
          <p class="mt-1 text-xs text-gray-500">Designs processed per batch (default 100).</p>
        </div>
        <div>
          <label
            for="settings-ai-commit-every"
            class="block text-sm font-semibold text-gray-700 mb-1"
          >
            Commit every <span class="font-normal text-gray-500">(optional)</span>
          </label>
          <input
            id="settings-ai-commit-every"
            type="number"
            min="1"
            bind:value={settingsAiCommitEvery}
            placeholder="e.g. 100"
            class="settings-input border rounded px-3 py-2 text-sm w-full"
          />
          <p class="mt-1 text-xs text-gray-500">Progress/commit cadence during a run (default 100).</p>
        </div>
        <div>
          <label
            for="settings-ai-workers"
            class="block text-sm font-semibold text-gray-700 mb-1"
          >
            Workers <span class="font-normal text-gray-500">(optional)</span>
          </label>
          <input
            id="settings-ai-workers"
            type="number"
            min="1"
            max="32"
            bind:value={settingsAiWorkers}
            placeholder={`e.g. ${settingsDefaultWorkers}${settingsAiFreeTier ? " (free tier)" : ""}`}
            class="settings-input border rounded px-3 py-2 text-sm w-full"
          />
          <p class="mt-1 text-xs text-gray-500">
            Designs tagged in parallel (default {settingsDefaultWorkers}). Lower to avoid Gemini
            rate-limit (429) errors.
          </p>
        </div>
      </div>

      <div>
        <label for="settings-ai-delay" class="block text-sm font-semibold text-gray-700 mb-1">
          Delay between Gemini calls (seconds) <span class="font-normal text-gray-500"
            >(optional)</span
          >
        </label>
        <input
          id="settings-ai-delay"
          type="number"
          min="0"
          step="0.5"
          bind:value={settingsAiDelay}
          placeholder={`e.g. ${settingsDefaultDelay}`}
          class="settings-input border rounded px-3 py-2 text-sm w-56"
        />
        <p class="mt-1 text-xs text-gray-500">
          Seconds to wait between API calls. Increase this if you see <em>429 Too Many Requests</em>
          errors. Leave blank for the default ({settingsDefaultDelay} s) shown above. Also applies to
          batch tagging actions on the
          <a href="#/admin/tagging-actions" class="text-indigo-600 hover:underline"
            >Tagging Actions</a
          > page.
        </p>
      </div>

      <div>
        <label for="settings-ai-model" class="block text-sm font-semibold text-gray-700 mb-1">
          Gemini model <span class="font-normal text-gray-500">(optional)</span>
        </label>
        <div class="flex flex-wrap items-center gap-2">
          <select
            id="settings-ai-model"
            bind:value={settingsAiGeminiModel}
            disabled={!settingsHasGoogleApiKey || busyActive}
            class="settings-input border rounded px-3 py-2 text-sm w-72"
          >
            <option value="">Auto-select (recommended)</option>
            {#each settingsGeminiModels as modelName}
              <option value={modelName}>{modelName}</option>
            {/each}
          </select>
          <button
            type="button"
            class="menu-button-secondary px-3 py-2 text-sm"
            onclick={loadGeminiModels}
            disabled={!settingsHasGoogleApiKey || settingsModelsLoading || busyActive}
          >
            {settingsModelsLoading ? "Loading…" : "Refresh"}
          </button>
          <button
            type="button"
            class="menu-button-secondary px-3 py-2 text-sm"
            onclick={testSelectedModel}
            disabled={!settingsHasGoogleApiKey || settingsModelTesting || busyActive}
          >
            {settingsModelTesting ? "Testing…" : "Test model"}
          </button>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          Model used for Visual AI tagging. Leave blank to let the app
          auto-select an available Gemini model. If a model you pick is later retired, the app
          falls back to auto-selection at run time.
        </p>
        <p class="mt-1 text-xs text-gray-500">
          <strong>Flash models are recommended</strong> — they are the fastest and cheapest for
          tagging. Pro/thinking models cost more and run slower for the same small tag prompt.
        </p>
        {#if settingsModelTestMessage}
          <p class="mt-1 text-xs text-gray-600">{settingsModelTestMessage}</p>
        {/if}
      </div>

      <div>
        <label
          for="settings-import-commit-batch-size"
          class="block text-sm font-semibold text-gray-700 mb-1"
        >
          Import database commit batch size <span class="font-normal text-gray-500">(optional)</span
          >
        </label>
        <input
          id="settings-import-commit-batch-size"
          type="number"
          min="1"
          bind:value={settingsImportCommitBatchSize}
          placeholder="e.g. 10 — leave blank for default"
          class="settings-input border rounded px-3 py-2 text-sm w-56"
        />
        <p class="mt-1 text-xs text-gray-500">
          Controls how many designs are written or tag-updated before each database commit during
          import. Leave blank to use the default batch size of 10. Lower values reduce rollback size
          on failure; higher values reduce commit overhead.
        </p>
      </div>

      <div>
        <label
          for="settings-db-idle-check-interval"
          class="block text-sm font-semibold text-gray-700 mb-1"
        >
          Database health check interval (seconds)
        </label>
        <input
          id="settings-db-idle-check-interval"
          type="number"
          min="5"
          bind:value={settingsDbIdleCheckIntervalSecs}
          placeholder="e.g. 1800"
          class="settings-input border rounded px-3 py-2 text-sm w-48"
        />
        <p class="mt-1 text-xs text-gray-500">
          How often the app checks for database fragmentation (default 1800 = 30 minutes). When free
          space exceeds 20% and 20&nbsp;MB, a background scan reclaims the space without pausing the
          app. Minimum 5 seconds (for testing).
        </p>
      </div>

      <div class="border-t pt-4 space-y-3">
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Database Maintenance</h2>
        <p class="text-sm text-gray-600">
          The catalogue database can grow as designs are added, edited and removed. This shows
          current storage usage and lets you compact the database to reclaim unused space. Your
          embroidery files are never modified.
        </p>

        {#if dbStats}
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div class="bg-gray-50 border rounded p-3">
              <p class="text-xs font-semibold text-gray-500 uppercase">Database size</p>
              <p class="text-lg font-bold text-gray-800">{formatBytes(dbStats.file_size_bytes)}</p>
            </div>
            <div class="bg-gray-50 border rounded p-3">
              <p class="text-xs font-semibold text-gray-500 uppercase">Recoverable</p>
              <p class="text-lg font-bold text-emerald-600">
                {formatBytes(dbStats.reclaimable_bytes)}
              </p>
            </div>
          </div>
        {:else}
          <p class="text-xs text-gray-500 italic">Database statistics unavailable.</p>
        {/if}

        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          onclick={runManualCompaction}
          disabled={isCompacting || busyActive}
        >
          {isCompacting ? "Compacting…" : "Optimize & Compact Database"}
        </button>
        <p class="text-xs text-gray-500">
          Runs a full database optimisation (VACUUM + PRAGMA optimize). This may take a moment for
          large databases and requires sufficient free disk space.
        </p>
      </div>

      <div class="border-t pt-4 space-y-3">
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Catalogue storage</h2>
        <p class="text-sm text-gray-600">
          Large catalogue data lives under a single home folder.
          {#if settingsCanConfigureDataRoot}
            For desktop installs you can point this to a larger drive. Changes apply after
            restarting the app, and any missing managed files are copied into the new location
            automatically.
          {:else}
            In {settingsAppMode} mode this location follows the application folder automatically.
          {/if}
        </p>

        {#if settingsCanConfigureDataRoot}
          <div>
            <label for="settings-data-root" class="block text-sm font-semibold text-gray-700 mb-1"
              >Catalogue data location</label
            >
            <div class="flex items-center gap-2">
              <input
                id="settings-data-root"
                type="text"
                bind:value={settingsLibraryRoot}
                placeholder="D:\\EmbroideryCatalogueData\\MachineEmbroideryDesigns"
                spellcheck="false"
                class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
              />
              <button
                type="button"
                class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
                onclick={browseDataRootFromBackend}
                disabled={busyActive}
              >
                Browse…
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500">
              This is the design library folder that directly holds your imported files
              (MachineEmbroideryDesigns). Choose it or its parent to relocate the catalogue.
            </p>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between gap-3">
        <p class="text-xs text-gray-500">
          These settings are stored in the catalogue database for this installation.
        </p>
        <button
          type="submit"
          class="settings-primary-button menu-button-primary"
          disabled={settingsSaveState === "saving" || busyActive}
        >
          {settingsSaveState === "saving" ? "Saving..." : "Save settings"}
        </button>
      </div>
    </form>

    <div class="settings-card settings-meta bg-white rounded shadow p-6 space-y-5">
      <div>
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Storage locations</h2>
        <p class="text-sm text-gray-600">
          The catalogue database and imported embroidery files live under the catalogue data
          location shown below. Logs are stored separately so they survive data moves.
        </p>
      </div>

      <div>
        <p class="block text-sm font-semibold text-gray-700 mb-1">Catalogue data location</p>
        <code
          class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all"
          >{settingsDataRoot}</code
        >
      </div>

      <div>
        <p class="block text-sm font-semibold text-gray-700 mb-1">Log folder</p>
        <code
          class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all"
          >{settingsLogFolder}</code
        >
      </div>

      <div>
        <p class="block text-sm font-semibold text-gray-700 mb-1">Database</p>
        <code
          class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all"
          >{settingsDatabasePath}</code
        >
      </div>
    </div>
  </div>
</section>

{#if showRestartConfirm}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Restart required"
    data-testid="settings-restart-dialog"
  >
    <div class="bg-white rounded-xl shadow-lg max-w-md w-full p-6 space-y-4">
      <h2 class="text-lg font-bold text-gray-800">Restart required</h2>
      <p class="text-sm text-gray-600">
        Your new data location has been saved. Embroidery Catalogue needs to restart so it can begin
        using <span class="font-medium text-gray-800">{settingsDataRoot}</span>.
      </p>
      <div class="flex items-center justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={handleRestart}
          disabled={restarting || busyActive}
          class="bg-indigo-600 text-white px-5 py-2 rounded text-sm font-medium
                 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500
                 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          data-testid="settings-restart-now"
        >
          {#if restarting}
            Restarting…
          {:else}
            Restart now
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if migrating || migrationProgress || migrationError}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Catalogue storage migration"
    data-testid="catalogue-migration-dialog"
  >
    <div class="bg-white rounded-xl shadow-lg max-w-md w-full p-6 space-y-4">
      <h2 class="text-lg font-bold text-gray-800">Moving your catalogue…</h2>
      <p class="text-sm text-gray-600">
        Your database and design library are being moved to the new storage location. Your original
        embroidery files remain untouched.
      </p>

      {#if migrationError}
        <div
          class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm"
          data-testid="catalogue-migration-error"
        >
          {migrationError}
        </div>
      {/if}

      {#if migrationProgress}
        <div>
          <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
            <span data-testid="catalogue-migration-status">{migrationProgress.status_message}</span>
            <span>{Math.round(migrationProgress.percent * 100)}%</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
            <div
              class="h-2 bg-indigo-600 rounded-full transition-all"
              style="width: {Math.round(migrationProgress.percent * 100)}%"
              data-testid="catalogue-migration-progress-bar"
            ></div>
          </div>
          <p class="mt-2 text-xs text-gray-500" data-testid="catalogue-migration-counts">
            {migrationProgress.items_copied} of {migrationProgress.total_items} files
          </p>
        </div>
      {/if}

      {#if migrating}
        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={handleCancelMigration}
            class="bg-gray-100 text-gray-700 border border-gray-300 px-4 py-2 rounded text-sm font-medium
                   hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            data-testid="cancel-catalogue-migration"
          >
            Cancel
          </button>
        </div>
      {:else if migrationError || migrationProgress}
        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={closeMigrationModal}
            class="bg-indigo-600 text-white px-5 py-2 rounded text-sm font-medium
                   hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            data-testid="close-catalogue-migration"
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
