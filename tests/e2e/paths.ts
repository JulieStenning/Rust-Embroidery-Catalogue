// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import path from "node:path";

/**
 * Shared path constants for the Playwright e2e harness.
 *
 * `REPO_ROOT` is derived from this module's own location (`tests/e2e/paths.ts`)
 * so it is correct regardless of the process working directory. Every other
 * path below is relative to it.
 */
export const REPO_ROOT = path.resolve(__dirname, "..", "..");

/** Absolute path to the debug executable produced by `npm run e2e:build`. */
export const EXE_PATH = path.join(
  REPO_ROOT,
  "target",
  "debug",
  "embroidery-catalogue.exe",
);

/** Absolute path of the throwaway data root prepared by the global setup. */
export const DATA_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".data-root",
);

/**
 * Throwaway data roots for specs that need an *empty* catalogue (no designs,
 * no hoops). The pristine install template is used as the empty seed database.
 * See `import-hoop-setup.spec.ts`.
 */
export const EMPTY_DATA_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".empty-data-root",
);

/** Empty catalogue that already has one hoop configured (the gate's negative case). */
export const HOOPS_DATA_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".hoops-data-root",
);

/**
 * Pristine, empty catalogue database shipped as the install template. It has the
 * full schema and system tags but 0 designs, 0 hoops, 0 designers/sources/projects.
 */
export const EMPTY_DB_PATH = path.join(
  REPO_ROOT,
  "src-tauri",
  "resources",
  "EmbroideryCatalogue.db",
);

/** Absolute path of the seed catalogue copied into the data root. */
export const TEST_DB_PATH = path.join(
  REPO_ROOT,
  "tests",
  "Test Assets",
  "EmbroideryCatalogue.db",
);

/** Absolute path of the designs library copied into the data root. */
export const TEST_DESIGNS_PATH = path.join(REPO_ROOT, "tests", "Test Designs");

/** Canonical database filename inside the data root. */
export const DATABASE_FILENAME = "EmbroideryCatalogue.db";

/** Folder inside the data root the app reads designs from. */
export const DESIGNS_CONTAINER = "MachineEmbroideryDesigns";

/**
 * Base port for the WebView2 CDP endpoint. Each worker gets its own port so
 * the harness can be parallelised later; today it runs with a single worker
 * because every test shares one database and one desktop window.
 */
export const CDP_PORT_BASE = 10000;
