// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

export interface ModalRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface ViewportSize {
  width: number;
  height: number;
}

export interface ModalOffset {
  x: number;
  y: number;
}

export interface DraggableModalOptions {
  /** CSS selector inside the dialog node that acts as the drag handle. */
  handleSelector?: string;
}

const MIN_VISIBLE_HORIZONTAL_PX = 120;
const TOP_EDGE_MARGIN_PX = 8;
const MIN_VISIBLE_HEADER_HEIGHT_PX = 56;
const INTERACTIVE_SELECTOR = "input, button, select, textarea, a, label";

/**
 * Clamp a modal's translation offset so its header stays inside the viewport
 * and enough of the dialog remains horizontally reachable to drag it back.
 */
export function clampModalOffset(
  rawOffset: ModalOffset,
  baseRect: ModalRect,
  viewport: ViewportSize
): ModalOffset {
  const minVisibleX = Math.min(MIN_VISIBLE_HORIZONTAL_PX, Math.max(0, baseRect.width));
  const minLeft = -(Math.max(0, baseRect.width) - minVisibleX);
  const maxLeft = Math.max(minLeft, viewport.width - minVisibleX);

  const minTop = TOP_EDGE_MARGIN_PX;
  const maxTop = Math.max(minTop, viewport.height - MIN_VISIBLE_HEADER_HEIGHT_PX);

  const minOffsetX = minLeft - baseRect.left;
  const maxOffsetX = maxLeft - baseRect.left;
  const minOffsetY = minTop - baseRect.top;
  const maxOffsetY = maxTop - baseRect.top;

  return {
    x: Math.round(Math.min(Math.max(rawOffset.x, minOffsetX), maxOffsetX)),
    y: Math.round(Math.min(Math.max(rawOffset.y, minOffsetY), maxOffsetY)),
  };
}

/**
 * Svelte action that makes a modal dialog draggable by a handle element
 * (defaulting to `[data-modal-drag-handle]`), clamped to the viewport.
 * Double-clicking the handle re-centres the modal.
 */
export function draggableModal(node: HTMLElement, options: DraggableModalOptions = {}) {
  const selector = options.handleSelector ?? "[data-modal-drag-handle]";
  let offset: ModalOffset = { x: 0, y: 0 };
  let dragging = false;
  let startPointerX = 0;
  let startPointerY = 0;
  let startOffsetX = 0;
  let startOffsetY = 0;
  let baseRect: ModalRect = { left: 0, top: 0, width: 0, height: 0 };
  let previousUserSelect = "";

  function applyTransform() {
    if (offset.x === 0 && offset.y === 0) {
      node.style.transform = "";
    } else {
      node.style.transform = `translate3d(${offset.x}px, ${offset.y}px, 0)`;
    }
  }

  function resetPosition() {
    offset = { x: 0, y: 0 };
    applyTransform();
  }

  function stopDrag() {
    if (!dragging) return;
    dragging = false;
    node.classList.remove("is-dragging");
    if (typeof document !== "undefined") {
      document.body.style.userSelect = previousUserSelect;
    }
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
    window.removeEventListener("pointercancel", onPointerUp);
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragging) return;
    const rawOffset: ModalOffset = {
      x: startOffsetX + (event.clientX - startPointerX),
      y: startOffsetY + (event.clientY - startPointerY),
    };
    offset = clampModalOffset(rawOffset, baseRect, {
      width: window.innerWidth,
      height: window.innerHeight,
    });
    applyTransform();
  }

  function onPointerUp() {
    stopDrag();
  }

  function getHandle(): HTMLElement | null {
    return node.querySelector<HTMLElement>(selector);
  }

  function onPointerDown(event: PointerEvent) {
    // Only drag on primary mouse/pen/touch button
    if (event.button !== 0) return;
    const target = event.target instanceof Element ? event.target : null;
    if (target?.closest(INTERACTIVE_SELECTOR)) return;

    const rect = node.getBoundingClientRect();
    baseRect = {
      left: rect.left - offset.x,
      top: rect.top - offset.y,
      width: rect.width,
      height: rect.height,
    };
    startPointerX = event.clientX;
    startPointerY = event.clientY;
    startOffsetX = offset.x;
    startOffsetY = offset.y;
    dragging = true;

    node.classList.add("is-dragging");
    if (typeof document !== "undefined") {
      previousUserSelect = document.body.style.userSelect;
      document.body.style.userSelect = "none";
    }

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
  }

  function onDoubleClick(event: MouseEvent) {
    const target = event.target instanceof Element ? event.target : null;
    if (target?.closest(INTERACTIVE_SELECTOR)) return;
    resetPosition();
  }

  const handle = getHandle();
  if (handle) {
    handle.addEventListener("pointerdown", onPointerDown);
    handle.addEventListener("dblclick", onDoubleClick);
  }

  return {
    destroy() {
      stopDrag();
      if (handle) {
        handle.removeEventListener("pointerdown", onPointerDown);
        handle.removeEventListener("dblclick", onDoubleClick);
      }
    },
  };
}
