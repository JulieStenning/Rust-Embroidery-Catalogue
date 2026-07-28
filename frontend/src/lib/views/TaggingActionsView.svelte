<script>
  import { onMount } from "svelte";
  import {
    getTaggingActionsViewModel,
    runUnifiedBackfill,
    stopUnifiedBackfill,
    getBackfillLogEntries,
    runStitchingBackfill
  } from "../api/commandAdapter.js";
  import { addToast } from "../stores/toastStore.js";

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
  /** @type {{processed: number, errors: number, stopped: boolean, actions: string[], error?: string} | null} */
  let taggingLastSummary = $state(null);
  /** @type {Array<{level: string, message: string}>} */
  let taggingLogEntries = $state([]);

  let taggingCommitValue = $derived(Math.max(1, Number.parseInt(taggingCommitEvery, 10) || 100));
  let taggingBatchValue = $derived(Math.max(1, Number.parseInt(taggingBatchSize, 10) || 100));
  let taggingWorkersValue = $derived(Math.max(1, Number.parseInt(taggingWorkers, 10) || 4));

  async function loadTaggingViewModel(force = false) {
    if (taggingActionsLoading && !force) return;
    if (!force && taggingActionsLoaded) return;

    taggingActionsLoading = true;
    try {
      const result = await getTaggingActionsViewModel();
      const model = /** @type {any} */ (result?.model) || {};
      taggingHasGoogleApiKey = Boolean(model?.has_google_api_key);
      taggingTier2Default = Boolean(model?.tier2_default);
      taggingTier3Default = Boolean(model?.tier3_default);
      taggingActionsLoaded = true;
      addToast(model?.has_google_api_key
        ? "API key detected. AI tagging actions are available."
        : "No Google API key set. AI tagging actions will be skipped.",
        "info");
    } catch (error) {
      addToast(`Could not load tagging action defaults: ${error}`, "error");
    } finally {
      taggingActionsLoading = false;
    }
  }

  async function runTaggingActions() {
    if (taggingRunInFlight) return;

    taggingRunInFlight = true;
    taggingLastSummary = null;
    addToast("Running selected actions...", "info");

    try {
      if (taggingRunStitching) {
        const stitchingOptions = /** @type {any} */ ({
          commit_every: taggingCommitValue,
          batch_size: taggingBatchValue,
          workers: taggingWorkersValue,
          clear_existing: taggingClearExistingStitching,
          image_redo: taggingImageRedo,
          use_preview_3d: taggingUsePreview3d,
        });
        const result = await runStitchingBackfill(stitchingOptions);
        if (result?.error) {
          addToast(`Stitching backfill failed: ${result.error}`, "error");
          taggingRunInFlight = false;
          return;
        }
        addToast(`Stitching backfill complete.`, "success");
      }

      if (taggingRunTier2 || taggingRunTier3 || taggingRunImages || taggingRunColorCounts) {
        const result = await runUnifiedBackfill({
          action_mode: taggingActionMode,
          run_tier2: taggingRunTier2,
          run_tier3: taggingRunTier3,
          run_images: taggingRunImages,
          image_redo: taggingImageRedo,
          upgrade_2d_to_3d: taggingUpgrade2dTo3d,
          use_preview_3d: taggingUsePreview3d,
          run_color_counts: taggingRunColorCounts,
          commit_every: taggingCommitValue,
          batch_size: taggingBatchValue,
          workers: taggingWorkersValue,
        });

        const processed = Number(result?.processed ?? 0);
        const errors = Number(result?.errors ?? 0);
        const stopped = Boolean(result?.stopped);
        taggingLastSummary = result || null;

        if (result?.error) {
          addToast(`Backfill failed: ${result.error}`, "error");
        } else {
          addToast(`Backfill complete: ${processed} processed, ${errors} errors${stopped ? " (stopped early)" : ""}.`,
            stopped ? "warning" : (errors > 0 ? "warning" : "success"));
        }
      }

      await loadTaggingLogEntries();
    } catch (e) {
      addToast(`Backfill run failed: ${e}`, "error");
    } finally {
      taggingRunInFlight = false;
    }
  }

  async function requestTaggingStop() {
    if (!taggingRunInFlight) return;
    try {
      await stopUnifiedBackfill();
      addToast("Stop requested.", "info");
    } catch (e) {
      addToast(`Stop request failed: ${e}`, "error");
    }
  }

  async function loadTaggingLogEntries() {
    try {
      const result = await getBackfillLogEntries();
      taggingLogEntries = Array.isArray(result?.entries) ? result.entries : [];
    } catch (e) {
      console.info("Could not load backfill log entries", e);
    }
  }

  onMount(() => {
    loadTaggingViewModel();
    loadTaggingLogEntries();
  });
</script>

