import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { EMPTY_DB_PATH, REPO_ROOT, TEST_DESIGNS_PATH } from "./paths";

/**
 * Helpers for specs that need their own catalogue state rather than the shared,
 * populated `DATA_ROOT_PATH` root. They assemble a throwaway root from the
 * pristine install-template database (see `EMPTY_DB_PATH`), which has the full
 * schema and system tags but no designs, hoops, designers, sources or projects.
 */

/** Throwaway folder used as the "scan this folder" source for a single design. */
export const IMPORT_SOURCE_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".import-source",
);

export interface SeedHoop {
  name: string;
  maxWidthMm: number;
  maxHeightMm: number;
}

export interface EmptyRootOptions {
  /** Hoops to pre-configure in the empty catalogue. */
  hoops?: SeedHoop[];
}

/**
 * Build a throwaway data root containing an empty catalogue.
 *
 * The layout matches what the app expects (`Database/`, `logs/`,
 * `MachineEmbroideryDesigns/`) and `initial_setup_completed` is set to the exact
 * value the backend checks so the app boots straight into the main window.
 */
export function prepareEmptyDataRoot(
  root: string,
  options: EmptyRootOptions = {},
): void {
  fs.rmSync(root, { recursive: true, force: true });
  fs.mkdirSync(path.join(root, "Database"), { recursive: true });
  fs.mkdirSync(path.join(root, "logs"), { recursive: true });
  fs.mkdirSync(path.join(root, "MachineEmbroideryDesigns"), {
    recursive: true,
  });

  if (!fs.existsSync(EMPTY_DB_PATH)) {
    throw new Error(
      `Empty catalogue template not found at:\n  ${EMPTY_DB_PATH}`,
    );
  }
  const databasePath = path.join(root, "Database", "EmbroideryCatalogue.db");
  fs.copyFileSync(EMPTY_DB_PATH, databasePath);

  const db = new DatabaseSync(databasePath);
  try {
    db.exec("DELETE FROM settings WHERE key = 'initial_setup_completed'");
    db.exec(
      "INSERT INTO settings (key, value) VALUES ('initial_setup_completed', 'TRUE')",
    );
    for (const hoop of options.hoops ?? []) {
      db.prepare(
        "INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES (?, ?, ?)",
      ).run(hoop.name, hoop.maxWidthMm, hoop.maxHeightMm);
    }
  } finally {
    db.close();
  }
}

/** Best-effort removal of a throwaway root (WebView2 may still hold handles). */
export function cleanupDataRoot(root: string): void {
  try {
    fs.rmSync(root, {
      recursive: true,
      force: true,
      maxRetries: 5,
      retryDelay: 200,
    });
  } catch {
    // A leftover temp root is harmless.
  }
}

let importSourceCounter = 0;

/**
 * Create a fresh single-design import source folder.
 *
 * A uniquely-named copy of a real design guarantees the import is never
 * deduplicated by a previous run, and keeps the scan/import fast (one file).
 * Returns the absolute folder path to type into the import wizard.
 */
export function prepareSingleDesignImportSource(): string {
  fs.rmSync(IMPORT_SOURCE_PATH, { recursive: true, force: true });
  fs.mkdirSync(IMPORT_SOURCE_PATH, { recursive: true });

  const sourceDesign = path.join(TEST_DESIGNS_PATH, "Cake 3.jef");
  if (!fs.existsSync(sourceDesign)) {
    throw new Error(`Import source design not found at:\n  ${sourceDesign}`);
  }

  importSourceCounter += 1;
  const uniqueName = `Playwright Import ${Date.now()}-${importSourceCounter}.jef`;
  fs.copyFileSync(sourceDesign, path.join(IMPORT_SOURCE_PATH, uniqueName));
  return IMPORT_SOURCE_PATH;
}
