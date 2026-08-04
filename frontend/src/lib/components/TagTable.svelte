<script>
  import { deleteTag as removeTag, updateTag } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import { tagChangeStore } from "../stores/tagChangeStore.js";

  /**
   * @typedef {import("../types/ipc").AdminTagSummary} AdminTagSummary
   * @typedef {{ id: number, description: string, design_count: number }} TagRow
   */

  /** @type {{ tags: TagRow[], group: string, onRefresh: (force?: boolean) => Promise<void> }} */
  let { tags = [], group = "", onRefresh = async () => {} } = $props();

  /** @type {number | null} */
  let editingTagId = $state(null);
  let editingTagDescription = $state("");
  /** @type {number | null} */
  let pendingDeleteTagId = $state(null);

  /**
   * @param {TagRow} tag
   */
  function beginEdit(tag) {
    if (!tag) return;
    pendingDeleteTagId = null;
    editingTagId = Number(tag.id);
    editingTagDescription = String(tag.description || "");
  }

  function cancelEdit() {
    editingTagId = null;
    editingTagDescription = "";
  }

  /** @param {TagRow} tag */
  async function saveEdit(tag) {
    const id = Number(tag?.id);
    const description = editingTagDescription.trim();
    if (!description) {
      addToast("Enter a tag name.", "error");
      return;
    }

    const result = await updateTag(id, description);
    if (!result?.persisted) {
      addToast(`Could not update tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    cancelEdit();
    addToast("Tag updated.", "success");
    await onRefresh(true);
    tagChangeStore.flagTagRenamed(Number(tag?.design_count) > 0);
  }

  /**
   * @param {TagRow} tag
   */
  function requestDelete(tag) {
    if (!tag) return;
    cancelEdit();
    pendingDeleteTagId = Number(tag.id);
    if (Number(tag.design_count) > 0) {
      addToast(
        `Deleting '${tag.description}' will remove it from ${tag.design_count} design(s).`,
        "info"
      );
      return;
    }
    addToast(`Delete '${tag.description}'? Click confirm delete to continue.`, "info");
  }

  function cancelDelete() {
    pendingDeleteTagId = null;
  }

  /** @param {number} id */
  async function confirmDelete(id) {
    const result = await removeTag(id);
    if (!result?.persisted) {
      addToast(`Could not delete tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }
    pendingDeleteTagId = null;
    addToast("Tag deleted.", "success");
    await onRefresh(true);
    tagChangeStore.flagTagDeleted();
  }

  const sortedTags = $derived(
    [...tags].sort((a, b) => String(a.description).localeCompare(String(b.description), undefined, { sensitivity: "base" }))
  );
</script>

<table class="w-full text-sm text-left">
  <thead class="bg-gray-50 text-gray-700 font-semibold border-b text-xs">
    <tr>
      <th class="px-4 py-2.5">Description</th>
      <th class="px-4 py-2.5 text-right">Used by</th>
      <th class="px-4 py-2.5"></th>
    </tr>
  </thead>
  <tbody class="divide-y divide-gray-100">
    {#if sortedTags.length === 0}
      <tr>
        <td colspan="3" class="px-4 py-3 text-gray-400 italic">No {group} tags yet.</td>
      </tr>
    {:else}
      {#each sortedTags as tag}
        <tr class="hover:bg-gray-50">
          <td class="px-4 py-2 font-medium">
            {#if editingTagId === tag.id}
              <input
                type="text"
                bind:value={editingTagDescription}
                class="admin-input border rounded px-2 py-1 text-sm w-full font-sans"
              />
            {:else}
              {tag.description}
            {/if}
          </td>
          <td class="px-4 py-2 text-right text-gray-600 font-mono">{Number(tag.design_count) || 0}</td>
          <td class="px-4 py-2 text-right">
            <div class="flex justify-end gap-2.5 flex-wrap">
              {#if editingTagId === tag.id}
                <button
                  type="button"
                  class="text-indigo-650 hover:underline text-xs font-semibold"
                  onclick={() => saveEdit(tag)}
                >
                  Save
                </button>
                <button
                  type="button"
                  class="text-gray-500 hover:underline text-xs font-semibold"
                  onclick={cancelEdit}
                >
                  Cancel
                </button>
              {:else if pendingDeleteTagId === tag.id}
                <button
                  type="button"
                  class="text-red-600 hover:underline text-xs font-bold"
                  onclick={() => confirmDelete(tag.id)}
                >
                  Confirm delete
                </button>
                <button
                  type="button"
                  class="text-gray-500 hover:underline text-xs font-semibold"
                  onclick={cancelDelete}
                >
                  Cancel
                </button>
              {:else}
                <button
                  type="button"
                  class="text-indigo-655 hover:underline text-xs font-semibold"
                  onclick={() => beginEdit(tag)}
                >
                  Edit
                </button>
                <button
                  type="button"
                  class="text-red-400 hover:underline text-xs font-semibold"
                  onclick={() => requestDelete(tag)}
                >
                  Delete
                </button>
              {/if}
            </div>
          </td>
        </tr>
        {#if pendingDeleteTagId === tag.id}
          <tr class="bg-amber-50">
            <td colspan="3" class="px-4 py-2 text-xs text-amber-800">
              {#if Number(tag.design_count) > 0}
                This tag is used by {tag.design_count} design(s). If you delete it, those designs will no longer have the tag assigned.
              {:else}
                Confirm deletion for this tag.
              {/if}
            </td>
          </tr>
        {/if}
      {/each}
    {/if}
  </tbody>
</table>