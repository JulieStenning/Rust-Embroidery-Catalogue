<script>
  import { onMount } from "svelte";
  import {
    listDesigners,
    createDesigner,
    updateDesigner,
    deleteDesigner as removeDesigner,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";

  /** @typedef {{ id: number, name: string, designCount: number }} AdminEntityRow */

  /** @type {AdminEntityRow[]} */
  let designers = $state([]);
  let newDesignerName = $state("");
  /** @type {number | null} */
  let editingDesignerId = $state(null);
  let editingDesignerName = $state("");
  /** @type {number | null} */
  let pendingDeleteDesignerId = $state(null);
  let adminLoading = $state(false);

  let canAddDesigner = $derived(newDesignerName.trim().length > 0);
  let canClearDesignerForm = $derived(newDesignerName.length > 0);

  /**
   * @template T
   * @param {{ items?: T[] } | null | undefined} response
   * @returns {T[]}
   */
  function getResponseItems(response) {
    const items = response?.items;
    return Array.isArray(items) ? items : [];
  }

  async function loadDesigners(force = false) {
    if (adminLoading && !force) return;

    adminLoading = true;
    try {
      const result = await listDesigners();
      const items = getResponseItems(result);
      designers = items.map((d) => ({ id: Number(d.id), name: String(d.name || ""), designCount: Number(d.design_count || 0) }));
    } catch (e) {
      addToast(`Failed to load designers: ${e}`, "error");
    } finally {
      adminLoading = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addDesigner(event) {
    event.preventDefault();
    const name = newDesignerName.trim();
    if (!name) return;

    const result = await createDesigner(name);
    if (!result?.persisted) {
      addToast(`Could not add designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newDesignerName = "";
    addToast("Designer added.", "success");
    await loadDesigners(true);
  }

  /** @param {AdminEntityRow} designer */
  function beginEditDesigner(designer) {
    if (!designer) return;
    pendingDeleteDesignerId = null;
    editingDesignerId = Number(designer.id);
    editingDesignerName = String(designer.name || "");
  }

  function cancelEditDesigner() {
    editingDesignerId = null;
    editingDesignerName = "";
  }

  /** @param {number} id */
  async function saveDesignerEdit(id) {
    const name = editingDesignerName.trim();
    if (!name) {
      addToast("Enter a designer name.", "error");
      return;
    }

    const result = await updateDesigner(id, name);
    if (!result?.persisted) {
      addToast(`Could not update designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditDesigner();
    addToast("Designer updated.", "success");
    await loadDesigners(true);
  }

  /** @param {AdminEntityRow} designer */
  function requestDeleteDesigner(designer) {
    if (!designer) return;
    cancelEditDesigner();
    pendingDeleteDesignerId = Number(designer.id);
    if (Number(designer.designCount) > 0) {
      addToast(`Deleting '${designer.name}' will clear assignment from ${designer.designCount} design(s).`, "info");
      return;
    }
    addToast(`Delete '${designer.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteDesigner() {
    pendingDeleteDesignerId = null;
  }

  /** @param {number} id */
  async function deleteDesigner(id) {
    const result = await removeDesigner(id);
    if (!result?.persisted) {
      addToast(`Could not delete designer: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteDesignerId = null;
    addToast("Designer deleted.", "success");
    await loadDesigners(true);
  }

  function clearNewDesignerForm() {
    newDesignerName = "";
  }

  onMount(() => {
    loadDesigners();
  });
</script>

<div class="space-y-1 font-sans">
  <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800">Manage Designers</h1>
  <p class="text-gray-600 text-sm">Designers are the creators or brands of embroidery designs.</p>
</div>

<div class="admin-card bg-white rounded shadow p-5 max-w-xl border mt-2">
  <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new designer</h2>
  <form class="flex gap-2" onsubmit={addDesigner}>
    <input
      type="text"
      bind:value={newDesignerName}
      required
      placeholder="New designer name..."
      class="admin-input flex-1 border rounded px-3 py-2 text-sm"
    />
    <button type="submit" class="menu-button-primary text-sm" disabled={!canAddDesigner}>Add</button>
    <button type="button" class="menu-button-secondary text-sm" onclick={clearNewDesignerForm} disabled={!canClearDesignerForm}>Clear</button>
  </form>
</div>

<div class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border">
  <table class="w-full text-sm text-left">
    <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
      <tr>
        <th class="px-4 py-3">Name</th>
        <th class="px-4 py-3 text-right">Used by</th>
        <th class="px-4 py-3"></th>
      </tr>
    </thead>
    <tbody class="divide-y divide-gray-100">
      {#if designers.length === 0}
        <tr>
          <td colspan="3" class="px-4 py-3 text-gray-400 italic">No designers yet.</td>
        </tr>
      {:else}
        {#each designers as designer}
          <tr class="hover:bg-gray-50">
            <td class="px-4 py-2 font-medium">
              {#if editingDesignerId === designer.id}
                <input
                  type="text"
                  bind:value={editingDesignerName}
                  class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                />
              {:else}
                {designer.name}
              {/if}
            </td>
            <td class="px-4 py-2 text-right text-gray-600 font-mono">{designer.designCount}</td>
            <td class="px-4 py-2 text-right">
              <div class="flex justify-end gap-2.5 flex-wrap">
                {#if editingDesignerId === designer.id}
                  <button type="button" class="text-indigo-650 hover:underline text-xs font-semibold" onclick={() => saveDesignerEdit(designer.id)}>
                    Save
                  </button>
                  <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelEditDesigner}>
                    Cancel
                  </button>
                {:else if pendingDeleteDesignerId === designer.id}
                  <button type="button" class="text-red-600 hover:underline text-xs font-bold" onclick={() => deleteDesigner(designer.id)}>
                    Confirm delete
                  </button>
                  <button type="button" class="text-gray-500 hover:underline text-xs font-semibold" onclick={cancelDeleteDesigner}>
                    Cancel
                  </button>
                {:else}
                  <button type="button" class="text-indigo-655 hover:underline text-xs font-semibold" onclick={() => beginEditDesigner(designer)}>
                    Edit
                  </button>
                  <button type="button" class="text-red-400 hover:underline text-xs font-semibold" onclick={() => requestDeleteDesigner(designer)}>
                    Delete
                  </button>
                {/if}
              </div>
            </td>
          </tr>
          {#if pendingDeleteDesignerId === designer.id}
            <tr class="bg-amber-50">
              <td colspan="3" class="px-4 py-2 text-xs text-amber-800">
                {#if designer.designCount > 0}
                  This designer is currently used by {designer.designCount} design(s). If you delete it, those designs will no longer have a designer assigned.
                {:else}
                  Confirm deletion for this designer.
                {/if}
              </td>
            </tr>
          {/if}
        {/each}
      {/if}
    </tbody>
  </table>
</div>