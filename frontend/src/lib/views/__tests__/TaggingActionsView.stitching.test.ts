import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
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

// Mock the toast store — the view calls addToast() on every run.
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

/** Helper that constructs a stitching backfill result. */
const backfillResult = (overrides = {}) => ({
  source: "rust",
  processed: 0,
  errors: 0,
  stopped: false,
  actions: [],
  ...overrides,
});

describe("TaggingActionsView run stitching backfill", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.runUnifiedBackfill.mockResolvedValue(backfillResult());
    adapterMocks.runStitchingBackfill.mockResolvedValue(backfillResult());
  });

  it("calls runStitchingBackfill with the stitching options when enabled", async () => {
    render(TaggingActionsView);
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run selected actions" })
      ).not.toBeDisabled();
    });

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Stitching tag detection/ })
    );
    await user.click(
      screen.getByRole("button", { name: "Run selected actions" })
    );

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalledWith({
        commit_every: 100,
        batch_size: 100,
        workers: 4,
        clear_existing: false,
        image_redo: false,
      });
    });
    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Stitching backfill complete.",
        "success"
      );
    });
  });

  it("passes clear_existing when clearing stitching tags is enabled", async () => {
    render(TaggingActionsView);
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run selected actions" })
      ).not.toBeDisabled();
    });

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Stitching tag detection/ })
    );
    await user.click(
      screen.getByRole("checkbox", { name: /Clear existing stitching tags/ })
    );
    await user.click(
      screen.getByRole("button", { name: "Run selected actions" })
    );

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ clear_existing: true })
      );
    });
  });

  it("shows an error toast when the stitching backfill reports an error", async () => {
    adapterMocks.runStitchingBackfill.mockResolvedValue(
      backfillResult({ error: "Malformed stitch data" })
    );
    render(TaggingActionsView);
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run selected actions" })
      ).not.toBeDisabled();
    });

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Stitching tag detection/ })
    );
    await user.click(
      screen.getByRole("button", { name: "Run selected actions" })
    );

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Stitching backfill failed: Malformed stitch data",
        "error"
      );
    });
  });

  it("runs both stitching and unified backfills when both are enabled", async () => {
    render(TaggingActionsView);
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run selected actions" })
      ).not.toBeDisabled();
    });

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Stitching tag detection/ })
    );
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(
      screen.getByRole("button", { name: "Run selected actions" })
    );

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalled();
    });
  });
});