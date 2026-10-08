<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  interface PaginationProps {
    currentPage?: number;
    totalPages?: number;
    onPageChange: (page: number) => void;
    disabled?: boolean;
    ariaLabel?: string;
    windowSize?: number;
    showFirstLast?: boolean;
  }

  let {
    currentPage = 1,
    totalPages = 1,
    onPageChange,
    disabled = false,
    ariaLabel = "Pagination",
    windowSize = 2,
    showFirstLast = false,
  }: PaginationProps = $props();

  let pageTokens = $derived.by(() => {
    if (totalPages <= 1) {
      return [1];
    }
    const pages: (number | string)[] = [];
    const windowStart = Math.max(1, currentPage - windowSize);
    const windowEnd = Math.min(totalPages, currentPage + windowSize);

    if (showFirstLast && windowStart > 1) {
      pages.push(1);
      if (windowStart > 2) {
        pages.push("...");
      }
    }

    for (let page = windowStart; page <= windowEnd; page += 1) {
      pages.push(page);
    }

    if (showFirstLast && windowEnd < totalPages) {
      if (windowEnd < totalPages - 1) {
        pages.push("...");
      }
      pages.push(totalPages);
    }

    return pages;
  });
</script>

{#if totalPages > 1}
  <nav class="flex flex-wrap items-center gap-2 mt-2 text-sm no-print" aria-label={ariaLabel}>
    {#if showFirstLast}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] text-sm font-medium hover:bg-[var(--surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(1)}
        disabled={disabled || currentPage <= 1}
      >
        &lt;&lt; First
      </button>
    {/if}

    {#if currentPage > 1}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] text-sm font-medium hover:bg-[var(--surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(currentPage - 1)}
        {disabled}
      >
        ‹ Prev
      </button>
    {/if}

    {#each pageTokens as pageToken}
      {#if pageToken === "..."}
        <span class="px-1 text-[var(--text-muted)] font-medium select-none">...</span>
      {:else if pageToken === currentPage}
        <span
          class="px-3 py-1.5 min-h-[2rem] border border-[var(--control-accent)] rounded bg-[var(--control-accent)] text-white text-sm font-semibold inline-flex items-center justify-center"
          aria-current="page">{pageToken}</span
        >
      {:else if typeof pageToken === "number"}
        <button
          type="button"
          class="px-3 py-1.5 min-h-[2rem] rounded border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] text-sm font-medium hover:bg-[var(--surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          onclick={() => onPageChange(pageToken)}
          {disabled}
        >
          {pageToken}
        </button>
      {/if}
    {/each}

    {#if currentPage < totalPages}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] text-sm font-medium hover:bg-[var(--surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(currentPage + 1)}
        {disabled}
      >
        Next ›
      </button>
    {/if}

    {#if showFirstLast}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] text-sm font-medium hover:bg-[var(--surface-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(totalPages)}
        disabled={disabled || currentPage >= totalPages}
      >
        Last &gt;&gt;
      </button>
    {/if}
  </nav>
{/if}
