import { describe, it, expect } from "vitest";
import { get } from "svelte/store";
import {
  restoreProgressStore,
  idleRestoreProgress,
  resetRestoreProgress,
} from "../restoreProgressStore";

describe("restoreProgressStore", () => {
  it("starts in the idle state", () => {
    expect(get(restoreProgressStore)).toEqual(idleRestoreProgress);
  });

  it("resetRestoreProgress restores the idle state", () => {
    restoreProgressStore.set({
      active: true,
      scope: "designs",
      phase: "designs",
      status: "running",
      terminal: false,
      scanned: 1,
      copied: 1,
      skipped: 0,
      totalBytes: 0,
      percent: 0.5,
      error: "x",
    });
    expect(get(restoreProgressStore).active).toBe(true);

    resetRestoreProgress();
    expect(get(restoreProgressStore)).toEqual(idleRestoreProgress);
    expect(get(restoreProgressStore).active).toBe(false);
  });
});
