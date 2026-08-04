<script>
  import { onMount } from "svelte";
  import { listTags, createTag } from "../api/commandAdapter";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { addToast } from "../stores/toastStore.js";
  import TagTable from "../components/TagTable.svelte";

  /** @typedef {import("../types/ipc").AdminTagSummary} AdminTagSummary */
  /** @typedef {{ id: number, description: string, tag_group: string, design_count: number }} TagRow */

  /** @type {TagRow[]} */
  let imageTags = $state([]);
  /** @type {TagRow[]} */
  let stitchingTags = $state([]);
  let newTagDescription = $state("");
  let newTagGroup = $state("image");
  let adminImageTagsOpen = $state(true);
  let adminStitchingTagsOpen = $state(true);
  let adminTagsPanelStateLoaded = $state(false);
  let tagsLoading = $state(false);

  /**
   * @template T
   * @param {{ items?: T[] } | null | undefined} response
   * @returns {T[]}
   */
  function getResponseItems(response) {
    const items = response?.items;
    return Array.isArray(items) ? items : [];
  }

  async function loadTags(force = false) {
    if (tagsLoading && !force) return;

    tagsLoading = true;
    try {
      const result = await listTags();
      const rawTags = /** @type {AdminTagSummary[]} */ (getResponseItems(result));
      const mappedTags = rawTags.map((t) => ({
        id: Number(t.id),
        description: String(t.description || ""),
        tag_group: String(t.tag_group || ""),
        design_count: Number(t.design_count ?? 0),
      }));

      const groups = splitTagsByGroup(mappedTags);
      imageTags = groups.image;
      stitchingTags = groups.stitching;
      adminTagsPanelStateLoaded = true;
    } catch (e) {
      addToast(`Failed to load tags: ${e}`, "error");
    } finally {
      tagsLoading = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addTag(event) {
    event.preventDefault();
    const desc = newTagDescription.trim();
    if (!desc) return;

    const result = await createTag(desc, newTagGroup);
    if (!result?.persisted) {
      addToast(`Could not add tag: ${result?.error || "Unknown error"}`, "error");
      return;
    }

    newTagDescription = "";
    addToast("Tag added.", "success");
    await loadTags(true);
  }

  /** @param {string} panel @param {Event} event */
  function handleAdminTagPanelToggle(panel, event) {
    const detailsNode = /** @type {HTMLDetailsElement | null} */ (event?.currentTarget);
    const isOpen = Boolean(detailsNode?.open);
    if (panel === "image") {
      adminImageTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.image", isOpen ? "open" : "closed");
      }
    }
    if (panel === "stitching") {
      adminStitchingTagsOpen = isOpen;
      if (typeof window !== "undefined") {
        window.localStorage.setItem("admin.tags.collapsible.stitching", isOpen ? "open" : "closed");
      }
    }
  }

  onMount(() => {
    // Hydrate saved collapsible panel states from localStorage (same logic as before)
    if (typeof window !== "undefined") {
      const imageSavedState = window.localStorage.getItem("admin.tags.collapsible.image");
      const stitchingSavedState = window.localStorage.getItem("admin.tags.collapsible.stitching");
      if (imageSavedState === "open" || imageSavedState === "closed") {
        adminImageTagsOpen = imageSavedState === "open";
      }
      if (stitchingSavedState === "open" || stitchingSavedState === "closed") {
        adminStitchingTagsOpen = stitchingSavedState === "open";
      }
    }
    loadTags();
  });
</script>

<section class="admin-page space-y-4">
  <h1 class="ui-page-title admin-title text-2xl font-bold text-gray-800">Manage Tags</h1>
  <p class="text-sm text-gray-500">
    Use Image tags for subject categories and Stitching tags for technique or style.
  </p>

  <div class="admin-card bg-white rounded shadow p-5 max-w-3xl border">
    <h2 class="text-sm font-semibold text-gray-700 mb-3">Add new tag</h2>
    <form class="flex flex-wrap gap-3 items-end" onsubmit={addTag}>
      <div>
        <label for="admin-tag-description" class="block text-xs font-semibold text-gray-650 mb-1">Description</label>
        <input
          id="admin-tag-description"
          type="text"
          bind:value={newTagDescription}
          required
          placeholder="e.g. Animals, Cross stitch..."
          class="admin-input border rounded px-3 py-2 text-sm w-56 font-sans"
        />
      </div>
      <div>
        <label for="admin-tag-group" class="block text-xs font-semibold text-gray-650 mb-1">Group</label>
        <select id="admin-tag-group" bind:value={newTagGroup} class="admin-input border rounded px-3 py-2 text-sm bg-white font-sans">
          <option value="image">Image</option>
          <option value="stitching">Stitching</option>
        </select>
      </div>
      <button type="submit" class="menu-button-primary text-sm py-2">Add</button>
    </form>
  </div>

  <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border" open={adminImageTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("image", event)}>
    <summary class="bg-green-50 border-b border-green-200 px-4 py-2.5 flex items-center gap-2 cursor-pointer select-none">
      <svg class={`h-4 w-4 text-green-700 transition-transform duration-200 ${adminImageTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
      </svg>
      <h2 class="text-sm font-bold text-green-800 tracking-wide">Image Tags</h2>
    </summary>
    <TagTable tags={imageTags} group="image" onRefresh={loadTags} />
  </details>

  <details class="admin-card bg-white rounded shadow overflow-hidden max-w-3xl border" open={adminStitchingTagsOpen} ontoggle={(event) => handleAdminTagPanelToggle("stitching", event)}>
    <summary class="bg-blue-50 border-b border-blue-200 px-4 py-2.5 flex items-center gap-2 cursor-pointer select-none">
      <svg class={`h-4 w-4 text-blue-700 transition-transform duration-200 ${adminStitchingTagsOpen ? "rotate-0" : "-rotate-90"}`} viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.176l3.71-3.946a.75.75 0 111.08 1.04l-4.25 4.52a.75.75 0 01-1.08 0l-4.25-4.52a.75.75 0 01.02-1.06z" clip-rule="evenodd"></path>
      </svg>
      <h2 class="text-sm font-bold text-blue-800 tracking-wide">Stitching Tags</h2>
    </summary>
    <TagTable tags={stitchingTags} group="stitching" onRefresh={loadTags} />
  </details>
</section>