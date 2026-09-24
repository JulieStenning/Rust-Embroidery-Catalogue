// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import TagWordMatchesView from "../TagWordMatchesView.svelte";
import * as commandAdapter from "../../api/commandAdapter";

vi.mock("../../api/commandAdapter", () => ({
  listTags: vi.fn(),
  listTagSynonymsGrouped: vi.fn(),
  addTagSynonyms: vi.fn(),
  deleteTagSynonym: vi.fn(),
  deleteAllTagSynonymsForTag: vi.fn(),
}));

describe("TagWordMatchesView.svelte", () => {
  const sampleTags = [
    { id: 1, description: "Animals", tag_group: "image", design_count: 5, is_system: false },
    { id: 2, description: "Floral", tag_group: "image", design_count: 12, is_system: false },
    { id: 3, description: "Dense Fill", tag_group: "stitching", design_count: 2, is_system: false },
  ];

  const sampleGroups = [
    {
      tag_id: 1,
      tag_description: "Animals",
      tag_group: "image",
      keywords: [
        { id: 101, keyword: "frog" },
        { id: 102, keyword: "bear" },
      ],
    },
    {
      tag_id: 3,
      tag_description: "Dense Fill",
      tag_group: "stitching",
      keywords: [{ id: 103, keyword: "heavy" }],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commandAdapter.listTags).mockResolvedValue({
      source: "rust",
      items: sampleTags,
    });
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
        { id: 104, keyword: "lion" },
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

  it("renders heading, search filters, and grouped word matches", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Tag Word Matches" })).toBeInTheDocument();
    });

    expect(screen.getByText("Animals")).toBeInTheDocument();
    expect(screen.getByText("frog")).toBeInTheDocument();
    expect(screen.getByText("bear")).toBeInTheDocument();
    expect(screen.getByText("Dense Fill")).toBeInTheDocument();
    expect(screen.getByText("heavy")).toBeInTheDocument();
  });

  it("filters displayed groups when typing in filter search box", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByText("Animals")).toBeInTheDocument();
    });

    const filterInput = screen.getByPlaceholderText(/filter tags or keywords/i);
    await fireEvent.input(filterInput, { target: { value: "heavy" } });
    await tick();

    expect(screen.getByText("Dense Fill")).toBeInTheDocument();
    expect(screen.queryByText("Animals")).not.toBeInTheDocument();
  });

  it("filters displayed groups by tag group (image vs stitching)", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByText("Animals")).toBeInTheDocument();
    });

    const stitchingBtn = screen.getByRole("button", { name: "Stitching" });
    await fireEvent.click(stitchingBtn);
    await tick();

    expect(screen.getByText("Dense Fill")).toBeInTheDocument();
    expect(screen.queryByText("Animals")).not.toBeInTheDocument();
  });

  it("deletes a word match chip when clicking remove", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByText("frog")).toBeInTheDocument();
    });

    const removeBtn = screen.getByLabelText("Remove frog");
    await fireEvent.click(removeBtn);
    await tick();

    expect(commandAdapter.deleteTagSynonym).toHaveBeenCalledWith(101);
  });

  it("displays existing matches in the Quick Add card when selecting a tag", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Tag Word Matches" })).toBeInTheDocument();
    });

    // Initially no quick add existing matches section
    expect(screen.queryByTestId("quick-add-existing-matches")).not.toBeInTheDocument();

    // Select Animals tag in the quick add combobox
    const comboboxInput = screen.getByPlaceholderText("Search for a tag...");
    await fireEvent.focus(comboboxInput);
    await tick();

    const animalsOption = screen
      .getAllByRole("option")
      .find((el) => el.textContent?.includes("Animals"));
    expect(animalsOption).toBeDefined();
    await fireEvent.click(animalsOption!);
    await tick();

    // Now quick-add-existing-matches should be rendered
    const section = screen.getByTestId("quick-add-existing-matches");
    expect(section).toBeInTheDocument();
    expect(section).toHaveTextContent('Existing matches for "Animals"');
    expect(section).toHaveTextContent("frog");
    expect(section).toHaveTextContent("bear");
  });

  it("shows duplicate warning when typing existing words in Quick Add input", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Tag Word Matches" })).toBeInTheDocument();
    });

    // Select Animals tag
    const comboboxInput = screen.getByPlaceholderText("Search for a tag...");
    await fireEvent.focus(comboboxInput);
    await tick();

    const animalsOption = screen
      .getAllByRole("option")
      .find((el) => el.textContent?.includes("Animals"));
    await fireEvent.click(animalsOption!);
    await tick();

    const wordsInput = screen.getByPlaceholderText("Enter words separated by commas...");
    await fireEvent.input(wordsInput, { target: { value: "frog, puppy" } });
    await tick();

    expect(screen.getByText(/Already added: frog/i)).toBeInTheDocument();
  });

  it("displays empty state message when selecting a tag with no existing matches", async () => {
    render(TagWordMatchesView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Tag Word Matches" })).toBeInTheDocument();
    });

    // Select Floral tag (tag_id: 2 has no group in sampleGroups)
    const comboboxInput = screen.getByPlaceholderText("Search for a tag...");
    await fireEvent.focus(comboboxInput);
    await tick();

    const floralOption = screen
      .getAllByRole("option")
      .find((el) => el.textContent?.includes("Floral"));
    await fireEvent.click(floralOption!);
    await tick();

    const section = screen.getByTestId("quick-add-existing-matches");
    expect(section).toBeInTheDocument();
    expect(section).toHaveTextContent('No word matches configured yet for "Floral"');
  });
});
