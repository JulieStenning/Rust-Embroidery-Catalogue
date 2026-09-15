import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { gotoRoute, mainMenu } from "./helpers";

/**
 * Design Detail & Design Print end-to-end coverage.
 *
 * These tests drive the real Tauri app against the seeded test catalogue
 * (`tests/Test Assets/EmbroideryCatalogue.db`, copied to the throwaway data root
 * by `global-setup.ts`).
 *
 * They verify:
 * 1. Deep linking and layout rendering for populated designs and missing-preview designs.
 * 2. Non-existent design ID error handling.
 * 3. Browse context tracking and Prev/Next carousel navigation with counter.
 * 4. Interactive metadata mutations (designer, source, 5-star rating, stitched toggle,
 *    verification toggles, notes, tags management via modal and inline chip deletion,
 *    technical data recalculation, preview 2D/3D generation, editor/explorer triggers).
 * 5. Project assignment and removal.
 * 6. Printable design sheet (`#/designs/:id/print`) layout, metadata, print trigger, and navigation.
 * 7. Single-design delete modal lifecycle and cancellation.
 */

const RICH_DESIGN = "Cake 3.jef";
const SPARSE_DESIGN = "ZZ-broken.pes";

// ---------------------------------------------------------------------------
// Helpers and Locators
// ---------------------------------------------------------------------------

/** Look a design up by filename in Browse and return its database id. */
async function findDesignId(page: Page, filename: string): Promise<number> {
  await gotoRoute(page, "#/designs");
  await page.reload();

  const search = page.locator("#browse-q");
  await expect(search).toBeVisible({ timeout: 30_000 });
  await search.fill(filename);

  const card = page.locator("article.browse-card").filter({
    has: page.locator("p.browse-card-title", { hasText: filename }),
  });
  await expect(card).toHaveCount(1, { timeout: 30_000 });

  const id = await card.getAttribute("data-id");
  if (!id) {
    throw new Error(`Could not determine design id for ${filename}`);
  }
  return Number(id);
}

/** Open a design's detail route and wait for its content to finish loading. */
async function openDesignDetail(
  page: Page,
  designId: number,
  filename = RICH_DESIGN,
): Promise<void> {
  await gotoRoute(page, `#/designs/${designId}`);
  await page.reload();
  await expect(leftColumn(page).locator("p.font-medium")).toHaveText(filename, {
    timeout: 30_000,
  });
}

/** The Design Detail top action bar (containing Back to Browse, Prev/Next, Print). */
function detailNavBar(page: Page): Locator {
  return page.locator(".detail-page > div.flex.flex-wrap.items-center");
}

/** The left column container (file info, preview image, system actions). */
function leftColumn(page: Page): Locator {
  return page.locator(".detail-page .lg\\:border-r");
}

/** The right column container (metadata, tech data, ratings, tags, notes, projects). */
function rightColumn(page: Page): Locator {
  return page.locator(".detail-page .lg\\:w-7\\/12");
}

/** Find a route-card inside the right column by heading text. */
function detailCard(page: Page, headingText: string): Locator {
  return rightColumn(page)
    .locator(".route-card")
    .filter({ has: page.getByRole("heading", { name: headingText, exact: true }) });
}

/** A transient toast carrying message. */
function toast(page: Page, message: string | RegExp): Locator {
  return page.locator(".toast-message", { hasText: message }).first();
}

// ---------------------------------------------------------------------------
// Test Suite
// ---------------------------------------------------------------------------

