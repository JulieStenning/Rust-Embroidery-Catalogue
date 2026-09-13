# End-to-end tests (Playwright -> Tauri WebView2)

> Developer guide for the Playwright harness in `tests/e2e/`. Run every command
> below from the repository root.

These tests drive the **real Tauri desktop app** - real Rust backend, real
SQLite database, real design files. Playwright cannot launch a Tauri binary as
a browser, so the harness spawns the built debug executable with WebView2
remote debugging enabled and attaches over the Chrome DevTools Protocol.

## Prerequisites

- Windows (WebView2 is required).
- `npm install` has been run at the repository root.

## Run the tests

From the **repository root**:

```powershell
npm run e2e:build   # one-off, and again after any Rust or frontend change
npm run e2e         # run the suite
```

`e2e:build` is required. A plain `cargo build` debug binary loads the Tauri
dev URL (the Vite dev server at http://localhost:5173) and shows a
can-not-reach-this-page document. `cargo tauri build --debug --no-bundle`
embeds `frontend/dist` so the app runs standalone.

Other commands:

```powershell
npm run e2e:ui       # interactive Playwright UI (best for authoring)
npm run e2e:debug    # step-through with the inspector
npx playwright test tests/e2e/navigation.spec.ts   # a single file
```

## How the first-run setup is handled

The app normally needs a data location (database + designs) and an onboarding
step on first run. The harness does all of this automatically - **you do not
run the setup wizard**.

1. `global-setup.ts` creates a throwaway data root at `tests/e2e/.data-root/`:
   - `Database/EmbroideryCatalogue.db` copied from
     `tests/Test Assets/EmbroideryCatalogue.db`
   - `MachineEmbroideryDesigns/` copied from `tests/Test Designs/`
   - it sets `initial_setup_completed = TRUE` (the exact value the backend
     expects) so the app boots straight into the main window
2. `fixtures.ts` launches the app with `EMBROIDERY_DATA_ROOT` pointing at that
   folder. This is a debug-build-only override (see `src/paths.rs`), so the app
   uses those locations instead of prompting for them.
3. `global-teardown.ts` deletes `.data-root` when the run finishes.

Your real `dev_data/` is never touched.

> `tests/Test Assets/EmbroideryCatalogue.db` **is tracked in git** and is the
> source of the Browse fixture contract below. It is a checked-in binary fixture:
> keep it in sync with `tests/Test Designs/` (every `filepath` must resolve to a
> real file) and verify with `node scripts/check-browse-fixtures.cjs`.

## Catalogue fixture contract (Browse tests)

`browse.spec.ts` asserts the *content* of the seeded catalogue, not just the UI,
so `tests/Test Assets/EmbroideryCatalogue.db` must be kept in sync with this
contract (and the matching files must exist in `tests/Test Designs`). The first
Browse test ("fixture contract") fails with a clear diff if the two drift.

Run `node scripts/check-browse-fixtures.cjs` after changing the seed database or
`tests/Test Designs/`. It checks every rule in the table below *and* that each
`filepath` resolves to a real file, and exits non-zero with a precise reason — a
fast pre-flight before `npx playwright test`.

Reference data: designers `Me`, `Wrenwood Studio`, `Thistlebury Stitch`, `Quillmark Designs`;
sources `Me`, `Heirloom Stash`, `Loomthread Embroidery Suite`, `Threadwise Guild`;
hoops `Hoop A` (126×110), `Hoop B` (200×140), `Giga Hoop` (230×200); no projects
(the project pickers therefore render their empty state). The 81 system tags already exist.

| filepath | designer | hoop (derived) | rating | stitched | image/stitch verified | tags | preview |
|---|---|---|---|---|---|---|---|
| `Cake 3.jef` | Me | **Hoop B** (asserted) | 4 | yes | yes / yes | `Food`, `Cross Stitch` | yes |
| `Cake 3 Cross Stitch Fred.jef` | — | Hoop B (same file) | — | no | yes / **no** | — | yes |
| `Cake 3 - Food.jef` | — | Hoop B (same file) | — | no | yes / yes | — | yes |
| `Cake 3 - to be verified.jef` | — | Hoop B (same file) | — | no | **no / no** | — | yes |
| `Cake Applique.jef` | Me | Hoop B (same file) | 4 | yes | yes / yes | `Flowers`, `Filled` | yes |
| `Cake Applique 2.jef` | Wrenwood Studio | derived | 2 | no | yes / yes | `Footwear`, `Applique` | yes |
| `Bean X.jef` | Thistlebury Stitch | derived | — | no | yes / yes | — | yes |
| `Cross/Bean Z.jef` | Thistlebury Stitch | derived (same file as `Bean X.jef`) | — | no | yes / yes | — | yes |
| `ZZ-broken.pes` | — | NULL (no dimensions) | — | no | no / no | — | **NULL** |
| `ZZ-broken-2.pes` | — | NULL (no dimensions) | — | no | no / no | — | **NULL** |
| ≥51 designs total | — | — | — | — | — | — | yes |

The two `ZZ-broken*.pes` files are plain text saved with a `.pes` extension so
decoding fails and the row is created with `image_data IS NULL` — the "Needs
attention" flag. The ≥51 total is what makes pagination reachable (the grid's
page size is `columns × 10`, i.e. 50 at the default 1280px window width).

