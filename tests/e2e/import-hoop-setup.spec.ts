// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./app-fixture";
import { gotoRoute, runImportToPrecheck } from "./helpers";
import {
  cleanupDataRoot,
  IMPORT_SOURCE_PATH,
  prepareEmptyDataRoot,
  prepareSingleDesignImportSource,
} from "./empty-root";
import { EMPTY_DATA_ROOT_PATH, HOOPS_DATA_ROOT_PATH } from "./paths";

/**
 * Import-time hoop setup gate.
 *
 * On the very first import into an empty catalogue with no hoops configured,
 * the backend returns `requires_skip_hoops_confirmation` and the wizard's
 * "Before You Import" step shows a warning plus a
 * "Confirm import without hoop setup" button. The import only runs once the
 * user confirms.
 *
 * This gate only exists when `design_count == 0 && hoop_count == 0`
 * (see `src/routes/bulk_import.rs`), so it cannot be exercised against the
 * shared, populated data root. Each describe below therefore assembles its own
 * empty throwaway root from the pristine install-template database.
 *
 * The actual *creation* of hoops is covered separately (Admin -> Hoops). These
 * tests cover the first-import gate: present without hoops, absent with them.
 */

// The generated single-design import source is shared by both describes.
test.afterAll(() => {
  cleanupDataRoot(IMPORT_SOURCE_PATH);
});

test.describe("first import without hoops configured", () => {
  test.use({ dataRoot: EMPTY_DATA_ROOT_PATH });

  test.beforeAll(() => {
    prepareEmptyDataRoot(EMPTY_DATA_ROOT_PATH);
    prepareSingleDesignImportSource();
  });

  test.afterAll(() => {
    cleanupDataRoot(EMPTY_DATA_ROOT_PATH);
  });

  test("asks the user to confirm skipping hoop setup, then imports", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    await runImportToPrecheck(page, prepareSingleDesignImportSource());

    // Act: the first "Import Designs" press does not import — it surfaces the
    // hoop setup warning.
    const importButton = page.getByRole("button", { name: "Import Designs" });
    await importButton.click();

    const gateText = page.getByText(
      "Hoops are not configured for a first import. Confirm to continue anyway.",
    );
    await expect(gateText).toBeVisible({ timeout: 20_000 });

    const confirmButton = page.getByRole("button", {
      name: "Confirm import without hoop setup",
    });
    await expect(confirmButton).toBeVisible();

    // Confirm and let the import run to completion.
    await confirmButton.click();
    await expect(importButton).toBeHidden({ timeout: 150_000 });

    // Assert: the design landed in the catalogue.
    const cards = page.locator("article.browse-card");
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
      .toBeGreaterThan(0);

    // Assert: no hoops were created as a side effect of skipping setup.
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    await expect(
      page.getByText("No hoops defined yet. Add your own machine hoops above."),
    ).toBeVisible();
  });
});

test.describe("first import with a hoop already configured", () => {
  test.use({ dataRoot: HOOPS_DATA_ROOT_PATH });

  test.beforeAll(() => {
    prepareEmptyDataRoot(HOOPS_DATA_ROOT_PATH, {
      hoops: [{ name: "Hoop A", maxWidthMm: 126, maxHeightMm: 110 }],
    });
    prepareSingleDesignImportSource();
  });

  test.afterAll(() => {
    cleanupDataRoot(HOOPS_DATA_ROOT_PATH);
  });

  test("does not ask the user to confirm skipping hoop setup", async ({
    page,
  }) => {
    test.setTimeout(180_000);

    await runImportToPrecheck(page, prepareSingleDesignImportSource());

    // Act: with a hoop configured the import starts immediately.
    const importButton = page.getByRole("button", { name: "Import Designs" });
    await importButton.click();
    await expect(importButton).toBeHidden({ timeout: 60_000 });

    // Assert: the hoop-setup gate must NOT appear (negative assertion).
    await expect(
      page.getByText(
        "Hoops are not configured for a first import. Confirm to continue anyway.",
      ),
    ).toBeHidden();
    await expect(
      page.getByRole("button", { name: "Confirm import without hoop setup" }),
    ).toBeHidden();

    // Assert: the design landed, and the configured hoop is still there.
    const cards = page.locator("article.browse-card");
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
      .toBeGreaterThan(0);

    await gotoRoute(page, "#/admin/data/hoops");
    await expect(page.getByRole("cell", { name: "Hoop A" })).toBeVisible();
  });
});
