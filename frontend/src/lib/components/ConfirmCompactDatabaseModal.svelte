<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /**
   * Modal dialog asking the user to confirm database compaction, with an option
   * to create a quick database backup first for extra peace of mind.
   *
   * @type {{
   *   open?: boolean,
   *   dbSizeBytes?: number,
   *   reclaimableBytes?: number,
   *   driveLetter?: string,
   *   isCompacting?: boolean,
   *   isBackingUp?: boolean,
   *   onClose?: () => void,
   *   onCompactOnly?: () => void,
   *   onBackupAndCompact?: () => void
   * }}
   */
  let {
    open = false,
    dbSizeBytes = 0,
    reclaimableBytes = 0,
    driveLetter = "",
    isCompacting = false,
    isBackingUp = false,
    onClose = () => {},
    onCompactOnly = () => {},
    onBackupAndCompact = () => {},
  } = $props();

  let busy = $derived(isCompacting || isBackingUp);

  /** @param {number} bytes */
  function formatBytes(bytes) {
    if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }
    return `${value.toFixed(value >= 100 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  }

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("compact-confirm-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
      },
    };
  }

  /** @param {Event} [event] */
  function handleBackdropClick(event) {
    if (busy) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key === "Escape" && !busy) handleBackdropClick();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="compact-confirm-modal-title"
    data-testid="compact-confirm-modal"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      class="fixed inset-0 bg-transparent border-0 cursor-default"
      aria-label="Close compact confirmation dialog"
      disabled={busy}
      onclick={handleBackdropClick}
    ></button>

    <div
      class="relative z-10 w-full max-w-lg rounded-xl bg-[var(--surface-card)] border border-[var(--border-default)] p-6 shadow-2xl space-y-5"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="flex items-center gap-2.5">
          <span class="text-2xl" aria-hidden="true">🧹</span>
          <h2
            id="compact-confirm-modal-title"
            class="text-lg font-bold text-[var(--text-primary)] m-0"
          >
            Optimize & Compact Database
          </h2>
        </div>
      </div>

      <div class="space-y-3 text-sm text-[var(--text-secondary)]">
        <ul class="space-y-2.5 list-none p-0 m-0">
          <li class="flex items-start gap-2">
            <span class="text-[var(--text-brand)] font-bold mt-0.5">•</span>
            <div>
              <strong class="text-[var(--text-primary)]">What it does:</strong>
              {#if reclaimableBytes > 0}
                Frees up <strong>{formatBytes(reclaimableBytes)}</strong> of unused disk space.
              {:else}
                Reclaims unused space and defragments database tables for faster searches.
              {/if}
            </div>
          </li>
          <li class="flex items-start gap-2">
            <span class="text-emerald-600 dark:text-emerald-400 font-bold mt-0.5">•</span>
            <div>
              <strong class="text-[var(--text-primary)]">Protection:</strong>
              Safe against power loss and sudden shutdowns — existing data is never touched until the
              new copy is completely ready.
            </div>
          </li>
          <li class="flex items-start gap-2">
            <span class="text-indigo-600 dark:text-indigo-400 font-bold mt-0.5">•</span>
            <div>
              <strong class="text-[var(--text-primary)]">Disk space:</strong>
              {#if dbSizeBytes > 0}
                Needs at least <strong>{formatBytes(dbSizeBytes)}</strong> free on
                <strong>{driveLetter || "the catalogue drive"}</strong>.
              {:else}
                Requires free space on <strong>{driveLetter || "the catalogue drive"}</strong> equal to
                at least the current database size.
              {/if}
            </div>
          </li>
        </ul>

        <p class="text-xs text-[var(--text-muted)] pt-1">
          For extra peace of mind, you can create a quick database backup before compacting.
        </p>
      </div>

      <div
        class="flex flex-wrap items-center justify-end gap-2.5 pt-2 border-t border-[var(--border-default)]"
      >
        <button
          type="button"
          class="settings-secondary-button border rounded px-3 py-2 text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          data-testid="compact-confirm-cancel"
          disabled={busy}
          onclick={onClose}
        >
          Cancel
        </button>
        <button
          type="button"
          class="settings-secondary-button border rounded px-3 py-2 text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          data-testid="compact-confirm-compact-only"
          disabled={busy}
          onclick={onCompactOnly}
        >
          {isCompacting && !isBackingUp ? "Compacting…" : "Compact Only"}
        </button>
        <button
          type="button"
          class="settings-primary-button menu-button-primary disabled:opacity-50 disabled:cursor-not-allowed"
          data-testid="compact-confirm-backup-and-compact"
          disabled={busy}
          onclick={onBackupAndCompact}
        >
          {isBackingUp ? "Backing up database…" : isCompacting ? "Compacting…" : "Backup & Compact"}
        </button>
      </div>
    </div>
  </div>
{/if}
