<script>
  let { currentPage = 1, totalPages = 1, onPageChange, disabled = false, ariaLabel = "Pagination", windowSize = 2, showFirstLast = false } = $props();

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
        class="px-3 py-1 rounded border text-sm hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none"
        onclick={() => onPageChange(1)}
        disabled={disabled || currentPage <= 1}
      >
        &lt;&lt; First
      </button>
    {/if}
    
    {#if currentPage > 1}
      <button
        type="button"
        class="px-3 py-1 rounded border text-sm hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none"
        onclick={() => onPageChange(currentPage - 1)}
        disabled={disabled}
      >
        ‹ Prev
      </button>
    {/if}

    {#each pageTokens as pageToken}
      {#if pageToken === "..."}
        <span class="px-1 text-gray-400">...</span>
      {:else if pageToken === currentPage}
        <span class="px-3 py-1 border rounded bg-indigo-600 text-white font-medium" aria-current="page">{pageToken}</span>
      {:else}
        <button
          type="button"
          class="px-3 py-1 rounded border text-sm hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none"
          onclick={() => onPageChange(pageToken)}
          disabled={disabled}
        >
          {pageToken}
        </button>
      {/if}
    {/each}

    {#if currentPage < totalPages}
      <button
        type="button"
        class="px-3 py-1 rounded border text-sm hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none"
        onclick={() => onPageChange(currentPage + 1)}
        disabled={disabled}
      >
        Next ›
      </button>
    {/if}

    {#if showFirstLast}
      <button
        type="button"
        class="px-3 py-1 rounded border text-sm hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none"
        onclick={() => onPageChange(totalPages)}
        disabled={disabled || currentPage >= totalPages}
      >
        Last &gt;&gt;
      </button>
    {/if}
  </nav>
{/if}
