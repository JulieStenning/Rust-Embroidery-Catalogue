<script>
  import { onMount, untrack } from "svelte";
  import {
    getDesignDetail,
    getDesignImageDataUrl,
    updateDesignMetadata,
    setDesignRating,
    setDesignStitched,
    setDesignTagsChecked,
    setDesignTags,
    removeDesignTag,
    addDesignToProject,
    removeDesignFromProject,
    bulkDeleteDesigns,
    openDesignInEditor,
    openDesignInExplorer,
    renderDesign3dPreview
  } from "../api/commandAdapter.js";
  import DeleteDesignsModal from "../components/DeleteDesignsModal.svelte";
  import TagSelectionModal from "../components/TagSelectionModal.svelte";
  import TechnicalDataGrid from "../components/TechnicalDataGrid.svelte";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { designSessionStore } from "../stores/designSessionStore.js";

  /**
   * @typedef {Object} DesignDetailItem
   * @property {number} id
   * @property {string} [filename]
   * @property {string} [filepath]
   * @property {string} [image_data_url]
   * @property {string} [image_type]
   * @property {string} [designer]
   * @property {string} [source]
   * @property {string} [hoop]
   * @property {string} [date_added]
   * @property {number} [width_mm]
   * @property {number} [height_mm]
   * @property {number} [stitch_count]
   * @property {number} [color_count]
   * @property {number} [color_change_count]
   * @property {number|null} [rating]
   * @property {boolean} [is_stitched]
   * @property {string} [notes]
   * @property {number|null} [designer_id]
   * @property {number|null} [source_id]
   * @property {boolean} [tags_checked]
   * @property {Array<{id: number, description: string, tag_group?: string}>} [tags]
   * @property {Array<{id: number, name: string}>} [designers]
   * @property {Array<{id: number, name: string}>} [sources]
   * @property {Array<{id: number, name: string}>} [projects]
   * @property {Array<{id: number, name: string}>} [available_projects]
   * @property {Array<{id: number, description: string, tag_group?: string}>} [all_tags]
   */

  let { detailDesignId, detailBrowseIds = [], detailBrowseIndex = -1, navigateTo, onDesignDeleted = () => {} } = $props();

  let detailLoading = $state(false);
  let detailError = $state("");
  let detailSaving = $state(false);
  let detailActionMessage = $state("");
  let detailActionIsError = $state(false);

  /** @type {DesignDetailItem | null} */
  let detailItem = $state(null);

  let ratingHover = $state(0);
  function handleStarClick(/** @type {number} */ score) {
    if (!detailItem?.id || detailSaving) return;
    // If clicking the already-active rating, clear it
    if (detailItem.rating === score) {
      submitDetailRating(null);
    } else {
      submitDetailRating(score);
    }
  }
  /** @param {number} score */
  function onStarMouseEnter(score) {
    ratingHover = score;
  }
  function onStarMouseLeave() {
    ratingHover = 0;
  }
  function onStarFocus(/** @type {number} */ score) {
    ratingHover = score;
  }
  function onStarBlur() {
    ratingHover = 0;
  }
  let effectiveRating = $derived(
    ratingHover > 0
      ? ratingHover
      : (/** @type {DesignDetailItem | null} */ (detailItem))?.rating ?? 0
  );
  let detailSource = $state("mock");
  let detailNotes = $state("");
  let detailDesignerId = $state("");
  let detailSourceId = $state("");
  let detailProjectToAdd = $state("");
  /** @type {number[]} */
  let detailTagSelection = $state([]);

  let detailDeleteModalOpen = $state(false);
  let browseBulkModalOpen = $state(false);

  /** @type {Array<{label: string, value: string | number | null | undefined}>} */
  let technicalItems = $derived(
    (() => {
      const item = /** @type {DesignDetailItem | null} */ (detailItem);
      if (!item) {
        return [
          { label: "Hoop", value: "?" },
          { label: "Date Added", value: "?" },
          { label: "Dimensions", value: "?" },
          { label: "Stitches", value: "?" },
          { label: "Colours", value: "?" },
          { label: "Colour Changes", value: "?" },
        ];
      }
      return [
        { label: "Hoop", value: item.hoop || "Unknown" },
        { label: "Date Added", value: item.date_added || "Unknown" },
        { label: "Dimensions", value: item.width_mm != null && item.height_mm != null ? `${item.width_mm} × ${item.height_mm} mm` : "?" },
        { label: "Stitches", value: item.stitch_count ?? "?" },
        { label: "Colours", value: item.color_count ?? "?" },
        { label: "Colour Changes", value: item.color_change_count ?? "?" },
      ];
    })()
  );

  /** @param {string} message @param {boolean} [isError] */
  function setDetailActionNotice(message, isError = false) {
    detailActionMessage = message;
    detailActionIsError = isError;
  }

  /** @param {number | string} designId */
  async function loadDesignDetail(designId) {
    if (designId == null) return;

    detailLoading = true;
    detailError = "";

    try {
      const result = await getDesignDetail(designId);
      if (designId !== detailDesignId) return;

      detailItem = result.item || null;
      detailSource = result.source || "mock";
      if (!detailItem && result?.error) {
        detailError = `Could not load design detail from Rust backend: ${result.error}`;
      }
      detailNotes = String(detailItem?.notes || "");
      detailDesignerId = detailItem?.designer_id == null ? "" : String(detailItem.designer_id);
      detailSourceId = detailItem?.source_id == null ? "" : String(detailItem.source_id);
      detailTagSelection = Array.isArray(detailItem?.tags)
        ? detailItem.tags.map((tag) => Number(tag?.id)).filter((id) => Number.isFinite(id))
        : [];
      detailProjectToAdd = Array.isArray(detailItem?.available_projects) && detailItem.available_projects.length > 0
        ? String(detailItem.available_projects[0].id)
        : "";
    } catch (error) {
      detailError = `Could not load design detail: ${error}`;
      detailItem = null;
      detailSource = "mock";
      detailProjectToAdd = "";
    } finally {
      detailLoading = false;
    }
  }

  async function refreshDetailAfterAction() {
    if (detailDesignId == null) return;
    await loadDesignDetail(detailDesignId);
  }

  async function saveDetailMetadata() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await updateDesignMetadata(detailItem.id, {
      notes: detailNotes,
      designer_id: detailDesignerId ? Number(detailDesignerId) : null,
      source_id: detailSourceId ? Number(detailSourceId) : null,
    });
    detailSaving = false;

    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      // Track the mutation for browse card sync
      const selectedDesigner = (detailItem.designers || []).find(
        /** @param {{id: number, name: string}} d */ (d) => d.id === Number(detailDesignerId)
      );
      const selectedSource = (detailItem.sources || []).find(
        /** @param {{id: number, name: string}} s */ (s) => s.id === Number(detailSourceId)
      );
      designSessionStore.trackMutation(detailItem.id, {
        designer: selectedDesigner?.name || detailItem.designer,
        source: selectedSource?.name || detailItem.source,
      });
      await refreshDetailAfterAction();
    }
  }

  /** @param {number | null} rating */
  async function submitDetailRating(rating) {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await setDesignRating(detailItem.id, rating);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { rating });
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailStitched() {
    if (!detailItem?.id || detailSaving) return;

    const newStitched = !detailItem?.is_stitched;
    detailSaving = true;
    const result = await setDesignStitched(detailItem.id, newStitched);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { is_stitched: newStitched });
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailTagsChecked() {
    if (!detailItem?.id || detailSaving) return;

    const newChecked = !detailItem?.tags_checked;
    detailSaving = true;
    const result = await setDesignTagsChecked(detailItem.id, newChecked);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { tagsChecked: newChecked });
      await refreshDetailAfterAction();
    }
  }

  async function saveDetailTags() {
    if (!detailItem?.id || detailSaving) return false;

    detailSaving = true;
    const result = await setDesignTags(detailItem.id, detailTagSelection);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      // Compute tag arrays from the selected tag IDs and the all_tags lookup
      const allTags = Array.isArray(detailItem?.all_tags) ? detailItem.all_tags : [];
      const selectedTags = allTags.filter(
        /** @param {{id: number, description: string, tag_group?: string}} t */ (t) =>
          detailTagSelection.includes(t.id)
      );
      const imageTags = selectedTags
        .filter(/** @param {{tag_group?: string}} t */ (t) => t.tag_group === "image")
        .map(/** @param {{description: string}} t */ (t) => t.description);
      const stitchingTags = selectedTags
        .filter(/** @param {{tag_group?: string}} t */ (t) => t.tag_group === "stitching")
        .map(/** @param {{description: string}} t */ (t) => t.description);
      const allTagDescriptions = selectedTags.map(/** @param {{description: string}} t */ (t) => t.description);

      designSessionStore.trackMutation(detailItem.id, {
        tags: allTagDescriptions,
        imageTags,
        stitchingTags,
        tagsChecked: true,
      });
      await refreshDetailAfterAction();
      return true;
    }
    return false;
  }

  /** @param {number} projectId */
  async function addDetailToProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) return;

    detailSaving = true;
    const result = await addDesignToProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      // Build updated project list from existing + the newly added project
      const addedProject = (detailItem?.available_projects || []).find(
        /** @param {{id: number, name: string}} p */ (p) => p.id === projectId
      );
      const currentProjects = Array.isArray(detailItem?.projects)
        ? detailItem.projects.map(/** @param {{name: string}} p */ (p) => p.name)
        : [];
      const updatedProjects = addedProject
        ? [...currentProjects, addedProject.name]
        : currentProjects;
      designSessionStore.trackMutation(detailItem.id, { projects: updatedProjects });
      await refreshDetailAfterAction();
    }
  }

  async function addSelectedDetailProject() {
    if (!detailProjectToAdd) return;
    await addDetailToProject(Number(detailProjectToAdd));
  }

  /** @param {number} projectId */
  async function removeDetailFromProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) return;

    detailSaving = true;
    const result = await removeDesignFromProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      // Build updated project list excluding the removed project
      const currentProjects = Array.isArray(detailItem?.projects)
        ? detailItem.projects
            .filter(/** @param {{id: number}} p */ (p) => p.id !== projectId)
            .map(/** @param {{name: string}} p */ (p) => p.name)
        : [];
      designSessionStore.trackMutation(detailItem.id, { projects: currentProjects });
      await refreshDetailAfterAction();
    }
  }

  async function launchDetailInEditor() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await openDesignInEditor(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted || !result?.result?.success);
  }

  async function launchDetailInExplorer() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await openDesignInExplorer(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted || !result?.result?.success);
  }

  async function renderDetail3dPreview() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await renderDesign3dPreview(detailItem.id);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      const refreshedImage = await getDesignImageDataUrl(detailItem.id);
      if (refreshedImage?.item?.data_url) {
        detailItem = {
          ...detailItem,
          image_data_url: refreshedImage.item.data_url,
          image_type: refreshedImage.item.image_type || detailItem.image_type,
        };
      }
      await refreshDetailAfterAction();
    }
  }

  function openDetailPrintView() {
    if (!detailItem?.id) return;
    navigateTo(`#/designs/${detailItem.id}/print`);
  }

  function goToPreviousDetail() {
    if (detailBrowseIndex <= 0) return;
    const prevId = detailBrowseIds[detailBrowseIndex - 1];
    if (Number.isFinite(prevId)) {
      navigateTo(`#/designs/${prevId}`);
    }
  }

  function goToNextDetail() {
    if (detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1) return;
    const nextId = detailBrowseIds[detailBrowseIndex + 1];
    if (Number.isFinite(nextId)) {
      navigateTo(`#/designs/${nextId}`);
    }
  }

  function openDeleteModal() {
    if (!detailItem?.id || detailSaving) return;
    detailDeleteModalOpen = true;
  }

  function closeDeleteModal() {
    if (detailSaving) return;
    detailDeleteModalOpen = false;
  }

  /** @param {any} result */
  function handleDetailDeleteResult(result) {
    detailDeleteModalOpen = false;
    if (result.persisted) {
      onDesignDeleted();
      navigateTo("#/designs");
    } else {
      setDetailActionNotice(result.errors?.[0] || "Could not delete design.", true);
    }
  }

  function openDetailTagModal() {
    if (!detailItem?.id || detailSaving) return;
    browseBulkModalOpen = true;
  }

  async function closeDetailTagModal() {
    browseBulkModalOpen = false;
    await refreshDetailAfterAction();
  }

  /** @param {number} tagId @param {MouseEvent} event */
  async function handleRemoveTag(tagId, event) {
    event.stopPropagation();
    if (!detailItem?.id || detailSaving) return;

    // Optimistic UI update: immediately remove the tag from the display
    detailItem = {
      ...detailItem,
      tags: (detailItem.tags || []).filter(t => t.id !== tagId),
    };
    detailTagSelection = detailTagSelection.filter(id => id !== tagId);

    // Persist to backend
    detailSaving = true;
    const result = await removeDesignTag(detailItem.id, tagId);
    detailSaving = false;

    if (result.persisted) {
      setDetailActionNotice(result.message, false);
      // Track mutation for browse card sync
      const updatedTags = (detailItem.tags || []).map(t => t.description);
      designSessionStore.trackMutation(detailItem.id, {
        tags: updatedTags,
        imageTags: (detailItem.tags || []).filter(t => t.tag_group === 'image').map(t => t.description),
        stitchingTags: (detailItem.tags || []).filter(t => t.tag_group === 'stitching').map(t => t.description),
      });
    } else {
      setDetailActionNotice(result.message, true);
      // Rollback optimistic update by re-fetching from backend
      await refreshDetailAfterAction();
    }
  }

  /** @param {number | string} rating */
  function ratingToStars(rating) {
    const numeric = Number(rating);
    if (!Number.isFinite(numeric) || numeric <= 0) return "";
    const clamped = Math.min(5, Math.max(0, numeric));
    return `${"★".repeat(clamped)}${"☆".repeat(5 - clamped)}`;
  }

  $effect(() => {
    if (detailDesignId !== null) {
      untrack(() => {
        loadDesignDetail(detailDesignId);
      });
    }
  });

  onMount(() => {
    if (detailDesignId !== null) {
      loadDesignDetail(detailDesignId);
    }
  });
