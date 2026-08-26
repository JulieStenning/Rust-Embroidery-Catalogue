/**
 * Subscribes to restore progress events emitted by the Rust backend
 * (`catalogue-restore-progress`) and mirrors them into the shared
 * `restoreProgressStore` so the UI can render live metrics.
 */

import type { UnlistenFn } from "@tauri-apps/api/event";
import type { RestoreProgress } from "../types/ipc";
import { restoreProgressStore } from "../stores/restoreProgressStore";

export const RESTORE_PROGRESS_EVENT = "catalogue-restore-progress";

/**
 * Install a Tauri event listener for restore progress. Returns an async
 * cleanup function (call it in `onDestroy` of the root component).
 */
export async function initRestoreProgressEvents(): Promise<UnlistenFn> {
  const { listen } = await import("@tauri-apps/api/event");
  return listen<RestoreProgress>(RESTORE_PROGRESS_EVENT, (event) => {
    const p = event.payload;
    restoreProgressStore.set({
      active: true,
      phase: String(p?.phase || ""),
      dbStatus: String(p?.db_status || ""),
      scanned: Number(p?.scanned ?? 0),
      copied: Number(p?.copied ?? 0),
      skipped: Number(p?.skipped ?? 0),
      totalBytes: Number(p?.total_bytes ?? 0),
      percent: Number(p?.percent ?? 0),
      error: p?.error ? String(p.error) : null,
    });
  });
}
