// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./app-fixture";
import { cleanupDataRoot } from "./empty-root";
import { DATABASE_FILENAME, DESIGNS_CONTAINER, REPO_ROOT } from "./paths";

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

test.describe("database recovery startup mode", () => {
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
