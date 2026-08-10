<script>
  import { onMount } from "svelte";
  import AdminDesignersView from "./views/AdminDesignersView.svelte";
  import AdminSourcesView from "./views/AdminSourcesView.svelte";
  import {
    browseDataRootFolder,
    completeInitialSetup,
    getAppStatus,
    getConfiguredDataRoot,
    setConfiguredDataRoot,
  } from "./api/commandAdapter";

  /** Callback prop — called when the user has finished or skipped setup */
  let { onInitialSetupCompleted } = $props();

  /** 0 = Designers, 1 = Sources, 2 = Data Location (Installed only) */
  let step = $state(0);
  let finishing = $state(false);
  let error = $state("");

  /** Execution mode from Rust: "dev" | "installed" ("" if unknown) */
  let mode = $state("");
  /** Current data-root text input value */
  let dataRootInput = $state("");
  /** Has the user already got a persisted data root? */
  let hasConfiguredDataRoot = $state(false);
  /** A previously-configured data root is no longer reachable (e.g. drive changed). */
  let dataRootMissing = $state(false);
  /** Whether the Installed "Data Location" step is shown */
  let needsDataStep = $derived(mode === "installed");
  let isLastStep = $derived(step === (needsDataStep ? 2 : 1));

  const totalSteps = $derived(needsDataStep ? 3 : 2);

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
    if (step === 2) {
      return { ...dataCopy, stepLabel: "Step 3 of 3 — Data Location" };
    }
    const stepNumber = step === 0 ? 1 : 2;
    const base = step === 0 ? designersCopy : sourcesCopy;
    return {
      ...base,
      stepLabel: `Step ${stepNumber} of ${totalSteps} — ${step === 0 ? "Designers" : "Sources"}`,
    };
  });

  /** Figure out the mode and pre-fill the data root on mount.
   *  If the configured root is missing (e.g. a drive letter changed), jump
   *  straight to the data-location step so the user can reselect it. */
  onMount(async () => {
    const statusRes = await getAppStatus();
    if (statusRes.status) {
      mode = statusRes.status.execution_mode;
      dataRootMissing = Boolean(statusRes.status.data_root_missing);
    }

    if (mode === "installed") {
      const rootRes = await getConfiguredDataRoot();
      if (rootRes.path) {
        hasConfiguredDataRoot = true;
        dataRootInput = rootRes.path;
      }
      // A previously-configured location is no longer reachable: go straight
      // to the Data Location step so the user can pick a new one.
      if (dataRootMissing) {
        step = 2;
      }
    }
  });

  /** Advance to the next step, or finish setup on the last step. */
  async function handleContinue() {
    if (isLastStep) {
      await finishSetup();
    } else {
      step = step + 1;
    }
  }

  /** Open a native folder picker for the data location. */
  async function handleBrowse() {
    const res = await browseDataRootFolder(dataRootInput);
    if (res.path) {
      dataRootInput = res.path;
    }
  }

  /** Persist the data root (Installed mode) then mark setup complete. */
  async function finishSetup() {
    if (finishing) return;
    finishing = true;
    error = "";
    try {
      // Persist the chosen data root first (only when it's configurable).
      if (needsDataStep) {
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
      }
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
      Adding your frequent Designers and Sources now makes the Bulk Import tool faster and
      easier to use. {needsDataStep ? "You can also choose where your data lives." : ""}
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
    {#if step === 0}
      <AdminDesignersView embedded={true} />
    {:else if step === 1}
      <AdminSourcesView embedded={true} />
    {:else if step === 2}
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
            You already have a configured data location; you can keep it or change it.
          </p>
        {/if}
      </div>
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