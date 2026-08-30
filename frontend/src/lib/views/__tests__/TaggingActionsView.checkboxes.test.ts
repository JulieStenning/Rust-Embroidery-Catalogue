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

describe("TaggingActionsView checkbox interactions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
  });

  it("toggles the Tagging checkbox between unchecked and checked", async () => {
    render(TaggingActionsView);

    const taggingCheckbox = await screen.findByRole("checkbox", {
      name: /Tagging/,
    });
    expect(taggingCheckbox).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(taggingCheckbox);
    expect(taggingCheckbox).toBeChecked();

    await user.click(taggingCheckbox);
    expect(taggingCheckbox).not.toBeChecked();
  });

  it("toggles the retag-all sub-checkbox when Tagging is enabled", async () => {
    render(TaggingActionsView);

    const tagging = await screen.findByRole("checkbox", { name: /Tagging/ });
    const retagAll = screen.getByRole("checkbox", {
      name: /Re-tag designs that already have tags/,
    });
    // Sub-option is disabled until the parent is checked.
    expect(retagAll).toBeDisabled();

    const user = userEvent.setup();
    await user.click(tagging);
    expect(retagAll).not.toBeDisabled();

    await user.click(retagAll);
    expect(retagAll).toBeChecked();

    await user.click(retagAll);
    expect(retagAll).not.toBeChecked();
  });

  it("toggles the Visual AI checkbox when Tagging is enabled and an API key is set", async () => {
    render(TaggingActionsView);

    const tagging = await screen.findByRole("checkbox", { name: /Tagging/ });
    const vision = screen.getByRole("checkbox", { name: /Run Visual AI/ });
    expect(vision).toBeDisabled();

    const user = userEvent.setup();
    await user.click(tagging);

    expect(vision).not.toBeDisabled();

    await user.click(vision);
    expect(vision).toBeChecked();
  });

  it("disables and unchecks Visual AI when no API key is set", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(
      viewModel({ has_google_api_key: false })
    );
    render(TaggingActionsView);

    const tagging = await screen.findByRole("checkbox", { name: /Tagging/ });
    const vision = screen.getByRole("checkbox", { name: /Run Visual AI/ });
    expect(vision).toBeDisabled();
    expect(vision).not.toBeChecked();

    // Toggling Tagging on does not enable it without a key.
    const user = userEvent.setup();
    await user.click(tagging);
    expect(vision).toBeDisabled();
  });

  it("toggles the stitching and overwrite sub-checkboxes", async () => {
    render(TaggingActionsView);

    const stitching = await screen.findByRole("checkbox", {
      name: /Stitching tag detection/,
    });
    const overwrite = screen.getByRole("checkbox", {
      name: /Overwrite stitching tags on designs that have already been processed/,
    });
    expect(stitching).not.toBeChecked();
    expect(overwrite).toBeDisabled();

    const user = userEvent.setup();
    await user.click(stitching);
    expect(stitching).toBeChecked();
    expect(overwrite).not.toBeDisabled();

    await user.click(overwrite);
    expect(overwrite).toBeChecked();
  });

  it("toggles the image redo sub-checkbox when Image generation is enabled", async () => {
    render(TaggingActionsView);

    const images = await screen.findByRole("checkbox", {
      name: /Image generation/,
    });
    const imageRedo = screen.getByRole("checkbox", {
      name: /Regenerate images/,
    });
    expect(images).not.toBeChecked();
    expect(imageRedo).toBeDisabled();

    const user = userEvent.setup();
    await user.click(images);
    expect(images).toBeChecked();
    expect(imageRedo).not.toBeDisabled();

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

  it("toggles the hoops / dimensions checkbox", async () => {
    render(TaggingActionsView);

    const hoops = await screen.findByRole("checkbox", {
      name: /Recalculate hoops/,
    });
    expect(hoops).not.toBeChecked();

    const user = userEvent.setup();
    await user.click(hoops);
    expect(hoops).toBeChecked();
  });
});
