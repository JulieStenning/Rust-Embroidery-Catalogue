// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";
import { isEditableElement, handleContextMenu, initContextMenuHandler } from "../contextMenu";
import { contextMenu, closeContextMenu } from "../../stores/contextMenuStore";

describe("contextMenu service", () => {
  beforeEach(() => {
    closeContextMenu();
  });
  describe("isEditableElement", () => {
    it("returns false for null or non-HTMLElement targets", () => {
      expect(isEditableElement(null)).toBe(false);
      expect(isEditableElement({} as EventTarget)).toBe(false);
    });

    it("returns false for standard layout elements", () => {
      const div = document.createElement("div");
      const button = document.createElement("button");
      const span = document.createElement("span");
      const header = document.createElement("h1");

      expect(isEditableElement(div)).toBe(false);
      expect(isEditableElement(button)).toBe(false);
      expect(isEditableElement(span)).toBe(false);
      expect(isEditableElement(header)).toBe(false);
    });

    it("returns true for textarea elements", () => {
      const textarea = document.createElement("textarea");
      expect(isEditableElement(textarea)).toBe(true);
    });

    it("returns true for text input elements", () => {
      const textInput = document.createElement("input");
      textInput.type = "text";

      const searchInput = document.createElement("input");
      searchInput.type = "search";

      const numberInput = document.createElement("input");
      numberInput.type = "number";

      expect(isEditableElement(textInput)).toBe(true);
      expect(isEditableElement(searchInput)).toBe(true);
      expect(isEditableElement(numberInput)).toBe(true);
    });

    it("returns false for non-text input types like button, checkbox, and radio", () => {
      const btnInput = document.createElement("input");
      btnInput.type = "button";

      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";

      const radio = document.createElement("input");
      radio.type = "radio";

      expect(isEditableElement(btnInput)).toBe(false);
      expect(isEditableElement(checkbox)).toBe(false);
      expect(isEditableElement(radio)).toBe(false);
    });

    it("returns true for contenteditable elements", () => {
      const editableDiv = document.createElement("div");
      editableDiv.setAttribute("contenteditable", "true");
      document.body.appendChild(editableDiv);
      expect(isEditableElement(editableDiv)).toBe(true);
      document.body.removeChild(editableDiv);
    });
  });

  describe("handleContextMenu", () => {
    it("prevents default and closes context menu store for static layout elements", () => {
      const div = document.createElement("div");
      const event = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
      });
      Object.defineProperty(event, "target", { value: div });

      const preventDefaultSpy = vi.spyOn(event, "preventDefault");
      handleContextMenu(event);
      expect(preventDefaultSpy).toHaveBeenCalledTimes(1);
      expect(get(contextMenu).isOpen).toBe(false);
    });

    it("prevents default and opens context menu store for text inputs", () => {
      const input = document.createElement("input");
      input.type = "text";
      input.value = "hello";
      const event = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
        clientX: 120,
        clientY: 240,
      });
      Object.defineProperty(event, "target", { value: input });

      const preventDefaultSpy = vi.spyOn(event, "preventDefault");
      handleContextMenu(event);
      expect(preventDefaultSpy).toHaveBeenCalledTimes(1);
      const state = get(contextMenu);
      expect(state.isOpen).toBe(true);
      expect(state.x).toBe(120);
      expect(state.y).toBe(240);
      expect(state.target).toBe(input);
      expect(state.hasContent).toBe(true);
    });

    it("prevents default and opens context menu store for textareas", () => {
      const textarea = document.createElement("textarea");
      textarea.value = "notes";
      const event = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
        clientX: 50,
        clientY: 75,
      });
      Object.defineProperty(event, "target", { value: textarea });

      const preventDefaultSpy = vi.spyOn(event, "preventDefault");
      handleContextMenu(event);
      expect(preventDefaultSpy).toHaveBeenCalledTimes(1);
      const state = get(contextMenu);
      expect(state.isOpen).toBe(true);
      expect(state.x).toBe(50);
      expect(state.y).toBe(75);
      expect(state.target).toBe(textarea);
      expect(state.hasContent).toBe(true);
    });

    it("permits default when Shift or Alt is pressed in dev mode", () => {
      const div = document.createElement("div");
      const shiftEvent = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
        shiftKey: true,
      });
      Object.defineProperty(shiftEvent, "target", { value: div });

      const preventDefaultSpy = vi.spyOn(shiftEvent, "preventDefault");
      handleContextMenu(shiftEvent);
      expect(preventDefaultSpy).not.toHaveBeenCalled();
    });
  });

  describe("initContextMenuHandler", () => {
    let mockWindow: Window;

    beforeEach(() => {
      mockWindow = {
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as Window;
    });

    it("attaches listener and returns cleanup function", () => {
      const cleanup = initContextMenuHandler(mockWindow);
      expect(mockWindow.addEventListener).toHaveBeenCalledWith("contextmenu", expect.any(Function));

      cleanup();
      expect(mockWindow.removeEventListener).toHaveBeenCalledWith(
        "contextmenu",
        expect.any(Function)
      );
    });
  });
});
