<script>
  /** @type {{ items: Array<{label: string, value: string | number | null | undefined}> }} */
  let { items = [] } = $props();
</script>

<div class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-1">
  {#each items as item, i}
    <div class="flex items-baseline gap-2 {i < items.length - 1 && i % 2 === 0 ? 'sm:border-b sm:border-gray-100 sm:pb-1' : ''} {i < items.length - 1 && i % 2 !== 0 ? 'sm:border-b sm:border-gray-100 sm:pb-1' : ''} {i < items.length - 1 ? 'border-b border-gray-100 pb-1' : ''}">
      <span class="text-[10px] font-semibold text-gray-400 uppercase tracking-wide shrink-0 w-[6rem] sm:w-[7rem]">{item.label}</span>
      <span class="text-sm font-medium text-gray-800 truncate">{item.value ?? "?"}</span>
    </div>
  {/each}
</div>