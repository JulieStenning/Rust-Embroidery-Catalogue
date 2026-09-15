<script>
  /**
   * Confirmation dialog shown before deleting a project.
   *
   * @type {{
   *   open?: boolean,
   *   projectName?: string,
   *   isDeleting?: boolean,
   *   onClose?: () => void,
   *   onConfirm?: () => void
   * }}
   */
  let {
    open = false,
    projectName = "",
    isDeleting = false,
    onClose = () => {},
    onConfirm = () => {},
  } = $props();

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("confirm-delete-project-modal-portal");
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
    if (isDeleting) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key === "Escape" && !isDeleting) handleBackdropClick();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="confirm-delete-project-modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-delete-project-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      class="confirm-delete-project-modal-backdrop"
      aria-label="Close project delete confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div class="confirm-delete-project-modal-dialog">
      <div class="confirm-delete-project-modal-header">
        <h2 id="confirm-delete-project-modal-title" class="text-lg font-bold text-gray-800 m-0">
          Delete project?
        </h2>
      </div>

      <div class="confirm-delete-project-modal-body">
        <p class="text-sm text-gray-700 mb-3">
          Are you sure you want to delete project <strong class="font-semibold text-gray-900"
            >"{projectName || "Untitled"}"</strong
          >? This action cannot be undone.
        </p>

        <p class="text-xs text-gray-500 m-0">
          Designs belonging to this project will remain in your catalogue.
        </p>
      </div>

      <div class="confirm-delete-project-modal-footer">
        <button type="button" class="menu-button-secondary" onclick={onClose} disabled={isDeleting}>
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-danger"
          onclick={onConfirm}
          disabled={isDeleting}
        >
          {isDeleting ? "Deleting..." : "Delete project"}
        </button>
      </div>
    </div>
  </div>
{/if}