<section class="tagging-actions-page space-y-6 font-sans">
  <h1 class="ui-page-title tagging-actions-title mb-2">Tagging Actions</h1>
  <p class="text-sm text-gray-500 mb-4">
    Run bulk tagging, image generation, or stitching calculation actions on your existing catalogue.
  </p>

  <div class="tagging-actions-layout max-w-3xl space-y-6">
    <!-- Action Mode -->
    {#if !taggingHasGoogleApiKey}
      <div class="bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-3 text-sm">
        No Google API key is configured in Settings. AI tagging actions will be skipped. Keyword-only tagging (Tier 1) always runs.
      </div>
    {:else}
      <div class="bg-amber-50 border border-amber-200 text-amber-800 rounded px-4 py-3 text-sm">
        API key detected — AI tagging actions are available. Gemini calls may incur charges on your Google account.
      </div>
    {/if}

    <div class="bg-white rounded shadow p-6 space-y-5">
      <h2 class="text-base font-semibold text-gray-800">Select actions</h2>

      <div class="space-y-3">
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input type="checkbox" checked={taggingActionMode === "tag_untagged"} class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" onchange={() => { taggingActionMode = (taggingActionMode === "tag_untagged" ? "tag_all" : "tag_untagged"); }} />
          <div>
            <span class="font-semibold">Tagging</span>
            <p class="text-gray-500 text-xs mt-0.5">Run keyword matching (Tier 1) on designs. If no API key, Tier 2 and 3 are skipped.</p>
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingRunTier2} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>Run <strong>Tier 2</strong> (Gemini text AI — suggest tags from filename)</span>
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingRunTier3} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>Run <strong>Tier 3</strong> (Gemini vision AI — suggest tags from preview image)</span>
          </label>
        </div>
      </div>

      <div class="space-y-3">
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input type="checkbox" bind:checked={taggingRunStitching} class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
          <div>
            <span class="font-semibold">Stitching tag detection</span>
            <p class="text-gray-500 text-xs mt-0.5">Detect stitching region percentages from design files and assign stitching tags.</p>
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingClearExistingStitching} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>Clear existing stitching tags before re-running</span>
          </label>
        </div>
      </div>

      <div class="space-y-3">
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input type="checkbox" bind:checked={taggingRunImages} class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
          <div>
            <span class="font-semibold">Image generation / regeneration</span>
            <p class="text-gray-500 text-xs mt-0.5">Generate flat preview images. Can be combined with upgrade to 3D and 2D/3D regeneration.</p>
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingImageRedo} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>Regenerate images for all designs, not just those without images</span>
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingUpgrade2dTo3d} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>Upgrade 2D previews to 3D stitch-simulated previews</span>
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={taggingUsePreview3d} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            <span>When generating stitching preview images, use 3D renderer (slower, more accurate). Uncheck for fast 2D</span>
          </label>
        </div>
      </div>

      <div class="space-y-3">
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input type="checkbox" bind:checked={taggingRunColorCounts} class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
          <div>
            <span class="font-semibold">Recalculate colour / stitch counts</span>
            <p class="text-gray-500 text-xs mt-0.5">Re-run stitch parsing to update colour count, stitch count, and colour change counts.</p>
          </div>
        </label>
      </div>
    </div>

    <!-- Run / Stop Buttons -->
    <div class="flex flex-wrap items-center gap-3">
      <button class="menu-button-primary" onclick={runTaggingActions} disabled={taggingRunInFlight || taggingActionsLoading}>
        {taggingRunInFlight ? "Running..." : "Run selected actions"}
      </button>
      <button class="menu-button-secondary text-red-600 border-red-200 hover:bg-red-50" onclick={requestTaggingStop} disabled={!taggingRunInFlight}>
        Stop
      </button>
    </div>

    <!-- Last summary -->
    {#if taggingLastSummary}
      <div class="bg-white rounded shadow p-4 space-y-1 text-sm">
        <p class="font-semibold text-gray-800">Last run summary</p>
        <p>Processed: <strong>{taggingLastSummary.processed ?? 0}</strong> &middot; Errors: <strong>{taggingLastSummary.errors ?? 0}</strong></p>
        {#if taggingLastSummary.stopped}
          <p class="text-amber-700 font-semibold">Stopped early</p>
        {/if}
        {#if taggingLastSummary.error}
          <p class="text-red-600">{taggingLastSummary.error}</p>
        {/if}
      </div>
    {/if}

    <!-- Log -->
    <details class="bg-white rounded shadow">
      <summary class="cursor-pointer px-4 py-3 text-sm font-semibold text-gray-700 hover:bg-gray-50 select-none">
        Backfill log ({taggingLogEntries.length} entries)
      </summary>
      <div class="max-h-80 overflow-y-auto px-4 pb-3 space-y-1">
        {#if taggingLogEntries.length === 0}
          <p class="text-xs text-gray-400 italic py-2">No log entries yet. Run an action to populate the log.</p>
        {:else}
          {#each taggingLogEntries as entry}
            <div class="text-xs font-mono p-1 rounded {entry.level === 'error' ? 'text-red-600 bg-red-50' : entry.level === 'warn' ? 'text-amber-700 bg-amber-50' : 'text-gray-700'}">
              <span class="font-semibold uppercase text-[10px] mr-1">{entry.level}</span>
              {entry.message}
            </div>
          {/each}
        {/if}
      </div>
    </details>
  </div>
</section>