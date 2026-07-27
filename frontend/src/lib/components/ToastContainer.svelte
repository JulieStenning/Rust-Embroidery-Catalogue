<script lang="ts">
  import { toasts, removeToast } from "../stores/toastStore";

  /**
   * Global floating toast container.
   *
   * Renders a fixed-position stack of transient notifications in the top-right
   * corner of the viewport.  Auto-dismiss is handled by the store; manual
   * dismissal is available via the × close button on each toast.
   */
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast-item toast-item-enter"
        class:toast-success={toast.type === "success"}
        class:toast-error={toast.type === "error"}
        class:toast-info={toast.type === "info"}
        role="alert"
      >
        <span class="toast-message">{toast.message}</span>
        <button
          class="toast-close"
          onclick={() => removeToast(toast.id)}
          aria-label="Dismiss"
        >&times;</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 24rem;
    pointer-events: none;
  }

  .toast-item {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.625rem 0.875rem;
    border-radius: 0.375rem;
    border: 1px solid;
    font-size: 0.8125rem;
    line-height: 1.35;
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    animation: toast-slide-in 250ms ease-out;
    word-break: break-word;
  }

  .toast-item.toast-success {
    background-color: #f0fdf4;
    border-color: #86efac;
    color: #166534;
  }

  .toast-item.toast-error {
    background-color: #fef2f2;
    border-color: #fca5a5;
    color: #991b1b;
  }

  .toast-item.toast-info {
    background-color: #eff6ff;
    border-color: #93c5fd;
    color: #1e40af;
  }

  .toast-message {
    flex: 1;
    min-width: 0;
  }

  .toast-close {
    flex-shrink: 0;
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    line-height: 1;
    font-size: 1.125rem;
    font-weight: 700;
    color: inherit;
    opacity: 0.5;
    cursor: pointer;
    transition: opacity 120ms ease;
  }

  .toast-close:hover {
    opacity: 1;
  }

  @keyframes toast-slide-in {
    from {
      opacity: 0;
      transform: translateX(1.5rem);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @media (prefers-color-scheme: dark) {
    .toast-item.toast-success {
      background-color: #052e16;
      border-color: #166534;
      color: #bbf7d0;
    }

    .toast-item.toast-error {
      background-color: #450a0a;
      border-color: #991b1b;
      color: #fca5a5;
    }

    .toast-item.toast-info {
      background-color: #1e1b4b;
      border-color: #3730a3;
      color: #c7d2fe;
    }
  }
</style>
