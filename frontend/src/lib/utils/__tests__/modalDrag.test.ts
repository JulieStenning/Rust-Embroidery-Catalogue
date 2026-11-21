// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { clampModalOffset, draggableModal } from "../modalDrag";

function dispatchPointer(
  target: EventTarget,
  type: string,
  init: { clientX?: number; clientY?: number; button?: number } = {}
) {
  const event = new MouseEvent(type, {
    bubbles: true,
    cancelable: true,
    clientX: init.clientX ?? 0,
    clientY: init.clientY ?? 0,
    button: init.button ?? 0,
  });
  target.dispatchEvent(event);
}

describe("clampModalOffset", () => {
  const viewport = { width: 1200, height: 800 };
  const centeredRect = { left: 280, top: 150, width: 640, height: 500 };

  it("preserves offsets that keep the modal comfortably within the viewport", () => {
    expect(clampModalOffset({ x: 120, y: -40 }, centeredRect, viewport)).toEqual({
      x: 120,
      y: -40,
    });
  });

  it("clamps upward movement so the top edge stays at least 8px from the top", () => {
    expect(clampModalOffset({ x: 0, y: -500 }, centeredRect, viewport)).toEqual({
      x: 0,
      y: -142, // 8 - 150
    });
  });

  it("clamps downward movement so the header stays reachable at the bottom", () => {
    expect(clampModalOffset({ x: 0, y: 1000 }, centeredRect, viewport)).toEqual({
      x: 0,
      y: 594, // (800 - 56) - 150
    });
  });

  it("allows sliding horizontally aside while keeping at least 120px reachable", () => {
    // Far left: minLeft = -(640 - 120) = -520 => minOffsetX = -520 - 280 = -800
    expect(clampModalOffset({ x: -2000, y: 0 }, centeredRect, viewport)).toEqual({
      x: -800,
      y: 0,
    });
    // Far right: maxLeft = 1200 - 120 = 1080 => maxOffsetX = 1080 - 280 = 800
    expect(clampModalOffset({ x: 2000, y: 0 }, centeredRect, viewport)).toEqual({
      x: 800,
      y: 0,
    });
  });
});

describe("draggableModal action", () => {
  let dialog: HTMLDivElement;
  let handle: HTMLDivElement;
  let action: { destroy: () => void };

  beforeEach(() => {
    vi.spyOn(window, "innerWidth", "get").mockReturnValue(1200);
    vi.spyOn(window, "innerHeight", "get").mockReturnValue(800);

    dialog = document.createElement("div");
    handle = document.createElement("div");
    handle.setAttribute("data-modal-drag-handle", "");
    const button = document.createElement("button");
    button.textContent = "Inside";
    handle.appendChild(button);
    dialog.appendChild(handle);
    document.body.appendChild(dialog);

    vi.spyOn(dialog, "getBoundingClientRect").mockReturnValue({
      left: 280,
      top: 150,
      width: 640,
      height: 500,
      right: 920,
      bottom: 650,
      x: 280,
      y: 150,
      toJSON: () => ({}),
    });

    action = draggableModal(dialog);
  });

  afterEach(() => {
    action.destroy();
    dialog.remove();
    vi.restoreAllMocks();
  });

  it("translates the dialog while dragging the handle and cleans up on pointerup", () => {
    dispatchPointer(handle, "pointerdown", { clientX: 400, clientY: 180, button: 0 });
    expect(dialog.classList.contains("is-dragging")).toBe(true);

    dispatchPointer(window, "pointermove", { clientX: 520, clientY: 230 });
    expect(dialog.style.transform).toBe("translate3d(120px, 50px, 0)");

    dispatchPointer(window, "pointerup", { clientX: 520, clientY: 230 });
    expect(dialog.classList.contains("is-dragging")).toBe(false);

    // Further movement after pointerup does not move the dialog
    dispatchPointer(window, "pointermove", { clientX: 700, clientY: 400 });
    expect(dialog.style.transform).toBe("translate3d(120px, 50px, 0)");
  });

  it("re-centres the dialog on double-click of the handle", () => {
    dispatchPointer(handle, "pointerdown", { clientX: 400, clientY: 180, button: 0 });
    dispatchPointer(window, "pointermove", { clientX: 500, clientY: 220 });
    dispatchPointer(window, "pointerup", { clientX: 500, clientY: 220 });
    expect(dialog.style.transform).toBe("translate3d(100px, 40px, 0)");

    handle.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(dialog.style.transform).toBe("");
  });

  it("ignores non-primary mouse buttons and interactive controls inside the handle", () => {
    // Right-click should not start drag
    dispatchPointer(handle, "pointerdown", { clientX: 400, clientY: 180, button: 2 });
    expect(dialog.classList.contains("is-dragging")).toBe(false);

    // Clicking a button inside the handle should not start drag
    const innerButton = handle.querySelector("button")!;
    dispatchPointer(innerButton, "pointerdown", { clientX: 400, clientY: 180, button: 0 });
    expect(dialog.classList.contains("is-dragging")).toBe(false);
  });
});
