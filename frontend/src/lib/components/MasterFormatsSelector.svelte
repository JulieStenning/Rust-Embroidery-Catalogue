<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /**
   * Reusable Master Formats selector component.
   * Allows selecting digitising software presets and custom extensions.
   */

  export const MASTER_FORMAT_PRESETS = [
    {
      id: "embird",
      name: "Embird Studio & Cross Stitch",
      exts: ["eof", "ecf"],
      desc: ".eof, .ecf",
    },
    {
      id: "wilcom_hatch",
      name: "Wilcom / Hatch",
      exts: ["emb"],
      desc: ".emb",
    },
    {
      id: "bernina",
      name: "Bernina",
      exts: ["art", "art60", "art70", "art80"],
      desc: ".art, .art60, .art70, .art80",
    },
    {
      id: "embrilliance",
      name: "Embrilliance",
      exts: ["be"],
      desc: ".be",
    },
    {
      id: "janome",
      name: "Janome Digitizer",
      exts: ["jan"],
      desc: ".jan",
    },
    {
      id: "mysewnet",
      name: "mySewnet / Premier+",
      exts: ["edo", "vp4"],
      desc: ".edo, .vp4",
    },
  ];

  /**
   * Parse a comma-separated format list into a Set of clean extensions.
   * @param {string} raw
   * @returns {Set<string>}
   */
  export function parseMasterFormatsSet(raw) {
    const list = String(raw || "")
      .split(/[,;\s]+/)
      .map((s) => s.trim().replace(/^\./, "").toLowerCase())
      .filter((s) => s.length > 0 && /^[a-z0-9]+$/.test(s));
    return new Set(list);
  }

  /**
   * Normalise a master format string to sorted comma-separated string.
   * @param {string} raw
   * @returns {string}
   */
  export function normalizeMasterFormats(raw) {
    const set = parseMasterFormatsSet(raw);
    return Array.from(set).sort().join(",");
  }

  /**
   * @typedef {Object} Props
   * @property {string} [value=""] - Comma-separated list of enabled format extensions
   * @property {boolean} [disabled=false]
   * @property {(val: string) => void} [onchange]
   */

  /** @type {Props} */
  let { value = $bindable(""), disabled = false, onchange } = $props();

  let customInput = $state("");

  // Sync custom input whenever external value changes
  $effect(() => {
    const allSet = parseMasterFormatsSet(value);
    const presetExts = new Set(MASTER_FORMAT_PRESETS.flatMap((p) => p.exts));
    const customList = Array.from(allSet).filter((ext) => !presetExts.has(ext));
    customInput = customList.join(", ");
  });

  /**
   * Check whether all extensions in a preset are enabled in value.
   * @param {{ exts: string[] }} preset
   */
  function isPresetEnabled(preset) {
    const currentSet = parseMasterFormatsSet(value);
    return preset.exts.every((ext) => currentSet.has(ext));
  }

  /**
   * Toggle a preset's extensions on or off.
   * @param {{ exts: string[] }} preset
   */
  function togglePreset(preset) {
    if (disabled) return;
    const currentSet = parseMasterFormatsSet(value);
    const enabled = isPresetEnabled(preset);
    if (enabled) {
      preset.exts.forEach((ext) => currentSet.delete(ext));
    } else {
      preset.exts.forEach((ext) => currentSet.add(ext));
    }
    const nextVal = Array.from(currentSet).sort().join(",");
    value = nextVal;
    if (onchange) onchange(nextVal);
  }

  /**
   * Handle edits to the custom formats input.
   * @param {string} rawCustom
   */
  function handleCustomInput(rawCustom) {
    customInput = rawCustom;
    const customSet = parseMasterFormatsSet(rawCustom);
    const currentSet = parseMasterFormatsSet(value);
    const presetExts = new Set(MASTER_FORMAT_PRESETS.flatMap((p) => p.exts));

    // Remove old custom extensions, keeping enabled presets
    const nextSet = new Set(Array.from(currentSet).filter((ext) => presetExts.has(ext)));
    // Add current custom extensions
    customSet.forEach((ext) => nextSet.add(ext));

    const nextVal = Array.from(nextSet).sort().join(",");
    value = nextVal;
    if (onchange) onchange(nextVal);
  }
</script>

<div class="space-y-4" data-testid="master-formats-selector">
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
    {#each MASTER_FORMAT_PRESETS as preset}
      <label
        class="flex items-start gap-2.5 p-3 rounded-lg border border-[var(--border-default)] bg-[var(--surface-card-subtle)] hover:bg-[var(--surface-hover)] cursor-pointer text-sm text-[var(--text-primary)] transition-colors"
        data-testid={`master-format-preset-${preset.id}`}
      >
        <input
          type="checkbox"
          class="ui-checkbox mt-0.5 rounded cursor-pointer"
          checked={isPresetEnabled(preset)}
          onchange={() => togglePreset(preset)}
          {disabled}
        />
        <div>
          <span class="font-medium text-[var(--text-primary)] block">{preset.name}</span>
          <span class="text-xs text-[var(--text-muted)] font-mono">{preset.desc}</span>
        </div>
      </label>
    {/each}
  </div>

  <div class="space-y-1.5 pt-1">
    <label
      for="custom-master-formats-input"
      class="block text-xs font-semibold text-[var(--text-secondary)]"
    >
      Additional / Custom extensions <span class="font-normal text-[var(--text-muted)]"
        >(comma-separated)</span
      >
    </label>
    <input
      id="custom-master-formats-input"
      type="text"
      value={customInput}
      oninput={(e) => handleCustomInput(/** @type {HTMLInputElement} */ (e.target).value)}
      placeholder="e.g. pxf, pat"
      {disabled}
      class="ui-text-input border border-[var(--border-default)] rounded-md px-3 py-2 text-sm font-mono w-full sm:w-80 focus:outline-none focus:ring-2 focus:ring-[var(--border-focus)] bg-[var(--surface-input)] text-[var(--text-primary)]"
      data-testid="settings-custom-master-formats"
    />
    <p class="text-xs text-[var(--text-muted)]">
      Active formats: <span class="font-mono text-[var(--text-brand)] font-medium"
        >{value || "none (machine stitch files only)"}</span
      >
    </p>
  </div>
</div>
