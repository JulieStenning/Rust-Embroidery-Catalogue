<script>
  import { onMount } from "svelte";
  import {
    getSettingsViewModel,
    saveSettings,
    browseSettingsDataRoot
  } from "../api/commandAdapter.js";

  let settingsLoading = $state(false);
  let settingsLoaded = $state(false);
  let settingsSaveState = $state("idle"); // "idle" | "saving" | "saved" | "error"
  let settingsSaveMessage = $state("");

  let settingsImagePreference = $state("2d");
  let settingsGoogleApiKey = $state("");
  let settingsApiKeyRevealed = $state(false);
  let settingsAiTier2Auto = $state(false);
  let settingsAiTier3Auto = $state(false);
  let settingsAiBatchSize = $state("");
  let settingsAiDelay = $state("");
  let settingsImportCommitBatchSize = $state("");
  let settingsCanConfigureDataRoot = $state(false);
  let settingsDataRoot = $state("");
  let settingsLogFolder = $state("");
  let settingsAppMode = $state("development");
  let settingsHelpUrl = $state("#/help");

  let settingsHasGoogleApiKey = $derived(settingsGoogleApiKey.trim().length > 0);

  function toggleSettingsApiKeyVisibility() {
    settingsApiKeyRevealed = !settingsApiKeyRevealed;
  }

  /** @param {Record<string, any>} model */
  function applySettingsModel(model = {}) {
    settingsImagePreference = model?.image_preference === "3d" ? "3d" : "2d";
    settingsGoogleApiKey = String(model?.google_api_key || "");
    settingsAiTier2Auto = Boolean(model?.ai_tier2_auto);
    settingsAiTier3Auto = Boolean(model?.ai_tier3_auto);
    settingsAiBatchSize = String(model?.ai_batch_size || "");
    settingsAiDelay = String(model?.ai_delay || "");
    settingsImportCommitBatchSize = String(model?.import_commit_batch_size || "");
    settingsCanConfigureDataRoot = Boolean(model?.can_configure_data_root);
    settingsDataRoot = String(model?.data_root || "");
    settingsLogFolder = String(model?.log_folder || "");
    settingsAppMode = String(model?.app_mode || "development");
    settingsHelpUrl = String(model?.ai_tagging_help_url || "#/help");
  }

  async function loadSettingsFromBackend(force = false) {
    if (settingsLoading) return;
    if (settingsLoaded && !force) return;

    settingsLoading = true;
    try {
      const result = await getSettingsViewModel();
      applySettingsModel(result.model || {});
      settingsLoaded = true;
    } catch (error) {
      settingsSaveState = "error";
      settingsSaveMessage = `Could not load settings: ${error}`;
    } finally {
      settingsLoading = false;
    }
  }

  /** @param {Event} event */
  async function saveSettingsFromBackend(event) {
    event.preventDefault();
    settingsSaveState = "saving";
    settingsSaveMessage = "";

    try {
      const result = await saveSettings({
        image_preference: settingsImagePreference,
        google_api_key: settingsGoogleApiKey,
        ai_tier2_auto: settingsAiTier2Auto,
        ai_tier3_auto: settingsAiTier3Auto,
        ai_batch_size: settingsAiBatchSize,
        ai_delay: settingsAiDelay,
        import_commit_batch_size: settingsImportCommitBatchSize,
        data_root: settingsDataRoot,
      });

      if (result.saved) {
        settingsSaveState = "saved";
        settingsSaveMessage = result.message || "Settings saved successfully.";
      } else {
        settingsSaveState = "error";
        settingsSaveMessage = result.message || "Settings could not be saved.";
      }
    } catch (error) {
      settingsSaveState = "error";
      settingsSaveMessage = `Could not save settings: ${error}`;
    }
  }

  async function browseDataRootFromBackend() {
    const result = await browseSettingsDataRoot(settingsDataRoot);
    if (result.path) {
      settingsDataRoot = result.path;
      settingsSaveState = "idle";
      settingsSaveMessage = "";
      return;
    }

    if (result.error) {
      settingsSaveState = "error";
      settingsSaveMessage = result.error;
    }
  }

  onMount(() => {
    loadSettingsFromBackend();
  });
</script>

