<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  import { portalToBody } from "../../utils/portal.js";
  import { draggableModal } from "../../utils/modalDrag";

  export interface BulkTagOption {
    id: number | string;
    description: string;
    tag_group?: string | null;
  }

  export interface GroupedBulkTagOptions {
    image: BulkTagOption[];
    stitching: BulkTagOption[];
    unclassified: BulkTagOption[];
  }

  let {
    open = false,
    applying = false,
    selectedCount = 0,
    groupedTagOptions = { image: [], stitching: [], unclassified: [] } as GroupedBulkTagOptions,
    clearAll = false,
    verifyImage = true,
    verifyStitching = true,
    tagChooserAria = (_tagId: number | string): "true" | "false" | "mixed" => "false",
    tagChooserGlyph = (_tagId: number | string): string => "",
    onClearAllChange = (_checked: boolean) => {},
    onVerifyImageChange = (_checked: boolean) => {},
    onVerifyStitchingChange = (_checked: boolean) => {},
    onToggleTag = (_tagId: number | string) => {},
    onClose = () => {},
    onApply = () => {},
  }: {
    open: boolean;
    applying: boolean;
    selectedCount: number;
    groupedTagOptions: GroupedBulkTagOptions;
    clearAll: boolean;
    verifyImage: boolean;
    verifyStitching: boolean;
    tagChooserAria: (tagId: number | string) => "true" | "false" | "mixed";
    tagChooserGlyph: (tagId: number | string) => string;
    onClearAllChange: (checked: boolean) => void;
    onVerifyImageChange: (checked: boolean) => void;
    onVerifyStitchingChange: (checked: boolean) => void;
    onToggleTag: (tagId: number | string) => void;
    onClose: () => void;
    onApply: () => void;
  } = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    use:portalToBody
    class="tag-chooser-overlay no-print"
    role="dialog"
    aria-modal="true"
    aria-busy={applying}
    aria-labelledby="bulk-tag-title"
    onkeydown={(event: KeyboardEvent) => {
      if (event.key === "Escape" && !applying) {
        onClose();
      }
    }}
  >
    <button
      type="button"
      class="tag-chooser-backdrop tag-chooser-backdrop-inspectable"
      aria-label="Close tag chooser"
      onclick={() => onClose()}
      disabled={applying}
    ></button>
    <div use:draggableModal class="tag-chooser-dialog" data-testid="bulk-tag-modal-dialog">
      <div
        class="tag-chooser-header tag-chooser-drag-handle"
        data-modal-drag-handle
        data-testid="bulk-tag-modal-header"
        title="Drag to move; double-click to re-centre"
      >
        <h2
          id="bulk-tag-title"
          class="text-lg font-bold text-[var(--text-primary)]"
          style="margin:0;"
        >
          Choose Tags for Selected Designs
        </h2>
        <span class="tag-chooser-drag-hint" aria-hidden="true">⋮⋮ Drag</span>
      </div>
      <div class="tag-chooser-body">
        <p class="text-xs text-[var(--text-muted)] font-semibold" style="margin:0 0 0.75rem 0;">
          {selectedCount} design{selectedCount === 1 ? "" : "s"} selected.
        </p>

        <div class="tag-chooser-section" style="margin-bottom:0.75rem;">
          <div class="tag-chooser-header-options">
            <label class="tag-chooser-option" style="font-weight:600;">
              <input
                type="checkbox"
                checked={clearAll}
                disabled={applying}
                onchange={(event) => onClearAllChange(event.currentTarget.checked)}
              />
              <span>Untagged (clear all tags)</span>
            </label>

            <label class="tag-chooser-option" style="font-weight:600;">
              <input
                type="checkbox"
                data-testid="bulk-verify-image-checkbox"
                checked={verifyImage}
                disabled={applying}
                onchange={(event) => onVerifyImageChange(event.currentTarget.checked)}
              />
              <span>Verify image tags</span>
            </label>

            <label class="tag-chooser-option" style="font-weight:600;">
              <input
                type="checkbox"
                data-testid="bulk-verify-stitching-checkbox"
                checked={verifyStitching}
                disabled={applying}
                onchange={(event) => onVerifyStitchingChange(event.currentTarget.checked)}
              />
              <span>Verify stitching tags</span>
            </label>
          </div>
        </div>

        <div class="tag-chooser-sections">
          {#if groupedTagOptions.image.length > 0}
            <section class="tag-chooser-section">
              <p class="tag-chooser-section-title tag-chooser-section-title-image font-semibold">
                Image tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.image as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    disabled={applying}
                    onclick={() => onToggleTag(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.stitching.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-stitching font-semibold"
              >
                Stitching tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.stitching as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    disabled={applying}
                    onclick={() => onToggleTag(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}

          {#if groupedTagOptions.unclassified.length > 0}
            <section class="tag-chooser-section">
              <p
                class="tag-chooser-section-title tag-chooser-section-title-unclassified font-semibold"
              >
                Unclassified tags
              </p>
              <div class="tag-chooser-grid">
                {#each groupedTagOptions.unclassified as tagOption (tagOption.id)}
                  <button
                    type="button"
                    class="tag-chooser-option"
                    role="checkbox"
                    aria-checked={tagChooserAria(tagOption.id)}
                    disabled={applying}
                    onclick={() => onToggleTag(tagOption.id)}
                  >
                    <span class="tag-chooser-box">{tagChooserGlyph(tagOption.id)}</span>
                    <span>{tagOption.description}</span>
                  </button>
                {/each}
              </div>
            </section>
          {/if}
        </div>
      </div>
      <div class="tag-chooser-footer">
        <button
          type="button"
          class="menu-button-secondary"
          onclick={() => onClose()}
          disabled={applying}
        >
          Cancel
        </button>
        <button
          type="button"
          class="menu-button-primary"
          onclick={() => onApply()}
          disabled={applying}
        >
          {applying ? "Applying tags..." : "Apply tags"}
        </button>
      </div>
    </div>
  </div>
{/if}
