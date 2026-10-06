<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { portalToBody } from "../../utils/portal.js";

  /** @typedef {import("../../types/ipc").ProjectListItem} ProjectListItem */

  /**
   * @type {{
   *   showBrowseBulkBar: boolean,
   *   browseSelectedCount: number,
   *   browseBulkProjectDropdownOpen: boolean,
   *   browseProjects: ProjectListItem[],
   *   browseBulkProjectSelection: number[],
   *   browseBulkBarNode: HTMLDivElement | null,
   *   onOpenBulkTagModal: () => void,
   *   onRunBulkVerify: () => void,
   *   onOpenBulkProjectModal: () => void,
   *   onCloseBulkProjectModal: () => void,
   *   onToggleBulkProjectSelection: (projectId: number | string, checked: boolean) => void,
   *   onAddSelectedToProject: () => void,
   *   onOpenBrowseDeleteConfirm: () => void,
   *   onClearBrowseSelection: () => void
   * }}
   */
  let {
    showBrowseBulkBar,
    browseSelectedCount,
    browseBulkProjectDropdownOpen,
    browseProjects,
    browseBulkProjectSelection,
    browseBulkBarNode = $bindable(null),
    onOpenBulkTagModal,
    onRunBulkVerify,
    onOpenBulkProjectModal,
    onCloseBulkProjectModal,
    onToggleBulkProjectSelection,
    onAddSelectedToProject,
    onOpenBrowseDeleteConfirm,
    onClearBrowseSelection,
  } = $props();
</script>

{#if showBrowseBulkBar}
  <div
    bind:this={browseBulkBarNode}
    use:portalToBody
    class="browse-bulk-bar ui-section-shell no-print fixed bottom-0 left-0 right-0 border-t p-4 shadow-lg flex flex-wrap items-center justify-between gap-4 z-40"
  >
    <div class="flex items-center gap-3 text-sm">
      <span class="font-semibold"
        >{browseSelectedCount} design{browseSelectedCount === 1 ? "" : "s"} selected</span
      >
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs"
        onclick={onOpenBulkTagModal}
      >
        Choose tags
      </button>

      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs"
        onclick={onRunBulkVerify}
      >
        Verify tags
      </button>

      <details class="relative" open={browseBulkProjectDropdownOpen} style="display:inline-block;">
        <summary
          class="menu-button-secondary ui-action-button text-xs cursor-pointer select-none list-none"
          onclick={(event) => {
            event.preventDefault();
            if (browseBulkProjectDropdownOpen) {
              onCloseBulkProjectModal();
            } else {
              onOpenBulkProjectModal();
            }
          }}
        >
          Add to project…
        </summary>
        <div
          class="browse-bulk-project-panel absolute bottom-full mb-2 right-0 border rounded shadow-lg p-3 max-h-48 overflow-auto min-w-[12rem] space-y-1.5 z-50"
        >
          {#if browseProjects.length === 0}
            <p class="text-xs text-gray-500 italic">No projects found. Create one first.</p>
          {:else}
            {#each browseProjects as project}
              <label class="ui-field-label flex items-center gap-2 text-xs cursor-pointer">
                <input
                  type="checkbox"
                  class="ui-checkbox accent-indigo-650 rounded"
                  checked={browseBulkProjectSelection.includes(Number(project.id))}
                  onchange={(event) =>
                    onToggleBulkProjectSelection(project.id, event.currentTarget.checked)}
                />
                <span>{project.name}</span>
              </label>
            {/each}
          {/if}
          <div class="pt-2 border-t flex justify-end">
            <button
              type="button"
              class="menu-button-primary text-[10px] py-1 px-2.5"
              onclick={onAddSelectedToProject}
              disabled={browseBulkProjectSelection.length === 0}
            >
              Apply
            </button>
          </div>
        </div>
      </details>

      <button
        type="button"
        class="menu-button-secondary ui-action-button text-xs text-red-500 border-red-200"
        onclick={onOpenBrowseDeleteConfirm}
      >
        Delete selected
      </button>

      <button
        type="button"
        class="menu-button-primary ui-action-button ui-action-button-primary text-xs"
        onclick={onClearBrowseSelection}
      >
        Clear selection
      </button>
    </div>
  </div>
{/if}
