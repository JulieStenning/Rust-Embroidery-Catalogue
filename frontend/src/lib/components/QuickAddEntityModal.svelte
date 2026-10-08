<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { createDesigner, createSource } from "../api/commandAdapter";
  import { addToast } from "../stores/toastStore.js";

  /**
   * @type {{
   *   open?: boolean,
   *   entityType?: "designer" | "source",
   *   existingNames?: string[],
   *   onClose?: () => void,
   *   onCreated?: (item: { id: number, name: string }) => void
   * }}
   */
  let {
    open = false,
    entityType = "designer",
    existingNames = [],
    onClose = () => {},
    onCreated = () => {},
  } = $props();

  let name = $state("");
  let isSubmitting = $state(false);
  let errorMessage = $state("");
  /** @type {HTMLInputElement | null} */
  let inputElement = $state(null);

  const title = $derived(entityType === "designer" ? "Add New Designer" : "Add New Source");
  const inputLabel = $derived(entityType === "designer" ? "Designer name" : "Source name");
  const inputPlaceholder = $derived(
    entityType === "designer" ? "e.g. Urban Threads" : "e.g. Purchased"
  );

  $effect(() => {
    if (open) {
      name = "";
      errorMessage = "";
      isSubmitting = false;
      setTimeout(() => {
        inputElement?.focus();
      }, 50);
    }
  });

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("quick-add-entity-modal-portal");
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
    if (isSubmitting) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key === "Escape" && !isSubmitting) {
      handleBackdropClick();
    }
  }

  /** @param {SubmitEvent} event */
  async function handleSubmit(event) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      errorMessage = "Please enter a name.";
      inputElement?.focus();
      return;
    }

    const lower = trimmed.toLowerCase();
    const isDuplicate = existingNames.some(
      (existing) =>
        String(existing || "")
          .trim()
          .toLowerCase() === lower
    );
    if (isDuplicate) {
      errorMessage = `A ${entityType} named "${trimmed}" already exists.`;
      inputElement?.focus();
      return;
    }

    errorMessage = "";
    isSubmitting = true;

    try {
      const response =
        entityType === "designer" ? await createDesigner(trimmed) : await createSource(trimmed);

      if (!response?.persisted || !response?.item) {
        errorMessage = response?.error || `Failed to create ${entityType}.`;
        addToast(errorMessage, "error");
        return;
      }

      const createdItem = {
        id: Number(response.item.id),
        name: String(response.item.name || trimmed),
      };

      addToast(
        `${entityType === "designer" ? "Designer" : "Source"} "${createdItem.name}" added.`,
        "success"
      );
      onCreated(createdItem);
      onClose();
    } catch (err) {
      errorMessage = String(err || `Failed to create ${entityType}.`);
      addToast(errorMessage, "error");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="quick-add-modal-title"
    data-testid="quick-add-entity-modal"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      class="modal-backdrop"
      aria-label={`Close ${title}`}
      onclick={handleBackdropClick}
    ></button>

    <div class="modal-dialog">
      <div class="modal-header">
        <h2 id="quick-add-modal-title" class="text-lg font-bold text-[var(--text-primary)] m-0">
          {title}
        </h2>
      </div>

      <form onsubmit={handleSubmit} class="flex flex-col m-0">
        <div class="modal-body space-y-3">
          <label for="quick-add-name-input" class="ui-field-label text-sm block">
            <span class="block font-medium mb-1">{inputLabel}</span>
            <input
              id="quick-add-name-input"
              bind:this={inputElement}
              bind:value={name}
              class="ui-text-input ui-control-text-inset w-full border rounded px-3 py-2 text-sm"
              placeholder={inputPlaceholder}
              disabled={isSubmitting}
              autocomplete="off"
            />
          </label>

          {#if errorMessage}
            <p class="text-xs text-[var(--notice-error-text)] m-0" role="alert">{errorMessage}</p>
          {/if}
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="menu-button-secondary"
            onclick={onClose}
            disabled={isSubmitting}
          >
            Cancel
          </button>
          <button
            type="submit"
            class="menu-button-primary ui-action-button ui-action-button-primary"
            disabled={isSubmitting || !name.trim()}
          >
            {isSubmitting
              ? "Adding..."
              : `Add ${entityType === "designer" ? "Designer" : "Source"}`}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
