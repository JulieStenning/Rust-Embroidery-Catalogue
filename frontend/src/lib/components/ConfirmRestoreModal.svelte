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
    activeKind === "both" ? "Restore both" : activeKind === "designs" ? "Sync designs" : "Restore database"
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
    if (event.key === "Escape") handleBackdropClick();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="confirm-restore-modal-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-restore-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;cursor:default;"
      aria-label="Close restore confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div
      class="confirm-restore-modal-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(34rem, calc(100vw - 2rem));background:#ffffff;border-radius:0.5rem;box-shadow:0 20px 60px rgba(0,0,0,0.35);overflow:hidden;"
    >
      <div
        class="confirm-restore-modal-header"
        style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;padding:1rem 1.5rem 0;"
      >
        <h2 id="confirm-restore-modal-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Are you sure you want to restore?
        </h2>
      </div>

      <div
        class="confirm-restore-modal-body"
        style="overflow-y:auto;flex:1;padding:1rem 1.5rem;"
      >
        <p class="text-sm text-amber-800" style="margin:0 0 0.75rem 0;">
          Restoring overwrites current data and cannot be undone from this screen.
        </p>

        {#if showsDatabaseNotes}
          <p class="text-sm text-gray-700" style="margin:0 0 0.75rem 0;">
            The current database will be replaced with the selected backup snapshot. A safety copy of
            your current database will be kept before overwriting and will be restored automatically if
            verification fails.
          </p>
        {/if}

        {#if showsDesignsNotes}
          <p class="text-sm text-gray-700" style="margin:0;">
            Design files from the backup folder will be copied into MachineEmbroideryDesigns. Files
            already present with identical sizes and timestamps will be skipped. This does not change database
            records.
          </p>
        {/if}
      </div>

      <div
        class="confirm-restore-modal-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;padding:1rem 1.5rem;border-top:1px solid #e5e7eb;"
      >
        <button type="button" class="menu-button-secondary" onclick={onClose}>
          Cancel
        </button>
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
