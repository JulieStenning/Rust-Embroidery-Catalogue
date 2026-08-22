<script>
  import { createTag, setDesignTags } from "../api/commandAdapter";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { designSessionStore } from "../stores/designSessionStore.js";

  /**
   * @typedef {Object} TagOption
   * @property {number} id
   * @property {string} description
   * @property {string} [tag_group]
   */

  let {
    designId = 0,
    allTags = [],
    selectedTagIds = [],
    open = false,
    onClose = () => {},
  } = $props();

  let searchQuery = $state("");
  let createTagGroup = $state("image");
  let saveTimer = /** @type {ReturnType<typeof setTimeout> | null} */ (null);
  let modalSaving = $state(false);
  let createError = $state("");

  /** @type {TagOption[]} */
  let localAllTags = $state([]);
  /** @type {number[]} */
  let localSelectedIds = $state([]);

  // Sync props into local mutable state when modal opens
  $effect(() => {
    if (open) {
      localAllTags = Array.isArray(allTags) ? allTags.map((t) => ({ ...t })) : [];
      localSelectedIds = Array.isArray(selectedTagIds) ? [...selectedTagIds] : [];
      searchQuery = "";
      createError = "";
    }
  });

  /** @param {TagOption} tag */
  function tagChooserSelectionIncludes(tag) {
    return localSelectedIds.includes(tag.id);
  }

  /** @param {TagOption} tag @param {boolean} checked */
  function toggleTagChooserSelection(tag, checked) {
    const id = Number(tag.id);
    if (!Number.isFinite(id)) return;
    if (checked) {
      localSelectedIds = Array.from(new Set([...localSelectedIds, id]));
    } else {
      localSelectedIds = localSelectedIds.filter((v) => v !== id);
    }
    scheduleAutoSave();
  }

  function scheduleAutoSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      flushAutoSave();
    }, 300);
  }

  async function flushAutoSave() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (modalSaving || !designId) return;

    modalSaving = true;
    createError = "";
    try {
      // A single design's tag review/edit always marks BOTH categories verified
      // (Rule 1). The backend leaves each flag exactly as supplied; passing
      // explicit `true` for both is the intent of a full single-design save.
      const result = await setDesignTags(designId, localSelectedIds, {
        imageTagsVerified: true,
        stitchingTagsVerified: true,
      });
      if (result.persisted) {
        const allTagsLookup = localAllTags;
        const selectedTags = allTagsLookup.filter((t) => localSelectedIds.includes(t.id));
        const imageTags = selectedTags
          .filter((t) => t.tag_group === "image")
          .map((t) => t.description);
        const stitchingTags = selectedTags
          .filter((t) => t.tag_group === "stitching")
          .map((t) => t.description);
        const allTagDescriptions = selectedTags.map((t) => t.description);

        designSessionStore.trackMutation(designId, {
          tags: allTagDescriptions,
          imageTags,
          stitchingTags,
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      } else {
        createError = `Auto-save failed: ${result.message || "Unknown error"}`;
      }
    } catch (error) {
      createError = `Auto-save error: ${error}`;
    } finally {
      modalSaving = false;
    }
  }

  let queryTrimmed = $derived(searchQuery.trim());

  /** @type {{image: TagOption[], stitching: TagOption[], unclassified: TagOption[]}} */
  let filteredGrouped = $derived.by(() => {
    const q = queryTrimmed.toLowerCase();
    const filtered =
      q === "" ? localAllTags : localAllTags.filter((t) => t.description.toLowerCase().includes(q));
    return splitTagsByGroup(filtered);
  });

  let exactMatchExists = $derived(
    queryTrimmed.length > 0 &&
      localAllTags.some((t) => t.description.toLowerCase() === queryTrimmed.toLowerCase())
  );

  let showCreateButton = $derived(queryTrimmed.length > 0 && !exactMatchExists);

  async function handleCreateTag() {
    if (!queryTrimmed || modalSaving) return;
    modalSaving = true;
    createError = "";

    try {
      // Use the selected tag group (image / stitching)
      const result = await createTag(queryTrimmed, createTagGroup);
      if (!result.persisted || !result.item) {
        createError = `Could not create tag: ${result.error || "Unknown error"}`;
        return;
      }

      const newTag = {
        id: Number(result.item.id),
        description: String(result.item.description),
        tag_group: result.item.tag_group || createTagGroup,
      };

      // Insert into correct alphabetical position within localAllTags
      localAllTags = [...localAllTags, newTag].sort((a, b) =>
        a.description.toLowerCase().localeCompare(b.description.toLowerCase())
      );
      // Mark as checked
      localSelectedIds = Array.from(new Set([...localSelectedIds, newTag.id]));
      // Clear search
      searchQuery = "";
      createTagGroup = "image";
      // Persist immediately
      await flushAutoSave();
    } catch (error) {
      createError = `Create tag error: ${error}`;
    } finally {
      modalSaving = false;
    }
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key === "Escape") {
      handleClose();
    }
    if (event.key === "Enter" && showCreateButton) {
      event.preventDefault();
      handleCreateTag();
    }
  }

  async function handleClose() {
    await flushAutoSave();
    onClose();
  }

  async function handleDone() {
    await flushAutoSave();
    onClose();
  }

  function handleOverlayClick() {
    handleClose();
  }

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("tag-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
      },
    };
  }
