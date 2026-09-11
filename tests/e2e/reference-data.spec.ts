import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";

/**
 * Reference Data hub coverage: tab switching plus an add-and-persist flow that
 * exercises the real IPC + SQLite round-trip.
 */
test.describe("reference data", () => {
  test("switches between hub tabs", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    await page.getByTestId("reference-data-tab-tags").click();
    await expect(page.getByRole("heading", { name: "Manage Tags" })).toBeVisible();

    await page.getByTestId("reference-data-tab-hoops").click();
    await expect(page.getByRole("heading", { name: "Manage Hoops" })).toBeVisible();
  });

  test("adds a designer and it persists across a reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    const name = "Playwright Designer";
    await page.getByPlaceholder("New designer name...").fill(name);
    await page.getByRole("button", { name: "Add", exact: true }).click();

    await expect(page.getByRole("cell", { name })).toBeVisible();

    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name })).toBeVisible();
  });
});