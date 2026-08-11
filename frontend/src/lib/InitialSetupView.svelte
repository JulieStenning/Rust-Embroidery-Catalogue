<script>
  import { onMount } from "svelte";
  import AdminDesignersView from "./views/AdminDesignersView.svelte";
  import AdminSourcesView from "./views/AdminSourcesView.svelte";
  import {
    browseDataRootFolder,
    completeInitialSetup,
    getAppStatus,
    getConfiguredDataRoot,
    restartApplication,
    setConfiguredDataRoot,
  } from "./api/commandAdapter";

  /** Callback prop — called when the user has finished or skipped setup */
  let { onInitialSetupCompleted } = $props();

  /** The currently visible step (0-based) */
  let step = $state(0);
  let finishing = $state(false);
  let error = $state("");
  /** Whether the restart confirmation dialog is showing */
  let showRestartConfirm = $state(false);
  /** Whether a restart is currently being launched */
  let restarting = $state(false);

  /** Execution mode from Rust: "dev" | "installed" ("" if unknown) */
  let mode = $state("");
  /** Current data-root text input value */
  let dataRootInput = $state("");
  /** Has the user already got a persisted data root? */
  let hasConfiguredDataRoot = $state(false);
  /** A previously-configured data root is no longer reachable (e.g. drive changed). */
  let dataRootMissing = $state(false);
  /** Whether the Installed "Data Location" step is shown at all. */
  let needsDataStep = $derived(mode === "installed");
  /**
   * Whether the Data Location step runs FIRST, before Designers/Sources.
   * True on first run (no configured root yet) or when the configured root is
   * unreachable — the user must set/repair the location before adding
   * designers and sources so the required restart happens before those steps.
   */
  let dataStepFirst = $state(false);

  /** Whether the visible flow includes a leading Data Location step. */
  const hasDataFirstFlow = $derived(needsDataStep && dataStepFirst);
  /** Index of the Data Location step in the visible flow (`-1` when hidden). */
  const dataStepIndex = $derived(hasDataFirstFlow ? 0 : -1);
  /** Index of the Designers step in the visible flow. */
  const designerStepIndex = $derived(hasDataFirstFlow ? 1 : 0);
  /** Index of the Sources step in the visible flow. */
  const sourceStepIndex = $derived(hasDataFirstFlow ? 2 : 1);
  /** Total number of visible steps. */
  const totalSteps = $derived(hasDataFirstFlow ? 3 : 2);

  /** Is the current step the Data Location step? */
  const isDataStep = $derived(needsDataStep && step === dataStepIndex);
  /** Is the current step the last visible step? */
  const isLastStep = $derived(step === (hasDataFirstFlow ? 2 : 1));

  const designersCopy = {
    question: "What are Designers?",
    answer:
      "Designers are the digitizers or creators who created the embroidery patterns. They allow you to filter, search, and organize your catalogue based on who created the pattern.",
    whyNowTitle: "Why do this now?",
    whyNow:
      "Setting up your frequent designers now enables the Bulk Import tool to display them in drop-down menus. This makes it effortless to assign a designer to an entire folder at once during import, rather than tagging designs individually later.",
    mandatory: "Not at all! This step is completely optional. You can skip this now and add or edit designers at any time in the settings.",
  };

  const sourcesCopy = {
    question: "What are Sources?",
    answer:
      "Sources describe where your embroidery designs came from — such as 'Purchased', 'Downloaded', or a specific website or shop name. They help you track the origin of every design in your catalogue.",
    whyNowTitle: "Why do this now?",
    whyNow:
      "Setting up your common sources now enables the Bulk Import tool to display them in drop-down menus. This lets you tag an entire folder's designs with the same source during import, rather than editing each design individually later.",
    mandatory: "Not at all! This step is completely optional. You can skip this now and add or edit sources at any time in the settings.",
  };

  const dataCopy = {
    question: "Where should your catalogue data live?",
    answer:
      "Your design files, database and thumbnails are stored in a data folder. By default this sits on your system drive, but if you have a large collection you can place it on another drive to keep your system drive free.",
    whyNowTitle: "Why choose now?",
    whyNow:
      "Choosing a location now ensures your data lives exactly where you want it before you start importing designs. You can change this later via Settings.",
    mandatory: "You can keep the suggested location, or choose your own folder.",
  };

  let activeCopy = $derived.by(() => {
    if (isDataStep) {
      return {
        ...dataCopy,
        stepLabel: `Step ${dataStepIndex + 1} of ${totalSteps} — Data Location`,
      };
    }
    if (step === designerStepIndex) {
      return {
        ...designersCopy,
        stepLabel: `Step ${designerStepIndex + 1} of ${totalSteps} — Designers`,
      };
    }
    return {
      ...sourcesCopy,
      stepLabel: `Step ${sourceStepIndex + 1} of ${totalSteps} — Sources`,
    };
  });

  /** Figure out the mode and decide the step order on mount.
   *
   *  - Installed + no configured root, or configured root unreachable:
   *    Data Location runs FIRST (data-first), so the required restart
   *    happens before designers/sources are added.
   *  - Installed + valid configured root: skip the Data step entirely and
   *    go straight to Designers → Sources.
   *  - Dev/Portable mode: never show the Data step. */
  onMount(async () => {
    const statusRes = await getAppStatus();
    if (statusRes.status) {
      mode = statusRes.status.execution_mode;
      dataRootMissing = Boolean(statusRes.status.data_root_missing);
    }

    if (needsDataStep) {
      const rootRes = await getConfiguredDataRoot();
      const configuredPath = rootRes.path ? String(rootRes.path) : null;
      if (configuredPath) {
        hasConfiguredDataRoot = true;
        dataRootInput = configuredPath;
      }
      const needsDataFirst = !configuredPath || dataRootMissing;
      dataStepFirst = needsDataFirst;
      step = 0; // Data Location when data-first, otherwise Designers.
    } else {
      dataStepFirst = false;
      step = 0; // Designers
    }
  });

  /** Open a native folder picker for the data location. */
  async function handleBrowse() {
    const res = await browseDataRootFolder(dataRootInput);
    if (res.path) {
      dataRootInput = res.path;
    }
  }

  /** Validate and persist the data location, then ask for a restart. */
  async function saveDataRoot() {
    if (finishing) return;
    finishing = true;
    error = "";
    try {
      const trimmed = dataRootInput.trim();
      if (!trimmed) {
        error = "Please enter a data location or choose a folder.";
        finishing = false;
        return;
      }
      const saved = await setConfiguredDataRoot(trimmed);
      if (!saved.persisted) {
        error = `Could not save the data location: ${saved.error || "unknown error"}.`;
        finishing = false;
        return;
      }
      // The data root is now persisted, but the running backend still points
      // at the old location. A restart is required before designers/sources
      // are added so they land in the correct database.
      showRestartConfirm = true;
    } catch (e) {
      error = `Failed to save the data location: ${e}. Please try again.`;
      console.error("initial setup data save failed:", e);
    } finally {
      finishing = false;
    }
  }

  /** Launch the application restart after the user confirms. */
  async function handleRestart() {
    if (restarting) return;
    restarting = true;
    error = "";
    const res = await restartApplication();
    if (!res.restarted) {
      error = `Could not restart the application: ${
        res.error || "unknown error"
      }. Please close and reopen it manually so your new data location takes effect.`;
      restarting = false;
      showRestartConfirm = false;
      return;
    }
    // On success the process is relaunching; the window will close shortly.
    // Nothing further to do here.
  }

  /** Advance through the visible steps, or finish on the last one. */
  async function handleContinue() {
    if (isDataStep) {
      await saveDataRoot();
      return;
    }
    if (isLastStep) {
      await finishSetup();
    } else {
      step = step + 1;
    }
  }

  /** Mark the wizard complete (only reached after the data step, if any). */
  async function finishSetup() {
    if (finishing) return;
    finishing = true;
    error = "";
    try {
      await completeInitialSetup();
      onInitialSetupCompleted();
    } catch (e) {
      error = `Failed to save setup status: ${e}. Please try again.`;
      console.error("initial setup failed:", e);
    } finally {
      finishing = false;
    }
  }
