import fs from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { gotoRoute, mainMenu } from "./helpers";
import { DATA_ROOT_PATH, DATABASE_FILENAME } from "./paths";

/**
 * Browse Designs end-to-end coverage.
 *
 * These tests run against the real Tauri app and the seeded test catalogue
 * (`tests/Test Assets/EmbroideryCatalogue.db`, copied to the throwaway data root
 * by `global-setup.ts`). They therefore depend on the documented Browse fixture
 * contract — see the README. The first test asserts that contract so a failure
 * points at the seed database rather than at the UI.
 *
 * The shared database is mutated by the batch-mutation tests at the end of the
 * file; every test before them is read-only, so declaration order matters.
 */

test.beforeAll(() => {
  const dbPath = path.join(DATA_ROOT_PATH, "Database", DATABASE_FILENAME);
  if (fs.existsSync(dbPath)) {
    try {
      const db = new DatabaseSync(dbPath);
      db.exec("DELETE FROM design_tags WHERE design_id NOT IN (5, 6, 7)");
      db.exec(
        "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 1 WHERE id NOT IN (4, 9, 10, 11)",
      );
      db.exec(
        "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 0 WHERE id = 4",
      );
      db.exec(
        "UPDATE designs SET image_tags_verified = 0, stitching_tags_verified = 0 WHERE id IN (9, 10, 11)",
      );
      db.close();
    } catch (err) {
      console.error("[browse.spec.ts] beforeAll reset error:", err);
    }
  }
});

const browseCards = (page: Page): Locator =>
  page.locator("article.browse-card");

const cardTitles = (page: Page): Promise<string[]> =>
  page
    .locator("p.browse-card-title")
    .allTextContents()
    .then((titles) => titles.map((title) => title.trim()));

/** The card whose filename label is exactly `filename`. */
const browseCard = (page: Page, filename: string): Locator =>
  browseCards(page).filter({
    has: page.locator("p.browse-card-title", { hasText: filename }),
  });

const selectionHeader = (page: Page): Locator =>
  page.locator(".selection-header");
const drawer = (page: Page): Locator =>
  page.locator("details.browse-additional-filters");
const sortSelect = (page: Page): Locator =>
  page.locator('select:has(option[value="date_added"])');
const dirSelect = (page: Page): Locator =>
  page.locator('select:has(option[value="desc"])');
const hoopSelect = (page: Page): Locator =>
  drawer(page).locator('select:has(option[value="__hoop_unknown__"])');
const ratingSelect = (page: Page): Locator =>
  drawer(page).locator('select:has(option[value="5"])');
const stitchedSelect = (page: Page): Locator =>
  drawer(page).locator('select:has(option[value="yes"])');
const resetButton = (page: Page): Locator =>
  page.getByRole("button", { name: "Reset filters" });
const needsAttention = (page: Page): Locator =>
  drawer(page)
    .locator("label", { hasText: "Needs attention" })
    .locator("input");
const unverifiedOnly = (page: Page): Locator =>
  page.locator("label", { hasText: "Unverified only" }).locator("input");

async function openBrowse(page: Page): Promise<void> {
  await gotoRoute(page, "#/designs");
  // The app under test is worker-scoped and shared across the whole file, so a
  // reload is the only reliable way to discard component-local state (selection,
  // open drawer) *and* the module-level browseSessionStore singleton between
  // tests. The hash survives the reload.
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Browse Designs" }),
  ).toBeVisible();
  await expect
    .poll(() => browseCards(page).count(), { timeout: 30_000 })
    .toBeGreaterThan(0);
}

/** Expand the Additional Filters drawer if it is collapsed. */
async function openFilters(page: Page): Promise<void> {
  const isOpen = await drawer(page).evaluate((el) => el.hasAttribute("open"));
  if (!isOpen) {
    await page.locator("summary.browse-additional-summary").click();
  }
  await expect(drawer(page).locator("span:text-is('Designer')")).toBeVisible();
}

/** Fill the general search box and wait out the live-query debounce. */
async function search(page: Page, term: string): Promise<void> {
  await page.locator("#browse-q").fill(term);
  // `q` is debounced by 250 ms before the backend re-query.
  await page.waitForTimeout(600);
}

/** Await a settled, order-independent result set of filenames. */
async function expectTitles(page: Page, expected: string[]): Promise<void> {
  const wanted = [...expected].sort();
  await expect
    .poll(async () => (await cardTitles(page)).sort(), { timeout: 20_000 })
    .toEqual(wanted);
}

/**
 * Await until the visible card titles contain every expected filename.
 *
 * Used where the exact membership is data-derived — e.g. hoop assignment, which
 * the importer computes from the design's dimensions at import time — rather
 * than authored metadata that the fixture controls.
 */
