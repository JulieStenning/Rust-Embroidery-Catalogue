<script>
  import { invoke } from "@tauri-apps/api/core";

  /** Callback prop — called when the user has successfully accepted the disclaimer */
  let { onDisclaimerAccepted } = $props();

  /** The HTML content of the disclaimer, loaded from the Rust backend */
  let disclaimerHtml = $state("");
  /** Whether the user has ticked the acceptance checkbox */
  let checked = $state(false);
  /** Whether an accept request is in flight */
  let accepting = $state(false);
  /** Whether the disclaimer text is still loading */
  let loadingText = $state(true);
  /** Error message shown to the user */
  let error = $state("");
  /** Whether the user has scrolled to the bottom of the disclaimer text */
  let hasScrolledToBottom = $state(false);

  /**
   * Reference to the scrollable disclaimer container, bound via `bind:this`.
   * @type {HTMLElement | null}
   */
  let disclaimerContainer = $state(null);

  /**
   * Tolerance (in px) used when checking whether the user has reached the
   * bottom of the scrollable area. Absorbs DPI scaling, sub-pixel rendering,
   * and minor zooming inconsistencies across operating systems.
   */
  const SCROLL_TOLERANCE_PX = 8;

  /** Load the disclaimer text via Tauri command on mount */
  async function loadDisclaimer() {
    try {
      disclaimerHtml = await invoke("get_disclaimer_text");
    } catch (e) {
      error = "Failed to load disclaimer text. Please restart the application.";
      console.error("get_disclaimer_text failed:", e);
    } finally {
      loadingText = false;
    }
  }

  /** Check whether the user has reached the bottom of the scrollable disclaimer container. */
  function handleScroll() {
    const el = disclaimerContainer;
    if (!el) return;

    if (el.scrollTop + el.clientHeight >= el.scrollHeight - SCROLL_TOLERANCE_PX) {
      hasScrolledToBottom = true;
    }
  }

  /** Handle the Accept button click */
  async function accept() {
    if (!checked || !hasScrolledToBottom || accepting) return;
    accepting = true;
    error = "";
    try {
      await invoke("accept_disclaimer");
      onDisclaimerAccepted();
    } catch (e) {
      error = `Failed to save acceptance: ${e}. Please try again.`;
      console.error("accept_disclaimer failed:", e);
    } finally {
      accepting = false;
    }
  }

  // Load disclaimer text when the component first mounts
  $effect(() => {
    loadDisclaimer();
  });

  // Once the disclaimer text has finished loading and the container is bound,
  // mark acceptance-ready automatically when no scrolling is needed (short
  // content or a large viewport). Also re-run the bottom-threshold check in
  // case the user is already at the bottom after a re-render.
  $effect(() => {
    if (!loadingText && disclaimerContainer) {
      const el = disclaimerContainer;
      if (el.scrollHeight <= el.clientHeight) {
        hasScrolledToBottom = true;
      } else if (el.scrollTop + el.clientHeight >= el.scrollHeight - SCROLL_TOLERANCE_PX) {
        hasScrolledToBottom = true;
      }
    }
  });
</script>

<div class="max-w-4xl mx-auto py-6 px-4 space-y-4">

  <!-- Banner -->
  <div class="bg-amber-50 border border-amber-300 text-amber-900 rounded-lg px-4 py-3 text-sm">
    Before using the app, please review and accept this disclaimer.
    You will only be asked once for this installation.
  </div>

  <!-- Main card -->
  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h1 class="ui-page-title">Disclaimer</h1>

    <p class="text-sm text-gray-600">
      Acceptance is remembered locally after you confirm.
    </p>

    <!-- Disclaimer content -->
    <div
      bind:this={disclaimerContainer}
      onscroll={handleScroll}
      class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 space-y-4 max-h-96 overflow-y-auto"
    >
      {#if loadingText}
        <p class="text-gray-400 italic">Loading disclaimer…</p>
      {:else}
        <!-- Disclaimer HTML is trusted content embedded in the binary at compile time -->
        {@html disclaimerHtml}
      {/if}
    </div>

    <!-- Error message -->
    {#if error}
      <div class="bg-red-50 border border-red-300 text-red-700 rounded px-3 py-2 text-sm">
        {error}
      </div>
    {/if}

    <!-- Checkbox -->
    <label class="flex items-start gap-3 text-sm text-gray-700 cursor-pointer select-none">
      <input
        type="checkbox"
        bind:checked
        disabled={!hasScrolledToBottom || loadingText}
        class="mt-1 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500 disabled:cursor-not-allowed"
      />
      <span>I have read and accept the disclaimer above.</span>
    </label>

    <!-- Helper text — shown until the user has scrolled to the bottom -->
    {#if !hasScrolledToBottom && !loadingText}
      <p class="text-xs text-gray-400" data-testid="scroll-hint">
        Please scroll to the end of the disclaimer to enable acceptance.
      </p>
    {/if}

    <!-- Accept button -->
    <div class="flex gap-3">
      <button
        onclick={accept}
        disabled={!hasScrolledToBottom || !checked || accepting || loadingText}
        class="bg-indigo-600 text-white px-5 py-2 rounded text-sm font-medium
               hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500
               disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {#if accepting}
          Saving…
        {:else}
          Accept and continue
        {/if}
      </button>
    </div>
  </div>
</div>