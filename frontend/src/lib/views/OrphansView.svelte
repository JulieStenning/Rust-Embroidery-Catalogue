<script>
  import { onMount } from "svelte";
  import {
    getOrphansPage,
    deleteOrphans as removeOrphans,
    deleteAllOrphans as removeAllOrphans,
    browseOrphanPath,
    openDesignInEditor,
    scanOrphans,
  } from "../api/commandAdapter";
  import Pagination from "../components/Pagination.svelte";
  import { addToast } from "../stores/toastStore.js";

  let orphansLoading = $state(false);
  let orphansLoaded = $state(false);
  let orphansError = $state("");
  /** @type {any[]} */
  let orphanItems = $state([]);
  let orphanPage = $state(1);
  let orphanPageSize = $state(100);
  let orphanTotal = $state(0);
  let orphanTotalPages = $state(1);
  /** @type {number[]} */
  let orphanSelectedIds = $state([]);

  /** @param {number} page */
  async function loadOrphansPage(page, force = false) {
    if (orphansLoading) return;
    if (!force && orphansLoaded && page === orphanPage) return;

    orphansLoading = true;
    orphansError = "";
    try {
      const result = await getOrphansPage({ page, pageSize: orphanPageSize });
      orphanItems = Array.isArray(result?.items) ? result.items : [];
      orphanPage = Math.max(1, Number(result?.page ?? page));
      orphanPageSize = Math.max(1, Number(result?.page_size ?? 100));
      orphanTotal = Math.max(0, Number(result?.total ?? 0));
      orphanTotalPages = Math.max(1, Number(result?.total_pages ?? 1));
      orphanSelectedIds = orphanItems.map((item) => Number(item.id));
      orphansLoaded = true;
    } catch (error) {
      orphansError = `Could not load orphans: ${error}`;
      orphanItems = [];
      orphanSelectedIds = [];
      orphanTotal = 0;
      orphanTotalPages = 1;
      orphansLoaded = false;
    } finally {
      orphansLoading = false;
    }
  }

  /** @param {number} id */
  function orphanIsSelected(id) {
    return orphanSelectedIds.includes(Number(id));
  }

  /** @param {number} id @param {boolean} checked */
  function toggleOrphanSelection(id, checked) {
    const normalizedId = Number(id);
    if (checked) {
      orphanSelectedIds = Array.from(new Set([...orphanSelectedIds, normalizedId]));
      return;
    }
    orphanSelectedIds = orphanSelectedIds.filter((value) => Number(value) !== normalizedId);
  }

  function selectAllOrphansOnPage() {
    orphanSelectedIds = orphanItems.map((item) => Number(item.id));
  }

  function deselectAllOrphansOnPage() {
    orphanSelectedIds = [];
  }

  /** @param {string} filepath */
  async function openOrphanPath(filepath) {
    const result = await browseOrphanPath(filepath);
    if (result.opened) {
      addToast(`Opened: ${result.opened}`, "success");
      return;
    }
    addToast(`Could not open folder: ${result?.error || "Unknown error"}`, "error");
  }

  async function deleteSelectedOrphans() {
    if (orphanSelectedIds.length === 0) {
      addToast("Select at least one orphan record first.", "error");
      return;
    }

    const confirmed = window.confirm(
      `Delete ${orphanSelectedIds.length} selected record(s)? This cannot be undone.`
    );
    if (!confirmed) return;

    const result = await removeOrphans(orphanSelectedIds);
    if (!result?.persisted) {
      addToast(`Could not delete selected orphans: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    addToast(`${result.deleted} record(s) deleted.`, "success");
    await loadOrphansPage(orphanPage, true);
    if (orphanPage > orphanTotalPages) {
      await loadOrphansPage(orphanTotalPages, true);
    }
  }

  async function deleteEveryOrphan() {
    if (orphanTotal <= 0) {
      addToast("There are no orphan records to delete.", "info");
      return;
    }

    const confirmed = window.confirm(
      `Delete ALL {orphanTotal} orphaned records? This cannot be undone.`
    );
    if (!confirmed) return;

    const result = await removeAllOrphans();
    if (!result?.persisted) {
      addToast(`Could not delete all orphans: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    addToast(`${result.deleted} record(s) deleted.`, "success");
    await loadOrphansPage(1, true);
  }

  /** @param {number} page */
  function goToOrphanPage(page) {
    const nextPage = Math.max(1, Math.min(orphanTotalPages, Number(page) || 1));
    if (nextPage === orphanPage) return;
    loadOrphansPage(nextPage, true);
  }

  /** @param {number} designId */
  function openOrphanDesign(designId) {
    const id = Number(designId);
    if (!Number.isFinite(id) || id <= 0) return;
    openDesignInEditor(id);
  }

  async function triggerDiskScan() {
    if (orphansLoading) return;
    orphansLoading = true;
    addToast("Scanning disk for orphaned records...", "info");
    try {
      const result = await scanOrphans();
      if (result?.source !== "rust" && result?.error) {
        throw new Error(result.error);
      }
      const orphanChecked = Number(result?.checked ?? 0);
      const orphanFound = Number(result?.found ?? 0);
      addToast(
        `Scan complete. Checked ${orphanChecked} file record(s). Found ${orphanFound} orphan(s).`,
        "success"
      );
      await loadOrphansPage(1, true);
    } catch (e) {
      addToast(`Could not complete scan: ${e}`, "error");
      orphansLoading = false;
    }
  }

  onMount(() => {
    loadOrphansPage(1);
  });
</script>

<div class="space-y-4">
  <div class="space-y-1 font-sans">
    <h1 class="ui-page-title text-2xl font-bold text-gray-800">Orphans</h1>
    <p class="text-gray-600 text-sm">
      Find and remove database records whose files no longer exist on disk.
    </p>
  </div>

  {#if orphansError}
    <div class="bg-red-50 border border-red-300 text-red-800 rounded px-4 py-2 text-sm">
      {orphansError}
    </div>
  {/if}

  <div class="flex items-center justify-between text-sm text-gray-600 flex-wrap gap-2">
    <span>
      {orphanTotal} orphaned record(s) total, page {orphanPage} of {orphanTotalPages}, showing {orphanItems.length}
    </span>
    <div class="flex gap-2">
      <button
        type="button"
        class="menu-button-primary"
        onclick={triggerDiskScan}
        disabled={orphansLoading}
      >
        {orphansLoading ? "Scanning..." : "Scan Disk"}
      </button>
      <button
        type="button"
        class="menu-button-secondary"
        onclick={() => loadOrphansPage(orphanPage, true)}
        disabled={orphansLoading}
      >
        Refresh
      </button>
      <button
        type="button"
        class="menu-button-secondary"
        onclick={selectAllOrphansOnPage}
        disabled={orphansLoading || orphanItems.length === 0}
      >
        Select all
      </button>
      <button
        type="button"
        class="menu-button-secondary"
        onclick={deselectAllOrphansOnPage}
        disabled={orphansLoading || orphanSelectedIds.length === 0}
      >
        Deselect all
      </button>
      <button
        type="button"
        class="menu-button-secondary"
        onclick={deleteSelectedOrphans}
        disabled={orphansLoading || orphanSelectedIds.length === 0}
      >
        Delete selected ({orphanSelectedIds.length})
      </button>
      <button
        type="button"
        class="menu-button-secondary text-red-600 border-red-200 hover:bg-red-50"
        onclick={deleteEveryOrphan}
        disabled={orphansLoading || orphanTotal === 0}
      >
        Delete all ({orphanTotal})
      </button>
    </div>
  </div>

  <div class="admin-table-shell overflow-auto max-h-[60vh] border rounded shadow bg-white">
    <table class="admin-table w-full text-left border-collapse text-sm">
      <thead>
        <tr class="bg-gray-50 border-b text-gray-700 font-semibold">
          <th class="px-4 py-2 w-10">Select</th>
          <th class="px-4 py-2 w-16 text-right">ID</th>
          <th class="px-4 py-2">Filename</th>
          <th class="px-4 py-2">Path</th>
          <th class="px-4 py-2 w-24 text-right">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-100">
        {#if orphanItems.length === 0}
          <tr>
            <td colspan="5" class="px-4 py-3 text-gray-400"
              >No orphaned records found. Refresh or scan to check.</td
            >
          </tr>
        {:else}
          {#each orphanItems as item}
            <tr class="hover:bg-gray-50">
              <td class="px-4 py-2">
                <input
                  type="checkbox"
                  class="rounded accent-indigo-500 cursor-pointer"
                  checked={orphanIsSelected(item.id)}
                  onchange={(e) => toggleOrphanSelection(item.id, e.currentTarget.checked)}
                />
              </td>
              <td class="px-4 py-2 text-right text-gray-500 font-mono">{item.id}</td>
              <td class="px-4 py-2 font-medium">
                <button
                  type="button"
                  class="text-indigo-600 hover:underline text-left font-medium"
                  onclick={() => openOrphanDesign(item.id)}
                  title="Click to view/edit in system editor (if file exists)"
                >
                  {item.filename || "Unknown"}
                </button>
              </td>
              <td class="px-4 py-2 text-xs text-gray-600 font-mono break-all">{item.filepath}</td>
              <td class="px-4 py-2 text-right">
                <button
                  type="button"
                  class="text-indigo-600 hover:text-indigo-800 text-xs font-semibold"
                  onclick={() => openOrphanPath(item.filepath)}
                >
                  Locate Folder
                </button>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <Pagination
    currentPage={orphanPage}
    totalPages={orphanTotalPages}
    onPageChange={goToOrphanPage}
    disabled={orphansLoading}
    showFirstLast={true}
    windowSize={3}
    ariaLabel="Orphans pagination"
  />
</div>
