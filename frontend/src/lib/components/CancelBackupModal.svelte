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
   *   databaseCopyDone?: boolean,
   *   onClose?: () => void,
   *   onConfirm?: () => void
   * }}
   */
  let {
    open = false,
    activeKind = "both",
    databaseCopyDone = false,
    onClose = () => {},
    onConfirm = () => {},
  } = $props();

  /** Whether a database backup is part of the running action at all. */
  let hasDatabaseCopy = $derived(activeKind === "database" || activeKind === "both");
  /**
   * Whether the design-file note should be shown. A combined backup copies the
   * database first, then the design files, so during the database phase the
   * design files have not been copied yet and the note is withheld; it appears
   * only once the database copy has completed (or for a designs-only backup).
   */
  let showsDesignsNotes = $derived(
    activeKind === "designs" || (activeKind === "both" && databaseCopyDone)
  );

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
    role="dialog"
    aria-modal="true"
    aria-labelledby="cancel-backup-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      class="cancel-backup-modal-backdrop"
      aria-label="Close cancel backup confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div class="cancel-backup-modal-dialog">
      <div class="cancel-backup-modal-header">
        <h2 id="cancel-backup-modal-title" class="text-lg font-bold text-gray-800 m-0">
          Are you sure you want to cancel the backup?
        </h2>
      </div>

      <div class="cancel-backup-modal-body">
        {#if hasDatabaseCopy}
          <p class="text-sm text-gray-700 mb-3">
            {databaseCopyDone
              ? "The database copy has completed."
              : "The database copy is currently running. If you proceed, the database backup will be aborted and the incomplete database file will be deleted."}
          </p>
        {/if}

        {#if showsDesignsNotes}
          <p class="text-sm text-gray-700 m-0">
            Any design files already copied up to the point of cancellation will not be undone —
            they will remain in the destination folder.
          </p>
        {/if}
      </div>

      <div class="cancel-backup-modal-footer">
        <button type="button" class="menu-button-secondary" onclick={onClose}>
          Continue backup
        </button>
        <button
          type="button"
          class="menu-button-danger"
          onclick={onConfirm}
        >
          Cancel backup
        </button>
      </div>
    </div>
  </div>
{/if}
