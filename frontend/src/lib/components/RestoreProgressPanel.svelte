<script>
  import { restoreProgressStore } from "../stores/restoreProgressStore.js";

  const phaseLabels = {
    "db-swap": "Database swap",
    designs: "Sync design files",
    "import-unmatched": "Import unmatched files",
    completed: "Completed",
  };
  const statusLabels = {
    starting: "Starting…",
    syncing: "Syncing files…",
    restored: "Database restored",
    "rolled-back": "Rolled back to previous database",
  };

  function labelForPhase(phase) {
    return phaseLabels[phase] || phase || "…";
  }
  function labelForStatus(status) {
    return statusLabels[status] || status || "…";
  }
  function percentText(percent) {
    const value = Number(percent) || 0;
    return `${Math.round(value * 100)}%`;
  }
</script>

{#if $restoreProgressStore.active}
  <div
    class="settings-card backup-card bg-white rounded shadow p-6 space-y-3"
    data-testid="restore-progress-panel"
  >
    <h2 class="text-base font-semibold text-gray-800">Restore in progress</h2>
    <p class="text-sm text-gray-600">
      Phase:
      <span class="font-medium">{labelForPhase($restoreProgressStore.phase)}</span>
    </p>
    {#if $restoreProgressStore.dbStatus}
      <p class="text-sm text-gray-600">
        Status:
        <span class="font-medium">{labelForStatus($restoreProgressStore.dbStatus)}</span>
      </p>
    {/if}
    <div class="w-full bg-gray-200 rounded-full h-2.5">
      <div
        class="bg-indigo-600 h-2.5 rounded-full"
        style={`width: ${percentText($restoreProgressStore.percent)}`}
      ></div>
    </div>
    <p class="text-xs text-gray-500">
      Copied {Number($restoreProgressStore.copied).toLocaleString()} ·
      Skipped {Number($restoreProgressStore.skipped).toLocaleString()} ·
      {percentText($restoreProgressStore.percent)}
    </p>
    {#if $restoreProgressStore.error}
      <p class="text-xs text-red-600">{$restoreProgressStore.error}</p>
    {/if}
  </div>
{/if}