The **hoop** column is informative, not enforced. `hoop_id` is derived by the
importer from each design's parsed dimensions (the smallest hoop that fits — see
`src/routes/bulk_import.rs`), so tweaking the fixture files changes it. The Browse
hoop test therefore asserts *membership* (the expected designs appear, and
designs with no hoop are excluded) rather than exact sets. If you later run
Batch Operations → *Recalculate Hoop Dimensions* against the fixture DB, hoop
assignments will be recomputed.

### Per-spec data roots

`tests/e2e/fixtures.ts` exposes the shared `test`, whose `browser` fixture is
**worker-scoped** (matching Playwright's built-in) and always points the app at
`DATA_ROOT_PATH` (`.data-root`).

Specs that need a different catalogue use `tests/e2e/app-fixture.ts`, whose
`page` fixture is **test-scoped** and launches its own app instance (separate CDP
port/profile) for the data root selected with `test.use({ dataRoot })`. This is
how `import-hoop-setup.spec.ts` runs against an empty catalogue: the first-import
hoop gate only exists when `design_count == 0 && hoop_count == 0`.

## What you will see

A desktop window (the real app) briefly opens and closes for each test. No
browser window opens. Video is not supported over CDP; a screenshot is saved
under `tests/e2e/test-results/` when a test fails.

## Recording and exploring (authoring new tests)

Playwright's built-in recorder (the Record button in UI mode, or
`npx playwright codegen`) launches its **own browser**, so it cannot attach to
the Tauri app and will not record your application. Use these instead:

- **UI mode** (`npm run e2e:ui`): run the suite, click a step to inspect its DOM
  snapshot and the locator it used, and use the **Pick locator** tool to
  discover selectors in the live app.
- **Explore workbench** (`npm run e2e:explore`): opens the app and pauses,
  leaving the Playwright Inspector attached so you can hover elements, copy
  locators and step through the UI. This spec is excluded from `npm run e2e`.

Then write the test by hand from the locators you discovered, and run
`npm run e2e`.

## Authoring a new test (worked example)

The loop is: discover a locator, write the test, run it in UI mode, iterate.

A complete worked example - this is a real, passing test in
`reference-data.spec.ts`:

