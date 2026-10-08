<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { HOOP_UNKNOWN_FILTER } from "../../utils/hoopConstants.js";

  /** @typedef {import("../../types/ipc").BrowseFilterState} BrowseFilterState */
  /** @typedef {import("../../types/ipc").BrowseTagOption} BrowseTagOption */

  /**
   * @type {{
   *   browseFilters: BrowseFilterState,
   *   browseLoading: boolean,
   *   browseAdditionalFiltersOpen: boolean,
   *   browseFiltersAreDefault: boolean,
   *   browseDesignerFilterOptions: string[],
   *   browseImageTagOptions: BrowseTagOption[],
   *   browseStitchingTagOptions: BrowseTagOption[],
   *   browseSourceFilterOptions: string[],
   *   browseHoopFilterOptions: string[],
   *   onUpdateFilter: (key: string, value: any) => void,
   *   onToggleFilter: (key: "designerFilters" | "imageTagFilters" | "stitchingTagFilters" | "sourceFilters", value: string) => void,
   *   onToggleAdditionalFilters: () => void,
   *   onClearFilters: () => void,
   *   onCancelSearch: () => void,
   *   onClearSearchInput: () => void,
   *   onSearchKeyDown: (event: KeyboardEvent) => void,
   *   onApplyFilters: () => void
   * }}
   */
  let {
    browseFilters,
    browseLoading,
    browseAdditionalFiltersOpen,
    browseFiltersAreDefault,
    browseDesignerFilterOptions,
    browseImageTagOptions,
    browseStitchingTagOptions,
    browseSourceFilterOptions,
    browseHoopFilterOptions,
    onUpdateFilter,
    onToggleFilter,
    onToggleAdditionalFilters,
    onClearFilters,
    onCancelSearch,
    onClearSearchInput,
    onSearchKeyDown,
    onApplyFilters,
  } = $props();
</script>

<form
  class="browse-search-shell space-y-3 no-print bg-[var(--surface-card)] rounded shadow p-4 border border-[var(--border-default)]"
  onsubmit={(event) => {
    event.preventDefault();
    onApplyFilters();
  }}
