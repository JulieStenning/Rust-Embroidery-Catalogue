<script>
  let {
    currentPage = 1,
    totalPages = 1,
    onPageChange = () => {},
    disabled = false,
    ariaLabel = "Pagination",
  } = $props();
</script>

<nav aria-label={ariaLabel} data-testid="pagination-mock" data-current-page={String(currentPage)} data-total-pages={String(totalPages)}>
  {#if totalPages > 1}
    <button type="button" onclick={() => onPageChange(Math.max(1, currentPage - 1))} disabled={disabled || currentPage <= 1}>Previous</button>
    <button type="button" onclick={() => onPageChange(Math.min(totalPages, currentPage + 1))} disabled={disabled || currentPage >= totalPages}>Next</button>
  {/if}
</nav>