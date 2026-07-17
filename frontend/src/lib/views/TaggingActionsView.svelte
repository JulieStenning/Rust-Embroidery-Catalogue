<script>
  import { onMount } from "svelte";
  import {
    getTaggingActionsViewModel,
    runUnifiedBackfill,
    stopUnifiedBackfill,
    getBackfillLogEntries,
    runStitchingBackfill
  } from "../api/commandAdapter.js";

  let taggingActionsLoaded = $state(false);
  let taggingActionsLoading = $state(false);
  let taggingRunInFlight = $state(false);
  let taggingHasGoogleApiKey = $state(false);
  let taggingTier2Default = $state(false);
  let taggingTier3Default = $state(false);
  let taggingBatchSize = $state("100");
  let taggingCommitEvery = $state("100");
  let taggingWorkers = $state("4");
  let taggingActionMode = $state("tag_untagged");
  let taggingRunTier2 = $state(false);
  let taggingRunTier3 = $state(false);
  let taggingRunStitching = $state(false);
  let taggingClearExistingStitching = $state(false);
  let taggingRunImages = $state(false);
  let taggingImageRedo = $state(false);
  let taggingUpgrade2dTo3d = $state(false);
  let taggingUsePreview3d = $state(true);
  let taggingRunColorCounts = $state(false);
  let taggingStatusType = $state("info");
  let taggingStatusMessage = $state("Configure actions, then run selected actions.");
  /** @type {{processed: number, errors: number, stopped: boolean, actions: string[], error?: string} | null} */
  let taggingLastSummary = $state(null);
  /** @type {Array<{level: string, message: string}>} */
  let taggingLogEntries = $state([]);

  let taggingCommitValue = $derived(Math.max(1, Number.parseInt(taggingCommitEvery, 10) || 100));
  let taggingBatchValue = $derived(Math.max(1, Number.parseInt(taggingBatchSize, 10) || 100));
  let taggingWorkersValue = $derived(Math.max(1, Number.parseInt(taggingWorkers, 10) || 4));

  /** @param {Record<string, any>} model */
  function applyTaggingViewModel(model) {
    taggingHasGoogleApiKey = Boolean(model?.has_google_api_key);
    taggingTier2Default = Boolean(model?.ai_tier2_auto);
    taggingTier3Default = Boolean(model?.ai_tier3_auto);

    const defaultBatch = Number(model?.default_batch_size ?? 100);
    const defaultCommit = Number(model?.default_commit_every ?? 100);
    const defaultWorkers = Number(model?.default_workers ?? 4);

    const configuredBatch = String(model?.ai_batch_size || "").trim();
    const configuredCommit = String(model?.import_commit_batch_size || "").trim();

    taggingBatchSize = configuredBatch || String(defaultBatch);
    taggingCommitEvery = configuredCommit || String(defaultCommit);
    taggingWorkers = String(defaultWorkers);
    taggingRunTier2 = taggingTier2Default && taggingHasGoogleApiKey;
    taggingRunTier3 = taggingTier3Default && taggingHasGoogleApiKey;
  }

  async function loadTaggingActionsViewModel(force = false) {
    if (taggingActionsLoading) return;
    if (taggingActionsLoaded && !force) return;

    taggingActionsLoading = true;
    try {
      const result = await getTaggingActionsViewModel();
      applyTaggingViewModel(result?.model || {});
      taggingActionsLoaded = true;
      taggingStatusType = "info";
      taggingStatusMessage = taggingHasGoogleApiKey
        ? "API key detected. Tier 2/3 options are available."
        : "No Google API key detected. Tier 2/3 calls are disabled.";
    } catch (error) {
      taggingStatusType = "error";
      taggingStatusMessage = `Could not load tagging action defaults: ${error}`;
    } finally {
      taggingActionsLoading = false;
    }
  }

  async function refreshBackfillLogEntries() {
    const result = await getBackfillLogEntries(10);
    taggingLogEntries = Array.isArray(result?.entries) ? result.entries : [];
  }

  /** @param {Event} event */
  async function runTaggingActions(event) {
    event?.preventDefault?.();
    if (taggingRunInFlight) return;

    const actions = {};
    actions.tagging = {
      enabled: true,
      action: taggingActionMode,
      tiers: [
        1,
        ...(taggingRunTier2 && taggingHasGoogleApiKey ? [2] : []),
        ...(taggingRunTier3 && taggingHasGoogleApiKey ? [3] : []),
      ],
    };

    if (taggingRunStitching) {
      actions.stitching = {
        enabled: true,
        clear_existing_stitching: taggingClearExistingStitching,
      };
    }

    if (taggingRunImages) {
      actions.images = {
        enabled: true,
        redo: taggingImageRedo,
        upgrade_2d_to_3d: taggingUpgrade2dTo3d && !taggingImageRedo,
        preview_3d: taggingUsePreview3d,
      };
    }

    if (taggingRunColorCounts) {
      actions.color_counts = { enabled: true };
    }

    taggingRunInFlight = true;
    taggingStatusType = "info";
    taggingStatusMessage = "Running selected actions...";

    try {
      const result = await runUnifiedBackfill({
        actions,
        batch_size: taggingBatchValue,
        commit_every: taggingCommitValue,
        workers: taggingWorkersValue,
        preview_3d: taggingUsePreview3d,
      });

      if (result?.error) {
        taggingStatusType = "error";
        taggingStatusMessage = `Backfill run failed: ${result.error}`;
      } else {
        taggingLastSummary = result;
        taggingStatusType = result.stopped ? "error" : (result.errors > 0 ? "error" : "success");
        taggingStatusMessage = `Processed ${result.processed} design(s), errors ${result.errors}, actions: ${(result.actions || []).join(", ") || "none"}.`;
      }
      await refreshBackfillLogEntries();
    } catch (error) {
      taggingStatusType = "error";
      taggingStatusMessage = `Backfill run failed: ${error}`;
    } finally {
      taggingRunInFlight = false;
    }
  }

  async function stopTaggingActionsRun() {
    const result = await stopUnifiedBackfill();
    taggingStatusType = "info";
    taggingStatusMessage = `Stop request sent: ${result?.status || "stopping"}.`;
  }

  async function runStitchingOnlyAction() {
    if (taggingRunInFlight) return;
    taggingRunInFlight = true;
    taggingStatusType = "info";
    taggingStatusMessage = "Running stitching-only backfill...";

    try {
      const result = await runStitchingBackfill({
        clearExistingStitching: taggingClearExistingStitching,
        batchSize: taggingBatchValue,
      });
      taggingLastSummary = result;
      taggingStatusType = result.errors > 0 ? "error" : "success";
      taggingStatusMessage = `Stitching backfill complete: processed ${result.processed}, errors ${result.errors}.`;
      await refreshBackfillLogEntries();
    } finally {
      taggingRunInFlight = false;
    }
  }

  onMount(() => {
    loadTaggingActionsViewModel();
    refreshBackfillLogEntries();
  });
