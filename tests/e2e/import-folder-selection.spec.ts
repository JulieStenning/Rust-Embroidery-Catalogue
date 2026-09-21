// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import {
  clickNav,
  folderRows,
  resetImportView,
  IMPORT_PATH_PLACEHOLDER,
} from "./helpers";
import {
  cleanupDataRoot,
  EMPTY_IMPORT_SOURCE_PATH,
  MIXED_IMPORT_SOURCE_PATH,
  prepareEmptyImportSource,
  prepareMixedImportSource,
  type MixedImportSource,
} from "./empty-root";

/**
 * Bulk Import step 1 (`#/import`): the folder rows, their disabled states, the
 * Add/Remove row management, the Reset path and the scan hand-off (including its
 * "nothing to import" diagnostics).
 *
 * Everything here is read-only against the catalogue: the scans run against throwaway
 * folders under `tests/e2e/` and none of them import a design. The shared app instance
 * is long-lived and the wizard mirrors its state into a module-level store, so every
 * test resets the wizard before asserting on the default layout.
 */

/** A path inside the repository that never exists. */
const MISSING_FOLDER_PATH = path.join(
  MIXED_IMPORT_SOURCE_PATH,
  "..",
  ".does-not-exist-import-source",
);

/** A plausible typed folder that is never actually scanned. */
const TYPED_FOLDER = "D:\\EmbroideryTests\\ValidFolder";

/** file path -> "size:mtime" snapshot, used to prove source files are untouched. */
function snapshotFolder(root: string): Record<string, string> {
  const snapshot: Record<string, string> = {};
  const walk = (dir: string): void => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(full);
        continue;
      }
      const stat = fs.statSync(full);
      snapshot[full] = `${stat.size}:${stat.mtimeMs}`;
    }
  };
  walk(root);
  return snapshot;
}

