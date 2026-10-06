<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { SvelteSet } from "svelte/reactivity";

  /** @typedef {import("../../types/ipc").BrowseDesignCard} BrowseDesignCard */
  /** @typedef {import("../../types/ipc").ProjectListItem} ProjectListItem */

  /**
   * @type {{
   *   browseItems: BrowseDesignCard[],
   *   browseLoading: boolean,
   *   browsePageRows: BrowseDesignCard[][],
   *   browseGridColumns: number,
   *   browseSelectedIds: SvelteSet<number>,
   *   browseSelectionLocked: boolean,
   *   browseBulkDeleteMax: number,
   *   browsePreviewById: Record<number, string | null>,
   *   browsePreviewsLoading: boolean,
   *   browseProjects: ProjectListItem[],
   *   browseGridContainer: HTMLDivElement | null,
   *   onCancelSearch: () => void,
   *   isBrowseRowFullySelected: (rowItems: BrowseDesignCard[]) => boolean,
   *   onToggleBrowseRowSelection: (rowItems: BrowseDesignCard[]) => void,
   *   onToggleBrowseCardSelection: (id: number, checked: boolean) => void,
   *   onHandleBrowseCardOpenDetail: (event: MouseEvent, item: BrowseDesignCard) => void,
   *   onHandleBrowseCardProjectDetailsToggle: (item: BrowseDesignCard, target: HTMLDetailsElement) => void,
   *   isBrowseCardProjectChecked: (item: BrowseDesignCard, projectId: number) => boolean,
   *   onUpdateBrowseCardProjectPending: (designId: number, projectId: number, checked: boolean) => void
   * }}
   */
  let {
    browseItems,
    browseLoading,
    browsePageRows,
    browseGridColumns,
    browseSelectedIds,
    browseSelectionLocked,
    browseBulkDeleteMax,
    browsePreviewById,
    browsePreviewsLoading,
    browseProjects,
    browseGridContainer = $bindable(null),
    onCancelSearch,
    isBrowseRowFullySelected,
    onToggleBrowseRowSelection,
    onToggleBrowseCardSelection,
    onHandleBrowseCardOpenDetail,
    onHandleBrowseCardProjectDetailsToggle,
    isBrowseCardProjectChecked,
    onUpdateBrowseCardProjectPending,
  } = $props();
</script>

<div
  bind:this={browseGridContainer}
  class="browse-grid-rows flex flex-col gap-5 {browseLoading && browseItems.length > 0
    ? 'opacity-50 pointer-events-none transition-opacity duration-200'
    : ''}"
