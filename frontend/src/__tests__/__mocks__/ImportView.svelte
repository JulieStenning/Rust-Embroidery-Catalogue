<script>
  let { onImportCompleted } = $props();
</script>

<div data-testid="import-view">
  <button
    data-testid="import-completed"
    onclick={() => onImportCompleted(2)}
  >
    Complete Import (has items)
  </button>
  <button
    data-testid="import-zero"
    onclick={() => onImportCompleted(0)}
  >
    Complete Import (no items)
  </button>
  Import View
</div>