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
    addDesignToProject,
    removeDesignFromProject,
    deleteDesign,
    openDesignInEditor,
    openDesignInExplorer,
    renderDesign3dPreview
  } from "../api/commandAdapter.js";
  import { splitTagsByGroup } from "../utils/tagHelpers.js";

  let { detailDesignId, detailBrowseIds = [], detailBrowseIndex = -1, navigateTo } = $props();

  let detailLoading = $state(false);
  let detailError = $state("");
  let detailSaving = $state(false);
  let detailActionMessage = $state("");
  let detailActionIsError = $state(false);

  let detailItem = $state(null);
  let detailSource = $state("mock");
  let detailNotes = $state("");
  let detailDesignerId = $state("");
  let detailSourceId = $state("");
  let detailProjectToAdd = $state("");
  let detailTagSelection = $state([]);

  let deleteModalStep = $state(null); // null | "choose" | "confirm-file-delete"
  let browseBulkModalOpen = $state(false);

  function setDetailActionNotice(message, isError = false) {
    detailActionMessage = message;
    detailActionIsError = isError;
  }

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
      await refreshDetailAfterAction();
    }
  }

  async function submitDetailRating(rating) {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await setDesignRating(detailItem.id, rating);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailStitched() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await setDesignStitched(detailItem.id, !detailItem?.is_stitched);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function toggleDetailTagsChecked() {
    if (!detailItem?.id || detailSaving) return;

    detailSaving = true;
    const result = await setDesignTagsChecked(detailItem.id, !detailItem?.tags_checked);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
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
      await refreshDetailAfterAction();
      return true;
    }
    return false;
  }

  async function addDetailToProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) return;

    detailSaving = true;
    const result = await addDesignToProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshDetailAfterAction();
    }
  }

  async function addSelectedDetailProject() {
    if (!detailProjectToAdd) return;
    await addDetailToProject(Number(detailProjectToAdd));
  }

  async function removeDetailFromProject(projectId) {
    if (!detailItem?.id || !projectId || detailSaving) return;

    detailSaving = true;
    const result = await removeDesignFromProject(detailItem.id, projectId);
    detailSaving = false;
    setDetailActionNotice(result.message, !result.persisted);
    if (result.persisted) {
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
    deleteModalStep = "choose";
  }

  function closeDeleteModal(event) {
    event?.preventDefault?.();
    event?.stopPropagation?.();
    if (detailSaving) return;
    deleteModalStep = null;
  }

  function handleDeleteWithFileChoice() {
    if (detailSaving) return;
    deleteModalStep = "confirm-file-delete";
  }

  function handleBackToFirstDeleteModal() {
    if (detailSaving) return;
    deleteModalStep = "choose";
  }

  async function deleteDetailDesign(deleteFile) {
    if (!detailItem?.id || detailSaving) return;

    const designId = detailItem.id;
    detailSaving = true;
    const result = await deleteDesign(designId, deleteFile);
    detailSaving = false;

    if (result.persisted) {
      deleteModalStep = null;
      navigateTo("#/designs");
    } else {
      deleteModalStep = null;
      setDetailActionNotice(result.error || "Could not delete design.", true);
    }
  }

  function handleDeleteCatalogueOnly() {
    deleteDetailDesign(false);
  }

  function handleDeleteWithFile() {
    deleteDetailDesign(true);
  }

  function openDetailTagModal() {
    if (!detailItem?.id || detailSaving) return;
    browseBulkModalOpen = true;
  }

  function closeDetailTagModal() {
    browseBulkModalOpen = false;
  }

  function tagChooserSelectionIncludes(tagId) {
    return detailTagSelection.includes(Number(tagId));
  }

  function toggleTagChooserSelection(tagId, checked) {
    const id = Number(tagId);
    if (!Number.isFinite(id)) return;
    if (checked) {
      detailTagSelection = Array.from(new Set([...detailTagSelection, id]));
    } else {
      detailTagSelection = detailTagSelection.filter((value) => value !== id);
    }
  }

  async function applySharedTagChooser() {
    const saved = await saveDetailTags();
    if (saved) {
      closeDetailTagModal();
    }
  }

  function portalToBody(node) {
    if (typeof document === "undefined") return {};
    const host = document.body;
    const parent = node.parentNode;
    const marker = document.createComment("detail-modal-portal");
    if (parent) parent.insertBefore(marker, node);
    host.appendChild(node);
    return {
      destroy() {
        if (node.parentNode === host) host.removeChild(node);
        if (marker.parentNode) marker.parentNode.removeChild(marker);
      },
    };
  }

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

<div class="space-y-4 font-sans">
  <div class="flex flex-wrap gap-2">
    <button class="menu-button-secondary" onclick={() => navigateTo("#/designs")}>Back to Browse</button>
    <button class="menu-button-secondary" onclick={openDetailPrintView} disabled={!detailItem}>Print View</button>
    <button class="menu-button-secondary" onclick={goToPreviousDetail} disabled={detailBrowseIndex <= 0}>‹ Prev</button>
    <button class="menu-button-secondary" onclick={goToNextDetail} disabled={detailBrowseIndex < 0 || detailBrowseIndex >= detailBrowseIds.length - 1}>Next ›</button>
    {#if detailBrowseIndex >= 0 && detailBrowseIds.length > 0}
      <span class="text-sm text-gray-500 self-center">{detailBrowseIndex + 1} / {detailBrowseIds.length}</span>
    {/if}
  </div>

  {#if detailActionMessage}
    <div class={`rounded border px-3 py-2 text-sm ${detailActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
      {detailActionMessage}
    </div>
  {/if}

  <div class="route-panel bg-white rounded shadow p-6">
    {#if detailLoading}
      <p>Loading design detail...</p>
    {:else if detailError}
      <p class="text-red-600">{detailError}</p>
    {:else if !detailItem}
      <p>No design found for id {detailDesignId}.</p>
    {:else}
      <div class="grid lg:grid-cols-2 gap-6 mt-3">
        <div class="space-y-4">
          <div class="route-card bg-gray-50 rounded border p-3">
            <strong>Filename:</strong> <span class="font-medium text-gray-800">{detailItem.filename || "Unknown"}</span>
          </div>
          <div class="route-card bg-gray-50 rounded border p-3 break-all font-mono text-xs">
            <strong>Path:</strong> {detailItem.filepath || "Unknown"}
          </div>
          {#if detailItem.image_data_url}
            <img
              src={detailItem.image_data_url}
              alt={detailItem.filename || "Design preview"}
              class="w-full rounded border border-gray-200 bg-white p-2 max-h-[24rem] object-contain shadow-sm"
            />
          {:else}
            <div class="route-card bg-gray-50 rounded border p-6 text-gray-500 text-center italic">No preview image saved yet.</div>
          {/if}

          <div class="flex flex-wrap gap-2 pt-2">
            <button class="menu-button-secondary font-medium" onclick={launchDetailInEditor} disabled={detailSaving}>Open in Editor</button>
            <button class="menu-button-secondary font-medium" onclick={launchDetailInExplorer} disabled={detailSaving}>Show in Explorer</button>
            <button class="menu-button-primary font-medium" onclick={renderDetail3dPreview} disabled={detailSaving}>
              {detailItem.image_data_url ? (detailItem.image_type === "3d" ? "✓ 3D Preview" : "Render 3D Preview") : "Generate 3D Preview"}
            </button>
          </div>
        </div>

        <div class="space-y-4">
          <div class="grid sm:grid-cols-2 gap-3">
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Designer:</strong> {detailItem.designer || "Unknown"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Source:</strong> {detailItem.source || "Unknown"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Hoop:</strong> {detailItem.hoop || "Unknown"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Date added:</strong> {detailItem.date_added || "Unknown"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Dimensions:</strong> {detailItem.width_mm ?? "?"} x {detailItem.height_mm ?? "?"} mm</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Stitches:</strong> {detailItem.stitch_count ?? "?"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Colours:</strong> {detailItem.color_count ?? "?"}</div>
            <div class="route-card bg-gray-50 rounded border p-3 text-sm"><strong>Colour changes:</strong> {detailItem.color_change_count ?? "?"}</div>
          </div>

          {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
            <div class="flex flex-wrap gap-2 items-center pt-2">
              {#each detailItem.tags as tag}
                <span class={`text-xs px-2.5 py-1 rounded-full font-medium ${tag.tag_group === "stitching" ? "bg-blue-100 text-blue-700" : tag.tag_group === "image" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-700"}`}>
                  {tag.description}
                </span>
              {/each}
            </div>
          {/if}

          <div class="route-panel bg-gray-50 rounded border p-4 space-y-3">
            <p class="font-semibold text-gray-800 text-sm">Rating and stitched status</p>
            <div class="flex flex-wrap gap-1.5 items-center">
              <span class="text-sm text-gray-600 mr-2">Current rating: {detailItem.rating ?? "None"}</span>
              {#each [1, 2, 3, 4, 5] as score}
                <button
                  class="px-2.5 py-1 rounded border hover:bg-gray-100 text-xs font-semibold"
                  onclick={() => submitDetailRating(score)}
                  disabled={detailSaving}
                >
                  {score}★
                </button>
              {/each}
              {#if detailItem.rating}
                <button class="px-2.5 py-1 rounded border text-red-500 hover:bg-red-50 text-xs font-semibold" onclick={() => submitDetailRating(null)} disabled={detailSaving}>Clear rating</button>
              {/if}
            </div>
            <div class="flex flex-wrap gap-2 pt-1">
              <button class="menu-button-secondary text-xs" onclick={toggleDetailStitched} disabled={detailSaving}>
                {detailItem.is_stitched ? "✓ Mark as Not Stitched" : "Mark as Stitched"}
              </button>
              {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                <button class="menu-button-secondary text-xs" onclick={toggleDetailTagsChecked} disabled={detailSaving}>
                  {detailItem.tags_checked ? "✓ Verified" : "⚠ Verify"}
                </button>
              {/if}
            </div>
          </div>

          <form
            class="route-panel bg-gray-50 rounded border p-4 space-y-3"
            onsubmit={(event) => {
              event.preventDefault();
              saveDetailMetadata();
            }}
          >
            <p class="font-semibold text-gray-800 text-sm">Metadata</p>
            <label class="block text-sm text-gray-700">
              <span class="block mb-1 font-medium">Notes</span>
              <textarea class="w-full border rounded px-2.5 py-1.5 text-sm" rows="3" bind:value={detailNotes}></textarea>
            </label>
            <div class="grid sm:grid-cols-2 gap-2">
              <label class="text-sm text-gray-700">
                <span class="block mb-1 font-medium">Designer</span>
                <select class="w-full border rounded px-2.5 py-1.5 text-sm bg-white" bind:value={detailDesignerId}>
                  <option value="">None</option>
                  {#each detailItem.designers || [] as designer}
                    <option value={String(designer.id)}>{designer.name}</option>
                  {/each}
                </select>
              </label>
              <label class="text-sm text-gray-700">
                <span class="block mb-1 font-medium">Source</span>
                <select class="w-full border rounded px-2.5 py-1.5 text-sm bg-white" bind:value={detailSourceId}>
                  <option value="">None</option>
                  {#each detailItem.sources || [] as source}
                    <option value={String(source.id)}>{source.name}</option>
                  {/each}
                </select>
              </label>
            </div>
            <button type="submit" class="menu-button-primary text-xs" disabled={detailSaving}>Save metadata</button>
          </form>

          <div class="route-panel bg-gray-50 rounded border p-4 space-y-3">
            <p class="font-semibold text-gray-800 text-sm">Projects</p>
            {#if Array.isArray(detailItem.projects) && detailItem.projects.length > 0}
              <div class="space-y-1.5">
                {#each detailItem.projects as project}
                  <div class="flex items-center justify-between border bg-white rounded px-2.5 py-1.5 text-sm">
                    <span class="font-medium text-gray-700">{project.name}</span>
                    <button class="text-red-500 hover:text-red-700 hover:underline text-xs font-semibold" onclick={() => removeDetailFromProject(project.id)} disabled={detailSaving}>Remove</button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="text-xs text-gray-500 italic">Not assigned to any project.</p>
            {/if}

            {#if Array.isArray(detailItem.available_projects) && detailItem.available_projects.length > 0}
              <div class="flex flex-col sm:flex-row gap-2 pt-1">
                <select class="w-full sm:flex-1 border rounded px-2.5 py-1.5 text-sm bg-white" bind:value={detailProjectToAdd} disabled={detailSaving}>
                  {#each detailItem.available_projects as project}
                    <option value={String(project.id)}>{project.name}</option>
                  {/each}
                </select>
                <button class="menu-button-primary text-xs" onclick={addSelectedDetailProject} disabled={detailSaving || !detailProjectToAdd}>
                  Add to Project
                </button>
              </div>
            {/if}
          </div>

          <details class="route-panel bg-gray-50 rounded border p-4 space-y-3" open>
            <summary class="font-semibold cursor-pointer text-gray-800 text-sm select-none">
              Tags
              {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
                <span class="ml-1.5 text-xs text-gray-500">({detailItem.tags.length} assigned: {detailItem.tags.map((tag) => tag.description).join(", ")})</span>
              {:else}
                <span class="ml-1.5 text-xs text-gray-400">(none assigned)</span>
              {/if}
              <span class={`ml-1.5 text-xs font-semibold ${detailItem.tags_checked ? "text-green-600" : "text-orange-500"}`}>
                {detailItem.tags_checked ? "✓ Tags checked" : "⚠ Tags not verified"}
              </span>
            </summary>
            <div class="space-y-3 pt-2">
              <div class="flex items-center gap-2">
                <button class="menu-button-primary text-xs" onclick={openDetailTagModal} disabled={detailSaving}>Choose tags...</button>
                <button class="menu-button-secondary text-xs" onclick={saveDetailTags} disabled={detailSaving}>Save tags</button>
                <span class="text-[11px] text-gray-500">Saving tags marks this design as verified.</span>
              </div>
            </div>
          </details>

          <div class="flex justify-end pt-2">
            <button class="menu-button-secondary text-red-500 border-red-200 hover:bg-red-50 text-xs" onclick={openDeleteModal} disabled={detailSaving}>Delete design</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

{#if deleteModalStep === "choose"}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="delete-design-choice-title"
    aria-describedby="delete-design-choice-description"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Cancel deleting design"
      onmousedown={closeDeleteModal}
      onclick={closeDeleteModal}
    ></button>

    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
    >
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
        <h2 id="delete-design-choice-title" class="text-lg font-semibold" style="margin:0;">
          Delete design?
        </h2>
      </div>

      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p id="delete-design-choice-description" class="text-sm" style="margin:0;">
          Do you also want to delete the design file from your computer?
        </p>
        {#if detailItem?.filepath}
          <p class="text-sm text-gray-500" style="margin:0.75rem 0 0 0;word-break:break-all;">
            {detailItem.filepath}
          </p>
        {/if}
      </div>

      <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
        <button type="button" class="menu-button-secondary" onclick={closeDeleteModal} disabled={detailSaving}>
          Cancel
        </button>
        <button type="button" class="menu-button-secondary" onclick={handleDeleteCatalogueOnly} disabled={detailSaving}>
          {detailSaving ? "Deleting..." : "No"}
        </button>
        <button type="button" class="menu-button-primary" onclick={handleDeleteWithFileChoice} disabled={detailSaving}>
          Yes
        </button>
      </div>
    </div>
  </div>
{:else if deleteModalStep === "confirm-file-delete"}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="delete-design-file-title"
    aria-describedby="delete-design-file-description"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Cancel deleting design"
      onmousedown={closeDeleteModal}
      onclick={closeDeleteModal}
    ></button>

    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;width:min(40rem, calc(100vw - 2rem));"
    >
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
        <h2 id="delete-design-file-title" class="text-lg font-semibold" style="margin:0;">
          Delete the design file?
        </h2>
      </div>

      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p id="delete-design-file-description" class="text-sm" style="margin:0;">
          Do you really want to delete the file? This cannot be undone.
        </p>
      </div>

      <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
        <button type="button" class="menu-button-secondary" onclick={handleBackToFirstDeleteModal} disabled={detailSaving}>
          No
        </button>
        <button type="button" class="menu-button-primary" onclick={handleDeleteWithFile} disabled={detailSaving}>
          {detailSaving ? "Deleting..." : "Yes"}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if browseBulkModalOpen}
  {@const tagOptionsForChooser = Array.isArray(detailItem?.all_tags) ? detailItem.all_tags : []}
  {@const groupedTagOptions = splitTagsByGroup(tagOptionsForChooser)}
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    style="position:fixed;left:0;right:0;top:0;bottom:0;display:flex;align-items:center;justify-content:center;z-index:2147483647;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="bulk-tag-title"
  >
    <button
      type="button"
      style="position:absolute;inset:0;background:rgba(0,0,0,0.6);z-index:0;"
      aria-label="Close tag chooser"
      onclick={closeDetailTagModal}
    ></button>
    <div
      class="tag-chooser-dialog"
      style="position:relative;display:flex;flex-direction:column;max-height:88vh;z-index:1;"
    >
      <div class="tag-chooser-header" style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
        <h2 id="bulk-tag-title" class="text-lg font-semibold" style="margin:0;">
          Choose tags for this design
        </h2>
      </div>
      <div class="tag-chooser-body" style="overflow-y:auto;flex:1;">
        <p class="text-sm font-medium" style="margin:0 0 0.75rem 0;">
          {detailItem?.filename || "Current design"}
        </p>

        <div class="tag-chooser-sections">
          {#if groupedTagOptions.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image font-semibold">Image tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.image as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.stitching.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-stitching font-semibold">Stitching tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.stitching as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-unclassified font-semibold">Unclassified tags</p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.unclassified as tagOption}
                  <label class="tag-chooser-option">
                    <input
                      type="checkbox"
                      checked={tagChooserSelectionIncludes(tagOption.id)}
                      onchange={(event) => toggleTagChooserSelection(tagOption.id, event.currentTarget.checked)}
                    />
                    <span>{tagOption.description}</span>
                  </label>
                {/each}
              </div>
            </section>
          {/if}
        </div>
      </div>
      <div class="tag-chooser-footer" style="display:flex;align-items:center;gap:0.75rem;justify-content:flex-end;">
        <button type="button" class="menu-button-secondary" onclick={closeDetailTagModal}>Cancel</button>
        <button type="button" class="menu-button-primary" onclick={applySharedTagChooser} disabled={detailSaving}>
          Save tags
        </button>
      </div>
    </div>
  </div>
{/if}
