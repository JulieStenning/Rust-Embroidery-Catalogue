// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import ContextMenu from "../ContextMenu.svelte";
import { openContextMenu, closeContextMenu } from "../../stores/contextMenuStore";

describe("ContextMenu component", () => {
  beforeEach(() => {
    closeContextMenu();
  });

  afterEach(() => {
    closeContextMenu();
    vi.restoreAllMocks();
  });

  it("does not render when store isOpen is false", () => {
    render(ContextMenu);
    expect(screen.queryByTestId("text-context-menu")).toBeNull();
  });

  it("renders when store isOpen is true", async () => {
    render(ContextMenu);
    const input = document.createElement("input");
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: false,
      hasContent: false,
    });

    await waitFor(() => {
      expect(screen.getByTestId("text-context-menu")).toBeInTheDocument();
    });

    expect(screen.getByTestId("context-menu-cut")).toBeDisabled();
    expect(screen.getByTestId("context-menu-copy")).toBeDisabled();
    expect(screen.getByTestId("context-menu-paste")).not.toBeDisabled();
    expect(screen.getByTestId("context-menu-select-all")).toBeDisabled();

    document.body.removeChild(input);
  });

  it("enables Cut and Copy when text is selected", async () => {
    render(ContextMenu);
    const input = document.createElement("input");
    input.value = "hello world";
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: true,
      hasContent: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("text-context-menu")).toBeInTheDocument();
    });

    expect(screen.getByTestId("context-menu-cut")).not.toBeDisabled();
    expect(screen.getByTestId("context-menu-copy")).not.toBeDisabled();
    expect(screen.getByTestId("context-menu-select-all")).not.toBeDisabled();

    document.body.removeChild(input);
  });

  it("copies selected text to clipboard and closes menu on Copy click", async () => {
    const writeTextSpy = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, {
      clipboard: { writeText: writeTextSpy },
    });

    render(ContextMenu);
    const input = document.createElement("input");
    input.value = "hello world";
    input.selectionStart = 0;
    input.selectionEnd = 5;
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: true,
      hasContent: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("context-menu-copy")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("context-menu-copy"));
    expect(writeTextSpy).toHaveBeenCalledWith("hello");

    await waitFor(() => {
      expect(screen.queryByTestId("text-context-menu")).toBeNull();
    });

    document.body.removeChild(input);
  });

  it("cuts selected text, updates input value, and closes menu on Cut click", async () => {
    const writeTextSpy = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, {
      clipboard: { writeText: writeTextSpy },
    });

    render(ContextMenu);
    const input = document.createElement("input");
    input.value = "hello world";
    input.selectionStart = 0;
    input.selectionEnd = 6;
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: true,
      hasContent: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("context-menu-cut")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("context-menu-cut"));
    expect(writeTextSpy).toHaveBeenCalledWith("hello ");
    expect(input.value).toBe("world");

    await waitFor(() => {
      expect(screen.queryByTestId("text-context-menu")).toBeNull();
    });

    document.body.removeChild(input);
  });

  it("pastes text from clipboard into input and closes menu", async () => {
    const readTextSpy = vi.fn().mockResolvedValue("sweet");
    Object.assign(navigator, {
      clipboard: { readText: readTextSpy },
    });

    render(ContextMenu);
    const input = document.createElement("input");
    input.value = "home";
    input.selectionStart = 0;
    input.selectionEnd = 0;
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: false,
      hasContent: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("context-menu-paste")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("context-menu-paste"));
    expect(readTextSpy).toHaveBeenCalled();
    expect(input.value).toBe("sweethome");

    await waitFor(() => {
      expect(screen.queryByTestId("text-context-menu")).toBeNull();
    });

    document.body.removeChild(input);
  });

  it("selects all text on Select All click and closes menu", async () => {
    render(ContextMenu);
    const input = document.createElement("input");
    input.value = "test string";
    const selectSpy = vi.spyOn(input, "select");
    document.body.appendChild(input);

    openContextMenu({
      x: 100,
      y: 150,
      target: input,
      hasSelection: false,
      hasContent: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("context-menu-select-all")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("context-menu-select-all"));
    expect(selectSpy).toHaveBeenCalled();

    await waitFor(() => {
      expect(screen.queryByTestId("text-context-menu")).toBeNull();
    });

    document.body.removeChild(input);
  });

  it("closes on Escape key press", async () => {
    render(ContextMenu);
    openContextMenu({
      x: 100,
      y: 150,
      target: null,
    });

    await waitFor(() => {
      expect(screen.getByTestId("text-context-menu")).toBeInTheDocument();
    });

    await fireEvent.keyDown(window, { key: "Escape" });

    await waitFor(() => {
      expect(screen.queryByTestId("text-context-menu")).toBeNull();
    });
  });
});
