<script>
  let { detailDesignId, onDesignDeleted } = $props();
</script>

<button data-testid="design-deleted" onclick={() => onDesignDeleted?.()}> Delete Design </button>
<div data-testid="design-detail-view" data-design-id={detailDesignId ?? ""}>Design Detail View</div>
