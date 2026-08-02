import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import TaggingActionsView from "../TaggingActionsView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getTaggingActionsViewModel: vi.fn(),
  runUnifiedBackfill: vi.fn(),
  stopUnifiedBackfill: vi.fn(),
  getBackfillLogEntries: vi.fn(),
  runStitchingBackfill: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast() on mount.
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

const viewModel = () => ({
  source: "rust",
  model: {
    has_google_api_key: false,
    tier2_default: false,
    tier3_default: false,
  },
});

describe("TaggingActionsView checkbox interactions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
  });

  it("toggles the action mode between tag_untagged and tag_all", async () => {
    render(TaggingActionsView);

    const taggingCheckbox = await screen.findByRole("checkbox", {
      name: /Tagging/,
    });
    expect(taggingCheckbox).toBeChecked();

    const user = userEvent.setup();
    await user.click(taggingCheckbox);
    expect(taggingCheckbox).not.toBeChecked();

    await user.click(taggingCheckbox);
    expect(taggingCheckbox).toBeChecked();
  });

  it("toggles the AI tier checkboxes", async () => {
    render(TaggingActionsView);

    const tier2 = await screen.findByRole("checkbox", { name: /Run Tier 2/ });
    const tier3 = screen.getByRole("checkbox", { name: /Run Tier 3/ });
    expect(tier2).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(tier2);
    expect(tier2).toBeChecked();

    await user.click(tier3);
    expect(tier3).toBeChecked();
  });

  it("toggles the stitching and clear-existing-stitching checkboxes", async () => {
    render(TaggingActionsView);

    const stitching = await screen.findByRole("checkbox", {
      name: /Stitching tag detection/,
    });
    const clearExisting = screen.getByRole("checkbox", {
      name: /Clear existing stitching tags/,
    });
    expect(stitching).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(stitching);
    expect(stitching).toBeChecked();

    await user.click(clearExisting);
    expect(clearExisting).toBeChecked();
  });

  it("toggles the image redo checkbox", async () => {
    render(TaggingActionsView);

    await screen.findByRole("checkbox", { name: /Image generation/ });
    const imageRedo = screen.getByRole("checkbox", {
      name: /Regenerate images/,
    });

    expect(imageRedo).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(imageRedo);
    expect(imageRedo).toBeChecked();
  });

  it("toggles the colour count checkbox", async () => {
    render(TaggingActionsView);

    const colourCounts = await screen.findByRole("checkbox", {
      name: /Recalculate colour/,
    });
    expect(colourCounts).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(colourCounts);
    expect(colourCounts).toBeChecked();
  });
});