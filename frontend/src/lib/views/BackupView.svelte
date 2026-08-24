<script>
  import { onMount } from "svelte";
  import {
    getBackupViewModel,
    saveBackupSettings,
    browseBackupFolder,
    runDatabaseBackup,
    runDesignsBackup,
    runBothBackups,
    requestCancelBackup,
    getSettingsViewModel,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import CancelBackupModal from "../components/CancelBackupModal.svelte";

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

  onMount(() => {
    loadBackupFromBackend();
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
</section>