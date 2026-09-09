<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  import {
    getBatchOperationsViewModel,
    runUnifiedBackfill,
    stopUnifiedBackfill,
    getBackfillLogEntries,
    runStitchingBackfill,
    runMaintenanceBackfill,
    countMissingPreviews,
    countTaggingCandidates,
    browseTaggingFolder,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  import { initBackfillProgressEvents } from "../services/backfillEvents";
  import {
    backfillProgressStore,
    resetBackfillProgress,
  } from "../stores/backfillProgressStore";
  import { browseSessionStore } from "../stores/browseSessionStore.js";

  let backfillProgressUnlisten: (() => void) | null = null;

  // ---------------------------------------------------------------------------
  // Non-technical workflow choices: Goal -> Scope -> Merge.
  // ---------------------------------------------------------------------------
  type TaggingGoal = "file_folder" | "ai_vision" | "full_rescan";
  type TaggingScope =
    | "untagged"
    | "folder"
    | "all"
    | "vision_not_analyzed"
    | "vision_no_match"
    | "vision_analyzed";
  type TaggingMerge = "add" | "reset";

  const GOAL_OPTIONS: Array<{
    id: TaggingGoal;
    title: string;
    subtitle: string;
    requiresAi: boolean;
  }> = [
    {
      id: "file_folder",
      title: "Apply file & folder rules",
      subtitle: "Automatically extract tags from folder names and file names. (Fast & Offline)",
      requiresAi: false,
    },
    {
      id: "ai_vision",
      title: "Enrich with visual AI",
      subtitle:
        "Analyze design thumbnails using Gemini Vision to detect subject matter. (Requires API key)",
      requiresAi: true,
    },
    {
      id: "full_rescan",
      title: "Full re-scan (both methods)",
      subtitle: "Run file & folder rules and visual AI and merge the results. (Requires API key)",
      requiresAi: true,
    },
  ];

  const BASE_SCOPE_OPTIONS: Array<{
    id: string;
    title: string;
    subtitle: string;
    disabled?: boolean;
  }> = [
    { id: "untagged", title: "Untagged designs only", subtitle: "Designs with no tags at all" },
    {
      id: "folder",
      title: "Specific folder(s)",
      subtitle: "Only designs in a specific folder branch",
    },
    { id: "all", title: "Entire collection", subtitle: "All designs in your library" },
  ];

  const MODE_SCOPE_OPTIONS: Record<
    string,
    Array<{ id: string; title: string; subtitle: string; disabled?: boolean }>
  > = {
    ai_vision: [
      {
        id: "vision_not_analyzed",
        title: "Designs missing Visual AI analysis",
        subtitle: "Designs that haven't been scanned with Visual AI yet",
      },
      {
        id: "vision_no_match",
        title: "Visual AI found no match",
        subtitle: "Designs Visual AI analyzed but found no tags for",
      },
      {
        id: "vision_analyzed",
        title: "Re-analyze (already analyzed by Visual AI)",
        subtitle: "Designs already analyzed by Visual AI — run again",
      },
    ],
  };

  const MERGE_OPTIONS: Array<{
    id: TaggingMerge;
    title: string;
    subtitle: string;
  }> = [
    {
      id: "add",
      title: "Add new tags only (recommended / safe)",
      subtitle: "Keep all existing tags and append any newly discovered tags",
    },
    {
      id: "reset",
      title: "Complete reset",
      subtitle: "Clear all existing tags on selected designs and start fresh",
    },
  ];

  // Live progress streamed from Rust during a backfill run ("Processed N…").
  let backfillProgress = $derived($backfillProgressStore);
  let taggingRunButtonLabel = $derived.by(() => {
    const p = backfillProgress;
    if (p.active && p.processed > 0) {
      return `Processing no. ${p.processed}`;
    }
    return "Getting ready for tagging";
  });

  let taggingActionsLoaded = $state(false);
  let taggingActionsLoading = $state(false);
  let taggingRunInFlight = $state(false);
  // Global UI lock: reflects busyState.active so secondary controls can be
  // disabled while a long-running task runs.
  let busyActive = $derived($busyState.active);
  let taggingHasGoogleApiKey = $state(false);
  let taggingFreeTier = $state(false);
  let taggingDelay = $state("");
  let taggingBatchSize = $state("100");
  let taggingCommitEvery = $state("100");
  let taggingWorkers = $state("4");

  // Step 1 – Goal, Step 2 – Scope, Step 3 – Merge.
  let goal = $state<TaggingGoal>("file_folder");
  let scope = $state<TaggingScope>("untagged");
  let merge = $state<TaggingMerge>("add");

  // Live candidate counts for the scope badges + pre-flight estimate.
  let scopeCounts = $state<Record<string, import("../types/ipc").TaggingScopeCounts>>({});
  let countsLoading = $state(false);
  // Protect human-verified designs from being overwritten by an automated pass.
  let excludeVerified = $state(true);
  let showConfirm = $state(false);
  // Two-tab organisation: "Tagging & Categorisation" (default) vs
  // "Maintenance & File Processing". Maintenance-only runs process the whole
  // catalogue via `runMaintenanceBackfill`.
  let activeTab = $state<"tagging" | "maintenance">("tagging");
  let showMaintenanceConfirm = $state(false);
  // Population scope for every maintenance task: the whole catalogue, or only
  // designs missing a stored preview image.
  let maintenanceScope = $state<"all" | "missing_previews">("all");
  let missingPreviewCount = $state<number | null>(null);
  let missingPreviewCountLoading = $state(false);
  // Distinguishes a maintenance-only run from a tagging run in the shared summary,
  // so "processed" is labelled as operations (a design may be counted once per task).
  let lastRunWasMaintenance = $state(false);

  // Folder-scoped retagging (Specific Folder or Category scope). Zero or more
  // library subfolders can be selected (union of their designs).
  let selectedFolderPaths = $state<string[]>([]);
  let includeSubfolders = $state(true);
  let dataStorageLocation = $state("");

  // Advanced options (collapsed by default) — optional extra passes.
  let taggingRunStitching = $state(false);
  let taggingRunImages = $state(false);
  let taggingRunColorCounts = $state(false);
  let taggingRunHoopDimensions = $state(false);
  let taggingStitchingOverwrite = $state(false);
  let taggingImageRedo = $state(false);

  // --- Maintenance tab derived state ---------------------------------------
  // Whether at least one maintenance task is selected (drives the enabled state
  // of "Review & Start Maintenance").
  let anyMaintenanceTaskSelected = $derived(
    taggingRunImages || taggingRunColorCounts || taggingRunHoopDimensions
  );
  let maintenanceValid = $derived(anyMaintenanceTaskSelected);
  let maintenanceRunButtonLabel = $derived.by(() => {
    const p = backfillProgress;
    if (p.active && p.processed > 0) {
      return `Processing ${p.processed} operations…`;
    }
    return "Review & Start Maintenance";
  });

  function switchToMaintenanceTab() {
    activeTab = "maintenance";
  }

  // Human-readable label for the selected Target Scope, incl. the live count.
  const maintenanceScopeLabel = $derived(
    maintenanceScope === "missing_previews"
      ? `Designs missing a preview image${
          missingPreviewCount !== null
            ? ` (${missingPreviewCount.toLocaleString()} designs)`
            : ""
        }`
      : "Entire catalogue"
  );

  async function loadMissingPreviewCount() {
    missingPreviewCountLoading = true;
    try {
      missingPreviewCount = await countMissingPreviews();
    } catch {
      missingPreviewCount = null;
    } finally {
      missingPreviewCountLoading = false;
    }
  }

  let taggingLastSummary = $state<{
    processed: number;
    errors: number;
    stopped: boolean;
    actions: string[];
    stitching_tag_count_before?: number;
    stitching_tag_count_after?: number;
    image_tag_count_before?: number;
    image_tag_count_after?: number;
    missing_preview_count_before?: number;
    missing_preview_count_after?: number;
    error?: string;
  } | null>(null);
  let taggingLogEntries = $state<Array<{ level: string; message: string }>>([]);

  /** Pre-apply the "Needs attention" Browse filter and jump to Browse Designs. */
  function reviewNeedsAttentionInBrowse() {
    browseSessionStore.focusNeedsAttention();
    if (typeof window !== "undefined") {
      window.location.hash = "#/designs";
    }
  }

  let taggingCommitValue = $derived(Math.max(1, Number.parseInt(taggingCommitEvery, 10) || 100));
  let taggingBatchValue = $derived(Math.max(1, Number.parseInt(taggingBatchSize, 10) || 100));
  let taggingWorkersValue = $derived(Math.max(1, Number.parseInt(taggingWorkers, 10) || 4));

  // Which scope options are visible depends on the selected goal's mode(s).
  const scopeOptionsFor = $derived.by((): Array<{
    id: string;
    title: string;
    subtitle: string;
    disabled?: boolean;
  }> => {
    const list = BASE_SCOPE_OPTIONS.slice();
    if (goal === "ai_vision" || goal === "full_rescan") {
      list.push(...MODE_SCOPE_OPTIONS.ai_vision);
    }
    return list;
  });

  // If the currently selected scope is no longer offered for this goal, fall back
  // to the always-available untagged scope.
  $effect(() => {
    if (!scopeOptionsFor.some((o) => o.id === scope)) {
      scope = "untagged";
    }
  });

  // Derived workflow mapping to the backend wire contract.
  const modes = $derived.by((): string[] => {
    if (goal === "file_folder") return ["path_rule"];
    if (goal === "ai_vision") return ["ai_vision"];
    return ["path_rule", "ai_vision"];
  });
  const action = $derived.by((): string => {
    switch (scope) {
      case "untagged":
        return "tag_untagged";
      case "folder":
        return "retag_all";
      case "all":
        return "retag_all";
      case "vision_not_analyzed":
        return "retag_all_vision_not_analyzed";
      case "vision_no_match":
        return "retag_all_vision_no_match";
      case "vision_analyzed":
        return "retag_all_vision_analyzed";
      default:
        return "tag_untagged";
    }
  });

  const goalLabel = $derived.by(() => {
    const found = GOAL_OPTIONS.find((o) => o.id === goal);
    return found ? found.title : goal;
  });
  const scopeLabel = $derived.by(() => {
    const found = scopeOptionsFor.find((o) => o.id === scope);
    return found ? found.title : scope;
  });
  const mergeLabel = $derived.by(() => {
    const found = MERGE_OPTIONS.find((o) => o.id === merge);
    return found ? found.subtitle : merge;
  });

  /** Active candidate total for a scope: unverified when verified designs are excluded. */
  function activeCountFor(scopeId: string): number | null {
    const counts = scopeCounts[scopeId];
    if (!counts) return null;
    return excludeVerified ? counts.unverified_count : counts.total_count;
  }

  const selectedCount = $derived.by((): number | null => activeCountFor(scope));
  // Human-readable label for a selected folder: root-relative when possible.
  function folderDisplayPath(folderPath: string): string {
    if (!folderPath) return "";
    if (dataStorageLocation && folderPath.startsWith(dataStorageLocation)) {
      const rel = folderPath.slice(dataStorageLocation.length).replace(/^[\\/]+/, "");
      return rel || folderPath;
    }
    return folderPath;
  }

  async function chooseTaggingFolder() {
    try {
      const result = await browseTaggingFolder(selectedFolderPaths[0] ?? null);
      if (result.error) {
        addToast(`Could not pick folder: ${result.error}`, "error");
        return;
      }
      const picked = Array.isArray(result.paths)
        ? result.paths.map((value) => String(value || "").trim()).filter(Boolean)
        : [];
      if (picked.length === 0) return; // user cancelled
      // Merge new picks, de-duplicating case-insensitively against the current list.
      const known = new Set(selectedFolderPaths.map((value) => String(value || "").toLowerCase()));
      const additions = picked.filter((value) => !known.has(String(value).toLowerCase()));
      if (additions.length > 0) {
        selectedFolderPaths = [...selectedFolderPaths, ...additions];
        await loadScopeCounts();
      }
    } catch (error) {
      addToast(`Could not pick folder: ${error}`, "error");
    }
  }

  function removeTaggingFolder(folderPath: string) {
    const target = String(folderPath || "").toLowerCase();
    const remaining = selectedFolderPaths.filter(
      (value) => String(value || "").toLowerCase() !== target
    );
    if (remaining.length === selectedFolderPaths.length) return;
    selectedFolderPaths = remaining;
    loadScopeCounts();
  }

  /** Fetch the folder-scope counts (when folder(s) are selected) into `next`. */
  async function loadFolderCountsInto(
    next: Record<string, import("../types/ipc").TaggingScopeCounts>
  ): Promise<void> {
    delete next.folder;
    if (selectedFolderPaths.length === 0) return;
    const result = await countTaggingCandidates("retag_all", selectedFolderPaths, includeSubfolders);
    next.folder = result.counts;
  }

  // Whether Vision AI participates in the run (drives the legacy `run_vision` flag).
  const visionInvolved = $derived(goal === "ai_vision" || goal === "full_rescan");

  // The effective per-request delay for a Visual AI run: the configured AI delay
  // wins if set; otherwise the free tier falls back to a conservative 10s and
  // paid keys use no artificial pacing.
  const effectiveVisualAiDelay = $derived.by((): number => {
    const rawDelay = Number(taggingDelay);
    return rawDelay > 0 ? rawDelay : taggingFreeTier ? 10 : 0;
  });

  // Whether the run is actually paced to respect Gemini rate limits. Only when
  // this is true does the estimate reflect rate-limit throttling.
  const visualAiPaced = $derived(effectiveVisualAiDelay > 0);

  // Rough wall-clock estimate for Visual AI runs (paced by delay and workers).
  // Paced runs are dominated by the per-call delay, so they take much longer than
  // non-paced runs; the free tier's ~15 req/min rate limit (~4s per call) is a
  // hard floor regardless of the configured worker count.
  const estimateMinutes = $derived.by((): number | null => {
    const count = selectedCount;
    if (count === null || count <= 0 || goal === "file_folder") return null;
    const delay = effectiveVisualAiDelay;
    const workers = Math.max(1, taggingWorkersValue);
    // Only Visual AI makes a Gemini call; File & Folder Rules are local and free.
    const aiModeCount = 1;
    let perDesignSeconds: number;
    if (delay > 0) {
      // Each worker waits `delay` seconds between calls; the free tier can never
      // do better than one request per ~4s due to the 15 req/min cap.
      perDesignSeconds = Math.max(delay / workers, taggingFreeTier ? 4 : 0);
    } else {
      // Non-paced: assume a rough ~1s per Gemini call, divided across workers.
      perDesignSeconds = 1.0 / workers;
    }
    return Math.max(1, Math.ceil((count * perDesignSeconds * aiModeCount) / 60));
  });
  async function loadScopeCounts() {
    countsLoading = true;
    const targets: Array<[string, string]> = [
      ["untagged", "tag_untagged"],
      ["all", "retag_all"],
      ["vision_not_analyzed", "retag_all_vision_not_analyzed"],
      ["vision_no_match", "retag_all_vision_no_match"],
      ["vision_analyzed", "retag_all_vision_analyzed"],
    ];
    const next: Record<string, import("../types/ipc").TaggingScopeCounts> = {};
    try {
      await Promise.all([
        ...targets.map(async ([key, scopeAction]) => {
          const result = await countTaggingCandidates(scopeAction);
          next[key] = result.counts;
        }),
        loadFolderCountsInto(next),
      ]);
      scopeCounts = next;
    } catch (error) {
      addToast(`Could not count tagging candidates: ${error}`, "error");
    } finally {
      countsLoading = false;
    }
  }

  async function loadTaggingViewModel(force = false) {
    if (taggingActionsLoading && !force) return;
    if (!force && taggingActionsLoaded) return;

    taggingActionsLoading = true;
    try {
      const result = await getBatchOperationsViewModel();
      const model =
        (result?.model as import("../types/ipc").BatchOperationsViewModel | null | undefined) ||
        null;
      taggingHasGoogleApiKey = Boolean(model?.has_google_api_key);
      taggingBatchSize = String(model?.ai_batch_size || "100");
      taggingCommitEvery = String(model?.ai_commit_every || "100");
      taggingWorkers = String(model?.ai_workers || "4");
      taggingFreeTier = Boolean(model?.ai_free_tier);
      taggingDelay = String(model?.ai_delay || "");
      dataStorageLocation = String(model?.data_storage_location || "");
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

  async function confirmStartTagging() {
    showConfirm = false;
    if (taggingRunInFlight) return;

    taggingRunInFlight = true;
    taggingLastSummary = null;
    resetBackfillProgress();
    lastRunWasMaintenance = false;
    beginBusy("Running tagging actions");
    addToast("Running selected actions...", "info");

    try {
      if (taggingRunStitching) {
        const stitchingOptions = /** @type {any} */ {
          commit_every: taggingCommitValue,
          batch_size: taggingBatchValue,
          workers: taggingWorkersValue,
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

      const result = await runUnifiedBackfill({
        action_mode: action,
        modes,
        merge_mode: merge,
        exclude_verified: excludeVerified,
        folder_paths:
          scope === "folder" && selectedFolderPaths.length > 0
            ? selectedFolderPaths.slice()
            : undefined,
        include_subfolders: scope === "folder" ? includeSubfolders : undefined,
        run_vision: visionInvolved,
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

      await loadTaggingLogEntries();
      await loadScopeCounts();
    } catch (e) {
      addToast(`Backfill run failed: ${e}`, "error");
    } finally {
      taggingRunInFlight = false;
      endBusy();
    }
  }

  async function confirmStartMaintenance() {
    showMaintenanceConfirm = false;
    if (taggingRunInFlight) return;

    taggingRunInFlight = true;
    taggingLastSummary = null;
    resetBackfillProgress();
    lastRunWasMaintenance = true;
    beginBusy("Running maintenance actions");
    addToast("Running maintenance actions...", "info");

    try {
      const result = await runMaintenanceBackfill({
        scope: maintenanceScope,
        generate_previews: taggingRunImages,
        recalc_color_counts: taggingRunColorCounts,
        recalc_hoop_dimensions: taggingRunHoopDimensions,
        commit_every: taggingCommitValue,
        batch_size: taggingBatchValue,
        workers: taggingWorkersValue,
      });

      taggingLastSummary = result || null;
      if (result?.error) {
        addToast(`Maintenance failed: ${result.error}`, "error");
      } else {
        addToast(
          `Maintenance complete: ${Number(result?.processed ?? 0)} operations, ${Number(
            result?.errors ?? 0
          )} errors.`,
          Number(result?.errors ?? 0) > 0 ? "warning" : "success"
        );
      }

      await loadTaggingLogEntries();
      await loadScopeCounts();
      await loadMissingPreviewCount();
    } catch (e) {
      addToast(`Maintenance run failed: ${e}`, "error");
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

  onMount(async () => {
    resetBackfillProgress();
    loadTaggingViewModel();
    loadTaggingLogEntries();
    loadScopeCounts();
    loadMissingPreviewCount();
    try {
      backfillProgressUnlisten = await initBackfillProgressEvents();
    } catch (error) {
      console.info("Backfill progress events unavailable.", error);
    }
  });

  onDestroy(() => {
    if (backfillProgressUnlisten) {
      backfillProgressUnlisten();
      backfillProgressUnlisten = null;
    }
    resetBackfillProgress();
  });
</script>
<section class="batch-operations-page space-y-6 font-sans">
  <h1 class="ui-page-title batch-operations-title mb-2">Batch Operations</h1>
  <p class="text-sm text-gray-500 mb-4">
    Automated AI categorisation, rule-based tagging, and library file maintenance.
  </p>

  <div class="batch-operations-layout max-w-3xl space-y-6">
    <!-- Sub-tab navigation: tagging config vs file maintenance -->
    <div
      class="flex items-end gap-1 border-b border-gray-200 mb-2"
      role="tablist"
      aria-label="Batch operations mode"
      data-testid="batch-operations-tablist"
    >
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "tagging"}
        class="px-4 py-2 text-sm font-semibold rounded-t border {activeTab === 'tagging'
          ? 'border-gray-200 border-b-transparent bg-white text-indigo-600'
          : 'border-transparent text-gray-500 hover:text-gray-700 hover:bg-gray-50'}"
        onclick={() => (activeTab = "tagging")}
      >
        Tagging &amp; Categorisation
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "maintenance"}
        class="px-4 py-2 text-sm font-semibold rounded-t border {activeTab === 'maintenance'
          ? 'border-gray-200 border-b-transparent bg-white text-indigo-600'
          : 'border-transparent text-gray-500 hover:text-gray-700 hover:bg-gray-50'}"
        onclick={switchToMaintenanceTab}
      >
        Maintenance &amp; File Processing
      </button>
    </div>

    {#if activeTab === "tagging"}
    <!-- API Key Status -->
    {#if !taggingHasGoogleApiKey}
      <div class="bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-3 text-sm">
        No Google API key is configured in Settings. Visual AI tagging will be skipped.
        File & Folder Rules always run.
      </div>
    {:else}
      <div class="bg-amber-50 border border-amber-200 text-amber-800 rounded px-4 py-3 text-sm">
        API key detected — AI tagging actions are available. Gemini calls may incur charges on your
        Google account.
      </div>
    {/if}

    {#if taggingFreeTier && taggingHasGoogleApiKey}
      <div class="bg-amber-50 border border-amber-200 text-amber-800 rounded px-4 py-3 text-sm">
        Free tier detected — Gemini limits are roughly 15 requests/minute and 1,500/day. If a 429
        rate-limit error occurs the run stops and tells you how long to wait; it will not retry
        automatically.
      </div>
    {/if}

    <!-- Step 1: Goal -->
    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">1. What do you want to do?</h2>
      <div class="space-y-2">
        {#each GOAL_OPTIONS as option}
          <label
            class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3 {goal === option.id
              ? 'border-indigo-400 bg-indigo-50'
              : 'hover:bg-gray-50'} {option.requiresAi && !taggingHasGoogleApiKey
              ? 'opacity-50 pointer-events-none'
              : ''}"
          >
            <input
              type="radio"
              name="tagging-goal"
              value={option.id}
              bind:group={goal}
              disabled={busyActive || (option.requiresAi && !taggingHasGoogleApiKey)}
              class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500"
            />
            <div>
              <span class="font-semibold">{option.title}</span>
              <p class="text-gray-500 text-xs mt-0.5">{option.subtitle}</p>
            </div>
          </label>
        {/each}
      </div>
      {#if !taggingHasGoogleApiKey}
        <p class="text-xs text-gray-400 italic">
          Configure a Gemini API key in
          <a href="#/admin/system/settings" class="text-indigo-600 underline font-medium">Settings</a>
          to enable AI-powered tag suggestions.
        </p>
      {/if}
    </div>

    <!-- Step 2: Scope -->
    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">2. Which designs should be processed?</h2>
      <div class="space-y-2">
        {#each scopeOptionsFor as option}
          {#if option.disabled}
            <div
              class="flex items-start gap-3 text-sm text-gray-400 rounded border border-dashed border-gray-300 p-3"
            >
              <input type="radio" disabled class="mt-1 h-4 w-4" />
              <div class="flex-1">
                <span class="font-semibold">{option.title}</span>
                <p class="text-xs mt-0.5">
                  {option.subtitle} <span class="font-medium text-indigo-500">(coming soon)</span>
                </p>
              </div>
            </div>
          {:else}
            <label
              class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3 {scope === option.id
                ? 'border-indigo-400 bg-indigo-50'
                : 'hover:bg-gray-50'}"
            >
              <input
                type="radio"
                name="tagging-scope"
                value={option.id}
                bind:group={scope}
                disabled={busyActive}
                class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500"
              />
              <div class="flex-1">
                <div class="flex items-center justify-between gap-3">
                  <span class="font-semibold">{option.title}</span>
                  {#if countsLoading}
                    <span class="text-xs text-gray-400 italic">(counting…)</span>
                  {:else if scopeCounts[option.id] !== undefined}
                    <span class="text-xs font-medium text-gray-500"
                      >{(activeCountFor(option.id) ?? 0).toLocaleString()} designs</span
                    >
                  {/if}
                </div>
                <p class="text-gray-500 text-xs mt-0.5">{option.subtitle}</p>
                {#if scopeCounts[option.id]}
                  <p class="text-xs text-gray-400 mt-0.5">
                    {scopeCounts[option.id].unverified_count.toLocaleString()} unverified &middot;{" "}
                    {scopeCounts[option.id].verified_count.toLocaleString()} verified
                  </p>
                {/if}
              </div>
            </label>
          {/if}
        {/each}
      </div>

      <!-- Folder selection (Specific Folder or Category scope) -->
      {#if scope === "folder"}
        <div class="rounded border border-gray-200 p-3 space-y-3">
          <p class="text-xs text-gray-600">
            Choose one or more library folders. Designs under any selected folder will
            be processed.
          </p>
          <button
            class="menu-button-secondary"
            onclick={chooseTaggingFolder}
            disabled={busyActive || countsLoading}
          >
            {selectedFolderPaths.length > 0 ? "Add folders…" : "Choose folders…"}
          </button>

          {#if selectedFolderPaths.length > 0}
            <ul class="space-y-1.5">
              {#each selectedFolderPaths as folderPath}
                <li
                  class="tagging-folder-row flex items-center gap-2 rounded border border-gray-200 bg-gray-50 px-2 py-1.5"
                  data-folder={folderPath}
                >
                  <span class="flex-1 font-mono text-xs text-gray-700 break-all">
                    {folderDisplayPath(folderPath)}
                  </span>
                  <button
                    type="button"
                    class="menu-button-secondary py-1 px-2 text-xs text-red-500 border-red-200"
                    onclick={() => removeTaggingFolder(folderPath)}
                    disabled={busyActive}
                    title="Remove this folder"
                  >
                    Remove
                  </button>
                </li>
              {/each}
            </ul>

            <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={includeSubfolders}
                onchange={() => loadScopeCounts()}
                disabled={busyActive}
                class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
              />
              <span>Include subfolders</span>
            </label>
          {/if}
          {#if dataStorageLocation}
            <p class="text-xs text-gray-400">
              Library: {dataStorageLocation}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Verified exclusion control -->
      <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3">
        <input
          type="checkbox"
          bind:checked={excludeVerified}
          disabled={busyActive}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
        />
        <div>
          <span class="font-semibold">Exclude human-verified designs (recommended)</span>
          <p class="text-gray-500 text-xs mt-0.5">
            Skip designs whose tags have been manually reviewed or marked as verified.
          </p>
        </div>
      </label>
    </div>

    <!-- Step 3: Merge -->
    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">3. What should happen to existing tags?</h2>
      <div class="space-y-2">
        {#each MERGE_OPTIONS as option}
          <label
            class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3 {merge === option.id
              ? 'border-indigo-400 bg-indigo-50'
              : 'hover:bg-gray-50'}"
          >
            <input
              type="radio"
              name="tagging-merge"
              value={option.id}
              bind:group={merge}
              disabled={busyActive}
              class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500"
            />
            <div>
              <span class="font-semibold">{option.title}</span>
              <p class="text-gray-500 text-xs mt-0.5">{option.subtitle}</p>
            </div>
          </label>
        {/each}
      </div>
      <p class="text-xs text-gray-400 italic">
        Retagging manages image tags only. Non-image tags are never removed. With
        "Add New Tags" (default), every existing tag is kept; "Complete Reset" replaces the
        image tags on the selected designs — including any you have added by hand.
      </p>
    </div>

    <!-- Advanced options (always visible) -->
    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">Advanced options</h2>

        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunStitching}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Also detect stitching tags</span>
            <p class="text-gray-500 text-xs mt-0.5">
              Analyze stitch coverage to tag designs (e.g. light fill, dense embroidery).
            </p>
          </div>
        </label>
        {#if taggingRunStitching}
          <label class="ml-8 flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingStitchingOverwrite}
              disabled={busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span>Overwrite stitching tags on already-processed designs</span>
          </label>
        {/if}

        <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={taggingRunImages}
            disabled={busyActive}
            class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <div>
            <span class="font-semibold">Also generate preview images</span>
            <p class="text-gray-500 text-xs mt-0.5">Generate preview images for designs that lack one.</p>
          </div>
        </label>
        {#if taggingRunImages}
          <label class="ml-8 flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={taggingImageRedo}
              disabled={busyActive}
              class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            />
            <span>Regenerate images for all designs, not just those without images</span>
          </label>
        {/if}

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
              Refresh thread colors, stitch totals, and color changes from the design files.
            </p>
          </div>
        </label>
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
              Refresh design dimensions and recommended hoop from the design files.
            </p>
          </div>
        </label>
    </div>

    <!-- Run / Stop Buttons -->
    <div class="flex flex-wrap items-center gap-3">
      <button
        class="menu-button-primary"
        onclick={() => (showConfirm = true)}
        disabled={taggingRunInFlight || taggingActionsLoading || busyActive}
      >
        {taggingRunInFlight ? taggingRunButtonLabel : "Review & Start Tagging"}
      </button>
      <button
        class="menu-button-secondary text-red-600 border-red-200 hover:bg-red-50"
        onclick={requestTaggingStop}
        disabled={!taggingRunInFlight}
      >
        Stop
      </button>
    </div>
    {:else}
    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">1. Target scope</h2>
      <p class="text-xs text-gray-500">
        All tasks below run against the population chosen here. <strong>Entire catalogue</strong>{" "}
        recomputes &amp; overwrites the data for every design; <strong>missing previews</strong>{" "}
        fills the gaps in the designs that lack a cached thumbnail. No tags are ever changed.
      </p>
      <div class="space-y-2">
        <label
          class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3 {maintenanceScope === 'all'
            ? 'border-indigo-400 bg-indigo-50'
            : 'hover:bg-gray-50'}"
        >
          <input
            type="radio"
            name="maintenance-scope"
            value="all"
            bind:group={maintenanceScope}
            disabled={busyActive}
            class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500"
          />
          <div class="flex-1">
            <span class="font-semibold">Entire catalogue</span>
            <p class="text-gray-500 text-xs mt-0.5">All designs stored in the local catalogue.</p>
          </div>
        </label>
        <label
          class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer rounded border border-gray-200 p-3 {maintenanceScope === 'missing_previews'
            ? 'border-indigo-400 bg-indigo-50'
            : 'hover:bg-gray-50'}"
        >
          <input
            type="radio"
            name="maintenance-scope"
            value="missing_previews"
            bind:group={maintenanceScope}
            disabled={busyActive}
            class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500"
          />
          <div class="flex-1">
            <div class="flex items-center justify-between gap-3">
              <span class="font-semibold">Designs missing preview images only</span>
              {#if missingPreviewCountLoading}
                <span class="text-xs text-gray-400 italic">(counting…)</span>
              {:else if missingPreviewCount !== null}
                <span class="text-xs font-medium text-gray-500"
                  >{missingPreviewCount.toLocaleString()} designs</span
                >
              {/if}
            </div>
            <p class="text-gray-500 text-xs mt-0.5">
              Targets only designs that lack a cached thumbnail.
            </p>
          </div>
        </label>
      </div>
    </div>

    <div class="bg-white rounded shadow p-6 space-y-4">
      <h2 class="text-base font-semibold text-gray-800">2. Maintenance tasks</h2>

      <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={taggingRunImages}
          disabled={busyActive}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
        />
        <div>
          <span class="font-semibold">Generate preview images</span>
        </div>
      </label>
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
            Refresh thread colors, stitch totals, and color changes from the design files.
          </p>
        </div>
      </label>
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
            Refresh design dimensions and recommended hoop from the design files.
          </p>
        </div>
      </label>
    </div>

    <!-- Run / Stop Buttons (Maintenance) -->
    <div class="flex flex-wrap items-center gap-3">
      <button
        class="menu-button-primary"
        onclick={() => (showMaintenanceConfirm = true)}
        disabled={!maintenanceValid || taggingRunInFlight || taggingActionsLoading || busyActive}
      >
        {taggingRunInFlight ? maintenanceRunButtonLabel : "Review & Start Maintenance"}
      </button>
      <button
        class="menu-button-secondary text-red-600 border-red-200 hover:bg-red-50"
        onclick={requestTaggingStop}
        disabled={!taggingRunInFlight}
      >
        Stop
      </button>
    </div>
    {/if}

    <!-- Last summary -->
    {#if taggingLastSummary}
      <div class="bg-white rounded shadow p-4 space-y-1 text-sm">
        <p class="font-semibold text-gray-800">Last run summary</p>
        <p>
          {lastRunWasMaintenance ? "Operations:" : "Processed:"}{" "}
          <strong>{taggingLastSummary.processed ?? 0}</strong> &middot; Errors:{" "}
          <strong>{taggingLastSummary.errors ?? 0}</strong>
        </p>
        {#if lastRunWasMaintenance}
          <p class="text-xs text-gray-500">
            A design may be counted once per selected task (e.g. previews + hoops = 2 operations on
            the same design). Tasks run: {(taggingLastSummary.actions || []).join(", ") || "—"}.
          </p>
        {/if}
        {#if taggingLastSummary.image_tag_count_before !== undefined}
          <p>
            Image tags: <strong>{taggingLastSummary.image_tag_count_before}</strong> before
            &rarr; <strong>{taggingLastSummary.image_tag_count_after ?? 0}</strong> after
          </p>
        {/if}
        {#if taggingLastSummary.stitching_tag_count_before !== undefined}
          <p>
            Stitching tags: <strong>{taggingLastSummary.stitching_tag_count_before}</strong> before
            &rarr; <strong>{taggingLastSummary.stitching_tag_count_after ?? 0}</strong> after
          </p>
        {/if}
        {#if (taggingLastSummary.actions || []).includes("images") &&
            taggingLastSummary.missing_preview_count_before !== undefined}
          <p>
            Needs attention:
            <strong>{taggingLastSummary.missing_preview_count_before}</strong> before
            &rarr; <strong>{taggingLastSummary.missing_preview_count_after ?? 0}</strong> after
          </p>
          {#if (taggingLastSummary.missing_preview_count_after ?? 0) > 0}
            <p class="mt-1">
              <button
                type="button"
                class="text-indigo-600 underline cursor-pointer"
                onclick={reviewNeedsAttentionInBrowse}
              >
                Review these in Browse
              </button>
              <span class="text-gray-500">
                (opens Browse with the “Needs attention” filter — designs without a preview)
              </span>
            </p>
          {/if}
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

  <!-- Pre-flight confirmation modal -->
  {#if showConfirm}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div
        class="bg-white rounded shadow-lg p-6 max-w-md w-full"
        role="dialog"
        aria-modal="true"
        data-testid="tagging-confirm-modal"
      >
        <h2 class="text-lg font-semibold text-gray-800">Ready to Retag</h2>
        <div class="mt-4 space-y-2 text-sm">
          <p><span class="font-semibold">Action:</span> {goalLabel}</p>
          <p>
            <span class="font-semibold">Target Scope:</span>
            {selectedCount !== null ? selectedCount.toLocaleString() : "…"} designs ({scopeLabel})
          </p>
          {#if scope === "folder" && selectedFolderPaths.length > 0}
            <p>
              <span class="font-semibold">Folder(s):</span>
              {selectedFolderPaths.length}
              {#if includeSubfolders}(incl. subfolders){/if}
            </p>
            <ul class="list-disc pl-5 text-xs text-gray-600 space-y-0.5">
              {#each selectedFolderPaths as folderPath}
                <li>{folderDisplayPath(folderPath)}</li>
              {/each}
            </ul>
          {/if}
          <p><span class="font-semibold">Tag Strategy:</span> {mergeLabel}</p>
          <p>
            <span class="font-semibold">Verified designs:</span>
            {excludeVerified ? "Excluded" : "Included"}
          </p>
          {#if estimateMinutes !== null}
            <p class="text-gray-500">
              <span class="font-semibold">Estimated Time:</span> ~{estimateMinutes} minutes
              {#if visualAiPaced}(paced to respect Gemini rate limits){/if}
            </p>
          {/if}
        </div>
        <div class="mt-6 flex justify-end gap-3">
          <button class="menu-button-secondary" onclick={() => (showConfirm = false)}>Cancel</button>
          <button class="menu-button-primary" onclick={confirmStartTagging}>Start Tagging</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Maintenance confirmation modal -->
  {#if showMaintenanceConfirm}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div
        class="bg-white rounded shadow-lg p-6 max-w-md w-full"
        role="dialog"
        aria-modal="true"
        data-testid="maintenance-confirm-modal"
      >
        <h2 class="text-lg font-semibold text-gray-800">Ready to Run Maintenance</h2>
        <div class="mt-4 space-y-2 text-sm">
          <p>
            <span class="font-semibold">Target Scope:</span> {maintenanceScopeLabel}
          </p>
          <p class="text-gray-600 text-xs">
            {maintenanceScope === "all"
              ? "Entire catalogue: every task recomputes and overwrites the data for all designs."
              : "Designs missing a preview image only."}
          </p>
          <p><span class="font-semibold">Maintenance Tasks:</span></p>
          <ul class="list-disc pl-5 text-xs text-gray-700 space-y-0.5">
            {#if taggingRunImages}
              <li>Generate preview images</li>
            {/if}
            {#if taggingRunColorCounts}
              <li>Recalculate colour / stitch counts</li>
            {/if}
            {#if taggingRunHoopDimensions}
              <li>Recalculate hoops / dimensions</li>
            {/if}
          </ul>
        </div>
        <div class="mt-6 flex justify-end gap-3">
          <button class="menu-button-secondary" onclick={() => (showMaintenanceConfirm = false)}>
            Cancel
          </button>
          <button class="menu-button-primary" onclick={confirmStartMaintenance}>
            Start Maintenance
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>