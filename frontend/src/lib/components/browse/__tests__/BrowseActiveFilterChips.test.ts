// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import BrowseActiveFilterChips from "../BrowseActiveFilterChips.svelte";
import type { BrowseFilterState } from "../../../types/ipc";

import { HOOP_UNKNOWN_FILTER } from "../../../utils/hoopConstants.js";

function createDefaultFilters(overrides: Partial<BrowseFilterState> = {}): BrowseFilterState {
  return {
    q: "",
    allWords: "",
    exactPhrase: "",
    anyWords: "",
    noneWords: "",
    filename: "",
    designerFilters: [],
    imageTagFilters: [],
    stitchingTagFilters: [],
    hoop: "",
    minWidth: "",
    maxWidth: "",
    minHeight: "",
    maxHeight: "",
    sourceFilters: [],
    rating: "",
    stitched: "",
    unverifiedOnly: false,
    verificationStatus: "all",
    imageTagVerification: "all",
    stitchingTagVerification: "all",
    needsAttention: "off",
    searchFilename: true,
    searchTags: true,
    searchFolder: true,
    sortBy: "date_added",
    sortDir: "desc",
    ...overrides,
  };
}

describe("BrowseActiveFilterChips.svelte", () => {
  it("renders nothing when all filters are at default values", () => {
    const filters = createDefaultFilters();
    const { container } = render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter: vi.fn(),
        onToggleFilter: vi.fn(),
        onClearFilters: vi.fn(),
      },
    });

    expect(container.querySelector('[data-testid="active-filter-chips"]')).toBeNull();
  });

  it("renders chips for active designer, tag, and source filters", () => {
    const filters = createDefaultFilters({
      designerFilters: ["Urban Threads", "Anita Goodesign"],
      imageTagFilters: ["Floral"],
      stitchingTagFilters: ["Cross Stitch"],
      sourceFilters: ["CD Box"],
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter: vi.fn(),
        onToggleFilter: vi.fn(),
        onClearFilters: vi.fn(),
      },
    });

    expect(screen.getByText("Designer: Urban Threads")).toBeInTheDocument();
    expect(screen.getByText("Designer: Anita Goodesign")).toBeInTheDocument();
    expect(screen.getByText("Image tag: Floral")).toBeInTheDocument();
    expect(screen.getByText("Stitching tag: Cross Stitch")).toBeInTheDocument();
    expect(screen.getByText("Source: CD Box")).toBeInTheDocument();
  });

  it("calls onToggleFilter when removing a designer chip", async () => {
    const onToggleFilter = vi.fn();
    const filters = createDefaultFilters({
      designerFilters: ["Urban Threads"],
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter: vi.fn(),
        onToggleFilter,
        onClearFilters: vi.fn(),
      },
    });

    const removeBtn = screen.getByTestId("active-filter-remove-designer-Urban Threads");
    await fireEvent.click(removeBtn);

    expect(onToggleFilter).toHaveBeenCalledWith("designerFilters", "Urban Threads");
  });

  it("renders chips for hoop and dimension bounds and invokes onUpdateFilter on dismissal", async () => {
    const onUpdateFilter = vi.fn();
    const filters = createDefaultFilters({
      hoop: "100x100",
      minWidth: "50",
      maxWidth: "120",
      minHeight: "40",
      maxHeight: "90",
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter,
        onToggleFilter: vi.fn(),
        onClearFilters: vi.fn(),
      },
    });

    expect(screen.getByText("Hoop: 100x100")).toBeInTheDocument();
    expect(screen.getByText("Width ≥ 50mm")).toBeInTheDocument();
    expect(screen.getByText("Width ≤ 120mm")).toBeInTheDocument();
    expect(screen.getByText("Height ≥ 40mm")).toBeInTheDocument();
    expect(screen.getByText("Height ≤ 90mm")).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("active-filter-remove-hoop"));
    expect(onUpdateFilter).toHaveBeenCalledWith("hoop", "");

    await fireEvent.click(screen.getByTestId("active-filter-remove-min-width"));
    expect(onUpdateFilter).toHaveBeenCalledWith("minWidth", "");
  });

  it("renders unknown hoop correctly", () => {
    const filters = createDefaultFilters({
      hoop: HOOP_UNKNOWN_FILTER,
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter: vi.fn(),
        onToggleFilter: vi.fn(),
        onClearFilters: vi.fn(),
      },
    });

    expect(screen.getByText("Hoop: Hoop unknown")).toBeInTheDocument();
  });

  it("renders status chips (rating, stitched, verification, needsAttention)", async () => {
    const onUpdateFilter = vi.fn();
    const filters = createDefaultFilters({
      rating: "4",
      stitched: "yes",
      imageTagVerification: "verified",
      stitchingTagVerification: "unverified",
      needsAttention: "cannot_draw",
      verificationStatus: "partial",
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter,
        onToggleFilter: vi.fn(),
        onClearFilters: vi.fn(),
      },
    });

    expect(screen.getByText("Rating: ≥ 4★")).toBeInTheDocument();
    expect(screen.getByText("Stitched: Yes")).toBeInTheDocument();
    expect(screen.getByText("Image tags: Verified")).toBeInTheDocument();
    expect(screen.getByText("Stitching tags: Unverified")).toBeInTheDocument();
    expect(screen.getByText("Needs attention: Cannot be drawn")).toBeInTheDocument();
    expect(screen.getByText("Verification: Partially verified")).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("active-filter-remove-rating"));
    expect(onUpdateFilter).toHaveBeenCalledWith("rating", "");

    await fireEvent.click(screen.getByTestId("active-filter-remove-stitched"));
    expect(onUpdateFilter).toHaveBeenCalledWith("stitched", "");

    await fireEvent.click(screen.getByTestId("active-filter-remove-needs-attention"));
    expect(onUpdateFilter).toHaveBeenCalledWith("needsAttention", "off");

    await fireEvent.click(screen.getByTestId("active-filter-remove-verification-status"));
    expect(onUpdateFilter).toHaveBeenCalledWith("verificationStatus", "all");
    expect(onUpdateFilter).toHaveBeenCalledWith("unverifiedOnly", false);
  });

  it("calls onClearFilters when clicking Clear all", async () => {
    const onClearFilters = vi.fn();
    const filters = createDefaultFilters({
      designerFilters: ["Urban Threads"],
    });

    render(BrowseActiveFilterChips, {
      props: {
        browseFilters: filters,
        onUpdateFilter: vi.fn(),
        onToggleFilter: vi.fn(),
        onClearFilters,
      },
    });

    const clearBtn = screen.getByTestId("active-filters-clear-all");
    await fireEvent.click(clearBtn);

    expect(onClearFilters).toHaveBeenCalledOnce();
  });
});
