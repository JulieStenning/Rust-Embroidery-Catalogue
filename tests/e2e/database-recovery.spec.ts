// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { test, expect } from "./app-fixture";
import { cleanupDataRoot } from "./empty-root";
import {
  DATABASE_FILENAME,
  DESIGNS_CONTAINER,
  REPO_ROOT,
  TEST_DB_PATH,
} from "./paths";
import type { Page } from "@playwright/test";

const MISSING_DB_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".recovery-missing-root",
);

const NEW_CATALOGUE_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".recovery-new-catalogue-root",
);

const CORRUPT_DB_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".recovery-corrupt-root",
);

const CORRUPT_FRESH_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".recovery-corrupt-fresh-root",
);

/**
 * Stub `browse_restore_file` IPC invoke in the WebView2 context to return a simulated path.
 */
async function stubBrowseRestoreFile(
  page: Page,
  selectedPath: string | null,
  errorMessage: string | null = null,
): Promise<void> {
  await page.evaluate(
    ({ pathValue, errorValue }) => {
      const win = window as unknown as {
        __E2E_IPC_STUBS__?: Record<string, (args?: unknown) => unknown>;
      };
      win.__E2E_IPC_STUBS__ = win.__E2E_IPC_STUBS__ || {};
      win.__E2E_IPC_STUBS__["browse_restore_file"] = () => ({
        path: pathValue,
        error: errorValue,
      });
    },
    { pathValue: selectedPath, errorValue: errorMessage },
  );
}

test.describe("database recovery startup mode - missing database", () => {
  test.use({ dataRoot: MISSING_DB_ROOT_PATH });

  test.beforeAll(() => {
    // Prepare a configured root with no database file to trigger recovery mode.
    cleanupDataRoot(MISSING_DB_ROOT_PATH);
    cleanupDataRoot(NEW_CATALOGUE_ROOT_PATH);
    fs.mkdirSync(MISSING_DB_ROOT_PATH, { recursive: true });
  });

  test.afterAll(() => {
    cleanupDataRoot(MISSING_DB_ROOT_PATH);
    cleanupDataRoot(NEW_CATALOGUE_ROOT_PATH);
  });

  test("shows recovery screen and creates fresh catalogue at custom location", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    // 1. App should detect the missing database and show the recovery view.
    const recoveryView = page.getByTestId("database-recovery-view");
    await expect(recoveryView).toBeVisible();
    await expect(
      page.getByText("Your catalogue database could not be found"),
    ).toBeVisible();

    // 2. Open the "Create a new empty catalogue" modal.
    const createNewButton = page.getByTestId("recovery-create-new");
    await expect(createNewButton).toBeVisible();
    await createNewButton.click();

    // 3. Verify previous location is default in the location input.
    const locationInput = page.getByTestId("recovery-new-location-input");
    await expect(locationInput).toBeVisible();
    const defaultValue = await locationInput.inputValue();
    expect(path.normalize(defaultValue)).toBe(
      path.normalize(MISSING_DB_ROOT_PATH),
    );

    // 4. Change location to a new custom directory.
    await locationInput.fill(NEW_CATALOGUE_ROOT_PATH);

    // 5. Confirm creation.
    const confirmButton = page.getByTestId("recovery-create-confirm");
    await expect(confirmButton).toBeEnabled();
    await confirmButton.click();

    // 6. Verify restart dialog is shown.
    const restartButton = page.getByTestId("recovery-restart-now");
    await expect(restartButton).toBeVisible();

    // 7. Verify the new catalogue structure and database were created on disk.
    const newDbPath = path.join(
      NEW_CATALOGUE_ROOT_PATH,
      "Database",
      DATABASE_FILENAME,
    );
    const newDesignsDir = path.join(NEW_CATALOGUE_ROOT_PATH, DESIGNS_CONTAINER);
    const newLogsDir = path.join(NEW_CATALOGUE_ROOT_PATH, "logs");

    expect(fs.existsSync(newDbPath)).toBe(true);
    expect(fs.existsSync(newDesignsDir)).toBe(true);
    expect(fs.existsSync(newLogsDir)).toBe(true);
  });
});

