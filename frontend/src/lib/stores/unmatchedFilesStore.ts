/**
 * Unmatched-files reconciliation state.
 *
 * Module-level store shared by the Backup & Restore view and the Batch
 * Operations view (both render `UnmatchedFilesReconciler`). It holds only the
 * *prompt* state, so a "Restore Both" run can open the reconciliation prompt and
 * the prompt survives navigating between the two views.
 */

import { writable } from "svelte/store";

export interface UnmatchedFilesState {
  showPrompt: boolean;
  count: number;
  checked: number;
  sample: string[];
}

export const idleUnmatchedFiles: UnmatchedFilesState = {
  showPrompt: false,
  count: 0,
  checked: 0,
  sample: [],
};

export const unmatchedFilesStore = writable<UnmatchedFilesState>(idleUnmatchedFiles);

/** Show the reconciliation prompt for `count` unmatched files. */
export function setUnmatchedFilesDetected(count: number, checked: number, sample: string[]): void {
  unmatchedFilesStore.set({
    showPrompt: true,
    count: Number(count) || 0,
    checked: Number(checked) || 0,
    sample: Array.isArray(sample) ? sample : [],
  });
}

/** Hide the prompt (e.g. after a dismiss or a successful import). */
export function dismissUnmatchedFiles(): void {
  unmatchedFilesStore.set(idleUnmatchedFiles);
}

/** Reset the store to its idle state. */
export function resetUnmatchedFiles(): void {
  unmatchedFilesStore.set(idleUnmatchedFiles);
}
