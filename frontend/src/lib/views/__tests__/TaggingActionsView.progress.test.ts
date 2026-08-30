import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
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
    ai_vision_auto: false,
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

  it("shows 'Getting ready for tagging' then the live count on the Run button", async () => {
    // Keep the run in flight so taggingRunInFlight stays true.
    adapterMocks.runUnifiedBackfill.mockReturnValue(new Promise(() => {}));
    render(TaggingActionsView);

    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: /Tagging/ }));
    await user.click(screen.getByRole("button", { name: "Run selected actions" }));

    // Before any design completes, the button shows the "getting ready" state.
    expect(
      screen.getByRole("button", { name: /Getting ready for tagging/ })
    ).toBeInTheDocument();

    backfillProgressStore.set({
      active: true,
      stage: "processing",
      processed: 12,
      errors: 0,
      currentAction: "tagging",
    });
    await tick();
    expect(screen.getByRole("button", { name: "Processing no. 12" })).toBeInTheDocument();
  });
});
