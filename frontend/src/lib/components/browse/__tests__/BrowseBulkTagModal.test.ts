// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import BrowseBulkTagModal from "../BrowseBulkTagModal.svelte";

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

describe("BrowseBulkTagModal", () => {
  beforeEach(() => {
    vi.spyOn(window, "innerWidth", "get").mockReturnValue(1280);
    vi.spyOn(window, "innerHeight", "get").mockReturnValue(800);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  function baseProps(overrides = {}) {
    return {
      open: true,
      applying: false,
      selectedCount: 2,
      groupedTagOptions: {
        image: [{ id: 1, description: "Floral", tag_group: "image" }],
        stitching: [{ id: 2, description: "Applique", tag_group: "stitching" }],
        unclassified: [],
      },
      clearAll: false,
      verifyImage: true,
      verifyStitching: true,
      tagChooserAria: () => "false" as const,
      tagChooserGlyph: () => "",
      onClearAllChange: vi.fn(),
      onVerifyImageChange: vi.fn(),
      onVerifyStitchingChange: vi.fn(),
      onToggleTag: vi.fn(),
      onClose: vi.fn(),
      onApply: vi.fn(),
      ...overrides,
    };
  }

  it("moves when dragged by its header and re-centres on double-click", () => {
    render(BrowseBulkTagModal, { props: baseProps() });

    const dialog = screen.getByTestId("bulk-tag-modal-dialog");
    const header = screen.getByTestId("bulk-tag-modal-header");

    vi.spyOn(dialog, "getBoundingClientRect").mockReturnValue({
      left: 320,
      top: 160,
      width: 640,
      height: 480,
      right: 960,
      bottom: 640,
      x: 320,
      y: 160,
      toJSON: () => ({}),
    });

    dispatchPointer(header, "pointerdown", { clientX: 500, clientY: 180, button: 0 });
    dispatchPointer(window, "pointermove", { clientX: 680, clientY: 240 });
    dispatchPointer(window, "pointerup", { clientX: 680, clientY: 240 });

    expect(dialog.style.transform).toBe("translate3d(180px, 60px, 0)");

    header.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(dialog.style.transform).toBe("");
  });

  it("uses the lightened inspectable backdrop", () => {
    render(BrowseBulkTagModal, { props: baseProps() });
    const backdrop = screen.getByRole("button", { name: "Close tag chooser" });
    expect(backdrop.classList.contains("tag-chooser-backdrop-inspectable")).toBe(true);
  });
});
