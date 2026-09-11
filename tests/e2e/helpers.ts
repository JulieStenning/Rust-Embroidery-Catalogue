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
  await mainMenu(page)
    .getByRole("link", { name, exact: true })
    .click();
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