import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";
import { TEST_DESIGNS_PATH } from "./paths";

/**
 * Bulk import coverage: drives the real wizard (typed folder path -> scan ->
 * review -> import) against the throwaway test data root, then proves the
 * designs landed in the library.
 *
 * The wizard accepts a typed path, so no native folder picker is needed. The
 * flow is genuinely slow (scan + copy + DB writes), hence the raised timeout.
 */
test.describe("bulk import", () => {
  test("imports designs from a folder and they appear in Browse", async ({ page }) => {
    test.setTimeout(180_000);

    const cards = page.locator("article.browse-card");

    // The catalogue is empty at the start of a run (see global-setup.ts).
    await gotoRoute(page, "#/designs");
    const before = await cards.count();

    // Step 1: type the source folder and scan it.
    await gotoRoute(page, "#/import");
    await page
      .getByPlaceholder("Enter path to your embroidery designs folder…")
      .fill(TEST_DESIGNS_PATH);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    // Step 2: review the scan, then continue to the precheck.
    const continueButton = page.getByRole("button", {
      name: /^Continue with \d+ design/,
    });
    await expect(continueButton).toBeVisible({ timeout: 30_000 });
    await continueButton.click();

    // Step 3: run the import.
    const importButton = page.getByRole("button", { name: "Import Designs" });
    await expect(importButton).toBeVisible({ timeout: 30_000 });
    await importButton.click();

    // A first import always asks the user to confirm skipping hoop setup (the
    // throwaway test catalogue has no hoops configured).
    const hoopsConfirm = page.getByRole("button", {
      name: "Confirm import without hoop setup",
    });
    await expect(hoopsConfirm).toBeVisible({ timeout: 20_000 });
    await hoopsConfirm.click();

    // The import is finished once the action button is gone.
    await expect(importButton).toBeHidden({ timeout: 150_000 });

    // Assert: the library holds more designs than before. The app routes itself
    // after an import, so re-navigate if it has moved away.
    await expect
      .poll(
        async () => {
          if (!page.url().includes("#/designs")) {
            await gotoRoute(page, "#/designs");
          }
          return cards.count();
        },
        { timeout: 60_000 },
      )
      .toBeGreaterThan(before);
  });
});