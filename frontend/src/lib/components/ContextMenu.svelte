<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { contextMenu, closeContextMenu } from "../stores/contextMenuStore";

  let menuElement = $state<HTMLDivElement | null>(null);

  let clampedX = $derived.by(() => {
    if (typeof window === "undefined") return $contextMenu.x;
    const menuWidth = 180;
    const maxX = window.innerWidth - menuWidth - 8;
    return Math.max(8, Math.min($contextMenu.x, maxX));
  });

  let clampedY = $derived.by(() => {
    if (typeof window === "undefined") return $contextMenu.y;
    const menuHeight = 150;
    const maxY = window.innerHeight - menuHeight - 8;
    return Math.max(8, Math.min($contextMenu.y, maxY));
  });

  async function handleCut(): Promise<void> {
    const target = $contextMenu.target;
    if (!target) {
      closeContextMenu();
      return;
    }

    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      const start = target.selectionStart ?? 0;
      const end = target.selectionEnd ?? 0;
      if (start < end) {
        const text = target.value.slice(start, end);
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          // Fallback if clipboard API is restricted
        }
        target.setRangeText("", start, end, "end");
        target.dispatchEvent(new Event("input", { bubbles: true }));
      }
    }
    closeContextMenu();
  }

  async function handleCopy(): Promise<void> {
    const target = $contextMenu.target;
    if (!target) {
      closeContextMenu();
      return;
    }

    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      const start = target.selectionStart ?? 0;
      const end = target.selectionEnd ?? 0;
      if (start < end) {
        const text = target.value.slice(start, end);
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          // Fallback if clipboard API is restricted
        }
      }
    } else {
      const selection = window.getSelection()?.toString();
      if (selection) {
        try {
          await navigator.clipboard.writeText(selection);
        } catch {
          // Fallback
        }
      }
    }
    closeContextMenu();
  }

  async function handlePaste(): Promise<void> {
    const target = $contextMenu.target;
    if (!target) {
      closeContextMenu();
      return;
    }

    try {
      const text = await navigator.clipboard.readText();
      if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
        const start = target.selectionStart ?? target.value.length;
        const end = target.selectionEnd ?? target.value.length;
        target.setRangeText(text, start, end, "end");
        target.dispatchEvent(new Event("input", { bubbles: true }));
      }
    } catch {
      // Clipboard read may fail or be denied in some sandboxes
    }
    closeContextMenu();
  }

  function handleSelectAll(): void {
    const target = $contextMenu.target;
    if (!target) {
      closeContextMenu();
      return;
    }

    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      target.focus();
      target.select();
    } else if (target.isContentEditable) {
      const range = document.createRange();
      range.selectNodeContents(target);
      const sel = window.getSelection();
      sel?.removeAllRanges();
      sel?.addRange(range);
    }
    closeContextMenu();
  }

  function onWindowPointerDown(event: PointerEvent): void {
    if (!$contextMenu.isOpen) return;
    if (menuElement && !menuElement.contains(event.target as Node)) {
      closeContextMenu();
    }
  }

  function onWindowKeyDown(event: KeyboardEvent): void {
    if (!$contextMenu.isOpen) return;
    if (event.key === "Escape") {
      closeContextMenu();
    }
  }

  function onWindowScroll(): void {
    if ($contextMenu.isOpen) {
      closeContextMenu();
    }
  }

  onMount(() => {
    if (typeof window !== "undefined") {
      window.addEventListener("pointerdown", onWindowPointerDown);
      window.addEventListener("keydown", onWindowKeyDown);
      window.addEventListener("scroll", onWindowScroll, true);
    }
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("pointerdown", onWindowPointerDown);
      window.removeEventListener("keydown", onWindowKeyDown);
      window.removeEventListener("scroll", onWindowScroll, true);
    }
  });
</script>

{#if $contextMenu.isOpen}
  <div
    bind:this={menuElement}
    data-testid="text-context-menu"
    role="menu"
    aria-label="Text editing options"
    class="fixed z-[10000] min-w-[170px] py-1 rounded shadow-lg bg-[var(--surface-card)] text-[var(--text-primary)] border border-[var(--border-default)] select-none text-xs font-sans"
    style="left: {clampedX}px; top: {clampedY}px;"
  >
    <button
      type="button"
      role="menuitem"
      data-testid="context-menu-cut"
      disabled={!$contextMenu.hasSelection}
      onclick={handleCut}
      class="flex items-center justify-between w-full px-3 py-1.5 text-left hover:bg-[var(--surface-hover)] disabled:opacity-40 disabled:hover:bg-transparent disabled:cursor-not-allowed transition-colors"
    >
      <span>Cut</span>
      <span class="ml-4 font-mono text-[10px] text-[var(--text-muted)]">Ctrl+X</span>
    </button>

    <button
      type="button"
      role="menuitem"
      data-testid="context-menu-copy"
      disabled={!$contextMenu.hasSelection}
      onclick={handleCopy}
      class="flex items-center justify-between w-full px-3 py-1.5 text-left hover:bg-[var(--surface-hover)] disabled:opacity-40 disabled:hover:bg-transparent disabled:cursor-not-allowed transition-colors"
    >
      <span>Copy</span>
      <span class="ml-4 font-mono text-[10px] text-[var(--text-muted)]">Ctrl+C</span>
    </button>

    <button
      type="button"
      role="menuitem"
      data-testid="context-menu-paste"
      onclick={handlePaste}
      class="flex items-center justify-between w-full px-3 py-1.5 text-left hover:bg-[var(--surface-hover)] disabled:opacity-40 disabled:hover:bg-transparent disabled:cursor-not-allowed transition-colors"
    >
      <span>Paste</span>
      <span class="ml-4 font-mono text-[10px] text-[var(--text-muted)]">Ctrl+V</span>
    </button>

    <div class="border-t border-[var(--border-default)] my-1" role="separator"></div>

    <button
      type="button"
      role="menuitem"
      data-testid="context-menu-select-all"
      disabled={!$contextMenu.hasContent}
      onclick={handleSelectAll}
      class="flex items-center justify-between w-full px-3 py-1.5 text-left hover:bg-[var(--surface-hover)] disabled:opacity-40 disabled:hover:bg-transparent disabled:cursor-not-allowed transition-colors"
    >
      <span>Select All</span>
      <span class="ml-4 font-mono text-[10px] text-[var(--text-muted)]">Ctrl+A</span>
    </button>
  </div>
{/if}
