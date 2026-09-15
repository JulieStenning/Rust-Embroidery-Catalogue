import { test, expect } from "./fixtures";
import { test as isolatedTest } from "./app-fixture";
import { gotoRoute, mainMenu } from "./helpers";
import { EMPTY_DATA_ROOT_PATH } from "./paths";
import { cleanupDataRoot, prepareEmptyDataRoot } from "./empty-root";
import type { Locator, Page } from "@playwright/test";

/**
 * Projects end-to-end coverage.
 *
 * These tests drive the real Tauri app against the seeded test catalogue
 * (`tests/Test Assets/EmbroideryCatalogue.db`, copied to the throwaway data root
 * by `global-setup.ts`). That catalogue deliberately contains **no projects**
 * (see the fixture contract in the e2e testing guide), so this spec creates the
 * projects it needs through the UI and builds its state up in declaration order
 * (`test.describe.serial`).
 *
 * The design-level fixture metadata used by the print-sheet assertions comes
 * from the same contract: `Cake 3.jef` has full metadata (designer `Me`,
 * `Hoop B`, preview), while `ZZ-broken.pes` has none (no dimensions, no hoop,
 * no designer, no preview) — the positive case for "skip empty fields".
 *
 * The empty-state case lives in its own describe against a throwaway empty root
 * so it stays deterministic no matter what the shared catalogue contains.
 */

// A per-run stamp keeps the generated names unique: project names are unique
// (case-insensitively) in the database, so a fixed name would fail on a retry.
const RUN_STAMP = Date.now();

const EMPTY_PROJECT_NAME = `Playwright Empty ${RUN_STAMP}`;
const EMPTY_PROJECT_RENAMED = `Playwright Empty Renamed ${RUN_STAMP}`;
const EMPTY_PROJECT_DESCRIPTION = "Planning notes for the empty project.";
const EMPTY_PROJECT_DESCRIPTION_RENAMED =
  "Planning notes, revised for the empty project.";

const DESIGN_PROJECT = `Playwright Design ${RUN_STAMP}`;
const DESIGN_PROJECT_DESCRIPTION = "Planning notes for the populated project.";

const RICH_DESIGN = "Cake 3.jef";
const SPARSE_DESIGN = "ZZ-broken.pes";

const NEW_NAME_PLACEHOLDER = "e.g. Christmas Stockings 2024";
const NEW_DESCRIPTION_PLACEHOLDER = "Optional notes, goals, or deadline";


// ---------------------------------------------------------------------------
// Locators and helpers
// ---------------------------------------------------------------------------

function projectTiles(page: Page): Locator {
  return page.locator("a.projects-tile");
}

/** The project tile whose title is exactly `name`. */
function projectTile(page: Page, name: string): Locator {
  return projectTiles(page).filter({
    has: page.locator("h2.projects-tile-title", { hasText: name }),
  });
}

/** A transient toast carrying `message` (see `ToastContainer.svelte`). */
function toast(page: Page, message: string): Locator {
  return page.locator(".toast-message", { hasText: message }).first();
}

/** The Design Detail "Projects" card. */
function designProjectsCard(page: Page): Locator {
  return page
    .locator(".route-card")
    .filter({ has: page.getByRole("heading", { name: "Projects", exact: true }) });
}

/** The project-detail design card for `filename`. */
function projectDesignCard(page: Page, filename: string): Locator {
  return page.locator(".projects-design-card").filter({
    has: page.locator("a.projects-design-title-link", { hasText: filename }),
  });
}

/** The print-sheet card for `filename`. */
function printCard(page: Page, filename: string): Locator {
  return page
    .locator(".projects-print-card")
    .filter({ has: page.locator("h3", { hasText: filename }) });
}

/** Open a project's detail route and wait for its inline form to load. */
async function openProjectDetail(page: Page, projectId: number): Promise<void> {
  await gotoRoute(page, `#/projects/${projectId}`);
  await expect(page.locator(".projects-title-input")).toBeVisible({
    timeout: 30_000,
  });
}

/**
 * Create a project through the New Project form, then read the new id back from
 * the card grid link. Leaves the browser on `#/projects`.
 */
