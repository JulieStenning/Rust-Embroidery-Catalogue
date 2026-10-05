// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./app-fixture";
import { gotoRoute } from "./helpers";
import { cleanupDataRoot, prepareEmptyDataRoot } from "./empty-root";
import { SETUP_DATA_ROOT_PATH } from "./paths";

/**
 * End-to-end tests for the first-run Initial Setup onboarding wizard.
 *
 * Covers wizard step navigation (Designers -> Sources -> Hoops -> Design Software),
 * back-navigation, master format configuration during onboarding, and persistence
 * to the catalogue settings upon completion.
 */
test.describe("first-run initial setup onboarding wizard", () => {
  test.use({ dataRoot: SETUP_DATA_ROOT_PATH });

  test.beforeAll(() => {
    prepareEmptyDataRoot(SETUP_DATA_ROOT_PATH, {
      initialSetupCompleted: false,
    });
  });

  test.afterAll(() => {
    cleanupDataRoot(SETUP_DATA_ROOT_PATH);
  });

  test("guides user through 4-step setup, configures digitising formats, and persists on finish", async ({
    page,
  }) => {
    test.setTimeout(120_000);

    // Initial onboarding welcome banner and heading
    await expect(
      page.getByText("Welcome to Embroidery Catalogue!"),
    ).toBeVisible({ timeout: 25_000 });
    await expect(
      page.getByRole("heading", { name: "Let's set up your catalogue" }),
    ).toBeVisible();

    // Step 1 of 4 — Designers
    await expect(page.getByText("Step 1 of 4 — Designers")).toBeVisible();
    await expect(page.getByText("What are Designers?")).toBeVisible();
    await expect(page.getByTestId("initial-setup-back")).toBeHidden();

    const continueBtn = page.getByTestId("initial-setup-continue");
    await expect(continueBtn).toHaveText("Continue →");
    await continueBtn.click();

    // Step 2 of 4 — Sources
    await expect(page.getByText("Step 2 of 4 — Sources")).toBeVisible();
    await expect(page.getByText("What are Sources?")).toBeVisible();
    await expect(page.getByTestId("initial-setup-back")).toBeVisible();
    await continueBtn.click();

    // Step 3 of 4 — Hoops
    await expect(page.getByText("Step 3 of 4 — Hoops")).toBeVisible();
    await expect(page.getByText("What are Hoops?")).toBeVisible();
    await continueBtn.click();

    // Step 4 of 4 — Design Software & Master Formats
    await expect(page.getByText("Step 4 of 4 — Design Software")).toBeVisible();
    await expect(
      page.getByText("What are Design Software & Master Formats?"),
    ).toBeVisible();
    await expect(page.getByTestId("initial-setup-formats-view")).toBeVisible();
    await expect(continueBtn).toHaveText("Finish");

    // Toggle software presets: Embird (.eof, .ecf) and Hatch (.emb)
    const embirdPreset = page.getByTestId("master-format-preset-embird");
    const hatchPreset = page.getByTestId("master-format-preset-wilcom_hatch");
    await expect(embirdPreset).toBeVisible();
    await expect(hatchPreset).toBeVisible();

    await embirdPreset.click();
    await hatchPreset.click();

    // Add custom extension (.can)
    const customInput = page.getByTestId("settings-custom-master-formats");
    await customInput.fill("can");

    // Test back-navigation to verify state preservation
    const backBtn = page.getByTestId("initial-setup-back");
    await backBtn.click();
    await expect(page.getByText("Step 3 of 4 — Hoops")).toBeVisible();

    await continueBtn.click();
    await expect(page.getByText("Step 4 of 4 — Design Software")).toBeVisible();
    await expect(embirdPreset.locator("input")).toBeChecked();
    await expect(hatchPreset.locator("input")).toBeChecked();

    // Complete setup wizard
    await continueBtn.click();

    // Wizard completes and main catalogue interface loads
    await expect(
      page.getByRole("heading", { name: "Let's set up your catalogue" }),
    ).toBeHidden({ timeout: 30_000 });

    // Navigate to Settings to verify SQLite persistence of enabled master formats
    await gotoRoute(page, "#/admin/system/settings");
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible({ timeout: 20_000 });

    // Verify Embird and Hatch preset checkboxes are checked in Settings
    const settingsEmbird = page.getByTestId("master-format-preset-embird");
    const settingsHatch = page.getByTestId("master-format-preset-wilcom_hatch");
    await expect(settingsEmbird.locator("input")).toBeChecked();
    await expect(settingsHatch.locator("input")).toBeChecked();
    await expect(
      page.getByTestId("settings-custom-master-formats"),
    ).toHaveValue("can");
  });
});
