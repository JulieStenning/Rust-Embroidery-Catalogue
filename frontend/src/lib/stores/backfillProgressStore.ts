/**
 * Backfill progress store.
 *
 * Module-level store mirroring the `backfill-progress` events emitted by the
 * Rust backend during a unified backfill run (Tagging Actions page). The view
 * subscribes here to render a live "Processed N designs — <action>…" message.
 */

import { writable } from "svelte/store";

export interface BackfillProgressState {
  active: boolean;
  stage: string;
  processed: number;
  errors: number;
  currentAction: string;
}

export const idleBackfillProgress: BackfillProgressState = {
  active: false,
  stage: "",
  processed: 0,
  errors: 0,
  currentAction: "",
};

export const backfillProgressStore = writable<BackfillProgressState>(idleBackfillProgress);

/** Reset the store to its idle state (e.g. when a backfill run starts/resets). */
export function resetBackfillProgress(): void {
  backfillProgressStore.set(idleBackfillProgress);
}
