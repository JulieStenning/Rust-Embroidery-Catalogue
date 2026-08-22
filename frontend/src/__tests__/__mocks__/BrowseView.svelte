<script>
  let { browseNeedsRefresh } = $props();
</script>

<div data-testid="browse-view">
  Browse View
  {#if browseNeedsRefresh}
    <p data-testid="browse-needs-refresh">refresh-requested</p>
  {/if}
</div>
