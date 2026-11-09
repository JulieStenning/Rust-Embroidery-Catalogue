// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Global text context menu store.
 *
 * Coordinates opening, positioning, and closing the tailored desktop
 * text context menu across all text inputs, textareas, and editable fields.
 */

import { writable } from "svelte/store";

export interface ContextMenuState {
  isOpen: boolean;
  x: number;
  y: number;
  target: HTMLInputElement | HTMLTextAreaElement | HTMLElement | null;
  hasSelection: boolean;
  hasContent: boolean;
}

const initialState: ContextMenuState = {
  isOpen: false,
  x: 0,
  y: 0,
  target: null,
  hasSelection: false,
  hasContent: false,
};

export const contextMenu = writable<ContextMenuState>({ ...initialState });

export interface OpenContextMenuOptions {
  x: number;
  y: number;
  target: HTMLInputElement | HTMLTextAreaElement | HTMLElement | null;
  hasSelection?: boolean;
  hasContent?: boolean;
}

/**
 * Opens the tailored text context menu at the specified viewport coordinates.
 */
export function openContextMenu(options: OpenContextMenuOptions): void {
  const { x, y, target, hasSelection = false, hasContent = false } = options;
  contextMenu.set({
    isOpen: true,
    x,
    y,
    target,
    hasSelection,
    hasContent,
  });
}

/**
 * Closes and resets the text context menu.
 */
export function closeContextMenu(): void {
  contextMenu.set({ ...initialState });
}
