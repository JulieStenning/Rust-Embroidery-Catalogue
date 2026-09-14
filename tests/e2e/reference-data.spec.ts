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
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    await page.getByTestId("reference-data-tab-hoops").click();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
  });

  test("adds a designer and it persists across a reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // A timestamped name keeps the test idempotent across retries (designer names
    // are unique, case-insensitively).
    const name = `Playwright Designer ${Date.now()}`;
    await page.getByPlaceholder("New designer name...").fill(name);
    await page.getByRole("button", { name: "Add", exact: true }).click();

    await expect(page.getByRole("cell", { name })).toBeVisible();

    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name })).toBeVisible();
  });

  test("adds an image tag and it persists across a reload", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const name = "Playwright Tag";
    await page.locator("#admin-tag-description").fill(name);
    await page.getByRole("button", { name: "Add", exact: true }).click();

    await expect(page.getByRole("cell", { name })).toBeVisible();

    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name })).toBeVisible();
  });

  test("adds a hoop and it persists across a reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // A timestamped name keeps the test idempotent across retries (hoop names
    // are unique, case-insensitively).
    const name = `Playwright Hoop ${Date.now()}`;
    await page.locator("#admin-hoop-name").fill(name);
    await page.locator("#admin-hoop-width").fill("150");
    await page.locator("#admin-hoop-height").fill("150");

    const addButton = page.getByRole("button", { name: "Add", exact: true });
    await expect(addButton).toBeEnabled();
    await addButton.click();

    await expect(page.getByRole("cell", { name })).toBeVisible();

    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name })).toBeVisible();
  });
});
