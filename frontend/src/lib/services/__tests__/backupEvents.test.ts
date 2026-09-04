import { describe, it, expect, vi, beforeEach } from "vitest";

// ---------------------------------------------------------------------------
// Mock Tauri's event API so tests can invoke the registered callback directly
// and assert the callback is wired to the database-backup-completed event.
// ---------------------------------------------------------------------------
const eventMocks = vi.hoisted(() => ({
  listen: vi.fn(),
  unlisten: vi.fn(),
  callback: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: eventMocks.listen,
}));

import { DATABASE_BACKUP_COMPLETED_EVENT, initDatabaseBackupCompletedEvent } from "../backupEvents";

describe("backupEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    eventMocks.listen.mockImplementation((eventName: string, callback: unknown) => {
      eventMocks.callback.mockImplementation(callback as (...args: unknown[]) => void);
      return Promise.resolve(eventMocks.unlisten);
    });
  });

  it("subscribes to the database-backup-completed event", async () => {
    await initDatabaseBackupCompletedEvent(() => {});
    expect(eventMocks.listen).toHaveBeenCalledWith(
      DATABASE_BACKUP_COMPLETED_EVENT,
      expect.any(Function)
    );
  });

  it("invokes the completion callback when the event fires", async () => {
    const onCompleted = vi.fn();
    await initDatabaseBackupCompletedEvent(onCompleted);
    expect(onCompleted).not.toHaveBeenCalled();
    eventMocks.callback({ payload: null });
    expect(onCompleted).toHaveBeenCalledTimes(1);
  });

  it("returns a cleanup function that unlistens", async () => {
    const cleanup = await initDatabaseBackupCompletedEvent(() => {});
    expect(eventMocks.unlisten).not.toHaveBeenCalled();
    cleanup();
    expect(eventMocks.unlisten).toHaveBeenCalledTimes(1);
  });
});
