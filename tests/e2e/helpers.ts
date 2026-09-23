// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { expect, type Locator, type Page } from "@playwright/test";

/**
 * The application's top navigation menu. Scoping link queries to this container
 * avoids strict-mode collisions with content links that happen to share a name
 * (e.g. the Help page contains a link named "Import").
 */
export function mainMenu(page: Page): Locator {
  return page.locator("nav.menu-shell");
}

/** Click a link in the top navigation menu. */
export async function clickNav(page: Page, name: string): Promise<void> {
  await mainMenu(page).getByRole("link", { name, exact: true }).click();
}

/**
 * Navigate the hash-routed SPA to a route (e.g. "#/projects" or "/projects").
 *
 * Prefer this over clicking the nav menu when a test's subject is the target
 * view rather than the navigation itself; the menu has its own spec.
 */
export async function gotoRoute(page: Page, route: string): Promise<void> {
  const hash = route.startsWith("#") ? route : `#${route}`;
  await page.evaluate((target: string) => {
    window.location.hash = target;
  }, hash);
}

/**
 * Assert the main application shell is mounted, i.e. the setup wizard has
 * already been completed (the top navigation bar only renders in MainView).
 */
export async function expectMainView(page: Page): Promise<void> {
  await expect(
    mainMenu(page).getByRole("link", { name: "Browse", exact: true }),
  ).toBeVisible();
  await expect(
    mainMenu(page).getByRole("link", { name: "Import", exact: true }),
  ).toBeVisible();
}

/** Placeholder text of the step 1 source-folder input (U+2026 ellipsis). */
export const IMPORT_PATH_PLACEHOLDER =
  "Enter path to your embroidery designs folder…";

/** Locator for the folder rows rendered in import step 1 (one per source folder). */
export function folderRows(page: Page): Locator {
  return page.locator(".import-folder-row");
}

/**
 * Return the import wizard to a clean step 1.
 *
 * The app instance is long-lived and the wizard mirrors its state into a module-level
 * store, so a spec that shares the page must reset explicitly before asserting on the
 * default layout. `Reset` is preferred (it exercises the real reset path); when it is
 * disabled because no path is set, the empty input is already the clean state.
 */
export async function resetImportView(page: Page): Promise<void> {
  await gotoRoute(page, "#/import");
  const input = page.locator("#import-root-path");
  await expect(input).toBeVisible();

  const reset = page.getByRole("button", { name: "Reset", exact: true });
  if (await reset.isEnabled().catch(() => false)) {
    await reset.click();
  }
  await expect(input).toHaveValue("");
  await expect(
    page.getByRole("button", { name: "Scan folder(s)" }),
  ).toBeDisabled();
}

/**
 * Drive the wizard from the folder picker to the "Before You Import" step.
 *
 * Scanning is slow (recursive crawl + catalogue de-duplication), so the caller should
 * raise its own timeout before using this helper.
 */
export async function runImportToPrecheck(
  page: Page,
  sourceFolder: string,
): Promise<void> {
  await gotoRoute(page, "#/import");
  await page.getByPlaceholder(IMPORT_PATH_PLACEHOLDER).fill(sourceFolder);
  await page.getByRole("button", { name: "Scan folder(s)" }).click();

  const continueButton = page.getByRole("button", {
    name: /^Continue with \d+ design/,
  });
  await expect(continueButton).toBeVisible({ timeout: 30_000 });
  await continueButton.click();

  // Step 3 is ready once the import action is available.
  await expect(
    page.getByRole("button", { name: "Import Designs" }),
  ).toBeVisible({
    timeout: 30_000,
  });
}
