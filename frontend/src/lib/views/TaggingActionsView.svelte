<script lang="ts">
  import { onMount } from "svelte";
  import {
    getTaggingActionsViewModel,
    runUnifiedBackfill,
    stopUnifiedBackfill,
    getBackfillLogEntries,
    runStitchingBackfill,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";

  let taggingActionsLoaded = $state(false);
  let taggingActionsLoading = $state(false);
  let taggingRunInFlight = $state(false);
  // Global UI lock: reflects busyState.active so secondary controls can be
  // disabled while a long-running task runs.
  let busyActive = $derived($busyState.active);
  let taggingHasGoogleApiKey = $state(false);
  let taggingBatchSize = $state("100");
  let taggingCommitEvery = $state("100");
  let taggingWorkers = $state("4");

  // Top-level action toggles (all default unchecked for safety).
  let taggingRunTagging = $state(false);
  let taggingRunStitching = $state(false);
  let taggingRunImages = $state(false);
  let taggingRunColorCounts = $state(false);
  let taggingRunHoopDimensions = $state(false);

  // Sub-options / child controls (all default false, disabled until parent checked).
  let taggingRetagAll = $state(false);
  let taggingRunTier2 = $state(false);
  let taggingRunTier3 = $state(false);
  let taggingStitchingOverwrite = $state(false);
  let taggingImageRedo = $state(false);

  let taggingLastSummary = $state<{
    processed: number;
    errors: number;
    stopped: boolean;
    actions: string[];
    stitching_tag_count_before?: number;
    stitching_tag_count_after?: number;
    error?: string;
  } | null>(null);
  let taggingLogEntries = $state<Array<{ level: string; message: string }>>([]);

  let taggingCommitValue = $derived(Math.max(1, Number.parseInt(taggingCommitEvery, 10) || 100));
  let taggingBatchValue = $derived(Math.max(1, Number.parseInt(taggingBatchSize, 10) || 100));
  let taggingWorkersValue = $derived(Math.max(1, Number.parseInt(taggingWorkers, 10) || 4));

  // Run button is only enabled when at least one top-level action is selected.
  let taggingAnyActionSelected = $derived(
    taggingRunTagging ||
      taggingRunStitching ||
      taggingRunImages ||
      taggingRunColorCounts ||
      taggingRunHoopDimensions
  );

  async function loadTaggingViewModel(force = false) {
    if (taggingActionsLoading && !force) return;
    if (!force && taggingActionsLoaded) return;

    taggingActionsLoading = true;
    try {
      const result = await getTaggingActionsViewModel();
      const model =
        (result?.model as import("../types/ipc").TaggingActionsViewModel | null | undefined) ||
        null;
      taggingHasGoogleApiKey = Boolean(model?.has_google_api_key);
      taggingActionsLoaded = true;
      addToast(
        model?.has_google_api_key
          ? "API key detected. AI tagging actions are available."
          : "No Google API key set. AI tagging actions will be skipped.",
        "info"
      );
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
    beginBusy("Running tagging actions");
    addToast("Running selected actions...", "info");

    try {
      if (taggingRunStitching) {
        const stitchingOptions = /** @type {any} */ {
          commit_every: taggingCommitValue,
          batch_size: taggingBatchValue,
          workers: taggingWorkersValue,
          // "all" only when the user opted to overwrite previously processed designs.
          clear_stitching_mode: taggingStitchingOverwrite ? "all" : "unverified",
          image_redo: taggingImageRedo,
        };
        const result = await runStitchingBackfill(stitchingOptions);
        if (result?.error) {
          addToast(`Stitching backfill failed: ${result.error}`, "error");
          taggingRunInFlight = false;
          return;
        }
        addToast(`Stitching backfill complete.`, "success");
      }

      if (taggingRunTagging || taggingRunImages || taggingRunColorCounts || taggingRunHoopDimensions) {
        const result = await runUnifiedBackfill({
          action_mode: taggingRetagAll ? "tag_all" : "tag_untagged",
          run_tier2: taggingHasGoogleApiKey && taggingRunTier2,
          run_tier3: taggingHasGoogleApiKey && taggingRunTier3,
          run_images: taggingRunImages,
          image_redo: taggingImageRedo,
          run_color_counts: taggingRunColorCounts,
          run_hoop_dimensions: taggingRunHoopDimensions,
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
          addToast(
            `Backfill complete: ${processed} processed, ${errors} errors${stopped ? " (stopped early)" : ""}.`,
            stopped ? "warning" : errors > 0 ? "warning" : "success"
          );
        }
      }

      await loadTaggingLogEntries();
    } catch (e) {
      addToast(`Backfill run failed: ${e}`, "error");
    } finally {
      taggingRunInFlight = false;
      endBusy();
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
    <!-- API Key Status -->
    {#if !taggingHasGoogleApiKey}
      <div class="bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-3 text-sm">
        No Google API key is configured in Settings. AI tagging actions will be skipped.
        Keyword-only tagging (Tier 1) always runs.
      </div>
    {:else}
      <div class="bg-amber-50 border border-amber-200 text-amber-800 rounded px-4 py-3 text-sm">
        API key detected — AI tagging actions are available. Gemini calls may incur charges on your
        Google account.
      </div>
    {/if}

    <div class="bg-white rounded shadow p-6 space-y-5">
      <h2 class="text-base font-semibold text-gray-800">Select actions</h2>

      <div class="space-y-3">
        <!-- Tagging -->
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunTagging}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Tagging</span>
            {#if taggingHasGoogleApiKey}
              <p class="text-gray-500 text-xs mt-0.5">
                Run local keyword matching (Tier 1) and optional Gemini AI suggestions (Tiers 2 &
                3).
              </p>
            {:else}
              <p class="text-gray-500 text-xs mt-0.5">
                Run local keyword matching (Tier 1) based on filenames and folder names.
              </p>
            {/if}
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingRetagAll}
              disabled={!taggingRunTagging || busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span>Re-tag designs that already have tags (instead of only untagged designs)</span>
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingRunTier2}
              disabled={!taggingRunTagging || !taggingHasGoogleApiKey || busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span
              >Run <strong>Tier 2</strong> (Gemini text AI — suggest tags from filename){#if !taggingHasGoogleApiKey}
                <span class="text-xs font-medium text-gray-400 italic">(API Key Required)</span
                >{/if}</span
            >
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingRunTier3}
              disabled={!taggingRunTagging || !taggingHasGoogleApiKey || busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span
              >Run <strong>Tier 3</strong> (Gemini vision AI — suggest tags from preview image){#if !taggingHasGoogleApiKey}
                <span class="text-xs font-medium text-gray-400 italic">(API Key Required)</span
                >{/if}</span
            >
          </label>
          {#if !taggingHasGoogleApiKey}
            <p class="text-xs text-gray-400 italic mt-1">
              Configure a Gemini API key in
              <a href="#/admin/settings" class="text-indigo-600 underline font-medium">Settings</a>
              to enable AI-powered tag suggestions.
            </p>
          {/if}
        </div>
      </div>

      <div class="space-y-3">
        <!-- Stitching tag detection -->
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunStitching}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Stitching tag detection</span>
            <p class="text-gray-500 text-xs mt-0.5">
              Analyze stitch coverage area to tag designs (e.g., light fill, dense embroidery,
              outline).
            </p>
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingStitchingOverwrite}
              disabled={!taggingRunStitching || busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span>Overwrite stitching tags on designs that have already been processed</span>
          </label>
        </div>
      </div>

      <div class="space-y-3">
        <!-- Image generation / regeneration -->
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunImages}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Image generation / regeneration</span>
            <p class="text-gray-500 text-xs mt-0.5">Generate preview images.</p>
          </div>
        </label>

        <div class="ml-8 space-y-2 border-l-2 border-gray-100 pl-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingImageRedo}
              disabled={!taggingRunImages || busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span>Regenerate images for all designs, not just those without images</span>
          </label>
        </div>
      </div>

      <div class="space-y-3">
        <!-- Recalculate colour / stitch counts -->
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunColorCounts}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Recalculate colour / stitch counts</span>
            <p class="text-gray-500 text-xs mt-0.5">
              Refresh thread colors, stitch totals, and color changes from the design files
            </p>
          </div>
        </label>

        <!-- Recalculate hoops / dimensions -->
        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunHoopDimensions}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Recalculate hoops / dimensions</span>
            <p class="text-gray-500 text-xs mt-0.5">
              Refresh design dimensions (width/height) and recommended hoop from the design files
            </p>
          </div>
        </label>
      </div>
    </div>

    <!-- Run / Stop Buttons -->
    <div class="flex flex-wrap items-center gap-3">
      <button
        class="menu-button-primary"
        onclick={runTaggingActions}
        disabled={taggingRunInFlight || taggingActionsLoading || busyActive || !taggingAnyActionSelected}
      >
        {taggingRunInFlight ? "Running..." : "Run selected actions"}
      </button>
      <button
        class="menu-button-secondary text-red-600 border-red-200 hover:bg-red-50"
        onclick={requestTaggingStop}
        disabled={!taggingRunInFlight}
      >
        Stop
      </button>
    </div>

    <!-- Last summary -->
    {#if taggingLastSummary}
      <div class="bg-white rounded shadow p-4 space-y-1 text-sm">
        <p class="font-semibold text-gray-800">Last run summary</p>
        <p>
          Processed: <strong>{taggingLastSummary.processed ?? 0}</strong> &middot; Errors:
          <strong>{taggingLastSummary.errors ?? 0}</strong>
        </p>
        {#if taggingLastSummary.stitching_tag_count_before !== undefined}
          <p>
            Stitching tags: <strong>{taggingLastSummary.stitching_tag_count_before}</strong> before
            &rarr; <strong>{taggingLastSummary.stitching_tag_count_after ?? 0}</strong> after
          </p>
        {/if}
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
      <summary
        class="cursor-pointer px-4 py-3 text-sm font-semibold text-gray-700 hover:bg-gray-50 select-none"
      >
        Backfill log ({taggingLogEntries.length} entries)
      </summary>
      <div class="max-h-80 overflow-y-auto px-4 pb-3 space-y-1">
        {#if taggingLogEntries.length === 0}
          <p class="text-xs text-gray-400 italic py-2">
            No log entries yet. Run an action to populate the log.
          </p>
        {:else}
          {#each taggingLogEntries as entry}
            <div
              class="text-xs font-mono p-1 rounded {entry.level === 'error'
                ? 'text-red-600 bg-red-50'
                : entry.level === 'warn'
                  ? 'text-amber-700 bg-amber-50'
                  : 'text-gray-700'}"
            >
              <span class="font-semibold uppercase text-[10px] mr-1">{entry.level}</span>
              {entry.message}
            </div>
          {/each}
        {/if}
      </div>
    </details>
  </div>
</section>