</script>

{#if open}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="tag-modal-title"
    onkeydown={handleKeydown}
    tabindex="-1"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Close tag chooser"
      onclick={handleOverlayClick}
    ></button>

    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
    >
      <!-- Header with search -->
      <div
        class="tag-chooser-header"
        style="display:flex;flex-direction:column;align-items:stretch;gap:0.5rem;"
      >
        <div style="display:flex;align-items:center;justify-content:space-between;">
          <h2 id="tag-modal-title" class="text-lg font-semibold" style="margin:0;">
            Choose tags for this design
          </h2>
        </div>

        <!-- Search + Quick-create -->
        <div style="display:flex;flex-direction:column;gap:0.25rem;">
          <div class="relative" style="display:flex;align-items:center;gap:0.5rem;">
            <input
              type="text"
              placeholder="🔍 Search or create tag..."
              bind:value={searchQuery}
              onkeydown={handleKeydown}
              style="flex:1;border:1px solid #d1d5db;border-radius:0.375rem;padding:0.5rem 0.75rem;font-size:0.875rem;background:white;"
            />
            {#if modalSaving}
              <span class="text-xs text-gray-400 shrink-0">Saving...</span>
            {/if}
          </div>

          {#if showCreateButton}
            <div style="display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap;">
              <button
                type="button"
                class="menu-button-primary"
                style="display:inline-flex;align-items:center;gap:0.375rem;font-size:0.8125rem;padding:0.375rem 0.75rem;"
                onclick={handleCreateTag}
                disabled={modalSaving}
              >
                ➕ Create "{queryTrimmed}"
              </button>
              <span class="text-xs text-gray-500">as</span>
              <select
                bind:value={createTagGroup}
                class="text-xs border rounded px-1.5 py-1 bg-white"
                disabled={modalSaving}
              >
                <option value="image">Image</option>
                <option value="stitching">Stitching</option>
              </select>
            </div>
          {/if}

          {#if createError}
            <p class="text-red-600 text-xs mt-1">{createError}</p>
          {/if}
        </div>
      </div>

      <!-- Scrollable grid body -->
      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;margin-top:0.75rem;">
        <p class="text-sm font-medium" style="margin:0 0 0.75rem 0;">
          Design #{designId}
        </p>

        <div class="tag-chooser-sections">
          {#if filteredGrouped.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image font-semibold">
                Image tags
              </p>
              <div class="tag-chooser-grid">
                {#each filteredGrouped.image as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption)}
                      onchange={(event) =>
                        toggleTagChooserSelection(tagOption, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if filteredGrouped.stitching.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-stitching font-semibold"
              >
                Stitching tags
              </p>
              <div class="tag-chooser-grid">
                {#each filteredGrouped.stitching as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption)}
                      onchange={(event) =>
                        toggleTagChooserSelection(tagOption, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if filteredGrouped.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-unclassified font-semibold"
              >
                Unclassified tags
              </p>
              <div class="tag-chooser-grid">
                {#each filteredGrouped.unclassified as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption)}
                      onchange={(event) =>
                        toggleTagChooserSelection(tagOption, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if filteredGrouped.image.length === 0 && filteredGrouped.stitching.length === 0 && filteredGrouped.unclassified.length === 0 && !showCreateButton}
            <p class="text-xs text-gray-400 italic py-4 text-center">No matching tags found.</p>
          {/if}
        </div>
      </div>

      <!-- Footer: Done button only -->
      <div
        class="tag-chooser-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;margin-top:0.75rem;"
      >
        <button
          type="button"
          class="menu-button-primary"
          onclick={handleDone}
          disabled={modalSaving}
        >
          {modalSaving ? "Saving..." : "Done"}
        </button>
      </div>
    </div>
  </div>
{/if}
