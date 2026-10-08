<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  interface Props {
    totalFilteredCount: number;
    selectedCountOnPage: number;
    totalCountOnPage: number;
    isAllSelectedOnPage: boolean;
    onToggleSelectAllPage: (checked: boolean) => void;
    busyActive?: boolean;
    isSearching?: boolean;
  }

  let {
    totalFilteredCount = 0,
    selectedCountOnPage = 0,
    totalCountOnPage = 0,
    isAllSelectedOnPage = false,
    onToggleSelectAllPage,
    busyActive = false,
    isSearching = false,
  }: Props = $props();
</script>

<div
  class="selection-header flex flex-wrap items-center gap-4 py-3 px-4 bg-[var(--surface-card-subtle)] border-b border-[var(--border-default)]"
>
  <div class="flex flex-wrap items-center gap-3">
    {#if isSearching}
      <span class="inline-flex items-center gap-1.5 text-sm font-medium text-[var(--text-brand)]">
        <svg
          class="animate-spin h-3.5 w-3.5 text-[var(--control-accent)]"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
        Searching designs...
      </span>
    {:else}
      <span class="text-sm font-medium text-[var(--text-secondary)]">
        {totalFilteredCount === 1 ? "1 design found" : `${totalFilteredCount} designs found`}
      </span>
    {/if}
    <span class="text-[var(--border-strong)] select-none" aria-hidden="true">•</span>
    <span class="text-sm font-medium text-[var(--text-secondary)]">
      {selectedCountOnPage} of {totalCountOnPage} selected
    </span>
    <label
      class="flex items-center gap-2 cursor-pointer select-none text-sm font-medium text-[var(--text-primary)]"
    >
      <input
        type="checkbox"
        class="ui-checkbox rounded cursor-pointer"
        checked={isAllSelectedOnPage}
        onchange={(e: Event) =>
          onToggleSelectAllPage((e.currentTarget as HTMLInputElement).checked)}
        disabled={totalCountOnPage === 0 || busyActive || isSearching}
      />
      Select all on page
    </label>
  </div>
</div>
