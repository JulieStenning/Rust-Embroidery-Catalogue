// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";
import {
  DATA_ROOT_PATH,
  DATABASE_FILENAME,
  DESIGNS_CONTAINER,
  TEST_DB_PATH,
  TEST_DESIGNS_PATH,
} from "./paths";
import type { Page } from "@playwright/test";

/**
 * End-to-end tests for the "Restore" tab on the Backup & Restore page
 * (accessed via top navigation "System" -> "Backup & Restore" -> "Restore" tab or deep linked `#/admin/system/backup`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, SQLite database,
 * file-system restoration, confirmation dialogs, progress tracking, and unmatched files reconciliation.
 */
test.describe.serial("system restore", () => {
  const testRestoreDbDir = path.join(DATA_ROOT_PATH, "e2e-test-restore-db");
  const testRestoreDesignsDir = path.join(
    DATA_ROOT_PATH,
    "e2e-test-restore-designs",
  );
  const liveDbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
  const liveDesignsDir = path.join(DATA_ROOT_PATH, DESIGNS_CONTAINER);

  const validBackupDbFile = path.join(testRestoreDbDir, "valid-catalogue-backup.db");
  const corruptBackupDbFile = path.join(testRestoreDbDir, "corrupt-catalogue-backup.db");
  const unmatchedTestFileName = "Playwright_Unmatched_Design.jef";
  const unmatchedFilePath = path.join(liveDesignsDir, unmatchedTestFileName);

  /**
   * Stub `browse_restore_file` IPC invoke in the WebView2 context to return a simulated path.
   */
  async function stubBrowseRestoreFile(
    page: Page,
    selectedPath: string | null,
    errorMessage: string | null = null,
  ): Promise<void> {
    await page.evaluate(
      ({ pathValue, errorValue }) => {
        const win = window as unknown as {
          __E2E_IPC_STUBS__?: Record<string, (args?: unknown) => unknown>;
        };
        win.__E2E_IPC_STUBS__ = win.__E2E_IPC_STUBS__ || {};
        win.__E2E_IPC_STUBS__["browse_restore_file"] = () => ({
          path: pathValue,
          error: errorValue,
        });
      },
      { pathValue: selectedPath, errorValue: errorMessage },
    );
  }

  test.beforeAll(() => {
    // Setup clean test fixture directories
    if (fs.existsSync(testRestoreDbDir)) {
      fs.rmSync(testRestoreDbDir, { recursive: true, force: true });
    }
    if (fs.existsSync(testRestoreDesignsDir)) {
      fs.rmSync(testRestoreDesignsDir, { recursive: true, force: true });
    }
    fs.mkdirSync(testRestoreDbDir, { recursive: true });
    fs.mkdirSync(testRestoreDesignsDir, { recursive: true });

    // Create a valid snapshot copy of the seed database
    fs.copyFileSync(TEST_DB_PATH, validBackupDbFile);

    // Create a corrupt/invalid snapshot file to test verification rollback
    fs.writeFileSync(
      corruptBackupDbFile,
      "NOT A VALID SQLITE DATABASE FILE FOR CORRUPTION RESTORE TEST",
      "utf-8",
    );

    // Populate test designs backup folder with design files
    if (fs.existsSync(TEST_DESIGNS_PATH)) {
      const designFiles = fs.readdirSync(TEST_DESIGNS_PATH);
      for (const file of designFiles) {
        const src = path.join(TEST_DESIGNS_PATH, file);
        if (fs.statSync(src).isFile()) {
          fs.copyFileSync(src, path.join(testRestoreDesignsDir, file));
        }
      }
    }
  });

  test.afterAll(() => {
    // Clean up temporary test directories
    if (fs.existsSync(testRestoreDbDir)) {
      fs.rmSync(testRestoreDbDir, { recursive: true, force: true });
    }
    if (fs.existsSync(testRestoreDesignsDir)) {
      fs.rmSync(testRestoreDesignsDir, { recursive: true, force: true });
    }
    if (fs.existsSync(unmatchedFilePath)) {
      try {
        fs.unlinkSync(unmatchedFilePath);
      } catch {
        // Ignore unlink error if already imported
      }
    }

    // Ensure pristine database state is restored for subsequent test suites
    if (fs.existsSync(liveDbPath)) {
      try {
        const db = new DatabaseSync(liveDbPath);
        db.exec("DELETE FROM designs WHERE id > 56");
        db.exec(
          "UPDATE designs SET hoop_id = (SELECT id FROM hoops WHERE name = 'Hoop B') WHERE filename LIKE 'Cake 3%'",
        );
        db.close();
      } catch {
        // Best effort cleanup
      }
    }
  });

  test("navigates to Restore tab via System menu and verifies initial UI layout", async ({
    page,
  }) => {
    // 1. Navigate via top menu link to System
    await clickNav(page, "System");

    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    const backupSubTab = page.getByTestId("system-tab-backup");
    await expect(backupSubTab).toBeVisible();
    await backupSubTab.click();

    // Verify System link in navbar remains active
    const systemNavlink = mainMenu(page).getByRole("link", { name: "System" });
    await expect(systemNavlink).toHaveClass(/menu-link-active/);

    // Page title and description
    await expect(
      page.getByRole("heading", { name: "Backup & Restore", exact: true }),
    ).toBeVisible();

    // 2. Select Restore sub-tab
    const pageTablist = page.getByRole("tablist", {
      name: "Backup and restore",
    });
    await expect(pageTablist).toBeVisible();

    const restoreTab = pageTablist.getByRole("tab", { name: "Restore" });
    await expect(restoreTab).toBeVisible();
    await restoreTab.click();
    await expect(restoreTab).toHaveAttribute("aria-selected", "true");

    // 3. Restore warning banner
    await expect(page.locator(".backup-important")).toBeVisible();
    await expect(
      page.getByText(
        "Restoring overwrites live data — a safety copy of your current database is kept before any overwrite",
      ),
    ).toBeVisible();

    // 4. Restore Database card layout
    await expect(
      page.getByRole("heading", { name: "Restore Database", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Replace the live catalogue database with a backup snapshot.",
      ),
    ).toBeVisible();

    const dbFileInput = page.locator("#restore-db-file");
    await expect(dbFileInput).toBeVisible();
    await expect(dbFileInput).toHaveAttribute("readonly", "");
    await expect(dbFileInput).toHaveValue("");

    const chooseFileBtn = page.getByRole("button", { name: "Choose file…" });
    await expect(chooseFileBtn).toBeVisible();
    await expect(chooseFileBtn).toBeEnabled();

    const restoreDbBtn = page.getByRole("button", {
      name: "Restore Database Now",
    });
    await expect(restoreDbBtn).toBeVisible();
    // Initially disabled because no file is selected
    await expect(restoreDbBtn).toBeDisabled();

    // 5. Sync Designs from Backup card layout
    await expect(
      page.getByRole("heading", { name: "Sync Designs from Backup", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText("Copies design files from the backup folder back into"),
    ).toBeVisible();

    // 6. Restore Both card layout
    await expect(
      page.getByRole("heading", { name: "Restore Both", exact: true }),
    ).toBeVisible();
    const restoreBothBtn = page.getByRole("button", { name: "Restore Both" });
    await expect(restoreBothBtn).toBeVisible();
    await expect(restoreBothBtn).toBeDisabled();

    // 7. Find unmatched design files section
    await expect(
      page.getByRole("heading", {
        name: "Find unmatched design files",
        exact: true,
      }),
    ).toBeVisible();
    const scanUnmatchedBtn = page.getByTestId("scan-unmatched-button");
    await expect(scanUnmatchedBtn).toBeVisible();
    await expect(scanUnmatchedBtn).toBeEnabled();
  });

  test("selects a database backup file via file picker and dynamically enables action buttons", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    // Switch to Restore tab
    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    const dbFileInput = page.locator("#restore-db-file");
    const chooseFileBtn = page.getByRole("button", { name: "Choose file…" });
    const restoreDbBtn = page.getByRole("button", {
      name: "Restore Database Now",
    });

    // Intercept browse_restore_file to return our valid test backup file
    await stubBrowseRestoreFile(page, validBackupDbFile);

    await chooseFileBtn.click();

    // Verify the input has updated with the selected file path
    await expect(dbFileInput).toHaveValue(validBackupDbFile);

    // Verify Restore Database Now button is now enabled
    await expect(restoreDbBtn).toBeEnabled();
  });

  test("displays the destructive confirmation modal and handles cancellation cleanly", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    // Ensure a DB file is selected
    await stubBrowseRestoreFile(page, validBackupDbFile);
    await page.getByRole("button", { name: "Choose file…" }).click();

    const restoreDbBtn = page.getByRole("button", {
      name: "Restore Database Now",
    });
    await expect(restoreDbBtn).toBeEnabled();

    // Click to open confirmation modal
    await restoreDbBtn.click();

    // Modal is visible
    const modalTitle = page.locator("#confirm-restore-modal-title");
    await expect(modalTitle).toBeVisible();
    await expect(modalTitle).toHaveText("Are you sure you want to restore?");

    await expect(
      page.getByText(
        "Restoring overwrites current data and cannot be undone from this screen.",
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        "The current database will be replaced with the selected backup snapshot.",
      ),
    ).toBeVisible();

    const cancelBtn = page.getByRole("button", { name: "Cancel", exact: true });
    const confirmBtn = page.getByRole("button", {
      name: "Restore database",
      exact: true,
    });
    await expect(cancelBtn).toBeVisible();
    await expect(confirmBtn).toBeVisible();

    // Test cancellation via Cancel button
    await cancelBtn.click();
    await expect(modalTitle).toBeHidden();

    // Re-open and test cancellation via Escape key
    await restoreDbBtn.click();
    await expect(modalTitle).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(modalTitle).toBeHidden();
  });

  test("executes a real Database Restore from snapshot and confirms success toast", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    // Select valid backup snapshot file
    await stubBrowseRestoreFile(page, validBackupDbFile);
    await page.getByRole("button", { name: "Choose file…" }).click();

    // Trigger restore and confirm in modal
    await page.getByRole("button", { name: "Restore Database Now" }).click();
    const confirmBtn = page.getByRole("button", {
      name: "Restore database",
      exact: true,
    });
    await expect(confirmBtn).toBeVisible();
    await confirmBtn.click();

    // Assert success toast containing design count
    await expect(page.getByText(/Database restored \(\d+ designs\)\./)).toBeVisible({
      timeout: 15_000,
    });
  });

  test("handles corrupt database restore with automatic safety rollback banner", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    // Select corrupt backup snapshot file
    await stubBrowseRestoreFile(page, corruptBackupDbFile);
    await page.getByRole("button", { name: "Choose file…" }).click();

    // Trigger restore and confirm in modal
    await page.getByRole("button", { name: "Restore Database Now" }).click();
    await page
      .getByRole("button", { name: "Restore database", exact: true })
      .click();

    // Assert error toast or rollback banner
    await expect(
      page.getByText("Restore rolled back", { exact: true }),
    ).toBeVisible({ timeout: 15_000 });
    await expect(
      page.getByText(
        "Your previous database was automatically restored after the restore failed verification.",
      ),
    ).toBeVisible();
  });

  test("executes incremental Designs Sync and verifies progress and toasts", async ({
    page,
  }) => {
    // 1. First configure the designs backup destination in Backup tab
    await gotoRoute(page, "#/admin/system/backup");

    const backupTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Backup" });
    await backupTab.click();

    const designsInput = page.locator("#backup-designs-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });

    await designsInput.fill(testRestoreDesignsDir);
    if (await saveBtn.isEnabled()) {
      await saveBtn.click();
      await expect(
        page.getByText("Backup destinations saved."),
      ).toBeVisible();
    }

    // 2. Switch to Restore tab
    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    const syncDesignsBtn = page.getByRole("button", {
      name: "Sync designs from backup",
    });
    await expect(syncDesignsBtn).toBeEnabled();

    // Click Sync designs (runs without destructive modal)
    await syncDesignsBtn.click();

    // Assert success toast
    await expect(
      page.getByText(/Designs restored: \d+ copied, \d+ skipped\./),
    ).toBeVisible({ timeout: 20_000 });
  });

  test("executes Restore Both with multi-step pipeline tracking", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const backupTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Backup" });
    await backupTab.click();

    // Ensure designs destination is set
    const designsInput = page.locator("#backup-designs-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });
    await designsInput.fill(testRestoreDesignsDir);
    if (await saveBtn.isEnabled()) {
      await saveBtn.click();
      await expect(
        page.getByText("Backup destinations saved."),
      ).toBeVisible();
    }

    // Switch to Restore tab
    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    // Select valid database file
    await stubBrowseRestoreFile(page, validBackupDbFile);
    await page.getByRole("button", { name: "Choose file…" }).click();

    const restoreBothBtn = page.getByRole("button", { name: "Restore Both" });
    await expect(restoreBothBtn).toBeEnabled();

    // Trigger Restore Both and confirm in modal
    await restoreBothBtn.click();

    const confirmBtn = page.getByRole("button", {
      name: "Restore both",
      exact: true,
    });
    await expect(confirmBtn).toBeVisible();
    await confirmBtn.click();

    // Assert completion toast
    await expect(
      page.getByText(/Database restored \(\d+ designs\)\./),
    ).toBeVisible({ timeout: 25_000 });
  });

  test("scans for unmatched design files and imports newly discovered files", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await gotoRoute(page, "#/admin/system/backup");

    const restoreTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Restore" });
    await restoreTab.click();

    const scanBtn = page.getByTestId("scan-unmatched-button");
    await expect(scanBtn).toBeVisible();

    const promptCard = page.getByTestId("unmatched-files-prompt");

    // 1. Initial scan: if unmatched files are detected (e.g. from previous tests), import them to reach clean baseline
    await scanBtn.click();
    const promptVisible = await promptCard
      .isVisible({ timeout: 2000 })
      .catch(() => false);
    if (promptVisible) {
      const initialImportBtn = promptCard.getByRole("button", {
        name: /Import \d+ file\(s\)/,
      });
      await expect(initialImportBtn).toBeVisible();
      await initialImportBtn.click();
      await expect(
        page.getByText(/Imported \d+ unmatched file\(s\)\./),
      ).toBeVisible({ timeout: 60_000 });
      await expect(promptCard).toBeHidden();

      // Scan again to assert zero unmatched files baseline
      await scanBtn.click();
      await expect(
        page.getByText(/No unmatched design files found \(checked \d+\)\./),
      ).toBeVisible({ timeout: 15_000 });
    } else {
      await expect(
        page.getByText(/No unmatched design files found \(checked \d+\)\./),
      ).toBeVisible({ timeout: 15_000 });
    }

    // 2. Place an unmatched file directly into MachineEmbroideryDesigns on disk
    const sampleSeedFile = path.join(TEST_DESIGNS_PATH, "Cake 3.jef");
    if (fs.existsSync(sampleSeedFile)) {
      fs.copyFileSync(sampleSeedFile, unmatchedFilePath);
    } else {
      // Fallback dummy jef file
      fs.writeFileSync(unmatchedFilePath, "dummy embroidery content");
    }

    // 3. Scan again to detect the unmatched file
    await scanBtn.click();

    await expect(promptCard).toBeVisible({ timeout: 15_000 });
    await expect(
      page.getByRole("heading", { name: "Unmatched files found" }),
    ).toBeVisible();
    await expect(
      page.getByText(/design file\(s\) on disk have no record in the catalogue/),
    ).toBeVisible();

    // 4. Execute batch import of the unmatched file
    const importBtn = promptCard.getByRole("button", {
      name: /Import \d+ file\(s\)/,
    });
    await expect(importBtn).toBeVisible();
    await importBtn.click();

    // Assert import success toast
    await expect(
      page.getByText(/Imported \d+ unmatched file\(s\)\./),
    ).toBeVisible({ timeout: 60_000 });

    // Prompt card dismisses automatically upon completion
    await expect(promptCard).toBeHidden();
  });

  test("cleans up backup destinations and restores clean state", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    // Switch to Backup tab to reset destination configurations
    const backupTab = page
      .getByRole("tablist", { name: "Backup and restore" })
      .getByRole("tab", { name: "Backup" });
    await backupTab.click();

    const dbInput = page.locator("#backup-db-destination");
    const designsInput = page.locator("#backup-designs-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });

    await dbInput.fill("");
    await designsInput.fill("");

    if (await saveBtn.isEnabled()) {
      await saveBtn.click();
      await expect(
        page.getByText("Backup destinations saved."),
      ).toBeVisible();
    }
  });
});
