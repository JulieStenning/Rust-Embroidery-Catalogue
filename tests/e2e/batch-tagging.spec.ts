// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";
import { DATA_ROOT_PATH, DATABASE_FILENAME } from "./paths";

/**
 * End-to-end tests for the "Tagging & Categorisation" tab on the Batch Operations
 * admin page (accessed via top navigation "Admin: Batch Operations" or `#/admin/batch-operations`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe.serial("batch operations - tagging & categorisation", () => {
  test("navigates to Batch Operations from the top menu link and defaults to Tagging tab", async ({
    page,
  }) => {
    // Navigate from Browse via top menu link
    await gotoRoute(page, "#/designs");
    await clickNav(page, "Batch Operations");

    // Page title and description are rendered
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Automated AI categorisation, rule-based tagging, and library file maintenance.",
      ),
    ).toBeVisible();

    // Mode tablist renders with "Tagging & Categorisation" active by default
    const tablist = page.getByTestId("batch-operations-tablist");
    await expect(tablist).toBeVisible();

    const taggingTab = tablist.getByRole("tab", {
      name: "Tagging & Categorisation",
    });
    const maintenanceTab = tablist.getByRole("tab", {
      name: "Maintenance & File Processing",
    });

    await expect(taggingTab).toBeVisible();
    await expect(taggingTab).toHaveAttribute("aria-selected", "true");
    await expect(maintenanceTab).toBeVisible();
    await expect(maintenanceTab).toHaveAttribute("aria-selected", "false");

    // Batch Operations admin link in navbar is active
    const batchNavlink = mainMenu(page).getByRole("link", {
      name: "Batch Operations",
    });
    await expect(batchNavlink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/batch-operations and supports tab switching", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const tablist = page.getByTestId("batch-operations-tablist");
    const taggingTab = tablist.getByRole("tab", {
      name: "Tagging & Categorisation",
    });
    const maintenanceTab = tablist.getByRole("tab", {
      name: "Maintenance & File Processing",
    });

    // Switch to Maintenance tab
    await maintenanceTab.click();
    await expect(maintenanceTab).toHaveAttribute("aria-selected", "true");
    await expect(taggingTab).toHaveAttribute("aria-selected", "false");
    await expect(
      page.getByRole("heading", { name: "1. Target scope" }),
    ).toBeVisible();

    // Switch back to Tagging tab
    await taggingTab.click();
    await expect(taggingTab).toHaveAttribute("aria-selected", "true");
    await expect(
      page.getByRole("heading", { name: "1. What do you want to do?" }),
    ).toBeVisible();
  });

  test("renders missing API key banner and disables AI goals when no key is configured", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    // Missing API key banner
    await expect(
      page.getByText(
        "No Google API key is configured in Settings. Gemini Vision tagging will be skipped. File & Folder Rules always run.",
      ),
    ).toBeVisible();

    // Settings helper link under Step 1
    const settingsLink = page
      .getByText("Configure a Gemini API key in")
      .getByRole("link", { name: "Settings" });
    await expect(settingsLink).toBeVisible();
    await expect(settingsLink).toHaveAttribute(
      "href",
      "#/admin/system/settings",
    );

    // Goal 1: File & Folder rules (Enabled & Selected by default)
    const fileFolderRadio = page.locator(
      'input[name="tagging-goal"][value="file_folder"]',
    );
    await expect(fileFolderRadio).toBeVisible();
    await expect(fileFolderRadio).toBeEnabled();
    await expect(fileFolderRadio).toBeChecked();

    // Goal 2 & 3: Gemini Vision and Full re-scan (Disabled without API key)
    const aiVisionRadio = page.locator(
      'input[name="tagging-goal"][value="ai_vision"]',
    );
    const fullRescanRadio = page.locator(
      'input[name="tagging-goal"][value="full_rescan"]',
    );

    await expect(aiVisionRadio).toBeDisabled();
    await expect(fullRescanRadio).toBeDisabled();
  });

  test("renders 3-step workflow with live candidate counts and advanced options", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    // Step 1: Goal
    await expect(
      page.getByRole("heading", { name: "1. What do you want to do?" }),
    ).toBeVisible();

    // Step 2: Scope
    await expect(
      page.getByRole("heading", {
        name: "2. Which designs should be processed?",
      }),
    ).toBeVisible();

    const untaggedScope = page.locator(
      'input[name="tagging-scope"][value="untagged"]',
    );
    const folderScope = page.locator(
      'input[name="tagging-scope"][value="folder"]',
    );
    const allScope = page.locator('input[name="tagging-scope"][value="all"]');

    await expect(untaggedScope).toBeVisible();
    await expect(untaggedScope).toBeChecked(); // Default scope
    await expect(folderScope).toBeVisible();
    await expect(allScope).toBeVisible();

    // Live scope candidate counts are populated
    await expect(page.getByText(/\d+ designs/i).first()).toBeVisible();
    await expect(
      page.locator("p", { hasText: "unverified" }).first(),
    ).toBeVisible();

    // Exclude human-verified designs checkbox (Checked by default)
    const excludeVerified = page.getByRole("checkbox", {
      name: "Exclude human-verified designs (recommended)",
    });
    await expect(excludeVerified).toBeVisible();
    await expect(excludeVerified).toBeChecked();

    // Step 3: Merge Strategy
    await expect(
      page.getByRole("heading", {
        name: "3. What should happen to existing tags?",
      }),
    ).toBeVisible();

    const addMerge = page.locator('input[name="tagging-merge"][value="add"]');
    const resetMerge = page.locator(
      'input[name="tagging-merge"][value="reset"]',
    );
    await expect(addMerge).toBeVisible();
    await expect(addMerge).toBeChecked(); // Default merge
    await expect(resetMerge).toBeVisible();

    // Advanced Options section
    await expect(
      page.getByRole("heading", { name: "Advanced options" }),
    ).toBeVisible();

    const stitchingCheckbox = page.getByRole("checkbox", {
      name: "Also detect stitching tags",
    });
    const imagesCheckbox = page.getByRole("checkbox", {
      name: "Also generate preview images",
    });
    const colorCountsCheckbox = page.getByRole("checkbox", {
      name: "Recalculate colour / stitch counts",
    });
    const hoopDimensionsCheckbox = page.getByRole("checkbox", {
      name: "Recalculate hoops / dimensions",
    });

    await expect(stitchingCheckbox).toBeVisible();
    await expect(stitchingCheckbox).not.toBeChecked();
    await expect(imagesCheckbox).toBeVisible();
    await expect(colorCountsCheckbox).toBeVisible();
    await expect(hoopDimensionsCheckbox).toBeVisible();

    // Toggling nested advanced options
    await stitchingCheckbox.check();
    await expect(
      page.getByRole("checkbox", {
        name: "Overwrite stitching tags on already-processed designs",
      }),
    ).toBeVisible();
    await stitchingCheckbox.uncheck();

    await imagesCheckbox.check();
    await expect(
      page.getByRole("checkbox", {
        name: "Regenerate images for all designs, not just those without images",
      }),
    ).toBeVisible();
    await imagesCheckbox.uncheck();
  });

  test("displays pre-flight confirmation modal and cancels cleanly", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    const reviewButton = page.getByRole("button", {
      name: "Review & Start Tagging",
      exact: true,
    });
    await expect(reviewButton).toBeVisible();
    await reviewButton.click();

    // Confirmation modal opens
    const modal = page.getByTestId("tagging-confirm-modal");
    await expect(modal).toBeVisible();
    await expect(
      modal.getByRole("heading", { name: "Ready to Retag" }),
    ).toBeVisible();

    // Summary lines inside modal
    await expect(modal.getByText("Action:")).toBeVisible();
    await expect(modal.getByText("Apply file & folder rules")).toBeVisible();
    await expect(modal.getByText("Target Scope:")).toBeVisible();
    await expect(modal.getByText("Tag Strategy:")).toBeVisible();
    await expect(
      modal.getByText(
        "Keep all existing tags and append any newly discovered tags",
      ),
    ).toBeVisible();
    await expect(modal.getByText("Verified designs:")).toBeVisible();
    await expect(modal.getByText("Excluded")).toBeVisible();

    // Cancel modal
    const cancelButton = modal.getByRole("button", {
      name: "Cancel",
      exact: true,
    });
    await cancelButton.click();
    await expect(modal).not.toBeVisible();
  });

  test("executes offline File & Folder rules, displays live progress, updates summary & log, and persists tags", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    // Select "Untagged designs only" scope with "Add new tags only"
    await page.locator('input[name="tagging-scope"][value="untagged"]').check();
    await page.locator('input[name="tagging-merge"][value="add"]').check();

    // Uncheck verified exclusion to process all untagged designs including Cake 3 - Food.jef
    const excludeVerifiedCheckbox = page.getByRole("checkbox", {
      name: "Exclude human-verified designs (recommended)",
    });
    await excludeVerifiedCheckbox.uncheck();
    await expect(excludeVerifiedCheckbox).not.toBeChecked();

    // Open confirmation modal and start tagging
    await page
      .getByRole("button", { name: "Review & Start Tagging", exact: true })
      .click();
    const modal = page.getByTestId("tagging-confirm-modal");
    await expect(modal).toBeVisible();

    // Verify modal reflects "Included" verified designs
    await expect(modal.getByText("Verified designs:")).toBeVisible();
    await expect(modal.getByText("Included")).toBeVisible();

    const startButton = modal.getByRole("button", {
      name: "Start Tagging",
      exact: true,
    });
    await startButton.click();

    // Modal closes upon starting
    await expect(modal).not.toBeVisible();

    // Completion toast appears
    await expect(
      page.getByText(/Backfill complete: \d+ processed, 0 errors\./i),
    ).toBeVisible({ timeout: 15_000 });

    // "Last run summary" card appears
    await expect(page.getByText("Last run summary")).toBeVisible();
    await expect(page.getByText(/Processed:\s*\d+/i)).toBeVisible();
    await expect(page.getByText(/Image tags:\s*\d+\s*before/i)).toBeVisible();

    // Backfill log contains recorded entries
    const logDetails = page
      .locator("details")
      .filter({ hasText: /Backfill log/i });
    await expect(logDetails).toBeVisible();
    await logDetails.locator("summary").click();
    await expect(logDetails.locator(".font-mono").first()).toBeVisible();

    // Navigate to Browse view and verify that tag rules were applied to designs
    await gotoRoute(page, "#/designs");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    const resetButton = page.getByRole("button", { name: "Reset filters" });
    if (await resetButton.isEnabled()) {
      await resetButton.click();
    }

    // Search for "Cake 3" to find Cake 3 - Food.jef without triggering '-' search negation
    const searchInput = page.getByPlaceholder(/e\.g\. rose/i);
    await searchInput.fill("Cake 3");
    await page.waitForTimeout(600);

    const foodCard = page.locator("article.browse-card", {
      hasText: "Cake 3 - Food.jef",
    });
    await expect(foodCard).toBeVisible();
    await expect(foodCard.locator(".browse-card-tags")).toContainText("Food");

    // Reload page to verify tags persisted across SQLite reloads
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    const reloadedSearchInput = page.getByPlaceholder(/e\.g\. rose/i);
    await reloadedSearchInput.fill("Cake 3");
    await page.waitForTimeout(600);

    const reloadedCard = page.locator("article.browse-card", {
      hasText: "Cake 3 - Food.jef",
    });
    await expect(reloadedCard).toBeVisible();
    await expect(reloadedCard.locator(".browse-card-tags")).toContainText(
      "Food",
    );
  });

  test("unlocks AI Vision goals and dynamic scopes when a mock API key is configured", async ({
    page,
  }) => {
    // 1. Configure mock API key in Settings
    await gotoRoute(page, "#/admin/system/settings");
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();

    // Ensure settings model has finished loading asynchronously
    await expect(page.locator("#settings-data-root")).toHaveValue(
      /MachineEmbroideryDesigns/,
    );

    const apiKeyInput = page.locator("#settings-google-api-key");
    await expect(apiKeyInput).toBeVisible();
    await apiKeyInput.fill("mock-google-api-key-playwright-e2e");

    const saveButton = page.getByRole("button", { name: "Save settings" });
    await expect(page.getByTestId("settings-dirty-hint")).toBeVisible();
    await expect(saveButton).toBeEnabled();
    await saveButton.click();
    await expect(page.getByTestId("settings-dirty-hint")).toBeHidden();

    // 2. Return to Batch Operations
    await gotoRoute(page, "#/admin/batch-operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations", exact: true }),
    ).toBeVisible();

    // Assert banner now reflects configured API key
    await expect(
      page.getByText(
        "API key detected — AI tagging actions are available. Gemini calls may incur charges on your Google account.",
      ),
    ).toBeVisible();

    // Assert AI goals are now enabled
    const aiVisionRadio = page.locator(
      'input[name="tagging-goal"][value="ai_vision"]',
    );
    const fullRescanRadio = page.locator(
      'input[name="tagging-goal"][value="full_rescan"]',
    );

    await expect(aiVisionRadio).toBeEnabled();
    await expect(fullRescanRadio).toBeEnabled();

    // 3. Select "Enrich with Gemini Vision" goal
    await aiVisionRadio.check();
    await expect(aiVisionRadio).toBeChecked();

    // Assert AI Vision scopes are dynamically appended to Step 2
    const visionNotAnalyzedScope = page.locator(
      'input[name="tagging-scope"][value="vision_not_analyzed"]',
    );
    const visionNoMatchScope = page.locator(
      'input[name="tagging-scope"][value="vision_no_match"]',
    );
    const visionAnalyzedScope = page.locator(
      'input[name="tagging-scope"][value="vision_analyzed"]',
    );

    await expect(visionNotAnalyzedScope).toBeVisible();
    await expect(visionNoMatchScope).toBeVisible();
    await expect(visionAnalyzedScope).toBeVisible();

    // 4. Check confirmation modal with AI goal displays estimated time
    await page
      .getByRole("button", { name: "Review & Start Tagging", exact: true })
      .click();
    const modal = page.getByTestId("tagging-confirm-modal");
    await expect(modal).toBeVisible();

    await expect(modal.getByText("Enrich with Gemini Vision")).toBeVisible();
    await expect(modal.getByText("Estimated Time:")).toBeVisible();

    // Cancel modal
    await modal.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(modal).not.toBeVisible();

    // 5. Clean up mock API key from Settings
    await gotoRoute(page, "#/admin/system/settings");
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();
    await expect(page.locator("#settings-data-root")).toHaveValue(
      /MachineEmbroideryDesigns/,
    );

    const cleanupApiKeyInput = page.locator("#settings-google-api-key");
    await expect(cleanupApiKeyInput).toBeVisible();
    await cleanupApiKeyInput.fill("");
    await expect(page.getByTestId("settings-dirty-hint")).toBeVisible();
    const cleanupSaveButton = page.getByRole("button", {
      name: "Save settings",
    });
    await expect(cleanupSaveButton).toBeEnabled();
    await cleanupSaveButton.click();
    await expect(page.getByTestId("settings-dirty-hint")).toBeHidden();
  });

  test.afterAll(() => {
    // Restore pristine seed database tags and verification flags for subsequent test suites
    const dbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
    if (fs.existsSync(dbPath)) {
      try {
        const db = new DatabaseSync(dbPath);
        db.exec("DELETE FROM design_tags WHERE design_id NOT IN (5, 6, 7)");
        db.exec(
          "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 1 WHERE id NOT IN (4, 9, 10, 11)",
        );
        db.exec(
          "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 0 WHERE id = 4",
        );
        db.exec(
          "UPDATE designs SET image_tags_verified = 0, stitching_tags_verified = 0 WHERE id IN (9, 10, 11)",
        );
        db.close();
      } catch (err) {
        console.error("[batch-tagging.spec.ts] afterAll reset error:", err);
      }
    }
  });
});
