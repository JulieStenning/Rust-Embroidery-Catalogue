<script>
  import { onMount, onDestroy } from "svelte";
  import SettingsView from "./SettingsView.svelte";
  import BackupView from "./BackupView.svelte";
  import OrphansView from "./OrphansView.svelte";
  import { parseSystemTab } from "../utils/routing.js";
  import { busyState } from "../stores/busyStore.js";

  // Lightweight system/maintenance hub shell. It owns ONLY the sub-tab bar +
  // child dispatch. No business logic, no IPC. The active sub-tab is always
  // derived from the hash URL (each sub-tab keeps its own URL, so it is
  // deep-linkable and survives refresh / Back / Forward). Children are
  // untouched and mount unchanged.

  const TABS = [
    { id: "settings", label: "Settings", route: "#/admin/system/settings" },
    { id: "backup", label: "Backup & Restore", route: "#/admin/system/backup" },
    { id: "orphans", label: "Orphaned Files", route: "#/admin/system/orphans" },
  ];

  let busyActive = $derived($busyState.active);
  let currentHash = $state("");

  function syncFromHash() {
    currentHash = String(window.location.hash || "");
  }

  const activeTab = $derived(parseSystemTab(currentHash) ?? "settings");

  /** Prevent a sub-tab from routing away while a long-running task runs. */
  function guardTabClick(event) {
    if (busyActive) {
      event.preventDefault();
    }
  }

  function tabLinkClass(active) {
    const base =
      "px-4 py-2 text-sm font-semibold rounded-t border inline-block select-none";
    const state = active
      ? "border-gray-200 border-b-transparent bg-white text-indigo-600"
      : "border-transparent text-gray-500 hover:text-gray-700 hover:bg-gray-50";
    const disabled = busyActive ? " opacity-50" : "";
    return `${base} ${state}${disabled}`;
  }

  onMount(() => {
    window.addEventListener("hashchange", syncFromHash);
    syncFromHash();
  });

  onDestroy(() => {
    window.removeEventListener("hashchange", syncFromHash);
  });
</script>

<div class="space-y-2">
  <div
    class="flex flex-wrap items-end gap-1 border-b border-gray-200"
    role="tablist"
    aria-label="System maintenance"
    data-testid="system-maintenance-tablist"
  >
    {#each TABS as tab (tab.id)}
      <a
        role="tab"
        href={tab.route}
        aria-selected={activeTab === tab.id}
        aria-disabled={busyActive}
        class={tabLinkClass(activeTab === tab.id)}
        data-testid={`system-tab-${tab.id}`}
        onclick={guardTabClick}
      >
        {tab.label}
      </a>
    {/each}
  </div>

  {#if activeTab === "settings"}
    <SettingsView />
  {:else if activeTab === "backup"}
    <BackupView />
  {:else if activeTab === "orphans"}
    <OrphansView />
  {/if}
</div>