>
  {#if browseLoading && browseItems.length === 0}
    <div class="py-16 text-center flex flex-col items-center justify-center">
      <svg
        class="animate-spin h-8 w-8 text-indigo-600 mb-3"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
        ></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
      <p class="text-sm font-semibold text-gray-700">Loading designs...</p>
      <p class="text-xs text-gray-500 mt-1 mb-4">Querying catalogue records...</p>
      <button
        type="button"
        class="inline-flex items-center px-3 py-1.5 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 hover:text-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        onclick={onCancelSearch}
      >
        Cancel search
      </button>
    </div>
  {:else if browseItems.length === 0}
    <p class="text-center py-12 text-gray-500 font-medium">No designs match your filters.</p>
  {:else}
    {#each browsePageRows as rowItems, rowIndex (rowIndex)}
      <div
        class="browse-grid-row grid gap-4"
        style={`grid-template-columns: 2rem repeat(${browseGridColumns}, minmax(0, 1fr));`}
      >
        <!-- Row selector checkbox -->
        <label
          class="browse-row-selector flex items-center justify-center bg-indigo-50 rounded cursor-pointer select-none"
          title={`Select row ${rowIndex + 1}`}
        >
          <span class="sr-only">Select row {rowIndex + 1}</span>
          <input
            type="checkbox"
            class="browse-row-checkbox rounded accent-indigo-500"
            checked={isBrowseRowFullySelected(rowItems)}
            onchange={() => onToggleBrowseRowSelection(rowItems)}
          />
        </label>

        {#each rowItems as item (item.id)}
          <article
            class="browse-card border rounded-lg overflow-hidden shadow-sm flex flex-col hover:shadow transition relative"
            data-id={item.id}
          >
            <!-- Selection checkbox -->
            <label class="absolute top-2.5 left-2.5 z-10 cursor-pointer select-none">
              <input
                type="checkbox"
                class="browse-design-checkbox rounded accent-indigo-650"
                checked={browseSelectedIds.has(item.id)}
                oninput={() =>
                  onToggleBrowseCardSelection(item.id, !browseSelectedIds.has(item.id))}
                disabled={browseSelectionLocked ||
                  (browseSelectedIds.size >= browseBulkDeleteMax &&
                    !browseSelectedIds.has(item.id))}
              />
            </label>

            <button
              class="browse-card-link w-full text-left flex flex-col flex-1"
              onclick={(event) => onHandleBrowseCardOpenDetail(event, item)}
            >
              {#if item.isMasterOnly}
                <div
                  class="browse-card-image-frame p-3 flex flex-col items-center justify-center h-48 border-b bg-amber-50/50 text-center"
                  data-testid="design-card-master-only"
                >
                  <span class="text-2xl mb-1" aria-hidden="true">🎨</span>
                  <p class="text-xs font-semibold text-amber-900 mb-1">Outline Master File</p>
                  <p class="text-[11px] text-amber-800 leading-snug px-2">
                    Export to a machine stitch format to generate preview and stitch data.
                  </p>
                </div>
              {:else if browsePreviewById[item.id]}
                <div
                  class="browse-card-image-frame p-2 flex items-center justify-center h-48 border-b"
                >
                  <img
                    src={browsePreviewById[item.id]}
                    alt={item.filename}
                    class="browse-card-image max-h-full object-contain"
                    loading="lazy"
                  />
                </div>
              {:else}
                <div
                  class="browse-card-image-frame p-3 flex items-center justify-center h-48 border-b"
                >
                  <p
                    class="text-xs ui-help-note text-center leading-snug"
                    data-testid="design-card-no-preview"
                  >
                    {browsePreviewsLoading
                      ? "Loading image..."
                      : "Preview could not be generated — the file may be corrupt or unreadable"}
                  </p>
                </div>
              {/if}
              <div class="browse-card-meta p-4 flex-1 flex flex-col justify-between">
                <div>
                  <div class="browse-card-title-row flex items-start justify-between gap-1.5">
                    <p
                      class="browse-card-title text-sm font-semibold truncate flex-1"
                      title={item.filename}
                    >
                      {item.filename}
                    </p>
                    {#if item.imageTagsVerified && item.stitchingTagsVerified}
                      <span
                        class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-green-500"
                        title="Verified"
                        aria-label="Verified"
                      >
                        ✓
                      </span>
                    {:else if item.imageTagsVerified}
                      <span
                        class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-amber-400"
                        title="Image Verified, Stitching Unverified"
                        aria-label="Image Verified, Stitching Unverified"
                      >
                        ◐
                      </span>
                    {:else if item.stitchingTagsVerified}
                      <span
                        class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-amber-400"
                        title="Stitching Verified, Image Unverified"
                        aria-label="Stitching Verified, Image Unverified"
                      >
                        ◑
                      </span>
                    {:else}
                      <span
                        class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0 bg-red-500"
                        title="Unverified"
                        aria-label="Unverified"
                      >
                        ○
                      </span>
                    {/if}
                  </div>
                  {#if item.masterFilepath && !item.isMasterOnly}
                    <span
                      class="inline-block text-[10px] font-semibold px-1.5 py-0.5 rounded bg-purple-100 text-purple-700 border border-purple-200 mt-1"
                      title={`Master file: ${item.masterFilepath}`}
                    >
                      🎨 Master: .{item.masterFilepath.split(".").pop()?.toLowerCase()}
                    </span>
                  {/if}
                  <p class="browse-card-hoop text-xs font-semibold mt-1">
                    {item.hoop || "Hoop unknown"}
                  </p>
                  {#if item.projects.length > 0}
                    <p
                      class="browse-card-projects text-[11px] mt-1 truncate"
                      title={item.projects.join(", ")}
                    >
                      {item.projects.join(", ")}
                    </p>
                  {/if}
                </div>
                <div class="pt-2">
                  {#if item.tags.length > 0}
                    <p class="browse-card-tags text-[11px] truncate" title={item.tags.join(", ")}>
                      {item.tags.join(", ")}
                    </p>
                  {:else}
                    <p class="browse-card-tags text-[11px] text-gray-400 italic">No tags</p>
                  {/if}
                  <p
                    class="browse-card-rating text-xs mt-1"
                    aria-label={item.rating != null && item.rating > 0
                      ? `Rating ${item.rating} out of 5`
                      : "Not rated"}
                  >
                    {#if item.rating != null && item.rating > 0}
                      <span class="text-amber-500">★</span>
                      <span class="font-bold ml-0.5">{item.rating}</span>
                    {:else}
                      <span class="text-gray-400">☆ —</span>
                    {/if}
                  </p>
                </div>
              </div>
            </button>

            <details
              class="browse-card-project-details px-4 py-2 border-t no-print"
              ontoggle={(event) =>
                onHandleBrowseCardProjectDetailsToggle(item, event.currentTarget)}
            >
              <summary
                class="browse-card-project-summary text-xs font-semibold cursor-pointer select-none"
              >
                + Add to project
              </summary>
              <div
                class="ui-checkbox-list-shell mt-1.5 max-h-36 overflow-auto px-2 py-1.5 border rounded space-y-1"
              >
                {#each browseProjects as project}
                  <label class="ui-field-label flex items-center gap-1.5 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      class="ui-checkbox accent-indigo-650 rounded"
                      checked={isBrowseCardProjectChecked(item, project.id)}
                      onchange={(event) =>
                        onUpdateBrowseCardProjectPending(
                          item.id,
                          project.id,
                          event.currentTarget.checked
                        )}
                    />
                    <span>{project.name}</span>
                  </label>
                {:else}
                  <p class="text-[11px] text-gray-500 italic px-1 py-0.5">
                    No projects found. Create one first.
                  </p>
                {/each}
              </div>
            </details>
          </article>
        {/each}
      </div>
    {/each}
  {/if}
</div>