```ts
import { test, expect } from "./fixtures";
import { gotoRoute } from "./helpers";

test("adds an image tag and it persists across a reload", async ({ page }) => {
  // Arrange: go straight to the view under test.
  await gotoRoute(page, "#/admin/data/tags");
  await expect(page.getByRole("heading", { name: "Manage Tags" })).toBeVisible();

  // Act: fill the form and submit.
  await page.locator("#admin-tag-description").fill("Playwright Tag");
  await page.getByRole("button", { name: "Add", exact: true }).click();

  // Assert it appears...
  await expect(page.getByRole("cell", { name: "Playwright Tag" })).toBeVisible();

  // ...and survived a reload, proving it reached the real database.
  await page.reload();
  await expect(page.getByRole("cell", { name: "Playwright Tag" })).toBeVisible();
});
```

Locator cheat-sheet for this app:

- ids: `#settings-ai-batch-size`, `#admin-tag-description`, `#admin-tag-group`
- test ids: `data-testid="reference-data-tab-tags"`, `settings-dirty-hint`
- roles: `getByRole("heading", { name: "Manage Tags" })`, links, buttons
- nav is scoped for you in `clickNav(page, "Browse")` (see `helpers.ts`)

## Writing tests

Import the harness fixtures, not `@playwright/test`:

```ts
import { test, expect } from './fixtures';
import { clickNav, gotoRoute, expectMainView } from './helpers';
```

See `navigation.spec.ts`, `settings.spec.ts`, `reference-data.spec.ts`,
`import.spec.ts` and `import-folder-selection.spec.ts` for worked examples.

For a spec that needs its own catalogue state, import `test`/`expect` from
`./app-fixture` instead and select the root:

```ts
import { test, expect } from "./app-fixture";
import { EMPTY_DATA_ROOT_PATH } from "./paths";

test.use({ dataRoot: EMPTY_DATA_ROOT_PATH });
```

See `import-hoop-setup.spec.ts` (the first-import hoop gate) for a worked example.

## What can (and cannot) be automated

- **Drivable:** anything reached through the in-app UI - including flows that
  normally start with a native dialog, provided the page also accepts a typed
  value. Bulk Import is like this: it takes a typed folder path, so the whole
  wizard (scan -> review -> import) can be driven without the file picker.
- **Not drivable:** flows whose *only* entry point is a native OS dialog
  (Backup / Restore browse, Settings -> data-root browse, Orphans browse).
  Playwright cannot interact with OS dialogs; those need the picker command
  stubbed at the IPC layer.
- `import.spec.ts` is slow (scan + copy + DB writes, ~30-40s) and confirms a
  one-time "skip hoop setup" prompt, which the test clicks. It also asserts the
  step 2 review contract (summary counts, global override defaults, per-folder shell,
  select-all/deselect-all) and the step 3 "Before You Import" panel, including
  **negative** assertions for the AI-tagging banner, the Tier 2/3 counters, "Change in
  Settings" and the 2D/3D preview picker that were removed from step 3.
- `import-folder-selection.spec.ts` covers step 1 only. It is read-only against the
  catalogue (the scans target throwaway folders under `tests/e2e/`), so it is cheap
  enough to run on its own:
  `npx playwright test tests/e2e/import-folder-selection.spec.ts`.

## Shared wizard state between tests

The app instance is long-lived (one per worker) and the import wizard mirrors its state
into a module-level store, so a spec that asserts on step 1's default layout must reset
it first. Helpers live in `tests/e2e/helpers.ts`:

- `resetImportView(page)` — navigate to `#/import` and click **Reset** (falling back to
  clearing the input when Reset is disabled because no path is set).
- `runImportToPrecheck(page, folder)` — drive folder -> scan -> review -> step 3.
- `folderRows(page)` — the step 1 folder rows (one per source folder).

## Troubleshooting

- Debug executable not found -> run `npm run e2e:build`.
- App shows the setup wizard -> the copied DB is missing
  `initial_setup_completed = TRUE`; check `global-setup.ts`.
- Title is `localhost` / can-not-reach-this-page -> the exe was built with
  plain `cargo build`; rebuild with `npm run e2e:build`.
- Changes not reflected -> tests run against the built app, so re-run
  `npm run e2e:build` after any Rust or frontend change.