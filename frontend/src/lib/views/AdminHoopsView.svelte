<script>
  import { onMount } from "svelte";
  import {
    listHoops,
    createHoop,
    updateHoop,
    deleteHoop as removeHoop,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";

  /** @typedef {{ id: number, name: string, maxWidthMm: number, maxHeightMm: number, designCount: number }} HoopRow */

  /** @type {HoopRow[]} */
  let hoops = $state([]);
  let newHoopName = $state("");
  let newHoopWidth = $state(0);
  let newHoopHeight = $state(0);
  /** @type {number | null} */
  let editingHoopId = $state(null);
  let editingHoopName = $state("");
  let editingHoopWidth = $state(0);
  let editingHoopHeight = $state(0);
  /** @type {number | null} */
  let pendingDeleteHoopId = $state(null);
  let adminLoading = $state(false);

  let canAddHoop = $derived(newHoopName.trim().length > 0 && newHoopWidth > 0 && newHoopHeight > 0);
  let canClearHoopForm = $derived(newHoopName.length > 0 || newHoopWidth > 0 || newHoopHeight > 0);

  /**
   * @template T
   * @param {{ items?: T[] } | null | undefined} response
   * @returns {T[]}
   */
  function getResponseItems(response) {
    const items = response?.items;
    return Array.isArray(items) ? items : [];
  }

  async function loadHoops(force = false) {
    if (adminLoading && !force) return;

    adminLoading = true;
    try {
      const result = await listHoops();
      const items = getResponseItems(result);
      hoops = items.map((h) => ({ id: Number(h.id), name: String(h.name || ""), maxWidthMm: Number(h.max_width_mm || 0), maxHeightMm: Number(h.max_height_mm || 0), designCount: Number(h.design_count || 0) }));
    } catch (e) {
      addToast(`Failed to load hoops: ${e}`, "error");
    } finally {
      adminLoading = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addHoop(event) {
    event.preventDefault();
    const name = newHoopName.trim();
    const w = Number(newHoopWidth);
    const h = Number(newHoopHeight);
    if (!name || w <= 0 || h <= 0) return;

    const result = await createHoop(name, w, h);
    if (!result?.persisted) {
      addToast(`Could not add hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newHoopName = "";
    newHoopWidth = 0;
    newHoopHeight = 0;
    addToast("Hoop added.", "success");
    await loadHoops(true);
  }

  /** @param {HoopRow} hoop */
  function beginEditHoop(hoop) {
    if (!hoop) return;
    pendingDeleteHoopId = null;
    editingHoopId = Number(hoop.id);
    editingHoopName = String(hoop.name || "");
    editingHoopWidth = Number(hoop.maxWidthMm || 0);
    editingHoopHeight = Number(hoop.maxHeightMm || 0);
  }

  function cancelEditHoop() {
    editingHoopId = null;
    editingHoopName = "";
    editingHoopWidth = 0;
    editingHoopHeight = 0;
  }

  /** @param {number} id */
  async function saveHoopEdit(id) {
    const name = editingHoopName.trim();
    const w = Number(editingHoopWidth);
    const h = Number(editingHoopHeight);
    if (!name || w <= 0 || h <= 0) {
      addToast("Enter hoop details.", "error");
      return;
    }

    const result = await updateHoop(id, name, w, h);
    if (!result?.persisted) {
      addToast(`Could not update hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditHoop();
    addToast("Hoop updated.", "success");
    await loadHoops(true);
  }

  /** @param {HoopRow} hoop */
  function requestDeleteHoop(hoop) {
    if (!hoop) return;
    cancelEditHoop();
    pendingDeleteHoopId = Number(hoop.id);
    if (Number(hoop.designCount) > 0) {
      addToast(`Deleting '${hoop.name}' will clear assignment from ${hoop.designCount} design(s).`, "info");
      return;
    }
    addToast(`Delete '${hoop.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteHoop() {
    pendingDeleteHoopId = null;
  }

  /** @param {number} id */
  async function deleteHoop(id) {
    const result = await removeHoop(id);
    if (!result?.persisted) {
      addToast(`Could not delete hoop: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteHoopId = null;
    addToast("Hoop deleted.", "success");
    await loadHoops(true);
  }

  function clearNewHoopForm() {
    newHoopName = "";
    newHoopWidth = 0;
    newHoopHeight = 0;
  }

  onMount(() => {
    loadHoops();
  });
</script>

<h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800 font-sans">Manage Hoops</h1>
<p class="text-sm text-gray-500">
  Hoop sizes depend on your machine and the frames you own. Add your own hoops below.
</p>

<div class="admin-card bg-white rounded shadow p-5 max-w-4xl border mt-2">
  <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new hoop</h2>
  <form class="flex gap-3 items-end flex-wrap" onsubmit={addHoop}>
    <div>
      <label for="admin-hoop-name" class="block text-xs font-semibold text-gray-650 mb-1">Name</label>
      <input
        id="admin-hoop-name"
        type="text"
        bind:value={newHoopName}
        required
        placeholder="e.g. 5x7 hoop"
        class="admin-input border rounded px-3 py-2 text-sm w-52 font-sans"
      />
    </div>
    <div>
      <label for="admin-hoop-width" class="block text-xs font-semibold text-gray-650 mb-1">Max Width (mm)</label>
      <input
        id="admin-hoop-width"
        type="number"
        min="1"
        step="1"
        bind:value={newHoopWidth}
        required
        class="admin-input border rounded px-3 py-2 text-sm w-36 font-sans text-right"
      />
    </div>
    <div>
      <label for="admin-hoop-height" class="block text-xs font-semibold text-gray-650 mb-1">Max Height (mm)</label>
      <input
        id="admin-hoop-height"
        type="number"
        min="1"
        step="1"
        bind:value={newHoopHeight}
        required
        class="admin-input border rounded px-3 py-2 text-sm w-36 font-sans text-right"
      />
    </div>
    <button type="submit" class="menu-button-primary text-sm py-2" disabled={!canAddHoop}>Add</button>
    <button type="button" class="menu-button-secondary text-sm py-2" onclick={clearNewHoopForm} disabled={!canClearHoopForm}>Clear</button>
  </form>
</div>

<div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
  <table class="w-full text-sm text-left">
    <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
      <tr>
        <th class="px-4 py-3">Name</th>
        <th class="px-4 py-3 text-right">Max width (mm)</th>
        <th class="px-4 py-3 text-right">Max height (mm)</th>
        <th class="px-4 py-3 text-right">Used by</th>
        <th class="px-4 py-3"></th>
      </tr>
    </thead>
    <tbody class="divide-y divide-gray-100">
      {#if hoops.length === 0}
        <tr>
          <td colspan="5" class="px-4 py-3 text-gray-400 italic">No hoops defined yet. Add your own machine hoops above.</td>
        </tr>
      {:else}
        {#each hoops as hoop}
          <tr class="hover:bg-gray-50">
            <td class="px-4 py-2 font-medium">
              {#if editingHoopId === hoop.id}
                <input
                  type="text"
                  bind:value={editingHoopName}
                  class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                />
              {:else}
                {hoop.name}
              {/if}
            </td>
            <td class="px-4 py-2 text-right font-mono">
              {#if editingHoopId === hoop.id}
                <input
                  type="number"
                  min="1"
                  step="1"
                  bind:value={editingHoopWidth}
                  class="admin-input border rounded px-2 py-1 text-sm w-28 text-right font-mono"
                />
              {:else}
                {hoop.maxWidthMm.toFixed(0)}
              {/if}
            </td>
            <td class="px-4 py-2 text-right font-mono">
              {#if editingHoopId === hoop.id}
                <input
                  type="number"
                  min="1"
                  step="1"
                  bind:value={editingHoopHeight}
                  class="admin-input border rounded px-2 py-1 text-sm w-28 text-right font-mono"
                />
              {:else}
                {hoop.maxHeightMm.toFixed(0)}
              {/if}
            </td>
            <td class="px-4 py-2 text-right text-gray-600 font-mono">{hoop.designCount}</td>
            <td class="px-4 py-2 text-right">
              <div class="flex justify-end gap-2.5 flex-wrap">
                {#if editingHoopId === hoop.id}
                  <button type="button" class="text-indigo-650 hover:underline text-xs font-semibold" onclick={() => saveHoopEdit(hoop.id)}>
                    Save
                  </button>
                  <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelEditHoop}>
                    Cancel
                  </button>
                {:else if pendingDeleteHoopId === hoop.id}
                  <button type="button" class="text-red-600 hover:underline text-xs font-bold" onclick={() => deleteHoop(hoop.id)}>
                    Confirm delete
                  </button>
                  <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelDeleteHoop}>
                    Cancel
                  </button>
                {:else}
                  <button type="button" class="text-indigo-655 hover:underline text-xs font-semibold" onclick={() => beginEditHoop(hoop)}>
                    Edit
                  </button>
                  <button type="button" class="text-red-400 hover:underline text-xs font-semibold" onclick={() => requestDeleteHoop(hoop)}>
                    Delete
                  </button>
                {/if}
              </div>
            </td>
          </tr>
          {#if pendingDeleteHoopId === hoop.id}
            <tr class="bg-amber-50">
              <td colspan="5" class="px-4 py-2 text-xs text-amber-800">
                {#if hoop.designCount > 0}
                  This hoop is currently used by {hoop.designCount} design(s). If you delete it, those designs will no longer have a hoop assigned.
                {:else}
                  Confirm deletion for this hoop.
                {/if}
              </td>
            </tr>
          {/if}
        {/each}
      {/if}
    </tbody>
  </table>
</div>