</script>

<div class="max-w-5xl mx-auto py-6 px-4 space-y-4">

  <!-- Welcome banner -->
  <div class="bg-indigo-50 border border-indigo-200 text-indigo-900 rounded-lg px-4 py-3 text-sm">
    <span class="font-semibold">Welcome to Embroidery Catalogue!</span>
    Before you import files, you can optionally add a few of your main Designers and Sources.
  </div>

  <!-- Main card -->
  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h1 class="ui-page-title text-2xl font-bold text-gray-800">Let's set up your catalogue</h1>
    <p class="text-sm text-gray-600">
      {#if hasDataFirstFlow}
        First, choose where your data lives. Then you can add your frequent
        Designers and Sources so the Bulk Import tool is faster and easier to use.
      {:else}
        Adding your frequent Designers and Sources now makes the Bulk Import tool faster and
        easier to use.
      {/if}
    </p>

    <!-- Step indicator -->
    <div class="flex items-center gap-2 text-sm text-gray-600">
      <span class="font-medium text-indigo-700">{activeCopy.stepLabel}</span>
    </div>

    <!-- Explanation box for the active step -->
    <div class="bg-gray-50 border rounded-lg p-4 space-y-3 text-sm text-gray-700">
      <div>
        <p class="font-semibold text-gray-800">{activeCopy.question}</p>
        <p class="mt-1">{activeCopy.answer}</p>
      </div>
      <div>
        <p class="font-semibold text-gray-800">{activeCopy.whyNowTitle}</p>
        <p class="mt-1">{activeCopy.whyNow}</p>
      </div>
      <div>
        <p class="font-semibold text-gray-800">Are they mandatory?</p>
        <p class="mt-1">{activeCopy.mandatory}</p>
      </div>
    </div>

    <!-- Error message -->
    {#if error}
      <div class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm">
        {error}
      </div>
    {/if}

    <!-- Embedded admin view or data-location input for the active step -->
    {#if isDataStep}
      <div class="space-y-3">
        {#if dataRootMissing}
          <div
            class="bg-amber-50 border border-amber-300 text-amber-800 rounded px-3 py-2 text-sm"
            data-testid="data-root-missing-notice"
          >
            Your previously configured data folder is no longer reachable. This can
            happen if a portable drive letter changed or the folder was moved.
            Please choose where your catalogue data should live now.
          </div>
        {/if}
        <label for="data-root-input" class="block text-sm font-medium text-gray-700">
          Data location
        </label>
        <div class="flex gap-2">
          <input
            id="data-root-input"
            type="text"
            bind:value={dataRootInput}
            class="flex-1 border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
            placeholder="e.g. D:\EmbroideryCatalogue"
            data-testid="data-root-input"
          />
          <button
            type="button"
            onclick={handleBrowse}
            class="bg-gray-100 text-gray-700 border border-gray-300 px-4 py-2 rounded text-sm font-medium
                   hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            data-testid="data-root-browse"
          >
            Browse…
          </button>
        </div>
        {#if hasConfiguredDataRoot}
          <p class="text-xs text-gray-500">
            Your previous data location was set; you can keep it or change it. If you
            change it, the app will restart to use the new location.
          </p>
        {/if}
      </div>
    {:else if step === designerStepIndex}
      <AdminDesignersView embedded={true} />
    {:else if step === sourceStepIndex}
      <AdminSourcesView embedded={true} />
    {/if}

    <!-- Bottom buttons -->
    <div class="flex items-center justify-end pt-2 border-t border-gray-200">
      <button
        type="button"
        onclick={handleContinue}
        disabled={finishing}
        class="bg-indigo-600 text-white px-5 py-2 rounded text-sm font-medium
               hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500
               disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        data-testid="initial-setup-continue"
      >
        {#if finishing}
          Saving…
        {:else}
          {isLastStep ? "Finish" : "Continue →"}
        {/if}
      </button>
    </div>
  </div>
</div>

<!-- Restart confirmation dialog -->
{#if showRestartConfirm}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Restart required"
    data-testid="restart-dialog"
  >
    <div class="bg-white rounded-xl shadow-lg max-w-md w-full p-6 space-y-4">
      <h2 class="text-lg font-bold text-gray-800">Restart required</h2>
      <p class="text-sm text-gray-600">
        Your new data location has been saved. Embroidery Catalogue needs to restart
        so it can begin using <span class="font-medium text-gray-800">{dataRootInput}</span>.
      </p>
      {#if error}
        <div class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm">
          {error}
        </div>
      {/if}
      <div class="flex items-center justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={handleRestart}
          disabled={restarting}
          class="bg-indigo-600 text-white px-5 py-2 rounded text-sm font-medium
                 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500
                 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          data-testid="restart-now"
        >
          {#if restarting}
            Restarting…
          {:else}
            Restart now
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}