async function expectTitlesContain(
  page: Page,
  expected: string[],
): Promise<void> {
  await expect
    .poll(
      async () => {
        const titles = await cardTitles(page);
        return expected.every((title) => titles.includes(title));
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

/** Set the "Search in" checkboxes to the supplied combination. */
async function setSearchIn(
  page: Page,
  combination: { file: boolean; folder: boolean; tags: boolean },
): Promise<void> {
  const boxes: Array<[string, boolean]> = [
    ["#search-filename-checkbox", combination.file],
    ["#search-folder-checkbox", combination.folder],
    ["#search-tags-checkbox", combination.tags],
  ];
  for (const [selector, value] of boxes) {
    const box = page.locator(selector);
    if ((await box.isChecked()) !== value) {
      await box.click();
    }
  }
}

/** The multi-select list container for a drawer section (e.g. "Designer"). */
function filterList(page: Page, sectionLabel: string): Locator {
  return drawer(page).locator("div.space-y-1", {
    has: page.locator(`span:text-is('${sectionLabel}')`),
  });
}

/** A checkbox for one option inside a drawer multi-select list. */
function optionCheckbox(
  page: Page,
  sectionLabel: string,
  option: string,
): Locator {
  return filterList(page, sectionLabel)
    .locator("label")
    .filter({ has: page.locator(`span:text-is('${option}')`) })
    .locator("input");
}

test.describe("fixture contract", () => {
  test("the seeded catalogue matches the Browse fixture contract", async ({
    page,
  }) => {
    await openBrowse(page);

    // Two deliberately unreadable designs supply the "needs attention" set.
    await search(page, "ZZ-broken");
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
    for (const name of ["ZZ-broken.pes", "ZZ-broken-2.pes"]) {
      await expect(
        browseCard(page, name).locator(
          '[data-testid="design-card-no-preview"]',
        ),
      ).toBeVisible();
    }

    // A healthy design with both tags, a hoop, a rating and a preview proves the
    // negative case for the "Needs attention" filter.
    await search(page, "Cake 3.jef");
    await expectTitles(page, ["Cake 3.jef"]);
    const cake = browseCard(page, "Cake 3.jef");
    await expect(cake.locator("img.browse-card-image")).toBeVisible();
    await expect(cake.locator(".browse-card-tags")).toContainText("Food");
    await expect(cake.locator(".browse-card-tags")).toContainText(
      "Cross Stitch",
    );
    await expect(cake.locator(".browse-card-hoop")).toHaveText("Hoop B");
    await expect(
      cake.locator('[aria-label="Rating 4 out of 5"]'),
    ).toBeVisible();
    await expect(cake.locator('[aria-label="Verified"]')).toBeVisible();
  });
});

test.describe("initial load", () => {
  test("renders the shell, defaults and the first page of results", async ({
    page,
  }) => {
    await openBrowse(page);

    // Top-level navigation is present.
    const nav = mainMenu(page);
    for (const name of ["Browse", "Import", "Projects", "Help"]) {
      await expect(nav.getByRole("link", { name, exact: true })).toBeVisible();
    }

    // Search controls and defaults.
    await expect(page.locator("#browse-q")).toHaveAttribute(
      "placeholder",
      'e.g. rose "cross stitch" -applique or *.hus',
    );
    await expect(page.locator("#search-filename-checkbox")).toBeChecked();
    await expect(page.locator("#search-folder-checkbox")).toBeChecked();
    await expect(page.locator("#search-tags-checkbox")).toBeChecked();
    await expect(page.locator("p.browse-general-help")).toContainText(
      "Supports Google-like syntax",
    );
    await expect(unverifiedOnly(page)).not.toBeChecked();

    // Sort/direction defaults and the disabled reset action.
    await expect(sortSelect(page)).toHaveValue("name");
    await expect(dirSelect(page)).toHaveValue("asc");
    await expect(resetButton(page)).toBeVisible();
    await expect(resetButton(page)).toBeDisabled();

    // Status line and selection controls.
    await expect(selectionHeader(page)).toContainText("designs found");
    await expect(selectionHeader(page)).toContainText("0 of");
    await expect(selectionHeader(page)).toContainText("selected");
    await expect(
      page.locator("label", { hasText: "Select all on page" }).locator("input"),
    ).not.toBeChecked();

    // The batch toolbar is hidden while nothing is selected.
    await expect(page.locator(".browse-bulk-bar")).toBeHidden();
  });
});

test.describe("additional filters drawer", () => {
  test("toggles open/closed and exposes the filter fields", async ({
    page,
  }) => {
    await openBrowse(page);
    const summary = page.locator("summary.browse-additional-summary");

    // Collapsed by default.
    await expect(drawer(page).locator("span:text-is('Designer')")).toBeHidden();

    await summary.click();
    for (const label of [
      "Designer",
      "Image tags",
      "Stitching tags",
      "Source",
      "Hoop size",
      "Minimum rating",
      "Stitched",
      "Needs attention",
    ]) {
      await expect(
        drawer(page).locator(`span:text-is('${label}')`),
      ).toBeVisible();
    }
    await expect(
      drawer(page).locator("p", { hasText: "Designs with no preview image" }),
    ).toContainText("the file may be corrupt or unreadable.");
    await expect(needsAttention(page)).not.toBeChecked();

    await summary.click();
    await expect(drawer(page).locator("span:text-is('Designer')")).toBeHidden();
  });

  test("Needs attention enables Reset filters, which clears it", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);

    await expect(resetButton(page)).toBeDisabled();
    await needsAttention(page).check();
    await expect(resetButton(page)).toBeEnabled();

    await resetButton(page).click();
    await expect(needsAttention(page)).not.toBeChecked();
    await expect(resetButton(page)).toBeDisabled();
  });
});

test.describe("needs attention", () => {
  test("restricts to designs with no stored preview", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    await needsAttention(page).check();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
    await expect(
      browseCard(page, "ZZ-broken.pes").locator(
        '[data-testid="design-card-no-preview"]',
      ),
    ).toBeVisible();

    await needsAttention(page).uncheck();
    await expect
      .poll(() => browseCards(page).count(), { timeout: 20_000 })
      .toBeGreaterThan(2);
  });

  test("narrows the intersection when combined with another filter", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);

    await needsAttention(page).check();
    // Let the Needs-attention reload settle before applying the second filter:
    // otherwise the two backend loads overlap and the earlier result can land
    // last, leaving the grid showing the needs-attention set.
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);

    // Both Thistlebury Stitch designs have previews, so the AND intersection is
    // empty (the two flagged designs have no designer at all).
    await optionCheckbox(page, "Designer", "Thistlebury Stitch").click();
    await expect(selectionHeader(page)).toContainText("0 designs found", {
      timeout: 20_000,
    });
    await expect(
      page.getByText("No designs match your filters."),
    ).toBeVisible();
  });

  test("shows an empty state without SQL errors", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    await needsAttention(page).check();
    await search(page, "Cake 3");
    await expect(selectionHeader(page)).toContainText("0 designs found", {
      timeout: 20_000,
    });
    await expect(
      page.getByText("No designs match your filters."),
    ).toBeVisible();

    await resetButton(page).click();
    await expect(needsAttention(page)).not.toBeChecked();
    await expect(page.locator("#browse-q")).toHaveValue("");
  });
});

