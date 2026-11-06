// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import { contextMenu, openContextMenu, closeContextMenu } from "../contextMenuStore";

describe("contextMenuStore", () => {
  beforeEach(() => {
    closeContextMenu();
  });

  it("initialises with closed state", () => {
    const state = get(contextMenu);
    expect(state.isOpen).toBe(false);
    expect(state.x).toBe(0);
    expect(state.y).toBe(0);
    expect(state.target).toBeNull();
    expect(state.hasSelection).toBe(false);
    expect(state.hasContent).toBe(false);
  });

  it("updates state when openContextMenu is called", () => {
    const input = document.createElement("input");
    openContextMenu({
      x: 150,
      y: 220,
      target: input,
      hasSelection: true,
      hasContent: true,
    });

    const state = get(contextMenu);
    expect(state.isOpen).toBe(true);
    expect(state.x).toBe(150);
    expect(state.y).toBe(220);
    expect(state.target).toBe(input);
    expect(state.hasSelection).toBe(true);
    expect(state.hasContent).toBe(true);
  });

  it("resets state when closeContextMenu is called", () => {
    const input = document.createElement("input");
    openContextMenu({
      x: 100,
      y: 100,
      target: input,
      hasSelection: true,
      hasContent: true,
    });
    expect(get(contextMenu).isOpen).toBe(true);

    closeContextMenu();
    const state = get(contextMenu);
    expect(state.isOpen).toBe(false);
    expect(state.target).toBeNull();
  });
});
