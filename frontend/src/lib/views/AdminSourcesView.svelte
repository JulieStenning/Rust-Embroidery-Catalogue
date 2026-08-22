<script>
  import { onMount } from "svelte";
  import {
    listSources,
    createSource,
    updateSource,
    deleteSource as removeSource,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";

  /** @typedef {{ id: number, name: string, designCount: number }} AdminEntityRow */

  /** When embedded (true), the standalone page title/description are hidden
   *  because the wrapping context provides the heading. */
  let { embedded = false } = $props();

  /** @type {AdminEntityRow[]} */
  let sources = $state([]);
  let newSourceName = $state("");
  /** @type {number | null} */
  let editingSourceId = $state(null);
  let editingSourceName = $state("");
  /** @type {number | null} */
  let pendingDeleteSourceId = $state(null);
  let adminLoading = $state(false);

  let canAddSource = $derived(newSourceName.trim().length > 0);
  let canClearSourceForm = $derived(newSourceName.length > 0);

  /**
   * @template T
   * @param {{ items?: T[] } | null | undefined} response
   * @returns {T[]}
   */
  function getResponseItems(response) {
    const items = response?.items;
    return Array.isArray(items) ? items : [];
  }

  async function loadSources(force = false) {
    if (adminLoading && !force) return;

    adminLoading = true;
    try {
      const result = await listSources();
      const items = getResponseItems(result);
      sources = items.map((s) => ({
        id: Number(s.id),
        name: String(s.name || ""),
        designCount: Number(s.design_count || 0),
      }));
    } catch (e) {
      addToast(`Failed to load sources: ${e}`, "error");
    } finally {
      adminLoading = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addSource(event) {
    event.preventDefault();
    const name = newSourceName.trim();
    if (!name) return;

    const result = await createSource(name);
    if (!result?.persisted) {
      addToast(`Could not add source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newSourceName = "";
    addToast("Source added.", "success");
    await loadSources(true);
  }

  /** @param {AdminEntityRow} source */
  function beginEditSource(source) {
    if (!source) return;
    pendingDeleteSourceId = null;
    editingSourceId = Number(source.id);
    editingSourceName = String(source.name || "");
  }

  function cancelEditSource() {
    editingSourceId = null;
    editingSourceName = "";
  }

  /** @param {number} id */
  async function saveSourceEdit(id) {
    const name = editingSourceName.trim();
    if (!name) {
      addToast("Enter a source name.", "error");
      return;
    }

    const result = await updateSource(id, name);
    if (!result?.persisted) {
      addToast(`Could not update source: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEditSource();
    addToast("Source updated.", "success");
    await loadSources(true);
  }

  /** @param {AdminEntityRow} source */
  function requestDeleteSource(source) {
    if (!source) return;
    cancelEditSource();
    pendingDeleteSourceId = Number(source.id);
    if (Number(source.designCount) > 0) {
      addToast(
        `Deleting '${source.name}' will clear assignment from ${source.designCount} design(s).`,
        "info"
      );
      return;
    }
    addToast(`Delete '${source.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteSource() {
    pendingDeleteSourceId = null;
  }

  /** @param {number} id */
  async function deleteSource(id) {
    const result = await removeSource(id);
    if (!result?.persisted) {
      addToast(`Could not delete source: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteSourceId = null;
    addToast("Source deleted.", "success");
    await loadSources(true);
  }

  function clearNewSourceForm() {
    newSourceName = "";
  }

  onMount(() => {
    loadSources();
  });
</script>

{#if !embedded}
  <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800 font-sans">
    Manage Sources
  </h1>
  <p class="text-sm text-gray-500">
    Sources describe where your designs came from, such as Purchased, Downloaded, or Gift.
  </p>
{/if}

<div class="admin-card bg-white rounded shadow p-5 max-w-xl border mt-2">
  <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new source</h2>
  <form class="flex gap-2" onsubmit={addSource}>
    <input
      type="text"
      bind:value={newSourceName}
      required
      placeholder="e.g. Purchased, Downloaded..."
      class="admin-input flex-1 border rounded px-3 py-2 text-sm"
    />
    <button type="submit" class="menu-button-primary text-sm" disabled={!canAddSource}>Add</button>
    <button
      type="button"
      class="menu-button-secondary text-sm"
      onclick={clearNewSourceForm}
      disabled={!canClearSourceForm}>Clear</button
    >
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
      {#if sources.length === 0}
        <tr>
          <td colspan="3" class="px-4 py-3 text-gray-400 italic">No sources yet.</td>
        </tr>
      {:else}
        {#each sources as source}
          <tr class="hover:bg-gray-50">
            <td class="px-4 py-2 font-medium">
              {#if editingSourceId === source.id}
                <input
                  type="text"
                  bind:value={editingSourceName}
                  class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
                />
              {:else}
                {source.name}
              {/if}
            </td>
            <td class="px-4 py-2 text-right text-gray-600 font-mono">{source.designCount}</td>
            <td class="px-4 py-2 text-right">
              <div class="flex justify-end gap-2.5 flex-wrap">
                {#if editingSourceId === source.id}
                  <button
                    type="button"
                    class="text-indigo-650 hover:underline text-xs font-semibold"
                    onclick={() => saveSourceEdit(source.id)}
                  >
                    Save
                  </button>
                  <button
                    type="button"
                    class="text-gray-500 hover:underline text-xs font-semibold"
                    onclick={cancelEditSource}
                  >
                    Cancel
                  </button>
                {:else if pendingDeleteSourceId === source.id}
                  <button
                    type="button"
                    class="text-red-600 hover:underline text-xs font-bold"
                    onclick={() => deleteSource(source.id)}
                  >
                    Confirm delete
                  </button>
                  <button
                    type="button"
                    class="text-gray-500 hover:underline text-xs font-semibold"
                    onclick={cancelDeleteSource}
                  >
                    Cancel
                  </button>
                {:else}
                  <button
                    type="button"
                    class="text-indigo-655 hover:underline text-xs font-semibold"
                    onclick={() => beginEditSource(source)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    class="text-red-400 hover:underline text-xs font-semibold"
                    onclick={() => requestDeleteSource(source)}
                  >
                    Delete
                  </button>
                {/if}
              </div>
            </td>
          </tr>
          {#if pendingDeleteSourceId === source.id}
            <tr class="bg-amber-50">
              <td colspan="3" class="px-4 py-2 text-xs text-amber-800">
                {#if source.designCount > 0}
                  This source is currently used by {source.designCount} design(s). If you delete it, those
                  designs will no longer have a source assigned.
                {:else}
                  Confirm deletion for this source.
                {/if}
              </td>
            </tr>
          {/if}
        {/each}
      {/if}
    </tbody>
  </table>
</div>