test.describe("search syntax", () => {
  test("supports exact phrases, OR groups and wildcards", async ({ page }) => {
    await openBrowse(page);

    // Exact phrase matches a tag on one design and a filename on another.
    await search(page, '"Cross Stitch"');
    await expectTitles(page, ["Cake 3 Cross Stitch Fred.jef", "Cake 3.jef"]);

    // Upper-case OR unions the two groups.
    await search(page, "Cake 3 OR Bean");
    await expectTitles(page, [
      "Bean X.jef",
      "Bean Z.jef",
      "Cake 3 - Food.jef",
      "Cake 3 - to be verified.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "Cake 3.jef",
    ]);

    // Wildcard extension: every .jef design, and no .pes design.
    await search(page, "*.jef");
    await expect(browseCard(page, "Bean X.jef")).toBeVisible();
    await expect(browseCard(page, "ZZ-broken.pes")).toHaveCount(0);
  });
});

test.describe("search-in scoping", () => {
  test("scopes the query to the ticked metadata fields", async ({ page }) => {
    await openBrowse(page);

    // File name only.
    await setSearchIn(page, { file: true, folder: false, tags: false });
    await search(page, "Cross");
    await expectTitles(page, ["Cake 3 Cross Stitch Fred.jef"]);

    // Tags only: matches the tag, not the similarly-named file.
    await setSearchIn(page, { file: false, folder: false, tags: true });
    await search(page, "Cross");
    await expectTitles(page, ["Cake 3.jef"]);
    await search(page, "Food");
    await expectTitles(page, ["Cake 3.jef"]);

    // Folder name only. The backend matches the whole canonical relative path,
    // so a design whose *filename* contains the term matches too.
    await setSearchIn(page, { file: false, folder: true, tags: false });
    await search(page, "Cross");
    await expectTitles(page, ["Bean Z.jef", "Cake 3 Cross Stitch Fred.jef"]);

    // Nothing ticked: no metadata field can match.
    await setSearchIn(page, { file: false, folder: false, tags: false });
    await search(page, "Cake");
    await expect(selectionHeader(page)).toContainText("0 designs found", {
      timeout: 20_000,
    });
    await expect(
      page.getByText("No designs match your filters."),
    ).toBeVisible();
  });
});

test.describe("reference filters", () => {
  test("filters by designer, tag and source", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    // Designers: OR within the category.
    await optionCheckbox(page, "Designer", "Thistlebury Stitch").click();
    await optionCheckbox(page, "Designer", "Wrenwood Studio").click();
    await expectTitles(page, [
      "Bean X.jef",
      "Bean Z.jef",
      "Cake Applique 2.jef",
    ]);
    await resetButton(page).click();

    // AND across categories: designer Me AND stitching tag Filled.
    await optionCheckbox(page, "Designer", "Me").click();
    await optionCheckbox(page, "Stitching tags", "Filled").click();
    await expectTitles(page, ["Cake Applique.jef"]);
    await resetButton(page).click();

    // Stitching tags are OR-ed within the category.
    await optionCheckbox(page, "Stitching tags", "Applique").click();
    await optionCheckbox(page, "Stitching tags", "Filled").click();
    await expectTitles(page, ["Cake Applique 2.jef", "Cake Applique.jef"]);
    await resetButton(page).click();

    // Source list.
    await optionCheckbox(page, "Source", "Threadwise Guild").click();
    await expectTitles(page, ["Bean Z.jef"]);
  });

  test("filters by hoop, minimum rating and stitched status", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);

    // Hoop assignment is derived at import time (the smallest hoop the design
    // fits), so exact membership is a property of the fixture files rather than
    // authored metadata. Assert the filter matches by hoop and excludes designs
    // that have no hoop.
    await hoopSelect(page).selectOption("Hoop A");
    await expectTitlesContain(page, ["Cake Applique 2.jef"]);
    await expect(browseCard(page, "Cake 3.jef")).toHaveCount(0);
    await expect(browseCard(page, "ZZ-broken.pes")).toHaveCount(0);

    await hoopSelect(page).selectOption("Hoop B");
    await expectTitlesContain(page, ["Cake 3.jef", "Cake Applique.jef"]);
    await expect(browseCard(page, "ZZ-broken.pes")).toHaveCount(0);
    await hoopSelect(page).selectOption("");

    await ratingSelect(page).selectOption("4");
    await expectTitles(page, ["Cake 3.jef", "Cake Applique.jef"]);
    await ratingSelect(page).selectOption("");

    await stitchedSelect(page).selectOption("yes");
    await expectTitles(page, ["Cake 3.jef", "Cake Applique.jef"]);

    await stitchedSelect(page).selectOption("no");
    await expect(browseCard(page, "Cake Applique 2.jef")).toBeVisible();
    await expect(browseCard(page, "Cake Applique.jef")).toHaveCount(0);
  });
});

