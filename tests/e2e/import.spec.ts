import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";
import {
  cleanupDataRoot,
  IMPORT_SOURCE_PATH,
  prepareSingleDesignImportSource,
} from "./empty-root";

/**
 * Bulk import coverage: drives the real wizard (typed folder path -> scan ->
 * review -> import) against the throwaway test data root, then proves the
 * design landed in the library.
 *
 * The wizard accepts a typed path, so no native folder picker is needed. The
 * source is a freshly-created, uniquely-named copy of a real design so a new
 * row is always created regardless of what the seed catalogue already contains
 * (the importer deduplicates on stored path and on filename+size+hash).
 */
test.describe("bulk import", () => {
  test.afterAll(() => {
    cleanupDataRoot(IMPORT_SOURCE_PATH);
  });

  test("imports designs from a folder and they appear in Browse", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    const sourceFolder = prepareSingleDesignImportSource();
    const cards = page.locator("article.browse-card");

    // Record the catalogue size before the import.
    await gotoRoute(page, "#/designs");
    const before = await cards.count();

    // Step 1: type the source folder and scan it.
    await gotoRoute(page, "#/import");
    await page
      .getByPlaceholder("Enter path to your embroidery designs folder…")
      .fill(sourceFolder);
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

    // The first import into an empty catalogue asks the user to confirm
    // skipping hoop setup. The shared test catalogue is normally populated
    // (hoops present), so only click the confirmation when it appears.
    const hoopsConfirm = page.getByRole("button", {
      name: "Confirm import without hoop setup",
    });
    if (await hoopsConfirm.isVisible({ timeout: 5_000 }).catch(() => false)) {
      await hoopsConfirm.click();
    }

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
