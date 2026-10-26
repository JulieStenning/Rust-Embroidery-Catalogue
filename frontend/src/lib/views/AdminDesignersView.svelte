<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount } from "svelte";
  import {
    listDesigners,
    createDesigner,
    updateDesigner,
    deleteDesigner as removeDesigner,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { beginBusy, endBusy, busyState } from "../stores/busyStore.js";

  /** @typedef {{ id: number, name: string, designCount: number }} AdminEntityRow */

  /** When embedded (true), the standalone page title/description are hidden
   *  because the wrapping context provides the heading. */
  let { embedded = false } = $props();

  /** @type {AdminEntityRow[]} */
  let designers = $state([]);
  let newDesignerName = $state("");
  /** @type {HTMLInputElement | null} */
  let designerNameInput = $state(null);
  /** @type {number | null} */
  let editingDesignerId = $state(null);
  let editingDesignerName = $state("");
  /** @type {number | null} */
  let pendingDeleteDesignerId = $state(null);
  /** @type {number | null} */
  let deletingDesignerId = $state(null);
  /** @type {number | null} */
  let savingDesignerId = $state(null);
  let addingDesigner = $state(false);
  let adminLoading = $state(false);

  let busyActive = $derived($busyState.active);
  const anyActionBusy = $derived(
    deletingDesignerId !== null || savingDesignerId !== null || addingDesigner || busyActive
  );

  let canAddDesigner = $derived(!anyActionBusy && newDesignerName.trim().length > 0);
  let canClearDesignerForm = $derived(!anyActionBusy && newDesignerName.length > 0);

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
      designers = items.map((d) => ({
        id: Number(d.id),
        name: String(d.name || ""),
        designCount: Number(d.design_count || 0),
      }));
    } catch (e) {
      addToast(`Failed to load designers: ${e}`, "error");
    } finally {
      adminLoading = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addDesigner(event) {
    event.preventDefault();
    if (anyActionBusy) return;
    const name = newDesignerName.trim();
    if (!name) return;

    addingDesigner = true;
    beginBusy(`Adding designer '${name}'`);
    try {
      const result = await createDesigner(name);
      if (!result?.persisted) {
        addToast(`Could not add designer: ${result?.error || "Unknown error"}`, "error");
        return;
      }

      newDesignerName = "";
      designerNameInput?.focus();
      addToast("Designer added.", "success");
      await loadDesigners(true);
    } finally {
      addingDesigner = false;
      endBusy();
    }
  }

  /** @param {AdminEntityRow} designer */
  function beginEditDesigner(designer) {
    if (!designer || anyActionBusy) return;
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
    if (anyActionBusy) return;
    const name = editingDesignerName.trim();
    if (!name) {
      addToast("Enter a designer name.", "error");
      return;
    }

    savingDesignerId = id;
    beginBusy(`Updating designer '${name}'`);
    try {
      const result = await updateDesigner(id, name);
      if (!result?.persisted) {
        addToast(`Could not update designer: ${result?.error || "Unknown error"}`, "error");
        return;
      }

      cancelEditDesigner();
      addToast("Designer updated.", "success");
      await loadDesigners(true);
    } finally {
      savingDesignerId = null;
      endBusy();
    }
  }

  /** @param {AdminEntityRow} designer */
  function requestDeleteDesigner(designer) {
    if (!designer || anyActionBusy) return;
    cancelEditDesigner();
    pendingDeleteDesignerId = Number(designer.id);
    if (Number(designer.designCount) > 0) {
      addToast(
        `Deleting '${designer.name}' will clear assignment from ${designer.designCount} design(s).`,
        "info"
      );
      return;
    }
    addToast(`Delete '${designer.name}'? Click confirm delete to continue.`, "info");
  }

  function cancelDeleteDesigner() {
    pendingDeleteDesignerId = null;
  }

  /** @param {number} id */
  async function deleteDesigner(id) {
    if (!id || anyActionBusy) return;
    const target = designers.find((d) => d.id === id);
    deletingDesignerId = id;
    beginBusy(`Deleting designer '${target?.name || id}'`);
    try {
      const result = await removeDesigner(id);
      if (!result?.persisted) {
        addToast(`Could not delete designer: ${result?.error || "Unknown error"}`, "error");
        return;
      }
      pendingDeleteDesignerId = null;
      addToast("Designer deleted.", "success");
      await loadDesigners(true);
    } finally {
      deletingDesignerId = null;
      endBusy();
    }
  }

  function clearNewDesignerForm() {
    newDesignerName = "";
    designerNameInput?.focus();
  }

  onMount(() => {
    loadDesigners();
    designerNameInput?.focus();
  });
</script>

{#if !embedded}
  <div class="space-y-1 font-sans">
    <h1 class="ui-page-title admin-title text-2xl font-bold">Manage Designers</h1>
    <p class="text-[var(--text-muted)] text-sm">
      Designers are the creators or brands of embroidery designs.
    </p>
  </div>
{/if}

<div class="admin-card rounded shadow p-5 max-w-xl border mt-2">
  <h2 class="text-sm font-semibold text-[var(--text-primary)] mb-3">Add New Designer</h2>
  <form class="flex gap-2" onsubmit={addDesigner}>
    <input
      bind:this={designerNameInput}
      type="text"
      bind:value={newDesignerName}
      required
      disabled={anyActionBusy}
      placeholder="New designer name..."
      class="admin-input flex-1 border rounded px-3 py-2 text-sm disabled:opacity-50"
    />
    <button type="submit" class="menu-button-primary text-sm" disabled={!canAddDesigner}>
      {addingDesigner ? "Adding..." : "Add"}
    </button>
    <button
      type="button"
      class="menu-button-secondary text-sm"
      onclick={clearNewDesignerForm}
      disabled={!canClearDesignerForm}>Clear</button
    >
  </form>
</div>

<div class="admin-card rounded shadow overflow-hidden max-w-3xl border">
  <table class="w-full text-sm text-left">
    <thead
      class="bg-[var(--surface-card-subtle)] text-[var(--text-secondary)] font-semibold border-b border-[var(--border-default)] text-xs"
    >
      <tr>
        <th class="px-4 py-3">Name</th>
        <th class="px-4 py-3 text-right">Used by</th>
        <th class="px-4 py-3"></th>
      </tr>
    </thead>
    <tbody class="divide-y divide-[var(--border-subtle)]">
      {#if designers.length === 0}
        <tr>
          <td colspan="3" class="px-4 py-3 text-[var(--text-muted)] italic">No designers yet.</td>
        </tr>
      {:else}
        {#each designers as designer}
          <tr class="hover:bg-[var(--surface-hover)]">
            <td class="px-4 py-2 font-medium text-[var(--text-primary)]">
              {#if editingDesignerId === designer.id}
                <input
                  type="text"
                  bind:value={editingDesignerName}
                  disabled={anyActionBusy}
                  class="admin-input border rounded px-2 py-1 text-sm w-full font-sans disabled:opacity-50"
                />
              {:else}
                {designer.name}
              {/if}
            </td>
            <td class="px-4 py-2 text-right text-[var(--text-muted)] font-mono"
              >{designer.designCount}</td
            >
            <td class="px-4 py-2 text-right">
              <div class="flex justify-end gap-2.5 flex-wrap">
                {#if editingDesignerId === designer.id}
                  <button
                    type="button"
                    class="text-[var(--text-brand)] hover:underline text-xs font-semibold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={() => saveDesignerEdit(designer.id)}
                  >
                    {savingDesignerId === designer.id ? "Saving..." : "Save"}
                  </button>
                  <button
                    type="button"
                    class="text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:underline text-xs font-semibold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={cancelEditDesigner}
                  >
                    Cancel
                  </button>
                {:else if pendingDeleteDesignerId === designer.id}
                  <button
                    type="button"
                    class="text-[var(--notice-error-text)] hover:underline text-xs font-bold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={() => deleteDesigner(designer.id)}
                  >
                    {deletingDesignerId === designer.id ? "Deleting..." : "Confirm delete"}
                  </button>
                  <button
                    type="button"
                    class="text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:underline text-xs font-semibold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={cancelDeleteDesigner}
                  >
                    Cancel
                  </button>
                {:else}
                  <button
                    type="button"
                    class="text-[var(--text-brand)] hover:underline text-xs font-semibold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={() => beginEditDesigner(designer)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    class="text-[var(--notice-error-text)] hover:underline text-xs font-semibold disabled:opacity-50 disabled:no-underline"
                    disabled={anyActionBusy}
                    onclick={() => requestDeleteDesigner(designer)}
                  >
                    Delete
                  </button>
                {/if}
              </div>
            </td>
          </tr>
          {#if pendingDeleteDesignerId === designer.id}
            <tr class="bg-[var(--notice-warn-bg)]">
              <td colspan="3" class="px-4 py-2 text-xs text-[var(--notice-warn-text)]">
                {#if designer.designCount > 0}
                  This designer is currently used by {designer.designCount} design(s). If you delete it,
                  those designs will no longer have a designer assigned.
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