async function createProject(
  page: Page,
  name: string,
  description: string,
): Promise<number> {
  await gotoRoute(page, "#/projects/new");
  await expect(
    page.getByRole("heading", { name: "New Project", exact: true }),
  ).toBeVisible();

  await page.getByPlaceholder(NEW_NAME_PLACEHOLDER).fill(name);
  if (description) {
    await page.getByPlaceholder(NEW_DESCRIPTION_PLACEHOLDER).fill(description);
  }
  await page.getByRole("button", { name: "Create Project" }).click();

  await expect(toast(page, "Project created.")).toBeVisible();

  const tile = projectTile(page, name);
  await expect(tile).toBeVisible({ timeout: 30_000 });
  const href = await tile.getAttribute("href");
  const match = href?.match(/\/projects\/(\d+)$/);
  if (!match) {
    throw new Error(`Could not determine project id from tile href: ${href}`);
  }
  return Number(match[1]);
}

/**
 * Look a design up by filename in Browse and return its database id. A reload
 * discards any Browse filter/session state left behind by earlier specs.
 */
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

/** Add a design to a project from the Design Detail "Projects" card. */
async function addDesignToProject(
  page: Page,
  designId: number,
  projectName: string,
): Promise<void> {
  await gotoRoute(page, `#/designs/${designId}`);

  const card = designProjectsCard(page);
  await expect(card).toBeVisible({ timeout: 30_000 });

  const select = card.locator("select");
  await expect(select).toBeVisible();
  await select.selectOption({ label: projectName });

  const addBtn = card.getByRole("button", { name: "Add", exact: true });
  await expect(addBtn).toBeEnabled({ timeout: 10_000 });
  await addBtn.click();
  await expect(toast(page, "Design added to project.")).toBeVisible({ timeout: 15_000 });
}


// ---------------------------------------------------------------------------
// Populated-catalogue workflows. Serial: the tests build project state up in
// order, and several later tests depend on projects created earlier.
// ---------------------------------------------------------------------------

