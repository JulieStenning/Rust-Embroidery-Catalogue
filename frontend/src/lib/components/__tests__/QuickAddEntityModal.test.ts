// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import QuickAddEntityModal from "../QuickAddEntityModal.svelte";

const adapterMocks = vi.hoisted(() => ({
  createDesigner: vi.fn(),
  createSource: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

const toastMocks = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMocks);

describe("QuickAddEntityModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders closed when open=false", () => {
    render(QuickAddEntityModal, { open: false });
    expect(screen.queryByTestId("quick-add-entity-modal")).not.toBeInTheDocument();
  });

  it("renders designer mode correctly when open=true", () => {
    render(QuickAddEntityModal, { open: true, entityType: "designer" });
    expect(screen.getByRole("heading", { name: "Add New Designer" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("e.g. Urban Threads")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Designer" })).toBeInTheDocument();
  });

  it("renders source mode correctly when open=true", () => {
    render(QuickAddEntityModal, { open: true, entityType: "source" });
    expect(screen.getByRole("heading", { name: "Add New Source" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("e.g. Purchased")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Source" })).toBeInTheDocument();
  });

  it("validates empty or whitespace input", async () => {
    render(QuickAddEntityModal, { open: true, entityType: "designer" });
    const submitBtn = screen.getByRole("button", { name: "Add Designer" });
    expect(submitBtn).toBeDisabled();
  });

  it("validates duplicate names client-side", async () => {
    render(QuickAddEntityModal, {
      open: true,
      entityType: "designer",
      existingNames: ["Urban Threads", "Rose Studio"],
    });

    const input = screen.getByPlaceholderText("e.g. Urban Threads");
    await fireEvent.input(input, { target: { value: "urban threads" } });

    const submitBtn = screen.getByRole("button", { name: "Add Designer" });
    expect(submitBtn).toBeEnabled();
    await fireEvent.click(submitBtn);

    expect(screen.getByRole("alert")).toHaveTextContent(
      'A designer named "urban threads" already exists.'
    );
    expect(adapterMocks.createDesigner).not.toHaveBeenCalled();
  });

  it("creates a designer successfully and calls onCreated and onClose", async () => {
    adapterMocks.createDesigner.mockResolvedValueOnce({
      source: "rust",
      persisted: true,
      item: { id: 42, name: "New Designer Studio", design_count: 0 },
    });

    const onCreated = vi.fn();
    const onClose = vi.fn();

    render(QuickAddEntityModal, {
      open: true,
      entityType: "designer",
      existingNames: ["Existing Studio"],
      onCreated,
      onClose,
    });

    const input = screen.getByPlaceholderText("e.g. Urban Threads");
    await fireEvent.input(input, { target: { value: "New Designer Studio" } });

    const submitBtn = screen.getByRole("button", { name: "Add Designer" });
    await fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(adapterMocks.createDesigner).toHaveBeenCalledWith("New Designer Studio");
      expect(onCreated).toHaveBeenCalledWith({ id: 42, name: "New Designer Studio" });
      expect(onClose).toHaveBeenCalled();
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        'Designer "New Designer Studio" added.',
        "success"
      );
    });
  });

  it("creates a source successfully and calls onCreated and onClose", async () => {
    adapterMocks.createSource.mockResolvedValueOnce({
      source: "rust",
      persisted: true,
      item: { id: 15, name: "Special Download", design_count: 0 },
    });

    const onCreated = vi.fn();
    const onClose = vi.fn();

    render(QuickAddEntityModal, {
      open: true,
      entityType: "source",
      existingNames: [],
      onCreated,
      onClose,
    });

    const input = screen.getByPlaceholderText("e.g. Purchased");
    await fireEvent.input(input, { target: { value: "Special Download" } });

    const submitBtn = screen.getByRole("button", { name: "Add Source" });
    await fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(adapterMocks.createSource).toHaveBeenCalledWith("Special Download");
      expect(onCreated).toHaveBeenCalledWith({ id: 15, name: "Special Download" });
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("handles cancel button click", async () => {
    const onClose = vi.fn();
    render(QuickAddEntityModal, { open: true, onClose });

    const cancelBtn = screen.getByRole("button", { name: "Cancel" });
    await fireEvent.click(cancelBtn);

    expect(onClose).toHaveBeenCalled();
  });
});
