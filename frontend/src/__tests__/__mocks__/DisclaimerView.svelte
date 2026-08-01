<script>
  // Stub DisclaimerView used by App.svelte tests.
  // Exposes the onDisclaimerAccepted prop as a clickable button so tests can
  // simulate the user accepting the disclaimer.
  let { onDisclaimerAccepted } = $props();
</script>

<div data-testid="disclaimer-view">
  <button onclick={onDisclaimerAccepted} data-testid="accept-disclaimer">
    Accept and continue
  </button>
</div>