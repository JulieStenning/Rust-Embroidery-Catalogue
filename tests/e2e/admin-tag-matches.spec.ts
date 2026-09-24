// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Tag Word Matches" admin feature (accessed via
 * "Manage Data" in the top navigation menu -> Word Matches tab or the "#/admin/data/tag-matches" route).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 */
test.describe("manage tag word matches", () => {
  test("navigates to Tag Word Matches from the Manage Data top menu link and sub-tab", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "Manage Data");

    // Click the Word Matches sub-tab in the reference data tablist
    const matchesTab = page.getByTestId("reference-data-tab-tag-matches");
    await expect(matchesTab).toBeVisible();
    await matchesTab.click();

    // Heading and description subtitle are rendered
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Define words and aliases in file or folder names that automatically assign tags during import.",
      ),
    ).toBeVisible();

    // Tab is selected
    await expect(matchesTab).toHaveAttribute("aria-selected", "true");

    // Manage Data admin link in navbar is active
    const manageDataLink = mainMenu(page).getByRole("link", {
      name: "Manage Data",
    });
    await expect(manageDataLink).toHaveClass(/menu-link-active/);
  });

  test("deep links directly to #/admin/data/tag-matches", async ({ page }) => {
    await gotoRoute(page, "#/admin/data/tag-matches");
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    const matchesTab = page.getByTestId("reference-data-tab-tag-matches");
    await expect(matchesTab).toHaveAttribute("aria-selected", "true");
  });

  test("renders seeded tag synonym groups and starter keywords", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tag-matches");
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    // Quick Add section is visible
    await expect(
      page.getByRole("heading", { name: "Quick Add Word Match" }),
    ).toBeVisible();

    // Filter controls are present
    await expect(
      page.getByPlaceholder("🔍 Filter tags or keywords..."),
    ).toBeVisible();

    // Seeded groups should be present (e.g. Animals, Words & Letters, Flowers)
    await expect(page.getByRole("heading", { name: "Animals" })).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Words & Letters" }),
    ).toBeVisible();
  });

  test("displays existing matches and duplicate warning in Quick Add card", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tag-matches");
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    // Initially no existing matches section in Quick Add card
    await expect(page.getByTestId("quick-add-existing-matches")).toBeHidden();

    // Focus and select "Animals" in the quick add combobox
    const combobox = page.locator("#quick-add-tag-combobox");
    await combobox.click();
    await combobox.fill("Anim");

    const option = page.getByRole("option", { name: "Animals" });
    await expect(option).toBeVisible();
    await option.click();

    // Existing matches section should appear for Animals
    const existingSection = page.getByTestId("quick-add-existing-matches");
    await expect(existingSection).toBeVisible();
    await expect(existingSection).toContainText(
      'Existing matches for "Animals"',
    );

    // Type an existing keyword (e.g. frog) into words input
    const wordsInput = page.locator("#quick-add-words-input");
    await wordsInput.fill("frog");

    // Duplicate indicator should appear
    await expect(page.getByText(/⚠️ Already added: frog/i)).toBeVisible();

    // Clear input
    await wordsInput.fill("");
    await expect(page.getByText(/⚠️ Already added:/i)).toBeHidden();
  });

  test("adds a word match via Quick Add card, persists it, and allows removal", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tag-matches");
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    const uniqueWord = `pwtest${Date.now()}`;

    // Select "Animals" tag
    const combobox = page.locator("#quick-add-tag-combobox");
    await combobox.click();
    await combobox.fill("Animals");
    await page.getByRole("option", { name: "Animals" }).click();

    // Add unique word
    const wordsInput = page.locator("#quick-add-words-input");
    await wordsInput.fill(uniqueWord);

    const submitBtn = page.getByRole("button", { name: "Add Match" });
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // Word should appear in the Quick Add card chips
    const existingSection = page.getByTestId("quick-add-existing-matches");
    await expect(existingSection.getByText(uniqueWord)).toBeVisible();

    // Word should also appear in the Animals card below
    const animalsCard = page.locator('[data-testid^="tag-match-card-"]', {
      has: page.getByRole("heading", { name: "Animals" }),
    });
    await expect(animalsCard.getByText(uniqueWord)).toBeVisible();

    // Reload page to verify database persistence
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    const reloadedCard = page.locator('[data-testid^="tag-match-card-"]', {
      has: page.getByRole("heading", { name: "Animals" }),
    });
    await expect(reloadedCard.getByText(uniqueWord)).toBeVisible();

    // Remove the test word match chip to clean up state
    const removeBtn = reloadedCard.getByLabel(`Remove ${uniqueWord}`);
    await expect(removeBtn).toBeVisible();
    await removeBtn.click();

    // Confirm it is removed
    await expect(reloadedCard.getByText(uniqueWord)).toBeHidden();
  });

  test("filters displayed groups by search text and tag group", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tag-matches");
    await expect(
      page.getByRole("heading", { name: "Tag Word Matches" }),
    ).toBeVisible();

    const filterInput = page.getByPlaceholder("🔍 Filter tags or keywords...");

    // Filter by specific keyword
    await filterInput.fill("alphabet");
    await expect(
      page.getByRole("heading", { name: "Words & Letters" }),
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: "Animals" })).toBeHidden();

    // Clear filter
    await filterInput.fill("");
    await expect(page.getByRole("heading", { name: "Animals" })).toBeVisible();

    // Filter by Stitching group
    const stitchingFilterBtn = page.getByRole("button", {
      name: "Stitching",
      exact: true,
    });
    await stitchingFilterBtn.click();

    // Animals is an image tag, so it should be hidden
    await expect(page.getByRole("heading", { name: "Animals" })).toBeHidden();

    // Reset to All
    await page.getByRole("button", { name: "All", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Animals" })).toBeVisible();
  });

  test("opens Word Match Modal from the Manage Tags table Matches button", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    // Click "Matches" on the Animals row
    const animalsRow = page.locator("tr", {
      has: page.getByText("Animals", { exact: true }),
    });
    await expect(animalsRow).toBeVisible();

    const matchesButton = animalsRow.getByRole("button", { name: "Matches" });
    await expect(matchesButton).toBeVisible();
    await matchesButton.click();

    // Tag Word Match Modal should open
    const modal = page.getByRole("dialog", { name: "Tag Word Matches" });
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("Animals");

    // Close modal via Done button
    await modal.getByRole("button", { name: "Done" }).click();
    await expect(modal).toBeHidden();
  });
});
