/**
 * Global toast notification store.
 *
 * Manages a transient list of floating toast notifications that auto-dismiss
 * after a short duration.  Used by the ToastContainer component to render
 * overlayed toast messages that do not cause layout shifts.
 */

import { writable } from "svelte/store";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Toast {
  /** Unique auto-incrementing identifier */
  id: number;
  /** Display message text */
  message: string;
  /** Visual variant — maps to green / red / blue / amber backgrounds */
  type: "success" | "error" | "info" | "warning";
  /** Timestamp (ms) for animation sequencing */
  createdAt: number;
  /** When true, the toast is NOT auto-dismissed (must be closed manually) */
  persistent: boolean;
}

// ---------------------------------------------------------------------------
// Store Implementation
// ---------------------------------------------------------------------------

const AUTO_DISMISS_MS = 2800;

/** Auto-incrementing ID counter */
let nextId = 1;

export const toasts = writable<Toast[]>([]);

/**
 * Add a transient notification to the toast stack.
 *
 * @param message   - The text to display.
 * @param type      - One of "success", "error", "info", or "warning".
 * @param persistent - When true the toast will not auto-dismiss (default false).
 * @param durationMs - Optional override for auto-dismiss duration in ms (default 2800).
 */
export function addToast(
  message: string,
  type: Toast["type"] = "info",
  persistent = false,
  durationMs: number | null = null
): void {
  const id = nextId++;
  const toast: Toast = { id, message, type, createdAt: Date.now(), persistent };

  toasts.update((list) => [...list, toast]);

  // Schedule auto-dismiss (skip if persistent)
  if (!persistent) {
    const ms = durationMs ?? AUTO_DISMISS_MS;
    setTimeout(() => {
      removeToast(id);
    }, ms);
  }
}

/**
 * Immediately remove a toast by its id.
 *
 * @param id - The toast identifier to remove.
 */
export function removeToast(id: number): void {
  toasts.update((list) => list.filter((t) => t.id !== id));
}
