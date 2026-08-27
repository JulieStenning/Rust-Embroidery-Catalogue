/**
 * Global UI busy-state store.
 *
 * Long-running background tasks (bulk import, backup/restore, bulk tagging,
 * catalogue storage migration, etc.) raise this signal so the app's global
 * chrome (navigation menus, footer links, the Back button) and every view's
 * non-essential controls are disabled while the task executes.  The only
 * controls that remain active are each operation's Stop / Cancel affordances,
 * which are gated by their own local flags — never by this store.
 *
 * The state is reference-counted so that nested / overlapping guards (e.g. a
 * backup that internally triggers an unmatched-import step, or a `finally`
 * that fires after a double-clear) cannot release the lock prematurely.
 */

import { writable } from "svelte/store";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface BusyState {
  /** True while at least one long-running operation is in flight. */
  active: boolean;
  /** Human-readable label for the currently active operation (aria / tooltips). */
  label: string;
  /** Reference count of active operations. */
  count: number;
}

// ---------------------------------------------------------------------------
// Store Implementation
// ---------------------------------------------------------------------------

const idle: BusyState = { active: false, label: "", count: 0 };

export const busyState = writable<BusyState>(idle);

/**
 * Mark the start of a long-running operation.
 *
 * @param label Human-readable description of the operation (used for
 *              accessible announcements / tooltips while it is running).
 */
export function beginBusy(label: string): void {
  busyState.update((s) => ({ active: true, label, count: s.count + 1 }));
}

/**
 * Mark the end of a long-running operation.  The store only returns to idle
 * once every outstanding `beginBusy` has been balanced by an `endBusy`.
 */
export function endBusy(): void {
  busyState.update((s) => {
    const count = Math.max(0, s.count - 1);
    return count === 0 ? { ...idle } : { ...s, count };
  });
}

/**
 * Force the store back to its idle state (used by tests to reset between
 * cases, and by error/cleanup paths that need a hard reset).
 */
export function resetBusy(): void {
  busyState.set(idle);
}