test.describe("unverified only", () => {
  test("restricts to designs with an unverified tag group", async ({
    page,
  }) => {
    await openBrowse(page);

    // A design counts as unverified when either tag group is unverified, so the
    // partially verified design is included alongside the fully unverified ones.
    await unverifiedOnly(page).check();
    await expectTitles(page, [
      "Cake 3 - to be verified.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "ZZ-broken-2.pes",
      "ZZ-broken.pes",
    ]);

    await unverifiedOnly(page).uncheck();
    await expect
      .poll(() => browseCards(page).count(), { timeout: 20_000 })
      .toBeGreaterThan(4);
  });
});

test.describe("sorting and direction", () => {
  test("reorders the grid and resets to the defaults", async ({ page }) => {
    await openBrowse(page);
    await search(page, "Cake 3");

    const ascending = await cardTitles(page);
    expect(ascending.length).toBe(4);

    await dirSelect(page).selectOption("desc");
    await expect
      .poll(() => cardTitles(page), { timeout: 15_000 })
      .toEqual([...ascending].reverse());

    await dirSelect(page).selectOption("asc");
    await expect
      .poll(() => cardTitles(page), { timeout: 15_000 })
      .toEqual(ascending);

    // Each sort option re-queries without changing the result set.
    await sortSelect(page).selectOption("rating");
    await expect(sortSelect(page)).toHaveValue("rating");
    await expect
      .poll(async () => (await cardTitles(page)).sort(), { timeout: 15_000 })
      .toEqual([...ascending].sort());

    await resetButton(page).click();
    await expect(sortSelect(page)).toHaveValue("name");
    await expect(dirSelect(page)).toHaveValue("asc");
    await expect(page.locator("#browse-q")).toHaveValue("");
  });
});

test.describe("selection and bulk toolbar", () => {
  test("supports card, row and page selection, then clears it", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3");
    const header = selectionHeader(page);

    // Card-level selection mounts the batch toolbar.
    await page.locator(".browse-design-checkbox").first().check();
    await expect(header).toContainText("1 of 4 selected");
    await expect(page.locator(".browse-bulk-bar")).toBeVisible();
    for (const name of [
      "Choose tags",
      "Verify tags",
      "Delete selected",
      "Clear selection",
    ]) {
      await expect(
        page.getByRole("button", { name, exact: true }),
      ).toBeVisible();
    }
    await expect(
      page.locator("summary", { hasText: "Add to project…" }),
    ).toBeVisible();

    // Row-level selection selects every card in the row (one row of four here).
    await page.locator(".browse-row-checkbox").first().check();
    await expect(header).toContainText("4 of 4 selected");
    await expect(
      page.locator("label", { hasText: "Select all on page" }).locator("input"),
    ).toBeChecked();

    await page.getByRole("button", { name: "Clear selection" }).click();
    await expect(header).toContainText("0 of 4 selected");
    await expect(page.locator(".browse-bulk-bar")).toBeHidden();
  });
});

