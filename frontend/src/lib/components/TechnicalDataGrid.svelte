<script>
  /** @type {{ items: Array<{label: string, value: string | number | null | undefined}> }} */
  let { items = [] } = $props();
</script>

<div class="grid grid-cols-2 sm:grid-cols-4 gap-x-3 gap-y-2">
  {#each items as item}
    <div class="flex flex-col bg-gray-50 rounded border border-gray-200 px-2.5 py-1.5">
      <span class="text-[10px] font-semibold text-gray-400 uppercase tracking-wide">{item.label}</span>
      <span class="text-sm font-medium text-gray-800">{item.value ?? "?"}</span>
    </div>
  {/each}
</div>