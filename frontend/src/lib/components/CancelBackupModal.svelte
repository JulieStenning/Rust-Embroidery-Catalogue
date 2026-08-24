<script>
  /**
   * Confirmation dialog shown while a backup is running.
   *
   * The dialog itself never blocks background execution: while it is open the
   * running backup command continues in the background. Only after the user
   * confirms does the parent call `requestCancelBackup()` to raise the
   * cooperative cancellation flag on the Rust backend.
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
    activeKind = "both",
    onClose = () => {},
    onConfirm = () => {},
  } = $props();

  let showsDatabaseNotes = $derived(activeKind === "database" || activeKind === "both");
  let showsDesignsNotes = $derived(activeKind === "designs" || activeKind === "both");

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("cancel-backup-modal-portal");
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
    class="cancel-backup-modal-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="cancel-backup-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;cursor:default;"
      aria-label="Close cancel backup confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div
      class="cancel-backup-modal-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(34rem, calc(100vw - 2rem));"
    >
      <div
        class="cancel-backup-modal-header"
        style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;"
      >
        <h2 id="cancel-backup-modal-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Are you sure you want to cancel the backup?
        </h2>
      </div>

      <div
        class="cancel-backup-modal-body"
        style="overflow-y:auto;flex:1;padding:1rem 1.5rem;"
      >
        {#if showsDatabaseNotes}
          <p class="text-sm text-gray-700" style="margin:0 0 0.75rem 0;">
            If the database copy is currently running, any partially created database backup
            file will be aborted and removed.
          </p>
        {/if}

        {#if showsDesignsNotes}
          <p class="text-sm text-gray-700" style="margin:0;">
            Any design files already copied up to the point of cancellation will not be undone —
            they will remain in the destination folder.
          </p>
        {/if}
      </div>

      <div
        class="cancel-backup-modal-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;padding:1rem 1.5rem;border-top:1px solid #e5e7eb;"
      >
        <button type="button" class="menu-button-secondary" onclick={onClose}>
          Continue backup
        </button>
        <button
          type="button"
          class="menu-button-primary"
          style="background-color:#dc2626;border-color:#dc2626;"
          onclick={onConfirm}
        >
          Cancel backup
        </button>
      </div>
    </div>
  </div>
{/if}