test.describe("pagination", () => {
  test("pages through a result set larger than one page", async ({ page }) => {
    await openBrowse(page);

    const nav = page.getByRole("navigation", { name: "Browse pagination" });
    await expect(nav).toBeVisible();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("1");

    await nav.getByRole("button", { name: /Next/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("2");
    await expect(browseCards(page).first()).toBeVisible();

    await nav.getByRole("button", { name: /First/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("1");
  });
});

test.describe("navigation persistence", () => {
  test("preserves the Needs attention filter across a detail round-trip", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);
    await needsAttention(page).check();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);

    // Open a flagged design, then return to Browse.
    await browseCards(page).first().locator(".browse-card-link").click();
    await expect(page).toHaveURL(/#\/designs\/\d+/);

    await gotoRoute(page, "#/designs");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
    await expect(needsAttention(page)).toBeChecked();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
  });
});

// ---------------------------------------------------------------------------
// Browse contract extensions. These mirror the Browse Designs user test plan
// (`docs/User Test Plans/Browse Designs.md`). Everything below is read-only:
// modals are cancelled, project checkboxes are never applied, so the shared
// catalogue is untouched and the mutating suite remains the tail of this file.
// ---------------------------------------------------------------------------

/** Parse the "0 of Z selected" page size from the selection header. */
async function pageSizeFromHeader(page: Page): Promise<number> {
  const text = (await selectionHeader(page).innerText()).replace(/\s+/g, " ");
  const match = text.match(/of\s+(\d+)\s+selected/i);
  return match ? Number(match[1]) : 0;
}

/** Parse the "X designs found" total from the selection header. */
async function filteredCount(page: Page): Promise<number> {
  const text = (await selectionHeader(page).innerText()).replace(/\s+/g, " ");
  const match = text.match(/(\d+)\s+designs? found/i);
  return match ? Number(match[1]) : 0;
}

/** Parse the "N designs selected" count from the batch action toolbar. */
async function selectedCount(page: Page): Promise<number> {
  const text = (await page.locator(".browse-bulk-bar").innerText()).replace(
    /\s+/g,
    " ",
  );
  const match = text.match(/(\d+)\s+designs? selected/i);
  return match ? Number(match[1]) : 0;
}

const selectAllOnPage = (page: Page): Locator =>
  page.locator("label", { hasText: "Select all on page" }).locator("input");

const paginationNav = (page: Page): Locator =>
  page.getByRole("navigation", { name: "Browse pagination" });

test.describe("card presentation", () => {
  test("renders the 4-state verification indicator", async ({ page }) => {
    await openBrowse(page);

    // Both tag groups verified -> green check.
    await search(page, "Cake 3.jef");
    const verifiedBadge = browseCard(page, "Cake 3.jef").locator(
      '[aria-label="Verified"]',
    );
    await expect(verifiedBadge).toBeVisible();
    await expect(verifiedBadge).toContainText("✓");
    await expect(verifiedBadge).toHaveClass(/bg-green-500/);

    // Image verified, stitching unverified -> amber half indicator.
    await search(page, "Cross Stitch Fred");
    const imageVerifiedBadge = browseCard(
      page,
      "Cake 3 Cross Stitch Fred.jef",
    ).locator('[aria-label="Image Verified, Stitching Unverified"]');
    await expect(imageVerifiedBadge).toBeVisible();
    await expect(imageVerifiedBadge).toContainText("◐");
    await expect(imageVerifiedBadge).toHaveClass(/bg-amber-400/);

    // Neither group verified -> red unverified indicator with white circle.
    await search(page, "to be verified");
    const unverified = browseCard(page, "Cake 3 - to be verified.jef");
    const unverifiedBadge = unverified.locator('[aria-label="Unverified"]');
    await expect(unverifiedBadge).toBeVisible();
    await expect(unverifiedBadge).toContainText("○");
    await expect(unverifiedBadge).toHaveClass(/bg-red-500/);
    await expect(unverified.locator('[aria-label="Verified"]')).toHaveCount(0);
    await expect(
      unverified.locator('[aria-label="Image Verified, Stitching Unverified"]'),
    ).toHaveCount(0);
    await expect(
      unverified.locator('[aria-label="Stitching Verified, Image Unverified"]'),
    ).toHaveCount(0);
  });

  test("renders the rating marker, unknown hoop and no-preview placeholder", async ({
    page,
  }) => {
    await openBrowse(page);

    await search(page, "Cake 3.jef");
    await expect(
      browseCard(page, "Cake 3.jef").locator('[aria-label="Rating 4 out of 5"]'),
    ).toContainText("4");

    // The unreadable fixtures have no preview, no rating and no hoop.
    await search(page, "ZZ-broken");
    for (const name of ["ZZ-broken.pes", "ZZ-broken-2.pes"]) {
      const card = browseCard(page, name);
      await expect(
        card.locator('[data-testid="design-card-no-preview"]'),
      ).toHaveText(
        "Preview could not be generated — the file may be corrupt or unreadable",
      );
      await expect(card.locator('[aria-label="Not rated"]')).toContainText("—");
      await expect(card.locator(".browse-card-hoop")).toHaveText(
        "Hoop unknown",
      );
    }
  });

  test("sizes each page from the grid column count", async ({ page }) => {
    await openBrowse(page);

    const pageSize = await pageSizeFromHeader(page);
    expect(pageSize).toBeGreaterThan(0);
    // Page size is 10 rows x the responsive column count.
    expect(pageSize % 10).toBe(0);
    await expect.poll(() => browseCards(page).count()).toBe(pageSize);
  });
});

test.describe("additional filter controls", () => {
  test("filters by the unknown/unset hoop", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    await hoopSelect(page).selectOption("__hoop_unknown__");
    await expectTitlesContain(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
    await expect(browseCard(page, "Cake 3.jef")).toHaveCount(0);

    await hoopSelect(page).selectOption("");
    await expect
      .poll(() => browseCards(page).count(), { timeout: 20_000 })
      .toBeGreaterThan(2);
  });

  test("exposes the documented dropdown option sets", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    await expect(
      hoopSelect(page).locator('option[value="__hoop_unknown__"]'),
    ).toHaveText("Hoop unknown");
    await expect(ratingSelect(page).locator("option")).toHaveText([
      "Any",
      "1★",
      "2★",
      "3★",
      "4★",
      "5★",
    ]);
    await expect(stitchedSelect(page).locator("option")).toHaveText([
      "Any",
      "Stitched",
      "Not Stitched",
    ]);
  });

  test("filters by image tags and stitching tags independently", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);

    // Image tags filter only by image tags.
    await optionCheckbox(page, "Image tags", "Flowers").click();
    await expectTitlesContain(page, ["Cake Applique.jef"]);
    await expect(browseCard(page, "Cake Applique 2.jef")).toHaveCount(0);
    await resetButton(page).click();

    // Stitching tags filter only by stitching tags.
    await optionCheckbox(page, "Stitching tags", "Filled").click();
    await expectTitles(page, ["Cake Applique.jef"]);
    await resetButton(page).click();

    // The two tag categories combine with AND.
    await optionCheckbox(page, "Image tags", "Flowers").click();
    await optionCheckbox(page, "Stitching tags", "Filled").click();
    await expectTitles(page, ["Cake Applique.jef"]);
  });

  test("enables Reset filters for any active filter and clears them all", async ({
    page,
  }) => {
    await openBrowse(page);
    await openFilters(page);
    await expect(resetButton(page)).toBeDisabled();

    // A general search term alone enables Reset.
    await search(page, "Cake 3");
    await expect(resetButton(page)).toBeEnabled();

    // Add one of every other filter kind.
    await optionCheckbox(page, "Designer", "Me").click();
    await hoopSelect(page).selectOption("Hoop B");
    await ratingSelect(page).selectOption("4");
    await stitchedSelect(page).selectOption("yes");
    await unverifiedOnly(page).check();
    await needsAttention(page).check();

    await resetButton(page).click();
    await expect(page.locator("#browse-q")).toHaveValue("");
    await expect(unverifiedOnly(page)).not.toBeChecked();
    await expect(needsAttention(page)).not.toBeChecked();
    await expect(hoopSelect(page)).toHaveValue("");
    await expect(ratingSelect(page)).toHaveValue("");
    await expect(stitchedSelect(page)).toHaveValue("");
    await expect(resetButton(page)).toBeDisabled();
    await expect
      .poll(() => browseCards(page).count(), { timeout: 20_000 })
      .toBeGreaterThan(2);
  });
});

