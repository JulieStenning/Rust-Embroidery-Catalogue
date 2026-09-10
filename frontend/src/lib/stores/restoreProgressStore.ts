/**
 * Restore progress store.
 *
 * Module-level store mirroring the `catalogue-restore-progress` events emitted
 * by the Rust backend during a restore. `RestoreProgressPanel` and
 * `BackupView` subscribe here for live restore progress (scope + phase + status
 * + file metrics).
 */

import { writable } from "svelte/store";

export interface RestoreProgressState {
  active: boolean;
  /** The requested operation: "database" | "designs" | "both" | "import-unmatched". */
  scope: string;
  /** The current step: "database" | "designs" | "reconcile" | "import" | "completed". */
  phase: string;
  /** Neutral status: "starting" | "running" | "done" | "failed" | "rolled-back". */
  status: string;
  /** True once the operation has reached its terminal ("completed") phase. */
  terminal: boolean;
  scanned: number;
  copied: number;
  skipped: number;
  totalBytes: number;
  percent: number;
  error: string | null;
}

export const idleRestoreProgress: RestoreProgressState = {
  active: false,
  scope: "",
  phase: "",
  status: "",
  terminal: false,
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
