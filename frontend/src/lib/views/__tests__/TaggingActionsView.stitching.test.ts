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

// The view installs a backfill-progress listener on mount. Provide a benign
// mock so it resolves cleanly and never touches real Tauri internals in jsdom.
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

const viewModel = () => ({
  source: "rust",
  model: {
    has_google_api_key: false,
    ai_vision_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    import_commit_batch_size: "",
    default_batch_size: 100,
    default_commit_every: 100,
    default_workers: 4,
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

  it("calls runStitchingBackfill with the default options when enabled", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Stitching tag detection/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalledWith({
        commit_every: 100,
        batch_size: 100,
        workers: 4,
        clear_stitching_mode: "unverified",
        image_redo: false,
      });
    });
    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith("Stitching backfill complete.", "success");
    });
  });

  it("passes clear_stitching_mode all when overwrite is selected", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Stitching tag detection/ }));
    await user.click(
      screen.getByRole("checkbox", {
        name: /Overwrite stitching tags on designs that have already been processed/,
      })
    );
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ clear_stitching_mode: "all" })
      );
    });
  });

  it("shows an error toast when the stitching backfill reports an error", async () => {
    adapterMocks.runStitchingBackfill.mockResolvedValue(
      backfillResult({ error: "Malformed stitch data" })
    );
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Stitching tag detection/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Stitching backfill failed: Malformed stitch data",
        "error"
      );
    });
  });

  it("runs both stitching and unified backfills when both are enabled", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Stitching tag detection/ }));
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Visual AI/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runStitchingBackfill).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalled();
    });
  });
});