</script>

<div class="space-y-4">
  <div class="space-y-1 font-sans">
    <h1 class="ui-page-title text-2xl font-bold text-gray-800">Tagging Actions</h1>
    <p class="text-sm tracking-wide text-indigo-600 font-semibold">Batch tagging placeholder</p>
    <p class="text-gray-600 text-sm">This page will expose tier controls and batch tagging execution options.</p>
  </div>

  <form class="settings-card settings-form bg-white rounded shadow p-6 space-y-5" onsubmit={runTaggingActions}>
    <div class="grid md:grid-cols-2 gap-4">
      <label class="settings-input">
        <span>Tagging mode</span>
        <select bind:value={taggingActionMode} class="border rounded px-2 py-1 text-sm">
          <option value="tag_untagged">Tag only untagged</option>
          <option value="retag_all_unverified">Tag untagged and unverified</option>
          <option value="retag_all">Re-tag ALL (including verified)</option>
        </select>
      </label>

      <div class="settings-input">
        <span>Tiers</span>
        <div class="flex flex-wrap gap-3 mt-1 text-sm">
          <label class="inline-flex items-center gap-2">
            <input type="checkbox" checked disabled />
            <span>Tier 1</span>
          </label>
          <label class="inline-flex items-center gap-2">
            <input type="checkbox" bind:checked={taggingRunTier2} disabled={!taggingHasGoogleApiKey} />
            <span>Tier 2</span>
          </label>
          <label class="inline-flex items-center gap-2">
            <input type="checkbox" bind:checked={taggingRunTier3} disabled={!taggingHasGoogleApiKey} />
            <span>Tier 3</span>
          </label>
        </div>
      </div>
    </div>

    <div class="grid md:grid-cols-3 gap-4">
      <label class="settings-input">
        <span>Batch size</span>
        <input type="number" min="1" step="1" bind:value={taggingBatchSize} class="border rounded px-2 py-1 text-sm" />
      </label>
      <label class="settings-input">
        <span>Commit every</span>
        <input type="number" min="1" step="1" bind:value={taggingCommitEvery} class="border rounded px-2 py-1 text-sm" />
      </label>
      <label class="settings-input">
        <span>Workers</span>
        <input type="number" min="1" step="1" bind:value={taggingWorkers} class="border rounded px-2 py-1 text-sm" />
      </label>
    </div>

    <div class="grid md:grid-cols-3 gap-4 border border-gray-200 rounded p-3 bg-gray-50">
      <label class="inline-flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={taggingRunStitching} />
        <span>Include stitching action</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={taggingRunImages} />
        <span>Include images action</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={taggingRunColorCounts} />
        <span>Include threads and colours action</span>
      </label>

      <label class="inline-flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={taggingClearExistingStitching} disabled={!taggingRunStitching} />
        <span>Clear existing stitching tags first</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={taggingImageRedo} disabled={!taggingRunImages} />
        <span>Re-process all images</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          bind:checked={taggingUpgrade2dTo3d}
          disabled={!taggingRunImages || taggingImageRedo}
        />
        <span>Upgrade 2D images to 3D</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm md:col-span-3">
        <input type="checkbox" bind:checked={taggingUsePreview3d} disabled={!taggingRunImages} />
        <span>Use 3D preview generation for image action</span>
      </label>
    </div>

    <div class="flex flex-wrap gap-2">
      <button type="submit" class="menu-button-primary" disabled={taggingRunInFlight}>Run selected actions</button>
      <button type="button" class="menu-button-secondary" onclick={stopTaggingActionsRun} disabled={!taggingRunInFlight}>
        Stop running
      </button>
      <button type="button" class="menu-button-secondary" onclick={runStitchingOnlyAction} disabled={taggingRunInFlight}>
        Run stitching-only
      </button>
      <button type="button" class="menu-button-secondary" onclick={refreshBackfillLogEntries} disabled={taggingRunInFlight}>
        Refresh log
      </button>
    </div>
  </form>

  <div
    class="border rounded px-3 py-2 text-sm"
    class:bg-green-50={taggingStatusType === "success"}
    class:border-green-300={taggingStatusType === "success"}
    class:text-green-800={taggingStatusType === "success"}
    class:bg-red-50={taggingStatusType === "error"}
    class:border-red-300={taggingStatusType === "error"}
    class:text-red-800={taggingStatusType === "error"}
    class:bg-blue-50={taggingStatusType !== "success" && taggingStatusType !== "error"}
    class:border-blue-200={taggingStatusType !== "success" && taggingStatusType !== "error"}
    class:text-blue-800={taggingStatusType !== "success" && taggingStatusType !== "error"}
  >
    {taggingStatusMessage}
  </div>

  {#if taggingLastSummary}
    <div class="route-panel text-sm">
      <p><strong>Last run:</strong> processed {taggingLastSummary.processed}, errors {taggingLastSummary.errors}, stopped {taggingLastSummary.stopped ? "yes" : "no"}.</p>
      <p><strong>Actions:</strong> {(taggingLastSummary.actions || []).join(", ") || "none"}</p>
    </div>
  {/if}

  <div class="route-panel text-sm">
    <p class="font-semibold mb-2">Backfill log (latest)</p>
    {#if taggingLogEntries.length === 0}
      <p class="text-gray-600">No log entries available.</p>
    {:else}
      <ul class="space-y-1">
        {#each taggingLogEntries as entry}
          <li><strong>{entry.level}:</strong> {entry.message}</li>
        {/each}
      </ul>
    {/if}
  </div>

  {#if !taggingHasGoogleApiKey}
    <div class="route-panel text-sm text-blue-800 bg-blue-50 border border-blue-200">
      No API key configured. Tier 2 and Tier 3 are disabled; Tier 1 keyword tagging still works.
    </div>
  {/if}
</div>
