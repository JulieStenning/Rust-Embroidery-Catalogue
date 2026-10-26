<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  import { HOOP_UNKNOWN_FILTER } from "../../utils/hoopConstants.js";

  /** @typedef {import("../../types/ipc").BrowseFilterState} BrowseFilterState */
  /** @typedef {import("../../types/ipc").BrowseActiveFilterItem} BrowseActiveFilterItem */
  /** @typedef {import("../../types/ipc").BrowseActiveFilterChipsProps} BrowseActiveFilterChipsProps */

  /** @type {BrowseActiveFilterChipsProps} */
  let { browseFilters, onUpdateFilter, onToggleFilter, onClearFilters } = $props();

  let activeChips = $derived(
    (() => {
      /** @type {BrowseActiveFilterItem[]} */
      const chips = [];

      // Designers
      if (Array.isArray(browseFilters.designerFilters)) {
        for (const designer of browseFilters.designerFilters) {
          chips.push({
            id: `designer-${designer}`,
            label: `Designer: ${designer}`,
            onRemove: () => onToggleFilter("designerFilters", designer),
          });
        }
      }

      // Image Tags
      if (Array.isArray(browseFilters.imageTagFilters)) {
        for (const tag of browseFilters.imageTagFilters) {
          chips.push({
            id: `image-tag-${tag}`,
            label: `Image tag: ${tag}`,
            onRemove: () => onToggleFilter("imageTagFilters", tag),
          });
        }
      }

      // Stitching Tags
      if (Array.isArray(browseFilters.stitchingTagFilters)) {
        for (const tag of browseFilters.stitchingTagFilters) {
          chips.push({
            id: `stitching-tag-${tag}`,
            label: `Stitching tag: ${tag}`,
            onRemove: () => onToggleFilter("stitchingTagFilters", tag),
          });
        }
      }

      // Sources
      if (Array.isArray(browseFilters.sourceFilters)) {
        for (const source of browseFilters.sourceFilters) {
          chips.push({
            id: `source-${source}`,
            label: `Source: ${source}`,
            onRemove: () => onToggleFilter("sourceFilters", source),
          });
        }
      }

      // Hoop
      if (browseFilters.hoop) {
        const hoopLabel =
          browseFilters.hoop === HOOP_UNKNOWN_FILTER
            ? "Hoop: Hoop unknown"
            : `Hoop: ${browseFilters.hoop}`;
        chips.push({
          id: "hoop",
          label: hoopLabel,
          onRemove: () => onUpdateFilter("hoop", ""),
        });
      }

      // Dimensions: Min/Max Width
      if (browseFilters.minWidth && browseFilters.minWidth.trim() !== "") {
        chips.push({
          id: "min-width",
          label: `Width ≥ ${browseFilters.minWidth}mm`,
          onRemove: () => onUpdateFilter("minWidth", ""),
        });
      }
      if (browseFilters.maxWidth && browseFilters.maxWidth.trim() !== "") {
        chips.push({
          id: "max-width",
          label: `Width ≤ ${browseFilters.maxWidth}mm`,
          onRemove: () => onUpdateFilter("maxWidth", ""),
        });
      }

      // Dimensions: Min/Max Height
      if (browseFilters.minHeight && browseFilters.minHeight.trim() !== "") {
        chips.push({
          id: "min-height",
          label: `Height ≥ ${browseFilters.minHeight}mm`,
          onRemove: () => onUpdateFilter("minHeight", ""),
        });
      }
      if (browseFilters.maxHeight && browseFilters.maxHeight.trim() !== "") {
        chips.push({
          id: "max-height",
          label: `Height ≤ ${browseFilters.maxHeight}mm`,
          onRemove: () => onUpdateFilter("maxHeight", ""),
        });
      }

      // Rating
      if (browseFilters.rating && browseFilters.rating.trim() !== "") {
        chips.push({
          id: "rating",
          label: `Rating: ≥ ${browseFilters.rating}★`,
          onRemove: () => onUpdateFilter("rating", ""),
        });
      }

      // Stitched
      if (browseFilters.stitched === "yes") {
        chips.push({
          id: "stitched",
          label: "Stitched: Yes",
          onRemove: () => onUpdateFilter("stitched", ""),
        });
      } else if (browseFilters.stitched === "no") {
        chips.push({
          id: "stitched",
          label: "Stitched: No",
          onRemove: () => onUpdateFilter("stitched", ""),
        });
      }

      // Image Tag Verification
      if (browseFilters.imageTagVerification && browseFilters.imageTagVerification !== "all") {
        const label =
          browseFilters.imageTagVerification === "verified"
            ? "Image tags: Verified"
            : "Image tags: Unverified";
        chips.push({
          id: "image-tag-verification",
          label,
          onRemove: () => onUpdateFilter("imageTagVerification", "all"),
        });
      }

      // Stitching Tag Verification
      if (
        browseFilters.stitchingTagVerification &&
        browseFilters.stitchingTagVerification !== "all"
      ) {
        const label =
          browseFilters.stitchingTagVerification === "verified"
            ? "Stitching tags: Verified"
            : "Stitching tags: Unverified";
        chips.push({
          id: "stitching-tag-verification",
          label,
          onRemove: () => onUpdateFilter("stitchingTagVerification", "all"),
        });
      }

      // Needs Attention
      if (browseFilters.needsAttention && browseFilters.needsAttention !== "off") {
        let label = "Needs attention: All";
        if (browseFilters.needsAttention === "cannot_draw") {
          label = "Needs attention: Cannot be drawn";
        } else if (browseFilters.needsAttention === "master_only") {
          label = "Needs attention: Master files only";
        }
        chips.push({
          id: "needs-attention",
          label,
          onRemove: () => onUpdateFilter("needsAttention", "off"),
        });
      }

      return chips;
    })()
  );
</script>

{#if activeChips.length > 0}
  <div
    class="browse-active-filter-chips flex flex-wrap items-center gap-1.5 py-1"
    data-testid="active-filter-chips"
    aria-label="Active filters"
  >
    <span class="text-xs font-semibold text-[var(--text-secondary)] mr-0.5">Active filters:</span>
    {#each activeChips as chip (chip.id)}
      <span
        class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium bg-[var(--surface-card-subtle)] border border-[var(--border-default)] text-[var(--text-primary)] shadow-2xs"
        data-testid={`active-filter-chip-${chip.id}`}
      >
        <span>{chip.label}</span>
        <button
          type="button"
          class="text-[var(--text-muted)] hover:text-[var(--notice-error-text)] rounded-full hover:bg-[var(--surface-hover)] p-0.5 leading-none transition-colors cursor-pointer"
          title={`Remove filter: ${chip.label}`}
          aria-label={`Remove filter: ${chip.label}`}
          onclick={chip.onRemove}
          data-testid={`active-filter-remove-${chip.id}`}
        >
          ✕
        </button>
      </span>
    {/each}
    <button
      type="button"
      class="text-xs font-medium text-[var(--text-brand)] hover:underline ml-1 cursor-pointer"
      onclick={onClearFilters}
      data-testid="active-filters-clear-all"
    >
      Clear all
    </button>
  </div>
{/if}
