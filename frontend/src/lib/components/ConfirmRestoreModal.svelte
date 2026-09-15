<script>
  /**
   * Confirmation dialog shown before a destructive restore action overwrites
   * live data. The dialog itself does nothing on its own — the parent proceeds
   * only after the user confirms via `onConfirm`.
   *
   * @type {{
   *   open?: boolean,
   *   activeKind?: "database" | "designs" | "both" | null,
   *   onClose?: () => void,
   *   onConfirm?: () => void
   * }}
   */
  let {
    open = false,
    activeKind = "database",
    onClose = () => {},
    onConfirm = () => {},
  } = $props();

  let showsDatabaseNotes = $derived(activeKind === "database" || activeKind === "both");
  let showsDesignsNotes = $derived(activeKind === "designs" || activeKind === "both");
  let confirmLabel = $derived(
    activeKind === "both"
      ? "Restore both"
      : activeKind === "designs"
        ? "Sync designs"
        : "Restore database"
  );

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("confirm-restore-modal-portal");
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
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (!open) return;
    if (event.key === "Escape") handleBackdropClick();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="confirm-restore-modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-restore-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      class="confirm-restore-modal-backdrop"
      aria-label="Close restore confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div class="confirm-restore-modal-dialog">
      <div class="confirm-restore-modal-header">
        <h2 id="confirm-restore-modal-title" class="text-lg font-bold text-gray-800 m-0">
          Are you sure you want to restore?
        </h2>
      </div>

      <div class="confirm-restore-modal-body">
        <p class="text-sm text-amber-800 mb-3">
          Restoring overwrites current data and cannot be undone from this screen.
        </p>

        {#if showsDatabaseNotes}
          <p class="text-sm text-gray-700 mb-3">
            The current database will be replaced with the selected backup snapshot. A safety copy
            of your current database will be kept before overwriting and will be restored
            automatically if verification fails.
          </p>
        {/if}

        {#if showsDesignsNotes}
          <p class="text-sm text-gray-700 m-0">
            Design files from the backup folder will be copied into MachineEmbroideryDesigns. Files
            already present with identical sizes and timestamps will be skipped. This does not
            change database records.
          </p>
        {/if}
      </div>

      <div class="confirm-restore-modal-footer">
        <button type="button" class="menu-button-secondary" onclick={onClose}> Cancel </button>
        <button
          type="button"
          class="settings-primary-button menu-button-primary"
          onclick={onConfirm}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
