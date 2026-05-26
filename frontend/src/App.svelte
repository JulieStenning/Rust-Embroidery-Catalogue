<script>
  import { invoke } from "@tauri-apps/api/core";
  import DisclaimerView from "./lib/DisclaimerView.svelte";
  import MainView from "./lib/MainView.svelte";

  /** Whether the disclaimer check has completed */
  let loading = $state(true);
  /** Whether the disclaimer has been accepted */
  let disclaimerAccepted = $state(false);
  /** Error message if the check fails */
  let checkError = $state("");

  /** Called once on mount to determine which view to show */
  async function checkDisclaimer() {
    try {
      disclaimerAccepted = await invoke("check_disclaimer");
    } catch (e) {
      checkError = `Could not verify disclaimer status: ${e}`;
      console.error("check_disclaimer failed:", e);
    } finally {
      loading = false;
    }
  }

  /** Called by DisclaimerView once the user has accepted */
  function onDisclaimerAccepted() {
    disclaimerAccepted = true;
  }

  // Run the check when the component first mounts
  $effect(() => {
    checkDisclaimer();
  });
</script>

{#if loading}
  <!-- Splash / loading state -->
  <div class="flex items-center justify-center min-h-screen">
    <div class="text-center space-y-3">
      <p class="text-2xl">🧵</p>
      <p class="text-gray-500 text-sm">Loading Embroidery Catalogue…</p>
    </div>
  </div>

{:else if checkError}
  <!-- Error state -->
  <div class="flex items-center justify-center min-h-screen">
    <div class="max-w-md text-center space-y-4 px-4">
      <p class="text-red-600 font-semibold">Startup Error</p>
      <p class="text-sm text-gray-600">{checkError}</p>
      <p class="text-xs text-gray-400">
        Try restarting the application. If the problem persists, check that the
        database directory is accessible.
      </p>
    </div>
  </div>

{:else if !disclaimerAccepted}
  <!-- Disclaimer must be accepted before the main app loads -->
  <DisclaimerView {onDisclaimerAccepted} />

{:else}
  <!-- Main application -->
  <MainView />
{/if}
