<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  /**
   * Confirmation dialog shown before deactivating an active licence key.
   */
  interface Props {
    open?: boolean;
    isDeactivating?: boolean;
    onClose?: () => void;
    onConfirm?: () => void;
  }

  let {
    open = false,
    isDeactivating = false,
    onClose = () => {},
    onConfirm = () => {},
  }: Props = $props();

  function portalToBody(node: HTMLElement) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("confirm-deactivate-licence-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
      },
    };
  }

  function handleBackdropClick(event?: Event) {
    if (isDeactivating) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape" && !isDeactivating) handleBackdropClick();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    use:portalToBody
    class="modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-deactivate-licence-modal-title"
    data-testid="confirm-deactivate-licence-modal"
  >
    <button
      type="button"
      class="modal-backdrop"
      aria-label="Close licence deactivation confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div class="modal-dialog">
      <div class="modal-header">
        <h2
          id="confirm-deactivate-licence-modal-title"
          class="text-lg font-bold text-gray-800 dark:text-gray-100 m-0"
        >
          Deactivate licence?
        </h2>
      </div>

      <div class="modal-body">
        <p class="text-sm text-gray-700 dark:text-gray-300 mb-3">
          Are you sure you want to deactivate this licence key?
        </p>

        <p class="text-xs text-amber-800 dark:text-amber-300 m-0">
          You will need to re-enter a valid licence key to continue using the application.
        </p>
      </div>

      <div class="modal-footer">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={onClose}
          disabled={isDeactivating}
          data-testid="cancel-deactivate-licence-button"
        >
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-danger"
          onclick={onConfirm}
          disabled={isDeactivating}
          data-testid="confirm-deactivate-licence-button"
        >
          {isDeactivating ? "Deactivating…" : "Deactivate licence"}
        </button>
      </div>
    </div>
  </div>
{/if}
