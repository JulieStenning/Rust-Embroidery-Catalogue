import { describe, it, expect, afterEach } from "vitest";
import { get } from "svelte/store";
import {
  backfillProgressStore,
  resetBackfillProgress,
  idleBackfillProgress,
} from "../backfillProgressStore";

describe("backfillProgressStore", () => {
  afterEach(() => resetBackfillProgress());

  it("starts idle", () => {
    expect(get(backfillProgressStore)).toEqual(idleBackfillProgress);
  });

  it("holds the latest progress snapshot", () => {
    backfillProgressStore.set({
      active: true,
      stage: "batch_committed",
      processed: 120,
      errors: 0,
      currentAction: "tagging",
    });
    const state = get(backfillProgressStore);
    expect(state.active).toBe(true);
    expect(state.processed).toBe(120);
    expect(state.stage).toBe("batch_committed");
    expect(state.currentAction).toBe("tagging");
  });

  it("resets to idle", () => {
    backfillProgressStore.set({
      active: true,
      stage: "completed",
      processed: 5,
      errors: 0,
      currentAction: "backfill",
    });
    resetBackfillProgress();
    expect(get(backfillProgressStore)).toEqual(idleBackfillProgress);
  });
});
