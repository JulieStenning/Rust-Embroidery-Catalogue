<script>
  let {
    totalFilteredCount = 0,
    selectedCountOnPage = 0,
    totalCountOnPage = 0,
    isAllSelectedOnPage = false,
    onToggleSelectAllPage = () => {},
  } = $props();
</script>

<div
  data-testid="selection-header-mock"
  data-total-filtered={String(totalFilteredCount)}
  data-selected-count={String(selectedCountOnPage)}
  data-total-count={String(totalCountOnPage)}
  data-all-selected={String(isAllSelectedOnPage)}
>
  <span data-testid="selection-total-count">{totalFilteredCount} designs found</span>
  <label>
    <input
      type="checkbox"
      data-testid="select-all-page-checkbox"
      checked={isAllSelectedOnPage}
      onchange={(event) => onToggleSelectAllPage(event.currentTarget.checked)}
      disabled={totalCountOnPage === 0}
    />
    Select all on page
  </label>
</div>
