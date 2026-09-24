// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./app-fixture";
import { cleanupDataRoot } from "./empty-root";
import { expectMainView, gotoRoute } from "./helpers";
import {
  clearLicenceFromDb,
  generateTestLicence,
} from "./licence-helper";
import {
  DATABASE_FILENAME,
  REPO_ROOT,
  TEST_DB_PATH,
} from "./paths";

const LICENCE_TEST_ROOT_PATH = path.join(
  REPO_ROOT,
  "tests",
  "e2e",
  ".licence-test-root",
);

test.describe.serial("Licence Activation and Verification Flow", () => {
  test.use({ dataRoot: LICENCE_TEST_ROOT_PATH });

  test.beforeEach(() => {
    cleanupDataRoot(LICENCE_TEST_ROOT_PATH);
    fs.mkdirSync(path.join(LICENCE_TEST_ROOT_PATH, "Database"), {
      recursive: true,
    });
    fs.mkdirSync(path.join(LICENCE_TEST_ROOT_PATH, "logs"), {
      recursive: true,
    });
    fs.mkdirSync(
      path.join(LICENCE_TEST_ROOT_PATH, "MachineEmbroideryDesigns"),
      { recursive: true },
    );

    const dbPath = path.join(
      LICENCE_TEST_ROOT_PATH,
      "Database",
      DATABASE_FILENAME,
    );
    fs.copyFileSync(TEST_DB_PATH, dbPath);
    clearLicenceFromDb(dbPath);
  });

  test.afterAll(() => {
    cleanupDataRoot(LICENCE_TEST_ROOT_PATH);
  });

  test("blocks startup with licence activation view when no valid licence is present", async ({
    page,
  }) => {
    test.setTimeout(120_000);

    // 1. App should detect no licence and display the LicenceActivationView
    const licenceView = page.getByTestId("licence-activation-view");
    await expect(licenceView).toBeVisible();

    await expect(
      page.getByRole("heading", { name: "Embroidery Catalogue", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Please enter your registered email address and licence key to activate access.",
      ),
    ).toBeVisible();

    // 2. Form fields are visible
    const emailInput = page.getByTestId("licence-email-input");
    const keyInput = page.getByTestId("licence-key-input");
    const activateButton = page.getByTestId("activate-licence-button");

    await expect(emailInput).toBeVisible();
    await expect(keyInput).toBeVisible();
    await expect(activateButton).toBeVisible();
  });

  test("validates required fields and rejects invalid licence keys", async ({
    page,
  }) => {
    test.setTimeout(120_000);

    const emailInput = page.getByTestId("licence-email-input");
    const keyInput = page.getByTestId("licence-key-input");
    const activateButton = page.getByTestId("activate-licence-button");
    const errorAlert = page.getByTestId("licence-error-alert");

    // 1. Click activate without entering email
    await activateButton.click();
    await expect(errorAlert).toBeVisible();
    await expect(errorAlert).toContainText(
      "Please enter your registered email address.",
    );

    // 2. Enter email but leave licence key empty
    await emailInput.fill("tester@example.com");
    await activateButton.click();
    await expect(errorAlert).toBeVisible();
    await expect(errorAlert).toContainText(
      "Please enter your licence key.",
    );

    // 3. Enter malformed licence key
    await keyInput.fill("NOT-A-VALID-KEY");
    await activateButton.click();
    await expect(errorAlert).toBeVisible();
    await expect(errorAlert).toContainText(
      "Invalid licence key format",
    );

    // 4. Enter valid signature for different email (mismatched email)
    const otherLicence = generateTestLicence("other@example.com", "beta", 30);
    await emailInput.fill("tester@example.com");
    await keyInput.fill(otherLicence.licenceKey);
    await activateButton.click();
    await expect(errorAlert).toBeVisible();
    await expect(errorAlert).toContainText(
      "does not match",
    );
  });

  test("activates successfully with valid signed key and displays licence in settings", async ({
    page,
  }) => {
    test.setTimeout(120_000);

    const email = "betatester@example.com";
    const validLicence = generateTestLicence(email, "beta", 90);

    const emailInput = page.getByTestId("licence-email-input");
    const keyInput = page.getByTestId("licence-key-input");
    const activateButton = page.getByTestId("activate-licence-button");

    // 1. Fill valid credentials and activate
    await emailInput.fill(email);
    await keyInput.fill(validLicence.licenceKey);
    await activateButton.click();

    // 2. Activation should succeed and gate transitions to main app
    await expect(page.getByTestId("licence-activation-view")).not.toBeVisible();
    await expectMainView(page);

    // 3. Navigate to Application Settings
    await gotoRoute(page, "#/admin/system/settings");

    // 4. Verify the Licence & Activation section in Settings
    const licenceSection = page.getByTestId("settings-licence-section");
    await expect(licenceSection).toBeVisible();

    const registeredEmail = page.getByTestId("settings-licence-email");
    await expect(registeredEmail).toBeVisible();
    await expect(registeredEmail).toHaveText(email);

    const deactivateBtn = page.getByTestId(
      "settings-deactivate-licence-button",
    );
    await expect(deactivateBtn).toBeVisible();
  });

  test("deactivates licence from settings and returns to activation gate", async ({
    page,
  }) => {
    test.setTimeout(120_000);

    const email = "deactivate-test@example.com";
    const validLicence = generateTestLicence(email, "beta", 90);

    const emailInput = page.getByTestId("licence-email-input");
    const keyInput = page.getByTestId("licence-key-input");
    const activateButton = page.getByTestId("activate-licence-button");

    // 1. Activate
    await emailInput.fill(email);
    await keyInput.fill(validLicence.licenceKey);
    await activateButton.click();
    await expectMainView(page);

    // 2. Navigate to Settings
    await gotoRoute(page, "#/admin/system/settings");
    const deactivateBtn = page.getByTestId(
      "settings-deactivate-licence-button",
    );
    await expect(deactivateBtn).toBeVisible();

    // 3. Accept confirmation dialog on deactivate
    page.once("dialog", async (dialog) => {
      await dialog.accept();
    });
    await deactivateBtn.click();

    // 4. Should return to LicenceActivationView
    await expect(
      page.getByTestId("licence-activation-view"),
    ).toBeVisible({ timeout: 15_000 });
  });
});
