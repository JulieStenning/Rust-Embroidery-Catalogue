// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Manage Sources" admin feature (accessed via
 * "Manage Data" in the top navigation menu -> Sources tab or the "#/admin/data/sources" route).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe("manage sources", () => {
  test("navigates to Manage Sources from the Manage Data top menu link and sub-tab", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Manage Data");

    // Click the Sources sub-tab in the reference data tablist
    const sourcesTab = page.getByTestId("reference-data-tab-sources");
    await expect(sourcesTab).toBeVisible();
    await sourcesTab.click();

    // Heading and description are rendered
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Sources describe where your designs came from, such as Purchased, Downloaded, or Gift.",
      ),
    ).toBeVisible();

    // Tab is selected
    await expect(sourcesTab).toHaveAttribute("aria-selected", "true");

    // Manage Data admin link in navbar is active
    const manageDataLink = mainMenu(page).getByRole("link", {
      name: "Manage Data",
    });
    await expect(manageDataLink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/data/sources", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    const sourcesTab = page.getByTestId("reference-data-tab-sources");
    await expect(sourcesTab).toHaveAttribute("aria-selected", "true");
  });

  test("renders seeded sources and verifies case-insensitive sort order", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Contract: seed database has Heirloom Stash, Loomthread Embroidery Suite, Me, Threadwise Guild
    const expectedSources = [
      "Heirloom Stash",
      "Loomthread Embroidery Suite",
      "Me",
      "Threadwise Guild",
    ];

    for (const name of expectedSources) {
      await expect(page.getByRole("cell", { name, exact: true })).toBeVisible();
    }

    // Verify alphabetical ordering of rows in the sources table
    const tableRows = page.locator("table tbody tr");
    const firstRowName = await tableRows
      .first()
      .locator("td")
      .first()
      .textContent();
    expect(firstRowName?.trim().toLowerCase()).toBe("heirloom stash");
  });

  test("enforces add form input validation and clear button behavior", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    const input = page.getByPlaceholder("e.g. Purchased, Downloaded...");
    const addButton = page.getByRole("button", { name: "Add", exact: true });
    const clearButton = page.getByRole("button", {
      name: "Clear",
      exact: true,
    });

    // Initial empty state: both disabled, input auto-focused
    await expect(input).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();
    await expect(input).toBeFocused();

    // Whitespace only: Clear enabled, Add disabled
    await input.fill("   ");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeEnabled();

    // Clear clicked: resets input and disables buttons, retains focus
    await clearButton.click();
    await expect(input).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();
    await expect(input).toBeFocused();

    // Valid name: both enabled
    await input.fill("Valid Source");
    await expect(addButton).toBeEnabled();
    await expect(clearButton).toBeEnabled();
  });

  test("adds a new source and persists it across reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    const name = `Playwright Source ${Date.now()}`;
    const input = page.getByPlaceholder("e.g. Purchased, Downloaded...");
    await input.fill(name);
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Toast notification and input cleared
    await expect(page.getByText("Source added.")).toBeVisible();
    await expect(input).toHaveValue("");

    // Source appears in table with 0 designs used
    const row = page.locator("tr", {
      has: page.getByRole("cell", { name, exact: true }),
    });
    await expect(row).toBeVisible();
    await expect(row.locator("td").nth(1)).toHaveText("0");

    // Persisted across reload (real SQLite database check)
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name, exact: true })).toBeVisible();
  });

  test("rejects duplicate source name creation", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Attempting to add an existing seeded name (case-insensitive)
    const input = page.getByPlaceholder("e.g. Purchased, Downloaded...");
    await input.fill("me");
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Error toast is displayed
    await expect(
      page.getByText(/Could not add source:.*Source 'me' already exists\./i),
    ).toBeVisible();
  });

  test("edits a source with cancellation, validation, and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Create a dedicated source to edit
    const originalName = `Edit Source Target ${Date.now()}`;
    await page
      .getByPlaceholder("e.g. Purchased, Downloaded...")
      .fill(originalName);
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Source added.")).toBeVisible();

    const row = page.locator("tr", {
      has: page.getByRole("cell", { name: originalName, exact: true }),
    });
    await expect(row).toBeVisible();

    // 1. Begin edit and cancel
    await row.getByRole("button", { name: "Edit", exact: true }).click();
    const editingRow = page.locator("tr", {
      has: page.locator("input.admin-input"),
    });
    const editInput = editingRow.locator("input.admin-input");
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue(originalName);

    await editingRow
      .getByRole("button", { name: "Cancel", exact: true })
      .click();
    await expect(
      page.getByRole("cell", { name: originalName, exact: true }),
    ).toBeVisible();

    // 2. Validate empty edit name rejection
    await page
      .locator("tr", {
        has: page.getByRole("cell", { name: originalName, exact: true }),
      })
      .getByRole("button", { name: "Edit", exact: true })
      .click();
    await editInput.fill("   ");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByText("Enter a source name.")).toBeVisible();

    // 3. Validate duplicate edit name rejection
    await editInput.fill("Me");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      page.getByText(/Could not update source:.*Source 'Me' already exists\./i),
    ).toBeVisible();

    // 4. Successful update and persistence
    const updatedName = `Renamed Source ${Date.now()}`;
    await editInput.fill(updatedName);
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();

    await expect(page.getByText("Source updated.")).toBeVisible();
    await expect(
      page.getByRole("cell", { name: updatedName, exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: originalName, exact: true }),
    ).not.toBeVisible();

    // Check persistence across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: updatedName, exact: true }),
    ).toBeVisible();
  });

  test("deletes an unused source with confirmation flow and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Create a temporary source with 0 designs
    const deleteTargetName = `Delete Source Target ${Date.now()}`;
    await page
      .getByPlaceholder("e.g. Purchased, Downloaded...")
      .fill(deleteTargetName);
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Source added.")).toBeVisible();

    const row = page.locator("tr", {
      has: page.getByRole("cell", { name: deleteTargetName, exact: true }),
    });
    await expect(row).toBeVisible();

    // Click Delete -> shows prompt toast and confirmation row
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await expect(
      page.getByText(
        `Delete '${deleteTargetName}'? Click confirm delete to continue.`,
      ),
    ).toBeVisible();
    await expect(
      page.getByText("Confirm deletion for this source."),
    ).toBeVisible();

    // Cancel deletion
    const cancelButton = row.getByRole("button", {
      name: "Cancel",
      exact: true,
    });
    await cancelButton.click();
    await expect(
      page.getByText("Confirm deletion for this source."),
    ).not.toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).toBeVisible();

    // Confirm deletion
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await row
      .getByRole("button", { name: "Confirm delete", exact: true })
      .click();

    await expect(page.getByText("Source deleted.").first()).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();

    // Verify deletion persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();
  });

  test("shows clear assignment warning when attempting to delete a source with linked designs", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // "Threadwise Guild" or "Me" is used by designs in the seed database (design_count > 0)
    const threadwiseRow = page.locator("tr", {
      has: page.getByRole("cell", { name: "Threadwise Guild", exact: true }),
    });
    await expect(threadwiseRow).toBeVisible();

    const designCountText = (
      await threadwiseRow.locator("td").nth(1).textContent()
    )?.trim();
    const designCount = Number(designCountText);
    expect(designCount).toBeGreaterThan(0);

    // Click Delete on assigned source
    await threadwiseRow
      .getByRole("button", { name: "Delete", exact: true })
      .click();

    // Warning toast and warning banner are displayed
    await expect(
      page.getByText(
        `Deleting 'Threadwise Guild' will clear assignment from ${designCount} design(s).`,
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        `This source is currently used by ${designCount} design(s). If you delete it, those designs will no longer have a source assigned.`,
      ),
    ).toBeVisible();

    // Cancel to preserve seed data
    await threadwiseRow
      .getByRole("button", { name: "Cancel", exact: true })
      .click();
    await expect(
      page.getByRole("cell", { name: "Threadwise Guild", exact: true }),
    ).toBeVisible();
  });

  test("switches tabs between Reference Data sub-tabs and returns to Sources", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/sources");
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Switch to Designers tab
    await page.getByTestId("reference-data-tab-designers").click();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Switch to Tags tab
    await page.getByTestId("reference-data-tab-tags").click();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Switch to Hoops tab
    await page.getByTestId("reference-data-tab-hoops").click();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Return to Sources tab
    await page.getByTestId("reference-data-tab-sources").click();
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();
  });
});
