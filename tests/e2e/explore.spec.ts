// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test } from "./fixtures";

/**
 * Interactive authoring workbench.
 *
 * This spec is excluded from the default suite (see `testIgnore` in
 * playwright.config.ts). Run it with:
 *
 *   npm run e2e:explore
 *
 * The app window opens and execution pauses, leaving the Playwright Inspector
 * attached so you can hover elements, copy locators, and step through the UI.
 * Resume or close the Inspector when you are done.
 */
test("explore the app interactively", async ({ page }) => {
  // Start somewhere useful - adjust the route as needed.
  await page.evaluate(() => {
    window.location.hash = "#/designs";
  });

  // Opens the Playwright Inspector and holds the app open until you resume.
  await page.pause();
});