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
  RESTORE_PROGRESS_EVENT,
  initRestoreProgressEvents,
} from "../restoreEvents";
import { restoreProgressStore, resetRestoreProgress } from "../../stores/restoreProgressStore";

describe("restoreEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetRestoreProgress();
    eventMocks.listen.mockImplementation((eventName: string, callback: unknown) => {
      eventMocks.callback.mockImplementation(callback as (...args: unknown[]) => void);
      return Promise.resolve(eventMocks.unlisten);
    });
  });

  afterEach(() => {
    resetRestoreProgress();
  });

  it("subscribes to the catalogue-restore-progress event", async () => {
    await initRestoreProgressEvents();
    expect(eventMocks.listen).toHaveBeenCalledWith(
      RESTORE_PROGRESS_EVENT,
      expect.any(Function)
    );
  });

  it("maps the payload into the restore progress store", async () => {
    await initRestoreProgressEvents();
    eventMocks.callback({
      payload: {
        phase: "db-swap",
        db_status: "starting",
        scanned: 5,
        copied: 2,
        skipped: 1,
        total_bytes: 100,
        percent: 0.25,
        error: null,
      },
    });

    expect(get(restoreProgressStore)).toEqual({
      active: true,
      phase: "db-swap",
      dbStatus: "starting",
      scanned: 5,
      copied: 2,
      skipped: 1,
      totalBytes: 100,
      percent: 0.25,
      error: null,
    });
  });

  it("maps an error payload onto the store", async () => {
    await initRestoreProgressEvents();
    eventMocks.callback({
      payload: {
        phase: "db-swap",
        db_status: "starting",
        scanned: 0,
        copied: 0,
        skipped: 0,
        total_bytes: 0,
        percent: 0,
        error: "boom",
      },
    });
    expect(get(restoreProgressStore).error).toBe("boom");
  });

  it("returns a cleanup function that unlistens", async () => {
    const cleanup = await initRestoreProgressEvents();
    expect(eventMocks.unlisten).not.toHaveBeenCalled();
    cleanup();
    expect(eventMocks.unlisten).toHaveBeenCalledTimes(1);
  });
});
