/**
 * Unit tests for busyStore.ts.
 *
 * Covers every exported function and all internal branches:
 *   - initial (idle) state
 *   - beginBusy() activates the store and records the label
 *   - endBusy() releases the lock once all begin/end calls are balanced
 *   - nested / overlapping guards keep the store active until the count
 *     returns to zero
 *   - endBusy() never drives the count below zero
 *   - resetBusy() forces the store back to idle
 */

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, beforeEach } from "vitest";
import { busyState, beginBusy, endBusy, resetBusy, type BusyState } from "../busyStore";

/** Helper that resolves the current busy-state snapshot. */
function currentState(): BusyState {
  let snapshot: BusyState = { active: false, label: "", count: 0 };
  const unsubscribe = busyState.subscribe((state) => {
    snapshot = state;
  });
  unsubscribe();
  return snapshot;
}

describe("busyStore", () => {
  beforeEach(() => {
    // Reset the store between tests so snapshot assertions are isolated.
    resetBusy();
  });

  describe("initial state", () => {
    it("starts idle (not active, empty label, zero count)", () => {
      expect(currentState()).toEqual({ active: false, label: "", count: 0 });
    });
  });

  describe("beginBusy", () => {
    it("activates the store and records the label", () => {
      beginBusy("Importing designs");
      expect(currentState()).toEqual({ active: true, label: "Importing designs", count: 1 });
    });

    it("supports nested calls, keeping the store active and incrementing the count", () => {
      beginBusy("Backing up");
      beginBusy("Importing unmatched files");
      const state = currentState();
      expect(state.active).toBe(true);
      expect(state.count).toBe(2);
      // The label reflects the most recent operation to begin.
      expect(state.label).toBe("Importing unmatched files");
    });
  });

  describe("endBusy", () => {
    it("releases the lock when the single outstanding guard is balanced", () => {
      beginBusy("Backing up");
      endBusy();
      expect(currentState()).toEqual({ active: false, label: "", count: 0 });
    });

    it("keeps the store active while nested guards remain outstanding", () => {
      beginBusy("Backing up");
      beginBusy("Importing unmatched files");
      endBusy(); // releases only the inner guard
      // The store stays active with one outstanding guard; the label is
      // informational and retains the most recent operation's description.
      expect(currentState()).toEqual({ active: true, label: "Importing unmatched files", count: 1 });
      endBusy(); // releases the outer guard
      expect(currentState()).toEqual({ active: false, label: "", count: 0 });
    });

    it("never drives the count below zero", () => {
      beginBusy("Backing up");
      endBusy();
      endBusy(); // extra balanced call must be a safe no-op
      expect(currentState()).toEqual({ active: false, label: "", count: 0 });
    });
  });

  describe("resetBusy", () => {
    it("forces the store back to idle even with outstanding guards", () => {
      beginBusy("Backing up");
      beginBusy("Tagging");
      resetBusy();
      expect(currentState()).toEqual({ active: false, label: "", count: 0 });
    });
  });
});
