import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";

// ---------------------------------------------------------------------------
// Mock Tauri's event API so tests can invoke the registered callback directly
// and assert on the store side-effects.
// ---------------------------------------------------------------------------
const eventMocks = vi.hoisted(() => ({
  listen: vi.fn(),
  unlisten: vi.fn(),
  callback: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: eventMocks.listen,
}));

import {
  BACKFILL_PROGRESS_EVENT,
  initBackfillProgressEvents,
} from "../backfillEvents";
import {
  backfillProgressStore,
  resetBackfillProgress,
} from "../../stores/backfillProgressStore";

describe("backfillEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetBackfillProgress();
    eventMocks.listen.mockImplementation((eventName: string, callback: unknown) => {
      eventMocks.callback.mockImplementation(callback as (...args: unknown[]) => void);
      return Promise.resolve(eventMocks.unlisten);
    });
  });

  afterEach(() => {
    resetBackfillProgress();
  });

  it("subscribes to the backfill-progress event", async () => {
    await initBackfillProgressEvents();
    expect(eventMocks.listen).toHaveBeenCalledWith(
      BACKFILL_PROGRESS_EVENT,
      expect.any(Function)
    );
  });

  it("maps the payload into the backfill progress store", async () => {
    await initBackfillProgressEvents();
    eventMocks.callback({
      payload: {
        stage: "batch_committed",
        processed: 250,
        errors: 3,
        current_action: "tagging",
      },
    });
    const state = get(backfillProgressStore);
    expect(state.active).toBe(true);
    expect(state.stage).toBe("batch_committed");
    expect(state.processed).toBe(250);
    expect(state.errors).toBe(3);
    expect(state.currentAction).toBe("tagging");
  });
});
