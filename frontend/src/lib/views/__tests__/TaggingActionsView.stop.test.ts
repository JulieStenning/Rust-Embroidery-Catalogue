import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import TaggingActionsView from "../TaggingActionsView.svelte";
import { resetBusy } from "../../stores/busyStore.js";

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
    has_google_api_key: true,
    ai_tier2_auto: false,
    ai_tier3_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    import_commit_batch_size: "",
    default_batch_size: 100,
    default_commit_every: 100,
    default_workers: 4,
  },
});

describe("TaggingActionsView stop behaviour", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetBusy();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.stopUnifiedBackfill.mockResolvedValue({
      source: "rust",
      status: "stopping",
    });
  });

  /**
   * Renders the view, enables Tagging + Tier 2 and starts a never-resolving
   * backfill so the Run button enters its in-flight state.
   */
  async function startInFlightRun() {
    adapterMocks.runUnifiedBackfill.mockReturnValue(new Promise(() => {}));
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Running..." })).toBeDisabled();
    });
  }

  it("disables Run and enables Stop while a backfill is in flight", async () => {
    await startInFlightRun();

    expect(screen.getByRole("button", { name: "Running..." })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Stop" })).not.toBeDisabled();
  });

  it("requests a stop via stopUnifiedBackfill", async () => {
    await startInFlightRun();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Stop" }));

    await waitFor(() => {
      expect(adapterMocks.stopUnifiedBackfill).toHaveBeenCalledTimes(1);
    });
    expect(toastMock.addToast).toHaveBeenCalledWith("Stop requested.", "info");
  });

  it("shows an error toast when the stop request fails", async () => {
    adapterMocks.stopUnifiedBackfill.mockRejectedValue(new Error("stop failed"));
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
