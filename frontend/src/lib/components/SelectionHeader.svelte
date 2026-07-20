<script lang="ts">
  interface Props {
    totalFilteredCount: number;
    selectedCountOnPage: number;
    totalCountOnPage: number;
    isAllSelectedOnPage: boolean;
    onToggleSelectAllPage: (checked: boolean) => void;
  }

  let {
    totalFilteredCount = 0,
    selectedCountOnPage = 0,
    totalCountOnPage = 0,
    isAllSelectedOnPage = false,
    onToggleSelectAllPage
  }: Props = $props();
</script>

<div class="selection-header flex flex-wrap items-center gap-4 py-3 px-4 bg-gray-50 border-b border-gray-200">
  <div class="flex flex-wrap items-center gap-3">
    <span class="text-sm font-medium text-gray-600">
      {totalFilteredCount === 1 ? "1 design found" : `${totalFilteredCount} designs found`}
    </span>
    <span class="text-gray-300 select-none" aria-hidden="true">•</span>
    <span class="text-sm font-medium text-gray-600">
      {selectedCountOnPage} of {totalCountOnPage} selected
    </span>
    <label class="flex items-center gap-2 cursor-pointer select-none text-sm font-medium text-gray-700">
      <input
        type="checkbox"
        class="ui-checkbox accent-indigo-650 rounded cursor-pointer"
        checked={isAllSelectedOnPage}
        onchange={(e: Event) => onToggleSelectAllPage((e.currentTarget as HTMLInputElement).checked)}
        disabled={totalCountOnPage === 0}
      />
      Select all on page
    </label>
  </div>
</div>
