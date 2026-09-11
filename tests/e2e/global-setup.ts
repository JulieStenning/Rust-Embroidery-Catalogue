import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import {
  DATA_ROOT_PATH,
  DATABASE_FILENAME,
  DESIGNS_CONTAINER,
  EXE_PATH,
  TEST_DB_PATH,
  TEST_DESIGNS_PATH,
} from "./paths";

/**
 * Build a pristine, throwaway data root for the whole test run.
 *
 * The Tauri app reads its database from `<data_root>/Database/` and its design
 * library from `<data_root>/MachineEmbroideryDesigns/`. We assemble exactly
 * that structure from the repository's test assets so every run starts from
 * the same known catalogue and the developer's real `dev_data/` is never
 * touched.
 */

/**
 * Mark the onboarding wizard as completed so feature tests boot straight into
 * the main application instead of the setup wizard. The setup wizard itself is
 * covered by its own spec, which is free to flip this flag back.
 */
function markInitialSetupComplete(databasePath: string): void {
  const db = new DatabaseSync(databasePath);
  try {
    db.exec("DELETE FROM settings WHERE key = 'initial_setup_completed'");
    db.exec(
      "INSERT INTO settings (key, value) VALUES ('initial_setup_completed', 'TRUE')",
    );
  } finally {
    db.close();
  }
}

export default async function globalSetup(): Promise<void> {
  if (!fs.existsSync(EXE_PATH)) {
    throw new Error(
      `Debug executable not found at:\n  ${EXE_PATH}\n` +
        "Build it once with:  npm run e2e:build",
    );
  }

  fs.rmSync(DATA_ROOT_PATH, { recursive: true, force: true });
  fs.mkdirSync(path.join(DATA_ROOT_PATH, "Database"), { recursive: true });
  fs.mkdirSync(path.join(DATA_ROOT_PATH, "logs"), { recursive: true });

  if (!fs.existsSync(TEST_DB_PATH)) {
    throw new Error(`Test database not found at:\n  ${TEST_DB_PATH}`);
  }
  const databasePath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
  fs.copyFileSync(TEST_DB_PATH, databasePath);
  markInitialSetupComplete(databasePath);

  if (!fs.existsSync(TEST_DESIGNS_PATH)) {
    throw new Error(`Test designs folder not found at:\n  ${TEST_DESIGNS_PATH}`);
  }
  fs.cpSync(TEST_DESIGNS_PATH, path.join(DATA_ROOT_PATH, DESIGNS_CONTAINER), {
    recursive: true,
  });

  console.log(`[e2e] Test data root prepared at ${DATA_ROOT_PATH}`);
}