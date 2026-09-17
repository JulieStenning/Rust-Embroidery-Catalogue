import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";
import {
  cleanupDataRoot,
  IMPORT_SOURCE_PATH,
  prepareSingleDesignImportSource,
} from "./empty-root";

/**
 * Bulk import coverage: drives the real wizard (typed folder path -> scan ->
 * review -> pre-import -> import) against the throwaway test data root, then proves
 * the design landed in the library.
 *
 * The wizard accepts a typed path, so no native folder picker is needed. The
 * source is a freshly-created, uniquely-named copy of a real design so a new
 * row is always created regardless of what the seed catalogue already contains
 * (the importer deduplicates on stored path and on filename+size+hash).
 *
 * Step 3 also carries explicit negative assertions for the UI that was removed from
 * it (AI-tagging banner, Tier 2/3 counters, "Change in Settings", the 2D/3D preview
 * picker and the Review Hoops/Tags/Sources/Designers buttons). Import is File & Folder
 * Rules only; Visual AI tagging lives on Batch Operations.
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

    // Step 1: type the source folder and scan it.
    await gotoRoute(page, "#/import");
    await page
      .getByPlaceholder("Enter path to your embroidery designs folder…")
      .fill(sourceFolder);
    const sourceFilesBefore = fs.readdirSync(sourceFolder).sort();
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    // Step 2: review the scan.
    await expect(page.getByText("Review scanned files")).toBeVisible({
      timeout: 30_000,
    });
    await expect(
      page.getByText("1 folder(s) scanned - 1 file(s) found."),
    ).toBeVisible();
    await expect(
      page.getByText("Apply to all folders (optional override)"),
    ).toBeVisible();
    await expect(
      page.locator(".import-step2-global-shell select option:checked").first(),
    ).toHaveText("Keep inferred (per folder)");

    const folderShell = page.getByTestId("import-folder-shell");
    await expect(folderShell).toHaveCount(1);
    await expect(folderShell.getByText("All 1 selected")).toBeVisible();

    const continueButton = page.getByRole("button", {
      name: /^Continue with 1 design$/,
    });
    await expect(continueButton).toBeEnabled();

    // Deselect all: the counter disappears and Continue is disabled.
    await page
      .getByRole("button", { name: "Deselect all", exact: true })
      .click();
    await expect(
      page.getByRole("button", { name: "Continue", exact: true }),
    ).toBeDisabled();
    await expect(folderShell.getByText("None selected")).toBeVisible();

    // Select all restores the count.
    await page.getByRole("button", { name: "Select all", exact: true }).click();
    await expect(continueButton).toBeEnabled();
    await continueButton.click();

    // Step 3: "Before You Import" shows the offline File & Folder Rules note only.
    await expect(page.getByText("Before You Import")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByText("Note on Tagging")).toBeVisible();
    await expect(
      page.getByText(
        "THe initial import uses fast, offline File & Folder Rules to index your designs instantly",
      ),
    ).toBeVisible();

    // Negative assertions: the removed controls must never come back.
    const removedUi = [
      "Google AI tagging is enabled for this installation",
      "Tier 2 auto",
      "Tier 3 auto",
      "Change in Settings",
      "2D - Fast flat preview",
      "3D - Detailed stitch simulation",
      "Review Hoops",
      "Review Designers",
    ];
    for (const stale of removedUi) {
      await expect(page.getByText(stale, { exact: false })).toHaveCount(0);
    }

    // Step 4: run the import.
    const importButton = page.getByRole("button", { name: "Import Designs" });
    await expect(importButton).toBeEnabled();
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

    // Copy semantics: importing never moves or alters the source files.
    expect(fs.readdirSync(sourceFolder).sort()).toEqual(sourceFilesBefore);
    for (const name of sourceFilesBefore) {
      expect(fs.existsSync(path.join(sourceFolder, name))).toBe(true);
    }

    // Assert: the imported design is in the library. Browse is paged and the seed
    // catalogue already fills the first page, so search for the unique filename
    // rather than counting cards. The app routes itself after an import, so wait for
    // the Browse search box instead of assuming the route has settled.
    await expect
      .poll(
        async () => {
          if (!page.url().includes("#/designs")) {
            await gotoRoute(page, "#/designs");
          }
          return page
            .locator("#browse-q")
            .isVisible()
            .catch(() => false);
        },
        { timeout: 60_000 },
      )
      .toBe(true);

    const importedStem = path.parse(sourceFilesBefore[0]).name;
    await page.locator("#browse-q").fill(importedStem);

    const importedTitle = page.locator("article.browse-card .browse-card-title");
    await expect
      .poll(async () => importedTitle.count(), { timeout: 30_000 })
      .toBe(1);
    await expect(importedTitle.first()).toContainText(importedStem);
  });
});
