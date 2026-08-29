import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
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

const viewModel = () => ({
  source: "rust",
  model: {
    has_google_api_key: false,
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

/** Type-guard helper so querySelector results can be passed to expect(). */
function element(value: Element | null | undefined, message?: string): HTMLElement {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value as HTMLElement;
}

describe("TaggingActionsView backfill log", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
  });

  it("shows the empty log placeholder with a zero count", async () => {
    render(TaggingActionsView);

    expect(await screen.findByText("Backfill log (0 entries)")).toBeInTheDocument();
    expect(
      screen.getByText("No log entries yet. Run an action to populate the log.")
    ).toBeInTheDocument();
  });

  it("renders log entries with level labels and the entry count", async () => {
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [
        { level: "info", message: "Backfill started" },
        { level: "warn", message: "Slow batch detected" },
        { level: "error", message: "A design failed to parse" },
      ],
    });
    render(TaggingActionsView);

    expect(await screen.findByText("Backfill log (3 entries)")).toBeInTheDocument();
    expect(screen.getByText("Backfill started")).toBeInTheDocument();
    expect(screen.getByText("Slow batch detected")).toBeInTheDocument();
    expect(screen.getByText("A design failed to parse")).toBeInTheDocument();
    expect(screen.getByText("info")).toBeInTheDocument();
    expect(screen.getByText("warn")).toBeInTheDocument();
    expect(screen.getByText("error")).toBeInTheDocument();
  });

  it("applies the level-specific styling to each log entry", async () => {
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [
        { level: "info", message: "Backfill started" },
        { level: "warn", message: "Slow batch detected" },
        { level: "error", message: "A design failed to parse" },
      ],
    });
    render(TaggingActionsView);

    await screen.findByText("Backfill log (3 entries)");

    const infoWrapper = element(
      screen.getByText("Backfill started").closest("div.text-gray-700"),
      "Expected the info entry wrapper to exist."
    );
    const warnWrapper = element(
      screen.getByText("Slow batch detected").closest("div.text-amber-700"),
      "Expected the warn entry wrapper to exist."
    );
    const errorWrapper = element(
      screen.getByText("A design failed to parse").closest("div.text-red-600"),
      "Expected the error entry wrapper to exist."
    );

    expect(infoWrapper).toBeInTheDocument();
    expect(warnWrapper).toHaveClass("bg-amber-50");
    expect(errorWrapper).toHaveClass("bg-red-50");
  });

  it("swallows log load failures and keeps the empty placeholder", async () => {
    adapterMocks.getBackfillLogEntries.mockRejectedValue(new Error("no log"));
    render(TaggingActionsView);

    expect(
      await screen.findByText("No log entries yet. Run an action to populate the log.")
    ).toBeInTheDocument();
    expect(screen.getByText("Backfill log (0 entries)")).toBeInTheDocument();
  });
});
