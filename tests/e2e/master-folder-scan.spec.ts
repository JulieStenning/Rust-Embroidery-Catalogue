// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";
import { DATA_ROOT_PATH, DATABASE_FILENAME } from "./paths";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";
import fs from "node:fs";

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

test.describe("Master outline folder stitch file pairing", () => {
  let masterDesignId: number;
  let testFolder: string;

  test.beforeEach(async () => {
    testFolder = path.join(
      DATA_ROOT_PATH,
      "MachineEmbroideryDesigns",
      "PairTestFolder",
    );
    fs.mkdirSync(testFolder, { recursive: true });

    // Write dummy Daisy.eof
    fs.writeFileSync(path.join(testFolder, "Daisy.eof"), "dummy-eof-content");

    // Copy valid Bean.pes as Daisy.pes
    const fixturePath = path.resolve("tests/Test Designs/Bean.pes");
    if (fs.existsSync(fixturePath)) {
      fs.copyFileSync(fixturePath, path.join(testFolder, "Daisy.pes"));
    }

    masterDesignId = withRetry(() => {
      const db = getTestDb();
      try {
        const stmt = db.prepare(
          `INSERT INTO designs (filename, filepath, master_filepath, is_master_only, notes, date_added)
           VALUES ('Daisy.eof', 'PairTestFolder/Daisy.eof', 'PairTestFolder/Daisy.eof', 1, 'Folder pair test note', datetime('now'))`,
        );
        const result = stmt.run();
        return Number(result.lastInsertRowid);
      } finally {
        db.close();
      }
    });
  });

  test.afterEach(async () => {
    if (masterDesignId) {
      withRetry(() => {
        const db = getTestDb();
        try {
          db.prepare("DELETE FROM design_tags WHERE design_id = ?").run(
            masterDesignId,
          );
          db.prepare("DELETE FROM designs WHERE id = ?").run(masterDesignId);
        } finally {
          db.close();
        }
      });
    }

    if (testFolder && fs.existsSync(testFolder)) {
      fs.rmSync(testFolder, { recursive: true, force: true });
    }
  });

  test("displays master-only banner and pairs with stitch file on folder scan", async ({
    page,
  }) => {
    await gotoRoute(page, `#/designs/${masterDesignId}`);

    // Verify master-only banner is rendered without an embedded button
    const banner = page.locator('[data-testid="design-master-only-banner"]');
    await expect(banner).toBeVisible({ timeout: 15_000 });
    await expect(banner).toContainText("Outline Master File");
    await expect(
      banner.locator('[data-testid="check-folder-stitch-files-banner-btn"]'),
    ).toHaveCount(0);

    const scanBtn = page.locator(
      '[data-testid="check-folder-stitch-files-action-btn"]',
    );
    await expect(scanBtn).toBeVisible();
    await expect(scanBtn).toContainText("Check Folder for Stitch Files");

    // Click button to scan folder and pair
    await scanBtn.click();

    // Verify banner disappears as design is upgraded to paired stitch design
    await expect(banner).not.toBeVisible({ timeout: 15_000 });

    // Verify preview image appears
    const previewImg = page.locator("img[alt*='Daisy']");
    await expect(previewImg).toBeVisible({ timeout: 10_000 });

    // Verify database record has been upgraded in place
    const row = withRetry(() => {
      const db = getTestDb();
      try {
        return db
          .prepare(
            "SELECT filename, filepath, master_filepath, is_master_only, notes FROM designs WHERE id = ?",
          )
          .get(masterDesignId) as {
          filename: string;
          filepath: string;
          master_filepath: string;
          is_master_only: number;
          notes: string;
        };
      } finally {
        db.close();
      }
    });

    expect(row.filename).toBe("Daisy.pes");
    expect(row.filepath).toBe("PairTestFolder/Daisy.pes");
    expect(row.master_filepath).toBe("PairTestFolder/Daisy.eof");
    expect(row.is_master_only).toBe(0);
    expect(row.notes).toBe("Folder pair test note");
  });
});
