<script>
  import { onMount, untrack } from "svelte";
  import { getDesignDetail } from "../api/commandAdapter.js";

  let { printDesignId, navigateTo } = $props();

  let detailLoading = $state(false);
  let detailError = $state("");
  /** @type {DesignItem | null} */
  let detailItem = $state(null);
  let detailSource = $state("mock");

  /**
   * @typedef {Object} DesignItem
   * @property {string} filename
   * @property {string} [image_data_url]
   * @property {string} [filepath]
   * @property {string} [designer]
   * @property {string} [source]
   * @property {string} [hoop]
   * @property {number} [width_mm]
   * @property {number} [height_mm]
   * @property {number} [stitch_count]
   * @property {number} [color_count]
   * @property {number} [color_change_count]
   * @property {string} [date_added]
   * @property {number} [rating]
   * @property {boolean} [is_stitched]
   * @property {string} [notes]
   * @property {Array<{description: string}>} [tags]
   */

  /** @param {number | null} designId */
  async function loadDesignDetail(designId) {
    if (designId == null) return;

    detailLoading = true;
    detailError = "";

    try {
      const result = await getDesignDetail(designId);
      if (designId !== printDesignId) return;

      detailItem = result.item || null;
      detailSource = result.source || "mock";
      if (!detailItem && result?.error) {
        detailError = `Could not load design detail: ${result.error}`;
      }
    } catch (error) {
      detailError = `Could not load design detail: ${error}`;
      detailItem = null;
      detailSource = "mock";
    } finally {
      detailLoading = false;
    }
  }

  function printCurrentView() {
    window.print();
  }

  /** @param {number | string | undefined | null} rating */
  function ratingToStars(rating) {
    const numeric = Number(rating);
    if (!Number.isFinite(numeric) || numeric <= 0) return "";
    const clamped = Math.min(5, Math.max(0, numeric));
    return `${"★".repeat(clamped)}${"☆".repeat(5 - clamped)}`;
  }

  $effect(() => {
    if (printDesignId !== null) {
      untrack(() => {
        loadDesignDetail(printDesignId);
      });
    }
  });

  onMount(() => {
    if (printDesignId !== null) {
      loadDesignDetail(printDesignId);
    }
  });
</script>

<div class="space-y-3 font-sans">
  <div class="flex flex-wrap gap-2 no-print">
    <button class="menu-button-secondary font-medium" onclick={() => navigateTo(`#/designs/${printDesignId}`)}>Back to Detail</button>
    <button class="menu-button-primary font-medium" onclick={printCurrentView}>Print</button>
  </div>

  <div class="route-panel print:p-0 print:shadow-none print:border-none bg-white rounded shadow p-6">
    {#if detailLoading}
      <p>Loading printable design detail...</p>
    {:else if detailError}
      <p class="text-red-600">{detailError}</p>
    {:else if !detailItem}
      <p>No design found for id {printDesignId}.</p>
    {:else}
      <div class="space-y-4">
        <h2 class="text-2xl font-bold text-gray-800">{detailItem.filename}</h2>
        {#if detailItem.image_data_url}
          <img src={detailItem.image_data_url} alt={detailItem.filename} class="w-full max-h-[32rem] object-contain border rounded p-2 bg-gray-50 shadow-sm" />
        {/if}
        <div class="grid sm:grid-cols-2 gap-3 text-sm">
          <div class="p-2 bg-gray-50 rounded border"><strong>File:</strong> <span class="break-all font-mono text-xs">{detailItem.filepath || "Unknown"}</span></div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Designer:</strong> {detailItem.designer || "Unknown"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Source:</strong> {detailItem.source || "Unknown"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Hoop:</strong> {detailItem.hoop || "Unknown"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Dimensions:</strong> {detailItem.width_mm ?? "?"} x {detailItem.height_mm ?? "?"} mm</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Stitches:</strong> {detailItem.stitch_count ?? "?"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Colours:</strong> {detailItem.color_count ?? "?"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Colour changes:</strong> {detailItem.color_change_count ?? "?"}</div>
          <div class="p-2 bg-gray-50 rounded border"><strong>Added:</strong> {detailItem.date_added || "Unknown"}</div>
        </div>
        {#if detailItem.rating}
          <div class="p-2 bg-gray-50 rounded border text-sm"><strong>Rating:</strong> <span class="text-yellow-500 font-bold">{ratingToStars(detailItem.rating)}</span></div>
        {/if}
        {#if detailItem.is_stitched}
          <div class="p-2 bg-gray-50 rounded border text-sm"><strong>Stitched:</strong> Yes</div>
        {/if}
        {#if detailItem.notes}
          <div class="p-4 bg-gray-50 rounded border">
            <p class="font-semibold text-sm text-gray-800 mb-1">Notes</p>
            <p class="text-sm text-gray-700 whitespace-pre-wrap">{detailItem.notes}</p>
          </div>
        {/if}
        {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
          <div class="p-4 bg-gray-50 rounded border">
            <p class="font-semibold text-sm text-gray-800 mb-1">Tags</p>
            <p class="text-sm text-gray-700">{detailItem.tags.map((/** @type {{description: string}} */ tag) => tag.description).join(", ")}</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