<section class="settings-page space-y-6">
  <h1 class="ui-page-title settings-title mb-6">Application Settings</h1>

  {#if settingsSaveState === "saved"}
    <div class="settings-alert settings-alert-success bg-green-50 border border-green-300 text-green-800 rounded px-4 py-2 text-sm">
      {settingsSaveMessage || "Settings saved successfully."}
    </div>
  {:else if settingsSaveState === "error"}
    <div class="settings-alert settings-alert-error bg-red-50 border border-red-300 text-red-800 rounded px-4 py-2 text-sm">
      {settingsSaveMessage || "Settings could not be saved."}
    </div>
  {/if}

  <div class="settings-layout max-w-3xl space-y-6">
    {#if settingsLoading && !settingsLoaded}
      <div class="settings-alert settings-alert-info bg-blue-50 border border-blue-200 text-blue-800 rounded px-4 py-2 text-sm">
        Loading settings...
      </div>
    {/if}

    <form class="settings-card settings-form bg-white rounded shadow p-6 space-y-5" onsubmit={saveSettingsFromBackend}>
      <div>
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Image preview preference</h2>
        <p class="text-sm text-gray-600 mb-3">
          Choose whether new imports generate <strong>2D</strong> (fast flat preview) or
          <strong>3D</strong> (detailed stitch-simulated preview) images by default.
          You can override this per import session on the precheck page.
          3D rendering is slower but produces more realistic previews.
        </p>

        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="radio"
              name="settings-image-preference"
              value="2d"
              checked={settingsImagePreference === "2d"}
              class="text-indigo-600 focus:ring-indigo-500"
              onchange={() => {
                settingsImagePreference = "2d";
              }}
            />
            <strong>2D</strong> — Fast flat preview (default, recommended for bulk imports)
          </label>

          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="radio"
              name="settings-image-preference"
              value="3d"
              checked={settingsImagePreference === "3d"}
              class="text-indigo-600 focus:ring-indigo-500"
              onchange={() => {
                settingsImagePreference = "3d";
              }}
            />
            <strong>3D</strong> — Detailed stitch-simulated preview (slower, more realistic)
          </label>
        </div>

        <p class="mt-2 text-xs text-gray-500">
          Current setting: <strong>{settingsImagePreference === "3d" ? "3D" : "2D"}</strong>.
          This affects the import pipeline only. Existing designs keep their current image type.
        </p>
      </div>

      <div>
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Google Gemini API key</h2>
        <p class="text-sm text-gray-600">
          The Google API key is only required if you want your designs to be tagged automatically by Google AI.
          <a href={settingsHelpUrl} class="text-indigo-600 hover:underline">Press here for more information.</a>
        </p>
      </div>

      <div>
        <label for="settings-google-api-key" class="block text-sm font-semibold text-gray-700 mb-1">API key</label>
        <div class="flex items-center gap-2">
          <input
            id="settings-google-api-key"
            type={settingsApiKeyRevealed ? "text" : "password"}
            bind:value={settingsGoogleApiKey}
            placeholder="AIzaSy..."
            autocomplete="off"
            spellcheck="false"
            class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
          />
          <button
            type="button"
            class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
            aria-label="Show or hide API key"
            aria-pressed={settingsApiKeyRevealed}
            title={settingsApiKeyRevealed ? "Hide API key" : "Show API key"}
            onclick={toggleSettingsApiKeyVisibility}
          >
            <span aria-hidden="true" class="settings-eye-icon">{settingsApiKeyRevealed ? "🙈" : "👁"}</span>
          </button>
        </div>
        <p class="mt-2 text-xs text-gray-500">
          {#if settingsHasGoogleApiKey}
            A key is currently saved in <code>.env</code>. You can leave it as-is or replace it here.
          {:else}
            Leave this blank if you only want keyword-based tagging with no Google AI calls.
          {/if}
        </p>
      </div>

      <div class="border-t pt-4">
        <h2 class="text-sm font-semibold text-gray-700 mb-1">AI tagging during import</h2>
        <p class="text-sm text-gray-600 mb-3">
          Control whether Gemini AI tagging runs automatically when you import designs.
          Tier 1 (keyword matching) always runs and is free.
          Tiers 2 and 3 call the Google Gemini API and require an API key.
        </p>

        {#if settingsHasGoogleApiKey}
          <div class="bg-amber-50 border border-amber-300 rounded p-3 mb-3 text-sm text-amber-900">
            <strong>⚠ Cost notice:</strong> Gemini usage may incur charges on your Google account.
            Free-tier limits are approximately <strong>15 requests per minute</strong> and
            <strong>1,500 requests per day</strong>.
            A historical estimate from February 2026 found that Tier 3 on 4,000 images cost about
            <strong>$0.33 on the paid tier</strong>; actual pricing may have changed.
            Check the latest rates at
            <a href="https://ai.google.dev/pricing" class="underline" target="_blank" rel="noopener">ai.google.dev/pricing</a>.
          </div>
        {:else}
          <div class="bg-blue-50 border border-blue-200 rounded p-3 mb-3 text-sm text-blue-900">
            No API key is saved. AI tagging options below will have no effect until you add a key above.
          </div>
        {/if}

        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={settingsAiTier2Auto} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            Run <strong>Tier 2</strong> (Gemini text AI from filename) automatically during import
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input type="checkbox" bind:checked={settingsAiTier3Auto} class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
            Run <strong>Tier 3</strong> (Gemini vision AI from preview image) automatically during import
          </label>
        </div>
      </div>

      <div>
        <label for="settings-ai-batch-size" class="block text-sm font-semibold text-gray-700 mb-1">
          AI tagging batch size <span class="font-normal text-gray-500">(optional)</span>
        </label>
        <input
          id="settings-ai-batch-size"
          type="number"
          min="1"
          bind:value={settingsAiBatchSize}
          placeholder="e.g. 100 — leave blank to tag all"
          class="settings-input border rounded px-3 py-2 text-sm w-48"
        />
        <p class="mt-1 text-xs text-gray-500">
          Limit AI tagging to this many newly imported designs per import run.
          Leave blank to tag all newly imported designs.
          Useful for very large imports where you want to spread Gemini calls over several runs.
        </p>
      </div>

      <div>
        <label for="settings-ai-delay" class="block text-sm font-semibold text-gray-700 mb-1">
          Delay between Gemini calls (seconds) <span class="font-normal text-gray-500">(optional)</span>
        </label>
        <input
          id="settings-ai-delay"
          type="number"
          min="0"
          step="0.5"
          bind:value={settingsAiDelay}
          placeholder="e.g. 6.0 — leave blank for default (5.0 s)"
          class="settings-input border rounded px-3 py-2 text-sm w-56"
        />
        <p class="mt-1 text-xs text-gray-500">
          Seconds to wait between API calls. Increase this if you see <em>429 Too Many Requests</em> errors.
          Default is 5.0 seconds. Also applies to batch tagging actions on the
          <a href="#/admin/tagging-actions" class="text-indigo-600 hover:underline">Tagging Actions</a> page.
        </p>
      </div>

      <div>
        <label for="settings-import-commit-batch-size" class="block text-sm font-semibold text-gray-700 mb-1">
          Import database commit batch size <span class="font-normal text-gray-500">(optional)</span>
        </label>
        <input
          id="settings-import-commit-batch-size"
          type="number"
          min="1"
          bind:value={settingsImportCommitBatchSize}
          placeholder="e.g. 10 — leave blank for default"
          class="settings-input border rounded px-3 py-2 text-sm w-56"
        />
        <p class="mt-1 text-xs text-gray-500">
          Controls how many designs are written or tag-updated before each database commit during import.
          Leave blank to use the default batch size of 10.
          Lower values reduce rollback size on failure; higher values reduce commit overhead.
        </p>
      </div>

      <div class="border-t pt-4 space-y-3">
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Catalogue storage</h2>
        <p class="text-sm text-gray-600">
          Large catalogue data lives under a single home folder.
          {#if settingsCanConfigureDataRoot}
            For desktop installs you can point this to a larger drive. Changes apply after restarting the app, and any missing managed files are copied into the new location automatically.
          {:else}
            In {settingsAppMode} mode this location follows the application folder automatically.
          {/if}
        </p>

        {#if settingsCanConfigureDataRoot}
          <div>
            <label for="settings-data-root" class="block text-sm font-semibold text-gray-700 mb-1">Catalogue data location</label>
            <div class="flex items-center gap-2">
              <input
                id="settings-data-root"
                type="text"
                bind:value={settingsDataRoot}
                placeholder="D:\\EmbroideryCatalogueData"
                spellcheck="false"
                class="settings-input flex-1 border rounded px-3 py-2 text-sm font-mono"
              />
              <button
                type="button"
                class="settings-secondary-button border rounded px-3 py-2 text-sm hover:bg-gray-50"
                onclick={browseDataRootFromBackend}
              >
                Browse…
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500">
              This home folder contains both the catalogue database and the managed design library.
            </p>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between gap-3">
        <p class="text-xs text-gray-500">These settings are stored in the catalogue database for this installation.</p>
        <button type="submit" class="settings-primary-button menu-button-primary" disabled={settingsSaveState === "saving"}>
          {settingsSaveState === "saving" ? "Saving..." : "Save settings"}
        </button>
      </div>
    </form>

    <div class="settings-card settings-meta bg-white rounded shadow p-6 space-y-5">
      <div>
        <h2 class="text-sm font-semibold text-gray-700 mb-1">Storage locations</h2>
        <p class="text-sm text-gray-600">
          The catalogue database and imported embroidery files live under the catalogue data location shown below.
          Logs are stored separately so they survive data moves.
        </p>
      </div>

      <div>
        <p class="block text-sm font-semibold text-gray-700 mb-1">Catalogue data location</p>
        <code class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all">{settingsDataRoot}</code>
      </div>

      <div>
        <p class="block text-sm font-semibold text-gray-700 mb-1">Log folder</p>
        <code class="settings-code block bg-gray-50 border rounded px-3 py-2 text-sm font-mono break-all">{settingsLogFolder}</code>
      </div>
    </div>
  </div>
</section>
