<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /**
   * @typedef {{ id: number, description: string, tag_group?: string | null }} TagItem
   */

  /**
   * @type {{
   *   id?: string,
   *   tags?: TagItem[],
   *   selectedTagId?: number | null,
   *   groupFilter?: string | null,
   *   placeholder?: string,
   *   disabled?: boolean,
   *   onSelect?: (tag: TagItem | null) => void
   * }}
   */
  let {
    id = "tag-combobox-input",
    tags = [],
    selectedTagId = $bindable(null),
    groupFilter = null,
    placeholder = "Type to search tags...",
    disabled = false,
    onSelect = () => {},
  } = $props();

  let searchTerm = $state("");
  let isOpen = $state(false);
  let highlightedIndex = $state(0);
  /** @type {HTMLInputElement | null} */
  let inputElement = $state(null);
  const listboxId = $derived(`${id}-listbox`);

  const filteredTags = $derived.by(() => {
    let list = tags;
    if (groupFilter) {
      const lowerGroup = groupFilter.toLowerCase();
      list = list.filter((t) => (t.tag_group || "").toLowerCase() === lowerGroup);
    }
    if (!searchTerm.trim()) {
      return list;
    }
    const term = searchTerm.toLowerCase();
    return list.filter((t) => t.description.toLowerCase().includes(term));
  });

  const selectedTag = $derived(tags.find((t) => t.id === selectedTagId) ?? null);

  $effect(() => {
    if (selectedTag && !isOpen) {
      searchTerm = selectedTag.description;
    }
  });

  /** @param {TagItem} tag */
  function chooseTag(tag) {
    selectedTagId = tag.id;
    searchTerm = tag.description;
    isOpen = false;
    onSelect(tag);
  }

  function findBestMatch() {
    const term = searchTerm.trim().toLowerCase();
    if (!term) return null;
    // 1. Exact match
    const exact = filteredTags.find((t) => t.description.toLowerCase() === term);
    if (exact) return exact;
    // 2. Single filtered item
    if (filteredTags.length === 1) return filteredTags[0];
    // 3. Highlighted item if open
    if (isOpen && filteredTags[highlightedIndex]) return filteredTags[highlightedIndex];
    return null;
  }

  function handleInput() {
    isOpen = true;
    highlightedIndex = 0;
  }

  function handleFocus() {
    if (!disabled) {
      isOpen = true;
      highlightedIndex = 0;
    }
  }

  /** @param {KeyboardEvent} e */
  function handleKeyDown(e) {
    if (e.key === "Tab") {
      const match = findBestMatch();
      if (match) {
        chooseTag(match);
      }
      return;
    }

    if (!isOpen) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        isOpen = true;
        e.preventDefault();
      }
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      highlightedIndex = (highlightedIndex + 1) % Math.max(1, filteredTags.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      highlightedIndex =
        (highlightedIndex - 1 + filteredTags.length) % Math.max(1, filteredTags.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const match = findBestMatch();
      if (match) {
        chooseTag(match);
      }
    } else if (e.key === "Escape") {
      isOpen = false;
    }
  }

  /** @param {FocusEvent} e */
  function handleBlur(e) {
    const match = findBestMatch();
    if (match && (!selectedTag || selectedTag.id !== match.id)) {
      chooseTag(match);
    }
    // Delay closing so mousedown on dropdown item can register
    setTimeout(() => {
      isOpen = false;
      if (selectedTag) {
        searchTerm = selectedTag.description;
      }
    }, 200);
  }
</script>

<div class="relative w-full">
  <div class="relative flex items-center">
    <input
      bind:this={inputElement}
      {id}
      type="text"
      role="combobox"
      aria-expanded={isOpen}
      aria-controls={listboxId}
      aria-autocomplete="list"
      {disabled}
      class="admin-input border rounded px-3 py-1.5 text-sm w-full bg-[var(--surface-primary)] text-[var(--text-primary)] border-[var(--border-default)] focus:border-[var(--brand-primary)] focus:outline-none"
      {placeholder}
      bind:value={searchTerm}
      oninput={handleInput}
      onfocus={handleFocus}
      onkeydown={handleKeyDown}
      onblur={handleBlur}
    />
    {#if selectedTag}
      <button
        type="button"
        class="absolute right-2 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] px-1 py-0.5 rounded"
        onclick={() => {
          selectedTagId = null;
          searchTerm = "";
          onSelect(null);
          inputElement?.focus();
        }}
        title="Clear selection"
      >
        ✕
      </button>
    {/if}
  </div>

  {#if isOpen && filteredTags.length > 0}
    <ul
      id={listboxId}
      role="listbox"
      class="absolute z-50 left-0 right-0 mt-1 max-h-56 overflow-y-auto bg-[var(--surface-card)] border border-[var(--border-default)] rounded-md shadow-lg py-1 text-sm list-none p-0 m-0"
    >
      {#each filteredTags as tag, index (tag.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          role="option"
          aria-selected={selectedTagId === tag.id}
          class="px-3 py-2 cursor-pointer flex items-center justify-between {index ===
          highlightedIndex
            ? 'bg-[var(--surface-hover)] text-[var(--text-brand)]'
            : 'text-[var(--text-primary)] hover:bg-[var(--surface-hover)]'} {selectedTagId ===
          tag.id
            ? 'font-semibold'
            : ''}"
          onmousedown={() => chooseTag(tag)}
          onclick={() => chooseTag(tag)}
          onmouseenter={() => (highlightedIndex = index)}
        >
          <span>{tag.description}</span>
          {#if tag.tag_group}
            <span
              class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded border {tag.tag_group.toLowerCase() ===
              'stitching'
                ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                : 'bg-indigo-50 text-indigo-700 border-indigo-200'}"
            >
              {tag.tag_group}
            </span>
          {/if}
        </li>
      {/each}
    </ul>
  {:else if isOpen && searchTerm.trim().length > 0}
    <div
      class="absolute z-50 left-0 right-0 mt-1 bg-[var(--surface-card)] border border-[var(--border-default)] rounded-md shadow-lg px-3 py-2 text-sm text-[var(--text-muted)]"
    >
      No matching tags found.
    </div>
  {/if}
</div>
