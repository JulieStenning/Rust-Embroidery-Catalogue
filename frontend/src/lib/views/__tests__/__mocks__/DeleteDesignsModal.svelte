<script>
  import { deleteResultHolder } from "./deleteResultHolder.js";

  let {
    designIds = [],
    previewItems = [],
    open = false,
    onClose = () => {},
    onDeleted = () => {},
  } = $props();
</script>

<div data-testid="delete-designs-modal" data-open={String(open)} data-design-count={String(designIds.length)}>
  {#each previewItems as item (item.id)}
    <span data-testid="delete-preview-item">{item.filename}</span>
  {/each}
  {#if open}
    <button type="button" onclick={() => onDeleted(deleteResultHolder.value ?? { persisted: true, deleted_count: designIds.length, files_trashed: 0, errors: [] })}>Confirm delete</button>
    <button type="button" onclick={() => onClose()}>Cancel delete</button>
  {/if}
</div>