</script>

<div class="detail-page font-sans h-screen flex flex-col">
  <!-- Top navigation bar -->
  <div class="flex flex-wrap items-center gap-2 px-4 pt-3 pb-2 shrink-0 no-print">
    <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={() => navigateTo("#/designs")}>&larr; Back to Browse</button>
    <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={openDetailPrintView} disabled={!detailItem}>Print View</button>
    <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={goToPreviousDetail} disabled={detailBrowseIndex <= 0}>&lsaquo; Prev</button>
    <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={goToNextDetail} disabled={detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1}>Next &rsaquo;</button>
    {#if detailBrowseIndex >= 0 && detailBrowseIds.length > 0}
      <span class="text-sm text-gray-500 font-medium">{detailBrowseIndex + 1} / {detailBrowseIds.length}</span>
    {/if}
  </div>

  <!-- Action notice banner -->
  {#if detailActionMessage}
    <div class="mx-4 mb-1 shrink-0 rounded border px-3 py-2 text-sm {detailActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}">
      {detailActionMessage}
    </div>
  {/if}

  <!-- Two-column body -->
  {#if detailLoading}
    <div class="flex-1 flex items-center justify-center text-gray-500 font-medium">
      <p>Loading design detail...</p>
    </div>
  {:else if detailError}
    <div class="flex-1 flex items-center justify-center">
      <p class="text-red-600">{detailError}</p>
    </div>
  {:else if !detailItem}
    <div class="flex-1 flex items-center justify-center">
      <p class="text-gray-500">No design found for id {detailDesignId}.</p>
    </div>
  {:else}
    <div class="flex-1 flex flex-col lg:flex-row min-h-0">
      <!-- LEFT COLUMN: Preview + Actions (sticky on large screens) -->
      <div class="lg:w-5/12 xl:w-2/5 lg:sticky lg:top-0 lg:self-start lg:max-h-full flex flex-col gap-3 p-4 pb-2 lg:pb-4 lg:border-r border-gray-200 overflow-y-auto">
        <!-- Filename -->
        <div class="route-card">
          <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Filename</span>
          <p class="font-medium text-gray-800 text-sm mt-0.5">{detailItem.filename || "Unknown"}</p>
        </div>

        <!-- Filepath (collapsible) -->
        <details class="text-xs">
          <summary class="cursor-pointer text-gray-500 hover:text-gray-700 font-medium select-none">Show file path</summary>
          <p class="mt-1 break-all font-mono text-xs text-gray-600 bg-gray-50 rounded border px-2.5 py-1.5">{detailItem.filepath || "Unknown"}</p>
        </details>

        <!-- Preview image -->
        {#if detailItem.image_data_url}
          <img
            src={detailItem.image_data_url}
            alt={detailItem.filename || "Design preview"}
            class="w-full rounded border border-gray-200 bg-white p-2 object-contain max-h-[28vh] lg:max-h-[20rem] shadow-sm"
          />
        {:else}
          <div class="route-card p-6 text-gray-500 text-center italic text-sm">No preview image saved yet.</div>
        {/if}

        <!-- Action buttons -->
        <div class="flex flex-wrap gap-2 pt-1">
          <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={launchDetailInEditor} disabled={detailSaving}>Open in Editor</button>
          <button class="menu-button-secondary text-xs px-2.5 py-1.5" onclick={launchDetailInExplorer} disabled={detailSaving}>Show in Explorer</button>
          <button class="menu-button-primary text-xs px-2.5 py-1.5" onclick={renderDetail3dPreview} disabled={detailSaving}>
            {detailItem.image_data_url ? (detailItem.image_type === "3d" ? "✓ 3D Preview" : "Render 3D Preview") : "Generate 3D Preview"}
          </button>
        </div>
      </div>

      <!-- RIGHT COLUMN: Editable content (scrollable) -->
      <div class="lg:w-7/12 xl:w-3/5 flex-1 overflow-y-auto p-4 pt-2 lg:pt-4 space-y-2.5">
        <!-- ============================================ -->
        <!-- ZONE A: Editable Metadata (Designer + Source) -->
        <!-- ============================================ -->
        <div class="route-card space-y-2">
          <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Designer & Source</h3>
          <div class="grid sm:grid-cols-2 gap-2.5">
            <label class="block text-sm">
              <span class="block mb-0.5 font-medium text-gray-600 text-xs">Designer</span>
              <select class="w-full border rounded px-2 py-1.5 text-sm bg-white" bind:value={detailDesignerId}>
                <option value="">None</option>
                {#each detailItem.designers || [] as designer}
                  <option value={String(designer.id)}>{designer.name}</option>
                {/each}
              </select>
            </label>
            <label class="block text-sm">
              <span class="block mb-0.5 font-medium text-gray-600 text-xs">Source</span>
              <select class="w-full border rounded px-2 py-1.5 text-sm bg-white" bind:value={detailSourceId}>
                <option value="">None</option>
                {#each detailItem.sources || [] as source}
                  <option value={String(source.id)}>{source.name}</option>
                {/each}
              </select>
            </label>
          </div>
        </div>

        <!-- ============================================ -->
        <!-- ZONE B: Read-Only Technical Facts            -->
        <!-- ============================================ -->
        <div class="route-card space-y-1.5">
          <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Technical Data</h3>
          <TechnicalDataGrid items={technicalItems} />
        </div>

        <!-- ============================================ -->
        <!-- ZONE C: Rating & Status (interactive)        -->
        <!-- ============================================ -->
        <div class="route-card flex flex-wrap items-center gap-x-3 gap-y-1.5">
          <h3 class="w-full text-xs font-semibold text-gray-500 uppercase tracking-wide mb-0.5">Rating & Status</h3>
          <!-- 5 Interactive Stars -->
          <div class="flex items-center gap-0.5" role="radiogroup" aria-label="Rating">
            {#each [1, 2, 3, 4, 5] as score}
              <button
                class="text-lg leading-none px-0.5 transition-colors duration-100
                  {score <= effectiveRating
                    ? 'text-indigo-600'
                    : 'text-gray-300 hover:text-indigo-400'}"
                onclick={() => handleStarClick(score)}
                onmouseenter={() => onStarMouseEnter(score)}
                onmouseleave={onStarMouseLeave}
                onfocus={() => onStarFocus(score)}
                onblur={onStarBlur}
                disabled={detailSaving}
                aria-label="{score} star{score !== 1 ? 's' : ''}"
                title="{score} star{score !== 1 ? 's' : ''}"
              >★</button>
            {/each}
          </div>

          <!-- Rating badge -->
          <span class="text-xs font-medium whitespace-nowrap {detailItem.rating ? 'text-indigo-700' : 'text-gray-400'}">
            {#if detailItem.rating}
              Rating: ★ {detailItem.rating} / 5
            {:else}
              Rating: Unrated
            {/if}
          </span>

          <!-- Clear rating (only shown when rated) -->
          {#if detailItem.rating}
            <button
              class="text-xs text-red-400 hover:text-red-600 hover:underline font-medium"
              onclick={() => submitDetailRating(null)}
              disabled={detailSaving}
            >Clear</button>
          {/if}

          <!-- Divider -->
          <span class="text-gray-300 select-none" aria-hidden="true">|</span>

          <!-- Stitched toggle -->
          <button
            class="text-xs px-2.5 py-1 rounded border font-semibold transition-colors
              {detailItem.is_stitched
                ? 'bg-green-50 border-green-300 text-green-700 hover:bg-green-100'
                : 'bg-white border-gray-300 text-gray-600 hover:bg-gray-100'}"
            onclick={toggleDetailStitched}
            disabled={detailSaving}
          >
            {detailItem.is_stitched ? '✓ Stitched' : 'Mark as Stitched'}
          </button>

          <!-- Verified toggle (only shown if tags exist) -->
          {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
            <button
              class="text-xs px-2.5 py-1 rounded border font-semibold transition-colors
                {detailItem.tags_checked
                  ? 'bg-green-50 border-green-300 text-green-700 hover:bg-green-100'
                  : 'bg-amber-50 border-amber-300 text-amber-700 hover:bg-amber-100'}"
              onclick={toggleDetailTagsChecked}
              disabled={detailSaving}
            >
              {detailItem.tags_checked ? '✓ Verified' : '⚠ Verify'}
            </button>
          {/if}
        </div>

        <!-- Tags -->
        <div class="route-card space-y-1.5">
          <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Tags</h3>
          {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
            <div class="flex flex-wrap gap-1.5">
              {#each detailItem.tags as tag}
                <span class="group relative inline-flex items-center gap-0.5 text-[11px] px-2 py-0.5 rounded-full font-medium {tag.tag_group === "stitching" ? "bg-blue-100 text-blue-700" : tag.tag_group === "image" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-700"}">
                  {tag.description}
                  <button
                    class="opacity-0 group-hover:opacity-100 transition-opacity ml-0.5 text-xs font-bold hover:text-red-600 rounded-full hover:bg-black/10 w-4 h-4 inline-flex items-center justify-center leading-none shrink-0"
                    onclick={(e) => handleRemoveTag(tag.id, e)}
                    disabled={detailSaving}
                    title="Remove tag"
                  >&times;</button>
                </span>
              {/each}
            </div>
          {:else}
            <p class="text-xs text-gray-400 italic">No tags assigned.</p>
          {/if}
          <button class="menu-button-primary text-xs px-2.5 py-1" onclick={openDetailTagModal} disabled={detailSaving}>Choose tags...</button>
        </div>

        <!-- Notes -->
        <div class="route-card space-y-1.5">
          <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Notes</h3>
          <textarea
            class="w-full border rounded px-2.5 py-1.5 text-sm bg-white"
            rows="2"
            bind:value={detailNotes}
            placeholder="Add notes about this design..."
          ></textarea>
          <div class="flex justify-end">
            <button class="menu-button-primary text-xs px-2.5 py-1" onclick={saveDetailMetadata} disabled={detailSaving || detailNotes === (detailItem?.notes ?? "")}>
              {detailSaving ? "Saving..." : "Save Notes"}
            </button>
          </div>
        </div>

        <!-- Projects -->
        <div class="route-card space-y-1.5">
          <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Projects</h3>
          {#if Array.isArray(detailItem.projects) && detailItem.projects.length > 0}
            <div class="space-y-1">
              {#each detailItem.projects as project}
                <div class="flex items-center justify-between border bg-white rounded px-2.5 py-1.5 text-sm">
                  <span class="font-medium text-gray-700">{project.name}</span>
                  <button class="text-red-500 hover:text-red-700 hover:underline text-xs font-semibold" onclick={() => removeDetailFromProject(project.id)} disabled={detailSaving}>Remove</button>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-xs text-gray-400 italic">Not assigned to any project.</p>
          {/if}
          {#if Array.isArray(detailItem.available_projects) && detailItem.available_projects.length > 0}
            <div class="flex gap-2 pt-0.5">
              <select class="flex-1 border rounded px-2.5 py-1.5 text-sm bg-white" bind:value={detailProjectToAdd} disabled={detailSaving}>
                {#each detailItem.available_projects as project}
                  <option value={String(project.id)}>{project.name}</option>
                {/each}
              </select>
              <button class="menu-button-primary text-xs px-2.5 py-1" onclick={addSelectedDetailProject} disabled={detailSaving || !detailProjectToAdd}>
                Add
              </button>
            </div>
          {/if}
        </div>

        <!-- Delete -->
        <div class="flex justify-end pt-0.5 pb-2">
          <button class="menu-button-secondary text-red-500 border-red-200 hover:bg-red-50 text-xs px-2.5 py-1.5" onclick={openDeleteModal} disabled={detailSaving}>Delete design</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Shared Delete Modal -->
<DeleteDesignsModal
  designIds={detailItem?.id != null ? [detailItem.id] : []}
  previewItems={detailItem ? [{
    id: detailItem.id,
    filename: detailItem.filename ?? '',
    filepath: detailItem.filepath ?? '',
    dataUrl: detailItem.image_data_url ?? null,
  }] : []}
  open={detailDeleteModalOpen}
  onClose={closeDeleteModal}
  onDeleted={handleDetailDeleteResult}
/>

<TagSelectionModal
  designId={detailItem?.id ?? 0}
  allTags={Array.isArray(detailItem?.all_tags) ? detailItem.all_tags : []}
  selectedTagIds={detailTagSelection}
  open={browseBulkModalOpen}
  onClose={closeDetailTagModal}
/>