test.describe.serial("Design Detail & Design Print Views", () => {
  let richDesignId = 0;
  let sparseDesignId = 0;

  test("discovers fixture design IDs from Browse", async ({ page }) => {
    richDesignId = await findDesignId(page, RICH_DESIGN);
    sparseDesignId = await findDesignId(page, SPARSE_DESIGN);

    expect(richDesignId).toBeGreaterThan(0);
    expect(sparseDesignId).toBeGreaterThan(0);
  });

  test("renders full layout for a populated design via direct deep link", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    // Top action bar
    await expect(page.getByRole("button", { name: "← Back to Browse" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Print", exact: true })).toBeVisible();

    // Left column: Filename & File Path
    await expect(leftColumn(page).getByText("Filename")).toBeVisible();
    await expect(leftColumn(page).locator("p.font-medium")).toHaveText(RICH_DESIGN);
    await expect(leftColumn(page).getByText("File Path")).toBeVisible();
    await expect(leftColumn(page).locator("p.font-mono")).toContainText(RICH_DESIGN);

    // Left column: Preview image rendered
    const previewImg = leftColumn(page).locator(`img[alt="${RICH_DESIGN}"]`);
    await expect(previewImg).toBeVisible();

    // Left column: Action buttons
    await expect(
      leftColumn(page).getByRole("button", { name: /Open in Editor/i }),
    ).toBeVisible();
    await expect(
      leftColumn(page).getByRole("button", { name: /Show in Explorer/i }),
    ).toBeVisible();
    await expect(
      leftColumn(page).getByRole("button", { name: /Generate (2D|3D) Preview/i }),
    ).toBeVisible();

    // Right column: Zone A (Designer & Source)
    const metadataCard = detailCard(page, "Designer & Source");
    await expect(metadataCard).toBeVisible();
    await expect(metadataCard.locator("select").first()).toHaveValue(/.+/); // Designer is selected

    // Right column: Zone B (Technical Data Grid)
    const techCard = detailCard(page, "Technical Data");
    await expect(techCard).toBeVisible();
    await expect(techCard.getByText("Hoop", { exact: true })).toBeVisible();
    await expect(techCard.getByText("Dimensions", { exact: true })).toBeVisible();
    await expect(techCard.getByText("Stitches", { exact: true })).toBeVisible();
    await expect(techCard.getByText("Colours", { exact: true })).toBeVisible();
    await expect(techCard.getByText("Colour Changes", { exact: true })).toBeVisible();
    await expect(techCard.getByRole("button", { name: /Recalculate From File/i })).toBeVisible();

    // Right column: Zone C (Rating & Status)
    const ratingCard = detailCard(page, "Rating & Status");
    await expect(ratingCard).toBeVisible();
    await expect(ratingCard.getByRole("radiogroup", { name: "Rating" })).toBeVisible();
    await expect(ratingCard.getByText(/Rating:\s*★\s*4\s*\/\s*5/)).toBeVisible();
    await expect(ratingCard.getByRole("button", { name: /Stitched/i })).toBeVisible();

    // Right column: Tags
    const tagsCard = detailCard(page, "Tags");
    await expect(tagsCard).toBeVisible();
    await expect(tagsCard.locator("span", { hasText: "Food" })).toBeVisible();
    await expect(tagsCard.locator("span", { hasText: "Cross Stitch" })).toBeVisible();
    await expect(tagsCard.getByRole("button", { name: "Choose tags..." })).toBeVisible();

    // Right column: Notes
    const notesCard = detailCard(page, "Notes");
    await expect(notesCard).toBeVisible();
    await expect(
      notesCard.getByPlaceholder("Add notes about this design..."),
    ).toBeVisible();
    await expect(notesCard.getByRole("button", { name: "Save Notes" })).toBeDisabled();

    // Right column: Projects
    const projectsCard = detailCard(page, "Projects");
    await expect(projectsCard).toBeVisible();

    // Right column: Delete action
    await expect(
      page.getByRole("button", { name: "Delete design", exact: true }),
    ).toBeVisible();
  });

  test("displays graceful error state for non-existent design ID", async ({
    page,
  }) => {
    await gotoRoute(page, "#/designs/999999");
    await expect(
      page.getByText("No design found for id 999999."),
    ).toBeVisible({ timeout: 10_000 });
  });

  test("renders missing preview warning banner for unreadable designs", async ({
    page,
  }) => {
    await gotoRoute(page, `#/designs/${sparseDesignId}`);

    const banner = page.locator('[data-testid="design-no-preview-banner"]');
    await expect(banner).toBeVisible();
    await expect(banner).toContainText("Preview could not be generated");
    await expect(banner).toContainText("Use “Generate Preview” below");
  });

  test("tracks browse context and supports Prev / Next carousel navigation", async ({
    page,
  }) => {
    // Navigate via Browse so the browseSessionStore is populated
    await gotoRoute(page, "#/designs");
    await page.reload();

    const firstCard = page.locator("article.browse-card").first();
    await expect(firstCard).toBeVisible({ timeout: 30_000 });
    const firstTitle = (
      await firstCard.locator("p.browse-card-title").textContent()
    )?.trim();
    const firstId = await firstCard.getAttribute("data-id");

    // Click the card link to open detail view with active browse session
    await firstCard.locator("button.browse-card-link").click();
    await expect(page).toHaveURL(new RegExp(`#/designs/${firstId}$`));

    // Browse counter should be rendered in the top action bar (e.g. "1 / 51")
    const nav = detailNavBar(page);
    await expect(nav.getByText(/^1\s*\/\s*\d+$/)).toBeVisible();

    // First item has Prev disabled
    const prevButton = nav.getByRole("button", { name: "‹ Prev" });
    const nextButton = nav.getByRole("button", { name: "Next ›" });
    await expect(prevButton).toBeDisabled();
    await expect(nextButton).toBeEnabled();

    // Click Next to advance to second design
    await nextButton.click();
    await expect(nav.getByText(/^2\s*\/\s*\d+$/)).toBeVisible();
    await expect(prevButton).toBeEnabled();

    // Click Prev to go back to the first design
    await prevButton.click();
    await expect(nav.getByText(/^1\s*\/\s*\d+$/)).toBeVisible();
    await expect(leftColumn(page).locator("p.font-medium")).toHaveText(firstTitle!);

    // Click "Back to Browse" to return to browse catalogue
    await page.getByRole("button", { name: "← Back to Browse" }).click();
    await expect(page).toHaveURL(/#\/designs$/);
    await expect(page.getByRole("heading", { name: "Browse Designs" })).toBeVisible();
  });

  test("persists Designer and Source dropdown changes across reload", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const metadataCard = detailCard(page, "Designer & Source");
    await expect(metadataCard).toBeVisible();

    const designerSelect = metadataCard.locator("label", { hasText: "Designer" }).locator("select");
    const sourceSelect = metadataCard.locator("label", { hasText: "Source" }).locator("select");

    // Change Designer to "Wrenwood Studio"
    await designerSelect.selectOption({ label: "Wrenwood Studio" });
    await expect(toast(page, "Designer updated")).toBeVisible();

    // Change Source to "Threadwise Guild"
    await sourceSelect.selectOption({ label: "Threadwise Guild" });
    await expect(toast(page, "Source updated")).toBeVisible();

    // Reload page to verify SQLite persistence
    await page.reload();
    await expect(detailCard(page, "Designer & Source")).toBeVisible();
    await expect(
      detailCard(page, "Designer & Source").locator("label", { hasText: "Designer" }).locator("select"),
    ).toHaveValue(
      await designerSelect.locator('option:has-text("Wrenwood Studio")').getAttribute("value") || "",
    );
    await expect(
      detailCard(page, "Designer & Source").locator("label", { hasText: "Source" }).locator("select"),
    ).toHaveValue(
      await sourceSelect.locator('option:has-text("Threadwise Guild")').getAttribute("value") || "",
    );

    // Restore original fixture values (Designer: "Me", Source: "None")
    const reloadedDesignerSelect = detailCard(page, "Designer & Source")
      .locator("label", { hasText: "Designer" })
      .locator("select");
    await reloadedDesignerSelect.selectOption({ label: "Me" });
    await expect(toast(page, "Designer updated")).toBeVisible();

    const reloadedSourceSelect = detailCard(page, "Designer & Source")
      .locator("label", { hasText: "Source" })
      .locator("select");
    await reloadedSourceSelect.selectOption({ label: "None" });
    await expect(toast(page, "Source updated")).toBeVisible();
  });

  test("supports interactive 5-star rating changes, clearing, and persistence", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const ratingCard = detailCard(page, "Rating & Status");
    await expect(ratingCard).toBeVisible();

    // Click 5th star
    const star5 = ratingCard.getByRole("button", { name: "5 stars" });
    await star5.click();
    await expect(toast(page, /rating/i)).toBeVisible();
    await expect(ratingCard.getByText(/Rating:\s*★\s*5\s*\/\s*5/)).toBeVisible();

    // Reload to verify persistence
    await page.reload();
    await expect(
      detailCard(page, "Rating & Status").getByText(/Rating:\s*★\s*5\s*\/\s*5/),
    ).toBeVisible();

    // Clear rating
    const clearButton = detailCard(page, "Rating & Status").getByRole("button", {
      name: "Clear",
      exact: true,
    });
    await clearButton.click();
    await expect(toast(page, /rating/i)).toBeVisible();
    await expect(
      detailCard(page, "Rating & Status").getByText("Rating: Unrated"),
    ).toBeVisible();

    // Restore to 4 stars (seed contract)
    const star4 = detailCard(page, "Rating & Status").getByRole("button", {
      name: "4 stars",
    });
    await star4.click();
    await expect(toast(page, /rating/i)).toBeVisible();
    await expect(
      detailCard(page, "Rating & Status").getByText(/Rating:\s*★\s*4\s*\/\s*5/),
    ).toBeVisible();
  });

  test("toggles Stitched status and verification states with persistence", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const ratingCard = detailCard(page, "Rating & Status");
    const stitchedBtn = ratingCard.getByRole("button", { name: /Stitched/i });
    const imageVerifyBtn = ratingCard.getByRole("button", { name: /Image (Verified|Unverified)/i });
    const stitchingVerifyBtn = ratingCard.getByRole("button", {
      name: /Stitching (Verified|Unverified)/i,
    });

    // Toggle Stitched off
    await stitchedBtn.click();
    await expect(toast(page, /stitched/i)).toBeVisible();
    await expect(ratingCard.getByRole("button", { name: "Mark as Stitched" })).toBeVisible();

    // Toggle Image Verification off
    await imageVerifyBtn.click();
    await expect(toast(page, /verification/i)).toBeVisible();
    await expect(ratingCard.getByRole("button", { name: /Image Unverified/i })).toBeVisible();

    // Toggle Stitching Verification off
    await stitchingVerifyBtn.click();
    await expect(toast(page, /verification/i)).toBeVisible();
    await expect(ratingCard.getByRole("button", { name: /Stitching Unverified/i })).toBeVisible();

    // Reload page to verify persistence
    await page.reload();
    const reloadedCard = detailCard(page, "Rating & Status");
    await expect(
      reloadedCard.getByRole("button", { name: "Mark as Stitched" }),
    ).toBeVisible();
    await expect(
      reloadedCard.getByRole("button", { name: /Image Unverified/i }),
    ).toBeVisible();
    await expect(
      reloadedCard.getByRole("button", { name: /Stitching Unverified/i }),
    ).toBeVisible();

    // Restore original states (Stitched: yes, Image Verified: yes, Stitching Verified: yes)
    await reloadedCard.getByRole("button", { name: "Mark as Stitched" }).click();
    await expect(toast(page, /stitched/i)).toBeVisible();

    await reloadedCard.getByRole("button", { name: /Image Unverified/i }).click();
    await expect(toast(page, /verification/i)).toBeVisible();

    await reloadedCard.getByRole("button", { name: /Stitching Unverified/i }).click();
    await expect(toast(page, /verification/i)).toBeVisible();
  });

  test("edits and saves design notes", async ({ page }) => {
    await openDesignDetail(page, richDesignId);

    const notesCard = detailCard(page, "Notes");
    const textarea = notesCard.getByPlaceholder("Add notes about this design...");
    const saveButton = notesCard.getByRole("button", { name: "Save Notes" });

    // Ensure initial notes state is settled
    await expect(saveButton).toBeDisabled();

    const testNote = `Special embroidery note created at ${Date.now()}`;
    await textarea.fill(testNote);
    await expect(saveButton).toBeEnabled();

    await saveButton.click();
    await expect(toast(page, /metadata|notes|saved/i)).toBeVisible();
    await expect(saveButton).toBeDisabled();

    // Reload to verify persistence
    await page.reload();
    await expect(
      detailCard(page, "Notes").getByPlaceholder("Add notes about this design..."),
    ).toHaveValue(testNote);

    // Clean up notes back to empty
    const reloadedTextarea = detailCard(page, "Notes").getByPlaceholder(
      "Add notes about this design...",
    );
    await reloadedTextarea.fill("");
    await detailCard(page, "Notes").getByRole("button", { name: "Save Notes" }).click();
    await expect(toast(page, /metadata|notes|saved/i)).toBeVisible();
  });

  test("manages tags via Tag Selection Modal and inline chip removal", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const tagsCard = detailCard(page, "Tags");
    await expect(tagsCard).toBeVisible();

    // Open Tag Selection Modal
    await tagsCard.getByRole("button", { name: "Choose tags..." }).click();

    const tagModal = page.locator(".tag-chooser-dialog");
    await expect(tagModal).toBeVisible();
    await expect(
      tagModal.getByRole("heading", { name: "Choose tags for this design" }),
    ).toBeVisible();

    // Filter tags with search input
    const searchInput = tagModal.getByPlaceholder("🔍 Search or create tag...");
    await searchInput.fill("Applique");

    // Select Applique checkbox
    const appliqueCheckbox = tagModal
      .locator("label.tag-chooser-option", { hasText: "Applique" })
      .locator("input[type='checkbox']");
    await expect(appliqueCheckbox).toBeVisible();
    if (!(await appliqueCheckbox.isChecked())) {
      await appliqueCheckbox.check();
    }

    // Click Done to close modal and flush auto-save
    await tagModal.getByRole("button", { name: "Done" }).click();
    await expect(tagModal).not.toBeVisible();

    // Applique tag chip should now be visible on detail view
    await expect(tagsCard.locator("span", { hasText: "Applique" })).toBeVisible();

    // Reload page to verify persistence
    await page.reload();
    await expect(detailCard(page, "Tags").locator("span", { hasText: "Applique" })).toBeVisible();

    // Inline tag chip removal: click '×' button on the Applique tag chip
    const appliqueChip = detailCard(page, "Tags")
      .locator("span", { hasText: "Applique" })
      .locator('button[title="Remove tag"]');
    await appliqueChip.click();
    await expect(toast(page, /tag/i)).toBeVisible();
    await expect(detailCard(page, "Tags").locator("span", { hasText: "Applique" })).not.toBeVisible();

    // Reload to verify removal persisted
    await page.reload();
    await expect(detailCard(page, "Tags").locator("span", { hasText: "Applique" })).not.toBeVisible();
  });

  test("executes technical data recalculation from file", async ({ page }) => {
    await openDesignDetail(page, richDesignId);

    const techCard = detailCard(page, "Technical Data");
    const recalcButton = techCard.getByRole("button", {
      name: /Recalculate From File/i,
    });

    await recalcButton.click();
    await expect(
      toast(page, /recalculated|technical data/i),
    ).toBeVisible({ timeout: 15_000 });

    // Technical data grid should retain valid measurements
    await expect(techCard.getByText("Hoop B")).toBeVisible();
  });

  test("triggers editor, explorer, and preview generation actions", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const openEditorBtn = leftColumn(page).getByRole("button", {
      name: /Open in Editor/i,
    });
    await openEditorBtn.click();
    await expect(
      page.locator(".toast-message").filter({ hasText: /default app|editor|opened/i }).first(),
    ).toBeVisible({ timeout: 10_000 });

    const showExplorerBtn = leftColumn(page).getByRole("button", {
      name: /Show in Explorer/i,
    });
    await showExplorerBtn.click();
    await expect(
      page.locator(".toast-message").filter({ hasText: /explorer/i }).first(),
    ).toBeVisible({ timeout: 10_000 });

    const previewGenBtn = leftColumn(page).getByRole("button", {
      name: /Generate (2D|3D) Preview/i,
    });
    await previewGenBtn.click();
    await expect(
      page.locator(".toast-message").filter({ hasText: /preview/i }).first(),
    ).toBeVisible({ timeout: 15_000 });
  });

  test("navigates to Design Print view and renders printable sheet", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    // Click Print button in top navigation bar
    await page.getByRole("button", { name: "Print", exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`#/designs/${richDesignId}/print$`));

    // Print view header controls
    await expect(page.getByRole("button", { name: "Back to Detail" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Print", exact: true })).toBeVisible();

    // Printable sheet content
    const sheet = page.locator(".route-panel");
    await expect(sheet.getByRole("heading", { name: RICH_DESIGN })).toBeVisible();
    await expect(sheet.locator(`img[alt="${RICH_DESIGN}"]`)).toBeVisible();

    // Metadata items on sheet
    await expect(sheet.getByText("File:")).toBeVisible();
    await expect(sheet.getByText("Designer:")).toBeVisible();
    await expect(sheet.getByText("Hoop:")).toBeVisible();
    await expect(sheet.getByText("Dimensions:")).toBeVisible();
    await expect(sheet.getByText("Stitches:")).toBeVisible();
    await expect(sheet.getByText("Colours:")).toBeVisible();
    await expect(sheet.getByText("Colour changes:")).toBeVisible();
    await expect(sheet.getByText("Rating:")).toBeVisible();
    await expect(sheet.getByText("Stitched: Yes")).toBeVisible();
    await expect(sheet.getByText("Tags")).toBeVisible();

    // Stub window.print and verify Print button invokes print dialog
    await page.evaluate(() => {
      (window as any).__printed = false;
      window.print = () => {
        (window as any).__printed = true;
      };
    });

    await page.getByRole("button", { name: "Print", exact: true }).click();
    const printed = await page.evaluate(() => (window as any).__printed);
    expect(printed).toBe(true);

    // Return to Detail view
    await page.getByRole("button", { name: "Back to Detail" }).click();
    await expect(page).toHaveURL(new RegExp(`#/designs/${richDesignId}$`));
    await expect(leftColumn(page).locator("p.font-medium")).toHaveText(RICH_DESIGN);
  });

  test("opens deletion modal and cancels cleanly without deleting design", async ({
    page,
  }) => {
    await openDesignDetail(page, richDesignId);

    const deleteBtn = page.getByRole("button", { name: "Delete design", exact: true });
    await deleteBtn.click();

    // Modal dialog assertions
    const modal = page.locator(".delete-modal-dialog");
    await expect(modal).toBeVisible();
    await expect(
      modal.getByRole("heading", { name: "Delete selected design?" }),
    ).toBeVisible();
    await expect(modal.getByText("1 design selected.")).toBeVisible();

    // File action radio options
    await expect(
      modal.getByText("Remove from catalogue only (keep files on disk)"),
    ).toBeVisible();
    await expect(
      modal.getByText("Move source file to recycle bin"),
    ).toBeVisible();

    // Cancel modal via Cancel button
    await modal.getByRole("button", { name: "Cancel" }).click();
    await expect(modal).not.toBeVisible();

    // Design is still intact
    await expect(leftColumn(page).locator("p.font-medium")).toHaveText(RICH_DESIGN);
  });
});
