import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/svelte";
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
  browseTaggingFolder: vi.fn(),
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
    ai_vision_auto: false,
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

/** Opens the pre-flight modal and confirms the run. */
async function startRun() {
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
  await user.click(screen.getByRole("button", { name: "Start Tagging" }));
}

describe("TaggingActionsView run unified backfill", () => {
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

  it("runs unified backfill with the default File & Folder Rules on untagged designs", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith({
        action_mode: "tag_untagged",
        modes: ["path_rule"],
        merge_mode: "add",
        exclude_verified: true,
        run_vision: false,
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

  it("runs Visual AI on designs missing AI analysis when those options are chosen", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    await user.click(
      screen.getByRole("radio", { name: /Designs missing Visual AI analysis/ })
    );

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith({
        action_mode: "retag_all_vision_not_analyzed",
        modes: ["ai_vision"],
        merge_mode: "add",
        exclude_verified: true,
        run_vision: true,
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

  it("passes the chosen scope and merge strategy", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Entire collection/ }));
    await user.click(screen.getByRole("radio", { name: /Complete Reset/ }));

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ action_mode: "retag_all", merge_mode: "reset" })
      );
    });
  });

  it("shows a pre-flight summary with the count and cancels without running", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    // Wait for the scope-count badges to resolve so the modal shows a real count.
    await screen.findAllByText("10 designs");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));

    const modal = within(screen.getByTestId("tagging-confirm-modal"));
    expect(modal.getByText(/Ready to Retag/)).toBeInTheDocument();
    expect(modal.getByText(/10 designs/)).toBeInTheDocument();
    expect(modal.getByText(/Keep all existing tags and append any newly discovered tags/)).toBeInTheDocument();

    await user.click(modal.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByTestId("tagging-confirm-modal")).not.toBeInTheDocument();
    expect(adapterMocks.runUnifiedBackfill).not.toHaveBeenCalled();
  });

  it("states verified exclusion in the modal and passes exclude_verified when unchecked", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await screen.findAllByText("10 unverified · 2 verified");

    // Default: verified designs excluded.
    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    let modalEl = screen.getByTestId("tagging-confirm-modal");
    expect(modalEl).toHaveTextContent(/Verified designs:/);
    expect(modalEl).toHaveTextContent(/Excluded/);

    // Unchecking exclusion flips the modal to "Included" and the run forwards false.
    await user.click(within(modalEl).getByRole("button", { name: "Cancel" }));
    await user.click(
      screen.getByRole("checkbox", { name: /Exclude human-verified designs/ })
    );
    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    modalEl = screen.getByTestId("tagging-confirm-modal");
    expect(modalEl).toHaveTextContent(/Included/);

    await user.click(within(modalEl).getByRole("button", { name: "Start Tagging" }));
    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ exclude_verified: false })
      );
    });
  });

  it("shows the rate-limit pacing note only when the run is actually paced", async () => {
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    await screen.findAllByText("10 unverified · 2 verified");

    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    const modalEl = screen.getByTestId("tagging-confirm-modal");
    expect(modalEl).toHaveTextContent(/Estimated Time/);
    // Paid key with the default (blank) delay is not paced, so no rate-limit note.
    expect(modalEl).not.toHaveTextContent(/paced to respect Gemini rate limits/);
  });

  it("shows the rate-limit pacing note for free-tier keys", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: { ...viewModel().model, ai_free_tier: true },
    });
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    await screen.findAllByText("10 unverified · 2 verified");

    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    const modalEl = screen.getByTestId("tagging-confirm-modal");
    expect(modalEl).toHaveTextContent(/paced to respect Gemini rate limits/);
  });

  it("makes the estimate considerably larger when the run is paced", async () => {
    function minutesFromModal(container: HTMLElement): number {
      const match = container.textContent?.match(/~\s*(\d+)\s*minutes/);
      return match ? Number(match[1]) : 0;
    }

    const bigCounts = { total_count: 10000, unverified_count: 10000, verified_count: 0 };
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: bigCounts,
    });

    // Paid key, blank delay -> not paced.
    const view = render(TaggingActionsView);
    let user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    await screen.findAllByText(/unverified ·/);
    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    const paidEl = screen.getByTestId("tagging-confirm-modal");
    const paidMinutes = minutesFromModal(paidEl);
    expect(paidMinutes).toBeGreaterThan(0);
    view.unmount();

    // Free tier -> paced, should be considerably larger due to the per-call delay.
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: { ...viewModel().model, ai_free_tier: true },
    });
    render(TaggingActionsView);
    user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await user.click(screen.getByRole("radio", { name: /Enrich with Visual AI/ }));
    await screen.findAllByText(/unverified ·/);
    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    const freeEl = screen.getByTestId("tagging-confirm-modal");
    const freeMinutes = minutesFromModal(freeEl);

    expect(freeMinutes).toBeGreaterThan(paidMinutes * 5);
  });

  it("passes the selected folder and subfolder flag to the run", async () => {
    adapterMocks.browseTaggingFolder.mockResolvedValue({
      path: "C:/library/MachineEmbroideryDesigns/Flowers",
      relative_path: "Flowers",
    });
    render(TaggingActionsView);
    const user = userEvent.setup();
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });
    await screen.findAllByText("10 unverified · 2 verified");

    await user.click(screen.getByRole("radio", { name: /Specific Folder or Category/ }));
    await user.click(screen.getByRole("button", { name: "Choose folder…" }));
    await waitFor(() => {
      expect(adapterMocks.browseTaggingFolder).toHaveBeenCalled();
    });

    // Toggle subfolders off, then start the run.
    const subfolders = screen.getByRole("checkbox", { name: /Include subfolders/ });
    await user.click(subfolders);
    expect(subfolders).not.toBeChecked();

    await user.click(screen.getByRole("button", { name: "Review & Start Tagging" }));
    await user.click(screen.getByRole("button", { name: "Start Tagging" }));

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          action_mode: "retag_all",
          folder_path: "C:/library/MachineEmbroideryDesigns/Flowers",
          include_subfolders: false,
        })
      );
    });
  });

  it("shows a stopped-early toast and the summary", async () => {
    adapterMocks.runUnifiedBackfill.mockResolvedValue(
      backfillResult({ processed: 3, errors: 0, stopped: true })
    );
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    await startRun();

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
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    await startRun();

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
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    await startRun();

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Backfill run failed: Error: backend unreachable",
        "error"
      );
    });
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Review & Start Tagging" })
      ).not.toBeDisabled();
    });
    expect(screen.queryByText("Last run summary")).not.toBeInTheDocument();
  });

  it("does not pass Visual AI when no API key is present", async () => {
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue({
      source: "rust",
      model: { ...viewModel().model, has_google_api_key: false },
    });
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({ run_vision: false, modes: ["path_rule"] })
      );
    });
  });

  it("runs Text AI on designs Text AI found no tags for when chosen", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    const user = userEvent.setup();
    await user.click(screen.getByRole("radio", { name: /Analyze with Text AI/ }));
    await user.click(screen.getByRole("radio", { name: /Text AI found no match/ }));

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          action_mode: "retag_all_text_no_match",
          modes: ["text_ai"],
        })
      );
    });
  });

  it("runs all three tiers on the whole collection for a full re-scan", async () => {
    render(TaggingActionsView);
    await screen.findByRole("radio", { name: /Apply File & Folder Rules/ });

    const user = userEvent.setup();
    await user.click(screen.getByRole("radio", { name: /Full Re-Scan/ }));
    await user.click(screen.getByRole("radio", { name: /Entire collection/ }));

    await startRun();

    await waitFor(() => {
      expect(adapterMocks.runUnifiedBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          action_mode: "retag_all",
          modes: ["path_rule", "text_ai", "ai_vision"],
        })
      );
    });
  });

});