<script>
  import { firstImportBannerVisible, dismissFirstImportBanner } from "../stores/firstImportStore";

  /** @param {string} target */
  function navigateToRoute(target) {
    if (typeof window !== "undefined") {
      window.location.hash = target;
    }
  }
</script>

{#if $firstImportBannerVisible}
  <div
    class="bg-indigo-50 border border-indigo-200 text-indigo-950 rounded-xl p-4 sm:p-5 shadow-sm space-y-3 mb-6 relative"
    role="region"
    aria-label="First import success notice"
    data-testid="first-import-success-banner"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="flex items-start gap-3">
        <span class="text-2xl select-none" aria-hidden="true">🎉</span>
        <div class="space-y-1">
          <h2 class="font-bold text-base text-indigo-950">First import complete!</h2>
          <p class="text-sm text-indigo-900 leading-relaxed">
            Your designs are tagged with fast, offline File &amp; Folder rules. If you would like
            AI-powered visual tagging, you can add a Google Gemini API key anytime in
            <a
              href="#/admin/system/settings"
              onclick={() => navigateToRoute("#/admin/system/settings")}
              class="font-semibold text-indigo-700 hover:text-indigo-900 underline decoration-indigo-300 underline-offset-2"
              >Settings</a
            >
            and run automated tagging from
            <a
              href="#/admin/batch-operations"
              onclick={() => navigateToRoute("#/admin/batch-operations")}
              class="font-semibold text-indigo-700 hover:text-indigo-900 underline decoration-indigo-300 underline-offset-2"
              >Batch Operations</a
            >.
          </p>
        </div>
      </div>
      <button
        type="button"
        onclick={dismissFirstImportBanner}
        class="text-indigo-600 hover:text-indigo-900 hover:bg-indigo-100 rounded-lg p-1.5 transition-colors -mr-1 -mt-1 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        aria-label="Dismiss first import notice"
        data-testid="first-import-banner-dismiss"
        title="Dismiss notice"
      >
        <span class="text-lg leading-none" aria-hidden="true">&times;</span>
      </button>
    </div>
  </div>
{/if}
