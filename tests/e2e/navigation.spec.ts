import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, expectMainView } from "./helpers";

/**
 * Navigation coverage for the application shell. These assert the real
 * hash-router wiring (nav links and deep links) against the live Tauri app.
 */
test.describe("main navigation", () => {
  test("boots into the main app on the Browse view", async ({ page }) => {
    await expectMainView(page);
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
  });

  test("top-level nav links route to their views", async ({ page }) => {
    await expectMainView(page);

    await clickNav(page, "Projects");
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();

    await clickNav(page, "Help");
    await expect(page.getByRole("heading", { name: "Help" })).toBeVisible();

    await clickNav(page, "Import");
    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();

    await clickNav(page, "Browse");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
  });

  test("admin hub links route to their default views", async ({ page }) => {
    await expectMainView(page);

    await clickNav(page, "Manage Data");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    await clickNav(page, "System");
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();

    await clickNav(page, "Batch Operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations" }),
    ).toBeVisible();
  });

  test("deep links render the target view", async ({ page }) => {
    await gotoRoute(page, "#/admin/system/backup");
    await expect(
      page.getByRole("heading", { name: "Backup & Restore" }),
    ).toBeVisible();

    await gotoRoute(page, "#/admin/data/tags");
    await expect(page.getByRole("heading", { name: "Manage Tags" })).toBeVisible();

    await gotoRoute(page, "#/projects");
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();
  });
});