test.describe.serial("projects", () => {
  let emptyProjectId = 0;
  let designProjectId = 0;
  let richDesignId = 0;
  let sparseDesignId = 0;
  let emptyProjectName = EMPTY_PROJECT_NAME;

  test("New Project form lay-out and the required-Name guard", async ({
    page,
  }) => {
    await gotoRoute(page, "#/projects/new");
    await expect(
      page.getByRole("heading", { name: "New Project", exact: true }),
    ).toBeVisible();

    // Placeholders and the mandatory-field marker.
    const nameInput = page.getByPlaceholder(NEW_NAME_PLACEHOLDER);
    const descriptionInput = page.getByPlaceholder(NEW_DESCRIPTION_PLACEHOLDER);
    await expect(nameInput).toBeVisible();
    await expect(descriptionInput).toBeVisible();
    await expect(page.getByText("Name *", { exact: true })).toBeVisible();
    expect(
      await nameInput.evaluate((el: HTMLInputElement) => el.required),
    ).toBe(true);

    // An empty Name keeps Create Project disabled...
    const create = page.getByRole("button", { name: "Create Project" });
    await expect(create).toBeDisabled();

    // ...and a valid Name enables it.
    await nameInput.fill(EMPTY_PROJECT_NAME);
    await descriptionInput.fill(EMPTY_PROJECT_DESCRIPTION);
    await expect(create).toBeEnabled();

    await create.click();
    await expect(toast(page, "Project created.")).toBeVisible();

    // Creation returns to the list and appends a fresh card.
    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible();
    const tile = projectTile(page, EMPTY_PROJECT_NAME);
    await expect(tile).toBeVisible({ timeout: 30_000 });

    const href = await tile.getAttribute("href");
    const match = href?.match(/\/projects\/(\d+)$/);
    if (!match) {
      throw new Error(`Could not determine project id from tile href: ${href}`);
    }
    emptyProjectId = Number(match[1]);
  });

  test("Project List view highlights the tab and renders project cards", async ({
    page,
  }) => {
    await gotoRoute(page, "#/projects");
    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible();

    // The top navigation highlights the active Projects tab.
    await expect(
      mainMenu(page).getByRole("link", { name: "Projects", exact: true }),
    ).toHaveClass(/menu-link-active/);

    // Sub-header copy plus the "Learn more" help link.
    await expect(
      page.getByText("Group designs for a planned embroidery task"),
    ).toBeVisible();
    const learnMore = page.locator('a[href="#/help?section=projects"]').first();
    await expect(learnMore).toBeVisible();

    // The card shows the name, description, created date and design-count badge.
    const tile = projectTile(page, EMPTY_PROJECT_NAME);
    await expect(tile).toBeVisible();
    await expect(tile.locator("h2.projects-tile-title")).toHaveText(
      EMPTY_PROJECT_NAME,
    );
    await expect(tile.locator(".projects-tile-description")).toHaveText(
      EMPTY_PROJECT_DESCRIPTION,
    );
    await expect(tile.locator(".projects-count-badge")).toHaveText("0 designs");
    await expect(tile.locator(".projects-tile-meta")).toHaveText(
      /^Created \d{4}-\d{2}-\d{2}$/,
    );
  });


  test("Project Detail view renders the empty project", async ({ page }) => {
    await openProjectDetail(page, emptyProjectId);

    // Breadcrumb back to the dashboard.
    await expect(page.locator("button.projects-back-link")).toHaveText(
      "← Projects",
    );

    // Project title + inline description, both editable.
    await expect(page.locator(".projects-title-input")).toHaveValue(
      EMPTY_PROJECT_NAME,
    );
    await expect(page.locator(".projects-textarea")).toHaveValue(
      EMPTY_PROJECT_DESCRIPTION,
    );

    // Action buttons top-right.
    await expect(
      page.getByRole("button", { name: "Print Sheet" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Delete Project" }),
    ).toBeVisible();

    // Designs section reflects the (zero) linked count and its empty state.
    await expect(
      page.getByRole("heading", { name: "Designs (0)" }),
    ).toBeVisible();
    await expect(
      page.getByText("No designs in this project yet."),
    ).toBeVisible();
  });

  test("edits the name and description with Save and Undo", async ({ page }) => {
    await openProjectDetail(page, emptyProjectId);

    const nameInput = page.locator(".projects-title-input");
    const descriptionInput = page.locator(".projects-textarea");
    const save = page.getByRole("button", { name: "Save", exact: true });
    const undo = page.getByRole("button", { name: "Undo", exact: true });

    // Untouched: both actions are disabled.
    await expect(save).toBeDisabled();
    await expect(undo).toBeDisabled();

    // An edit enables them; Undo restores the stored values.
    await nameInput.fill("Transient rename");
    await expect(save).toBeEnabled();
    await expect(undo).toBeEnabled();
    await undo.click();
    await expect(nameInput).toHaveValue(EMPTY_PROJECT_NAME);
    await expect(save).toBeDisabled();
    await expect(undo).toBeDisabled();

    // A committed edit persists through the backend (survives a reload).
    await nameInput.fill(EMPTY_PROJECT_RENAMED);
    await descriptionInput.fill(EMPTY_PROJECT_DESCRIPTION_RENAMED);
    await expect(save).toBeEnabled();
    await save.click();
    await expect(toast(page, "Project updated.")).toBeVisible();

    await page.reload();
    await expect(page.locator(".projects-title-input")).toHaveValue(
      EMPTY_PROJECT_RENAMED,
    );
    await expect(page.locator(".projects-textarea")).toHaveValue(
      EMPTY_PROJECT_DESCRIPTION_RENAMED,
    );

    // Later tests read the renamed title.
    emptyProjectName = EMPTY_PROJECT_RENAMED;
  });

  test("Print Sheet renders an empty project and dispatches printing", async ({
    page,
  }) => {
    await gotoRoute(page, `#/projects/${emptyProjectId}/print`);

    // Action bar: Back to Project + Print.
    await expect(
      page.getByRole("button", { name: "Back to Project" }),
    ).toBeVisible();
    const print = page.getByRole("button", { name: "Print", exact: true });
    await expect(print).toBeVisible();

    // Print-optimised layout with the project name and description.
    await expect(page.locator(".projects-print-shell")).toBeVisible();
    await expect(
      page.getByRole("heading", { name: emptyProjectName }),
    ).toBeVisible();
    await expect(
      page.getByText(EMPTY_PROJECT_DESCRIPTION_RENAMED),
    ).toBeVisible();
    await expect(
      page.getByText("No designs in this project yet."),
    ).toBeVisible();

    // The Print trigger must call window.print(). Stub it first so the native
    // OS print dialog never opens while the test is running.
    await page.evaluate(() => {
      const target = window as unknown as {
        __printCalls: number;
        print: () => void;
      };
      target.__printCalls = 0;
      target.print = () => {
        target.__printCalls += 1;
      };
    });
    await print.click();
    await expect
      .poll(() =>
        page.evaluate(
          () => (window as unknown as { __printCalls: number }).__printCalls,
        ),
      )
      .toBe(1);
  });


  test("creates a project and links designs from Design Detail", async ({
    page,
  }) => {
    designProjectId = await createProject(
      page,
      DESIGN_PROJECT,
      DESIGN_PROJECT_DESCRIPTION,
    );

    richDesignId = await findDesignId(page, RICH_DESIGN);
    sparseDesignId = await findDesignId(page, SPARSE_DESIGN);

    await addDesignToProject(page, richDesignId, DESIGN_PROJECT);
    await addDesignToProject(page, sparseDesignId, DESIGN_PROJECT);

    // The list badge counts both linked designs.
    await gotoRoute(page, "#/projects");
    await expect(
      projectTile(page, DESIGN_PROJECT).locator(".projects-count-badge"),
    ).toHaveText("2 designs");

    // The detail grid lists a card per design, with thumbnail, filename,
    // designer metadata where available, and a Remove control.
    await openProjectDetail(page, designProjectId);
    await expect(
      page.getByRole("heading", { name: "Designs (2)" }),
    ).toBeVisible();

    const richCard = projectDesignCard(page, RICH_DESIGN);
    await expect(richCard).toBeVisible();
    await expect(richCard.locator(".projects-design-image")).toBeVisible();
    await expect(
      richCard.getByRole("button", { name: "Remove" }),
    ).toBeVisible();
    await expect(richCard).toContainText("Me");

    const sparseCard = projectDesignCard(page, SPARSE_DESIGN);
    await expect(sparseCard).toBeVisible();
    await expect(
      sparseCard.getByRole("button", { name: "Remove" }),
    ).toBeVisible();
  });

  test("design cards open the matching Design Detail view", async ({
    page,
  }) => {
    await openProjectDetail(page, designProjectId);
    await projectDesignCard(page, RICH_DESIGN)
      .locator("a.projects-design-link")
      .click();

    await expect(page).toHaveURL(new RegExp(`#/designs/${richDesignId}$`));
    await expect(
      page.locator(".route-card p.font-medium", { hasText: RICH_DESIGN }),
    ).toBeVisible({ timeout: 30_000 });
  });

  test("Print Sheet renders per-design specs and skips empty fields", async ({
    page,
  }) => {
    await gotoRoute(page, `#/projects/${designProjectId}/print`);
    await expect(
      page.getByRole("heading", { name: DESIGN_PROJECT }),
    ).toBeVisible();
    await expect(page.getByText(DESIGN_PROJECT_DESCRIPTION)).toBeVisible();

    // Full metadata: every labelled row renders, with the preview image.
    const richCard = printCard(page, RICH_DESIGN);
    await expect(richCard).toBeVisible();
    await expect(richCard.locator("img.projects-print-image")).toBeVisible();
    await expect(richCard).toContainText("Size:");
    await expect(richCard).toContainText("Hoop:");
    await expect(richCard).toContainText("Stitches:");
    await expect(richCard).toContainText("Designer:");
    await expect(richCard).toContainText("Rating:");
    await expect(richCard).toContainText("Stitched: Yes");

    // Sparse metadata (no dimensions / hoop / designer / preview): the labels
    // are skipped entirely and the image falls back to "No image".
    const sparseCard = printCard(page, SPARSE_DESIGN);
    await expect(sparseCard).toBeVisible();
    await expect(sparseCard).toContainText("No image");
    await expect(sparseCard).not.toContainText("Size:");
    await expect(sparseCard).not.toContainText("Hoop:");
    await expect(sparseCard).not.toContainText("Designer:");

    // No raw null/undefined values leak into the printed output.
    const sheet = page.locator(".projects-print-shell");
    await expect(sheet).not.toContainText("null");
    await expect(sheet).not.toContainText("undefined");
  });


  test("Remove unlinks a design and updates the section count", async ({
    page,
  }) => {
    await openProjectDetail(page, designProjectId);
    await expect(
      page.getByRole("heading", { name: "Designs (2)" }),
    ).toBeVisible();

    await projectDesignCard(page, SPARSE_DESIGN)
      .getByRole("button", { name: "Remove" })
      .click();
    await expect(toast(page, "Design removed from project.")).toBeVisible();

    // The targeted card leaves the grid and the count decrements.
    await expect(
      page.getByRole("heading", { name: "Designs (1)" }),
    ).toBeVisible();
    await expect(projectDesignCard(page, SPARSE_DESIGN)).toHaveCount(0);
    await expect(projectDesignCard(page, RICH_DESIGN)).toBeVisible();
  });

  test("Delete Project confirms first, then preserves the design record", async ({
    page,
  }) => {
    await openProjectDetail(page, designProjectId);

    // Dismissing the confirmation modal leaves the project untouched.
    await page.getByRole("button", { name: "Delete Project" }).click();
    const modal = page.getByRole("dialog");
    await expect(modal).toBeVisible();
    await modal.getByRole("button", { name: "Cancel" }).click();
    await expect(modal).not.toBeVisible();
    await expect(page).toHaveURL(
      new RegExp(`#/projects/${designProjectId}$`),
    );

    // Confirming deletes the project (the design records are untouched).
    await page.getByRole("button", { name: "Delete Project" }).click();
    await expect(modal).toBeVisible();
    await modal.getByRole("button", { name: "Delete project" }).click();

    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible({ timeout: 30_000 });
    await expect(projectTile(page, DESIGN_PROJECT)).toHaveCount(0);

    // The design itself survives and is simply no longer assigned.
    await gotoRoute(page, `#/designs/${richDesignId}`);
    await expect(
      page.locator(".route-card p.font-medium", { hasText: RICH_DESIGN }),
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      designProjectsCard(page).getByText("Not assigned to any projects."),
    ).toBeVisible();
  });

  test("Back links return to the list and to the project", async ({ page }) => {
    // New Project -> Projects.
    await gotoRoute(page, "#/projects/new");
    await expect(
      page.getByRole("heading", { name: "New Project", exact: true }),
    ).toBeVisible();
    await page.locator("button.projects-back-link").click();
    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible();

    // Project Detail -> Projects.
    await openProjectDetail(page, emptyProjectId);
    await page.locator("button.projects-back-link").click();
    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible();
    await expect(projectTile(page, emptyProjectName)).toBeVisible();

    // Print Sheet -> Project Detail.
    await gotoRoute(page, `#/projects/${emptyProjectId}/print`);
    await expect(
      page.getByRole("heading", { name: emptyProjectName }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Back to Project" }).click();
    await expect(page).toHaveURL(
      new RegExp(`#/projects/${emptyProjectId}$`),
    );
  });
});


// ---------------------------------------------------------------------------
// Empty-catalogue state. Runs against its own throwaway data root (the pristine
// install-template database) so the assertion is independent of the shared,
// populated catalogue — and of any projects created above.
// ---------------------------------------------------------------------------

isolatedTest.describe("projects empty state", () => {
  isolatedTest.use({ dataRoot: EMPTY_DATA_ROOT_PATH });

  isolatedTest.beforeAll(() => {
    prepareEmptyDataRoot(EMPTY_DATA_ROOT_PATH);
  });

  isolatedTest.afterAll(() => {
    cleanupDataRoot(EMPTY_DATA_ROOT_PATH);
  });

  isolatedTest("shows the empty state and the Create one shortcut", async ({
    page,
  }) => {
    await gotoRoute(page, "#/projects");
    await expect(
      page.getByRole("heading", { name: "Projects", exact: true }),
    ).toBeVisible();

    await expect(page.getByText("No projects yet.")).toBeVisible();
    await expect(projectTiles(page)).toHaveCount(0);

    await page.getByRole("button", { name: "Create one" }).click();
    await expect(page).toHaveURL(/#\/projects\/new$/);
    await expect(
      page.getByRole("heading", { name: "New Project", exact: true }),
    ).toBeVisible();
  });
});

