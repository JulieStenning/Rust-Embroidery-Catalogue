// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import TagWordMatchModal from "../TagWordMatchModal.svelte";
import * as commandAdapter from "../../api/commandAdapter";

vi.mock("../../api/commandAdapter", () => ({
  listTagSynonymsGrouped: vi.fn(),
  addTagSynonyms: vi.fn(),
  deleteTagSynonym: vi.fn(),
  deleteAllTagSynonymsForTag: vi.fn(),
}));

describe("TagWordMatchModal.svelte", () => {
  const sampleGroups = [
    {
      tag_id: 10,
      tag_description: "Animals",
      tag_group: "image",
      keywords: [
        { id: 101, keyword: "frog" },
        { id: 102, keyword: "bear" },
      ],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commandAdapter.listTagSynonymsGrouped).mockResolvedValue({
      source: "rust",
      items: sampleGroups,
    });
    vi.mocked(commandAdapter.addTagSynonyms).mockResolvedValue({
      source: "rust",
      persisted: true,
      item: [
        { id: 101, keyword: "frog" },
        { id: 102, keyword: "bear" },
        { id: 103, keyword: "cat" },
      ],
    });
    vi.mocked(commandAdapter.deleteTagSynonym).mockResolvedValue({
      source: "rust",
      persisted: true,
    });
    vi.mocked(commandAdapter.deleteAllTagSynonymsForTag).mockResolvedValue({
      source: "rust",
      persisted: true,
    });
  });

  it("renders nothing when open is false", () => {
    render(TagWordMatchModal, {
      props: {
        open: false,
        tagId: 10,
        tagDescription: "Animals",
        tagGroup: "image",
        onClose: vi.fn(),
      },
    });

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("renders modal with existing keywords when open is true", async () => {
    render(TagWordMatchModal, {
      props: {
        open: true,
        tagId: 10,
        tagDescription: "Animals",
        tagGroup: "image",
        onClose: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    expect(screen.getByText("Animals")).toBeInTheDocument();
    expect(screen.getByText("frog")).toBeInTheDocument();
    expect(screen.getByText("bear")).toBeInTheDocument();
  });

  it("adds new keywords and refreshes words list", async () => {
    const onMatchesChanged = vi.fn();
    render(TagWordMatchModal, {
      props: {
        open: true,
        tagId: 10,
        tagDescription: "Animals",
        tagGroup: "image",
        onClose: vi.fn(),
        onMatchesChanged,
      },
    });

    await waitFor(() => {
      expect(screen.getByText("frog")).toBeInTheDocument();
    });

    const input = screen.getByPlaceholderText(/e\.g\. frog, toad, newt/i);
    await fireEvent.input(input, { target: { value: "cat, dog" } });
    await tick();

    const addBtn = screen.getByRole("button", { name: "Add" });
    await fireEvent.click(addBtn);
    await tick();

    expect(commandAdapter.addTagSynonyms).toHaveBeenCalledWith(10, "cat, dog");
    expect(onMatchesChanged).toHaveBeenCalled();
  });

  it("deletes a single keyword when clicking remove button", async () => {
    const onMatchesChanged = vi.fn();
    render(TagWordMatchModal, {
      props: {
        open: true,
        tagId: 10,
        tagDescription: "Animals",
        tagGroup: "image",
        onClose: vi.fn(),
        onMatchesChanged,
      },
    });

    await waitFor(() => {
      expect(screen.getByText("frog")).toBeInTheDocument();
    });

    const removeFrogBtn = screen.getByLabelText("Remove frog");
    await fireEvent.click(removeFrogBtn);
    await tick();

    expect(commandAdapter.deleteTagSynonym).toHaveBeenCalledWith(101);
    expect(onMatchesChanged).toHaveBeenCalled();
  });

  it("calls onClose when clicking Done button", async () => {
    const onClose = vi.fn();
    render(TagWordMatchModal, {
      props: {
        open: true,
        tagId: 10,
        tagDescription: "Animals",
        tagGroup: "image",
        onClose,
      },
    });

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    const doneBtn = screen.getByRole("button", { name: "Done" });
    await fireEvent.click(doneBtn);
    await tick();

    expect(onClose).toHaveBeenCalled();
  });
});
