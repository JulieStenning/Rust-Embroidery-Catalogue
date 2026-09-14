import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";
import { DATA_ROOT_PATH, DATABASE_FILENAME } from "./paths";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

/**
 * End-to-end tests for the "Orphaned Files" tab on the System Maintenance page
 * (accessed via top navigation "System" -> "Orphaned Files" tab or `#/admin/system/orphans`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */

function getTestDb(): DatabaseSync {
  const dbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
  const db = new DatabaseSync(dbPath);
  db.exec("PRAGMA busy_timeout = 10000;");
  return db;
}

function withRetry<T>(fn: () => T, maxRetries = 10, delayMs = 200): T {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return fn();
    } catch (err: any) {
      if (
        (err?.message?.includes("locked") || err?.message?.includes("busy")) &&
        i < maxRetries - 1
      ) {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, delayMs);
        continue;
      }
      throw err;
    }
  }
  throw new Error("Retry exhausted");
}

function insertSyntheticOrphans(
  records: Array<{ filename: string; filepath: string }>,
): number[] {
  return withRetry(() => {
    const db = getTestDb();
    const ids: number[] = [];
    try {
      const insertStmt = db.prepare(
        `INSERT INTO designs (filename, filepath, date_added)
         VALUES (?, ?, datetime('now'))`,
      );
      for (const rec of records) {
        const result = insertStmt.run(rec.filename, rec.filepath);
        ids.push(Number(result.lastInsertRowid));
      }
    } finally {
      db.close();
    }
    return ids;
  });
}

function deleteSyntheticOrphans(ids: number[]) {
  if (ids.length === 0) return;
  withRetry(() => {
    const db = getTestDb();
    try {
      const placeholders = ids.map(() => "?").join(",");
      db.prepare(`DELETE FROM designs WHERE id IN (${placeholders})`).run(...ids);
    } finally {
      db.close();
    }
  });
}

function getDesignCount(id: number): number {
  return withRetry(() => {
    const db = getTestDb();
    try {
      const row = db
        .prepare(`SELECT COUNT(*) as count FROM designs WHERE id = ?`)
        .get(id) as { count: number } | undefined;
      return Number(row?.count ?? 0);
    } finally {
      db.close();
    }
  });
}

