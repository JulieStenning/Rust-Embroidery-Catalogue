// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";
import { DATA_ROOT_PATH, DATABASE_FILENAME } from "./paths";

/**
 * End-to-end tests for the "Maintenance & File Processing" tab on the Batch Operations
 * admin page (accessed via top navigation "Batch Operations" -> "Maintenance & File Processing"
 * or deep linked via `#/admin/batch-operations`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe.serial("batch operations - maintenance & file processing", () => {
  test("navigates to Batch Operations, switches to Maintenance tab and verifies initial UI state", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Batch Operations");

    // Page title and description
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const tablist = page.getByTestId("batch-operations-tablist");
    await expect(tablist).toBeVisible();

    const maintenanceTab = tablist.getByRole("tab", {
      name: "Maintenance & File Processing",
    });
    const taggingTab = tablist.getByRole("tab", {
      name: "Tagging & Categorisation",
    });

    // Switch to Maintenance tab
    await maintenanceTab.click();
    await expect(maintenanceTab).toHaveAttribute("aria-selected", "true");
    await expect(taggingTab).toHaveAttribute("aria-selected", "false");

    // Batch Operations admin link in navbar is active
    const batchNavlink = mainMenu(page).getByRole("link", {
      name: "Batch Operations",
    });
    await expect(batchNavlink).toHaveClass(/menu-link-active/);

    // Section 1: Target scope heading and options
    await expect(
      page.getByRole("heading", { name: "1. Target scope" }),
    ).toBeVisible();

    const entireCatalogueRadio = page.locator(
      'input[name="maintenance-scope"][value="all"]',
    );
    const missingPreviewsRadio = page.locator(
      'input[name="maintenance-scope"][value="missing_previews"]',
    );

    await expect(entireCatalogueRadio).toBeVisible();
    await expect(entireCatalogueRadio).toBeChecked(); // Default scope
    await expect(missingPreviewsRadio).toBeVisible();

    // Live missing preview badge count (seeded ZZ-broken.pes & ZZ-broken-2.pes)
    await expect(page.getByText(/\d+ designs/i).first()).toBeVisible();

    // Section 2: Maintenance tasks heading
    await expect(
      page.getByRole("heading", { name: "2. Maintenance tasks" }),
    ).toBeVisible();

    // Reconciler card
    await expect(
      page.getByRole("heading", { name: "Find unmatched design files" }),
    ).toBeVisible();
    await expect(
      page.getByText(
        /Scans MachineEmbroideryDesigns for design files that have no record in the catalogue/i,
      ),
    ).toBeVisible();

    // Primary run button is disabled when no maintenance tasks are checked
    const runButton = page.getByRole("button", {
      name: "Review & Start Maintenance",
      exact: true,
    });
    await expect(runButton).toBeVisible();
    await expect(runButton).toBeDisabled();

    // Stop button is disabled when idle
    const stopButton = page.getByRole("button", {
      name: "Stop",
      exact: true,
    });
    await expect(stopButton).toBeVisible();
    await expect(stopButton).toBeDisabled();
  });

  test("validates task selection checkboxes and enables Review & Start Maintenance button", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    const generatePreviewsCheckbox = page.getByRole("checkbox", {
      name: "Generate preview images",
    });
    const colorCountsCheckbox = page.getByRole("checkbox", {
      name: "Recalculate colour / stitch counts",
    });
    const hoopDimensionsCheckbox = page.getByRole("checkbox", {
      name: "Recalculate hoops / dimensions",
    });

    const runButton = page.getByRole("button", {
      name: "Review & Start Maintenance",
      exact: true,
    });

    // Initially none checked -> disabled
    await expect(generatePreviewsCheckbox).not.toBeChecked();
    await expect(colorCountsCheckbox).not.toBeChecked();
    await expect(hoopDimensionsCheckbox).not.toBeChecked();
    await expect(runButton).toBeDisabled();

    // Check "Generate preview images" -> enabled
    await generatePreviewsCheckbox.check();
    await expect(generatePreviewsCheckbox).toBeChecked();
    await expect(runButton).toBeEnabled();

    // Uncheck "Generate preview images" -> disabled again
    await generatePreviewsCheckbox.uncheck();
    await expect(runButton).toBeDisabled();

    // Check "Recalculate hoops / dimensions" and "Recalculate colour / stitch counts" -> enabled
    await colorCountsCheckbox.check();
    await hoopDimensionsCheckbox.check();
    await expect(colorCountsCheckbox).toBeChecked();
    await expect(hoopDimensionsCheckbox).toBeChecked();
    await expect(runButton).toBeEnabled();
  });

  test("displays maintenance confirmation modal and cancels cleanly", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    // Select a task so run button is enabled
    const hoopDimensionsCheckbox = page.getByRole("checkbox", {
      name: "Recalculate hoops / dimensions",
    });
    await hoopDimensionsCheckbox.check();

    const runButton = page.getByRole("button", {
      name: "Review & Start Maintenance",
      exact: true,
    });
    await runButton.click();

    // Confirmation modal opens
    const modal = page.getByTestId("maintenance-confirm-modal");
    await expect(modal).toBeVisible();
    await expect(
      modal.getByRole("heading", { name: "Ready to Run Maintenance" }),
    ).toBeVisible();

    // Modal contents
    await expect(
      modal.locator("p", { hasText: "Target Scope:" }),
    ).toContainText("Entire catalogue");
    await expect(modal.getByText("Maintenance Tasks:")).toBeVisible();
    await expect(
      modal.getByText("Recalculate hoops / dimensions"),
    ).toBeVisible();

    // Cancel modal
    const cancelButton = modal.getByRole("button", {
      name: "Cancel",
      exact: true,
    });
    await cancelButton.click();
    await expect(modal).not.toBeVisible();
  });

  test("executes maintenance tasks against Entire catalogue, updates last run summary and backfill log", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    // Ensure "Entire catalogue" scope is selected
    await page.locator('input[name="maintenance-scope"][value="all"]').check();

    // Check hoop dimensions task only for fast execution across full catalogue
    await page
      .getByRole("checkbox", { name: "Generate preview images" })
      .uncheck();
    await page
      .getByRole("checkbox", { name: "Recalculate colour / stitch counts" })
      .uncheck();
    await page
      .getByRole("checkbox", { name: "Recalculate hoops / dimensions" })
      .check();

    // Open confirmation modal and start maintenance
    await page
      .getByRole("button", {
        name: "Review & Start Maintenance",
        exact: true,
      })
      .click();

    const modal = page.getByTestId("maintenance-confirm-modal");
    await expect(modal).toBeVisible();

    const startButton = modal.getByRole("button", {
      name: "Start Maintenance",
      exact: true,
    });
    await startButton.click();

    // Modal closes
    await expect(modal).not.toBeVisible();

    // Completion toast appears (55 operations, 2 errors from seeded ZZ-broken*.pes files)
    await expect(
      page.getByText(/Maintenance complete: \d+ operations, \d+ errors\./i),
    ).toBeVisible({ timeout: 60_000 });

    // "Last run summary" card appears
    await expect(page.getByText("Last run summary")).toBeVisible();
    await expect(page.getByText(/Operations:\s*\d+/i)).toBeVisible();
    await expect(page.getByText(/Errors:\s*\d+/i)).toBeVisible();
    await expect(page.getByText(/Tasks run:\s*hoop_dimensions/i)).toBeVisible();

    // Backfill log contains entries
    const logDetails = page
      .locator("details")
      .filter({ hasText: /Backfill log/i });
    await expect(logDetails).toBeVisible();
    await logDetails.locator("summary").click();
    await expect(logDetails.locator(".font-mono").first()).toBeVisible();
  });

  test("runs missing previews maintenance scope and provides Review in Browse link for failed thumbnails", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    // Select "Designs missing preview images only"
    const missingPreviewsRadio = page.locator(
      'input[name="maintenance-scope"][value="missing_previews"]',
    );
    await missingPreviewsRadio.check();
    await expect(missingPreviewsRadio).toBeChecked();

    // Select "Generate preview images"
    await page
      .getByRole("checkbox", { name: "Generate preview images" })
      .check();
    await page
      .getByRole("checkbox", { name: "Recalculate colour / stitch counts" })
      .uncheck();
    await page
      .getByRole("checkbox", { name: "Recalculate hoops / dimensions" })
      .uncheck();

    // Open confirmation modal and verify scope text includes count
    await page
      .getByRole("button", {
        name: "Review & Start Maintenance",
        exact: true,
      })
      .click();

    const modal = page.getByTestId("maintenance-confirm-modal");
    await expect(modal).toBeVisible();
    await expect(
      modal.locator("p", { hasText: "Target Scope:" }),
    ).toContainText("Designs missing a preview image");

    await modal
      .getByRole("button", { name: "Start Maintenance", exact: true })
      .click();
    await expect(modal).not.toBeVisible();

    // Completion toast
    await expect(
      page.getByText(/Maintenance complete: \d+ operations/i).last(),
    ).toBeVisible({ timeout: 30_000 });

    // Last run summary contains Needs attention report (from seeded ZZ-broken*.pes files)
    await expect(page.getByText("Last run summary")).toBeVisible();
    await expect(
      page.getByText(/Needs attention:\s*\d+\s*before\s*→\s*\d+\s*after/i),
    ).toBeVisible();

    // "Review these in Browse" link is present
    const reviewInBrowseButton = page.getByRole("button", {
      name: "Review these in Browse",
      exact: true,
    });
    await expect(reviewInBrowseButton).toBeVisible();

    // Clicking "Review these in Browse" routes to Browse Designs with Needs attention filter
    await reviewInBrowseButton.click();
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    // Assert that the Needs attention filter active badge or broken cards are visible
    await expect(page.locator("article.browse-card").first()).toBeVisible();
  });

  test("scans for unmatched files, detects uncatalogued designs, and tests prompt dismissal", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    // Click "Scan for unmatched files" button
    const scanButton = page.getByTestId("scan-unmatched-button");
    await expect(scanButton).toBeVisible();
    await scanButton.click();

    // Prompt appears with detected uncatalogued count
    const prompt = page.getByTestId("unmatched-files-prompt");
    await expect(prompt).toBeVisible({ timeout: 10_000 });
    await expect(
      prompt.getByRole("heading", { name: "Unmatched files found" }),
    ).toBeVisible();
    await expect(
      prompt.getByText(
        /\d+ design file\(s\) on disk have no record in the catalogue/i,
      ),
    ).toBeVisible();

    // Test Dismiss button
    const dismissButton = prompt.getByRole("button", {
      name: "Dismiss",
      exact: true,
    });
    await expect(dismissButton).toBeVisible();
    await dismissButton.click();
    await expect(prompt).not.toBeVisible();
  });

  test("imports unmatched design files and verifies clean reconciliation status", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const maintenanceTab = page
      .getByTestId("batch-operations-tablist")
      .getByRole("tab", { name: "Maintenance & File Processing" });
    await maintenanceTab.click();

    // Scan for unmatched files
    const scanButton = page.getByTestId("scan-unmatched-button");
    await scanButton.click();

    const prompt = page.getByTestId("unmatched-files-prompt");
    await expect(prompt).toBeVisible({ timeout: 10_000 });

    const importButton = prompt.getByRole("button", {
      name: /Import \d+ file\(s\)/i,
    });
    await expect(importButton).toBeVisible();
    await importButton.click();

    // Toast confirms import
    await expect(
      page.getByText(/Imported \d+ unmatched file\(s\)\./i),
    ).toBeVisible({ timeout: 45_000 });

    // Prompt is dismissed automatically after import
    await expect(prompt).not.toBeVisible();

    // Verify automatic tagging (file, folder, and stitching tags) applied to imported designs
    const dbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
    if (fs.existsSync(dbPath)) {
      const db = new DatabaseSync(dbPath);
      const taggedDesigns = db
        .prepare(
          "SELECT d.id, d.filename, COUNT(dt.tag_id) AS tag_count FROM designs d JOIN design_tags dt ON dt.design_id = d.id WHERE d.id > 56 GROUP BY d.id",
        )
        .all() as { id: number; filename: string; tag_count: number }[];
      expect(taggedDesigns.length).toBeGreaterThan(0);
      for (const row of taggedDesigns) {
        expect(row.tag_count).toBeGreaterThan(0);
      }
      db.close();
    }

    // Subsequent scan verifies reconciliation (all supported files on disk are catalogued)
    await scanButton.click();
    await expect(
      page.getByText(/No unmatched design files found/i),
    ).toBeVisible({ timeout: 15_000 });
  });

  test.afterAll(async () => {
    // Delete any imported unmatched designs to return catalogue to 55-design seed baseline
    const dbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
    if (fs.existsSync(dbPath)) {
      try {
        const db = new DatabaseSync(dbPath);
        db.exec("DELETE FROM designs WHERE id > 56");
        // Reset recommended hoop back to Hoop B for Cake 3
        db.exec(
          "UPDATE designs SET hoop_id = (SELECT id FROM hoops WHERE name = 'Hoop B') WHERE filename LIKE 'Cake 3%'",
        );
        db.close();
      } catch {
        // best effort cleanup
      }
    }
  });
});
