<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { onMount, untrack } from "svelte";
  import { getDesignDetail } from "../api/commandAdapter";

  /** @typedef {import("../types/ipc").DesignDetail} DesignItem */

  let { printDesignId, navigateTo } = $props();

  let detailLoading = $state(false);
  let detailError = $state("");
  /** @type {DesignItem | null} */
  let detailItem = $state(null);

  /** @param {number | null} designId */
  async function loadDesignDetail(designId) {
    if (designId == null) return;

    detailLoading = true;
    detailError = "";

    try {
      const result = await getDesignDetail(designId);
      if (designId !== printDesignId) return;

      detailItem = result.item || null;
      if (!detailItem && result?.error) {
        detailError = `Could not load design detail: ${result.error}`;
      }
    } catch (error) {
      detailError = `Could not load design detail: ${error}`;
      detailItem = null;
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
    <button
      class="menu-button-secondary font-medium"
      onclick={() => navigateTo(`#/designs/${printDesignId}`)}>Back to Detail</button
    >
    <button class="menu-button-primary font-medium" onclick={printCurrentView}>Print</button>
  </div>

  <div
    class="route-panel print:p-0 print:shadow-none print:border-none bg-[var(--surface-card)] rounded shadow p-6 border border-[var(--border-default)]"
  >
    {#if detailLoading}
      <p class="text-[var(--text-muted)]">Loading printable design detail...</p>
    {:else if detailError}
      <p class="text-[var(--notice-error-text)]">{detailError}</p>
    {:else if !detailItem}
      <p class="text-[var(--text-muted)]">No design found for id {printDesignId}.</p>
    {:else}
      <div class="space-y-4 text-[var(--text-primary)]">
        <h2 class="text-2xl font-bold text-[var(--text-primary)]">{detailItem.filename}</h2>
        {#if detailItem.imageDataUrl}
          <img
            src={detailItem.imageDataUrl}
            alt={detailItem.filename}
            class="w-full max-h-[32rem] object-contain border border-[var(--border-default)] rounded p-2 bg-[var(--surface-preview)] shadow-sm"
          />
        {/if}
        <div class="grid sm:grid-cols-2 gap-3 text-sm">
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>File:</strong>
            <span class="break-all font-mono text-xs text-[var(--text-secondary)]"
              >{detailItem.filepath || "Unknown"}</span
            >
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Designer:</strong>
            {detailItem.designer || "Unknown"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Source:</strong>
            {detailItem.source || "Unknown"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Hoop:</strong>
            {detailItem.hoop || "Unknown"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Dimensions:</strong>
            {detailItem.widthMm ?? "?"} x {detailItem.heightMm ?? "?"} mm
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Stitches:</strong>
            {detailItem.stitchCount ?? "?"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Colours:</strong>
            {detailItem.colorCount ?? "?"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Colour changes:</strong>
            {detailItem.colorChangeCount ?? "?"}
          </div>
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <strong>Added:</strong>
            {detailItem.dateAdded || "Unknown"}
          </div>
        </div>
        {#if detailItem.rating}
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)] text-sm"
          >
            <strong>Rating:</strong>
            <span class="text-rating-star font-bold">{ratingToStars(detailItem.rating)}</span>
          </div>
        {/if}
        {#if detailItem.isStitched}
          <div
            class="p-2 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)] text-sm"
          >
            <strong>Stitched:</strong> Yes
          </div>
        {/if}
        {#if detailItem.notes}
          <div
            class="p-4 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <p class="font-semibold text-sm text-[var(--text-primary)] mb-1">Notes</p>
            <p class="text-sm text-[var(--text-secondary)] whitespace-pre-wrap">
              {detailItem.notes}
            </p>
          </div>
        {/if}
        {#if Array.isArray(detailItem.tags) && detailItem.tags.length > 0}
          <div
            class="p-4 bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)]"
          >
            <p class="font-semibold text-sm text-[var(--text-primary)] mb-1">Tags</p>
            <p class="text-sm text-[var(--text-secondary)]">
              {detailItem.tags
                .map((/** @type {{description: string}} */ tag) => tag.description)
                .join(", ")}
            </p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
