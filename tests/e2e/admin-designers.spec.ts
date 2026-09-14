import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Manage Designers" admin feature (accessed via
 * "Manage Data" in the top navigation menu or the "#/admin/data/designers" route).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe("manage designers", () => {
  test("navigates to Manage Designers from the Manage Data top menu link", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Manage Data");

    // Heading and description are rendered
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(
      page.getByText("Designers are the creators or brands of embroidery designs."),
    ).toBeVisible();

    // Tablist has Designers selected
    const designersTab = page.getByTestId("reference-data-tab-designers");
    await expect(designersTab).toBeVisible();
    await expect(designersTab).toHaveAttribute("aria-selected", "true");

    // Manage Data admin link in navbar is active
    const manageDataLink = mainMenu(page).getByRole("link", {
      name: "Manage Data",
    });
    await expect(manageDataLink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/data and #/admin/data/designers", async ({
    page,
  }) => {
    // Bare hub URL defaults to designers
    await gotoRoute(page, "#/admin/data");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Explicit sub-route
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
  });

  test("renders seeded designers and verifies case-insensitive sort order", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Contract: seed database has Me, Quillmark Designs, Thistlebury Stitch, Wrenwood Studio
    const expectedDesigners = [
      "Me",
      "Quillmark Designs",
      "Thistlebury Stitch",
      "Wrenwood Studio",
    ];

    for (const name of expectedDesigners) {
      await expect(page.getByRole("cell", { name, exact: true })).toBeVisible();
    }

    // Verify alphabetical ordering of rows in the designers table
    const tableRows = page.locator("table tbody tr");
    const firstRowName = await tableRows.first().locator("td").first().textContent();
    expect(firstRowName?.trim().toLowerCase()).toBe("me");
  });

  test("enforces add form input validation and clear button behavior", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    const input = page.getByPlaceholder("New designer name...");
    const addButton = page.getByRole("button", { name: "Add", exact: true });
    const clearButton = page.getByRole("button", { name: "Clear", exact: true });

    // Initial empty state: both disabled
    await expect(input).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();

    // Whitespace only: Clear enabled, Add disabled
    await input.fill("   ");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeEnabled();

    // Clear clicked: resets input and disables buttons
    await clearButton.click();
    await expect(input).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();

    // Valid name: both enabled
    await input.fill("Valid Designer");
    await expect(addButton).toBeEnabled();
    await expect(clearButton).toBeEnabled();
  });

  test("adds a new designer and persists it across reload", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    const name = `Playwright Designer ${Date.now()}`;
    const input = page.getByPlaceholder("New designer name...");
    await input.fill(name);
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Toast notification and input cleared
    await expect(page.getByText("Designer added.")).toBeVisible();
    await expect(input).toHaveValue("");

    // Designer appears in table with 0 designs used
    const row = page.locator("tr", { has: page.getByRole("cell", { name, exact: true }) });
    await expect(row).toBeVisible();
    await expect(row.locator("td").nth(1)).toHaveText("0");

    // Persisted across reload (real SQLite database check)
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name, exact: true })).toBeVisible();
  });

  test("rejects duplicate designer name creation", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Attempting to add an existing seeded name (case-insensitive)
    const input = page.getByPlaceholder("New designer name...");
    await input.fill("me");
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Error toast is displayed
    await expect(
      page.getByText(/Could not add designer:.*Designer 'me' already exists\./i),
    ).toBeVisible();
  });

  test("edits a designer with cancellation, validation, and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Create a dedicated designer to edit
    const originalName = `Edit Target ${Date.now()}`;
    await page.getByPlaceholder("New designer name...").fill(originalName);
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Designer added.")).toBeVisible();

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

    await editingRow.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.getByRole("cell", { name: originalName, exact: true })).toBeVisible();

    // 2. Validate empty edit name rejection
    await page
      .locator("tr", { has: page.getByRole("cell", { name: originalName, exact: true }) })
      .getByRole("button", { name: "Edit", exact: true })
      .click();
    await editInput.fill("   ");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByText("Enter a designer name.")).toBeVisible();

    // 3. Validate duplicate edit name rejection
    await editInput.fill("Me");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      page.getByText(/Could not update designer:.*Designer 'Me' already exists\./i),
    ).toBeVisible();

    // 4. Successful update and persistence
    const updatedName = `Renamed Designer ${Date.now()}`;
    await editInput.fill(updatedName);
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();

    await expect(page.getByText("Designer updated.")).toBeVisible();
    await expect(page.getByRole("cell", { name: updatedName, exact: true })).toBeVisible();
    await expect(
      page.getByRole("cell", { name: originalName, exact: true }),
    ).not.toBeVisible();

    // Check persistence across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(page.getByRole("cell", { name: updatedName, exact: true })).toBeVisible();
  });

  test("deletes an unused designer with confirmation flow and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // Create a temporary designer with 0 designs
    const deleteTargetName = `Delete Target ${Date.now()}`;
    await page.getByPlaceholder("New designer name...").fill(deleteTargetName);
    await page.getByRole("button", { name: "Add", exact: true }).click();
    await expect(page.getByText("Designer added.")).toBeVisible();

    const row = page.locator("tr", {
      has: page.getByRole("cell", { name: deleteTargetName, exact: true }),
    });
    await expect(row).toBeVisible();

    // Click Delete -> shows prompt toast and confirmation row
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await expect(
      page.getByText(`Delete '${deleteTargetName}'? Click confirm delete to continue.`),
    ).toBeVisible();
    await expect(page.getByText("Confirm deletion for this designer.")).toBeVisible();

    // Cancel deletion
    const cancelButton = row.getByRole("button", { name: "Cancel", exact: true });
    await cancelButton.click();
    await expect(page.getByText("Confirm deletion for this designer.")).not.toBeVisible();
    await expect(page.getByRole("cell", { name: deleteTargetName, exact: true })).toBeVisible();

    // Confirm deletion
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await row.getByRole("button", { name: "Confirm delete", exact: true }).click();

    await expect(page.getByText("Designer deleted.")).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();

    // Verify deletion persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();
  });

  test("shows clear assignment warning when attempting to delete a designer with linked designs", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    // "Me" is used by designs in the seed database (design_count > 0)
    const meRow = page.locator("tr", {
      has: page.getByRole("cell", { name: "Me", exact: true }),
    });
    await expect(meRow).toBeVisible();

    const designCountText = (await meRow.locator("td").nth(1).textContent())?.trim();
    const designCount = Number(designCountText);
    expect(designCount).toBeGreaterThan(0);

    // Click Delete on assigned designer
    await meRow.getByRole("button", { name: "Delete", exact: true }).click();

    // Warning toast and warning banner are displayed
    await expect(
      page.getByText(
        `Deleting 'Me' will clear assignment from ${designCount} design(s).`,
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        `This designer is currently used by ${designCount} design(s). If you delete it, those designs will no longer have a designer assigned.`,
      ),
    ).toBeVisible();

    // Cancel to preserve seed data
    await meRow.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.getByRole("cell", { name: "Me", exact: true })).toBeVisible();
  });

  test("switches tabs between Reference Data sub-tabs and returns to Designers", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/designers");
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

    // Switch to Hoops tab
    await page.getByTestId("reference-data-tab-hoops").click();
    await expect(
      page.getByRole("heading", { name: "Manage Hoops" }),
    ).toBeVisible();

    // Return to Designers tab
    await page.getByTestId("reference-data-tab-designers").click();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();
  });
});
