<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { activateLicence } from "./api/commandAdapter";

  /** @type {{ onLicenceActivated: (status: import("./types/licence").LicenceStatus) => void }} */
  let { onLicenceActivated } = $props();

  let email = $state("");
  let licenceKey = $state("");
  let busy = $state(false);
  let errorMessage = $state("");

  async function handleActivate() {
    if (busy) return;
    const trimmedEmail = email.trim();
    const trimmedKey = licenceKey.trim();

    if (!trimmedEmail) {
      errorMessage = "Please enter your registered email address.";
      return;
    }
    if (!trimmedKey) {
      errorMessage = "Please enter your licence key.";
      return;
    }

    busy = true;
    errorMessage = "";

    try {
      const status = await activateLicence(trimmedEmail, trimmedKey);
      if (status.is_valid) {
        onLicenceActivated(status);
      } else {
        errorMessage =
          status.error_message || "Licence verification failed. Please check your details.";
      }
    } catch (err) {
      errorMessage = String(err);
    } finally {
      busy = false;
    }
  }

  /** @param {KeyboardEvent} e */
  function handleKeyDown(e) {
    if (e.key === "Enter" && !busy) {
      handleActivate();
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
  <div
    class="w-full max-w-lg bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-8 space-y-6"
  >
    <!-- Header -->
    <div class="text-center space-y-2">
      <div class="text-4xl">🧵</div>
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Embroidery Catalogue</h1>
      <p class="text-sm text-gray-600 dark:text-gray-300">
        Please enter your registered email address and licence key to activate access.
      </p>
    </div>

    <!-- Error alert -->
    {#if errorMessage}
      <div
        class="bg-red-50 dark:bg-red-900/30 border border-red-300 dark:border-red-700 text-red-700 dark:text-red-300 rounded-lg p-3 text-sm flex items-start gap-2"
        role="alert"
        data-testid="licence-error-alert"
      >
        <span class="font-bold">⚠️</span>
        <span class="flex-1">{errorMessage}</span>
      </div>
    {/if}

    <!-- Form -->
    <div class="space-y-4">
      <div>
        <label
          for="licence-email-input"
          class="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1"
        >
          Registered Email Address
        </label>
        <input
          id="licence-email-input"
          type="email"
          data-testid="licence-email-input"
          bind:value={email}
          onkeydown={handleKeyDown}
          disabled={busy}
          placeholder="e.g. tester@example.com"
          class="w-full px-3 py-2 text-sm border rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white border-gray-300 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
        />
      </div>

      <div>
        <label
          for="licence-key-input"
          class="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1"
        >
          Licence Key
        </label>
        <textarea
          id="licence-key-input"
          data-testid="licence-key-input"
          bind:value={licenceKey}
          onkeydown={handleKeyDown}
          disabled={busy}
          rows="3"
          placeholder="e.g. EMB1.eyJlbWFpbCI6..."
          class="w-full px-3 py-2 text-xs font-mono border rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white border-gray-300 dark:border-gray-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 resize-none"
        ></textarea>
      </div>

      <button
        type="button"
        data-testid="activate-licence-button"
        onclick={handleActivate}
        disabled={busy}
        class="w-full py-2.5 px-4 rounded-lg font-medium text-sm text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
      >
        {#if busy}
          <span class="animate-spin">⏳</span>
          <span>Verifying Licence…</span>
        {:else}
          <span>Activate Application</span>
        {/if}
      </button>
    </div>

    <!-- Footer helper -->
    <div class="border-t border-gray-100 dark:border-gray-700 pt-4 text-center">
      <p class="text-xs text-gray-500 dark:text-gray-400">
        Beta tester? Enter the email address and preview licence key provided in your invitation.
      </p>
    </div>
  </div>
</div>
