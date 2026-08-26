<script>
  import { onDestroy, onMount } from "svelte";
  import {
    getBackupViewModel,
    saveBackupSettings,
    browseBackupFolder,
    runDatabaseBackup,
    runDesignsBackup,
    runBothBackups,
    requestCancelBackup,
    getSettingsViewModel,
    browseRestoreFile,
    restoreDatabase,
    restoreDesignsIncremental,
    restoreBoth,
    importUnmatchedDesignFiles,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { resetRestoreProgress } from "../stores/restoreProgressStore.js";
  import { initRestoreProgressEvents } from "../services/restoreEvents.js";
  import CancelBackupModal from "../components/CancelBackupModal.svelte";
  import RestoreProgressPanel from "../components/RestoreProgressPanel.svelte";

  let backupDbDestination = $state("");
  let backupDesignsDestination = $state("");
  let backupSavedDbDestination = $state("");
  let backupSavedDesignsDestination = $state("");
  let backupDbSourcePath = $state("(not available yet)");
  let backupDesignsSourcePath = $state("(not available yet)");
  let backupLoaded = $state(false);
  let backupLoading = $state(false);
  let backupDatabaseRunning = $state(false);
  let backupDesignsRunning = $state(false);

  let settingsDataRoot = $state("");

  // Cancellation UI state. The confirmation modal never pauses background
  // execution — the running backup command continues until the user confirms.
  let showCancelConfirm = $state(false);
  let cancelling = $state(false);
  let activeBackupAction = $state(/** @type {"database" | "designs" | "both" | null} */ (null));

  // ── Restore state ─────────────────────────────────────────────────────────
  let restoreDbFile = $state(""); // selected database backup (.db) file
  let restoreDesignsSource = $state(""); // designs backup folder to restore from
  let restoreDatabaseRunning = $state(false);
  let restoreDesignsRunning = $state(false);
  let restoreAnyRunning = $derived(restoreDatabaseRunning || restoreDesignsRunning);
  let restoreSchemaVersion = $state(null);
  let restorePreviousSchemaVersion = $state(null);
  let restoreRolledBack = $state(false);
  let restoreError = $state("");
  // Unmatched-files (post-restore reconciliation) prompt.
  let showUnmatchedPrompt = $state(false);
  let unmatchedCount = $state(0);
  let unmatchedChecked = $state(0);
  let unmatchedSample = $state([]);
  let importingUnmatched = $state(false);
  /** @type {import("@tauri-apps/api/event").UnlistenFn | null} */
  let unlistenRestore = $state(null);

  let restoreSchemaChanged = $derived(
    restoreSchemaVersion !== null &&
      restorePreviousSchemaVersion !== null &&
      Number(restoreSchemaVersion) !== Number(restorePreviousSchemaVersion)
  );

  let backupHasUnsavedChanges = $derived(
    backupDbDestination.trim() !== backupSavedDbDestination.trim() ||
      backupDesignsDestination.trim() !== backupSavedDesignsDestination.trim()
  );
  let backupHasDbDestination = $derived(backupSavedDbDestination.trim().length > 0);
  let backupHasDesignsDestination = $derived(backupSavedDesignsDestination.trim().length > 0);
  let backupAnyRunning = $derived(backupDatabaseRunning || backupDesignsRunning);

  async function loadBackupFromBackend(force = false) {
    if (backupLoading) return;
    if (backupLoaded && !force) return;

    backupLoading = true;
    try {
      // Also get settings data root for fallback
      try {
        const settingsRes = await getSettingsViewModel();
        settingsDataRoot = String(settingsRes?.model?.data_root || "");
      } catch (e) {
        console.error("Could not load settings in backup view", e);
      }

      const result = await getBackupViewModel();
      const model = result?.model;
      backupDbDestination = String(model?.db_destination || "");
      backupDesignsDestination = String(model?.designs_destination || "");
      backupSavedDbDestination = backupDbDestination;
      backupSavedDesignsDestination = backupDesignsDestination;
      restoreDesignsSource = backupSavedDesignsDestination;

      const fallbackDataRoot = settingsDataRoot ? String(settingsDataRoot) : "";
      backupDbSourcePath = String(
        model?.db_source_path ||
          (fallbackDataRoot ? `${fallbackDataRoot}\\database\\catalogue.db` : "(not available yet)")
      );
      backupDesignsSourcePath = String(
        model?.designs_source_path ||
          (fallbackDataRoot
            ? `${fallbackDataRoot}\\MachineEmbroideryDesigns`
            : "(not available yet)")
      );

      backupLoaded = true;
    } catch (error) {
      addToast(`Could not load backup settings: ${error}`, "error");
    } finally {
      backupLoading = false;
    }
  }

  /** @param {"database" | "designs"} kind */
  async function browseBackupDestination(kind) {
    const startDir = kind === "database" ? backupDbDestination : backupDesignsDestination;
    const result = await browseBackupFolder(startDir);

    if (result.path) {
      if (kind === "database") {
        backupDbDestination = result.path;
      } else {
        backupDesignsDestination = result.path;
      }
      return;
    }

    if (result.error) {
      addToast(result.error, "error");
    }
  }

  /** @param {SubmitEvent} event */
  async function saveBackupDestinations(event) {
    event.preventDefault();

    if (!backupHasUnsavedChanges) {
      addToast("There are no destination changes to save.", "error");
      return;
    }

    const result = await saveBackupSettings({
      dbDestination: backupDbDestination,
      designsDestination: backupDesignsDestination,
    });

    if (result.saved) {
      backupSavedDbDestination = String(result.db_destination || backupDbDestination).trim();
      backupSavedDesignsDestination = String(
        result.designs_destination || backupDesignsDestination
      ).trim();
      backupDbDestination = backupSavedDbDestination;
      backupDesignsDestination = backupSavedDesignsDestination;
      addToast(result.message || "Backup destinations saved.", "success");
      return;
    }

    addToast(result.message || "Could not save backup destinations.", "error");
  }

  /** @param {"database" | "designs" | "both"} action */
  async function runBackupAction(action) {
    if (backupAnyRunning) return;

    if (action === "database" && !backupHasDbDestination) {
      addToast(
        "No database backup destination is configured. Please set one below and save destinations.",
        "error"
      );
      return;
    }

    if (action === "designs" && !backupHasDesignsDestination) {
      addToast(
        "No designs backup destination is configured. Please set one below and save destinations.",
        "error"
      );
      return;
    }

    if (action === "both" && (!backupHasDbDestination || !backupHasDesignsDestination)) {
      addToast(
        "Both backup destinations must be configured before you can run both backups.",
        "error"
      );
      return;
    }

    const runsDatabase = action === "database" || action === "both";
    const runsDesigns = action === "designs" || action === "both";
    if (runsDatabase) backupDatabaseRunning = true;
    if (runsDesigns) backupDesignsRunning = true;
    activeBackupAction = action;

    try {
      if (action === "database") {
        const result = await runDatabaseBackup();
        if (result.cancelled) {
          addToast(
            "Database backup cancelled. The partial backup file was removed.",
            "warning"
          );
          return;
        }
        if (!result.success) {
          addToast(result.error || "Database backup failed.", "error");
          return;
        }

        const mb = (Number(result.size_bytes || 0) / (1024 * 1024)).toFixed(2);
        addToast(
          `Database backup created: ${result.backup_path || "(path unavailable)"} (${mb} MB).`,
          "success"
        );
        return;
      }

      if (action === "designs") {
        const result = await runDesignsBackup();
        if (result.cancelled) {
          addToast(
            "Designs backup cancelled. Already copied files were kept.",
            "info"
          );
          return;
        }
        if (!result.success) {
          addToast(result.error || "Designs backup failed.", "error");
          return;
        }

        addToast(
          `Designs backup complete: scanned ${result.scanned}, copied ${result.copied}, updated ${result.updated}, unchanged ${result.unchanged}, archived ${result.archived}.`,
          "success"
        );
        return;
      }

      const result = await runBothBackups();
      const dbCancelled = Boolean(result?.database?.cancelled);
      const designsCancelled = Boolean(result?.designs?.cancelled);

      // Prefer explicit cancellation messaging when either phase was cancelled.
      if (dbCancelled && designsCancelled) {
        addToast(
          "Backup cancelled. Partially created database backup files were removed; already copied design files were kept.",
          "warning"
        );
        return;
      }

      if (dbCancelled) {
        addToast(
          "Database backup cancelled. The partial backup file was removed; design files already copied were kept.",
          "warning"
        );
        return;
      }

      if (designsCancelled) {
        addToast(
          "Designs backup cancelled. Already copied design files were kept.",
          "warning"
        );
        return;
      }

      const dbOk = Boolean(result?.database?.success);
      const designsOk = Boolean(result?.designs?.success);

      if (dbOk && designsOk) {
        addToast("Both backups completed successfully.", "success");
        return;
      }

      const dbError = String(result?.database?.error || "").trim();
      const designsError = String(result?.designs?.error || "").trim();
      addToast(
        `Backup results: database ${dbOk ? "ok" : "failed"}${dbError ? ` (${dbError})` : ""}; designs ${designsOk ? "ok" : "failed"}${designsError ? ` (${designsError})` : ""}.`,
        "error"
      );
    } finally {
      if (runsDatabase) backupDatabaseRunning = false;
      if (runsDesigns) backupDesignsRunning = false;
      activeBackupAction = null;
      showCancelConfirm = false;
      cancelling = false;
    }
  }

  /** Open the confirmation modal when the user clicks "Cancel Backup". */
  function requestCancel() {
    if (!backupAnyRunning) return;
    showCancelConfirm = true;
  }

  /** User confirmed cancellation: raise the backend flag (non-blocking). */
  async function confirmCancel() {
    if (!backupAnyRunning || cancelling) return;
    showCancelConfirm = false;
    cancelling = true;
    try {
      const result = await requestCancelBackup();
      if (!result.cancel_requested) {
        addToast("Could not request backup cancellation.", "error");
        cancelling = false;
      }
      // On success the in-flight run command resolves with cancelled: true and
      // its finally block clears `cancelling`. Nothing else to do here.
    } catch (error) {
      addToast(`Could not request backup cancellation: ${error}`, "error");
      cancelling = false;
    }
  }

  function closeCancelModal() {
    showCancelConfirm = false;
  }

  // ── Restore handlers ─────────────────────────────────────────────────────

  /** Open the database backup file picker, defaulting to the configured folder. */
  async function chooseRestoreDbFile() {
    const startDir = restoreDbFile || backupSavedDbDestination || "";
    const result = await browseRestoreFile(startDir);
    if (result.path) {
      restoreDbFile = result.path;
      return;
    }
    if (result.error) {
      addToast(result.error, "error");
    }
  }

  /** Show the schema-change warning after a successful database restore. */
  function applyRestoreDatabaseResult(result) {
    restoreSchemaVersion = result?.schema_version_hint ?? null;
    restorePreviousSchemaVersion = result?.previous_schema_version_hint ?? null;
    restoreRolledBack = Boolean(result?.rolled_back);

    if (result?.rolled_back) {
      addToast(result?.error || "Database restore failed and was rolled back.", "error", true);
      return;
    }
    if (!result?.success) {
      addToast(result?.error || "Database restore failed.", "error", true);
      return;
    }
    const count = Number(result?.design_count || 0).toLocaleString();
    addToast(`Database restored (${count} designs).`, "success");
  }

  /** Run a restore action ("database", "designs", or "both"). */
  async function runRestoreAction(action) {
    if (restoreAnyRunning) return;

    const runsDatabase = action === "database" || action === "both";
    const runsDesigns = action === "designs" || action === "both";

    if (runsDatabase && !restoreDbFile.trim()) {
      addToast("Choose a database backup (.db) file first.", "error");
      return;
    }
    if (runsDesigns && !restoreDesignsSource.trim() && !backupHasDesignsDestination) {
      addToast(
        "No designs backup folder is configured. Save a designs backup destination first.",
        "error"
      );
      return;
    }

    if (runsDatabase) restoreDatabaseRunning = true;
    if (runsDesigns) restoreDesignsRunning = true;
    restoreError = "";
    restoreRolledBack = false;
    restoreSchemaVersion = null;
    restorePreviousSchemaVersion = null;
    showUnmatchedPrompt = false;
    resetRestoreProgress();

    try {
      if (action === "database") {
        const result = await restoreDatabase(restoreDbFile.trim());
        applyRestoreDatabaseResult(result);
        return;
      }

      if (action === "designs") {
        const result = await restoreDesignsIncremental({
          designsSourceDir: restoreDesignsSource.trim() || undefined,
        });
        if (!result.success) {
          addToast(result.error || "Designs restore failed.", "error");
          return;
        }
        addToast(
          `Designs restored: ${result.copied} copied, ${result.skipped} skipped.`,
          "success"
        );
        return;
      }

      if (action === "both") {
        const result = await restoreBoth(restoreDbFile.trim(), {
          designsSourceDir: restoreDesignsSource.trim() || undefined,
        });
        if (!result?.database?.success) {
          applyRestoreDatabaseResult(result?.database);
          return;
        }
        applyRestoreDatabaseResult(result.database);
        if (result?.designs?.success) {
          addToast(
            `Designs restored: ${result.designs.copied} copied, ${result.designs.skipped} skipped.`,
            "success"
          );
        }
        const unmatched = result?.unmatched;
        if (unmatched && Number(unmatched.unmatched) > 0) {
          unmatchedCount = Number(unmatched.unmatched) || 0;
          unmatchedChecked = Number(unmatched.checked) || 0;
          unmatchedSample = Array.isArray(unmatched.sample) ? unmatched.sample : [];
          showUnmatchedPrompt = true;
        }
        return;
      }
    } catch (error) {
      restoreError = String(error);
      addToast(`Restore failed: ${error}`, "error", true);
    } finally {
      if (runsDatabase) restoreDatabaseRunning = false;
      if (runsDesigns) restoreDesignsRunning = false;
    }
  }

  /** Batch-import unmatched design files as new catalogue records. */
  async function handleImportUnmatched() {
    if (importingUnmatched) return;
    importingUnmatched = true;
    try {
      const result = await importUnmatchedDesignFiles();
      const message =
        result.failed > 0
          ? `Imported ${result.imported} file(s); ${result.failed} failed.`
          : `Imported ${result.imported} unmatched file(s).`;
      addToast(message, result.failed > 0 ? "warning" : "success");
      showUnmatchedPrompt = false;
    } catch (error) {
      addToast(`Import failed: ${error}`, "error");
    } finally {
      importingUnmatched = false;
    }
  }

  function dismissUnmatched() {
    showUnmatchedPrompt = false;
    unmatchedCount = 0;
    unmatchedChecked = 0;
    unmatchedSample = [];
  }

  onMount(async () => {
    loadBackupFromBackend();
    try {
      unlistenRestore = await initRestoreProgressEvents();
    } catch (error) {
      console.info("Restore progress events unavailable.", error);
    }
  });

  onDestroy(() => {
    if (unlistenRestore) {
      unlistenRestore();
    }
    resetRestoreProgress();
  });
</script>

<section class="backup-page space-y-4">
  <h1 class="ui-page-title backup-title mb-2">Backup</h1>
  <p class="text-sm text-gray-500 mb-4">
    Back up your catalogue database and embroidery design files to folders of your choice. The
    database backup saves your catalogue data, settings, tags, and projects. The designs backup
    saves the actual embroidery files.
  </p>

  <div
    class="backup-important mb-2 bg-amber-50 border border-amber-300 text-amber-900 rounded px-4 py-3 text-sm space-y-1"
  >
    <p class="font-semibold">Important</p>
    <p>
      Ensure backup folders reside on a separate drive from your library.
    </p>
  </div>

  <div class="settings-layout max-w-3xl space-y-6">
    <form
      class="settings-card backup-card bg-white rounded shadow p-6 space-y-5"
      onsubmit={saveBackupDestinations}
    >
      <h2 class="text-base font-semibold text-gray-800">Backup Destinations</h2>
      <p class="text-sm text-gray-600">
        Set separate destination folders for the database and designs backups.
      </p>

      <div>
        <label for="backup-db-destination" class="block text-sm font-semibold text-gray-700 mb-1"
          >Database backup folder</label
        >
        <div class="flex gap-2">
          <input
            id="backup-db-destination"
            type="text"
            bind:value={backupDbDestination}
            placeholder="e.g. C:\\Backups\\EmbroideryDB"
            spellcheck="false"
            class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
          />
          <button
            type="button"
            class="settings-secondary-button border rounded px-3 py-2 text-sm whitespace-nowrap"
            onclick={() => browseBackupDestination("database")}
          >
            Browse…
          </button>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          This backup contains your catalogue data only - <strong>not</strong> the embroidery design files.
        </p>
      </div>

      <div>
        <label
          for="backup-designs-destination"
          class="block text-sm font-semibold text-gray-700 mb-1">Designs backup folder</label
        >
        <div class="flex gap-2">
          <input
            id="backup-designs-destination"
            type="text"
            bind:value={backupDesignsDestination}
            placeholder="e.g. C:\\Backups\\EmbroideryDesigns"
            spellcheck="false"
            class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
          />
          <button
            type="button"
            class="settings-secondary-button border rounded px-3 py-2 text-sm whitespace-nowrap"
            onclick={() => browseBackupDestination("designs")}
          >
            Browse…
          </button>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          This backup contains file copies only - <strong>not</strong> the catalogue database.
        </p>
      </div>

      <div class="flex justify-end">
        <button
          type="submit"
          class="settings-primary-button menu-button-primary"
          disabled={!backupHasUnsavedChanges}
          title={!backupHasUnsavedChanges ? "No unsaved destination changes" : undefined}
        >
          Save destinations
        </button>
      </div>
    </form>

    <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">Database Backup</h2>
      <p class="text-sm text-gray-600">
        Creates a timestamped copy of your SQLite database catalogue file.
      </p>
      <div class="text-xs text-gray-500 space-y-0.5">
        <p>
          Source: <code class="settings-code inline-block border rounded px-2 py-1 font-mono"
            >{backupDbSourcePath}</code
          >
        </p>
        <p>
          Saved destination folder: <code
            class="settings-code inline-block border rounded px-2 py-1 font-mono"
            >{backupSavedDbDestination || "(not set)"}</code
          >
        </p>
      </div>

      {#if backupAnyRunning}
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled
          title="A backup is already running"
        >
          {backupDatabaseRunning ? "Backing up database..." : "Database backup idle"}
        </button>
      {:else}
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled={!backupHasDbDestination}
          title={!backupHasDbDestination ? "Set a database backup destination first" : undefined}
          onclick={() => runBackupAction("database")}
        >
          Backup database now
        </button>
      {/if}
    </div>

    <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">Designs Backup</h2>
      <p class="text-sm text-gray-600">
        Runs an incremental mirror backup of the designs folder. Only new or changed files are
        copied; unchanged files are skipped.
      </p>
      <div class="text-xs text-gray-500 space-y-0.5">
        <p>
          Source: <code class="settings-code inline-block border rounded px-2 py-1 font-mono"
            >{backupDesignsSourcePath}</code
          >
        </p>
        <p>
          Saved destination folder: <code
            class="settings-code inline-block border rounded px-2 py-1 font-mono"
            >{backupSavedDesignsDestination || "(not set)"}</code
          >
        </p>
      </div>

      {#if backupAnyRunning}
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled
          title="A backup is already running"
        >
          {backupDesignsRunning ? "Running incremental backup..." : "Designs backup idle"}
        </button>
      {:else}
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled={!backupHasDesignsDestination}
          title={!backupHasDesignsDestination ? "Set a designs backup destination first" : undefined}
          onclick={() => runBackupAction("designs")}
        >
          Run incremental backup
        </button>
      {/if}
    </div>

    <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">Backup Both</h2>
      <p class="text-sm text-gray-600">
        Run the database backup and the incremental designs backup in one step.
      </p>

      {#if backupAnyRunning}
        <button
          type="button"
          class="menu-button-primary"
          style="background-color:#dc2626;border-color:#dc2626;"
          disabled={cancelling}
          onclick={requestCancel}
          data-testid="cancel-backup-button"
        >
          {cancelling ? "Cancelling..." : "Cancel Backup"}
        </button>
      {:else}
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled={!backupHasDbDestination || !backupHasDesignsDestination}
          title={!backupHasDbDestination || !backupHasDesignsDestination
            ? "Set both backup destinations first"
            : undefined}
          onclick={() => runBackupAction("both")}
        >
          Run both backups
        </button>
      {/if}
    </div>
  </div>

  <CancelBackupModal
    open={showCancelConfirm}
    activeKind={activeBackupAction}
    onClose={closeCancelModal}
    onConfirm={confirmCancel}
  />

  <div class="border-t border-gray-200 pt-6 mt-2">
    <h2 class="text-lg font-semibold text-gray-800 mb-1">Restore</h2>
    <p class="text-sm text-gray-500 mb-4">
      Restore a database backup snapshot and/or sync design files back from a backup folder. A safety
      copy of your current database is kept before any overwrite.
    </p>
  </div>

  <RestoreProgressPanel />

  {#if restoreSchemaChanged}
    <div
      class="backup-important mb-2 bg-amber-50 border border-amber-300 text-amber-900 rounded px-4 py-3 text-sm space-y-1"
    >
      <p class="font-semibold">Schema version changed</p>
      <p>
        The restored database reports schema version {restoreSchemaVersion} (previous was
        {restorePreviousSchemaVersion}). No automated migrations were run; if this backup is from an
        older version of the app, some newer features may be unavailable.
      </p>
    </div>
  {/if}

  {#if restoreRolledBack}
    <div
      class="backup-important mb-2 bg-red-50 border border-red-300 text-red-900 rounded px-4 py-3 text-sm"
    >
      <p class="font-semibold">Restore rolled back</p>
      <p>Your previous database was automatically restored after the restore failed verification.</p>
    </div>
  {/if}

  <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
    <h2 class="text-base font-semibold text-gray-800">Restore Database</h2>
    <p class="text-sm text-gray-600">
      Replace the live catalogue database with a backup snapshot. A safety copy of the current
      database is kept before overwriting.
    </p>
    <div class="flex gap-2 items-center">
      <input
        id="restore-db-file"
        type="text"
        readonly
        bind:value={restoreDbFile}
        placeholder="Choose an EmbroideryCatalogue.db backup file…"
        spellcheck="false"
        class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
      />
      <button
        type="button"
        class="settings-secondary-button border rounded px-3 py-2 text-sm whitespace-nowrap"
        onclick={chooseRestoreDbFile}
        disabled={restoreAnyRunning}
      >
        Choose file…
      </button>
    </div>
    <div class="text-xs text-gray-500">
      <p>
        Defaults to:
        <code class="settings-code inline-block border rounded px-2 py-1 font-mono"
          >{backupSavedDbDestination || "(not set)"}</code
        >
      </p>
    </div>
    {#if restoreAnyRunning}
      <button type="button" class="settings-primary-button menu-button-primary" disabled
        title="A restore is already running"
        >Restore database idle</button
      >
    {:else}
      <button
        type="button"
        class="settings-primary-button menu-button-primary"
        disabled={!restoreDbFile.trim()}
        title={!restoreDbFile.trim() ? "Choose a database backup file first" : undefined}
        onclick={() => runRestoreAction("database")}
      >
        Restore database now
      </button>
    {/if}
  </div>

  <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
    <h2 class="text-base font-semibold text-gray-800">Sync Designs from Backup</h2>
    <p class="text-sm text-gray-600">
      Copies design <strong>files</strong> from the backup folder back into
      <code>MachineEmbroideryDesigns</code>, skipping files already present there (same size and
      timestamp). This restores <strong>files only</strong> — it does <strong>not</strong> add or
      change database records.
    </p>
    <div class="text-xs text-gray-500 space-y-0.5">
      <p>
        Backup source folder:
        <code class="settings-code inline-block border rounded px-2 py-1 font-mono"
          >{restoreDesignsSource || backupSavedDesignsDestination || "(not set)"}</code
        >
      </p>
    </div>
    {#if restoreAnyRunning}
      <button type="button" class="settings-primary-button menu-button-primary" disabled
        title="A restore is already running"
        >Sync designs idle</button
      >
    {:else}
      <button
        type="button"
        class="settings-primary-button menu-button-primary"
        disabled={!restoreDesignsSource.trim() && !backupHasDesignsDestination}
        title={
          !restoreDesignsSource.trim() && !backupHasDesignsDestination
            ? "Set a designs backup folder first"
            : undefined
        }
        onclick={() => runRestoreAction("designs")}
      >
        Sync designs from backup
      </button>
    {/if}
  </div>

  <div class="settings-card backup-card bg-white rounded shadow p-6 space-y-4">
    <h2 class="text-base font-semibold text-gray-800">Restore Both</h2>
    <p class="text-sm text-gray-600">
      Restore the database, then sync design files, and finally check for design files on disk that have no
      database record (you can import those afterwards).
    </p>
    {#if restoreAnyRunning}
      <button type="button" class="settings-primary-button menu-button-primary" disabled
        title="A restore is already running"
        >Restore both idle</button
      >
    {:else}
      <button
        type="button"
        class="settings-primary-button menu-button-primary"
        disabled={!restoreDbFile.trim() || (!restoreDesignsSource.trim() && !backupHasDesignsDestination)}
        title={!restoreDbFile.trim() ? "Choose a database backup file first" : undefined}
        onclick={() => runRestoreAction("both")}
      >
        Restore both
      </button>
    {/if}
  </div>

  {#if showUnmatchedPrompt}
    <div
      class="settings-card backup-card bg-white rounded shadow p-6 space-y-3"
      data-testid="unmatched-files-prompt"
    >
      <h2 class="text-base font-semibold text-gray-800">Unmatched files found</h2>
      <p class="text-sm text-gray-600">
        {unmatchedCount} design file(s) on disk have no record in the restored database
        {unmatchedChecked > 0 ? `(scanned ${unmatchedChecked})` : ""}. You can import them as new
        catalogue records.
      </p>
      {#if unmatchedSample.length > 0}
        <ul class="text-xs text-gray-500 list-disc pl-5 space-y-0.5 max-h-32 overflow-auto">
          {#each unmatchedSample as path}
            <li class="font-mono break-all">{path}</li>
          {/each}
        </ul>
      {/if}
      <div class="flex gap-2 pt-1">
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          disabled={importingUnmatched}
          onclick={handleImportUnmatched}
        >
          {importingUnmatched ? "Importing…" : `Import ${unmatchedCount} file(s)`}
        </button>
        <button type="button" class="menu-button-secondary" onclick={dismissUnmatched}>Dismiss</button>
      </div>
    </div>
  {/if}
</section>