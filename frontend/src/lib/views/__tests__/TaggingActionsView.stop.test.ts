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

describe("TaggingActionsView stop behaviour", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.stopUnifiedBackfill.mockResolvedValue({
      source: "rust",
      stopRequested: true,
      message: "Stop requested.",
    });
  });

  /**
   * Renders the view, enables Tier 2 and starts a never-resolving backfill
   * so the Run button enters its in-flight state.
   */
  async function startInFlightRun() {
    adapterMocks.runUnifiedBackfill.mockReturnValue(new Promise(() => {}));
    render(TaggingActionsView);
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run selected actions" })
      ).not.toBeDisabled();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(
      screen.getByRole("button", { name: "Run selected actions" })
    );

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Running..." })
      ).toBeDisabled();
    });
  }

  it("disables Run and enables Stop while a backfill is in flight", async () => {
    await startInFlightRun();

    expect(
      screen.getByRole("button", { name: "Running..." })
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Stop" })).not.toBeDisabled();
  });

  it("requests a stop via stopUnifiedBackfill", async () => {
    await startInFlightRun();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Stop" }));

    await waitFor(() => {
      expect(adapterMocks.stopUnifiedBackfill).toHaveBeenCalledTimes(1);
    });
    expect(toastMock.addToast).toHaveBeenCalledWith(
      "Stop requested.",
      "info"
    );
  });

  it("shows an error toast when the stop request fails", async () => {
    adapterMocks.stopUnifiedBackfill.mockRejectedValue(
      new Error("stop failed")
    );
    await startInFlightRun();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Stop" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Stop request failed: Error: stop failed",
        "error"
      );
    });
  });
});