import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { gotoRoute, mainMenu } from "./helpers";

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
    await expect(card.locator('[aria-label="Verified"]')).toHaveCount(0);

    await page.locator(".browse-design-checkbox").first().check();
    await page.getByRole("button", { name: "Verify tags" }).click();

    await expect(
      browseCard(page, "Cake 3 - to be verified.jef").locator(
        '[aria-label="Verified"]',
      ),
    ).toBeVisible({ timeout: 20_000 });
  });
});
