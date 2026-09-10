<script>
  import { restoreProgressStore } from "../stores/restoreProgressStore.js";
  import { requestCancelRestore } from "../api/commandAdapter";

  /** @type {{ onclose?: () => void }} */
  let { onclose = () => {} } = $props();

  let cancelling = $state(false);

  // Reset the cancel-in-flight flag whenever the run finishes.
  $effect(() => {
    if ($restoreProgressStore.terminal || !$restoreProgressStore.active) {
      cancelling = false;
    }
  });

  /** Ask the backend to stop the running phase after the current step. */
  async function handleCancel() {
    if (cancelling) return;
    cancelling = true;
    try {
      await requestCancelRestore();
    } catch {
      cancelling = false;
    }
  }

  /** @type {Record<string, string>} */
  const scopeSummaries = {
    database: "Restoring database",
    designs: "Syncing design files from backup",
    both: "Restoring database and design files",
    "import-unmatched": "Importing unmatched design files",
  };
  const stepLabels = ["Database", "Design files", "Reconcile"];
  /** @type {Record<string, number>} */
  const stepOrder = { database: 0, designs: 1, reconcile: 2 };

  /** @param {string} scope */
  function scopeSummary(scope) {
    return scopeSummaries[scope] || "Restoring catalogue";
  }

  /** @param {import("../stores/restoreProgressStore").RestoreProgressState} progress */
  function terminalTitle(progress) {
    if (progress.status === "rolled-back") return "Database restore rolled back";
    if (progress.error || progress.status === "failed") return "Restore failed";
    if (progress.status === "cancelled") {
      if (progress.scope === "designs") return "Design sync cancelled";
      if (progress.scope === "import-unmatched") return "Import cancelled";
      return "Restore cancelled";
    }
    if (progress.scope === "designs") return "Design sync complete";
    if (progress.scope === "import-unmatched") return "Import complete";
    if (progress.scope === "database") return "Database restore complete";
    return "Restore complete";
  }

  /** @param {number} percent */
  function percentText(percent) {
    const value = Number(percent) || 0;
    return `${Math.round(value * 100)}%`;
  }

  /** @param {string} phase @param {number} index */
  function stepMarker(phase, index) {
    const order = stepOrder[phase] ?? 0;
    if (index < order) return "✓";
    if (index === order) return "●";
    return "○";
  }

  /** File-copy metrics only apply to designs-sync / import phases. */
  /** @param {string} scope @param {string} phase */
  function showsFileMetrics(scope, phase) {
    if (scope === "designs" || scope === "import-unmatched") return true;
    if (scope === "both") {
      return phase === "designs" || phase === "reconcile" || phase === "completed";
    }
    return false;
  }
</script>

{#if $restoreProgressStore.active}
  {@const progress = $restoreProgressStore}
  {@const terminal = progress.terminal}
  {@const fileMetrics = showsFileMetrics(progress.scope, progress.phase)}
  <div
    class="settings-card backup-card bg-white rounded shadow p-6 space-y-3"
    data-testid="restore-progress-panel"
  >
    <h2 class="text-base font-semibold text-gray-800">
      {terminal ? terminalTitle(progress) : "Restore in progress"}
    </h2>
    <p class="text-sm text-gray-600">
      {scopeSummary(progress.scope)}{#if !terminal}…{/if}
    </p>

    {#if progress.scope === "both" && !terminal}
      <ol class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-600" data-testid="restore-steps">
        {#each stepLabels as label, index}
          <li class="flex items-center gap-1">
            <span aria-hidden="true">{stepMarker(progress.phase, index)}</span>
            <span>{label}</span>
          </li>
        {/each}
      </ol>
    {/if}

    {#if fileMetrics}
      <div class="w-full bg-gray-200 rounded-full h-2.5">
        <div
          class="bg-indigo-600 h-2.5 rounded-full"
          style={`width: ${percentText(progress.percent)}`}
        ></div>
      </div>
      <p class="text-xs text-gray-500">
        Copied {Number(progress.copied).toLocaleString()} ·
        Skipped {Number(progress.skipped).toLocaleString()} ·
        {percentText(progress.percent)}
      </p>
    {:else if !terminal}
      <p class="text-xs text-gray-500">Working…</p>
    {/if}

    {#if progress.error}
      <p class="text-xs text-red-600">{progress.error}</p>
    {/if}

    {#if !terminal && (progress.scope === "designs" || progress.scope === "both")}
      <div class="flex justify-end pt-1">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={handleCancel}
          disabled={cancelling || progress.phase !== "designs"}
          title={progress.phase !== "designs" ? "This step can't be interrupted" : undefined}
          data-testid="cancel-restore-button"
        >
          {cancelling ? "Cancelling…" : "Cancel"}
        </button>
      </div>
    {/if}

    {#if terminal}
      <div class="flex justify-end pt-1">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={onclose}
          data-testid="restore-progress-close"
        >
          Close
        </button>
      </div>
    {/if}
  </div>
{/if}
