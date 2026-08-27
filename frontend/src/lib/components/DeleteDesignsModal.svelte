<script>
  import { bulkDeleteDesigns } from "../api/commandAdapter";
  import { beginBusy, endBusy } from "../stores/busyStore.js";

  /**
   * Shared unified deletion modal used by both the Browse page and the Design Detail page.
   *
   * When designIds.length === 1, the collapsible preview drawer is auto-hidden
   * (single-design context).
   *
   * @type {{
   *   designIds?: number[],
   *   previewItems?: Array<{id: number, filename: string, filepath: string, dataUrl?: string|null}>,
   *   open?: boolean,
   *   onClose?: () => void,
   *   onDeleted?: (result: any) => void
   * }}
   */
  let {
    designIds = [],
    previewItems = [],
    open = false,
    onClose = () => {},
    onDeleted = () => {},
  } = $props();

  let deleteFile = $state(false);
  let previewOpen = $state(false);
  let busy = $state(false);

  let selectedCount = $derived(designIds.length);
  let isSingleItem = $derived(designIds.length <= 1);

  /** @param {HTMLElement} node */
  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("delete-modal-portal");
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
    if (busy) return;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    onClose();
  }

  async function confirmDelete() {
    if (busy || designIds.length === 0) return;

    busy = true;
    beginBusy("Deleting designs");
    try {
      const result = await bulkDeleteDesigns(designIds, deleteFile);
      if (result.persisted) {
        resetState();
        onDeleted(result);
      } else {
        // Even if not fully persisted, fire callback so caller can handle errors
        resetState();
        onDeleted(result);
      }
    } catch (error) {
      resetState();
      onDeleted({
        source: "mock",
        persisted: false,
        deleted_count: 0,
        files_trashed: 0,
        errors: [String(error)],
      });
    } finally {
      endBusy();
    }
  }

  function resetState() {
    busy = false;
    previewOpen = false;
    deleteFile = false;
  }

  function handleCancel() {
    if (busy) return;
    resetState();
    onClose();
  }

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key === "Escape" && !busy) handleCancel();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="delete-modal-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="delete-modal-title"
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;cursor:default;"
      aria-label="Close delete confirmation"
      onclick={handleBackdropClick}
    ></button>

    <div
      class="delete-modal-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));background:#ffffff;border-radius:0.5rem;box-shadow:0 20px 60px rgba(0,0,0,0.35);overflow:hidden;"
    >
      <div
        class="delete-modal-header"
        style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;padding:1rem 1.5rem 0;"
      >
        <h2 id="delete-modal-title" class="text-lg font-bold text-gray-800" style="margin:0;">
          Delete selected design{selectedCount === 1 ? "" : "s"}?
        </h2>
      </div>

      <div class="delete-modal-body" style="overflow-y:auto;flex:1;padding:1rem 1.5rem;">
        {#if selectedCount > 0}
          <p class="text-xs text-gray-500 font-semibold" style="margin:0 0 0.75rem 0;">
            {selectedCount} design{selectedCount === 1 ? "" : "s"} selected.
          </p>
        {/if}

        <!-- File action toggle -->
        <div class="border rounded p-3 mb-3 bg-gray-50 space-y-2">
          <p class="text-xs font-semibold text-gray-700">What should happen to the source files?</p>
          <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
            <input
              type="radio"
              name="delete-file-action"
              class="accent-indigo-600"
              checked={!deleteFile}
              disabled={busy}
              onchange={() => {
                deleteFile = false;
              }}
            />
            <span>Remove from catalogue only (keep files on disk)</span>
          </label>
          <label class="flex items-center gap-2 text-xs text-gray-700 cursor-pointer">
            <input
              type="radio"
              name="delete-file-action"
              class="accent-indigo-600"
              checked={deleteFile}
              disabled={busy}
              onchange={() => {
                deleteFile = true;
              }}
            />
            <span>Move source file{selectedCount === 1 ? "" : "s"} to recycle bin</span>
          </label>
          {#if deleteFile}
            <p
              class="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded px-2 py-1.5"
            >
              ⚠️ Source file{selectedCount === 1 ? "" : "s"} will be moved to the system recycle bin.
              You can restore them from there if needed.
            </p>
          {/if}
        </div>

        <!-- Collapsible preview list (hidden for single-item context) -->
        {#if !isSingleItem}
          <details class="border rounded p-2 bg-gray-50" bind:open={previewOpen}>
            <summary
              class="text-xs font-semibold text-gray-600 cursor-pointer select-none list-none flex items-center gap-1"
            >
              <span>{previewOpen ? "▼" : "▶"}</span>
              <span>Review selected designs ({previewItems.length})</span>
            </summary>
            <div class="mt-2 space-y-1 max-h-48 overflow-y-auto">
              {#each previewItems as item (item.id)}
                <div class="flex items-center gap-2 px-2 py-1 bg-white rounded border text-xs">
                  {#if item.dataUrl}
                    <img
                      src={item.dataUrl}
                      alt={item.filename}
                      class="w-8 h-8 object-contain rounded"
                    />
                  {:else}
                    <div
                      class="w-8 h-8 bg-gray-100 rounded flex items-center justify-center text-gray-400 font-bold"
                    >
                      ?
                    </div>
                  {/if}
                  <div class="flex-1 min-w-0">
                    <p class="font-medium text-gray-800 truncate">{item.filename}</p>
                    <p class="text-gray-400 truncate" title={item.filepath}>
                      {item.filepath || "No filepath"}
                    </p>
                  </div>
                </div>
              {/each}
            </div>
          </details>
        {/if}
      </div>

      <div
        class="delete-modal-footer"
        style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;padding:1rem 1.5rem;border-top:1px solid #e5e7eb;"
      >
        <button type="button" class="menu-button-secondary" onclick={handleCancel} disabled={busy}>
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-primary"
          style="background-color:#dc2626;border-color:#dc2626;"
          onclick={confirmDelete}
          disabled={busy || designIds.length === 0}
        >
          {busy ? "Deleting..." : `Delete ${selectedCount} design${selectedCount === 1 ? "" : "s"}`}
        </button>
      </div>
    </div>
  </div>
{/if}