test.describe("database recovery startup mode - corrupted database and restore", () => {
  test.use({ dataRoot: CORRUPT_DB_ROOT_PATH });

  const corruptDbDir = path.join(CORRUPT_DB_ROOT_PATH, "Database");
  const corruptDbFile = path.join(corruptDbDir, DATABASE_FILENAME);
  const validBackupFile = path.join(CORRUPT_DB_ROOT_PATH, "valid-backup.db");
  const corruptContent = "MALFORMED NON-SQLITE DATABASE TEXT CONTENT";

  test.beforeAll(() => {
    cleanupDataRoot(CORRUPT_DB_ROOT_PATH);
    fs.mkdirSync(corruptDbDir, { recursive: true });
    fs.writeFileSync(corruptDbFile, corruptContent, "utf-8");
    fs.copyFileSync(TEST_DB_PATH, validBackupFile);
  });

  test.afterAll(() => {
    cleanupDataRoot(CORRUPT_DB_ROOT_PATH);
  });

  test("detects corrupted database and recovers by restoring from backup", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    // 1. App should detect the corrupted database and show the recovery view with warning
    const recoveryView = page.getByTestId("database-recovery-view");
    await expect(recoveryView).toBeVisible();
    await expect(
      page.getByRole("heading", {
        name: "Your catalogue database is unreadable",
      }),
    ).toBeVisible();
    await expect(
      page.getByText("Embroidery Catalogue found a database file"),
    ).toBeVisible();

    // 2. Buttons are available for recovery
    const restoreBackupBtn = page.getByTestId("recovery-restore-backup");
    await expect(restoreBackupBtn).toBeVisible();
    await expect(page.getByTestId("recovery-browse")).toBeVisible();
    await expect(page.getByTestId("recovery-create-new")).toBeVisible();

    // 3. Stub the file picker to choose our valid test backup file
    await stubBrowseRestoreFile(page, validBackupFile);
    await restoreBackupBtn.click();

    // 4. Verify confirmation modal is presented
    const modalHeading = page.getByRole("heading", {
      name: "Restore catalogue from backup",
    });
    await expect(modalHeading).toBeVisible();
    await expect(page.getByText(validBackupFile)).toBeVisible();

    // 5. Confirm restore
    const confirmBtn = page.getByTestId("recovery-restore-confirm");
    await expect(confirmBtn).toBeEnabled();
    await confirmBtn.click();

    // 6. Verify restart dialog is presented
    const restartButton = page.getByTestId("recovery-restart-now");
    await expect(restartButton).toBeVisible({ timeout: 15_000 });

    // 7. Verify disk state:
    // a) Corrupt file was archived aside to EmbroideryCatalogue.corrupt-<timestamp>.db
    const dbFiles = fs.readdirSync(corruptDbDir);
    const archivedCorruptFiles = dbFiles.filter(
      (f) => f.startsWith("EmbroideryCatalogue.corrupt-") && f.endsWith(".db"),
    );
    expect(archivedCorruptFiles.length).toBeGreaterThanOrEqual(1);

    const archivedContent = fs.readFileSync(
      path.join(corruptDbDir, archivedCorruptFiles[0]),
      "utf-8",
    );
    expect(archivedContent).toBe(corruptContent);

    // b) Live database file exists and is now a valid readable SQLite database
    expect(fs.existsSync(corruptDbFile)).toBe(true);
    const restoredDb = new DatabaseSync(corruptDbFile);
    const row = restoredDb
      .prepare("SELECT COUNT(*) AS count FROM designs")
      .get() as { count: number };
    expect(row.count).toBeGreaterThan(0);
    restoredDb.close();
  });
});

test.describe("database recovery startup mode - corrupted database fresh catalogue", () => {
  test.use({ dataRoot: CORRUPT_FRESH_ROOT_PATH });

  const corruptDbDir = path.join(CORRUPT_FRESH_ROOT_PATH, "Database");
  const corruptDbFile = path.join(corruptDbDir, DATABASE_FILENAME);
  const corruptContent = "ANOTHER MALFORMED DATABASE FILE FOR FRESH INIT TEST";

  test.beforeAll(() => {
    cleanupDataRoot(CORRUPT_FRESH_ROOT_PATH);
    fs.mkdirSync(corruptDbDir, { recursive: true });
    fs.writeFileSync(corruptDbFile, corruptContent, "utf-8");
  });

  test.afterAll(() => {
    cleanupDataRoot(CORRUPT_FRESH_ROOT_PATH);
  });

  test("starts fresh clean catalogue while safely archiving corrupted database", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    // 1. Detect unreadable database
    const recoveryView = page.getByTestId("database-recovery-view");
    await expect(recoveryView).toBeVisible();
    await expect(
      page.getByRole("heading", {
        name: "Your catalogue database is unreadable",
      }),
    ).toBeVisible();

    // 2. Click "Start fresh with a clean catalogue"
    const createNewBtn = page.getByTestId("recovery-create-new");
    await expect(createNewBtn).toBeVisible();
    await createNewBtn.click();

    // 3. Confirm clean catalogue creation
    const confirmBtn = page.getByTestId("recovery-create-confirm");
    await expect(confirmBtn).toBeEnabled();
    await confirmBtn.click();

    // 4. Verify restart dialog is presented
    const restartButton = page.getByTestId("recovery-restart-now");
    await expect(restartButton).toBeVisible({ timeout: 15_000 });

    // 5. Verify disk state:
    // a) Corrupt file was archived aside
    const dbFiles = fs.readdirSync(corruptDbDir);
    const archivedCorruptFiles = dbFiles.filter(
      (f) => f.startsWith("EmbroideryCatalogue.corrupt-") && f.endsWith(".db"),
    );
    expect(archivedCorruptFiles.length).toBeGreaterThanOrEqual(1);

    const archivedContent = fs.readFileSync(
      path.join(corruptDbDir, archivedCorruptFiles[0]),
      "utf-8",
    );
    expect(archivedContent).toBe(corruptContent);

    // b) Live database file exists and is a valid seeded SQLite database
    expect(fs.existsSync(corruptDbFile)).toBe(true);
    const freshDb = new DatabaseSync(corruptDbFile);
    const row = freshDb
      .prepare("SELECT COUNT(*) AS count FROM settings")
      .get() as { count: number };
    expect(row.count).toBeGreaterThan(0);
    freshDb.close();
  });
});
