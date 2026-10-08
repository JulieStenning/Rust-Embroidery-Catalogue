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

<div
  class="flex items-center justify-center min-h-screen bg-[var(--surface-canvas)] p-4"
  data-testid="licence-activation-view"
>
  <div class="route-card w-full max-w-lg rounded-xl shadow-lg p-8 space-y-6">
    <!-- Header -->
    <div class="text-center space-y-2">
      <div class="text-4xl">🧵</div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">Embroidery Catalogue</h1>
      <p class="text-sm text-[var(--text-secondary)]">
        Please enter your registered email address and licence key to activate access.
      </p>
    </div>

    <!-- Error alert -->
    {#if errorMessage}
      <div
        class="notice-error rounded-lg p-3 text-sm flex items-start gap-2"
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
          class="block text-xs font-semibold text-[var(--text-secondary)] mb-1"
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
          class="input-field w-full px-3 py-2 text-sm rounded-lg"
        />
      </div>

      <div>
        <label
          for="licence-key-input"
          class="block text-xs font-semibold text-[var(--text-secondary)] mb-1"
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
          class="input-field w-full px-3 py-2 text-xs font-mono rounded-lg resize-none"></textarea>
      </div>

      <button
        type="button"
        data-testid="activate-licence-button"
        onclick={handleActivate}
        disabled={busy}
        class="btn-primary w-full py-2.5 px-4 rounded-lg font-medium text-sm flex items-center justify-center gap-2"
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
    <div class="border-t border-[var(--border-default)] pt-4 text-center">
      <p class="text-xs text-[var(--text-muted)]">
        Beta tester? Enter the email address and preview licence key provided in your invitation.
      </p>
    </div>
  </div>
</div>
