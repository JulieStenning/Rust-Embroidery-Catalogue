<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  let {
    currentPage = 1,
    totalPages = 1,
    onPageChange,
    disabled = false,
    ariaLabel = "Pagination",
    windowSize = 2,
    showFirstLast = false,
  } = $props();

  let pageTokens = $derived.by(() => {
    if (totalPages <= 1) {
      return [1];
    }
    const pages = [];
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
        class="px-3 py-1.5 min-h-[2rem] rounded border border-gray-300 bg-white text-gray-800 text-sm font-medium hover:bg-gray-100 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(1)}
        disabled={disabled || currentPage <= 1}
      >
        &lt;&lt; First
      </button>
    {/if}

    {#if currentPage > 1}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-gray-300 bg-white text-gray-800 text-sm font-medium hover:bg-gray-100 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(currentPage - 1)}
        {disabled}
      >
        ‹ Prev
      </button>
    {/if}

    {#each pageTokens as pageToken}
      {#if pageToken === "..."}
        <span class="px-1 text-gray-500 font-medium select-none">...</span>
      {:else if pageToken === currentPage}
        <span
          class="px-3 py-1.5 min-h-[2rem] border border-indigo-600 rounded bg-indigo-600 text-white text-sm font-semibold inline-flex items-center justify-center"
          aria-current="page">{pageToken}</span
        >
      {:else}
        <button
          type="button"
          class="px-3 py-1.5 min-h-[2rem] rounded border border-gray-300 bg-white text-gray-800 text-sm font-medium hover:bg-gray-100 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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
        class="px-3 py-1.5 min-h-[2rem] rounded border border-gray-300 bg-white text-gray-800 text-sm font-medium hover:bg-gray-100 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(currentPage + 1)}
        {disabled}
      >
        Next ›
      </button>
    {/if}

    {#if showFirstLast}
      <button
        type="button"
        class="px-3 py-1.5 min-h-[2rem] rounded border border-gray-300 bg-white text-gray-800 text-sm font-medium hover:bg-gray-100 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={() => onPageChange(totalPages)}
        disabled={disabled || currentPage >= totalPages}
      >
        Last &gt;&gt;
      </button>
    {/if}
  </nav>
{/if}
