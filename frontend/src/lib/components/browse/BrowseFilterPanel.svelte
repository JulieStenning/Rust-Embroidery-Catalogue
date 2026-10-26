<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { HOOP_UNKNOWN_FILTER } from "../../utils/hoopConstants.js";
  import BrowseActiveFilterChips from "./BrowseActiveFilterChips.svelte";
  import BrowseFilterCheckboxList from "./BrowseFilterCheckboxList.svelte";

  /** @typedef {import("../../types/ipc").BrowseFilterPanelProps} BrowseFilterPanelProps */

  /** @type {BrowseFilterPanelProps} */
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

  let additionalFiltersActiveCount = $derived(
    (Array.isArray(browseFilters.designerFilters) ? browseFilters.designerFilters.length : 0) +
      (Array.isArray(browseFilters.imageTagFilters) ? browseFilters.imageTagFilters.length : 0) +
      (Array.isArray(browseFilters.stitchingTagFilters)
        ? browseFilters.stitchingTagFilters.length
        : 0) +
      (Array.isArray(browseFilters.sourceFilters) ? browseFilters.sourceFilters.length : 0) +
      (browseFilters.hoop ? 1 : 0) +
      (browseFilters.minWidth && browseFilters.minWidth.trim() !== "" ? 1 : 0) +
      (browseFilters.maxWidth && browseFilters.maxWidth.trim() !== "" ? 1 : 0) +
      (browseFilters.minHeight && browseFilters.minHeight.trim() !== "" ? 1 : 0) +
      (browseFilters.maxHeight && browseFilters.maxHeight.trim() !== "" ? 1 : 0) +
      (browseFilters.rating && browseFilters.rating.trim() !== "" ? 1 : 0) +
      (browseFilters.stitched && browseFilters.stitched !== "all" && browseFilters.stitched !== ""
        ? 1
        : 0) +
      (browseFilters.imageTagVerification && browseFilters.imageTagVerification !== "all" ? 1 : 0) +
      (browseFilters.stitchingTagVerification && browseFilters.stitchingTagVerification !== "all"
        ? 1
        : 0) +
      (browseFilters.needsAttention && browseFilters.needsAttention !== "off" ? 1 : 0)
  );

  /** @param {Event & { currentTarget: HTMLSelectElement }} e */
  function onVerificationChange(e) {
    onUpdateFilter("verificationStatus", e.currentTarget.value);
    onUpdateFilter("unverifiedOnly", e.currentTarget.value === "unverified");
  }
</script>

<form
  class="browse-search-shell space-y-3 no-print bg-[var(--surface-card)] rounded shadow p-4 border border-[var(--border-default)]"
  onsubmit={(event) => {
    event.preventDefault();
    onApplyFilters();
  }}
