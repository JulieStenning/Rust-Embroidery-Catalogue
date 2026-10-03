<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount } from "svelte";
  import {
    browseRestoreFile,
    browseSettingsDataRoot,
    detectRelocatedDataRoot,
    getDatabaseStatus,
    recoverDatabaseFromBackup,
    restartApplication,
    seedDatabaseToDataRoot,
    setConfiguredDataRoot,
    validateDatabasePath,
  } from "./api/commandAdapter";

  /** @typedef {import("./types/ipc").DetectedDataRoot} DetectedDataRoot */
  /** @typedef {import("./types/ipc").DatabaseValidation} DatabaseValidation */

  let configuredRoot = $state("");
  let newCataloguePath = $state("");
  let relocatedRoot = $state("");
  let isCorrupted = $state(false);
  let dbErrorMessage = $state("");
  let selectedBackupPath = $state("");
  let scanning = $state(true);
  let busy = $state(false);
  let error = $state("");
  let createError = $state("");
  let restoreError = $state("");
  let validationMessage = $state("");
  let validationIsError = $state(false);
  let showSeedConfirm = $state(false);
  let showRestoreConfirm = $state(false);
  let showRestartConfirm = $state(false);
  let restarting = $state(false);

  onMount(async () => {
    const dbStatus = await getDatabaseStatus();
    if (dbStatus.status) {
      configuredRoot = String(dbStatus.status.configured_data_root || "");
      newCataloguePath = configuredRoot;
      isCorrupted = dbStatus.status.status === "corrupted";
      dbErrorMessage = String(dbStatus.status.error_message || "");
    }

    // Try the drive-letter relocation quick fix (e.g. D: -> E:) if not found.
    if (configuredRoot && !isCorrupted) {
      const detected = await detectRelocatedDataRoot(configuredRoot);
      if (detected && !detected.error && detected.detected) {
        relocatedRoot = String(detected.detected.data_root || "");
      }
    }
    scanning = false;
  });

  /** Open a native folder picker and validate the chosen catalogue root. */
  async function handleBrowse() {
    if (busy) return;
    busy = true;
    error = "";
    validationMessage = "";
    validationIsError = false;
    try {
      const picked = await browseSettingsDataRoot(configuredRoot);
      if (!picked || !picked.path) return;
      await validateCandidate(picked.path);
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  /** One-click re-connect to the detected relocated root. */
  async function handleReconnect() {
    if (busy || !relocatedRoot) return;
    busy = true;
    error = "";
    validationMessage = "";
    validationIsError = false;
    try {
      await validateCandidate(relocatedRoot);
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  /** Persist the candidate root after it validates. */
  /** @param {string} candidateRoot */
  async function validateCandidate(candidateRoot) {
    const validation = await validateDatabasePath(candidateRoot);
    if (!validation || !validation.validation) {
      throw new Error("Could not validate the selected location.");
    }
    const result = validation.validation;
    if (!result.valid) {
      validationMessage =
        result.error || "This folder does not look like an Embroidery Catalogue data location.";
      validationIsError = true;
      return;
    }
    if (!result.embroidery_dir_exists) {
      validationMessage =
        "Database found, but the MachineEmbroideryDesigns folder is missing — design files may need re-pointing.";
      validationIsError = true;
    } else {
      validationMessage = "";
      validationIsError = false;
    }
    // Persist the new data root; the app must restart to mount it.
    configuredRoot = result.data_root;
    const saved = await setConfiguredDataRoot(result.data_root);
    if (!saved || !saved.persisted) {
      throw new Error(saved?.error || "Could not save the new data location.");
    }
    showRestartConfirm = true;
  }

  /** Open a native file picker to select a .db backup file. */
  async function handleBrowseBackup() {
    if (busy) return;
    error = "";
    restoreError = "";
    try {
      const picked = await browseRestoreFile(configuredRoot);
      if (picked && picked.path) {
        selectedBackupPath = picked.path;
        showRestoreConfirm = true;
      }
    } catch (err) {
      error = String(err);
    }
  }

  /** Recover from the chosen database backup file. */
  async function handleConfirmRestore() {
    if (busy || !selectedBackupPath) return;
    busy = true;
    restoreError = "";
    try {
      const targetRoot = configuredRoot || newCataloguePath;
      if (!targetRoot) {
        throw new Error("Catalogue data location is not set.");
      }
      const outcome = await recoverDatabaseFromBackup(targetRoot, selectedBackupPath);
      if (!outcome || !outcome.restored) {
        throw new Error(outcome?.error || "Failed to recover from the selected backup file.");
      }
      const saved = await setConfiguredDataRoot(targetRoot);
      if (!saved || !saved.persisted) {
        throw new Error(saved?.error || "Could not save the catalogue location.");
      }
      showRestoreConfirm = false;
      showRestartConfirm = true;
    } catch (err) {
      restoreError = String(err);
    } finally {
      busy = false;
    }
  }

  /** Open the modal for creating a new empty catalogue. */
  function openCreateModal() {
    createError = "";
    if (!newCataloguePath && configuredRoot) {
      newCataloguePath = configuredRoot;
    }
    showSeedConfirm = true;
  }

  /** Open a native folder picker for the new catalogue location. */
  async function handleBrowseNewLocation() {
    if (busy) return;
    createError = "";
    try {
      const picked = await browseSettingsDataRoot(newCataloguePath || configuredRoot);
      if (picked && picked.path) {
        newCataloguePath = picked.path;
      }
    } catch (err) {
      createError = String(err);
    }
  }

  /** Create a fresh empty catalogue at the chosen target root (guarded). */
  async function handleCreateNew() {
    const target = newCataloguePath.trim();
    if (busy || !target) {
      createError = "Please specify a location for the new catalogue.";
      return;
    }
    busy = true;
    createError = "";
    try {
      const seeded = await seedDatabaseToDataRoot(target, true);
      if (!seeded || !seeded.persisted) {
        throw new Error(seeded?.error || "Could not create a new catalogue.");
      }
      const saved = await setConfiguredDataRoot(target);
      if (!saved || !saved.persisted) {
        throw new Error(saved?.error || "Could not save the new catalogue location.");
      }
      configuredRoot = target;
      showSeedConfirm = false;
      showRestartConfirm = true;
    } catch (err) {
      createError = String(err);
    } finally {
      busy = false;
    }
  }

  /** Launch the application restart after the user confirms. */
  async function handleRestart() {
    if (restarting) return;
    restarting = true;
    error = "";
    const res = await restartApplication();
    if (!res || !res.restarted) {
      error =
        res?.error || "Could not restart the application. Please close and reopen it manually.";
      restarting = false;
      showRestartConfirm = false;
    }
  }
</script>

<div
  class="flex items-center justify-center min-h-screen bg-gray-50 p-4"
  data-testid="database-recovery-view"
>
  <div class="max-w-lg w-full space-y-4">
    <div class="route-card bg-white rounded-xl shadow p-6 space-y-4">
      {#if isCorrupted}
        <div class="space-y-2">
          <div class="flex items-center gap-2">
            <span class="text-xl">⚠️</span>
            <h1 class="text-xl font-bold text-gray-800">Your catalogue database is unreadable</h1>
          </div>
          <p class="text-sm text-gray-600">
            Embroidery Catalogue found a database file
            {#if configuredRoot}
              at <code class="font-mono text-xs bg-gray-100 px-1 rounded">{configuredRoot}</code>
            {/if}, but could not open it (it may be damaged or malformed). Your original embroidery
            design files in
            <code class="font-mono text-xs">MachineEmbroideryDesigns</code> are safe.
          </p>
          {#if dbErrorMessage}
            <p class="text-xs text-red-600 font-mono bg-red-50 p-2 rounded">
              {dbErrorMessage}
            </p>
          {/if}
        </div>
      {:else}
        <h1 class="text-xl font-bold text-gray-800">Your catalogue database could not be found</h1>

        {#if scanning}
          <p class="text-sm text-gray-500">Checking for your catalogue…</p>
        {:else}
          <p class="text-sm text-gray-600">
            Embroidery Catalogue could not find its database
            {#if configuredRoot}
              at <code class="font-mono text-xs bg-gray-100 px-1 rounded">{configuredRoot}</code>
            {/if}. This usually happens when a portable drive changes letter (for example from
            <code class="font-mono text-xs">D:</code>
            to <code class="font-mono text-xs">E:</code>) or the data folder was moved. Your
            original files are safe — choose how to continue.
          </p>
        {/if}
      {/if}

      {#if !scanning}
        {#if error}
          <div
            class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm"
            data-testid="recovery-error"
          >
            {error}
          </div>
        {/if}

        {#if validationMessage}
          <div
            class="{validationIsError
              ? 'bg-amber-50 border-amber-300 text-amber-800'
              : 'bg-green-50 border-green-300 text-green-800'} border rounded px-3 py-2 text-sm"
            data-testid="recovery-validation"
          >
            {validationMessage}
          </div>
        {/if}

        {#if relocatedRoot}
          <div
            class="bg-indigo-50 border border-indigo-200 text-indigo-900 rounded-lg px-4 py-3 text-sm space-y-2"
          >
            <p class="font-semibold">We found your catalogue on another drive!</p>
            <p class="text-xs">
              A copy of your catalogue appears to be at
              <code class="font-mono text-xs bg-indigo-100 px-1 rounded">{relocatedRoot}</code>.
              Reconnect to it with one click.
            </p>
            <button
              type="button"
              onclick={handleReconnect}
              disabled={busy}
              class="bg-indigo-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-indigo-700 disabled:opacity-50"
              data-testid="recovery-reconnect"
            >
              Re-connect to this location
            </button>
          </div>
        {/if}

        <div class="space-y-3 pt-2">
          <!-- Option 1: Restore from Backup -->
          <button
            type="button"
            onclick={handleBrowseBackup}
            disabled={busy}
            class="w-full bg-indigo-600 text-white px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-indigo-700 disabled:opacity-50 flex items-center justify-center gap-2"
            data-testid="recovery-restore-backup"
          >
            <span>📥</span>
            <span>Restore from a database backup…</span>
          </button>

          <!-- Option 2: Choose existing folder -->
          <button
            type="button"
            onclick={handleBrowse}
            disabled={busy}
            class="w-full bg-white border border-gray-300 text-gray-700 px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-gray-50 disabled:opacity-50 flex items-center justify-center gap-2"
            data-testid="recovery-browse"
          >
            <span>📁</span>
            <span>Choose another catalogue folder…</span>
          </button>

          <!-- Option 3: Fresh catalogue -->
          <button
            type="button"
            onclick={openCreateModal}
            disabled={busy}
            class="w-full bg-white border border-gray-300 text-gray-700 px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-gray-50 disabled:opacity-50 flex items-center justify-center gap-2"
            data-testid="recovery-create-new"
          >
            <span>✨</span>
            <span>Start fresh with a clean catalogue</span>
          </button>
        </div>

        <p class="text-xs text-gray-500 pt-1">
          If you start fresh or restore from backup, your current database file will be safely
          preserved with a
          <code class="font-mono text-xs">.corrupt-&lt;timestamp&gt;</code> extension so nothing is lost.
        </p>
      {/if}
    </div>
  </div>
</div>

{#if showRestoreConfirm}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Restore database from backup"
  >
    <div class="bg-white rounded-xl shadow-lg p-6 max-w-lg w-full space-y-4">
      <h2 class="text-lg font-semibold text-gray-800">Restore catalogue from backup</h2>
      <p class="text-sm text-gray-600">You are about to restore the database from:</p>
      <div
        class="bg-gray-50 border border-gray-200 p-3 rounded text-xs font-mono text-gray-800 break-all"
      >
        {selectedBackupPath}
      </div>
      <p class="text-xs text-gray-500">
        Your current database will be automatically moved aside to a safe backup file before the
        restore is applied.
      </p>

      {#if restoreError}
        <div
          class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm"
          data-testid="recovery-restore-error"
        >
          {restoreError}
        </div>
      {/if}

      <div class="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => {
            showRestoreConfirm = false;
            restoreError = "";
          }}
          disabled={busy}
          class="px-4 py-2 rounded text-sm font-medium text-gray-600 hover:bg-gray-100 disabled:opacity-50"
          data-testid="recovery-restore-cancel"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleConfirmRestore}
          disabled={busy}
          class="px-4 py-2 rounded text-sm font-medium bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
          data-testid="recovery-restore-confirm"
        >
          {#if busy}Restoring…{:else}Restore database{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showSeedConfirm}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Create new catalogue confirmation"
  >
    <div class="bg-white rounded-xl shadow-lg p-6 max-w-lg w-full space-y-4">
      <h2 class="text-lg font-semibold text-gray-800">Start fresh with a clean catalogue</h2>
      <p class="text-sm text-gray-600">
        This will install a clean catalogue database at your chosen location. Any existing database
        file will be safely preserved with a <code class="font-mono text-xs"
          >.corrupt-&lt;timestamp&gt;</code
        > extension.
      </p>

      <div class="space-y-2">
        <label for="recovery-location-input" class="block text-xs font-medium text-gray-700">
          Catalogue Location
        </label>
        <div class="flex gap-2">
          <input
            id="recovery-location-input"
            type="text"
            bind:value={newCataloguePath}
            disabled={busy}
            placeholder="e.g. C:\EmbroideryCatalogue"
            class="flex-1 px-3 py-2 border border-gray-300 rounded text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:bg-gray-100 disabled:text-gray-500"
            data-testid="recovery-new-location-input"
          />
          <button
            type="button"
            onclick={handleBrowseNewLocation}
            disabled={busy}
            class="px-3 py-2 bg-gray-100 border border-gray-300 text-gray-700 rounded text-sm font-medium hover:bg-gray-200 disabled:opacity-50"
            data-testid="recovery-new-location-browse"
          >
            Browse…
          </button>
        </div>
      </div>

      {#if createError}
        <div
          class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm"
          data-testid="recovery-create-error"
        >
          {createError}
        </div>
      {/if}

      <div class="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => {
            showSeedConfirm = false;
            createError = "";
          }}
          disabled={busy}
          class="px-4 py-2 rounded text-sm font-medium text-gray-600 hover:bg-gray-100 disabled:opacity-50"
          data-testid="recovery-create-cancel"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCreateNew}
          disabled={busy || !newCataloguePath.trim()}
          class="px-4 py-2 rounded text-sm font-medium bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
          data-testid="recovery-create-confirm"
        >
          Create clean catalogue
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showRestartConfirm}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Restart required"
  >
    <div class="bg-white rounded-xl shadow-lg p-6 max-w-sm w-full space-y-4">
      <h2 class="font-semibold text-gray-800">Restart required</h2>
      <p class="text-sm text-gray-600">
        Your catalogue has been updated. Embroidery Catalogue needs to restart so it can begin using <span
          class="font-medium">{configuredRoot}</span
        >.
      </p>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          onclick={handleRestart}
          disabled={restarting}
          class="px-4 py-2 rounded text-sm font-medium bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
          data-testid="recovery-restart-now"
        >
          {#if restarting}Restarting…{:else}Restart now{/if}
        </button>
      </div>
    </div>
  </div>
{/if}
