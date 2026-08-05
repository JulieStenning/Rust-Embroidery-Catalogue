<script>
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import DisclaimerView from "./lib/DisclaimerView.svelte";
  import InitialSetupView from "./lib/InitialSetupView.svelte";
  import MainView from "./lib/MainView.svelte";
  import ToastContainer from "./lib/components/ToastContainer.svelte";
  import { initDbMaintenanceEvents } from "./lib/services/dbMaintenanceEvents";
  import { checkInitialSetup } from "./lib/api/commandAdapter";

  /** Cleanup function returned by initDbMaintenanceEvents(), if subscribed. */
  let stopDbMaintenanceEvents = $state(() => {});

  /** Whether the disclaimer check has completed */
  let loading = $state(true);
  /** Whether the disclaimer has been accepted */
  let disclaimerAccepted = $state(false);
  /** Whether the initial setup wizard has been completed or skipped */
  let initialSetupCompleted = $state(false);
  /** Error message if the check fails */
  let checkError = $state("");

  function hasTauriInvoke() {
    if (typeof window === "undefined") {
      return false;
    }
    return typeof (/** @type {any} */ (window)).__TAURI_INTERNALS__?.invoke === "function";
  }

  /** Called once on mount to determine which view to show */
  async function checkDisclaimer() {
    // In plain browser dev mode there is no Tauri bridge. Skip disclaimer gate
    // so route-level frontend smoke tests can run.
    if (!hasTauriInvoke()) {
      disclaimerAccepted = true;
      initialSetupCompleted = true;
      loading = false;
      return;
    }

    try {
      disclaimerAccepted = await invoke("check_disclaimer");
      initialSetupCompleted = await checkInitialSetup();
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

  /** Called by InitialSetupView once the user has finished or skipped setup.
   *  Route to the Bulk Import page first, since there are no designs yet. */
  function onInitialSetupCompleted() {
    if (typeof window !== "undefined") {
      window.location.hash = "#/import";
    }
    initialSetupCompleted = true;
  }

  // Subscribe to database maintenance lifecycle events as early as possible
  // (so the completion toast is shown even if ToastContainer mounts later).
  onMount(() => {
    initDbMaintenanceEvents().then((stop) => {
      stopDbMaintenanceEvents = stop;
    });
  });

  onDestroy(() => {
    stopDbMaintenanceEvents();
  });

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

{:else if !initialSetupCompleted}
  <!-- Initial setup wizard (designers & sources) -->
  <InitialSetupView {onInitialSetupCompleted} />

{:else}
  <!-- Main application -->
  <MainView />
  <ToastContainer />
{/if}