>
  <div class="ui-section-shell browse-general-search space-y-1.5">
    <label
      class="ui-section-label browse-general-search-label block text-xs font-semibold text-[var(--text-secondary)] uppercase"
      for="browse-q">General search</label
    >
    <p></p>
    <div class="browse-general-search-row flex items-center gap-2">
      <div class="relative flex-1 min-w-[20rem] flex items-center">
        <input
          id="browse-q"
          class="ui-text-input ui-control-text-inset browse-general-input text-sm w-full font-mono border border-[var(--border-default)] rounded px-3 py-2 pr-24 bg-[var(--surface-input)] text-[var(--text-primary)]"
          placeholder="e.g. rose &quot;cross stitch&quot; -applique or *.hus"
          value={browseFilters.q}
          oninput={(event) => onUpdateFilter("q", event.currentTarget.value)}
          onkeydown={(event) => onSearchKeyDown(event)}
        />
        {#if browseLoading}
          <div class="absolute right-2 flex items-center gap-1.5 select-none">
            <svg
              class="animate-spin h-4 w-4 text-[var(--control-accent)]"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            <button
              type="button"
              class="text-xs font-medium text-[var(--notice-error-text)] bg-[var(--notice-error-bg)] border border-[var(--notice-error-border)] hover:bg-[var(--surface-hover)] px-1.5 py-0.5 rounded transition-colors"
              title="Cancel search (Esc)"
              aria-label="Cancel search"
              onclick={onCancelSearch}
            >
              Cancel
            </button>
          </div>
        {:else if browseFilters.q}
          <button
            type="button"
            class="absolute right-2 text-sm text-[var(--text-dim)] hover:text-[var(--text-primary)] px-1.5 py-0.5"
            title="Clear search"
            aria-label="Clear search"
            onclick={onClearSearchInput}
          >
            ✕
          </button>
        {/if}
      </div>
      <label
        class="ui-field-label browse-unverified-label flex items-center gap-1.5 cursor-pointer select-none text-sm text-[var(--text-primary)] whitespace-nowrap"
      >
        <input
          type="checkbox"
          class="ui-checkbox browse-unverified-checkbox rounded"
          checked={browseFilters.unverifiedOnly}
          onchange={(event) => onUpdateFilter("unverifiedOnly", event.currentTarget.checked)}
        />
        Unverified only
      </label>
    </div>
    <div
      class="browse-search-in-row flex flex-wrap items-center gap-4 text-xs text-[var(--text-primary)] my-1.5 py-1.5 px-3 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
    >
      <span class="font-semibold text-[var(--text-secondary)] uppercase text-[11px] tracking-wide"
        >Search in:</span
      >
      <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
        <input
          id="search-filename-checkbox"
          type="checkbox"
          class="ui-checkbox rounded cursor-pointer"
          checked={browseFilters.searchFilename}
          onchange={(event) => onUpdateFilter("searchFilename", event.currentTarget.checked)}
        />
        <span>File name</span>
      </label>
      <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
        <input
          id="search-folder-checkbox"
          type="checkbox"
          class="ui-checkbox rounded cursor-pointer"
          checked={browseFilters.searchFolder}
          onchange={(event) => onUpdateFilter("searchFolder", event.currentTarget.checked)}
        />
        <span>Folder name</span>
      </label>
      <label class="ui-field-label flex items-center gap-1.5 cursor-pointer select-none">
        <input
          id="search-tags-checkbox"
          type="checkbox"
          class="ui-checkbox rounded cursor-pointer"
          checked={browseFilters.searchTags}
          onchange={(event) => onUpdateFilter("searchTags", event.currentTarget.checked)}
        />
        <span>Tags</span>
      </label>
    </div>
    <p class="ui-help-note browse-general-help text-xs text-[var(--text-muted)] mt-0.5">
      Supports Google-like syntax: "exact phrase" · -exclude · word1 OR word2 · *.hus ·
      <a href="#/help?section=search" class="text-[var(--text-brand)] hover:underline"
        >Search help</a
      >
    </p>
  </div>

  <details
    class="ui-section-shell browse-additional-filters overflow-visible relative"
    open={browseAdditionalFiltersOpen}
  >
    <summary
      class="ui-section-label browse-additional-summary cursor-pointer text-xs font-semibold text-[var(--text-secondary)] uppercase select-none list-none flex items-center gap-1"
      onclick={(event) => {
        event.preventDefault();
        onToggleAdditionalFilters();
      }}
    >
      <span>{browseAdditionalFiltersOpen ? "▼" : "▶"}</span>
      <span>Additional Filters</span>
    </summary>
    <div
      class="grid sm:grid-cols-2 md:grid-cols-4 gap-4 pt-3 border-t border-[var(--border-subtle)] mt-2 px-4"
    >
      <!-- Designers Filter -->
      <div class="space-y-1">
        <span class="block text-xs font-semibold text-[var(--text-secondary)]">Designer</span>
        <div
          class="border border-[var(--border-default)] rounded bg-[var(--surface-input)] max-h-36 overflow-auto p-1.5 space-y-1"
        >
          {#each browseDesignerFilterOptions as opt}
            <label
              class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer"
            >
              <input
                type="checkbox"
                checked={browseFilters.designerFilters.includes(opt)}
                onchange={() => onToggleFilter("designerFilters", opt)}
                class="ui-checkbox rounded"
              />
              <span>{opt}</span>
            </label>
          {/each}
        </div>
      </div>

      <!-- Image Tags Filter -->
      <div class="space-y-1">
        <span class="block text-xs font-semibold text-[var(--text-secondary)]">Image tags</span>
        <div
          class="border border-[var(--border-default)] rounded bg-[var(--surface-input)] max-h-36 overflow-auto p-1.5 space-y-1"
        >
          {#each browseImageTagOptions as opt}
            <label
              class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer"
            >
              <input
                type="checkbox"
                checked={browseFilters.imageTagFilters.includes(opt.description)}
                onchange={() => onToggleFilter("imageTagFilters", opt.description)}
                class="ui-checkbox rounded"
              />
              <span>{opt.description}</span>
            </label>
          {/each}
        </div>
      </div>

      <!-- Stitching Tags Filter -->
      <div class="space-y-1">
        <span class="block text-xs font-semibold text-[var(--text-secondary)]">Stitching tags</span>
        <div
          class="border border-[var(--border-default)] rounded bg-[var(--surface-input)] max-h-36 overflow-auto p-1.5 space-y-1"
        >
          {#each browseStitchingTagOptions as opt}
            <label
              class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer"
            >
              <input
                type="checkbox"
                checked={browseFilters.stitchingTagFilters.includes(opt.description)}
                onchange={() => onToggleFilter("stitchingTagFilters", opt.description)}
                class="ui-checkbox rounded"
              />
              <span>{opt.description}</span>
            </label>
          {/each}
        </div>
      </div>

      <!-- Sources Filter -->
      <div class="space-y-1">
        <span class="block text-xs font-semibold text-[var(--text-secondary)]">Source</span>
        <div
          class="border border-[var(--border-default)] rounded bg-[var(--surface-input)] max-h-36 overflow-auto p-1.5 space-y-1"
        >
          {#each browseSourceFilterOptions as opt}
            <label
              class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer"
            >
              <input
                type="checkbox"
                checked={browseFilters.sourceFilters.includes(opt)}
                onchange={() => onToggleFilter("sourceFilters", opt)}
                class="ui-checkbox rounded"
              />
              <span>{opt}</span>
            </label>
          {/each}
        </div>
      </div>

      <!-- Other Properties & Dimensions -->
      <div class="space-y-2.5 text-xs sm:col-span-2 md:col-span-2">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <label class="block">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1">Hoop size</span>
            <select
              class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
              value={browseFilters.hoop}
              onchange={(e) => onUpdateFilter("hoop", e.currentTarget.value)}
            >
              <option value="">Any hoop</option>
              {#each browseHoopFilterOptions as opt}
                <option value={opt}>{opt}</option>
              {/each}
              <option value={HOOP_UNKNOWN_FILTER}>Hoop unknown</option>
            </select>
          </label>

          <div class="space-y-1">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1"
              >Dimensions (mm)</span
            >
            <div class="space-y-1.5">
              <div class="flex items-center gap-1.5">
                <span class="text-[11px] font-medium text-[var(--text-secondary)] w-3">W</span>
                <input
                  id="filter-min-width"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="Min"
                  aria-label="Minimum width (mm)"
                  class="ui-text-input border border-[var(--border-default)] rounded px-2 py-1 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
                  value={browseFilters.minWidth}
                  oninput={(e) => onUpdateFilter("minWidth", e.currentTarget.value)}
                />
                <span class="text-[var(--text-dim)] text-xs">–</span>
                <input
                  id="filter-max-width"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="Max"
                  aria-label="Maximum width (mm)"
                  class="ui-text-input border border-[var(--border-default)] rounded px-2 py-1 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
                  value={browseFilters.maxWidth}
                  oninput={(e) => onUpdateFilter("maxWidth", e.currentTarget.value)}
                />
              </div>
              <div class="flex items-center gap-1.5">
                <span class="text-[11px] font-medium text-[var(--text-secondary)] w-3">H</span>
                <input
                  id="filter-min-height"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="Min"
                  aria-label="Minimum height (mm)"
                  class="ui-text-input border border-[var(--border-default)] rounded px-2 py-1 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
                  value={browseFilters.minHeight}
                  oninput={(e) => onUpdateFilter("minHeight", e.currentTarget.value)}
                />
                <span class="text-[var(--text-dim)] text-xs">–</span>
                <input
                  id="filter-max-height"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="Max"
                  aria-label="Maximum height (mm)"
                  class="ui-text-input border border-[var(--border-default)] rounded px-2 py-1 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
                  value={browseFilters.maxHeight}
                  oninput={(e) => onUpdateFilter("maxHeight", e.currentTarget.value)}
                />
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2">
          <label class="block">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1">Minimum rating</span
            >
            <select
              class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
              value={browseFilters.rating}
              onchange={(e) => onUpdateFilter("rating", e.currentTarget.value)}
            >
              <option value="">Any</option>
              {#each [1, 2, 3, 4, 5] as score}
                <option value={String(score)}>{score}★</option>
              {/each}
            </select>
          </label>
          <label class="block">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1">Stitched</span>
            <select
              class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
              value={browseFilters.stitched}
              onchange={(e) => onUpdateFilter("stitched", e.currentTarget.value)}
            >
              <option value="">Any</option>
              <option value="yes">Stitched</option>
              <option value="no">Not Stitched</option>
            </select>
          </label>
        </div>

        <label class="flex items-center gap-2 pt-1 cursor-pointer text-[var(--text-primary)]">
          <input
            type="checkbox"
            class="ui-checkbox rounded"
            checked={browseFilters.needsAttention}
            onchange={(event) => onUpdateFilter("needsAttention", event.currentTarget.checked)}
          />
          <span>Needs attention</span>
        </label>
        <p class="text-[11px] text-[var(--text-dim)] leading-snug">
          Designs with no preview image or awaiting stitch file export.
        </p>
      </div>
    </div>
  </details>

  <!-- Sorting and Columns -->
  <div
    class="flex flex-wrap items-center justify-between gap-3 pt-2 pb-4 text-xs border-t border-[var(--border-subtle)] text-[var(--text-secondary)] px-4"
  >
    <div class="flex flex-wrap items-center gap-3">
      <label class="flex items-center gap-1.5 font-medium">
        Sort by:
        <select
          class="ui-select-input border border-[var(--border-default)] rounded px-2 py-1 bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
          value={browseFilters.sortBy}
          onchange={(e) => onUpdateFilter("sortBy", e.currentTarget.value)}
        >
          <option value="name">Name</option>
          <option value="rating">Rating</option>
          <option value="stitched">Stitched</option>
          <option value="folder">Folder</option>
          <option value="date_added">Date Added</option>
        </select>
      </label>
      <label class="flex items-center gap-1.5 font-medium">
        Direction:
        <select
          class="ui-select-input border border-[var(--border-default)] rounded px-2 py-1 bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
          value={browseFilters.sortDir}
          onchange={(e) => onUpdateFilter("sortDir", e.currentTarget.value)}
        >
          <option value="asc">Ascending</option>
          <option value="desc">Descending</option>
        </select>
      </label>
      <button
        type="button"
        class="browse-search-reset-button text-[var(--text-brand)] hover:underline disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={onClearFilters}
        disabled={browseFiltersAreDefault}>Reset filters</button
      >
    </div>
  </div>
</form>
