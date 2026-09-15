import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Manage Hoops" admin feature (accessed via
 * "Manage Data" in the top navigation menu -> Hoops tab or the "#/admin/data/hoops" route).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe("manage hoops", () => {
  test("navigates to Manage Hoops from the Manage Data top menu link and sub-tab", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Manage Data");

    // Click the Hoops sub-tab in the reference data tablist
    const hoopsTab = page.getByTestId("reference-data-tab-hoops");
    await expect(hoopsTab).toBeVisible();
    await hoopsTab.click();

    // Heading and description are rendered
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Hoop sizes depend on your machine and the frames you own. Add your own hoops below.",
      ),
    ).toBeVisible();

    // Tab is selected
    await expect(hoopsTab).toHaveAttribute("aria-selected", "true");

    // Manage Data admin link in navbar is active
    const manageDataLink = mainMenu(page).getByRole("link", {
      name: "Manage Data",
    });
    await expect(manageDataLink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/data/hoops", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    const hoopsTab = page.getByTestId("reference-data-tab-hoops");
    await expect(hoopsTab).toHaveAttribute("aria-selected", "true");
  });

  test("renders seeded hoops and verifies dimension-based sort order", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Contract: seed database has Hoop A (126x110), Hoop B (200x140), Gigahoop (230x200)
    const expectedHoops = [
      { name: "Hoop A", width: "126", height: "110" },
      { name: "Hoop B", width: "200", height: "140" },
      { name: "Gigahoop", width: "230", height: "200" },
    ];

    for (const hoop of expectedHoops) {
      const row = page.locator("tr", {
        has: page.getByRole("cell", { name: hoop.name, exact: true }),
      });
      await expect(row).toBeVisible();
      await expect(row.locator("td").nth(1)).toHaveText(hoop.width);
      await expect(row.locator("td").nth(2)).toHaveText(hoop.height);
    }

    // Verify ordering: max_width_mm ASC, max_height_mm ASC, name COLLATE NOCASE ASC
    // Hoop A (126mm) < Hoop B (200mm) < Giga Hoop (230mm)
    const tableRows = page.locator("table tbody tr:not(.bg-amber-50)");
    const firstRowName = await tableRows.first().locator("td").first().textContent();
    expect(firstRowName?.trim()).toBe("Hoop A");
  });

  test("enforces add form input validation and clear button behavior", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    const nameInput = page.locator("#admin-hoop-name");
    const widthInput = page.locator("#admin-hoop-width");
    const heightInput = page.locator("#admin-hoop-height");
    const addButton = page.getByRole("button", { name: "Add", exact: true });
    const clearButton = page.getByRole("button", { name: "Clear", exact: true });

    // Initial empty state: both buttons disabled
    await expect(nameInput).toHaveValue("");
    await expect(widthInput).toHaveValue("0");
    await expect(heightInput).toHaveValue("0");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();

    // Name filled only: Clear enabled, Add disabled (width/height still 0)
    await nameInput.fill("Partial Hoop");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeEnabled();

    // Clear clicked: resets input and disables buttons
    await clearButton.click();
    await expect(nameInput).toHaveValue("");
    await expect(widthInput).toHaveValue("0");
    await expect(heightInput).toHaveValue("0");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();

    // Fill valid name, width, and height: both buttons enabled
    await nameInput.fill("Valid Hoop");
    await widthInput.fill("100");
    await heightInput.fill("100");
    await expect(addButton).toBeEnabled();
    await expect(clearButton).toBeEnabled();
  });

  test("rejects reserved hoop name creation", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Attempting to add system reserved sentinel name __hoop_unknown__
    await page.locator("#admin-hoop-name").fill("__hoop_unknown__");
    await page.locator("#admin-hoop-width").fill("100");
    await page.locator("#admin-hoop-height").fill("100");
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Error toast is displayed
    await expect(
      page.getByText(
        '"__hoop_unknown__" is reserved for the system and cannot be used as a hoop name.',
      ),
    ).toBeVisible();
  });

  test("adds a new hoop and persists it across reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    const name = `Playwright Hoop ${Date.now()}`;
    const nameInput = page.locator("#admin-hoop-name");
    const widthInput = page.locator("#admin-hoop-width");
    const heightInput = page.locator("#admin-hoop-height");

    await nameInput.fill(name);
    await widthInput.fill("150");
    await heightInput.fill("150");
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Toast notification and inputs cleared/reset
    await expect(page.getByText("Hoop added.")).toBeVisible();
    await expect(nameInput).toHaveValue("");
    await expect(widthInput).toHaveValue("0");
    await expect(heightInput).toHaveValue("0");

    // Hoop appears in table with correct dimensions and 0 designs used
    const row = page.locator("tr", { has: page.getByRole("cell", { name, exact: true }) });
    await expect(row).toBeVisible();
    await expect(row.locator("td").nth(1)).toHaveText("150");
    await expect(row.locator("td").nth(2)).toHaveText("150");
    await expect(row.locator("td").nth(3)).toHaveText("0");

    // Persisted across reload (real SQLite database check)
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    const reloadedRow = page.locator("tr", { has: page.getByRole("cell", { name, exact: true }) });
    await expect(reloadedRow).toBeVisible();
    await expect(reloadedRow.locator("td").nth(1)).toHaveText("150");
    await expect(reloadedRow.locator("td").nth(2)).toHaveText("150");

    // Clean up created hoop
    await reloadedRow.getByRole("button", { name: "Delete", exact: true }).click();
    await reloadedRow.getByRole("button", { name: "Confirm delete", exact: true }).click();
    await expect(page.getByRole("cell", { name, exact: true })).toBeHidden();
  });

  test("rejects duplicate hoop name creation", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Attempting to add an existing seeded name (case-insensitive)
    await page.locator("#admin-hoop-name").fill("hoop a");
    await page.locator("#admin-hoop-width").fill("126");
    await page.locator("#admin-hoop-height").fill("110");
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Error toast is displayed
    await expect(
      page.getByText(/Could not add hoop:.*Hoop 'hoop a' already exists\./i),
    ).toBeVisible();
  });

  test("edits a hoop with cancellation, validation, and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Create a dedicated hoop to edit
    const originalName = `Edit Hoop Target ${Date.now()}`;
    await page.locator("#admin-hoop-name").fill(originalName);
    await page.locator("#admin-hoop-width").fill("120");
    await page.locator("#admin-hoop-height").fill("120");
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Hoop added.")).toBeVisible();

    const row = page.locator("tr", {
      has: page.getByRole("cell", { name: originalName, exact: true }),
    });
    await expect(row).toBeVisible();

    // 1. Begin edit and cancel
    await row.getByRole("button", { name: "Edit", exact: true }).click();
    const editingRow = page.locator("tr", {
      has: page.locator("input.admin-input"),
    });
    const editNameInput = editingRow.locator("input.admin-input").nth(0);
    const editWidthInput = editingRow.locator("input.admin-input").nth(1);
    const editHeightInput = editingRow.locator("input.admin-input").nth(2);

    await expect(editNameInput).toBeVisible();
    await expect(editNameInput).toHaveValue(originalName);
    await expect(editWidthInput).toHaveValue("120");
    await expect(editHeightInput).toHaveValue("120");

    await editingRow.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.getByRole("cell", { name: originalName, exact: true })).toBeVisible();

    // 2. Validate empty edit details rejection
    await page
      .locator("tr", { has: page.getByRole("cell", { name: originalName, exact: true }) })
      .getByRole("button", { name: "Edit", exact: true })
      .click();
    await editNameInput.fill("   ");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByText("Enter hoop details.")).toBeVisible();

    // 3. Validate reserved name edit rejection
    await editNameInput.fill("__hoop_unknown__");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      page.getByText(
        '"__hoop_unknown__" is reserved for the system and cannot be used as a hoop name.',
      ),
    ).toBeVisible();

    // 4. Validate duplicate edit name rejection
    await editNameInput.fill("Hoop A");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      page.getByText(/Could not update hoop:.*Hoop 'Hoop A' already exists\./i),
    ).toBeVisible();

    // 5. Successful update and persistence
    const updatedName = `Renamed Hoop ${Date.now()}`;
    await editNameInput.fill(updatedName);
    await editWidthInput.fill("180");
    await editHeightInput.fill("160");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();

    await expect(page.getByText("Hoop updated.")).toBeVisible();
    const updatedRow = page.locator("tr", {
      has: page.getByRole("cell", { name: updatedName, exact: true }),
    });
    await expect(updatedRow).toBeVisible();
    await expect(updatedRow.locator("td").nth(1)).toHaveText("180");
    await expect(updatedRow.locator("td").nth(2)).toHaveText("160");
    await expect(
      page.getByRole("cell", { name: originalName, exact: true }),
    ).not.toBeVisible();

    // Check persistence across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    const persistedRow = page.locator("tr", {
      has: page.getByRole("cell", { name: updatedName, exact: true }),
    });
    await expect(persistedRow).toBeVisible();
    await expect(persistedRow.locator("td").nth(1)).toHaveText("180");
    await expect(persistedRow.locator("td").nth(2)).toHaveText("160");

    // Clean up edited hoop
    await persistedRow.getByRole("button", { name: "Delete", exact: true }).click();
    await persistedRow.getByRole("button", { name: "Confirm delete", exact: true }).click();
    await expect(page.getByRole("cell", { name: updatedName, exact: true })).toBeHidden();
  });

  test("deletes an unused hoop with confirmation flow and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Create a temporary hoop with 0 designs
    const deleteTargetName = `Delete Hoop Target ${Date.now()}`;
    await page.locator("#admin-hoop-name").fill(deleteTargetName);
    await page.locator("#admin-hoop-width").fill("140");
    await page.locator("#admin-hoop-height").fill("140");
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Hoop added.")).toBeVisible();

    const row = page.locator("tr", {
      has: page.getByRole("cell", { name: deleteTargetName, exact: true }),
    });
    await expect(row).toBeVisible();

    // Click Delete -> shows prompt toast and confirmation row
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await expect(
      page.getByText(`Delete '${deleteTargetName}'? Click confirm delete to continue.`),
    ).toBeVisible();
    await expect(page.getByText("Confirm deletion for this hoop.")).toBeVisible();

    // Cancel deletion
    const cancelButton = row.getByRole("button", { name: "Cancel", exact: true });
    await cancelButton.click();
    await expect(page.getByText("Confirm deletion for this hoop.")).not.toBeVisible();
    await expect(page.getByRole("cell", { name: deleteTargetName, exact: true })).toBeVisible();

    // Confirm deletion
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await row.getByRole("button", { name: "Confirm delete", exact: true }).click();

    await expect(page.getByText("Hoop deleted.").first()).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();

    // Verify deletion persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();
  });

  test("shows clear assignment warning when attempting to delete a hoop with linked designs", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // "Hoop B" is used by designs in the seed database (design_count > 0)
    const hoopBRow = page.locator("tr", {
      has: page.getByRole("cell", { name: "Hoop B", exact: true }),
    });
    await expect(hoopBRow).toBeVisible();

    const designCountText = (await hoopBRow.locator("td").nth(3).textContent())?.trim();
    const designCount = Number(designCountText);
    expect(designCount).toBeGreaterThan(0);

    // Click Delete on assigned hoop
    await hoopBRow.getByRole("button", { name: "Delete", exact: true }).click();

    // Warning toast and warning banner are displayed
    await expect(
      page.getByText(
        `Deleting 'Hoop B' will clear assignment from ${designCount} design(s).`,
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        `This hoop is currently used by ${designCount} design(s). If you delete it, those designs will no longer have a hoop assigned.`,
      ),
    ).toBeVisible();

    // Cancel to preserve seed data
    await hoopBRow.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(
      page.getByRole("cell", { name: "Hoop B", exact: true }),
    ).toBeVisible();
  });

  test("switches tabs between Reference Data sub-tabs and returns to Hoops", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/hoops");
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
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

    // Switch to Sources tab
    await page.getByTestId("reference-data-tab-sources").click();
    await expect(
      page.getByRole("heading", { name: "Manage Sources" }),
    ).toBeVisible();

    // Return to Hoops tab
    await page.getByTestId("reference-data-tab-hoops").click();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();
  });
});
