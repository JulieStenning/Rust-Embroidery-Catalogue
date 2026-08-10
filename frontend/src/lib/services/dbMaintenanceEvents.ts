/**
 * Subscribes to database maintenance events emitted by the Rust backend
 * (`db-maintenance-started` / `db-maintenance-finished`) and surfaces them as
 * lightweight toast notifications via the global toast store.
 *
 * Because incremental-vacuum runs quickly and non-blockingly (sub-second
 * write transactions with the connection yielded between steps), maintenance
 * does NOT prevent the user from using the application. We therefore only
 * surface a short "complete" toast and keep any "started" handling minimal.
 */

// `UnlistenFn` is a type-only import (erased at build time). `listen` is
// imported dynamically inside `initDbMaintenanceEvents()` so the module can
// be code-split as a separate chunk, matching `ImportView.svelte`'s dynamic
// import of `@tauri-apps/api/event`.
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  DB_MAINTENANCE_FINISHED,
  DB_MAINTENANCE_STARTED,
  type DbMaintenanceFinishedEvent,
  type DbMaintenanceStartedEvent,
} from "../types/dbMaintenance";
import { addToast } from "../stores/toastStore";

/**
 * Number of milliseconds before the completion toast auto-dismisses.
 * Slightly longer than the default so the user has time to notice it.
 */
const COMPLETION_TOAST_DURATION_MS = 5000;

/**
 * Install Tauri event listeners for database maintenance lifecycle events.
 *
 * Returns an async cleanup function that removes the listeners (call it in
 * `onDestroy` of the root component).
 */
export async function initDbMaintenanceEvents(): Promise<UnlistenFn> {
  const { listen } = await import("@tauri-apps/api/event");

  // The "started" event fires before the toast container is mounted (on first
  // run the disclaimer may still be pending). We simply log it; the "finished"
  // toast is what the user sees.
  const unlistenStarted = await listen<DbMaintenanceStartedEvent>(
    DB_MAINTENANCE_STARTED,
    (event) => {
      console.info(
        `[db-maintenance] started — free ratio ${(event.payload.free_ratio * 100).toFixed(1)}%, ` +
          `${event.payload.reclaimable_bytes} bytes reclaimable`,
      );
    },
  );

  const unlistenFinished = await listen<DbMaintenanceFinishedEvent>(
    DB_MAINTENANCE_FINISHED,
    (event) => {
      const { reclaimed_pages } = event.payload;
      addToast(
        `Database optimisation complete — ${reclaimed_pages.toLocaleString()} pages reclaimed`,
        "success",
        false,
        COMPLETION_TOAST_DURATION_MS,
      );
    },
  );

  return () => {
    unlistenStarted();
    unlistenFinished();
  };
}