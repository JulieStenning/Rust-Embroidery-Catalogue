<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /**
   * Reusable Alphabet & Font Compression setting card.
   */
  let {
    checked = $bindable(true),
    disabled = false,
    testIdPrefix = "settings",
    showBorder = true,
  } = $props();
</script>

<div
  class={showBorder ? "border-b border-[var(--border-default)] pb-5" : ""}
  data-testid="{testIdPrefix}-alphabet-compression-section"
>
  <h2 class="text-sm font-semibold text-[var(--text-primary)] mb-1 flex items-center gap-1.5">
    <span>🔤</span>
    <span>Alphabet &amp; Font Compression</span>
  </h2>
  <p class="text-sm text-[var(--text-secondary)] mb-3">
    Choose how digital embroidery font and alphabet packages are catalogued during import.
  </p>
  <label
    class="flex items-start gap-2.5 p-3 rounded border border-[var(--border-default)] bg-[var(--surface-card-subtle)] hover:bg-[var(--surface-hover)] cursor-pointer text-sm text-[var(--text-primary)] transition-colors"
    data-testid="{testIdPrefix}-zip-alphabets-label"
  >
    <input
      type="checkbox"
      class="mt-0.5 rounded border-[var(--border-default)] accent-[var(--text-brand)] cursor-pointer"
      bind:checked
      {disabled}
      data-testid="{testIdPrefix}-zip-alphabets-checkbox"
    />
    <div>
      <span class="font-medium text-[var(--text-primary)] block">
        Compress full alphabet sets and catalogue preview samples only
      </span>
      <span class="text-xs text-[var(--text-secondary)] block mt-0.5">
        When enabled, imports detect alphabet collections and compress the full font set into a
        companion .zip archive in the designs folder. Only representative preview glyphs (Upper A,
        Lower a, 9, and all punctuation &amp; symbol glyphs) for each size are catalogued in the
        database.
      </span>
    </div>
  </label>
</div>
