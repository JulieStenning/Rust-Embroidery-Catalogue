<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount } from "svelte";
  import { getLicenceStatus, deactivateLicence } from "../api/licenceAdapter";
  import ConfirmDeactivateLicenceModal from "../components/ConfirmDeactivateLicenceModal.svelte";
  import { addToast } from "../stores/toastStore.js";
  import { busyState } from "../stores/busyStore.js";

  const APP_NAME = "Embroidery Catalogue";
  const APP_VERSION = "0.1.0";

  let licenceStatus = $state(/** @type {import("../types/licence").LicenceStatus | null} */ (null));
  let showDeactivateModal = $state(false);
  let isDeactivatingLicence = $state(false);

  let busyActive = $derived($busyState.active);

  async function loadLicence() {
    if (typeof getLicenceStatus !== "function") return;
    try {
      licenceStatus = await getLicenceStatus();
    } catch (error) {
      console.info("Could not load licence status.", error);
    }
  }

  function handleDeactivateLicence() {
    if (busyActive || isDeactivatingLicence) return;
    showDeactivateModal = true;
  }

  async function handleConfirmDeactivateLicence() {
    if (busyActive || isDeactivatingLicence) return;
    isDeactivatingLicence = true;
    try {
      const updated = await deactivateLicence();
      licenceStatus = updated;
      showDeactivateModal = false;
      addToast("Licence deactivated. Reloading…", "info");
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    } catch (err) {
      addToast(`Could not deactivate licence: ${err}`, "error");
    } finally {
      isDeactivatingLicence = false;
    }
  }

  onMount(() => {
    loadLicence();
  });

  /**
   * The verbatim MIT notice required by pyembroidery's licence. It is reproduced
   * in full here (and in the repo-root `NOTICE` file) because the MIT License
   * requires its copyright and permission notices to be retained in derived
   * works. Note the upstream notice's copyright line carries no name — that is
   * how it appears in pyembroidery's own LICENSE file, so it is kept verbatim.
   */
  const PYEMBROIDERY_MIT_NOTICE = `MIT License

Copyright (c) 2018

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;
</script>

<div class="max-w-4xl mx-auto space-y-6 font-sans">
  <div class="route-card rounded-xl shadow p-6 space-y-4">
    <div class="flex items-center gap-4">
      <span class="text-4xl" aria-hidden="true">🧵</span>
      <div>
        <h1 class="text-2xl font-bold text-[var(--text-primary)] font-sans">{APP_NAME}</h1>
        <p class="text-sm text-[var(--text-muted)]">Version v{APP_VERSION}</p>
      </div>
    </div>

    <p class="text-sm text-[var(--text-secondary)]">
      Embroidery Catalogue is a local, desktop catalogue for browsing, tagging, and managing a
      collection of digital embroidery designs. It reads a broad range of embroidery formats
      directly via built-in readers (<code>.jef</code>, <code>.pes</code>, <code>.hus</code>,
      <code>.vp3</code>, <code>.dst</code>, <code>.exp</code>) and stores all catalogue data in a
      local database. An Internet connection is required for advanced tagging.
    </p>

    <div
      class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg p-4 text-sm space-y-1"
    >
      <h2 class="ui-section-label font-semibold text-[var(--text-primary)]">Primary Copyright</h2>
      <p class="text-[var(--text-secondary)]">Copyright (C) 2026 Julie Stenning</p>
    </div>
  </div>

  <div class="route-card rounded-xl shadow p-6 space-y-4" data-testid="about-licence-section">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-bold text-[var(--text-primary)] font-sans flex items-center gap-2">
        <span>🔐</span>
        <span>Product Licence &amp; Activation</span>
      </h2>
      {#if licenceStatus?.is_valid}
        <span
          class="px-2.5 py-0.5 text-xs font-semibold rounded bg-[var(--notice-success-bg)] text-[var(--notice-success-text)] border border-[var(--notice-success-border)] uppercase tracking-wide"
        >
          {licenceStatus.tier ? licenceStatus.tier.toUpperCase() : "ACTIVE"}
        </span>
      {/if}
    </div>
    <p class="text-sm text-[var(--text-secondary)]">
      Manage your application licence key and registration status.
    </p>

    {#if licenceStatus?.is_valid}
      <div
        class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg p-4 space-y-3 text-sm"
      >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div>
            <span class="block text-xs text-[var(--text-muted)] font-medium">Registered Email</span>
            <span class="font-medium text-[var(--text-primary)]" data-testid="about-licence-email">
              {licenceStatus.email || "—"}
            </span>
          </div>
          <div>
            <span class="block text-xs text-[var(--text-muted)] font-medium">Validity</span>
            <span class="font-medium text-[var(--text-primary)]" data-testid="about-licence-expiry">
              {licenceStatus.expires_at_formatted
                ? `Valid until ${licenceStatus.expires_at_formatted}`
                : "Lifetime (No Expiry)"}
            </span>
          </div>
        </div>
        <div class="pt-1">
          <button
            type="button"
            data-testid="about-deactivate-licence-button"
            onclick={handleDeactivateLicence}
            disabled={busyActive}
            class="text-xs text-[var(--notice-error-text)] hover:underline font-medium"
          >
            Deactivate / Switch Licence Key
          </button>
        </div>
      </div>
    {:else}
      <p class="text-xs text-[var(--text-muted)]">No active licence detected.</p>
    {/if}
  </div>

  <div class="route-card rounded-xl shadow p-6 space-y-4">
    <h2 class="text-lg font-bold text-[var(--text-primary)] font-sans">
      Acknowledgements &amp; Code Porting Attributions
    </h2>
    <div class="space-y-4 text-sm">
      <div
        class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg p-4"
      >
        <h3 class="ui-section-label font-semibold text-[var(--text-primary)] mb-1">
          pyembroidery (MIT License)
        </h3>
        <p class="text-[var(--text-secondary)]">
          Binary parsing routines for <code>.pes</code>, <code>.jef</code>, <code>.vp3</code>,
          <code>.hus</code>, <code>.dst</code> and <code>.exp</code>, together with the PNG preview
          rendering logic derived from pyembroidery's PngWriter module, were ported/derived into
          native Rust modules from
          <a
            href="https://github.com/EmbroidePy/pyembroidery"
            target="_blank"
            rel="noopener noreferrer"
            class="text-[var(--text-brand)] hover:underline font-medium break-all"
            >https://github.com/EmbroidePy/pyembroidery</a
          >.
        </p>
        <p class="text-[var(--text-secondary)] mt-2">
          Copyright (c) 2018 Tatarize and the EmbroidePy pyembroidery contributors.
        </p>
        <p class="text-[var(--text-secondary)] mt-2">
          Our thanks go to Tatarize and the EmbroidePy pyembroidery contributors for publishing
          their work under a permissive licence, and for providing the reference implementations of
          the embroidery file formats that made this application possible.
        </p>
        <details class="mt-3" data-testid="pyembroidery-mit-notice">
          <summary
            class="cursor-pointer ui-section-label font-semibold text-[var(--text-brand)] hover:underline"
          >
            View the MIT License notice required by pyembroidery
          </summary>
          <pre
            class="whitespace-pre-wrap text-xs text-[var(--text-secondary)] bg-[var(--surface-card)] border border-[var(--border-default)] rounded-lg p-3 mt-2 overflow-x-auto font-mono">{PYEMBROIDERY_MIT_NOTICE}</pre>
        </details>
      </div>
    </div>
  </div>

  <div class="route-card rounded-xl shadow p-6 flex items-center justify-between gap-4">
    <p class="text-sm text-[var(--text-secondary)]">
      The full open-source application licence, Rust crate credits, and frontend package credits are
      available here.
    </p>
    <a href="#/about/licence" class="btn-primary shrink-0">
      View Open-Source Licences &amp; Credits
    </a>
  </div>
</div>

<ConfirmDeactivateLicenceModal
  open={showDeactivateModal}
  isDeactivating={isDeactivatingLicence}
  onClose={() => (showDeactivateModal = false)}
  onConfirm={handleConfirmDeactivateLicence}
/>
