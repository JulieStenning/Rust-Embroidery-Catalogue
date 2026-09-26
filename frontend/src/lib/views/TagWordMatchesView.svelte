<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount } from "svelte";
  import {
    listTagSynonymsGrouped,
    addTagSynonyms,
    deleteTagSynonym,
    deleteAllTagSynonymsForTag,
  } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";
  import TagCombobox from "../components/TagCombobox.svelte";
  import TagWordMatchModal from "../components/TagWordMatchModal.svelte";

  /**
   * @typedef {import("../types/ipc").AdminTagSynonymGroup} AdminTagSynonymGroup
   * @typedef {import("../types/ipc").AdminTagSynonymKeyword} AdminTagSynonymKeyword
   * @typedef {import("../types/ipc").AdminTagSummary} AdminTagSummary
   * @typedef {{ id: number, description: string, tag_group?: string | null }} TagOption
   */

  /** @type {AdminTagSynonymGroup[]} */
  let groups = $state([]);
  const allTags = $derived(
    groups.map((g) => ({
      id: Number(g.tag_id),
      description: String(g.tag_description || ""),
      tag_group: g.tag_group ? String(g.tag_group) : null,
    }))
  );
  let loading = $state(false);
  let searchQuery = $state("");
  let selectedGroupFilter = $state("all"); // 'all' | 'image' | 'stitching'
  let showMatchesOnly = $state(true);

  // Quick Add Bar State
  /** @type {number | null} */
  let quickAddTagId = $state(null);
  let quickAddWords = $state("");
  let quickAddSaving = $state(false);

  // Inline Quick Add per tag
  /** @type {Record<number, string>} */
  let inlineWordsInput = $state({});
  /** @type {Record<number, boolean>} */
  let inlineSaving = $state({});

  // Modal State
  let modalOpen = $state(false);
  /** @type {number | null} */
  let modalTagId = $state(null);
  let modalTagDescription = $state("");
  let modalTagGroup = $state("");

  async function loadData(force = false) {
    if (loading && !force) return;
    loading = true;
    try {
      const res = await listTagSynonymsGrouped();
      if (res?.items) {
        groups = res.items;
      }
    } catch (e) {
      addToast(`Failed to load word matches: ${e}`, "error");
    } finally {
      loading = false;
    }
  }

  async function reloadSynonyms() {
    try {
      const res = await listTagSynonymsGrouped();
      if (res?.items) {
        groups = res.items;
      }
    } catch (e) {
      addToast(`Failed to reload word matches: ${e}`, "error");
    }
  }

  onMount(() => {
    loadData(true);
  });

  // Filtered groups computation
  const filteredGroups = $derived.by(() => {
    let list = groups;

    // Filter by tag group
    if (selectedGroupFilter !== "all") {
      const lower = selectedGroupFilter.toLowerCase();
      list = list.filter((g) => (g.tag_group || "").toLowerCase() === lower);
    }

    // Filter by matches only
    if (showMatchesOnly) {
      list = list.filter((g) => g.keywords && g.keywords.length > 0);
    }

    // Filter by search query (across tag description and keyword chips)
    if (searchQuery.trim()) {
      const term = searchQuery.trim().toLowerCase();
      list = list.filter((g) => {
        const matchesTag = g.tag_description.toLowerCase().includes(term);
        const matchesWord = g.keywords.some((k) => k.keyword.toLowerCase().includes(term));
        return matchesTag || matchesWord;
      });
    }

    return list;
  });

  const totalMatchCount = $derived(
    groups.reduce((acc, g) => acc + (g.keywords ? g.keywords.length : 0), 0)
  );

  // Quick Add tag and keyword lookup
  const selectedQuickAddGroup = $derived(groups.find((g) => g.tag_id === quickAddTagId) ?? null);
  const selectedQuickAddTag = $derived(
    allTags.find((t) => t.id === quickAddTagId) ??
      (selectedQuickAddGroup
        ? {
            id: selectedQuickAddGroup.tag_id,
            description: selectedQuickAddGroup.tag_description,
            tag_group: selectedQuickAddGroup.tag_group,
          }
        : null)
  );
  const quickAddExistingKeywords = $derived(selectedQuickAddGroup?.keywords ?? []);

  // Parse entered words and detect overlaps
  const quickAddTypedWords = $derived(
    quickAddWords
      .split(/[\s,]+/)
      .map((w) => w.trim().toLowerCase())
      .filter(Boolean)
  );

  const quickAddDuplicateWords = $derived.by(() => {
    if (!quickAddExistingKeywords.length || !quickAddTypedWords.length) return [];
    const existingSet = new Set(quickAddExistingKeywords.map((k) => k.keyword.toLowerCase()));
    return Array.from(new Set(quickAddTypedWords.filter((w) => existingSet.has(w))));
  });

  /** @param {SubmitEvent | MouseEvent} e */
  async function handleQuickAdd(e) {
    e.preventDefault();
    if (!quickAddTagId) {
      addToast("Please select a tag first.", "error");
      return;
    }
    const trimmed = quickAddWords.trim();
    if (!trimmed) return;

    const targetTagId = quickAddTagId;
    quickAddSaving = true;
    try {
      const res = await addTagSynonyms(targetTagId, trimmed);
      if (res?.persisted) {
        if (res.item) {
          const target = groups.find((g) => g.tag_id === targetTagId);
          if (target) {
            target.keywords = res.item;
          }
        } else {
          await reloadSynonyms();
        }
        quickAddWords = "";
        addToast("Word matches added.", "success");
      } else {
        addToast(`Could not add matches: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not add matches: ${err}`, "error");
    } finally {
      quickAddSaving = false;
    }
  }

  /**
   * @param {number} tagId
   * @param {SubmitEvent | MouseEvent} e
   */
  async function handleInlineAdd(tagId, e) {
    e.preventDefault();
    const input = (inlineWordsInput[tagId] || "").trim();
    if (!input) return;

    inlineSaving[tagId] = true;
    try {
      const res = await addTagSynonyms(tagId, input);
      if (res?.persisted) {
        if (res.item) {
          const target = groups.find((g) => g.tag_id === tagId);
          if (target) {
            target.keywords = res.item;
          }
        } else {
          await reloadSynonyms();
        }
        inlineWordsInput[tagId] = "";
        addToast("Word match added.", "success");
      } else {
        addToast(`Could not add match: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not add match: ${err}`, "error");
    } finally {
      inlineSaving[tagId] = false;
    }
  }

  /**
   * @param {number} synonymId
   */
  async function handleDeleteKeyword(synonymId) {
    try {
      const res = await deleteTagSynonym(synonymId);
      if (res?.persisted) {
        for (const group of groups) {
          const idx = group.keywords.findIndex((k) => k.id === synonymId);
          if (idx !== -1) {
            group.keywords.splice(idx, 1);
            break;
          }
        }
      } else {
        addToast(`Could not delete match: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not delete match: ${err}`, "error");
    }
  }

  /**
   * @param {number} tagId
   */
  async function handleClearTagMatches(tagId) {
    try {
      const res = await deleteAllTagSynonymsForTag(tagId);
      if (res?.persisted) {
        const target = groups.find((g) => g.tag_id === tagId);
        if (target) {
          target.keywords = [];
        }
        addToast("Word matches cleared for tag.", "success");
      } else {
        addToast(`Could not clear matches: ${res?.error || "Unknown error"}`, "error");
      }
    } catch (err) {
      addToast(`Could not clear matches: ${err}`, "error");
    }
  }

  /**
   * @param {number | null} [tagId]
   * @param {AdminTagSynonymKeyword[]} [keywords]
   */
  function handleModalMatchesChanged(tagId, keywords) {
    if (tagId !== undefined && tagId !== null && keywords) {
      const target = groups.find((g) => g.tag_id === tagId);
      if (target) {
        target.keywords = [...keywords];
        return;
      }
    }
    reloadSynonyms();
  }

  /**
   * @param {AdminTagSynonymGroup} group
   */
  function openModalForGroup(group) {
    modalTagId = group.tag_id;
    modalTagDescription = group.tag_description;
    modalTagGroup = group.tag_group || "";
    modalOpen = true;
  }
</script>

<div class="space-y-6" data-testid="tag-word-matches-view">
  <!-- Header -->
  <div
    class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 border-b border-[var(--border-default)] pb-4"
  >
    <div>
      <h1 class="text-xl font-bold text-[var(--text-primary)]">Tag Word Matches</h1>
      <p class="text-xs text-[var(--text-muted)] mt-0.5">
        Define words and aliases in file or folder names that automatically assign tags during
        import.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <span
        class="text-xs font-medium px-2.5 py-1 rounded-full bg-[var(--surface-card)] border border-[var(--border-default)] text-[var(--text-secondary)]"
      >
        {totalMatchCount} total word {totalMatchCount === 1 ? "match" : "matches"}
      </span>
      <button
        type="button"
        onclick={() => {
          modalTagId = null;
          modalTagDescription = "";
          modalTagGroup = "";
          modalOpen = true;
        }}
        class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-[var(--brand-primary)] text-white hover:bg-[var(--brand-hover)] transition-colors inline-flex items-center gap-1.5 shadow-sm"
      >
        <span>➕ Add Matches</span>
      </button>
    </div>
  </div>

  <!-- Quick Add Card -->
  <div
    class="bg-[var(--surface-card)] border border-[var(--border-default)] rounded-xl p-4 shadow-sm space-y-3"
  >
    <h2 class="text-xs font-bold text-[var(--text-secondary)] uppercase tracking-wider">
      Quick Add Word Match
    </h2>
    <form onsubmit={handleQuickAdd} class="grid grid-cols-1 md:grid-cols-12 gap-3 items-end">
      <div class="md:col-span-5 space-y-1">
        <label
          for="quick-add-tag-combobox"
          class="block text-xs font-medium text-[var(--text-muted)]"
        >
          Select Tag
        </label>
        <TagCombobox
          id="quick-add-tag-combobox"
          tags={allTags}
          bind:selectedTagId={quickAddTagId}
          placeholder="Search for a tag..."
          onSelect={(tag) => {
            if (tag) {
              setTimeout(() => {
                document.getElementById("quick-add-words-input")?.focus();
              }, 50);
            }
          }}
        />
      </div>
      <div class="md:col-span-5 space-y-1">
        <label
          for="quick-add-words-input"
          class="block text-xs font-medium text-[var(--text-muted)]"
        >
          Words to Match (e.g. frog, toad, newt)
        </label>
        <input
          id="quick-add-words-input"
          type="text"
          bind:value={quickAddWords}
          placeholder={quickAddTagId
            ? "Enter words separated by commas (e.g. frog, toad, newt)..."
            : "Select a tag first..."}
          disabled={!quickAddTagId || quickAddSaving}
          class="admin-input border rounded-lg px-3 py-1.5 text-sm w-full bg-[var(--surface-primary)] text-[var(--text-primary)] border-[var(--border-default)] focus:border-[var(--brand-primary)] focus:outline-none disabled:opacity-60 disabled:cursor-not-allowed"
        />
      </div>
      <div class="md:col-span-2">
        <button
          type="submit"
          disabled={quickAddSaving || !quickAddTagId || !quickAddWords.trim()}
          class="w-full py-1.5 text-sm font-semibold rounded-lg bg-[var(--brand-primary)] text-white hover:bg-[var(--brand-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors shadow-sm"
        >
          {quickAddSaving ? "Adding..." : "Add Match"}
        </button>
      </div>
    </form>

    {#if quickAddTagId && selectedQuickAddTag}
      <div
        class="pt-3 border-t border-[var(--border-subtle)] space-y-2"
        data-testid="quick-add-existing-matches"
      >
        <div class="flex items-center justify-between flex-wrap gap-2">
          <div class="flex items-center gap-2 text-xs font-semibold text-[var(--text-secondary)]">
            <span>
              Existing matches for <span class="text-[var(--text-primary)]"
                >"{selectedQuickAddTag.description}"</span
              >:
            </span>
            <span class="text-[var(--text-muted)] font-normal font-mono">
              ({quickAddExistingKeywords.length}
              {quickAddExistingKeywords.length === 1 ? "word" : "words"})
            </span>
          </div>
          {#if quickAddDuplicateWords.length > 0}
            <span
              class="text-xs text-amber-600 dark:text-amber-400 font-medium flex items-center gap-1"
            >
              ⚠️ Already added: {quickAddDuplicateWords.join(", ")}
            </span>
          {/if}
        </div>

        {#if quickAddExistingKeywords.length > 0}
          <div class="flex flex-wrap gap-1.5 items-center">
            {#each quickAddExistingKeywords as kw (kw.id)}
              {@const isDupe = quickAddDuplicateWords.includes(kw.keyword.toLowerCase())}
              <span
                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border shadow-2xs transition-all {isDupe
                  ? 'bg-amber-50 dark:bg-amber-950/40 text-amber-800 dark:text-amber-200 border-amber-300 dark:border-amber-700 ring-2 ring-amber-400/50'
                  : 'bg-[var(--surface-primary)] border-[var(--border-default)] text-[var(--text-primary)] hover:border-[var(--brand-primary)]'}"
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
        {:else}
          <p class="text-xs italic text-[var(--text-muted)]">
            No word matches configured yet for "{selectedQuickAddTag.description}". Add some words
            above to start matching automatically.
          </p>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Search & Filter Controls -->
  <div
    class="flex flex-col sm:flex-row items-center justify-between gap-3 bg-[var(--surface-card)] border border-[var(--border-default)] rounded-xl p-3 shadow-sm"
  >
    <div class="flex-1 w-full sm:w-auto relative">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="🔍 Filter tags or keywords..."
        class="admin-input border rounded-lg pl-3 pr-8 py-1.5 text-sm w-full bg-[var(--surface-primary)] text-[var(--text-primary)] border-[var(--border-default)] focus:border-[var(--brand-primary)] focus:outline-none"
      />
      {#if searchQuery}
        <button
          type="button"
          class="absolute right-2.5 top-1/2 -translate-y-1/2 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          onclick={() => (searchQuery = "")}
        >
          ✕
        </button>
      {/if}
    </div>

    <div class="flex items-center gap-3 w-full sm:w-auto justify-between sm:justify-end">
      <!-- Tag Group Filter -->
      <div
        class="flex items-center rounded-lg border border-[var(--border-default)] p-0.5 bg-[var(--surface-primary)] text-xs"
      >
        <button
          type="button"
          class="px-2.5 py-1 rounded-md font-medium transition-colors {selectedGroupFilter === 'all'
            ? 'bg-[var(--surface-card)] text-[var(--text-brand)] shadow-xs font-semibold'
            : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'}"
          onclick={() => (selectedGroupFilter = "all")}
        >
          All
        </button>
        <button
          type="button"
          class="px-2.5 py-1 rounded-md font-medium transition-colors {selectedGroupFilter ===
          'image'
            ? 'bg-[var(--surface-card)] text-[var(--text-brand)] shadow-xs font-semibold'
            : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'}"
          onclick={() => (selectedGroupFilter = "image")}
        >
          Image
        </button>
        <button
          type="button"
          class="px-2.5 py-1 rounded-md font-medium transition-colors {selectedGroupFilter ===
          'stitching'
            ? 'bg-[var(--surface-card)] text-[var(--text-brand)] shadow-xs font-semibold'
            : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'}"
          onclick={() => (selectedGroupFilter = "stitching")}
        >
          Stitching
        </button>
      </div>

      <!-- Show Matches Only Toggle -->
      <label
        class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] select-none cursor-pointer"
      >
        <input
          type="checkbox"
          bind:checked={showMatchesOnly}
          class="rounded border-[var(--border-default)] text-[var(--brand-primary)] focus:ring-[var(--brand-primary)]"
        />
        <span>With matches only</span>
      </label>
    </div>
  </div>

  <!-- Grouped Tag Cards List -->
  {#if loading}
    <div class="p-12 text-center text-sm text-[var(--text-muted)]">Loading word matches...</div>
  {:else if filteredGroups.length === 0}
    <div
      class="bg-[var(--surface-card)] border border-dashed border-[var(--border-default)] rounded-xl p-12 text-center space-y-2"
    >
      <p class="text-sm font-semibold text-[var(--text-primary)]">No matching tags found</p>
      <p class="text-xs text-[var(--text-muted)]">
        {showMatchesOnly
          ? "No tags match your search with active word rules. Uncheck 'With matches only' to view all tags."
          : "No tags match your search criteria."}
      </p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each filteredGroups as group (group.tag_id)}
        <div
          class="bg-[var(--surface-card)] border border-[var(--border-default)] rounded-xl p-4 shadow-sm hover:border-[var(--border-hover)] transition-all space-y-3"
          data-testid={`tag-match-card-${group.tag_id}`}
        >
          <!-- Card Header -->
          <div class="flex items-center justify-between flex-wrap gap-2">
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-[var(--text-primary)]">{group.tag_description}</h3>
              {#if group.tag_group}
                <span
                  class="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full border {group.tag_group.toLowerCase() ===
                  'stitching'
                    ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                    : 'bg-indigo-50 text-indigo-700 border-indigo-200'}"
                >
                  {group.tag_group}
                </span>
              {/if}
              <span class="text-xs text-[var(--text-muted)]">
                ({group.keywords.length}
                {group.keywords.length === 1 ? "word" : "words"})
              </span>
            </div>

            <div class="flex items-center gap-3">
              <button
                type="button"
                onclick={() => openModalForGroup(group)}
                class="text-xs font-semibold text-[var(--text-brand)] hover:underline"
              >
                Manage
              </button>
              {#if group.keywords.length > 0}
                <button
                  type="button"
                  onclick={() => handleClearTagMatches(group.tag_id)}
                  class="text-xs font-medium text-red-500 hover:text-red-700 hover:underline"
                >
                  Clear All
                </button>
              {/if}
            </div>
          </div>

          <!-- Word Chips Grid -->
          {#if group.keywords.length > 0}
            <div class="flex flex-wrap gap-1.5 items-center">
              {#each group.keywords as kw (kw.id)}
                <span
                  class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-[var(--surface-primary)] border border-[var(--border-default)] text-[var(--text-primary)] shadow-xs hover:border-[var(--brand-primary)] transition-colors"
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
          {:else}
            <p class="text-xs italic text-[var(--text-muted)]">No custom word matches yet.</p>
          {/if}

          <!-- Inline Add Field -->
          <form
            onsubmit={(e) => handleInlineAdd(group.tag_id, e)}
            class="flex items-center gap-2 pt-1"
          >
            <input
              type="text"
              bind:value={inlineWordsInput[group.tag_id]}
              placeholder="+ Add word (e.g. alias) and press Enter..."
              disabled={Boolean(inlineSaving[group.tag_id])}
              class="admin-input border rounded-lg px-2.5 py-1 text-xs w-64 bg-[var(--surface-primary)] text-[var(--text-primary)] border-[var(--border-default)] focus:border-[var(--brand-primary)] focus:outline-none"
            />
            {#if (inlineWordsInput[group.tag_id] || "").trim().length > 0}
              <button
                type="submit"
                disabled={Boolean(inlineSaving[group.tag_id])}
                class="px-2.5 py-1 text-xs font-semibold rounded-lg bg-[var(--brand-primary)] text-white hover:bg-[var(--brand-hover)] transition-colors shadow-xs"
              >
                {inlineSaving[group.tag_id] ? "Adding..." : "Add"}
              </button>
            {/if}
          </form>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Modal Dialog Component -->
<TagWordMatchModal
  open={modalOpen}
  tagId={modalTagId}
  tagDescription={modalTagDescription}
  tagGroup={modalTagGroup}
  {allTags}
  onClose={() => (modalOpen = false)}
  onMatchesChanged={handleModalMatchesChanged}
/>
