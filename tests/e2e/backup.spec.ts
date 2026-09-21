// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";
import { DATA_ROOT_PATH } from "./paths";

/**
 * End-to-end tests for the "Backup" tab on the Backup & Restore page
 * (accessed via top navigation "System" -> "Backup & Restore" tab or deep linked `#/admin/system/backup`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, SQLite database,
 * and real file system operations for database and designs backup snapshots.
 */
test.describe.serial("system backup", () => {
  const testDbBackupDir = path.join(DATA_ROOT_PATH, "e2e-test-backup-db");
  const testDesignsBackupDir = path.join(
    DATA_ROOT_PATH,
    "e2e-test-backup-designs",
  );

  test.beforeAll(() => {
    // Ensure clean test directories
    if (fs.existsSync(testDbBackupDir)) {
      fs.rmSync(testDbBackupDir, { recursive: true, force: true });
    }
    if (fs.existsSync(testDesignsBackupDir)) {
      fs.rmSync(testDesignsBackupDir, { recursive: true, force: true });
    }
    fs.mkdirSync(testDbBackupDir, { recursive: true });
    fs.mkdirSync(testDesignsBackupDir, { recursive: true });
  });

  test.afterAll(() => {
    // Clean up temporary backup directories
    if (fs.existsSync(testDbBackupDir)) {
      fs.rmSync(testDbBackupDir, { recursive: true, force: true });
    }
    if (fs.existsSync(testDesignsBackupDir)) {
      fs.rmSync(testDesignsBackupDir, { recursive: true, force: true });
    }
  });

  test("navigates to Backup & Restore from the System menu and verifies initial UI state", async ({
    page,
  }) => {
    // Navigate via top menu link to System
    await clickNav(page, "System");

    // Page title and system tabs
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    const systemTablist = page.getByTestId("system-maintenance-tablist");
    await expect(systemTablist).toBeVisible();

    const backupSubTab = page.getByTestId("system-tab-backup");
    await expect(backupSubTab).toBeVisible();

    // Click Backup & Restore sub-tab
    await backupSubTab.click();
    await expect(backupSubTab).toHaveAttribute("aria-selected", "true");

    // Verify System link in navbar remains active
    const systemNavlink = mainMenu(page).getByRole("link", { name: "System" });
    await expect(systemNavlink).toHaveClass(/menu-link-active/);

    // Backup & Restore page title and description
    await expect(
      page.getByRole("heading", { name: "Backup & Restore", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Back up your catalogue database and embroidery design files to folders of your choice",
      ),
    ).toBeVisible();

    // In-page sub-tablist for Backup vs Restore
    const pageTablist = page.getByRole("tablist", {
      name: "Backup and restore",
    });
    await expect(pageTablist).toBeVisible();

    const backupTab = pageTablist.getByRole("tab", { name: "Backup" });
    const restoreTab = pageTablist.getByRole("tab", { name: "Restore" });
    await expect(backupTab).toBeVisible();
    await expect(backupTab).toHaveAttribute("aria-selected", "true");
    await expect(restoreTab).toBeVisible();
    await expect(restoreTab).toHaveAttribute("aria-selected", "false");

    // Important banner
    await expect(page.locator(".backup-important")).toBeVisible();
    await expect(
      page.getByText(
        "Ensure backup folders reside on a separate drive from your library.",
      ),
    ).toBeVisible();

    // Destination inputs and Browse buttons
    const dbInput = page.locator("#backup-db-destination");
    const designsInput = page.locator("#backup-designs-destination");
    await expect(dbInput).toBeVisible();
    await expect(designsInput).toBeVisible();

    const browseButtons = page.getByRole("button", { name: "Browse…" });
    await expect(browseButtons).toHaveCount(2);

    // Save destinations button is initially disabled (no unsaved changes)
    const saveBtn = page.getByRole("button", { name: "Save destinations" });
    await expect(saveBtn).toBeVisible();
    await expect(saveBtn).toBeDisabled();

    // Database Backup card
    await expect(
      page.getByRole("heading", { name: "Database Backup", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText("Creates a timestamped copy of your SQLite database"),
    ).toBeVisible();

    // Designs Backup card
    await expect(
      page.getByRole("heading", { name: "Designs Backup", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Runs an incremental mirror backup of the designs folder.",
      ),
    ).toBeVisible();

    // Backup Everything Now card
    await expect(
      page.getByRole("heading", { name: "Backup Everything Now", exact: true }),
    ).toBeVisible();
  });

  test("deep links directly to #/admin/system/backup and supports tab switching", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    await expect(
      page.getByRole("heading", { name: "Backup & Restore", exact: true }),
    ).toBeVisible();

    const pageTablist = page.getByRole("tablist", {
      name: "Backup and restore",
    });
    const backupTab = pageTablist.getByRole("tab", { name: "Backup" });
    const restoreTab = pageTablist.getByRole("tab", { name: "Restore" });

    // Switch to Restore tab
    await restoreTab.click();
    await expect(restoreTab).toHaveAttribute("aria-selected", "true");
    await expect(backupTab).toHaveAttribute("aria-selected", "false");

    // Restore cards are visible
    await expect(
      page.getByRole("heading", { name: "Restore Database" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Sync Designs from Backup" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Restore Both" }),
    ).toBeVisible();

    // Switch back to Backup tab
    await backupTab.click();
    await expect(backupTab).toHaveAttribute("aria-selected", "true");
    await expect(restoreTab).toHaveAttribute("aria-selected", "false");

    // Backup cards are visible again
    await expect(
      page.getByRole("heading", { name: "Backup Destinations" }),
    ).toBeVisible();
  });

  test("tracks dirty state and enables Save button only when changes exist", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const dbInput = page.locator("#backup-db-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });

    const originalDbVal = await dbInput.inputValue();

    // Modify DB destination -> button becomes enabled
    await dbInput.fill("C:\\TempBackupTest");
    await expect(saveBtn).toBeEnabled();

    // Revert back -> button becomes disabled
    await dbInput.fill(originalDbVal);
    await expect(saveBtn).toBeDisabled();

    // Modify Designs destination -> button becomes enabled
    const designsInput = page.locator("#backup-designs-destination");
    const originalDesignsVal = await designsInput.inputValue();

    await designsInput.fill("C:\\TempDesignsTest");
    await expect(saveBtn).toBeEnabled();

    // Revert back -> button becomes disabled
    await designsInput.fill(originalDesignsVal);
    await expect(saveBtn).toBeDisabled();
  });

  test("configures backup destinations and persists them across page reload", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const dbInput = page.locator("#backup-db-destination");
    const designsInput = page.locator("#backup-designs-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });

    // Fill configured test directories
    await dbInput.fill(testDbBackupDir);
    await designsInput.fill(testDesignsBackupDir);

    await expect(saveBtn).toBeEnabled();
    await saveBtn.click();

    // Confirmation toast
    await expect(
      page.getByText("Backup destinations saved."),
    ).toBeVisible();
    await expect(saveBtn).toBeDisabled();

    // Action buttons are now enabled
    const dbBackupBtn = page.getByRole("button", {
      name: "Backup Database Now",
    });
    const designsBackupBtn = page.getByRole("button", {
      name: "Run incremental backup",
    });
    const bothBackupBtn = page.getByRole("button", {
      name: "Backup Everything Now",
    });

    await expect(dbBackupBtn).toBeEnabled();
    await expect(designsBackupBtn).toBeEnabled();
    await expect(bothBackupBtn).toBeEnabled();

    // Reload page to verify SQLite round-trip persistence
    await page.reload();

    await expect(
      page.getByRole("heading", { name: "Backup & Restore", exact: true }),
    ).toBeVisible();
    await expect(page.locator("#backup-db-destination")).toHaveValue(
      testDbBackupDir,
    );
    await expect(page.locator("#backup-designs-destination")).toHaveValue(
      testDesignsBackupDir,
    );
    await expect(
      page.getByRole("button", { name: "Backup Database Now" }),
    ).toBeEnabled();
  });

  test("executes Database Backup and creates a snapshot file on disk", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const dbBackupBtn = page.getByRole("button", {
      name: "Backup Database Now",
    });
    await expect(dbBackupBtn).toBeEnabled();

    // Click backup database
    await dbBackupBtn.click();

    // Assert success toast
    await expect(
      page.getByText(/Database backup created:.*\.db \(\d+(\.\d+)? MB\)\./),
    ).toBeVisible();

    // Verify on disk that a .db file was created
    const createdFiles = fs.readdirSync(testDbBackupDir);
    const dbFiles = createdFiles.filter((f) => f.endsWith(".db"));
    expect(dbFiles.length).toBeGreaterThanOrEqual(1);

    const backupFilePath = path.join(testDbBackupDir, dbFiles[0]);
    const fileStat = fs.statSync(backupFilePath);
    expect(fileStat.size).toBeGreaterThan(0);

    // Card displays a formatted Last backup timestamp (not "—")
    await expect(page.getByText(/Last backup: \d+/)).toBeVisible();
  });

  test("executes incremental Designs Backup and verifies mirrored files on disk", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const designsBackupBtn = page.getByRole("button", {
      name: "Run incremental backup",
    });
    await expect(designsBackupBtn).toBeEnabled();

    // Initial designs backup
    await designsBackupBtn.click();

    // Assert success toast with copied count > 0
    await expect(
      page.getByText(
        /Designs backup complete: scanned \d+, copied [1-9]\d*, updated 0, unchanged 0, archived 0\./,
      ),
    ).toBeVisible();

    // Verify copied files exist on disk in test destination
    const copiedFiles = fs.readdirSync(testDesignsBackupDir);
    expect(copiedFiles.length).toBeGreaterThan(0);
    expect(
      copiedFiles.some(
        (f) => f.endsWith(".jef") || f.endsWith(".pes") || f.includes("Cake"),
      ),
    ).toBe(true);

    // Card displays a formatted Last sync timestamp
    await expect(page.getByText(/Last sync: \d+/)).toBeVisible();

    // Run incremental backup again -> unchanged count reflects existing files
    await designsBackupBtn.click();
    await expect(
      page.getByText(
        /Designs backup complete: scanned \d+, copied 0, updated 0, unchanged [1-9]\d*, archived 0\./,
      ),
    ).toBeVisible();
  });

  test("executes Backup Everything Now and updates both timestamps", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const bothBackupBtn = page.getByRole("button", {
      name: "Backup Everything Now",
    });
    await expect(bothBackupBtn).toBeEnabled();

    await bothBackupBtn.click();

    // Assert combined success toast
    await expect(
      page.getByText("Both backups completed successfully."),
    ).toBeVisible();

    // Verify both timestamps are populated
    await expect(page.getByText(/Last backup: \d+/)).toBeVisible();
    await expect(page.getByText(/Last sync: \d+/)).toBeVisible();
  });

  test("resets destinations and cleans up configuration", async ({ page }) => {
    await gotoRoute(page, "#/admin/system/backup");

    const dbInput = page.locator("#backup-db-destination");
    const designsInput = page.locator("#backup-designs-destination");
    const saveBtn = page.getByRole("button", { name: "Save destinations" });

    // Clear destinations
    await dbInput.fill("");
    await designsInput.fill("");

    await expect(saveBtn).toBeEnabled();
    await saveBtn.click();

    await expect(
      page.getByText("Backup destinations saved."),
    ).toBeVisible();

    // Action buttons are now disabled because paths are empty
    await expect(
      page.getByRole("button", { name: "Backup Database Now" }),
    ).toBeDisabled();
    await expect(
      page.getByRole("button", { name: "Run incremental backup" }),
    ).toBeDisabled();
    await expect(
      page.getByRole("button", { name: "Backup Everything Now" }),
    ).toBeDisabled();

    // Reload page to confirm empty values persisted
    await page.reload();
    await expect(page.locator("#backup-db-destination")).toHaveValue("");
    await expect(page.locator("#backup-designs-destination")).toHaveValue("");
  });
});
