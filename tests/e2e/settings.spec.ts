import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";

/**
 * Application Settings coverage: edit a value, save it through the real IPC
 * layer, then reload and prove it round-tripped through SQLite.
 */
test.describe("application settings", () => {
  test("persists an edited numeric setting across a reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/system/settings");
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();

    const batchSize = page.locator("#settings-ai-batch-size");
    await batchSize.fill("173");

    // Editing marks the form dirty and enables Save.
    await expect(page.getByTestId("settings-dirty-hint")).toBeVisible();
    await page.getByRole("button", { name: "Save settings" }).click();

    // Save completes and the form is clean again.
    await expect(page.getByTestId("settings-dirty-hint")).toBeHidden();

    // Reload re-mounts the app and reloads settings from the database.
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();
    await expect(page.locator("#settings-ai-batch-size")).toHaveValue("173");
  });
});