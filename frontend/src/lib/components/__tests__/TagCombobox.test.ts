// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import TagCombobox from "../TagCombobox.svelte";

describe("TagCombobox.svelte", () => {
  const sampleTags = [
    { id: 1, description: "Animals", tag_group: "image" },
    { id: 2, description: "Floral", tag_group: "image" },
    { id: 3, description: "Dense Fill", tag_group: "stitching" },
  ];

  it("renders input placeholder and filters options when typing", async () => {
    const onSelect = vi.fn();
    render(TagCombobox, {
      props: {
        tags: sampleTags,
        selectedTagId: null,
        onSelect,
        placeholder: "Search tags...",
      },
    });

    const input = screen.getByPlaceholderText("Search tags...");
    expect(input).toBeInTheDocument();

    await fireEvent.focus(input);
    await fireEvent.input(input, { target: { value: "anim" } });
    await tick();

    expect(screen.getByText("Animals")).toBeInTheDocument();
    expect(screen.queryByText("Floral")).not.toBeInTheDocument();
  });

  it("selects a tag on click", async () => {
    const onSelect = vi.fn();
    render(TagCombobox, {
      props: {
        tags: sampleTags,
        selectedTagId: null,
        onSelect,
      },
    });

    const input = screen.getByRole("combobox");
    await fireEvent.focus(input);
    await tick();

    const option = screen.getByText("Floral");
    await fireEvent.click(option);
    await tick();

    expect(onSelect).toHaveBeenCalledWith(sampleTags[1]);
  });

  it("filters tags by groupFilter prop", async () => {
    render(TagCombobox, {
      props: {
        tags: sampleTags,
        selectedTagId: null,
        groupFilter: "stitching",
        onSelect: vi.fn(),
      },
    });

    const input = screen.getByRole("combobox");
    await fireEvent.focus(input);
    await tick();

    expect(screen.getByText("Dense Fill")).toBeInTheDocument();
    expect(screen.queryByText("Animals")).not.toBeInTheDocument();
    expect(screen.queryByText("Floral")).not.toBeInTheDocument();
  });
});