test.describe("bulk import step 1", () => {
  test.afterAll(() => {
    cleanupDataRoot(EMPTY_IMPORT_SOURCE_PATH);
    cleanupDataRoot(MIXED_IMPORT_SOURCE_PATH);
  });

  test("renders the default folder-selection chrome", async ({ page }) => {
    await resetImportView(page);

    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();
    await expect(page.getByText("Source Folder(s) *")).toBeVisible();

    const intro = page.locator(".import-step1-intro");
    await expect(intro).toContainText("Sub-folders are included automatically.");
    await expect(intro).toContainText(
      "Your original files are never altered or moved.",
    );
    await expect(intro).toContainText("safely copied into the catalogue");

    await expect(
      page.getByRole("link", { name: "Import help" }),
    ).toHaveAttribute("href", "#/help?section=importing");

    const input = page.locator("#import-root-path");
    await expect(input).toHaveAttribute("placeholder", IMPORT_PATH_PLACEHOLDER);
    await expect(input).toHaveValue("");
    await expect(folderRows(page)).toHaveCount(1);
    await expect(page.getByRole("button", { name: "Browse…" })).toBeEnabled();
  });

  test("gates Scan, Reset, Add another folder and Remove on a non-empty path", async ({
    page,
  }) => {
    await resetImportView(page);

    const scan = page.getByRole("button", { name: "Scan folder(s)" });
    const reset = page.getByRole("button", { name: "Reset", exact: true });
    const addFolder = page.getByRole("button", { name: "Add another folder" });
    const remove = folderRows(page)
      .first()
      .getByRole("button", { name: "Remove" });

    await expect(scan).toBeDisabled();
    await expect(reset).toBeDisabled();
    await expect(addFolder).toBeDisabled();
    await expect(remove).toBeDisabled();

    await page.locator("#import-root-path").fill(TYPED_FOLDER);

    await expect(scan).toBeEnabled();
    await expect(reset).toBeEnabled();
    await expect(addFolder).toBeEnabled();
    await expect(remove).toBeEnabled();

    await reset.click();
    await expect(scan).toBeDisabled();
  });

  test("primary Remove clears the input when it is the only row", async ({
    page,
  }) => {
    await resetImportView(page);

    const input = page.locator("#import-root-path");
    const remove = folderRows(page)
      .first()
      .getByRole("button", { name: "Remove" });

    await expect(remove).toBeDisabled();
    await input.fill(TYPED_FOLDER);
    await expect(remove).toBeEnabled();

    await remove.click();
    await expect(input).toHaveValue("");
    await expect(folderRows(page)).toHaveCount(1);
  });

  test("Add another folder appends a read-only row that can be removed in isolation", async ({
    page,
  }) => {
    await resetImportView(page);

    const input = page.locator("#import-root-path");
    await input.fill(TYPED_FOLDER);
    await page.getByRole("button", { name: "Add another folder" }).click();
    await expect(folderRows(page)).toHaveCount(2);

    const extraRow = folderRows(page).nth(1);
    await expect(extraRow.getByRole("textbox")).toHaveAttribute("readonly", "");
    await expect(extraRow.getByRole("textbox")).toHaveValue("");
    await expect(extraRow.getByRole("button", { name: "Browse…" })).toBeEnabled();
    await expect(extraRow.getByRole("button", { name: "Remove" })).toBeEnabled();

    await extraRow.getByRole("button", { name: "Remove" }).click();
    await expect(folderRows(page)).toHaveCount(1);
    await expect(input).toHaveValue(TYPED_FOLDER);
  });

  test("Reset clears the rows and the input", async ({ page }) => {
    await resetImportView(page);

    const input = page.locator("#import-root-path");
    await input.fill(TYPED_FOLDER);
    await page.getByRole("button", { name: "Add another folder" }).click();
    await expect(folderRows(page)).toHaveCount(2);

    await page.getByRole("button", { name: "Reset", exact: true }).click();

    await expect(folderRows(page)).toHaveCount(1);
    await expect(input).toHaveValue("");
    await expect(
      page.getByRole("button", { name: "Scan folder(s)" }),
    ).toBeDisabled();
    await expect(
      page.getByRole("button", { name: "Reset", exact: true }),
    ).toBeDisabled();
  });

  test("leaving and returning to Import preserves the typed path", async ({
    page,
  }) => {
    await resetImportView(page);
    await page.locator("#import-root-path").fill(TYPED_FOLDER);

    await clickNav(page, "Browse");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    await clickNav(page, "Import");
    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();
    await expect(page.locator("#import-root-path")).toHaveValue(TYPED_FOLDER);
  });

  test("scanning a non-existent folder reports it on step 2", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    await resetImportView(page);

    await page.locator("#import-root-path").fill(MISSING_FOLDER_PATH);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    await expect(
      page.getByText("No supported files discovered in this preview."),
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      page.getByText(
        "The selected folder(s) could not be found on disk. Check that the path is correct and the drive is available.",
      ),
    ).toBeVisible();

    await page.getByRole("button", { name: "Back to Step 1" }).click();
    await expect(page.locator("#import-root-path")).toHaveValue(
      MISSING_FOLDER_PATH,
    );
  });

  test("scanning an empty folder reports no supported files", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const emptySource = prepareEmptyImportSource();
    await resetImportView(page);

    await page.locator("#import-root-path").fill(emptySource);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    await expect(
      page.getByText("No supported files discovered in this preview."),
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      page.getByText(
        "No supported embroidery files (JEF, PES, HUS, DST, EXP, VP3) were found in the selected folder(s).",
      ),
    ).toBeVisible();
  });


  test("includes nested sub-folders and ignores non-embroidery files", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const source: MixedImportSource = prepareMixedImportSource();
    await resetImportView(page);

    await page.locator("#import-root-path").fill(source.root);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();

    await expect(page.getByText("Review scanned files")).toBeVisible({
      timeout: 30_000,
    });

    // Two leaf folders (the source root + `Nested`) holding one supported design each.
    // The two decoys are not counted and never listed.
    await expect(
      page.getByText("2 folder(s) scanned - 2 file(s) found."),
    ).toBeVisible();
    await expect(page.getByText(path.basename(source.rootDesign))).toBeVisible();
    await expect(
      page.getByText(path.basename(source.nestedDesign)),
    ).toBeVisible();
    await expect(page.getByText("playwright-notes.txt")).toHaveCount(0);
    await expect(page.getByText("playwright-manual.pdf")).toHaveCount(0);
  });

  test("cancelling the step 2 review returns to step 1", async ({ page }) => {
    test.setTimeout(120_000);
    const source = prepareMixedImportSource();
    await resetImportView(page);

    await page.locator("#import-root-path").fill(source.root);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();
    await expect(page.getByText("Review scanned files")).toBeVisible({
      timeout: 30_000,
    });

    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.getByText("Source Folder(s) *")).toBeVisible();
  });

  test("scanning leaves the source files untouched", async ({ page }) => {
    test.setTimeout(120_000);
    const source = prepareMixedImportSource();
    const before = snapshotFolder(source.root);
    expect(Object.keys(before)).toHaveLength(4);

    await resetImportView(page);
    await page.locator("#import-root-path").fill(source.root);
    await page.getByRole("button", { name: "Scan folder(s)" }).click();
    await expect(page.getByText("Review scanned files")).toBeVisible({
      timeout: 30_000,
    });

    expect(snapshotFolder(source.root)).toEqual(before);
  });
});

