// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Manage Tags" admin feature (accessed via
 * "Manage Data" in the top navigation menu -> Tags tab or the "#/admin/data/tags" route).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe("manage tags", () => {
  test("navigates to Manage Tags from the Manage Data top menu link and sub-tab", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Manage Data");

    // Click the Tags sub-tab in the reference data tablist
    const tagsTab = page.getByTestId("reference-data-tab-tags");
    await expect(tagsTab).toBeVisible();
    await tagsTab.click();

    // Heading and description subtitle are rendered
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Use Image tags for subject categories and Stitching tags for technique or style.",
      ),
    ).toBeVisible();

    // Tab is selected
    await expect(tagsTab).toHaveAttribute("aria-selected", "true");

    // Manage Data admin link in navbar is active
    const manageDataLink = mainMenu(page).getByRole("link", {
      name: "Manage Data",
    });
    await expect(manageDataLink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/data/tags", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const tagsTab = page.getByTestId("reference-data-tab-tags");
    await expect(tagsTab).toHaveAttribute("aria-selected", "true");
  });

  test("renders add tag form with default image group and both tag drawers", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Add new tag section
    await expect(
      page.getByRole("heading", { name: "Add new tag" }),
    ).toBeVisible();
    const descInput = page.locator("#admin-tag-description");
    const groupSelect = page.locator("#admin-tag-group");
    const addButton = page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true });
    const clearButton = page
      .locator("form")
      .getByRole("button", { name: "Clear", exact: true });

    await expect(descInput).toBeVisible();
    await expect(descInput).toHaveValue("");
    await expect(groupSelect).toBeVisible();
    await expect(groupSelect).toHaveValue("image");
    await expect(addButton).toBeVisible();
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeVisible();
    await expect(clearButton).toBeDisabled();

    // Both Image Tags and Stitching Tags drawers are rendered
    await expect(
      page.getByRole("heading", { name: "Image Tags" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Stitching Tags" }),
    ).toBeVisible();
  });

  test("enforces add form input validation and clear button behavior", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const descInput = page.locator("#admin-tag-description");
    const addButton = page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true });
    const clearButton = page
      .locator("form")
      .getByRole("button", { name: "Clear", exact: true });

    // Initial empty state: both disabled, input auto-focused
    await expect(descInput).toBeVisible();
    await expect(descInput).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();
    await expect(descInput).toBeFocused();

    // Whitespace only: Clear enabled, Add disabled
    await descInput.fill("   ");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeEnabled();

    // Clear clicked: resets input and disables buttons, retains focus
    await clearButton.click();
    await expect(descInput).toHaveValue("");
    await expect(addButton).toBeDisabled();
    await expect(clearButton).toBeDisabled();
    await expect(descInput).toBeFocused();

    // Valid description: both enabled
    await descInput.fill("Valid Tag");
    await expect(addButton).toBeEnabled();
    await expect(clearButton).toBeEnabled();
  });

  test("renders seeded tags in case-insensitive alphabetical order in each drawer", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Verify known seeded tags exist in the catalogue contract
    // Seeded image tags include: Flowers, Food, Footwear
    // Seeded stitching tags include: Applique, Cross Stitch, Filled
    await expect(
      page.getByRole("cell", { name: "Food", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "Flowers", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "Footwear", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "Cross Stitch", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "Applique", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "Filled", exact: true }),
    ).toBeVisible();

    // Verify rows in the first table (Image tags) are sorted alphabetically
    const imageTableRows = page
      .locator("details")
      .first()
      .locator("table tbody tr");
    const count = await imageTableRows.count();
    expect(count).toBeGreaterThan(1);

    const firstImageTag = await imageTableRows
      .first()
      .locator("td")
      .first()
      .textContent();
    const secondImageTag = await imageTableRows
      .nth(1)
      .locator("td")
      .first()
      .textContent();
    expect(
      firstImageTag
        ?.trim()
        .localeCompare(secondImageTag?.trim() || "", undefined, {
          sensitivity: "base",
        }),
    ).toBeLessThanOrEqual(0);
  });

  test("toggles collapsible drawers independently and persists open state to localStorage", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const imageDetails = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const stitchingDetails = page
      .locator("details")
      .filter({ hasText: "Stitching Tags" });

    // Both are open by default
    await expect(imageDetails).toHaveAttribute("open", "");
    await expect(stitchingDetails).toHaveAttribute("open", "");

    // Click Image Tags summary to collapse it
    await imageDetails.locator("summary").click();
    await expect(imageDetails).not.toHaveAttribute("open", "");
    // Stitching tags remains open
    await expect(stitchingDetails).toHaveAttribute("open", "");

    // Verify localStorage persistence
    await expect
      .poll(
        async () =>
          page.evaluate(() =>
            window.localStorage.getItem("admin.tags.collapsible.image"),
          ),
        { timeout: 10_000 },
      )
      .toBe("closed");

    // Reload and assert state is restored from localStorage
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const reloadedImageDetails = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const reloadedStitchingDetails = page
      .locator("details")
      .filter({ hasText: "Stitching Tags" });
    await expect(reloadedImageDetails).not.toHaveAttribute("open", "");
    await expect(reloadedStitchingDetails).toHaveAttribute("open", "");

    // Re-open Image Tags drawer
    await reloadedImageDetails.locator("summary").click();
    await expect(reloadedImageDetails).toHaveAttribute("open", "");
  });

  test("adds a new Image tag and persists it across reload", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const tagName = `Playwright Image Tag ${Date.now()}`;
    const descInput = page.locator("#admin-tag-description");
    const groupSelect = page.locator("#admin-tag-group");
    const addButton = page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true });

    await descInput.fill(tagName);
    await groupSelect.selectOption("image");
    await addButton.click();

    // Toast notification and input cleared
    await expect(page.getByText("Tag added.")).toBeVisible();
    await expect(descInput).toHaveValue("");

    // Tag appears in Image Tags drawer with 0 design count
    const imageSection = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const row = imageSection.locator("tr", {
      has: page.getByRole("cell", { name: tagName, exact: true }),
    });
    await expect(row).toBeVisible();
    await expect(row.locator("td").nth(1)).toHaveText("0");

    // Persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: tagName, exact: true }),
    ).toBeVisible();
  });

  test("adds a new Stitching tag and persists it across reload", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    const tagName = `Playwright Stitch Tag ${Date.now()}`;
    const descInput = page.locator("#admin-tag-description");
    const groupSelect = page.locator("#admin-tag-group");
    const addButton = page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true });

    await descInput.fill(tagName);
    await groupSelect.selectOption("stitching");
    await addButton.click();

    // Toast notification and input cleared
    await expect(page.getByText("Tag added.")).toBeVisible();
    await expect(descInput).toHaveValue("");

    // Tag appears in Stitching Tags drawer
    const stitchingSection = page
      .locator("details")
      .filter({ hasText: "Stitching Tags" });
    const row = stitchingSection.locator("tr", {
      has: page.getByRole("cell", { name: tagName, exact: true }),
    });
    await expect(row).toBeVisible();
    await expect(row.locator("td").nth(1)).toHaveText("0");

    // Persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(
      stitchingSection.getByRole("cell", { name: tagName, exact: true }),
    ).toBeVisible();
  });

  test("rejects duplicate tag name creation", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Attempting to add an existing seeded tag name (case-insensitive)
    const descInput = page.locator("#admin-tag-description");
    await descInput.fill("food");
    await page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true })
      .click();

    // Error toast is displayed
    await expect(
      page.getByText(/Could not add tag:.*Tag 'food' already exists\./i),
    ).toBeVisible();
  });

  test("edits an Image tag with cancellation, validation, and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Create a tag to edit
    const originalName = `Edit Tag Target ${Date.now()}`;
    await page.locator("#admin-tag-description").fill(originalName);
    await page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true })
      .click();
    await expect(page.getByText("Tag added.")).toBeVisible();

    const imageSection = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const row = imageSection.locator("tr", {
      has: page.getByRole("cell", { name: originalName, exact: true }),
    });
    await expect(row).toBeVisible();

    // 1. Begin edit and cancel
    await row.getByRole("button", { name: "Edit", exact: true }).click();
    const editingRow = imageSection.locator("tr", {
      has: page.locator("input.admin-input"),
    });
    const editInput = editingRow.locator("input.admin-input");
    await expect(editInput).toBeVisible();
    await expect(editInput).toHaveValue(originalName);

    await editingRow
      .getByRole("button", { name: "Cancel", exact: true })
      .click();
    await expect(
      imageSection.getByRole("cell", { name: originalName, exact: true }),
    ).toBeVisible();

    // 2. Validate empty edit name rejection
    await imageSection
      .locator("tr", {
        has: page.getByRole("cell", { name: originalName, exact: true }),
      })
      .getByRole("button", { name: "Edit", exact: true })
      .click();
    await editInput.fill("   ");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByText("Enter a tag name.")).toBeVisible();

    // 3. Validate duplicate edit name rejection
    await editInput.fill("Food");
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      page.getByText(/Could not update tag:.*Tag 'Food' already exists\./i),
    ).toBeVisible();

    // 4. Successful update and persistence
    const updatedName = `Renamed Tag ${Date.now()}`;
    await editInput.fill(updatedName);
    await editingRow.getByRole("button", { name: "Save", exact: true }).click();

    await expect(page.getByText("Tag updated.")).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: updatedName, exact: true }),
    ).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: originalName, exact: true }),
    ).not.toBeVisible();

    // Verify persistence across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: updatedName, exact: true }),
    ).toBeVisible();
  });

  test("enforces lock protection on system-defined stitching tags", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // System tags in the Stitching drawer (e.g. Cross Stitch, Applique, Filled) have is_system = true
    const stitchingSection = page
      .locator("details")
      .filter({ hasText: "Stitching Tags" });
    const crossStitchRow = stitchingSection.locator("tr", {
      has: page.getByRole("cell", { name: "Cross Stitch", exact: true }),
    });
    await expect(crossStitchRow).toBeVisible();

    // Assert that the row displays "Locked" badge and no Edit/Delete buttons
    const lockedBadge = crossStitchRow.locator("span", { hasText: "Locked" });
    await expect(lockedBadge).toBeVisible();
    await expect(lockedBadge).toHaveAttribute(
      "title",
      "System tags cannot be edited or deleted.",
    );
    await expect(
      crossStitchRow.getByRole("button", { name: "Edit", exact: true }),
    ).not.toBeVisible();
    await expect(
      crossStitchRow.getByRole("button", { name: "Delete", exact: true }),
    ).not.toBeVisible();
  });

  test("deletes an unused tag with inline confirmation flow and persistence", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Create a temporary unused tag
    const deleteTargetName = `Delete Tag Target ${Date.now()}`;
    await page.locator("#admin-tag-description").fill(deleteTargetName);
    await page
      .locator("form")
      .getByRole("button", { name: "Add", exact: true })
      .click();
    await expect(page.getByText("Tag added.")).toBeVisible();

    const imageSection = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const row = imageSection.locator("tr", {
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
      page.getByText("Confirm deletion for this tag."),
    ).toBeVisible();

    // Cancel deletion
    const cancelButton = row.getByRole("button", {
      name: "Cancel",
      exact: true,
    });
    await cancelButton.click();
    await expect(
      page.getByText("Confirm deletion for this tag."),
    ).not.toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).toBeVisible();

    // Click Delete and Confirm delete
    await row.getByRole("button", { name: "Delete", exact: true }).click();
    await row
      .getByRole("button", { name: "Confirm delete", exact: true })
      .click();

    await expect(page.getByText("Tag deleted.").first()).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();

    // Verify deletion persisted across reload
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
    await expect(
      imageSection.getByRole("cell", { name: deleteTargetName, exact: true }),
    ).not.toBeVisible();
  });

  test("shows clear usage warning when attempting to delete a tag linked to designs", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // "Food" is an image tag linked to Cake 3 designs in the seed database (design_count > 0)
    const imageSection = page
      .locator("details")
      .filter({ hasText: "Image Tags" });
    const foodRow = imageSection.locator("tr", {
      has: page.getByRole("cell", { name: "Food", exact: true }),
    });
    await expect(foodRow).toBeVisible();

    const designCountText = (
      await foodRow.locator("td").nth(1).textContent()
    )?.trim();
    const designCount = Number(designCountText);
    expect(designCount).toBeGreaterThan(0);

    // Click Delete on assigned tag
    await foodRow.getByRole("button", { name: "Delete", exact: true }).click();

    // Warning toast and warning row are displayed
    await expect(
      page.getByText(
        `Deleting 'Food' will remove it from ${designCount} design(s).`,
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        `This tag is used by ${designCount} design(s). If you delete it, those designs will no longer have the tag assigned.`,
      ),
    ).toBeVisible();

    // Cancel to preserve seed data
    await foodRow.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(
      imageSection.getByRole("cell", { name: "Food", exact: true }),
    ).toBeVisible();
  });

  test("switches tabs between Reference Data sub-tabs and returns to Tags", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Switch to Designers tab
    await page.getByTestId("reference-data-tab-designers").click();
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
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

    // Return to Tags tab
    await page.getByTestId("reference-data-tab-tags").click();
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();
  });
});