>
  <div class="ui-section-shell browse-general-search space-y-1.5">
    <div class="browse-general-search-row flex items-start gap-3">
      <div class="flex-1 min-w-[20rem] space-y-1.5">
        <label
          class="ui-section-label browse-general-search-label block text-xs font-semibold text-[var(--text-secondary)] uppercase"
          for="browse-q"
        >
          General search
        </label>
        <div class="relative w-full flex items-center">
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
      </div>
      <div class="shrink-0 space-y-1.5">
        <label
          for="browse-verification-select"
          class="ui-section-label block text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wide cursor-pointer"
        >
          Verification
        </label>
        <select
          id="browse-verification-select"
          class="ui-select-input browse-verification-select border border-[var(--border-default)] rounded px-2.5 py-2 text-xs bg-[var(--surface-input)] text-[var(--text-primary)] h-[38px] cursor-pointer"
          value={browseFilters.verificationStatus ||
            (browseFilters.unverifiedOnly ? "unverified" : "all")}
          onchange={onVerificationChange}
          data-testid="filter-verification-status"
          aria-label="Verification status"
        >
          <option value="all">All</option>
          <option value="unverified">Unverified only</option>
          <option value="verified">Verified only</option>
          <option value="partial">Partially verified</option>
        </select>
      </div>
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

  <BrowseActiveFilterChips {browseFilters} {onUpdateFilter} {onToggleFilter} {onClearFilters} />

  <details
    class="ui-section-shell browse-additional-filters overflow-visible relative"
    open={browseAdditionalFiltersOpen}
  >
    <summary
      class="ui-section-label browse-additional-summary cursor-pointer text-xs font-semibold text-[var(--text-secondary)] uppercase select-none list-none flex items-center gap-1.5"
      onclick={(event) => {
        event.preventDefault();
        onToggleAdditionalFilters();
      }}
    >
      <span>{browseAdditionalFiltersOpen ? "▼" : "▶"}</span>
      <span>Additional Filters</span>
      {#if additionalFiltersActiveCount > 0}
        <span
          class="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-[var(--control-accent)] text-white select-none leading-tight"
          data-testid="additional-filters-count-badge"
        >
          {additionalFiltersActiveCount}
        </span>
      {/if}
    </summary>
    <div
      class="grid sm:grid-cols-2 md:grid-cols-4 gap-4 pt-3 border-t border-[var(--border-subtle)] mt-2 px-4"
    >
      <!-- Designers Filter -->
      <BrowseFilterCheckboxList
        title="Designer"
        options={browseDesignerFilterOptions}
        selected={browseFilters.designerFilters}
        onToggle={(opt) => onToggleFilter("designerFilters", opt)}
      />

      <!-- Image Tags Filter -->
      <BrowseFilterCheckboxList
        title="Image tags"
        options={browseImageTagOptions}
        selected={browseFilters.imageTagFilters}
        onToggle={(opt) => onToggleFilter("imageTagFilters", opt)}
      />

      <!-- Stitching Tags Filter -->
      <BrowseFilterCheckboxList
        title="Stitching tags"
        options={browseStitchingTagOptions}
        selected={browseFilters.stitchingTagFilters}
        onToggle={(opt) => onToggleFilter("stitchingTagFilters", opt)}
      />

      <!-- Sources Filter -->
      <BrowseFilterCheckboxList
        title="Source"
        options={browseSourceFilterOptions}
        selected={browseFilters.sourceFilters}
        onToggle={(opt) => onToggleFilter("sourceFilters", opt)}
      />

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

        <div class="grid grid-cols-2 gap-2">
          <label class="block">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1"
              >Image tag status</span
            >
            <select
              id="filter-image-tag-verification"
              class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
              value={browseFilters.imageTagVerification || "all"}
              onchange={(e) => onUpdateFilter("imageTagVerification", e.currentTarget.value)}
              data-testid="filter-image-tag-verification"
              aria-label="Image tags verification"
            >
              <option value="all">Any</option>
              <option value="verified">Verified</option>
              <option value="unverified">Unverified</option>
            </select>
          </label>
          <label class="block">
            <span class="block font-semibold text-[var(--text-secondary)] mb-1"
              >Stitching tag status</span
            >
            <select
              id="filter-stitching-tag-verification"
              class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
              value={browseFilters.stitchingTagVerification || "all"}
              onchange={(e) => onUpdateFilter("stitchingTagVerification", e.currentTarget.value)}
              data-testid="filter-stitching-tag-verification"
              aria-label="Stitching tags verification"
            >
              <option value="all">Any</option>
              <option value="verified">Verified</option>
              <option value="unverified">Unverified</option>
            </select>
          </label>
        </div>

        <label class="block">
          <span class="block font-semibold text-[var(--text-secondary)] mb-1">Needs attention</span>
          <select
            class="ui-select-input border border-[var(--border-default)] rounded px-2.5 py-1.5 w-full bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
            value={browseFilters.needsAttention}
            onchange={(e) => onUpdateFilter("needsAttention", e.currentTarget.value)}
          >
            <option value="off">Off</option>
            <option value="all">All needing attention</option>
            <option value="cannot_draw">Cannot be drawn (missing preview)</option>
            <option value="master_only">Master files without stitch files</option>
          </select>
        </label>
        <p class="text-[11px] text-[var(--text-dim)] leading-snug">
          Filter by designs that cannot be drawn or master files awaiting stitch file export.
        </p>
      </div>
    </div>
  </details>

  <!-- Sorting and Columns -->
  <div
    class="flex flex-wrap items-center justify-between gap-3 pt-2 pb-4 text-xs border-t border-[var(--border-subtle)] text-[var(--text-secondary)] px-4"
  >
    <div class="flex flex-wrap items-end gap-3">
      <label class="block font-medium">
        <span class="block mb-1">Sort by:</span>
        <select
          class="ui-select-input border border-[var(--border-default)] rounded px-2 py-1 bg-[var(--surface-input)] text-xs text-[var(--text-primary)]"
          value={browseFilters.sortBy}
          onchange={(e) => onUpdateFilter("sortBy", e.currentTarget.value)}
        >
          <option value="date_added">Date Added</option>
          <option value="date_updated">Date Updated</option>
          <option value="folder">Folder</option>
          <option value="name">Name</option>
          <option value="rating">Rating</option>
          <option value="stitched">Stitched</option>
        </select>
      </label>
      <label class="block font-medium">
        <span class="block mb-1">Direction:</span>
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
        class="browse-search-reset-button text-[var(--text-brand)] hover:underline disabled:opacity-50 disabled:cursor-not-allowed pb-1.5"
        onclick={onClearFilters}
        disabled={browseFiltersAreDefault}>Reset filters</button
      >
    </div>
  </div>
</form>
