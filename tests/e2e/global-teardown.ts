import fs from "node:fs";
import {
  DATA_ROOT_PATH,
  EMPTY_DATA_ROOT_PATH,
  HOOPS_DATA_ROOT_PATH,
} from "./paths";
import { IMPORT_SOURCE_PATH } from "./empty-root";

import { getCoverageInstance } from "./coverage-helper";

/**
 * Remove the throwaway data roots created by the global setup (and by specs
 * that need an empty catalogue), plus the generated import source folder.
 *
 * The app process itself is shut down by the `browser` fixture, so this only
 * needs to clean up the on-disk artefacts left behind. Every removal is
 * best-effort so a locked leftover never fails the run.
 */
export default async function globalTeardown(): Promise<void> {
  const mcr = getCoverageInstance();
  if (mcr.hasCache()) {
    await mcr.generate();
  }

  const roots = [
    DATA_ROOT_PATH,
    EMPTY_DATA_ROOT_PATH,
    HOOPS_DATA_ROOT_PATH,
    IMPORT_SOURCE_PATH,
  ];
  for (const root of roots) {
    try {
      fs.rmSync(root, { recursive: true, force: true });
    } catch {
      // Best-effort: a leftover temp root is harmless.
    }
  }
}
