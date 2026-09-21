<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount, onDestroy } from "svelte";
  import { addToast } from "../stores/toastStore.js";
  import { busyState, beginBusy, endBusy } from "../stores/busyStore.js";
  import {
    detectDesignFilesAbsentFromDatabase,
    importUnmatchedDesignFiles,
    requestCancelRestore,
  } from "../api/commandAdapter";
  import {
    unmatchedFilesStore,
    setUnmatchedFilesDetected,
    dismissUnmatchedFiles,
  } from "../stores/unmatchedFilesStore.js";
  import { restoreProgressStore, resetRestoreProgress } from "../stores/restoreProgressStore.js";
  import { initRestoreProgressEvents } from "../services/restoreEvents.js";

  let busyActive = $derived($busyState.active);
  let scanning = $state(false);
  let importing = $state(false);
  let cancelling = $state(false);
  let totalToImport = $state(0);
  /** @type {(() => void) | null} */
  let unlistenRestore = null;

  onMount(async () => {
    try {
      unlistenRestore = await initRestoreProgressEvents();
    } catch (error) {
      console.info("Restore progress events unavailable.", error);
    }
  });

  onDestroy(() => {
    if (unlistenRestore) {
      unlistenRestore();
      unlistenRestore = null;
    }
  });

  let importButtonLabel = $derived.by(() => {
    if (importing) {
      const progress = $restoreProgressStore;
      if (progress.active && progress.scope === "import-unmatched" && progress.scanned > 0) {
        const total =
          progress.total > 0 ? progress.total : totalToImport || $unmatchedFilesStore.count;
        const current = Math.min(progress.scanned, total);
        return `Processing ${current} of ${total} files`;
      }
      return "Importing…";
    }
    return `Import ${$unmatchedFilesStore.count} file(s)`;
  });

  /** Scan for design files on disk that have no catalogue record. */
  async function handleScan() {
    if (scanning || busyActive) return;
    scanning = true;
    beginBusy("Scanning for unmatched design files");
    try {
      const result = await detectDesignFilesAbsentFromDatabase();
      if (result.error) {
        addToast(`Scan failed: ${result.error}`, "error");
        return;
      }
      const unmatched = Number(result.unmatched || 0);
      if (unmatched > 0) {
        setUnmatchedFilesDetected(
          unmatched,
          Number(result.checked) || 0,
          Array.isArray(result.sample) ? result.sample : []
        );
      } else {
        dismissUnmatchedFiles();
        addToast(
          `No unmatched design files found (checked ${Number(result.checked) || 0}).`,
          "success"
        );
      }
    } catch (error) {
      addToast(`Scan failed: ${error}`, "error");
    } finally {
      scanning = false;
      endBusy();
    }
  }

  /** Batch-import unmatched design files as new catalogue records. */
  async function handleImport() {
    if (importing || busyActive) return;
    resetRestoreProgress();
    importing = true;
    cancelling = false;
    totalToImport = $unmatchedFilesStore.count;
    beginBusy("Importing unmatched design files");
    try {
      const result = await importUnmatchedDesignFiles();
      if (result.cancelled) {
        addToast(`Import cancelled — ${result.imported} file(s) imported.`, "warning");
        dismissUnmatchedFiles();
        return;
      }
      const flagged = Number(result.flagged ?? 0);
      const failed = Number(result.failed ?? 0);
      const parts = [`Imported ${result.imported} unmatched file(s).`];
      if (flagged > 0) {
        parts.push(
          `${flagged} need attention (preview could not be generated) — regenerate in Batch Operations.`
        );
      }
      if (failed > 0) {
        parts.push(`${failed} failed.`);
      }
      addToast(parts.join(" "), flagged > 0 || failed > 0 ? "warning" : "success");
      dismissUnmatchedFiles();
    } catch (error) {
      addToast(`Import failed: ${error}`, "error");
    } finally {
      importing = false;
      cancelling = false;
      totalToImport = 0;
      resetRestoreProgress();
      endBusy();
    }
  }

  /** Ask the backend to stop the running import after the current file. */
  async function handleCancelImport() {
    if (!importing || cancelling) return;
    cancelling = true;
    try {
      await requestCancelRestore();
    } catch {
      cancelling = false;
    }
  }
</script>

<div class="route-card rounded shadow p-6 space-y-3">
  <h2 class="text-base font-semibold text-gray-800">Find unmatched design files</h2>
  <p class="text-sm text-gray-600">
    Scans <code>MachineEmbroideryDesigns</code> for design files that have no record in the catalogue
    — for example after syncing designs from a backup without restoring the database. You can then import
    them as new catalogue records.
  </p>
  <div class="flex gap-2 pt-1">
    <button
      type="button"
      class="settings-primary-button menu-button-primary"
      disabled={scanning || busyActive}
      onclick={handleScan}
      data-testid="scan-unmatched-button"
    >
      {scanning ? "Scanning…" : "Scan for unmatched files"}
    </button>
  </div>
</div>

{#if $unmatchedFilesStore.showPrompt}
  <div class="route-card rounded shadow p-6 space-y-3" data-testid="unmatched-files-prompt">
    <h2 class="text-base font-semibold text-gray-800">Unmatched files found</h2>
    <p class="text-sm text-gray-600">
      {$unmatchedFilesStore.count} design file(s) on disk have no record in the catalogue
      {$unmatchedFilesStore.checked > 0 ? `(scanned ${$unmatchedFilesStore.checked})` : ""}. You can
      import them as new catalogue records.
    </p>
    <div class="flex gap-2 pt-1">
      <button
        type="button"
        class="settings-primary-button menu-button-primary"
        disabled={importing || busyActive}
        onclick={handleImport}
      >
        {importButtonLabel}
      </button>
      {#if importing}
        <button
          type="button"
          class="menu-button-danger"
          onclick={handleCancelImport}
          disabled={cancelling}
          data-testid="cancel-unmatched-import"
        >
          {cancelling ? "Cancelling…" : "Cancel"}
        </button>
      {/if}
      <button
        type="button"
        class="menu-button-secondary"
        onclick={dismissUnmatchedFiles}
        disabled={busyActive}>Dismiss</button
      >
    </div>
  </div>
{/if}
