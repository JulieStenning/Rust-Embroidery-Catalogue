<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onDestroy, onMount } from "svelte";
  import DatabaseRecoveryView from "./lib/DatabaseRecoveryView.svelte";
  import InitialSetupView from "./lib/InitialSetupView.svelte";
  import LicenceActivationView from "./lib/LicenceActivationView.svelte";
  import MainView from "./lib/MainView.svelte";
  import ToastContainer from "./lib/components/ToastContainer.svelte";
  import { initDbMaintenanceEvents } from "./lib/services/dbMaintenanceEvents";
  import { checkInitialSetup, getDatabaseStatus } from "./lib/api/commandAdapter";
  import { getLicenceStatus } from "./lib/api/licenceAdapter";
  import { initTheme } from "./lib/stores/themeStore";

  /** Cleanup function returned by initDbMaintenanceEvents(), if subscribed. */
  let stopDbMaintenanceEvents = $state(() => {});
  /** Cleanup function returned by initTheme(). */
  let stopTheme = () => {};

  /** Whether the startup check has completed */
  let loading = $state(true);
  /** Whether database schema migrations or index optimizations are running */
  let migrating = $state(false);
  /** Informative status message shown during migration */
  let migrationMessage = $state("");
  /** Whether a valid licence is active */
  let licenceValid = $state(false);
  /** Whether the initial setup wizard has been completed or skipped */
  let initialSetupCompleted = $state(false);
  /** True when the configured database is missing and the recovery view must block the app. */
  let databaseMissing = $state(false);
  /** Error message if the check fails */
  let checkError = $state("");

  function hasTauriInvoke() {
    if (typeof window === "undefined") {
      return false;
    }
    return typeof (/** @type {any} */ (window).__TAURI_INTERNALS__?.invoke) === "function";
  }

  /** Called once on mount to determine which view to show */
  async function checkStartup() {
    // In plain browser dev mode there is no Tauri bridge. Skip the setup gate
    // so route-level frontend smoke tests can run.
    if (!hasTauriInvoke()) {
      licenceValid = true;
      initialSetupCompleted = true;
      loading = false;
      return;
    }

    try {
      // Check the database recovery status first. If the configured database
      // is currently migrating, keep the splash screen with optimization text.
      // If it is missing or corrupt, block the main UI until the user recovers via
      // backup, re-points the location, or explicitly creates a new catalogue.
      const dbStatus = await getDatabaseStatus();
      if (dbStatus.status?.status === "migrating") {
        migrating = true;
        migrationMessage =
          "Updating database schema & optimizing indexes… Please do not close the app.";
        loading = true;
        return;
      }

      if (
        dbStatus.status &&
        (dbStatus.status.status === "missing" || dbStatus.status.status === "corrupted")
      ) {
        databaseMissing = true;
        loading = false;
        return;
      }

      let licence = null;
      if (typeof getLicenceStatus === "function") {
        licence = await getLicenceStatus();
      }
      licenceValid = licence ? Boolean(licence.is_valid) : true;
      if (!licenceValid) {
        loading = false;
        return;
      }

      initialSetupCompleted = await checkInitialSetup();
    } catch (e) {
      const errorStr = String(e);
      if (
        errorStr.includes("no such table") ||
        errorStr.includes("database") ||
        errorStr.includes("corrupt")
      ) {
        databaseMissing = true;
      } else {
        checkError = `Could not verify setup status: ${e}`;
      }
      console.error("check_initial_setup failed:", e);
    } finally {
      if (!migrating) {
        loading = false;
      }
    }
  }

  /** Called by LicenceActivationView once a valid licence key is activated */
  async function onLicenceActivated() {
    licenceValid = true;
    loading = true;
    try {
      initialSetupCompleted = await checkInitialSetup();
    } catch (e) {
      console.error("check_initial_setup failed after activation:", e);
    } finally {
      loading = false;
    }
  }

  /** Called by InitialSetupView once the user has finished or skipped setup.
   *  Route to the Bulk Import page first, since there are no designs yet. */
  function onInitialSetupCompleted() {
    if (typeof window !== "undefined") {
      window.location.hash = "#/import";
    }
    initialSetupCompleted = true;
  }

  /** Cleanup function for database migration status listener. */
  let stopMigrationEvents = () => {};

  // Subscribe to database maintenance lifecycle events as early as possible
  // (so the completion toast is shown even if ToastContainer mounts later).
  onMount(() => {
    stopTheme = initTheme();
    initDbMaintenanceEvents().then((stop) => {
      stopDbMaintenanceEvents = stop;
    });

    if (hasTauriInvoke()) {
      import("@tauri-apps/api/event").then(({ listen }) => {
        listen("database-migration-status", (event) => {
          const payload = /** @type {any} */ (event.payload);
          if (payload?.status === "migrating") {
            migrating = true;
            migrationMessage =
              payload.message ||
              "Updating database schema & optimizing indexes… Please do not close the app.";
            loading = true;
          } else if (payload?.status === "connected") {
            migrating = false;
            migrationMessage = "";
            checkStartup();
          } else if (payload?.status === "corrupted") {
            migrating = false;
            databaseMissing = true;
            loading = false;
            if (payload.message) {
              checkError = payload.message;
            }
          }
        }).then((unlisten) => {
          stopMigrationEvents = unlisten;
        });
      });
    }
  });

  onDestroy(() => {
    stopDbMaintenanceEvents();
    stopMigrationEvents();
    stopTheme();
  });

  // Run the check when the component first mounts
  $effect(() => {
    checkStartup();
  });
</script>

{#if loading}
  <!-- Splash / loading state -->
  <div class="flex items-center justify-center min-h-screen">
    <div class="text-center space-y-4 max-w-md px-6">
      <p class="text-4xl animate-bounce">🧵</p>
      <div class="space-y-1">
        <p class="text-base font-medium text-[var(--text-primary)]">
          {migrating ? "Preparing Catalogue Database" : "Loading Embroidery Catalogue…"}
        </p>
        <p class="text-sm text-[var(--text-muted)]">
          {migrating ? migrationMessage : "Please wait…"}
        </p>
      </div>
      {#if migrating}
        <div class="flex items-center justify-center space-x-2 pt-2">
          <svg
            class="animate-spin h-5 w-5 text-[var(--brand-primary)]"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
            ></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
          <span class="text-xs text-[var(--text-secondary)]"
            >Optimizing indexes and checking tables…</span
          >
        </div>
      {/if}
    </div>
  </div>
{:else if checkError}
  <!-- Error state -->
  <div class="flex items-center justify-center min-h-screen">
    <div class="max-w-md text-center space-y-4 px-4">
      <p class="text-[var(--notice-error-text)] font-semibold">Startup Error</p>
      <p class="text-sm text-[var(--text-secondary)]">{checkError}</p>
      <p class="text-xs text-[var(--text-muted)]">
        Try restarting the application. If the problem persists, check that the database directory
        is accessible.
      </p>
    </div>
  </div>
{:else if databaseMissing}
  <!-- Database recovery view: the configured database is missing (e.g. drive
       letter changed). This blocks the main UI until the user re-points the
       location or explicitly creates a new catalogue. -->
  <DatabaseRecoveryView />
{:else if !licenceValid}
  <!-- Licence activation view: requires valid email + key before proceeding -->
  <LicenceActivationView {onLicenceActivated} />
{:else if !initialSetupCompleted}
  <!-- Initial setup wizard (data location, designers & sources) -->
  <InitialSetupView {onInitialSetupCompleted} />
{:else}
  <!-- Main application -->
  <MainView />
  <ToastContainer />
{/if}
