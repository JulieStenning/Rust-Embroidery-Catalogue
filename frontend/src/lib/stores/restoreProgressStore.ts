/**
 * Restore progress store.
 *
 * Module-level store mirroring the `catalogue-restore-progress` events emitted
 * by the Rust backend during a restore. `RestoreProgressPanel` and
 * `BackupView` subscribe here for live database-swap and design-sync metrics.
 */

import { writable } from "svelte/store";

export interface RestoreProgressState {
  active: boolean;
  phase: string;
  dbStatus: string;
  scanned: number;
  copied: number;
  skipped: number;
  totalBytes: number;
  percent: number;
  error: string | null;
}

export const idleRestoreProgress: RestoreProgressState = {
  active: false,
  phase: "",
  dbStatus: "",
  scanned: 0,
  copied: 0,
  skipped: 0,
  totalBytes: 0,
  percent: 0,
  error: null,
};

export const restoreProgressStore = writable<RestoreProgressState>(idleRestoreProgress);

/** Reset the store to its idle state (e.g. when a restore completes or is reset). */
export function resetRestoreProgress(): void {
  restoreProgressStore.set(idleRestoreProgress);
}
