import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import TaggingActionsView from "../TaggingActionsView.svelte";
import {
  backfillProgressStore,
  resetBackfillProgress,
} from "../../stores/backfillProgressStore";

const adapterMocks = vi.hoisted(() => ({
  getTaggingActionsViewModel: vi.fn(),
  runUnifiedBackfill: vi.fn(),
  stopUnifiedBackfill: vi.fn(),
  getBackfillLogEntries: vi.fn(),
  runStitchingBackfill: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

// The view installs a progress-event listener on mount. Provide a benign mock
// so it resolves cleanly and never touches real Tauri internals in jsdom.
const eventMocks = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: eventMocks.listen }));

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

describe("TaggingActionsView live progress", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetBackfillProgress();
    adapterMocks.getTaggingActionsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({ source: "rust", entries: [] });
    eventMocks.listen.mockResolvedValue(() => {});
  });

  afterEach(() => {
    resetBackfillProgress();
  });

  it("shows a live message when a batch is committed", async () => {
    render(TaggingActionsView);
    backfillProgressStore.set({
      active: true,
      stage: "batch_committed",
      processed: 450,
      errors: 2,
      currentAction: "tagging",
    });
    await tick();
    const panel = screen.getByTestId("backfill-progress");
    expect(panel).toHaveTextContent("Tagging — Processed 450 designs (2 errors)…");
  });

  it("shows a completed message", async () => {
    render(TaggingActionsView);
    backfillProgressStore.set({
      active: true,
      stage: "completed",
      processed: 3,
      errors: 0,
      currentAction: "backfill",
    });
    await tick();
    const panel = screen.getByTestId("backfill-progress");
    expect(panel).toHaveTextContent("Backfill — Completed 3 designs");
  });

  it("hides the panel when no progress has been reported", async () => {
    render(TaggingActionsView);
    await tick();
    expect(screen.queryByTestId("backfill-progress")).not.toBeInTheDocument();
  });
});
