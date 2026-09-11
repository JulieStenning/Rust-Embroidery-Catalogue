import fs from "node:fs";
import { DATA_ROOT_PATH } from "./paths";

/**
 * Remove the throwaway data root created by the global setup.
 *
 * The app process itself is shut down by the `browser` fixture, so this only
 * needs to clean up the on-disk artefacts left behind.
 */
export default async function globalTeardown(): Promise<void> {
  fs.rmSync(DATA_ROOT_PATH, { recursive: true, force: true });
}

