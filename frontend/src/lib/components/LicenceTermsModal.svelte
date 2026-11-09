<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  import type { DependencyItem } from "../types/licence";

  interface Props {
    open?: boolean;
    item?: DependencyItem | null;
    onClose?: () => void;
  }

  let { open = false, item = null, onClose = () => {} }: Props = $props();

  let copied = $state(false);
  let copyTimeout: ReturnType<typeof setTimeout> | null = null;

  function portalToBody(node: HTMLElement) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("licence-terms-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
        if (copyTimeout) clearTimeout(copyTimeout);
      },
    };
  }

  function handleBackdropClick(event?: Event) {
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape") handleBackdropClick();
  }

  async function handleCopy() {
    if (!item?.text) return;
    try {
      await navigator.clipboard.writeText(item.text);
      copied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      // Ignore clipboard write failure
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && item}
  <div
    use:portalToBody
    class="modal-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-labelledby="licence-terms-modal-title"
    data-testid="licence-terms-modal"
  >
    <button
      type="button"
      class="modal-backdrop"
      aria-label="Close licence terms modal"
      onclick={handleBackdropClick}
    ></button>

    <div
      class="modal-dialog max-w-2xl w-full bg-[var(--surface-card)] border border-[var(--border-default)] rounded-xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden"
    >
      <div
        class="modal-header px-6 py-4 border-b border-[var(--border-default)] flex items-center justify-between gap-4"
      >
        <div>
          <h2
            id="licence-terms-modal-title"
            class="text-base font-bold text-[var(--text-primary)] m-0 font-sans flex items-center gap-2"
          >
            <span>{item.name}</span>
            {#if item.version}
              <span class="text-xs font-normal text-[var(--text-muted)]">(v{item.version})</span>
            {/if}
          </h2>
          <div class="mt-1 flex items-center gap-2">
            <span
              class="text-xs font-semibold px-2 py-0.5 rounded bg-[var(--surface-card-subtle)] text-[var(--text-brand)] border border-[var(--border-default)]"
            >
              {item.licence}
            </span>
            {#if item.repository}
              <a
                href={item.repository}
                target="_blank"
                rel="noopener noreferrer"
                class="text-xs text-[var(--text-brand)] hover:underline truncate max-w-xs"
              >
                {item.repository}
              </a>
            {/if}
          </div>
        </div>
        <button
          type="button"
          class="text-[var(--text-muted)] hover:text-[var(--text-primary)] p-1 rounded transition"
          aria-label="Close modal"
          onclick={onClose}
        >
          <svg
            class="w-5 h-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="modal-body p-6 overflow-y-auto flex-1 font-sans">
        {#if item.text}
          <pre
            data-testid="licence-terms-text"
            class="whitespace-pre-wrap text-xs text-[var(--text-secondary)] bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg p-4 font-mono shadow-inner max-h-[50vh] overflow-y-auto select-text">{item.text}</pre>
        {:else}
          <p class="text-sm text-[var(--text-muted)] italic">
            Full licence text is not bundled locally for this dependency. Please refer to the
            repository link above or the standard
            <strong class="font-semibold text-[var(--text-primary)]">{item.licence}</strong>
            specification.
          </p>
        {/if}
      </div>

      <div
        class="modal-footer px-6 py-3 border-t border-[var(--border-default)] bg-[var(--surface-card-subtle)] flex items-center justify-between gap-3"
      >
        <div>
          {#if item.text}
            <button
              type="button"
              class="menu-button-secondary text-xs px-3 py-1.5"
              onclick={handleCopy}
              data-testid="copy-licence-terms-button"
            >
              {copied ? "Copied!" : "Copy Text"}
            </button>
          {/if}
        </div>
        <button
          type="button"
          class="menu-button-secondary text-xs px-4 py-1.5"
          onclick={onClose}
          data-testid="close-licence-terms-modal"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
