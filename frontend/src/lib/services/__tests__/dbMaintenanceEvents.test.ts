import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// ---------------------------------------------------------------------------
// Mock Tauri's event API — we capture the registered callbacks so tests can
// invoke them directly and assert on the side-effects (console.info / addToast).
// ---------------------------------------------------------------------------
const eventMocks = vi.hoisted(() => ({
  listen: vi.fn(),
  unlistenStarted: vi.fn(),
  unlistenFinished: vi.fn(),
  startedCallback: vi.fn(),
  finishedCallback: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: eventMocks.listen,
}));

// Mock the toast store — dbMaintenanceEvents calls addToast() when the
// "finished" event fires. The path is relative to THIS test file:
// frontend/src/lib/services/__tests__/ → frontend/src/lib/stores/toastStore.
const toastMocks = vi.hoisted(() => ({
  addToast: vi.fn(),
}));

vi.mock("../../stores/toastStore", () => toastMocks);

// The module under test is imported AFTER the mocks are registered.
import {
  DB_MAINTENANCE_STARTED,
  DB_MAINTENANCE_FINISHED,
  type DbMaintenanceStartedEvent,
  type DbMaintenanceFinishedEvent,
} from "../../types/dbMaintenance";
import { initDbMaintenanceEvents } from "../dbMaintenanceEvents";

/** Spy created fresh per-test so it is never detached by afterEach cleanup. */
let consoleInfoSpy: ReturnType<typeof vi.spyOn>;

describe("dbMaintenanceEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Silence the "started" event's console.info output and capture it for
    // assertions. Created inside beforeEach so afterEach's mockRestore() does
    // not leave a detached spy for the next test.
    consoleInfoSpy = vi.spyOn(console, "info").mockImplementation(() => {});

    // Default: listen() resolves with the two unlisten functions, and
    // registers each callback so tests can fire the events manually.
    eventMocks.listen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === DB_MAINTENANCE_STARTED) {
        eventMocks.startedCallback.mockImplementation(callback as (...args: unknown[]) => void);
      } else {
        eventMocks.finishedCallback.mockImplementation(callback as (...args: unknown[]) => void);
      }
      return Promise.resolve(
        eventName === DB_MAINTENANCE_STARTED
          ? eventMocks.unlistenStarted
          : eventMocks.unlistenFinished
      );
    });
  });

  afterEach(() => {
    consoleInfoSpy.mockRestore();
  });

  it("registers a listener for both maintenance events", async () => {
    await initDbMaintenanceEvents();

    expect(eventMocks.listen).toHaveBeenCalledTimes(2);
    expect(eventMocks.listen).toHaveBeenCalledWith(DB_MAINTENANCE_STARTED, expect.any(Function));
    expect(eventMocks.listen).toHaveBeenCalledWith(DB_MAINTENANCE_FINISHED, expect.any(Function));
  });

  it("logs the started event payload via console.info", async () => {
    const event: { payload: DbMaintenanceStartedEvent } = {
      payload: {
        page_count: 100,
        freelist_pages: 25,
        free_ratio: 0.25,
        reclaimable_bytes: 4096,
      },
    };

    await initDbMaintenanceEvents();

    eventMocks.startedCallback(event);

    expect(consoleInfoSpy).toHaveBeenCalledWith(
      "[db-maintenance] started — free ratio 25.0%, 4096 bytes reclaimable"
    );
  });

  it("shows a completion toast when the finished event fires", async () => {
    const event: { payload: DbMaintenanceFinishedEvent } = {
      payload: {
        reclaimed_pages: 12345,
        reclaimable_bytes_before: 100000,
        reclaimable_bytes_after: 50000,
        duration_ms: 250,
      },
    };

    await initDbMaintenanceEvents();

    eventMocks.finishedCallback(event);

    expect(toastMocks.addToast).toHaveBeenCalledWith(
      "Database optimisation complete — 12,345 pages reclaimed",
      "success",
      false,
      5000
    );
  });

  it("returns a cleanup function that unlistens from both events", async () => {
    const cleanup = await initDbMaintenanceEvents();

    expect(eventMocks.unlistenStarted).not.toHaveBeenCalled();
    expect(eventMocks.unlistenFinished).not.toHaveBeenCalled();

    cleanup();

    expect(eventMocks.unlistenStarted).toHaveBeenCalledTimes(1);
    expect(eventMocks.unlistenFinished).toHaveBeenCalledTimes(1);
  });
});
