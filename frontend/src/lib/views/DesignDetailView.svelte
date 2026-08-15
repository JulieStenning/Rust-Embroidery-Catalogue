<script>
  import { onMount, untrack } from "svelte";
  import {
    getDesignDetail,
    getDesignImageDataUrl,
    updateDesignMetadata,
    setDesignRating,
    setDesignStitched,
    setDesignVerification,
    setDesignTags,
    removeDesignTag,
    addDesignToProject,
    removeDesignFromProject,
    bulkDeleteDesigns,
    openDesignInEditor,
    openDesignInExplorer,
    renderDesign3dPreview,
    reparseDesignFile
  } from "../api/commandAdapter";
  import DeleteDesignsModal from "../components/DeleteDesignsModal.svelte";
  import TagSelectionModal from "../components/TagSelectionModal.svelte";
  import TechnicalDataGrid from "../components/TechnicalDataGrid.svelte";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";
  import { designSessionStore } from "../stores/designSessionStore.js";
  import { addToast } from "../stores/toastStore.js";
  /** @typedef {import("../types/ipc").DesignDetail} DesignDetailItem */
  /** @typedef {import("../types/ipc").DesignTagDetail} DesignTagDetail */

  let { detailDesignId, detailBrowseIds = [], detailBrowseIndex = -1, navigateTo, onDesignDeleted = () => {} } = $props();

  let detailLoading = $state(false);
  let detailError = $state("");
  let detailSaving = $state(false);
  let detailReparsing = $state(false);
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
  let previousDesignerId = $state("");
  let previousSourceId = $state("");
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
        { label: "Date Added", value: item.dateAdded || "Unknown" },
        { label: "Dimensions", value: item.widthMm != null && item.heightMm != null ? `${item.widthMm} × ${item.heightMm} mm` : "?" },
        { label: "Stitches", value: item.stitchCount ?? "?" },
        { label: "Colours", value: item.colorCount ?? "?" },
        { label: "Colour Changes", value: item.colorChangeCount ?? "?" },
      ];
    })()
  );

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
      detailDesignerId = detailItem?.designerId == null ? "" : String(detailItem.designerId);
      detailSourceId = detailItem?.sourceId == null ? "" : String(detailItem.sourceId);
      previousDesignerId = detailDesignerId;
      previousSourceId = detailSourceId;
      detailTagSelection = Array.isArray(detailItem?.tags)
        ? detailItem.tags.map((tag) => Number(tag?.id)).filter((id) => Number.isFinite(id))
        : [];
      detailProjectToAdd = "";
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

    addToast(result.message, result.persisted ? "success" : "error");
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

  /**
   * Auto-save handler for the Designer dropdown.
   * Uses detailSaving guard to prevent concurrent writes.
   * On failure, reverts the dropdown to its previous value and shows an error toast.
   */
  async function handleDesignerChange() {
    if (!detailItem?.id || detailSaving) return;
    const newValue = detailDesignerId;
    const oldValue = previousDesignerId;

    detailSaving = true;
    const result = await updateDesignMetadata(detailItem.id, {
      designer_id: newValue ? Number(newValue) : null,
      source_id: detailSourceId ? Number(detailSourceId) : null,
    });
    detailSaving = false;

    if (result.persisted) {
      previousDesignerId = newValue;
      const selectedDesigner = (detailItem.designers || []).find(
        /** @param {{id: number, name: string}} d */ (d) => d.id === Number(newValue)
      );
      designSessionStore.trackMutation(detailItem.id, {
        designer: selectedDesigner?.name || detailItem.designer,
      });
      addToast("Designer updated", "success");
    } else {
      // Revert dropdown to previous known good value
      detailDesignerId = oldValue;
      addToast(result.message || "Failed to update designer", "error");
    }
  }

  /**
   * Auto-save handler for the Source dropdown.
   * Uses detailSaving guard to prevent concurrent writes.
   * On failure, reverts the dropdown to its previous value and shows an error toast.
   */
  async function handleSourceChange() {
    if (!detailItem?.id || detailSaving) return;
    const newValue = detailSourceId;
    const oldValue = previousSourceId;

    detailSaving = true;
    const result = await updateDesignMetadata(detailItem.id, {
      designer_id: detailDesignerId ? Number(detailDesignerId) : null,
      source_id: newValue ? Number(newValue) : null,
    });
    detailSaving = false;

    if (result.persisted) {
      previousSourceId = newValue;
      const selectedSource = (detailItem.sources || []).find(
        /** @param {{id: number, name: string}} s */ (s) => s.id === Number(newValue)
      );
      designSessionStore.trackMutation(detailItem.id, {
        source: selectedSource?.name || detailItem.source,
      });
      addToast("Source updated", "success");
    } else {
      // Revert dropdown to previous known good value
      detailSourceId = oldValue;
      addToast(result.message || "Failed to update source", "error");
    }
  }

  /** @param {number | null} rating */
  async function submitDetailRating(rating) {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await setDesignRating(detailItem.id, rating);
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { rating });
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailStitched() {
    if (!detailItem?.id || detailSaving) return;

    const newStitched = !detailItem?.isStitched;
    detailSaving = true;
    const result = await setDesignStitched(detailItem.id, newStitched);
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { isStitched: newStitched });
      await refreshDetailAfterAction();
    }
  }

  async function toggleImageTagsVerified() {
    if (!detailItem?.id || detailSaving) return;

    const newValue = !detailItem?.imageTagsVerified;
    detailSaving = true;
    const result = await setDesignVerification(detailItem.id, { imageTagsVerified: newValue });
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { imageTagsVerified: newValue });
      await refreshDetailAfterAction();
    }
  }

  async function toggleStitchingTagsVerified() {
    if (!detailItem?.id || detailSaving) return;

    const newValue = !detailItem?.stitchingTagsVerified;
    detailSaving = true;
    const result = await setDesignVerification(detailItem.id, { stitchingTagsVerified: newValue });
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      designSessionStore.trackMutation(detailItem.id, { stitchingTagsVerified: newValue });
      await refreshDetailAfterAction();
    }
  }

  async function saveDetailTags() {
    if (!detailItem?.id || detailSaving) return false;

    detailSaving = true;
    // A single-design tag save marks BOTH image and stitching categories
    // verified (Rule 1).
    const result = await setDesignTags(detailItem.id, detailTagSelection, {
      imageTagsVerified: true,
      stitchingTagsVerified: true,
    });
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      // Compute tag arrays from the selected tag IDs and the all_tags lookup
      const allTags = Array.isArray(detailItem?.allTags) ? detailItem.allTags : [];
      const selectedTags = allTags.filter(
        /** @param {{id: number, description: string, tag_group: string | null}} t */ (t) =>
          detailTagSelection.includes(t.id)
      );
      const imageTags = selectedTags
        .filter(/** @param {{tag_group: string | null}} t */ (t) => t.tag_group === "image")
        .map(/** @param {{description: string}} t */ (t) => t.description);
      const stitchingTags = selectedTags
        .filter(/** @param {{tag_group: string | null}} t */ (t) => t.tag_group === "stitching")
        .map(/** @param {{description: string}} t */ (t) => t.description);
      const allTagDescriptions = selectedTags.map(/** @param {{description: string}} t */ (t) => t.description);

      designSessionStore.trackMutation(detailItem.id, {
        tags: allTagDescriptions,
        imageTags,
        stitchingTags,
        imageTagsVerified: true,
        stitchingTagsVerified: true,
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
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      // Build updated project list from existing + the newly added project
      const addedProject = (detailItem?.availableProjects || []).find(
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
    addToast(result.message, result.persisted ? "success" : "error");
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
    addToast(result.message, (!result.persisted || !result?.result?.success) ? "error" : "success");
  }

  async function launchDetailInExplorer() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await openDesignInExplorer(detailItem.id);
    detailSaving = false;
    addToast(result.message, (!result.persisted || !result?.result?.success) ? "error" : "success");
  }

  async function renderDetailPreview() {
    if (!detailItem?.id || detailSaving) return;

    const currentIs3d = detailItem.imageType === "3d";
    detailSaving = true;
    const result = await renderDesign3dPreview(detailItem.id, !currentIs3d);
    detailSaving = false;
    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted) {
      const refreshedImage = await getDesignImageDataUrl(detailItem.id);
      if (refreshedImage?.item?.data_url) {
        detailItem = {
          ...detailItem,
          imageDataUrl: refreshedImage.item.data_url,
          imageType: refreshedImage.item.image_type || detailItem.imageType,
        };
      }
      await refreshDetailAfterAction();
    }
  }

  async function recalculateFromFile() {
    if (!detailItem?.id || detailSaving || detailReparsing) return;

    detailReparsing = true;
    const result = await reparseDesignFile(detailItem.id);
    detailReparsing = false;

    addToast(result.message, result.persisted ? "success" : "error");
    if (result.persisted && result.result) {
      const r = result.result;
      detailItem = {
        ...detailItem,
        widthMm: r.widthMm,
        heightMm: r.heightMm,
        stitchCount: r.stitchCount,
        colorCount: r.colorCount,
        colorChangeCount: r.colorChangeCount,
        hoopId: r.hoopId,
        hoop: r.hoop,
      };
      designSessionStore.trackMutation(detailItem.id, {
        hoop: r.hoop,
      });
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
      addToast(result.errors?.[0] || "Could not delete design.", "error");
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
      addToast(result.message, "success");
      // A single-design tag edit marks BOTH categories verified (Rule 1).
      const setVerify = await setDesignVerification(detailItem.id, {
        imageTagsVerified: true,
        stitchingTagsVerified: true,
      });
      if (!setVerify.persisted) {
        addToast(setVerify.message || "Tag removed but verification update failed.", "error");
      }
      // Track mutation for browse card sync; also patch both verified flags.
      const updatedTags = (detailItem.tags || []).map(t => t.description);
      designSessionStore.trackMutation(detailItem.id, {
        tags: updatedTags,
        imageTags: (detailItem.tags || []).filter(t => t.tag_group === 'image').map(t => t.description),
        stitchingTags: (detailItem.tags || []).filter(t => t.tag_group === 'stitching').map(t => t.description),
        imageTagsVerified: true,
        stitchingTagsVerified: true,
      });
    } else {
      addToast(result.message, "error");
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
  <div class="flex flex-wrap items-center gap-1.5 px-4 pt-3 pb-2 shrink-0 no-print">
    <button class="menu-button-primary text-xs px-2.5 py-1" onclick={() => navigateTo("#/designs")}>&larr; Back to Browse</button>
    <span class="flex-1" aria-hidden="true"></span>
    <button class="menu-button-nav" onclick={goToPreviousDetail} disabled={detailBrowseIndex <= 0} title="Previous design">&lsaquo; Prev</button>
    {#if detailBrowseIndex >= 0 && detailBrowseIds.length > 0}
      <span class="text-sm text-gray-500 font-medium tabular-nums mx-1">{detailBrowseIndex + 1} / {detailBrowseIds.length}</span>
    {/if}
    <button class="menu-button-nav" onclick={goToNextDetail} disabled={detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1} title="Next design">Next &rsaquo;</button>
    <span class="text-gray-300 select-none mx-0.5" aria-hidden="true">|</span>
    <button class="menu-button-nav" onclick={openDetailPrintView} disabled={!detailItem} title="Print view">Print</button>
  </div>

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
        {#if detailItem.imageDataUrl}
          <img
            src={detailItem.imageDataUrl}
            alt={detailItem.filename || "Design preview"}
            class="w-full rounded border border-gray-200 bg-white p-2 object-contain max-h-[28vh] lg:max-h-[20rem] shadow-sm"
          />
        {:else}
          <div class="route-card p-6 text-gray-500 text-center italic text-sm">No preview image saved yet.</div>
        {/if}

        <!-- Action buttons -->
        <div class="flex flex-wrap gap-2 pt-1">
          <button class="menu-button-ghost" onclick={launchDetailInEditor} disabled={detailSaving}><span aria-hidden="true" class="text-[10px]">&#9998;</span> Open in Editor</button>
          <button class="menu-button-ghost" onclick={launchDetailInExplorer} disabled={detailSaving}><span aria-hidden="true" class="text-[10px]">&#128193;</span> Show in Explorer</button>
          <button class="menu-button-primary text-xs px-2.5 py-1.5" onclick={renderDetailPreview} disabled={detailSaving}>
            {detailItem.imageType === "3d" ? "Generate 2D Preview" : "Generate 3D Preview"}
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
              <select class="w-full border rounded px-2 py-1.5 text-sm bg-white" bind:value={detailDesignerId} onchange={handleDesignerChange}>
                <option value="">None</option>
                {#each detailItem.designers || [] as designer}
                  <option value={String(designer.id)}>{designer.name}</option>
                {/each}
              </select>
            </label>
            <label class="block text-sm">
              <span class="block mb-0.5 font-medium text-gray-600 text-xs">Source</span>
              <select class="w-full border rounded px-2 py-1.5 text-sm bg-white" bind:value={detailSourceId} onchange={handleSourceChange}>
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
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Technical Data</h3>
            <button
              class="menu-button-primary text-xs px-2.5 py-1"
              onclick={recalculateFromFile}
              disabled={detailSaving || detailReparsing}
              title="Re-read the file on disk and recalculate technical metadata"
            >
              {#if detailReparsing}
                <span class="inline-block animate-spin mr-1" aria-hidden="true">&#9696;</span>
                Recalculating…
              {:else}
                <span aria-hidden="true" class="mr-1">&#10227;</span> Recalculate From File
              {/if}
            </button>
          </div>
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
            class="menu-button-toggle {detailItem.isStitched
              ? 'bg-green-50 border-green-300 text-green-700 hover:bg-green-100 hover:border-green-400'
              : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50 hover:border-gray-400'}"
            onclick={toggleDetailStitched}
            disabled={detailSaving}
            title={detailItem.isStitched ? 'Mark as not stitched' : 'Mark as stitched'}
          >
            {#if detailItem.isStitched}
              <span aria-hidden="true">&#10003;</span> Stitched
            {:else}
              Mark as Stitched
            {/if}
          </button>

          <!-- Image verification toggle (only shown if tags exist) -->
          {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
            <button
              class="menu-button-toggle {detailItem.imageTagsVerified
                ? 'bg-green-50 border-green-300 text-green-700 hover:bg-green-100 hover:border-green-400'
                : 'bg-amber-50 border-amber-300 text-amber-700 hover:bg-amber-100 hover:border-amber-400'}"
              onclick={toggleImageTagsVerified}
              disabled={detailSaving}
              title={detailItem.imageTagsVerified ? 'Mark image tags as unverified' : 'Mark image tags as verified'}
            >
              {#if detailItem.imageTagsVerified}
                <span aria-hidden="true">&#10003;</span> Image Verified
              {:else}
                <span aria-hidden="true">&#9888;</span> Image Unverified
              {/if}
            </button>
          {/if}

          <!-- Stitching verification toggle (only shown if tags exist) -->
          {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
            <button
              class="menu-button-toggle {detailItem.stitchingTagsVerified
                ? 'bg-green-50 border-green-300 text-green-700 hover:bg-green-100 hover:border-green-400'
                : 'bg-amber-50 border-amber-300 text-amber-700 hover:bg-amber-100 hover:border-amber-400'}"
              onclick={toggleStitchingTagsVerified}
              disabled={detailSaving}
              title={detailItem.stitchingTagsVerified ? 'Mark stitching tags as unverified' : 'Mark stitching tags as verified'}
            >
              {#if detailItem.stitchingTagsVerified}
                <span aria-hidden="true">&#10003;</span> Stitching Verified
              {:else}
                <span aria-hidden="true">&#9888;</span> Stitching Unverified
              {/if}
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
            <div class="flex flex-wrap gap-1.5">
              {#each detailItem.projects as project}
                <span class="group relative inline-flex items-center gap-0.5 text-[11px] px-2 py-0.5 rounded-full font-medium bg-amber-100 text-amber-700">
                  <span aria-hidden="true">&#128193;</span>
                  {project.name}
                  <button
                    class="opacity-0 group-hover:opacity-100 transition-opacity ml-0.5 text-xs font-bold hover:text-red-600 rounded-full hover:bg-black/10 w-4 h-4 inline-flex items-center justify-center leading-none shrink-0"
                    onclick={() => removeDetailFromProject(project.id)}
                    disabled={detailSaving}
                    title="Remove from project"
                  >&times;</button>
                </span>
              {/each}
            </div>
          {:else}
            <p class="text-xs text-gray-400 italic">Not assigned to any projects.</p>
          {/if}

          {#if Array.isArray(detailItem.availableProjects) && detailItem.availableProjects.length > 0}
            <div class="flex gap-2 pt-0.5">
              <select class="flex-1 border rounded px-2.5 py-1.5 text-sm bg-white" bind:value={detailProjectToAdd} disabled={detailSaving}>
                <option value="">-- Select project to add --</option>
                {#each detailItem.availableProjects as project}
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
    dataUrl: detailItem.imageDataUrl ?? null,
  }] : []}
  open={detailDeleteModalOpen}
  onClose={closeDeleteModal}
  onDeleted={handleDetailDeleteResult}
/>

<TagSelectionModal
  designId={detailItem?.id ?? 0}
  allTags={Array.isArray(detailItem?.allTags) ? detailItem.allTags : []}
  selectedTagIds={detailTagSelection}
  open={browseBulkModalOpen}
  onClose={closeDetailTagModal}
/>