test.describe("selection cap and delete confirmation", () => {
  test("sizes the batch selection to the page and scopes it to the page", async ({
    page,
  }) => {
    await openBrowse(page);

    const total = await filteredCount(page);
    const pageSize = await pageSizeFromHeader(page);
    // A batch holds at most 50 designs (BROWSE_BULK_DELETE_MAX) and a page
    // renders at most 10 rows x 5 columns, so the cap is never exceeded.
    expect(pageSize).toBeGreaterThan(0);
    expect(pageSize).toBeLessThanOrEqual(50);
    expect(total).toBeGreaterThanOrEqual(pageSize);

    await selectAllOnPage(page).check();
    await expect(page.locator(".browse-bulk-bar")).toBeVisible();
    await expect
      .poll(() => selectedCount(page), { timeout: 15_000 })
      .toBe(pageSize);

    // Selection is scoped to the visible page: paging discards it.
    const nav = paginationNav(page);
    if ((await nav.getByRole("button", { name: /Next/ }).count()) > 0) {
      await nav.getByRole("button", { name: /Next/ }).click();
      await expect(browseCards(page).first()).toBeVisible();
      await expect(page.locator(".browse-bulk-bar")).toBeHidden();
    }
  });

  test("locks selection while the delete confirmation is open", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3");

    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Delete selected" }).click();

    const dialog = page.getByRole("dialog");
    await expect(
      dialog.getByRole("heading", { name: /Delete selected design/ }),
    ).toBeVisible();
    await expect(dialog.getByText("1 design selected.")).toBeVisible();
    await expect(page.locator(".browse-design-checkbox").first()).toBeDisabled();

    // Cancel leaves the catalogue and the selection untouched.
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toBeHidden();
    await expect(page.locator(".browse-design-checkbox").first()).toBeEnabled();
  });

  test("keeps the batch toolbar fixed while the grid scrolls", async ({
    page,
  }) => {
    await openBrowse(page);
    await page.locator(".browse-design-checkbox").first().check();
    await expect(page.locator(".browse-bulk-bar")).toBeVisible();

    const position = await page
      .locator(".browse-bulk-bar")
      .evaluate((el: Element) => getComputedStyle(el).position);
    expect(position).toBe("fixed");
  });
});

test.describe("add to project", () => {
  test("bulk dropdown shows the empty state and a disabled Apply", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3");

    await page.locator(".browse-design-checkbox").first().check();
    const bar = page.locator(".browse-bulk-bar");
    const summary = bar.locator("summary", { hasText: "Add to project" });
    await summary.click();

    const dropdown = bar.locator("details.relative > div");
    // The seed catalogue has no projects, so the picker renders its empty state
    // and Apply stays disabled (it only enables once a project is ticked).
    await expect(
      dropdown.getByText("No projects found. Create one first."),
    ).toBeVisible();
    const apply = bar.getByRole("button", { name: "Apply" });
    await expect(apply).toBeDisabled();

    // Close without applying so nothing is committed.
    await summary.click();
    await expect(apply).toBeHidden();
    await page.getByRole("button", { name: "Clear selection" }).click();
  });

  test("per-card bar shows the same project picker", async ({ page }) => {
    await openBrowse(page);
    await search(page, "Cake 3.jef");

    const card = browseCard(page, "Cake 3.jef");
    await card.locator("summary.browse-card-project-summary").click();
    await expect(
      card
        .locator(".browse-card-project-details")
        .getByText("No projects found. Create one first."),
    ).toBeVisible();
  });
});

