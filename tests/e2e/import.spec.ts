// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

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
 * Step 3 verifies the offline File & Folder Rules note. Import is File & Folder
 * Rules only; Gemini Vision tagging lives on Batch Operations.
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
        "The initial import uses fast, offline File & Folder Rules to index your designs instantly",
      ),
    ).toBeVisible();

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

    // Assert: First-import success banner is displayed above browse search on first import.
    const banner = page.getByTestId("first-import-success-banner");
    await expect(banner).toBeVisible({ timeout: 10_000 });
    await expect(banner.getByText("First import complete!")).toBeVisible();
    await expect(
      banner.getByText(
        /Your designs are tagged with fast, offline File & Folder rules/,
      ),
    ).toBeVisible();
    await expect(
      banner.getByRole("link", { name: "Settings" }),
    ).toHaveAttribute("href", "#/admin/system/settings");
    await expect(
      banner.getByRole("link", { name: "Batch Operations" }),
    ).toHaveAttribute("href", "#/admin/batch-operations");

    // Dismiss the banner.
    await banner
      .getByRole("button", { name: "Dismiss first import notice" })
      .click();
    await expect(banner).toBeHidden();

    const importedStem = path.parse(sourceFilesBefore[0]).name;
    await page.locator("#browse-q").fill(importedStem);

    const importedTitle = page.locator(
      "article.browse-card .browse-card-title",
    );
    await expect
      .poll(async () => importedTitle.count(), { timeout: 30_000 })
      .toBe(1);
    await expect(importedTitle.first()).toContainText(importedStem);
  });

  test("allows quick-adding a new designer and source on step 2 review", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    const sourceFolder = prepareSingleDesignImportSource();

    // Step 1: scan folder
    await gotoRoute(page, "#/import");
    await page
      .getByPlaceholder("Enter path to your embroidery designs folder…")
      .fill(sourceFolder);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    // Step 2: review the scan
    await expect(page.getByText("Review scanned files")).toBeVisible({
      timeout: 30_000,
    });

    const uniqueSuffix = Date.now().toString().slice(-4);
    const newDesignerName = `Quick Designer ${uniqueSuffix}`;
    const newSourceName = `Quick Source ${uniqueSuffix}`;

    // Click "+" next to Global Designer
    await page.getByRole("button", { name: "Add new designer" }).click();
    const modal = page.getByTestId("quick-add-entity-modal");
    await expect(modal).toBeVisible();
    await expect(modal.getByRole("heading", { name: "Add New Designer" })).toBeVisible();

    // Type name and submit
    await modal.getByPlaceholder("e.g. Urban Threads").fill(newDesignerName);
    await modal.getByRole("button", { name: "Add Designer" }).click();
    await expect(modal).toBeHidden();

    // Global designer select should now have the newly added designer selected
    await expect(
      page.locator(".import-step2-global-shell select option:checked").first(),
    ).toHaveText(newDesignerName);

    // Click "+" next to Folder Source
    const folderShell = page.getByTestId("import-folder-shell").first();
    await folderShell.getByRole("button", { name: /Add new source for/i }).click();
    await expect(modal).toBeVisible();
    await expect(modal.getByRole("heading", { name: "Add New Source" })).toBeVisible();

    // Type source name and submit
    await modal.getByPlaceholder("e.g. Purchased").fill(newSourceName);
    await modal.getByRole("button", { name: "Add Source" }).click();
    await expect(modal).toBeHidden();

    // Verify folder source select now has the newly added source selected
    const folderSourceSelect = folderShell.locator("select").nth(1);
    await expect(folderSourceSelect.locator("option:checked")).toHaveText(newSourceName);

    // Cancel out or navigate back cleanly
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.getByText("Select one or more folders")).toBeVisible();
  });
});
