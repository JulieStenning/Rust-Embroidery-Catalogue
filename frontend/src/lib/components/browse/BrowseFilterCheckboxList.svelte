<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /**
   * @type {{
   *   title: string,
   *   options: Array<string | { description: string }>,
   *   selected: string[],
   *   onToggle: (value: string) => void
   * }}
   */
  let { title, options, selected, onToggle } = $props();
</script>

<div class="space-y-1">
  <span class="block text-xs font-semibold text-[var(--text-secondary)]">{title}</span>
  <div
    class="border border-[var(--border-default)] rounded bg-[var(--surface-input)] max-h-36 overflow-auto p-1.5 space-y-1"
  >
    {#each options as opt}
      {@const val = typeof opt === "string" ? opt : opt.description}
      <label class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer">
        <input
          type="checkbox"
          checked={selected.includes(val)}
          onchange={() => onToggle(val)}
          class="ui-checkbox rounded"
        />
        <span>{val}</span>
      </label>
    {/each}
  </div>
</div>
