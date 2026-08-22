<script>
  import { onMount } from "svelte";
  import {
    browseSettingsDataRoot,
    detectRelocatedDataRoot,
    getDatabaseStatus,
    restartApplication,
    seedDatabaseToDataRoot,
    setConfiguredDataRoot,
    validateDatabasePath,
  } from "./api/commandAdapter";

  /** @typedef {import("./types/ipc").DetectedDataRoot} DetectedDataRoot */
  /** @typedef {import("./types/ipc").DatabaseValidation} DatabaseValidation */

  let configuredRoot = $state("");
  let relocatedRoot = $state("");
  let scanning = $state(true);
  let busy = $state(false);
  let error = $state("");
  let validationMessage = $state("");
  let validationIsError = $state(false);
  let showSeedConfirm = $state(false);
  let showRestartConfirm = $state(false);
  let restarting = $state(false);

  onMount(async () => {
    const dbStatus = await getDatabaseStatus();
    if (dbStatus.status) {
      configuredRoot = String(dbStatus.status.configured_data_root || "");
    }

    // Try the drive-letter relocation quick fix (e.g. D: -> E:).
    if (configuredRoot) {
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

  /** Create a fresh empty catalogue at the configured root (guarded). */
  async function handleCreateNew() {
    if (busy || !configuredRoot) return;
    busy = true;
    error = "";
    try {
      const seeded = await seedDatabaseToDataRoot(configuredRoot, false);
      if (!seeded || !seeded.persisted) {
        throw new Error(seeded?.error || "Could not create a new catalogue.");
      }
      showSeedConfirm = false;
      showRestartConfirm = true;
    } catch (err) {
      error = String(err);
      showSeedConfirm = false;
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
  class="flex items-center justify-center min-h-screen bg-gray-50"
  data-testid="database-recovery-view"
>
  <div class="max-w-lg w-full p-6 space-y-4">
    <div class="bg-white rounded-xl shadow p-6 space-y-4">
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
          to <code class="font-mono text-xs">E:</code>) or the data folder was moved. Your original
          files are safe — choose how to continue.
        </p>

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

        <div class="space-y-2">
          <button
            type="button"
            onclick={handleBrowse}
            disabled={busy}
            class="w-full bg-indigo-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-indigo-700 disabled:opacity-50"
            data-testid="recovery-browse"
          >
            Choose my catalogue folder…
          </button>

          <button
            type="button"
            onclick={() => (showSeedConfirm = true)}
            disabled={busy}
            class="w-full bg-white border border-gray-300 text-gray-700 px-4 py-2 rounded text-sm font-medium hover:bg-gray-50 disabled:opacity-50"
            data-testid="recovery-create-new"
          >
            Create a new empty catalogue
          </button>
        </div>

        <p class="text-xs text-gray-500">
          Choose <span class="font-semibold">"Create a new empty catalogue"</span> only if you are sure
          you do not have an existing catalogue to recover — it starts fresh.
        </p>
      {/if}
    </div>
  </div>
</div>

{#if showSeedConfirm}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-label="Create new catalogue confirmation"
  >
    <div class="bg-white rounded-xl shadow p-6 max-w-sm w-full space-y-4">
      <h2 class="font-semibold text-gray-800">Create a new empty catalogue?</h2>
      <p class="text-sm text-gray-600">
        This will create a fresh catalogue at
        <code class="font-mono text-xs bg-gray-100 px-1 rounded"
          >{configuredRoot || "the configured location"}</code
        >. It will not touch any existing database files unless you confirm overwrite. Only use this
        if you are certain you do not have an existing catalogue to recover.
      </p>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          onclick={() => (showSeedConfirm = false)}
          class="px-4 py-2 rounded text-sm font-medium text-gray-600 hover:bg-gray-100"
          data-testid="recovery-create-cancel"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCreateNew}
          disabled={busy}
          class="px-4 py-2 rounded text-sm font-medium bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
          data-testid="recovery-create-confirm"
        >
          Create new catalogue
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showRestartConfirm}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-label="Restart required"
  >
    <div class="bg-white rounded-xl shadow p-6 max-w-sm w-full space-y-4">
      <h2 class="font-semibold text-gray-800">Restart required</h2>
      <p class="text-sm text-gray-600">
        Your new catalogue location has been saved. Embroidery Catalogue needs to restart so it can
        begin using <span class="font-medium">{configuredRoot}</span>.
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
