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
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-delete-project-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;cursor:default;"
      aria-label="Close project delete confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div
      class="confirm-delete-project-modal-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(34rem, calc(100vw - 2rem));background:#ffffff;border-radius:0.5rem;box-shadow:0 20px 60px rgba(0,0,0,0.35);overflow:hidden;"
    >
      <div
        class="confirm-delete-project-modal-header"
        style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;padding:1rem 1.5rem 0;"
      >
        <h2 id="confirm-delete-project-modal-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Delete project?
        </h2>
      </div>

      <div
        class="confirm-delete-project-modal-body"
        style="overflow-y:auto;flex:1;padding:1rem 1.5rem;"
      >
        <p class="text-sm text-gray-700" style="margin:0 0 0.75rem 0;">
          Are you sure you want to delete project <strong class="font-semibold text-gray-900">"{projectName || "Untitled"}"</strong>? This action cannot be undone.
        </p>

        <p class="text-xs text-gray-500" style="margin:0;">
          Designs belonging to this project will remain in your catalogue.
        </p>
      </div>

      <div
        class="confirm-delete-project-modal-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;padding:1rem 1.5rem;border-top:1px solid #e5e7eb;"
      >
        <button
          type="button"
          class="menu-button-secondary"
          onclick={onClose}
          disabled={isDeleting}
        >
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-primary"
          style="background-color:#dc2626;border-color:#dc2626;"
          onclick={onConfirm}
          disabled={isDeleting}
        >
          {isDeleting ? "Deleting..." : "Delete project"}
        </button>
      </div>
    </div>
  </div>
{/if}
