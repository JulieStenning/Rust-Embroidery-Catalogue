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
  countTaggingCandidates: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast() on mount.
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

// The view installs a backfill-progress listener on mount. Provide a benign
// mock so it resolves cleanly and never touches real Tauri internals in jsdom.
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

const viewModel = (overrides = {}) => ({
  source: "rust",
  model: {
    has_google_api_key: true,
    ai_vision_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    import_commit_batch_size: "",
    default_batch_size: 100,
    default_commit_every: 100,
    default_workers: 4,
    ...overrides,
  },
});

describe("TaggingActionsView workflow selection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
  });

  it("defaults to File & Folder, untagged, and add-new-tags", async () => {
    render(TaggingActionsView);

    expect(
      await screen.findByRole("radio", { name: /Apply File & Folder Rules/ })
    ).toBeChecked();
    expect(screen.getByRole("radio", { name: /Untagged designs only/ })).toBeChecked();
    expect(screen.getByRole("radio", { name: /Add New Tags Only/ })).toBeChecked();
  });

  it("selects a Visual AI goal and changes the scope and merge", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    const user = userEvent.setup();
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    expect(screen.getByRole("radio", { name: /Enrich with Visual AI/ })).toBeChecked();

    await user.click(
      screen.getByRole("radio", { name: /Designs missing Visual AI analysis/ })
    );
    expect(screen.getByRole("radio", { name: /Designs missing Visual AI analysis/ })).toBeChecked();

    await user.click(screen.getByRole("radio", { name: /Complete Reset/ }));
    expect(screen.getByRole("radio", { name: /Complete Reset/ })).toBeChecked();
  });

  it("shows the folder scope option as disabled (coming soon)", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    expect(screen.getByText(/Specific Folder or Category/)).toBeInTheDocument();
    expect(screen.getByText(/coming soon/)).toBeInTheDocument();
  });

  it("disables Visual AI goals without an API key", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(
      viewModel({ has_google_api_key: false })
    );
    render(TaggingActionsView);

    expect(
      await screen.findByRole("radio", { name: /Enrich with Visual AI/ })
    ).toBeDisabled();
    expect(screen.getByRole("radio", { name: /Both Methods/ })).toBeDisabled();
  });

  it("defaults to excluding verified designs and shows the unverified/verified breakdown", async () => {
    render(TaggingActionsView);

    const toggle = await screen.findByRole("checkbox", {
      name: /Exclude human-verified designs/,
    });
    expect(toggle).toBeChecked();

    // With the toggle checked the primary badge shows unverified_count (10) and
    // every scope card renders the breakdown text.
    await screen.findAllByText("10 unverified · 2 verified");
    expect(screen.getAllByText("10 designs").length).toBeGreaterThan(0);
  });

  it("switches the active count to the total when exclusion is unchecked", async () => {
    render(TaggingActionsView);

    const toggle = await screen.findByRole("checkbox", {
      name: /Exclude human-verified designs/,
    });
    await screen.findAllByText("10 unverified · 2 verified");

    const user = userEvent.setup();
    await user.click(toggle);
    expect(toggle).not.toBeChecked();

    // Now the primary badge shows total_count (12).
    expect(screen.getAllByText("12 designs").length).toBeGreaterThan(0);
    // Breakdown stays visible regardless of the toggle.
    expect(screen.getAllByText("10 unverified · 2 verified").length).toBeGreaterThan(0);
  });
});

describe("TaggingActionsView advanced options", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
  });

  it("toggles the stitching checkbox and its overwrite sub-option", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();

    const stitching = screen.getByRole("checkbox", { name: /Also detect stitching tags/ });
    expect(stitching).not.toBeChecked();
    await user.click(stitching);
    expect(stitching).toBeChecked();

    const overwrite = screen.getByRole("checkbox", {
      name: /Overwrite stitching tags on already-processed designs/,
    });
    await user.click(overwrite);
    expect(overwrite).toBeChecked();
  });

  it("toggles the image generation checkbox and its redo sub-option", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();

    const images = screen.getByRole("checkbox", { name: /Also generate preview images/ });
    await user.click(images);
    expect(images).toBeChecked();

    const redo = screen.getByRole("checkbox", { name: /Regenerate images for all designs/ });
    await user.click(redo);
    expect(redo).toBeChecked();
  });

  it("toggles the colour count and hoop dimension checkboxes", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();

    const colour = screen.getByRole("checkbox", { name: /Recalculate colour/ });
    await user.click(colour);
    expect(colour).toBeChecked();

    const hoops = screen.getByRole("checkbox", { name: /Recalculate hoops/ });
    await user.click(hoops);
    expect(hoops).toBeChecked();
  });
});