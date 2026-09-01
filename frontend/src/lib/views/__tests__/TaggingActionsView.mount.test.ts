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
  countTaggingCandidates: vi.fn(),
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

/** Helper that constructs the tagging actions view model returned by the adapter. */
const viewModel = (overrides = {}) => ({
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
    ...overrides,
  },
});

function configureMocks(overrides = {}) {
  vi.clearAllMocks();
  adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel(overrides));
  adapterMocks.getBackfillLogEntries.mockResolvedValue({
    source: "rust",
    entries: [],
  });
  adapterMocks.countTaggingCandidates.mockResolvedValue({
    source: "rust",
    action: "tag_untagged",
    counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
  });
}

describe("TaggingActionsView mount behaviour", () => {
  beforeEach(() => configureMocks());

  it("loads the view model, backfill log, and scope counts on mount", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(adapterMocks.getTaggingActionsViewModel).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(adapterMocks.getBackfillLogEntries).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(adapterMocks.countTaggingCandidates).toHaveBeenCalledTimes(8);
    });
  });

  it("shows the no-API-key notice and info toast when no key is set", async () => {
    render(TaggingActionsView);

    expect(
      await screen.findByText(
        /No Google API key is configured in Settings\. Text AI and Vision AI tagging will be skipped\./
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
    configureMocks({ has_google_api_key: true });
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
    adapterMocks.getTaggingActionsViewModel.mockRejectedValue(new Error("no backend"));
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
  beforeEach(() => configureMocks());

  it("renders the page title and subtitle", () => {
    render(TaggingActionsView);

    expect(screen.getByRole("heading", { name: "Tagging Actions" })).toBeInTheDocument();
    expect(
      screen.getByText(/Retag or backfill your catalogue with clear goals/)
    ).toBeInTheDocument();
  });

  it("renders the Goal / Scope / Merge workflow with sensible defaults", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("radio", { name: /Apply File & Folder Rules/ })).toBeChecked();
    });
    expect(screen.getByRole("radio", { name: /Untagged designs only/ })).toBeChecked();
    expect(screen.getByRole("radio", { name: /Add New Tags Only/ })).toBeChecked();
  });

  it("disables Visual AI goals when no API key is set", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("radio", { name: /Enrich with Visual AI/ })).toBeDisabled();
    });
    expect(screen.getByRole("radio", { name: /Full Re-Scan/ })).toBeDisabled();
  });

  it("enables the Run button when ready (a default action is always selected)", async () => {
    render(TaggingActionsView);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Review & Start Tagging" })).not.toBeDisabled();
    });
    expect(screen.getByRole("button", { name: "Stop" })).toBeDisabled();
  });
});