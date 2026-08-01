<script>
  import { untrack } from "svelte";
  import ImportView from "../ImportView.svelte";

  /**
   * Test harness that owns the wizard route in local state.  ImportView calls
   * `navigateTo()` whenever the internal wizard advances or rewinds; without
   * this wrapper the route prop would stay static and the multi-step flow
   * (step 1 -> 2 -> 3) could not be exercised in a single render.
   */
  let { initialRoute = "#/import", onImportCompleted = () => {}, onNavigate = () => {} } = $props();
  // The route prop is deliberately read only once; subsequent navigation is
  // driven entirely by ImportView's navigateTo() calls.
  let route = $state(untrack(() => initialRoute));

  /** @param {string} next */
  function navigateTo(next) {
    route = next;
    onNavigate(next);
  }
</script>

<ImportView currentRoute={route} {navigateTo} {onImportCompleted} />