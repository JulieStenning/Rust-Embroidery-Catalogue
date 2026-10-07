<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  /**
   * Confirmation dialog shown before deleting orphaned records from the database.
   */
  interface Props {
    open?: boolean;
    mode?: "selected" | "all";
    count?: number;
    isDeleting?: boolean;
    onClose?: () => void;
    onConfirm?: () => void;
  }

  let {
    open = false,
    mode = "selected",
    count = 0,
    isDeleting = false,
    onClose = () => {},
    onConfirm = () => {},
  }: Props = $props();

  let isAll = $derived(mode === "all");
  let dialogTitle = $derived(
    isAll ? "Delete all orphan records?" : "Delete selected records?"
  );
  let confirmLabel = $derived(
    isDeleting
      ? "Deleting…"
      : isAll
        ? "Delete all"
        : count === 1
          ? "Delete record"
          : "Delete records"
  );

  function portalToBody(node: HTMLElement) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("confirm-delete-orphans-modal-portal");
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
    if (isDeleting) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape" && !isDeleting) handleBackdropClick();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-delete-orphans-modal-title"
    data-testid="confirm-delete-orphans-modal"
  >
    <button
      type="button"
      class="modal-backdrop"
      aria-label="Close orphan deletion confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div class="modal-dialog">
      <div class="modal-header">
        <h2
          id="confirm-delete-orphans-modal-title"
          class="text-lg font-bold text-gray-800 dark:text-gray-100 m-0"
        >
          {dialogTitle}
        </h2>
      </div>

      <div class="modal-body">
        {#if isAll}
          <p class="text-sm text-gray-700 dark:text-gray-300 mb-3">
            Are you sure you want to delete all <strong class="font-semibold text-gray-900 dark:text-white">{count}</strong> orphaned records from the catalogue database?
          </p>
        {:else}
          <p class="text-sm text-gray-700 dark:text-gray-300 mb-3">
            Are you sure you want to delete <strong class="font-semibold text-gray-900 dark:text-white">{count}</strong> selected {count === 1 ? "record" : "records"} from the catalogue database?
          </p>
        {/if}

        <p class="text-xs text-amber-800 dark:text-amber-300 m-0">
          This removes the catalogue database entries for missing files. This action cannot be undone.
        </p>
      </div>

      <div class="modal-footer">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={onClose}
          disabled={isDeleting}
          data-testid="cancel-delete-orphans-button"
        >
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-danger"
          onclick={onConfirm}
          disabled={isDeleting}
          data-testid="confirm-delete-orphans-button"
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
