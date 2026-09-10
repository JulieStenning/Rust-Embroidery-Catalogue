import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import BatchOperationsView from "../BatchOperationsView.svelte";
import { resetUnmatchedFiles } from "../../stores/unmatchedFilesStore";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getBatchOperationsViewModel: vi.fn(),
  runUnifiedBackfill: vi.fn(),
  stopUnifiedBackfill: vi.fn(),
  getBackfillLogEntries: vi.fn(),
  runStitchingBackfill: vi.fn(),
  runMaintenanceBackfill: vi.fn(),
  countMissingPreviews: vi.fn(),
  countTaggingCandidates: vi.fn(),
  browseTaggingFolder: vi.fn(),
  detectDesignFilesAbsentFromDatabase: vi.fn(),
  importUnmatchedDesignFiles: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast() on mount and after runs.
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

const backfillResult = (overrides = {}) => ({
  source: "rust",
  processed: 0,
  errors: 0,
  stopped: false,
  actions: [],
  ...overrides,
});

async function gotoMaintenanceTab() {
  const user = userEvent.setup();
  await user.click(screen.getByRole("tab", { name: /Maintenance & File Processing/i }));
}

describe("BatchOperationsView two-tab navigation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getBatchOperationsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({ source: "rust", entries: [] });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
    adapterMocks.runMaintenanceBackfill.mockResolvedValue(backfillResult());
    adapterMocks.countMissingPreviews.mockResolvedValue(293);
  });

  it("shows the Tagging tab by default with its actions, not the Maintenance ones", async () => {
    render(BatchOperationsView);
    expect(screen.getByRole("tab", { name: /Tagging & Categorisation/i })).toHaveAttribute(
      "aria-selected",
      "true"
    );
    expect(screen.getByRole("button", { name: "Review & Start Tagging" })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Review & Start Maintenance" })
    ).not.toBeInTheDocument();
  });

  it("switches to the Maintenance tab and shows only the maintenance tasks", async () => {
    render(BatchOperationsView);
    await screen.findByRole("radio", { name: /Apply file & folder rules/i });

    await gotoMaintenanceTab();

    expect(
      screen.getByRole("tab", { name: /Maintenance & File Processing/i })
    ).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("button", { name: "Review & Start Maintenance" })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Review & Start Tagging" })
    ).not.toBeInTheDocument();

    // No maintenance task is selected → the run is disabled.
    expect(screen.getByRole("checkbox", { name: /Generate preview images/ })).not.toBeChecked();
    expect(
      screen.getByRole("button", { name: "Review & Start Maintenance" })
    ).toBeDisabled();
  });
});


describe("BatchOperationsView maintenance run + redirect", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getBatchOperationsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({ source: "rust", entries: [] });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
    adapterMocks.runMaintenanceBackfill.mockResolvedValue(backfillResult());
    adapterMocks.countMissingPreviews.mockResolvedValue(293);
  });

  it("enables Review & Start Maintenance once a task is ticked and dispatches runMaintenanceBackfill", async () => {
    render(BatchOperationsView);
    await screen.findByRole("radio", { name: /Apply file & folder rules/i });
    await gotoMaintenanceTab();

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Generate preview images/ }));
    await user.click(
      screen.getByRole("checkbox", { name: /Recalculate colour \/ stitch counts/ })
    );

    const runButton = screen.getByRole("button", { name: "Review & Start Maintenance" });
    expect(runButton).toBeEnabled();
    await user.click(runButton);

    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(screen.getByText("Ready to Run Maintenance")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Start Maintenance" }));

    await waitFor(() => {
      expect(adapterMocks.runMaintenanceBackfill).toHaveBeenCalledWith({
        scope: "all",
        generate_previews: true,
        recalc_color_counts: true,
        recalc_hoop_dimensions: false,
        commit_every: 100,
        batch_size: 100,
        workers: 4,
      });
    });
  });

  it("shows the live missing-preview count and dispatches the missing_previews scope", async () => {
    render(BatchOperationsView);
    await screen.findByRole("radio", { name: /Apply file & folder rules/i });
    await gotoMaintenanceTab();

    // Live count badge derived from countMissingPreviews.
    await waitFor(() => expect(screen.getByText("293 designs")).toBeInTheDocument());

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("radio", { name: /Designs missing preview images only/ })
    );
    await user.click(screen.getByRole("checkbox", { name: /Generate preview images/ }));
    await user.click(
      screen.getByRole("checkbox", { name: /Recalculate colour \/ stitch counts/ })
    );

    const runButton = screen.getByRole("button", { name: "Review & Start Maintenance" });
    expect(runButton).toBeEnabled();
    await user.click(runButton);

    // The confirm summary reflects the scope + live count.
    expect(
      screen.getByText(/Designs missing a preview image \(293 designs\)/)
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Start Maintenance" }));

    await waitFor(() => {
      expect(adapterMocks.runMaintenanceBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          scope: "missing_previews",
          generate_previews: true,
          recalc_color_counts: true,
        })
      );
    });
  });
});

describe("BatchOperationsView file reconciliation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetUnmatchedFiles();
    adapterMocks.getBatchOperationsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({ source: "rust", entries: [] });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
    adapterMocks.countMissingPreviews.mockResolvedValue(0);
    adapterMocks.detectDesignFilesAbsentFromDatabase.mockResolvedValue({
      source: "rust",
      checked: 4,
      unmatched: 2,
      sample: ["MachineEmbroideryDesigns/a.pes"],
    });
  });

  it("scans from the Maintenance tab and shows the unmatched prompt", async () => {
    render(BatchOperationsView);
    await screen.findByRole("radio", { name: /Apply file & folder rules/i });
    await gotoMaintenanceTab();

    const user = userEvent.setup();
    await user.click(screen.getByTestId("scan-unmatched-button"));

    await waitFor(() =>
      expect(adapterMocks.detectDesignFilesAbsentFromDatabase).toHaveBeenCalled()
    );
    expect(await screen.findByTestId("unmatched-files-prompt")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Import 2 file/ })).toBeInTheDocument();
  });
});