test.describe("tag chooser semantics", () => {
  test("closes on the backdrop and names its taxonomy sections", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3.jef");
    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Choose tags" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog.getByText("Image tags")).toBeVisible();
    await expect(dialog.getByText("Stitching tags")).toBeVisible();

    await expect(
      dialog.getByRole("button", { name: "Close tag chooser" }),
    ).toBeAttached();
    // Click the full-screen backdrop away from the centred dialog panel.
    await page.mouse.click(5, 5);
    await expect(dialog).toBeHidden();
    await page.getByRole("button", { name: "Clear selection" }).click();
  });

  test("shows a mixed state for disagreeing tags and Untagged clears it", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3");

    // Cake 3.jef carries Food + Cross Stitch; Cake 3 - Food.jef carries none,
    // so those tags are indeterminate across the selection.
    await browseCard(page, "Cake 3.jef")
      .locator(".browse-design-checkbox")
      .check();
    await browseCard(page, "Cake 3 - Food.jef")
      .locator(".browse-design-checkbox")
      .check();
    await page.getByRole("button", { name: "Choose tags" }).click();

    const dialog = page.getByRole("dialog");
    const mixed = dialog.locator('[role="checkbox"][aria-checked="mixed"]');
    await expect
      .poll(() => mixed.count(), { timeout: 10_000 })
      .toBeGreaterThan(0);
    await expect(mixed.first().locator(".tag-chooser-box")).toHaveText("−");

    // Untagged is replace mode: it wipes the mixed/add/remove state on screen.
    await dialog.getByRole("checkbox", { name: /Untagged/ }).check();
    await expect(
      dialog.locator('[role="checkbox"][aria-checked="mixed"]'),
    ).toHaveCount(0);

    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toBeHidden();
    await page.getByRole("button", { name: "Clear selection" }).click();
  });
});

test.describe("tag chooser toggling", () => {
  test("cycles a shared tag off and back on without committing", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3.jef");
    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Choose tags" }).click();

    const dialog = page.getByRole("dialog");
    const food = dialog.getByRole("checkbox", { name: "Food" });
    await expect(food).toHaveAttribute("aria-checked", "true");
    await food.click();
    await expect(food).toHaveAttribute("aria-checked", "false");
    await food.click();
    await expect(food).toHaveAttribute("aria-checked", "true");

    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toBeHidden();
    await page.getByRole("button", { name: "Clear selection" }).click();
  });
});

