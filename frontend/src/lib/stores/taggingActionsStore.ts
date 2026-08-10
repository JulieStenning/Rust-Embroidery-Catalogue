/**

 * Persistent state for the Tagging Actions view.
 *
 * Keeps user choice such as the stitching clear mode ("none" / "unverified" /
 * "all") alive across route navigation, because the Svelte component is
 * re-created when the route changes and a local $state would reset.
 */
import { writable } from "svelte/store";

/** Stitching clear mode values sent to the Rust backfill. */
export type ClearStitchingMode = "none" | "unverified" | "all";

const DEFAULT_CLEAR_STITCHING_MODE: ClearStitchingMode = "none";

export const taggingClearStitchingMode = writable<ClearStitchingMode>(DEFAULT_CLEAR_STITCHING_MODE);

/**
 * Set the clear-stitching mode, guarding against invalid values.
 *
 * @param value - The new mode ("none" | "unverified" | "all").
 */
export function setTaggingClearStitchingMode(value: string): void {
  if (value === "none" || value === "unverified" || value === "all") {
    taggingClearStitchingMode.set(value);
  } else {
    taggingClearStitchingMode.set(DEFAULT_CLEAR_STITCHING_MODE);
  }
}