test.describe.serial("orphaned files management", () => {
  let createdOrphanIds: number[] = [];

  test.afterEach(() => {
    if (createdOrphanIds.length > 0) {
      deleteSyntheticOrphans(createdOrphanIds);
      createdOrphanIds = [];
    }
  });

  test("navigates to Orphaned Files from top menu link and verifies tab switching and active states", async ({
    page,
  }) => {
    // 1. Navigate via top menu link
    await clickNav(page, "System");

    // Page title and header are rendered
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    const tablist = page.getByTestId("system-maintenance-tablist");
    await expect(tablist).toBeVisible();

    const settingsTab = page.getByTestId("system-tab-settings");
    const backupTab = page.getByTestId("system-tab-backup");
    const orphansTab = page.getByTestId("system-tab-orphans");

    await expect(settingsTab).toHaveAttribute("aria-selected", "true");
    await expect(orphansTab).toHaveAttribute("aria-selected", "false");

    // Switch to Orphaned Files tab
    await orphansTab.click();
    await expect(orphansTab).toHaveAttribute("aria-selected", "true");
    await expect(settingsTab).toHaveAttribute("aria-selected", "false");
    await expect(backupTab).toHaveAttribute("aria-selected", "false");

    // Heading and description are rendered
    await expect(
      page.getByRole("heading", { name: "Orphans", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Find and remove database records whose files no longer exist on disk.",
      ),
    ).toBeVisible();

    // Top navigation link remains active
    const systemNavlink = mainMenu(page).getByRole("link", { name: "System" });
    await expect(systemNavlink).toHaveClass(/menu-link-active/);
  });

  test("renders initial clean state when no orphans exist on disk", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/orphans");
    await expect(
      page.getByRole("heading", { name: "Orphans", exact: true }),
    ).toBeVisible();

    // Summary counter shows 0 records
    await expect(
      page.getByText(/0 orphaned record\(s\) total, page 1 of 1, showing 0/i),
    ).toBeVisible();

    // Empty state table body row
    await expect(
      page.getByText("No orphaned records found. Refresh or scan to check."),
    ).toBeVisible();

    // Table header columns
    const thead = page.locator(".admin-table thead");
    await expect(thead.getByText("Select")).toBeVisible();
    await expect(thead.getByText("ID")).toBeVisible();
    await expect(thead.getByText("Filename")).toBeVisible();
    await expect(thead.getByText("Path")).toBeVisible();
    await expect(thead.getByText("Actions")).toBeVisible();

    // Action buttons initial states
    const scanButton = page.getByRole("button", {
      name: "Scan Disk",
      exact: true,
    });
    const refreshButton = page.getByRole("button", {
      name: "Refresh",
      exact: true,
    });
    const selectAllButton = page.getByRole("button", {
      name: "Select all",
      exact: true,
    });
    const deselectAllButton = page.getByRole("button", {
      name: "Deselect all",
      exact: true,
    });
    const deleteSelectedButton = page.getByRole("button", {
      name: /^Delete selected/i,
    });
    const deleteAllButton = page.getByRole("button", { name: /^Delete all/i });

    await expect(scanButton).toBeEnabled();
    await expect(refreshButton).toBeEnabled();
    await expect(selectAllButton).toBeDisabled();
    await expect(deselectAllButton).toBeDisabled();
    await expect(deleteSelectedButton).toBeDisabled();
    await expect(deleteAllButton).toBeDisabled();
  });

  test("scans disk for orphans when all files exist and verifies scan toast and counts", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/orphans");
    await expect(
      page.getByRole("heading", { name: "Orphans", exact: true }),
    ).toBeVisible();

    const scanButton = page.getByRole("button", {
      name: "Scan Disk",
      exact: true,
    });
    await scanButton.click();

    // Toast notification confirms scan completion
    await expect(
      page.getByText(
        /Scan complete\. Checked \d+ file record\(s\)\. Found 0 orphan\(s\)\./i,
      ),
    ).toBeVisible({ timeout: 15_000 });

    // Stays in clean state
    await expect(
      page.getByText(/0 orphaned record\(s\) total, page 1 of 1, showing 0/i),
    ).toBeVisible();
    await expect(
      page.getByText("No orphaned records found. Refresh or scan to check."),
    ).toBeVisible();
  });

  test("detects synthetic orphaned records after disk scan and verifies row rendering", async ({
    page,
  }) => {
    // Inject 3 synthetic records with missing filepaths
    createdOrphanIds = insertSyntheticOrphans([
      {
        filename: "Missing_Alpha.jef",
        filepath: "NonExistent/Missing_Alpha.jef",
      },
      { filename: "Missing_Beta.pes", filepath: "Missing_Beta.pes" },
      {
        filename: "Missing_Gamma.dst",
        filepath: "SubFolder/Missing_Gamma.dst",
      },
    ]);

    await gotoRoute(page, "#/admin/system/orphans");
    await expect(
      page.getByRole("heading", { name: "Orphans", exact: true }),
    ).toBeVisible();

    // Click Scan Disk to discover newly injected orphans
    const scanButton = page.getByRole("button", {
      name: "Scan Disk",
      exact: true,
    });
    await scanButton.click();

    // Scan toast confirms detection
    await expect(
      page.getByText(/Found 3 orphan\(s\)\./i),
    ).toBeVisible({ timeout: 15_000 });

    // Counter updates
    await expect(
      page.getByText(/3 orphaned record\(s\) total, page 1 of 1, showing 3/i),
    ).toBeVisible();

    // Rows render accurately
    const rows = page.locator(".admin-table tbody tr");
    await expect(rows).toHaveCount(3);

    // Verify row contents
    await expect(
      page.getByRole("button", { name: "Missing_Alpha.jef", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Missing_Beta.pes", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Missing_Gamma.dst", exact: true }),
    ).toBeVisible();

    await expect(
      page.getByRole("cell", {
        name: "NonExistent/Missing_Alpha.jef",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", {
        name: "SubFolder/Missing_Gamma.dst",
        exact: true,
      }),
    ).toBeVisible();

    // Verify all 3 rows are selected by default on load
    const deleteSelectedBtn = page.getByRole("button", {
      name: "Delete selected (3)",
      exact: true,
    });
    await expect(deleteSelectedBtn).toBeVisible();
    await expect(deleteSelectedBtn).toBeEnabled();

    const deleteAllBtn = page.getByRole("button", {
      name: "Delete all (3)",
      exact: true,
    });
    await expect(deleteAllBtn).toBeVisible();
    await expect(deleteAllBtn).toBeEnabled();
  });

  test("verifies checkbox selection, 'Select all', and 'Deselect all' controls", async ({
    page,
  }) => {
    createdOrphanIds = insertSyntheticOrphans([
      {
        filename: "Selection_Test_1.jef",
        filepath: "Missing/Selection_Test_1.jef",
      },
      {
        filename: "Selection_Test_2.pes",
        filepath: "Missing/Selection_Test_2.pes",
      },
      {
        filename: "Selection_Test_3.dst",
        filepath: "Missing/Selection_Test_3.dst",
      },
    ]);

    await gotoRoute(page, "#/admin/system/orphans");
    await page.reload();

    await expect(
      page.getByText(/3 orphaned record\(s\) total/i),
    ).toBeVisible();

    const deleteSelectedBtn = page.getByRole("button", {
      name: /^Delete selected/i,
    });
    const deselectAllBtn = page.getByRole("button", {
      name: "Deselect all",
      exact: true,
    });
    const selectAllBtn = page.getByRole("button", {
      name: "Select all",
      exact: true,
    });

    // Initially all 3 are selected
    await expect(deleteSelectedBtn).toHaveText("Delete selected (3)");
    await expect(deselectAllBtn).toBeEnabled();

    // Uncheck first row's checkbox
    const firstRowCheckbox = page
      .locator(".admin-table tbody tr")
      .first()
      .locator('input[type="checkbox"]');
    await firstRowCheckbox.uncheck();
    await expect(deleteSelectedBtn).toHaveText("Delete selected (2)");

    // Uncheck second row's checkbox
    const secondRowCheckbox = page
      .locator(".admin-table tbody tr")
      .nth(1)
      .locator('input[type="checkbox"]');
    await secondRowCheckbox.uncheck();
    await expect(deleteSelectedBtn).toHaveText("Delete selected (1)");

    // Click "Deselect all"
    await deselectAllBtn.click();
    await expect(deleteSelectedBtn).toHaveText("Delete selected (0)");
    await expect(deleteSelectedBtn).toBeDisabled();
    await expect(deselectAllBtn).toBeDisabled();

    // Click "Select all"
    await selectAllBtn.click();
    await expect(deleteSelectedBtn).toHaveText("Delete selected (3)");
    await expect(deleteSelectedBtn).toBeEnabled();
    await expect(deselectAllBtn).toBeEnabled();
  });

  test("triggers 'Locate Folder' action for an orphan file record", async ({
    page,
  }) => {
    createdOrphanIds = insertSyntheticOrphans([
      {
        filename: "Locate_Me.jef",
        filepath: "LocateSubdir/Locate_Me.jef",
      },
    ]);

    await gotoRoute(page, "#/admin/system/orphans");
    await page.reload();

    await expect(
      page.getByRole("button", { name: "Locate_Me.jef", exact: true }),
    ).toBeVisible();

    const locateBtn = page
      .locator(".admin-table tbody tr", { hasText: "Locate_Me.jef" })
      .getByRole("button", { name: "Locate Folder", exact: true });
    await expect(locateBtn).toBeVisible();
    await locateBtn.click();

    // Toast confirms folder resolution/opening
    await expect(page.getByText(/Opened:/i)).toBeVisible({
      timeout: 10_000,
    });
  });

  test("cancels and confirms selective orphan deletion with confirmation dialog", async ({
    page,
  }) => {
    createdOrphanIds = insertSyntheticOrphans([
      {
        filename: "Selective_Keep.jef",
        filepath: "Missing/Selective_Keep.jef",
      },
      {
        filename: "Selective_Delete.pes",
        filepath: "Missing/Selective_Delete.pes",
      },
    ]);

    const targetDeleteId = createdOrphanIds[1];
    const targetKeepId = createdOrphanIds[0];

    await gotoRoute(page, "#/admin/system/orphans");
    await page.reload();

    await expect(
      page.getByText(/2 orphaned record\(s\) total/i),
    ).toBeVisible();

    // Uncheck Keep row so only Delete row is selected
    const keepRowCheckbox = page
      .locator(".admin-table tbody tr", { hasText: "Selective_Keep.jef" })
      .locator('input[type="checkbox"]');
    await keepRowCheckbox.uncheck();

    const deleteSelectedBtn = page.getByRole("button", {
      name: "Delete selected (1)",
      exact: true,
    });
    await expect(deleteSelectedBtn).toBeVisible();

    // 1. Test Cancellation: dismiss confirmation dialog
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("Delete 1 selected record(s)?");
      await dialog.dismiss();
    });

    await deleteSelectedBtn.click();

    // Both records still exist
    await expect(
      page.getByRole("button", { name: "Selective_Keep.jef", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Selective_Delete.pes", exact: true }),
    ).toBeVisible();
    expect(getDesignCount(targetDeleteId)).toBe(1);
    expect(getDesignCount(targetKeepId)).toBe(1);

    // 2. Test Confirmation: accept dialog
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("Delete 1 selected record(s)?");
      await dialog.accept();
    });

    await deleteSelectedBtn.click();

    // Toast confirms deletion
    await expect(
      page.getByText("1 record(s) deleted."),
    ).toBeVisible({ timeout: 10_000 });

    // Deleted record is removed from UI
    await expect(
      page.getByRole("button", { name: "Selective_Delete.pes", exact: true }),
    ).not.toBeVisible();
    await expect(
      page.getByRole("button", { name: "Selective_Keep.jef", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText(/1 orphaned record\(s\) total, page 1 of 1, showing 1/i),
    ).toBeVisible();

    // Database verification: record is purged from SQLite
    expect(getDesignCount(targetDeleteId)).toBe(0);
    expect(getDesignCount(targetKeepId)).toBe(1);
  });

  test("cancels and confirms bulk 'Delete all' orphans with confirmation dialog and verifies persistence", async ({
    page,
  }) => {
    createdOrphanIds = insertSyntheticOrphans([
      {
        filename: "Bulk_Orphan_1.jef",
        filepath: "Missing/Bulk_Orphan_1.jef",
      },
      {
        filename: "Bulk_Orphan_2.pes",
        filepath: "Missing/Bulk_Orphan_2.pes",
      },
    ]);

    const [id1, id2] = createdOrphanIds;

    await gotoRoute(page, "#/admin/system/orphans");
    await page.reload();

    await expect(
      page.getByText(/2 orphaned record\(s\) total/i),
    ).toBeVisible();

    const deleteAllBtn = page.getByRole("button", {
      name: "Delete all (2)",
      exact: true,
    });

    // 1. Test Cancellation
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("Delete ALL {orphanTotal} orphaned records?");
      await dialog.dismiss();
    });

    await deleteAllBtn.click();

    // Records still exist
    await expect(
      page.getByRole("button", { name: "Bulk_Orphan_1.jef", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Bulk_Orphan_2.pes", exact: true }),
    ).toBeVisible();
    expect(getDesignCount(id1)).toBe(1);
    expect(getDesignCount(id2)).toBe(1);

    // 2. Test Confirmation
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("Delete ALL {orphanTotal} orphaned records?");
      await dialog.accept();
    });

    await deleteAllBtn.click();

    // Toast confirms all deleted
    await expect(
      page.getByText("2 record(s) deleted."),
    ).toBeVisible({ timeout: 10_000 });

    // UI returns to clean empty state
    await expect(
      page.getByText(/0 orphaned record\(s\) total, page 1 of 1, showing 0/i),
    ).toBeVisible();
    await expect(
      page.getByText("No orphaned records found. Refresh or scan to check."),
    ).toBeVisible();

    // Database verification: purged from SQLite
    expect(getDesignCount(id1)).toBe(0);
    expect(getDesignCount(id2)).toBe(0);

    // Cleared from tracking
    createdOrphanIds = [];
  });

  test("handles pagination when orphan records exceed single page size", async ({
    page,
  }) => {
    // Generate 105 synthetic orphan records (page size is 100)
    const records = [];
    for (let i = 1; i <= 105; i++) {
      const padded = String(i).padStart(3, "0");
      records.push({
        filename: `Paging_Orphan_${padded}.jef`,
        filepath: `MissingPaging/Paging_Orphan_${padded}.jef`,
      });
    }
    createdOrphanIds = insertSyntheticOrphans(records);

    await gotoRoute(page, "#/admin/system/orphans");
    await page.reload();

    // Summary indicates multi-page setup
    await expect(
      page.getByText(/105 orphaned record\(s\) total, page 1 of 2, showing 100/i),
    ).toBeVisible({ timeout: 15_000 });

    const pagination = page.getByRole("navigation", {
      name: "Orphans pagination",
    });
    await expect(pagination).toBeVisible();

    // Navigate to Page 2
    const page2Button = pagination.getByRole("button", {
      name: "2",
      exact: true,
    });
    await page2Button.click();

    // Page 2 displays remaining 5 records
    await expect(
      page.getByText(/105 orphaned record\(s\) total, page 2 of 2, showing 5/i),
    ).toBeVisible();

    // Clean up via Delete all (105)
    page.once("dialog", (dialog) => dialog.accept());
    const deleteAllBtn = page.getByRole("button", {
      name: "Delete all (105)",
      exact: true,
    });
    await deleteAllBtn.click();

    await expect(
      page.getByText("105 record(s) deleted."),
    ).toBeVisible({ timeout: 15_000 });

    await expect(
      page.getByText(/0 orphaned record\(s\) total, page 1 of 1, showing 0/i),
    ).toBeVisible();

    createdOrphanIds = [];
  });
});