test.describe("sort keys and pagination controls", () => {
  test("re-queries each sort key without changing the result set", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Cake 3");
    const expected = [
      "Cake 3 - Food.jef",
      "Cake 3 - to be verified.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "Cake 3.jef",
    ];

    for (const key of ["folder", "date_added", "rating", "stitched"]) {
      await sortSelect(page).selectOption(key);
      await expect(sortSelect(page)).toHaveValue(key);
      await expect
        .poll(async () => (await cardTitles(page)).sort(), { timeout: 15_000 })
        .toEqual([...expected].sort());
    }
  });

  test("resets to page 1 when the sort key changes", async ({ page }) => {
    await openBrowse(page);
    const nav = paginationNav(page);
    await nav.getByRole("button", { name: /Next/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("2");

    await sortSelect(page).selectOption("rating");
    await expect(nav.locator('[aria-current="page"]')).toHaveText("1");
  });

  test("supports First, Prev, Next and Last", async ({ page }) => {
    await openBrowse(page);
    const nav = paginationNav(page);

    // Last -> the final page, where Next is not offered.
    await nav.getByRole("button", { name: /Last/ }).click();
    const lastPage = Number(
      (await nav.locator('[aria-current="page"]').innerText()).trim(),
    );
    expect(lastPage).toBeGreaterThan(1);
    await expect(nav.getByRole("button", { name: /Next/ })).toHaveCount(0);

    // First -> page 1, where Prev is not offered.
    await nav.getByRole("button", { name: /First/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("1");
    await expect(nav.getByRole("button", { name: /Prev/ })).toHaveCount(0);

    // Next then Prev step forward and back.
    await nav.getByRole("button", { name: /Next/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("2");
    await nav.getByRole("button", { name: /Prev/ }).click();
    await expect(nav.locator('[aria-current="page"]')).toHaveText("1");
  });
});

test.describe("search operators", () => {
  test("exposes the Search help link", async ({ page }) => {
    await openBrowse(page);
    const link = page.locator("p.browse-general-help a");
    await expect(link).toHaveAttribute("href", "#/help?section=search");
    await link.click();
    await expect(page).toHaveURL(/#\/help/);
  });

  test("treats OR case-insensitively", async ({ page }) => {
    await openBrowse(page);
    const union = [
      "Bean X.jef",
      "Bean Z.jef",
      "Cake 3 - Food.jef",
      "Cake 3 - to be verified.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "Cake 3.jef",
    ];

    await search(page, "Cake 3 OR Bean");
    await expectTitles(page, union);

    // The `OR` operator is matched case-insensitively.
    await search(page, "Cake 3 or Bean");
    await expectTitles(page, union);
  });
});

test.describe("search scoping re-runs", () => {
  test("supports an exclusion-only query", async ({ page }) => {
    await openBrowse(page);

    await search(page, "-cross");
    // Everything that only matched via "cross" is removed...
    for (const name of [
      "Bean Z.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "Cake 3.jef",
    ]) {
      await expect(browseCard(page, name)).toHaveCount(0);
    }
    // ...while unrelated designs remain.
    await expect(browseCard(page, "Bean X.jef")).toBeVisible();
  });

  test("re-runs the query when the Search in scope changes", async ({
    page,
  }) => {
    await openBrowse(page);
    await setSearchIn(page, { file: true, folder: true, tags: true });
    await search(page, "Cross");
    await expectTitlesContain(page, [
      "Bean Z.jef",
      "Cake 3 Cross Stitch Fred.jef",
      "Cake 3.jef",
    ]);

    // Untick Folder name: the folder-only match disappears, the input survives.
    await setSearchIn(page, { file: true, folder: false, tags: true });
    await expect(browseCard(page, "Bean Z.jef")).toHaveCount(0);
    await expect(page.locator("#browse-q")).toHaveValue("Cross");
  });
});

test.describe("card navigation and scroll restore", () => {
  test("opens Design Detail from any card", async ({ page }) => {
    await openBrowse(page);
    await browseCards(page).first().locator(".browse-card-link").click();
    await expect(page).toHaveURL(/#\/designs\/\d+/);
  });

  test("restores the previous scroll position on return", async ({ page }) => {
    await openBrowse(page);

    // Scroll down so window.scrollY is non-zero and captured into browseSessionStore
    await page.evaluate(() => window.scrollTo(0, 300));
    await page.waitForTimeout(200);

    const lastCard = browseCards(page).last();
    await lastCard.locator(".browse-card-link").click();
    await expect(page).toHaveURL(/#\/designs\/\d+/);

    // Return to Browse via Back to Browse button
    await page.getByRole("button", { name: /Back to Browse/i }).click();
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
    await expect
      .poll(() => page.evaluate(() => window.scrollY), { timeout: 15_000 })
      .toBeGreaterThan(0);
  });
});

test.describe("needs attention combinations", () => {
  test("combines with Unverified only", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);

    await needsAttention(page).check();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);

    // Both flagged designs are unverified, so the AND intersection is unchanged.
    await unverifiedOnly(page).check();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
  });

  test("sorts and paginates only the flagged set", async ({ page }) => {
    await openBrowse(page);
    await openFilters(page);
    await needsAttention(page).check();
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);

    // Only two flagged designs, so no pagination controls are offered.
    await expect(paginationNav(page)).toHaveCount(0);

    await sortSelect(page).selectOption("rating");
    await expectTitles(page, ["ZZ-broken-2.pes", "ZZ-broken.pes"]);
  });
});

// ---------------------------------------------------------------------------
// Mutating tests. These change the shared catalogue (tags / verification), so
// they are declared last and each targets a different design.
// ---------------------------------------------------------------------------

test.describe("batch mutations", () => {
  test("the Choose tags modal applies tags to the selection", async ({
    page,
  }) => {
    await openBrowse(page);
    await search(page, "Bean X.jef");
    await expectTitles(page, ["Bean X.jef"]);

    const card = browseCard(page, "Bean X.jef");
    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Choose tags" }).click();

    const dialog = page.getByRole("dialog");
    await expect(
      dialog.getByRole("heading", { name: "Choose tags for selected designs" }),
    ).toBeVisible();
    await expect(dialog.getByText("1 design selected.")).toBeVisible();
    await expect(
      dialog.getByRole("checkbox", { name: /Untagged/ }),
    ).toBeVisible();

    // Cancel must not commit anything.
    await dialog.getByRole("checkbox", { name: /Food/ }).click();
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toBeHidden();
    await expect(card.locator(".browse-card-tags")).toContainText("No tags");

    // Apply commits the tag and refreshes the card badge.
    await page.getByRole("button", { name: "Choose tags" }).click();
    await dialog.getByRole("checkbox", { name: /Food/ }).click();
    await dialog.getByRole("button", { name: "Apply tags" }).click();
    await expect(dialog).toBeHidden();
    await expect(card.locator(".browse-card-tags")).toContainText("Food");
  });

  test("Verify tags marks the selection verified", async ({ page }) => {
    await openBrowse(page);
    await search(page, "to be verified");
    await expectTitles(page, ["Cake 3 - to be verified.jef"]);

    const card = browseCard(page, "Cake 3 - to be verified.jef");
    await expect(card.locator('[aria-label="Unverified"]')).toBeVisible();
    await expect(card.locator('[aria-label="Verified"]')).toHaveCount(0);

    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Verify tags" }).click();

    await expect(
      browseCard(page, "Cake 3 - to be verified.jef").locator(
        '[aria-label="Verified"]',
      ),
    ).toBeVisible({ timeout: 20_000 });
    await expect(
      browseCard(page, "Cake 3 - to be verified.jef").locator(
        '[aria-label="Unverified"]',
      ),
    ).toHaveCount(0);
  });
});
