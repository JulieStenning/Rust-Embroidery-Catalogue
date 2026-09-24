<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import {
    listTagSynonymsGrouped,
    addTagSynonyms,
    deleteTagSynonym,
    deleteAllTagSynonymsForTag,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import TagCombobox from "./TagCombobox.svelte";

  /**
   * @typedef {import("../types/ipc").AdminTagSynonymKeyword} AdminTagSynonymKeyword
   * @typedef {import("../types/ipc").AdminTagSynonymGroup} AdminTagSynonymGroup
   * @typedef {{ id: number, description: string, tag_group?: string | null }} TagOption
   */

  /**
   * @type {{
   *   open?: boolean,
   *   tagId?: number | null,
   *   tagDescription?: string,
   *   tagGroup?: string | null,
   *   allTags?: TagOption[],
   *   onClose?: () => void,
   *   onMatchesChanged?: () => void
   * }}
   */
  let {
    open = false,
    tagId = null,
    tagDescription = "",
    tagGroup = null,
    allTags = [],
    onClose = () => {},
    onMatchesChanged = () => {},
  } = $props();

  /** @type {number | null} */
  let activeTagId = $state(null);
  let wordsInput = $state("");
  let isSaving = $state(false);
  let isLoading = $state(false);
  /** @type {AdminTagSynonymKeyword[]} */
  let currentKeywords = $state([]);

  // Sync active tag ID whenever modal opens or props change
  $effect(() => {
    if (open) {
      activeTagId = tagId;
      wordsInput = "";
      if (activeTagId) {
        loadTagKeywords(activeTagId);
      } else {
        currentKeywords = [];
      }
    }
  });

  const activeTag = $derived(
    allTags.find((t) => t.id === activeTagId) ??
      (tagId ? { id: tagId, description: tagDescription, tag_group: tagGroup } : null)
  );

  /** @param {number} id */
  async function loadTagKeywords(id) {
    isLoading = true;
    try {
      const res = await listTagSynonymsGrouped();
      if (res?.items) {
        const group = res.items.find((g) => g.tag_id === id);
        currentKeywords = group?.keywords ? [...group.keywords] : [];
      }
    } catch (e) {
      addToast(`Could not load word matches: ${e}`, "error");
    } finally {
      isLoading = false;
    }
  }

  /** @param {SubmitEvent | MouseEvent} e */
  async function handleAddWords(e) {
    e.preventDefault();
    if (!activeTagId) {
      addToast("Please select a tag first.", "error");
      return;
    }
    const trimmed = wordsInput.trim();
    if (!trimmed) return;

    isSaving = true;
    try {
      const res = await addTagSynonyms(activeTagId, trimmed);
      if (res?.persisted && res.item) {
        currentKeywords = res.item;
        wordsInput = "";
        addToast("Word matches added.", "success");
        onMatchesChanged();
      } else {
        addToast(`Could not add matches: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not add matches: ${err}`, "error");
    } finally {
      isSaving = false;
    }
  }

  /** @param {number} synonymId */
  async function handleDeleteKeyword(synonymId) {
    try {
      const res = await deleteTagSynonym(synonymId);
      if (res?.persisted) {
        currentKeywords = currentKeywords.filter((k) => k.id !== synonymId);
        onMatchesChanged();
      } else {
        addToast(`Could not delete word match: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not delete word match: ${err}`, "error");
    }
  }

  async function handleClearAll() {
    if (!activeTagId) return;
    try {
      const res = await deleteAllTagSynonymsForTag(activeTagId);
      if (res?.persisted) {
        currentKeywords = [];
        addToast("All word matches cleared for this tag.", "success");
        onMatchesChanged();
      } else {
        addToast(`Could not clear matches: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not clear matches: ${err}`, "error");
    }
  }

  /** @param {KeyboardEvent} e */
  function handleKeyDown(e) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="tag-word-matches-title"
  >
    <!-- Backdrop button -->
    <button
      type="button"
      class="fixed inset-0 bg-black/50 backdrop-blur-sm cursor-default"
      aria-label="Close modal"
      onclick={onClose}
    ></button>

    <!-- Modal Dialog Box -->
    <div
      class="relative bg-[var(--surface-card)] text-[var(--text-primary)] border border-[var(--border-default)] rounded-xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh] z-10"
    >
      <!-- Header -->
      <div
        class="px-6 py-4 border-b border-[var(--border-default)] flex items-center justify-between"
      >
        <div>
          <h2 id="tag-word-matches-title" class="text-lg font-bold text-[var(--text-primary)]">
            Tag Word Matches
          </h2>
          <p class="text-xs text-[var(--text-muted)] mt-0.5">
            Words in file or folder names that will automatically assign this tag.
          </p>
        </div>
        <button
          type="button"
          class="text-[var(--text-muted)] hover:text-[var(--text-primary)] p-1 rounded-lg hover:bg-[var(--surface-hover)] text-lg leading-none"
          onclick={onClose}
          aria-label="Close modal"
        >
          ✕
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5 overflow-y-auto flex-1">
        <!-- Tag Selector (if not pre-locked) -->
        {#if !tagId}
          <div class="space-y-1.5">
            <label
              for="modal-target-tag-combobox"
              class="block text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider"
            >
              Target Tag
            </label>
            <TagCombobox
              id="modal-target-tag-combobox"
              tags={allTags}
              selectedTagId={activeTagId}
              groupFilter={tagGroup}
              onSelect={(tag) => {
                activeTagId = tag ? tag.id : null;
                if (activeTagId) {
                  loadTagKeywords(activeTagId);
                  setTimeout(() => {
                    document.getElementById("modal-words-input")?.focus();
                  }, 50);
                } else {
                  currentKeywords = [];
                }
              }}
            />
          </div>
        {:else if activeTag}
          <div
            class="flex items-center gap-2 bg-[var(--surface-hover)] px-3 py-2 rounded-lg border border-[var(--border-default)]"
          >
            <span class="font-semibold text-sm">{activeTag.description}</span>
            {#if activeTag.tag_group}
              <span
                class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded border {activeTag.tag_group.toLowerCase() ===
                'stitching'
                  ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                  : 'bg-indigo-50 text-indigo-700 border-indigo-200'}"
              >
                {activeTag.tag_group}
              </span>
            {/if}
          </div>
        {/if}

        <!-- Add Words Input -->
        {#if activeTagId}
          <form onsubmit={handleAddWords} class="space-y-2">
            <label
              for="modal-words-input"
              class="block text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider"
            >
              Add Words or Aliases
            </label>
            <div class="flex gap-2">
              <input
                id="modal-words-input"
                type="text"
                bind:value={wordsInput}
                placeholder="e.g. frog, toad, newt"
                disabled={isSaving}
                class="admin-input border rounded-lg px-3 py-1.5 text-sm flex-1 bg-[var(--surface-primary)] text-[var(--text-primary)] border-[var(--border-default)] focus:border-[var(--brand-primary)] focus:outline-none"
              />
              <button
                type="submit"
                disabled={isSaving || !wordsInput.trim()}
                class="px-4 py-1.5 text-sm font-medium rounded-lg bg-[var(--brand-primary)] text-white hover:bg-[var(--brand-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {isSaving ? "Adding..." : "Add"}
              </button>
            </div>
            <p class="text-[11px] text-[var(--text-muted)]">
              Separate multiple words with commas. Singular and plural forms match automatically.
            </p>
          </form>

          <!-- Word Chips Section -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span
                class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider"
              >
                Configured Words ({currentKeywords.length})
              </span>
              {#if currentKeywords.length > 0}
                <button
                  type="button"
                  onclick={handleClearAll}
                  class="text-xs text-red-500 hover:text-red-700 hover:underline font-medium"
                >
                  Clear All
                </button>
              {/if}
            </div>

            {#if isLoading}
              <p class="text-xs text-[var(--text-muted)] py-4 text-center">Loading words...</p>
            {:else if currentKeywords.length === 0}
              <div
                class="bg-[var(--surface-primary)] border border-dashed border-[var(--border-default)] rounded-lg p-4 text-center text-xs text-[var(--text-muted)]"
              >
                No custom word matches yet for this tag. Direct words in the tag name still match
                automatically.
              </div>
            {:else}
              <div
                class="flex flex-wrap gap-1.5 p-3 bg-[var(--surface-primary)] border border-[var(--border-default)] rounded-lg max-h-48 overflow-y-auto"
              >
                {#each currentKeywords as kw (kw.id)}
                  <span
                    class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-[var(--surface-card)] border border-[var(--border-default)] text-[var(--text-primary)] shadow-sm hover:border-[var(--brand-primary)] transition-colors"
                  >
                    <span>{kw.keyword}</span>
                    <button
                      type="button"
                      onclick={() => handleDeleteKeyword(kw.id)}
                      class="text-[var(--text-muted)] hover:text-red-500 rounded-full hover:bg-[var(--surface-hover)] p-0.5 leading-none transition-colors"
                      aria-label={`Remove ${kw.keyword}`}
                      title={`Remove ${kw.keyword}`}
                    >
                      ✕
                    </button>
                  </span>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="px-6 py-3.5 bg-[var(--surface-hover)] border-t border-[var(--border-default)] flex justify-end"
      >
        <button
          type="button"
          class="px-4 py-2 text-sm font-semibold rounded-lg bg-[var(--brand-primary)] text-white hover:bg-[var(--brand-hover)] transition-colors"
          onclick={onClose}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
