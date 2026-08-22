import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import SelectionHeader from "../SelectionHeader.svelte";

// ---------------------------------------------------------------------------
// SelectionHeader.svelte — presentational header row for the Browse page.
//
// Behaviour under test:
//   - Renders the total filtered count (singular vs. plural text).
//   - Renders how many of the current page's rows are selected.
//   - "Select all on page" checkbox reflects `isAllSelectedOnPage`.
//   - Checkbox is disabled when the page has no rows (totalCountOnPage === 0).
//   - Flipping the checkbox fires `onToggleSelectAllPage` with the new value.
// ---------------------------------------------------------------------------

interface SelectionHeaderProps {
  totalFilteredCount?: number;
  selectedCountOnPage?: number;
  totalCountOnPage?: number;
  isAllSelectedOnPage?: boolean;
  onToggleSelectAllPage?: (checked: boolean) => void;
}

function renderHeader(props: SelectionHeaderProps = {}) {
  const base = {
    totalFilteredCount: 5,
    selectedCountOnPage: 2,
    totalCountOnPage: 10,
    isAllSelectedOnPage: false,
    onToggleSelectAllPage: vi.fn(),
  };
  return render(SelectionHeader, { props: { ...base, ...props } });
}

describe("SelectionHeader.svelte", () => {
  it("renders default states when no props are supplied", () => {
    render(SelectionHeader);

    expect(screen.getByText("0 designs found")).toBeInTheDocument();
    expect(screen.getByText("0 of 0 selected")).toBeInTheDocument();
    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox).not.toBeChecked();
    expect(checkbox).toBeDisabled();
  });

  it("renders the plural total-count text", () => {
    renderHeader({ totalFilteredCount: 5 });

    expect(screen.getByText("5 designs found")).toBeInTheDocument();
  });

  it("renders the singular total-count text when exactly one design is found", () => {
    renderHeader({ totalFilteredCount: 1 });

    expect(screen.getByText("1 design found")).toBeInTheDocument();
    expect(screen.queryByText("1 designs found")).not.toBeInTheDocument();
  });

  it("renders the selected-on-page summary", () => {
    renderHeader({ selectedCountOnPage: 3, totalCountOnPage: 10 });

    expect(screen.getByText("3 of 10 selected")).toBeInTheDocument();
  });

  it("reflects isAllSelectedOnPage=true as a checked checkbox", () => {
    renderHeader({ isAllSelectedOnPage: true });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox).toBeChecked();
  });

  it("reflects isAllSelectedOnPage=false as an unchecked checkbox", () => {
    renderHeader({ isAllSelectedOnPage: false });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox).not.toBeChecked();
  });

  it("disables the checkbox when the page has no rows", () => {
    renderHeader({ totalCountOnPage: 0 });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox).toBeDisabled();
  });

  it("enables the checkbox when the page has at least one row", () => {
    renderHeader({ totalCountOnPage: 1 });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox).toBeEnabled();
  });

  it("calls onToggleSelectAllPage(true) when the checkbox is unchecked and clicked", async () => {
    const onToggle = vi.fn();
    renderHeader({ isAllSelectedOnPage: false, onToggleSelectAllPage: onToggle });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    await fireEvent.click(checkbox);
    await tick();

    expect(onToggle).toHaveBeenCalledWith(true);
  });

  it("calls onToggleSelectAllPage(false) when the checkbox is checked and clicked", async () => {
    const onToggle = vi.fn();
    renderHeader({ isAllSelectedOnPage: true, onToggleSelectAllPage: onToggle });

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    await fireEvent.click(checkbox);
    await tick();

    expect(onToggle).toHaveBeenCalledWith(false);
  });
});
