/**
 * Subscribes to backfill progress events emitted by the Rust backend
 * (`backfill-progress`) and mirrors them into the shared
 * `backfillProgressStore` so the Tagging Actions view can render live metrics.
 */

import type { UnlistenFn } from "@tauri-apps/api/event";
import type { BackfillProgress } from "../types/ipc";
import { backfillProgressStore } from "../stores/backfillProgressStore";

export const BACKFILL_PROGRESS_EVENT = "backfill-progress";

/**
 * Install a Tauri event listener for backfill progress. Returns an async
 * cleanup function (call it in `onDestroy` of the owning component).
 */
export async function initBackfillProgressEvents(): Promise<UnlistenFn> {
  const { listen } = await import("@tauri-apps/api/event");
  return listen<BackfillProgress>(BACKFILL_PROGRESS_EVENT, (event) => {
    const p = event.payload;
    backfillProgressStore.set({
      active: true,
      stage: String(p?.stage || ""),
      processed: Number(p?.processed ?? 0),
      errors: Number(p?.errors ?? 0),
      currentAction: String(p?.current_action ?? ""),
    });
  });
}
