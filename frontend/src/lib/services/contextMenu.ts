// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { openContextMenu, closeContextMenu } from "../stores/contextMenuStore";

/**
 * Determines whether a given DOM event target represents an editable text control
 * where the native context menu (Cut, Copy, Paste, Select All) should be preserved.
 */
export function isEditableElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (
    target.isContentEditable ||
    target.getAttribute("contenteditable") === "true" ||
    target.getAttribute("contenteditable") === ""
  ) {
    return true;
  }

  if (target.tagName === "TEXTAREA") {
    return true;
  }

  if (target.tagName === "INPUT") {
    const input = target as HTMLInputElement;
    const nonTextTypes = ["button", "checkbox", "radio", "submit", "reset", "image", "file"];
    return !nonTextTypes.includes((input.type || "text").toLowerCase());
  }

  return false;
}

/**
 * Handles the global 'contextmenu' event to suppress default browser actions
 * (Save As, Back, Reload, Share, Print, Import Passwords) across all elements,
 * presenting a tailored desktop context menu (Cut, Copy, Paste, Select All)
 * when right-clicking editable fields.
 *
 * In local development mode (`import.meta.env.DEV`), holding Shift or Alt
 * allows native element inspection when needed.
 */
export function handleContextMenu(event: MouseEvent): void {
  // Developer override in local dev mode
  if (import.meta.env?.DEV && (event.shiftKey || event.altKey)) {
    return;
  }

  // Always suppress the default browser/WebView2 context menu
  event.preventDefault();

  if (isEditableElement(event.target)) {
    const target = event.target as HTMLElement;
    let hasSelection = false;
    let hasContent = false;

    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      const start = target.selectionStart ?? 0;
      const end = target.selectionEnd ?? 0;
      hasSelection = start < end;
      hasContent = target.value.length > 0;
    } else if (target.isContentEditable || target.getAttribute("contenteditable") === "true") {
      const selection = window.getSelection();
      hasSelection = Boolean(
        selection && !selection.isCollapsed && selection.toString().length > 0
      );
      hasContent = (target.textContent || "").length > 0;
    }

    openContextMenu({
      x: event.clientX,
      y: event.clientY,
      target: target as HTMLInputElement | HTMLTextAreaElement | HTMLElement,
      hasSelection,
      hasContent,
    });
  } else {
    closeContextMenu();
  }
}

/**
 * Attaches the context menu handler to the window.
 * Returns an unbind / cleanup function.
 */
export function initContextMenuHandler(targetWindow: Window = window): () => void {
  if (typeof targetWindow === "undefined" || !targetWindow.addEventListener) {
    return () => {};
  }

  const listener = (event: MouseEvent) => handleContextMenu(event);
  targetWindow.addEventListener("contextmenu", listener);

  return () => {
    targetWindow.removeEventListener("contextmenu", listener);
  };
}
