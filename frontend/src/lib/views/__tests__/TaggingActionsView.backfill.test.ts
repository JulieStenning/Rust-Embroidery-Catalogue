import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { tick } from "svelte";
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
    has_google_api_key: true,
    ai_tier2_auto: false,
    ai_tier3_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    ai_commit_every: "",
    ai_workers: "",
    ai_free_tier: false,
    import_commit_batch_size: "",
    default_batch_size: 100,
    default_commit_every: 100,
    default_workers: 4,
    default_delay: 5,
  },
});

/** Helper that constructs a unified/stitching backfill result. */
const backfillResult = (overrides = {}) => ({
  source: "rust",
  processed: 0,
  errors: 0,
  stopped: false,
  actions: [],
  ...overrides,
});

/** Type-guard helper so querySelector results can be passed to expect(). */
function element(value: Element | null | undefined, message?: string): HTMLElement {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value as HTMLElement;
}

describe("TaggingActionsView run unified backfill", () => {
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

  it("shows the free-tier rate-limit hint when the key is declared free tier", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: { ...viewModel().model, ai_free_tier: true },
    });

    render(TaggingActionsView);

    await waitFor(() =>
      expect(screen.getByText(/Free tier detected/)).toBeInTheDocument()
    );
    expect(screen.getByText(/15 requests\/minute and 1,500\/day/)).toBeInTheDocument();
  });

  it("runs unified backfill when Tagging + Tier 2 are enabled with API key", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith({
        action_mode: "tag_untagged",
        run_tier2: true,
        run_tier3: false,
        run_images: false,
        image_redo: false,
        run_color_counts: false,
        run_hoop_dimensions: false,
        commit_every: 100,
        batch_size: 100,
        workers: 4,
      });
    });
    expect(toastMock.addToast).toHaveBeenCalledWith("Running selected actions...", "info");
  });

  it("builds the unified backfill payload from all enabled options", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 3/ }));
    await user.click(screen.getByRole("checkbox", { name: /Image generation/ }));
    await user.click(screen.getByRole("checkbox", { name: /Regenerate images/ }));
    await user.click(screen.getByRole("checkbox", { name: /Recalculate colour/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith({
        action_mode: "tag_untagged",
        run_tier2: true,
        run_tier3: true,
        run_images: true,
        image_redo: true,
        run_color_counts: true,
        run_hoop_dimensions: false,
        commit_every: 100,
        batch_size: 100,
        workers: 4,
      });
    });
  });

  it("uses the batch size, commit every, and workers from Settings", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: {
        ...viewModel().model,
        ai_batch_size: "25",
        ai_commit_every: "10",
        ai_workers: "2",
      },
    });
    render(TaggingActionsView);

    await waitFor(() => expect(adapterMocks.getTaggingActionsViewModel).toHaveBeenCalled());
    await tick();

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ workers: 2, batch_size: 25, commit_every: 10 })
      );
    });
  });

  it("passes run_hoop_dimensions when Recalculate hoops / dimensions is checked", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Recalculate hoops/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ run_hoop_dimensions: true })
      );
    });
  });

  it("passes action_mode tag_all when retag-all is selected", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(
      screen.getByRole("checkbox", {
        name: /Re-tag designs that already have tags/,
      })
    );
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ action_mode: "tag_all" })
      );
    });
  });

  it("does not run any backfill command when run button is disabled", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Run selected actions" })).toBeDisabled();
    });

    expect(adapterMocks.runUnifiedBackfill).not.toHaveBeenCalled();
    expect(adapterMocks.runStitchingBackfill).not.toHaveBeenCalled();
  });

  it("shows the last run summary with processed and error counts", async () => {
    adapterMocks.runUnifiedBackfill.mockResolvedValue(backfillResult({ processed: 12, errors: 2 }));
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(screen.getByText("Last run summary")).toBeInTheDocument();
    });
    // The summary line renders as "Processed: <strong>12</strong> · Errors:
    // <strong>2</strong>" — the numbers live inside child <strong> elements,
    // so scope the queries to the summary card and check each value.
    const summaryCard = element(
      screen.getByText("Last run summary").closest("div.bg-white"),
      "Expected the summary card to exist."
    );
    expect(within(summaryCard).getByText("12")).toBeInTheDocument();
    expect(within(summaryCard).getByText("2")).toBeInTheDocument();
    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill complete: 12 processed, 2 errors.",
        "warning"
      );
    });
  });

  it("shows a success toast when a clean run completes", async () => {
    adapterMocks.runUnifiedBackfill.mockResolvedValue(backfillResult({ processed: 5 }));
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill complete: 5 processed, 0 errors.",
        "success"
      );
    });
  });

  it("shows a warning toast and stopped-early note when the run was stopped", async () => {
    adapterMocks.runUnifiedBackfill.mockResolvedValue(
      backfillResult({ processed: 3, stopped: true })
    );
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill complete: 3 processed, 0 errors (stopped early).",
        "warning"
      );
    });
    expect(screen.getByText("Stopped early")).toBeInTheDocument();
  });

  it("shows an error toast and the error message when the backfill reports an error", async () => {
    adapterMocks.runUnifiedBackfill.mockResolvedValue(
      backfillResult({ error: "Database is locked" })
    );
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill failed: Database is locked",
        "error"
      );
    });
    expect(screen.getByText("Database is locked")).toBeInTheDocument();
  });

  it("shows an error toast when the unified backfill throws", async () => {
    adapterMocks.runUnifiedBackfill.mockRejectedValue(new Error("backend unreachable"));
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("checkbox", { name: /Run Tier 2/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill run failed: Error: backend unreachable",
        "error"
      );
    });
    // The run is no longer in flight, so the button returns to its idle label.
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Run selected actions" })).not.toBeDisabled();
    });
    expect(screen.queryByText("Last run summary")).not.toBeInTheDocument();
  });

  it("runs only Tagging with Tier 1 when Tagging is checked without AI tiers", async () => {
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith({
        action_mode: "tag_untagged",
        run_tier2: false,
        run_tier3: false,
        run_images: false,
        image_redo: false,
        run_color_counts: false,
        run_hoop_dimensions: false,
        commit_every: 100,
        batch_size: 100,
        workers: 4,
      });
    });
  });

  it("does not pass AI tiers when no API key is present even if checked", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: {
        ...viewModel().model,
        has_google_api_key: false,
      },
    });
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    const tier2 = screen.getByRole("checkbox", { name: /Run Tier 2/ });
    expect(tier2).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ run_tier2: false, run_tier3: false })
      );
    });
  });
});
