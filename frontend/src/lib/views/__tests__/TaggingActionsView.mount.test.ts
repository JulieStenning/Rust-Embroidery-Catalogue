import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
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

/** Helper that constructs the tagging actions view model returned by the adapter. */
const viewModel = (overrides = {}) => ({
  source: "rust",
  model: {
    has_google_api_key: false,
    tier2_default: false,
    tier3_default: false,
    ...overrides,
  },
});

describe("TaggingActionsView mount behaviour", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
  });

  it("loads the tagging view model and backfill log entries on mount", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(adapterMocks.getTaggingActionsViewModel).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(adapterMocks.getBackfillLogEntries).toHaveBeenCalledTimes(1);
    });
  });

  it("shows the no-API-key notice and info toast when no key is set", async () => {
    render(TaggingActionsView);

    expect(
      await screen.findByText(
        /No Google API key is configured in Settings\. AI tagging actions will be skipped\./
      )
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "No Google API key set. AI tagging actions will be skipped.",
        "info"
      );
    });
  });

  it("shows the API-key notice and info toast when a key is set", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(
      viewModel({ has_google_api_key: true })
    );
    render(TaggingActionsView);

    expect(
      await screen.findByText(/API key detected — AI tagging actions are available\./)
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "API key detected. AI tagging actions are available.",
        "info"
      );
    });
  });

  it("shows an error toast when the view model fails to load", async () => {
    adapterMocks.getTaggingActionsViewModel.mockRejectedValue(
      new Error("no backend")
    );
    render(TaggingActionsView);

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Could not load tagging action defaults: Error: no backend",
        "error"
      );
    });
  });
});

describe("TaggingActionsView initial render", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
  });

  it("renders the page title and subtitle", () => {
    render(TaggingActionsView);

    expect(
      screen.getByRole("heading", { name: "Tagging Actions" })
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /Run bulk tagging, image generation, or stitching calculation actions/
      )
    ).toBeInTheDocument();
  });

  it("renders the default checkbox states", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("checkbox", { name: /Tagging/ })).toBeChecked();
    });
    expect(
      screen.getByRole("checkbox", { name: /Run Tier 2/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Run Tier 3/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Stitching tag detection/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Clear existing stitching tags/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Image generation/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Regenerate images/ })
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Recalculate colour/ })
    ).not.toBeChecked();
  });

  it("disables the Run button while the view model is loading", () => {
    adapterMocks.getTaggingActionsViewModel.mockReturnValue(
      new Promise(() => {})
    );
    render(TaggingActionsView);

    expect(
      screen.getByRole("button", { name: "Run selected actions" })
    ).toBeDisabled();
  });

  it("disables the Stop button when no backfill is in flight", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Stop" })).toBeDisabled();
    });
    expect(
      screen.getByRole("button", { name: "Run selected actions" })
    ).not.toBeDisabled();
  });
});