/**
 * Unit tests for toastStore.ts.
 *
 * Covers every exported function and all internal branches:
 *   - store initial state
 *   - addToast() defaults, all four type variants, persistent flag,
 *     custom duration, default auto-dismiss, ID auto-increment
 *   - removeToast() happy path, preservation of other toasts, no-op for
 *     unknown ids
 *
 * setTimeout() behaviour is driven with vi.useFakeTimers() so the
 * 2800 ms default and custom durations are asserted deterministically.
 */

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { toasts, addToast, removeToast, type Toast } from "../toastStore";

/** Helper that resolves the current toast snapshot. */
function currentToasts(): Toast[] {
  let snapshot: Toast[] = [];
  const unsubscribe = toasts.subscribe((list) => {
    snapshot = list;
  });
  unsubscribe();
  return snapshot;
}

describe("toastStore", () => {
  beforeEach(() => {
    // Reset the store between tests so snapshot assertions are isolated.
    toasts.set([]);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("toasts store", () => {
    it("initialises to an empty array", () => {
      expect(currentToasts()).toEqual([]);
    });
  });

  describe("addToast", () => {
    it("adds a toast with the default type 'info' and default flags", () => {
      addToast("Hello world");

      const [toast] = currentToasts();
      expect(toast).toBeDefined();
      expect(toast.message).toBe("Hello world");
      expect(toast.type).toBe("info");
      expect(toast.persistent).toBe(false);
      // ID is auto-generated; the module-scoped counter is shared across the
      // whole test file so assert presence rather than an exact value.
      expect(typeof toast.id).toBe("number");
    });

    it("records a createdAt timestamp near the current time", () => {
      const before = Date.now();
      addToast("Timestamped");
      const after = Date.now();

      const [toast] = currentToasts();
      expect(toast.createdAt).toBeGreaterThanOrEqual(before);
      expect(toast.createdAt).toBeLessThanOrEqual(after);
    });

    it("supports all four visual types", () => {
      const types: Toast["type"][] = ["success", "error", "info", "warning"];

      for (const type of types) {
        toasts.set([]);
        addToast(`Type ${type}`, type);

        const [toast] = currentToasts();
        expect(toast.type).toBe(type);
      }
    });

    it("does NOT schedule an auto-dismiss when persistent is true", () => {
      const spy = vi.spyOn(globalThis, "setTimeout");

      addToast("Persistent", "info", true);

      expect(spy).not.toHaveBeenCalled();
      spy.mockRestore();

      // The toast remains in the store after all timers have passed.
      vi.advanceTimersByTime(100_000);
      expect(currentToasts()).toHaveLength(1);
    });

    it("uses a custom durationMs when provided", () => {
      const spy = vi.spyOn(globalThis, "setTimeout");

      addToast("Custom", "warning", false, 500);

      expect(spy).toHaveBeenCalledTimes(1);
      expect(spy).toHaveBeenCalledWith(expect.any(Function), 500);
      spy.mockRestore();
    });

    it("schedules an auto-dismiss with the default 2800 ms", () => {
      const spy = vi.spyOn(globalThis, "setTimeout");

      addToast("Transient");

      expect(spy).toHaveBeenCalledTimes(1);
      expect(spy).toHaveBeenCalledWith(expect.any(Function), 2800);
      spy.mockRestore();
    });

    it("removes a non-persistent toast after the default timeout elapses", () => {
      addToast("Will disappear");

      expect(currentToasts()).toHaveLength(1);

      vi.advanceTimersByTime(2799);
      expect(currentToasts()).toHaveLength(1);

      vi.advanceTimersByTime(1);
      expect(currentToasts()).toHaveLength(0);
    });

    it("removes a non-persistent toast after a custom timeout elapses", () => {
      addToast("Faster", "error", false, 100);

      expect(currentToasts()).toHaveLength(1);

      vi.advanceTimersByTime(99);
      expect(currentToasts()).toHaveLength(1);

      vi.advanceTimersByTime(1);
      expect(currentToasts()).toHaveLength(0);
    });

    it("auto-increments ids across multiple calls", () => {
      // Probe to discover the current module-scoped id counter position, since
      // `nextId` is shared across the whole test file rather than resetting.
      toasts.set([]);
      addToast("Probe");
      const probeId = currentToasts()[0].id;

      toasts.set([]);
      addToast("First");
      addToast("Second", "success");
      addToast("Third", "warning", true);

      const list = currentToasts();
      expect(list.map((t) => t.id)).toEqual([
        probeId + 1,
        probeId + 2,
        probeId + 3,
      ]);
    });

    it("appends new toasts to the end of the list", () => {
      toasts.set([]);
      addToast("A");
      addToast("B");

      const messages = currentToasts().map((t) => t.message);
      expect(messages).toEqual(["A", "B"]);
    });
  });

  describe("removeToast", () => {
    it("removes the toast with the matching id", () => {
      toasts.set([]);
      addToast("One");
      const oneId = currentToasts()[0].id;
      addToast("Two");
      const twoId = currentToasts()[1].id;
      removeToast(oneId);

      const remaining = currentToasts();
      expect(remaining).toHaveLength(1);
      expect(remaining[0].id).toBe(twoId);
      expect(remaining[0].message).toBe("Two");
    });

    it("leaves the other toasts untouched", () => {
      toasts.set([]);
      addToast("One");
      addToast("Two");
      const twoId = currentToasts()[1].id;
      addToast("Three");
      removeToast(twoId);

      const messages = currentToasts().map((t) => t.message);
      expect(messages).toEqual(["One", "Three"]);
    });

    it("is a no-op when the id does not exist", () => {
      toasts.set([]);
      addToast("Only");
      const onlyId = currentToasts()[0].id;

      removeToast(999_999);

      const remaining = currentToasts();
      expect(remaining).toHaveLength(1);
      expect(remaining[0].id).toBe(onlyId);
    });
  });
});