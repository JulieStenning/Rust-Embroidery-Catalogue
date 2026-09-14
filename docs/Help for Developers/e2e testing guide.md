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
hoops `Hoop A` (126×110), `Hoop B` (200×140), `Gigahoop` (230×200); no projects
(the project pickers therefore render their empty state). The 81 system tags already exist.

`projects.spec.ts` also leans on this contract: the seed has **no projects** (so that
spec creates its own through the UI), and its print-sheet test contrasts a
fully-populated design (`Cake 3.jef` — size/hoop/designer/rating) with one that has no
metadata at all (`ZZ-broken.pes`) to prove the "skip empty fields" behaviour.

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

See `navigation.spec.ts`, `help.spec.ts`, `settings.spec.ts`,
`reference-data.spec.ts`, `admin-designers.spec.ts`, `admin-tags.spec.ts`,
`admin-sources.spec.ts`, `admin-hoops.spec.ts`, `batch-tagging.spec.ts`, `batch-maintenance.spec.ts`,
`import.spec.ts`, `import-folder-selection.spec.ts` and `projects.spec.ts` for worked examples.

For a spec that needs its own catalogue state, import `test`/`expect` from
`./app-fixture` instead and select the root:

```ts
import { test, expect } from "./app-fixture";
import { EMPTY_DATA_ROOT_PATH } from "./paths";

test.use({ dataRoot: EMPTY_DATA_ROOT_PATH });
```

See `import-hoop-setup.spec.ts` (the first-import hoop gate) for a worked example.

### JavaScript dialogs are not DOM modals

A native `window.confirm` / `window.alert` / `window.prompt` is a JavaScript dialog
owned by the webview; it is **not** a DOM modal, so `getByRole("dialog")` will not find
it. Drive it through Playwright's dialog event instead:

```ts
page.once("dialog", (dialog) => dialog.accept()); // or dialog.dismiss()
await page.getByRole("button", { name: "Delete Project" }).click();
```

Assert what the dialog says from inside the handler (`dialog.message()`). If the app
later replaces it with a styled in-app modal, switch to `getByRole("dialog")` and click
the modal's own buttons.

### In-row inline editing vs static cells

Admin tables (such as Manage Designers and Manage Tags) support in-place inline
editing. When a row enters edit mode, the static cell text is replaced by an
`<input class="admin-input">` element.

If your locator filters by cell text (e.g. `page.locator("tr", { has: page.getByRole("cell", { name: originalName }) })`),
it will **no longer match** while the row is being edited. Scope the active row
via its input instead:

```ts
// Enter edit mode
await row.getByRole("button", { name: "Edit" }).click();

// Locate the editing row and its controls
const editingRow = page.locator("tr", { has: page.locator("input.admin-input") });
const editInput = editingRow.locator("input.admin-input");
await editInput.fill(updatedName);
await editingRow.getByRole("button", { name: "Save" }).click();
```

## What can (and cannot) be automated

- **Drivable:** anything reached through the in-app UI - including flows that
  normally start with a native dialog, provided the page also accepts a typed
  value. Bulk Import is like this: it takes a typed folder path, so the whole
  wizard (scan -> review -> import) can be driven without the file picker.
- **Not drivable:** flows whose *only* entry point is a native OS dialog
  (Backup / Restore browse, Settings -> data-root browse, Orphans browse).
  Playwright cannot interact with OS dialogs; those need the picker command
  stubbed at the IPC layer.
- `admin-designers.spec.ts` covers the full Designer management lifecycle in the
  Manage Data admin hub (`#/admin/data/designers`, reached via top nav *Manage Data*):
  - Alphabetical case-insensitive sorting and seed contract verification (`Me`,
    `Quillmark Designs`, `Thistlebury Stitch`, `Wrenwood Studio`).
  - Add form input validation states and the `Clear` button.
  - Adding new designers and verifying real SQLite persistence across `page.reload()`.
  - Backend duplicate rejection (case-insensitive collision returns `invalid input: Designer '...' already exists.`).
  - Inline editing with cancellation, empty-name validation (`Enter a designer name.`), and persistence.
  - Two-tier deletion flow: 0-design deletion confirmation prompt vs. assigned-designer
    (`design_count > 0`, e.g. seeded `Me`) warning banner and toast (`Deleting 'Me' will clear assignment from X design(s).`),
    proving design records are preserved.
  - Sub-tab switching between Designers, Tags, Sources, and Hoops.
- `admin-tags.spec.ts` covers Tag reference data CRUD operations (`#/admin/data/tags`).
  Tag deletion uses inline in-row DOM buttons (`Confirm delete` / `Cancel`) rather
  than native dialogs. The spec also verifies system tag lock protection (81 seeded
  system tags are immutable with `is_system = true`) and proves that deleting an
  assigned user tag dissociates it from designs without deleting the designs themselves.
- `admin-sources.spec.ts` covers the Source management lifecycle in the Manage Data
  admin hub (`#/admin/data/sources`, reached via top nav *Manage Data* -> *Sources* tab):
  - Alphabetical case-insensitive sorting and seed contract verification (`Heirloom Stash`,
    `Loomthread Embroidery Suite`, `Me`, `Threadwise Guild`).
  - Add form input validation states and the `Clear` button.
  - Adding new sources and verifying real SQLite persistence across `page.reload()`.
  - Backend duplicate rejection (case-insensitive collision returns `invalid input: Source '...' already exists.`).
  - Inline editing with cancellation, empty-name validation (`Enter a source name.`),
    duplicate collision rejection, and persistence across `page.reload()`.
  - Two-tier deletion flow: 0-design deletion confirmation prompt vs. assigned-source
    (`design_count > 0`, e.g. seeded `Threadwise Guild` / `Me`) warning banner and toast
    (`Deleting '...' will clear assignment from X design(s).`), proving design records are preserved.
  - Sub-tab switching between Designers, Tags, Sources, and Hoops.
- `admin-hoops.spec.ts` covers the Hoop management lifecycle in the Manage Data
  admin hub (`#/admin/data/hoops`, reached via top nav *Manage Data* -> *Hoops* tab):
  - Dimension-based sorting verification (`max_width_mm ASC, max_height_mm ASC`) and seed
    contract verification (`Hoop A` 126×110, `Hoop B` 200×140, `Gigahoop` 230×200).
  - Add form input validation states (name, width, height) and the `Clear` button.
  - Reserved hoop name rejection (`"__hoop_unknown__"` is reserved for system use).
  - Adding new hoops and verifying real SQLite persistence across `page.reload()`.
  - Backend duplicate rejection (case-insensitive collision returns `invalid input: Hoop '...' already exists.`).
  - Inline editing with cancellation, input detail validation (`Enter hoop details.`),
    reserved name rejection, duplicate collision rejection, and persistence across `page.reload()`.
  - Two-tier deletion flow: 0-design deletion confirmation prompt vs. assigned-hoop
    (`design_count > 0`, e.g. seeded `Hoop B`) warning banner and toast
    (`Deleting '...' will clear assignment from X design(s).`), proving design records are preserved.
  - Sub-tab switching between Designers, Tags, Sources, and Hoops.
- `batch-tagging.spec.ts` covers the **Tagging & Categorisation** workflow on the Batch
  Operations admin page (`#/admin/batch-operations`, reached via top nav *Batch Operations*):
  - Sub-tab switching between *Tagging & Categorisation* and *Maintenance & File Processing*.
  - Unconfigured Google API key detection banner (`"No Google API key is configured in Settings..."`)
  - and Settings direct link (`#/admin/system/settings`).
  - Disabling of Visual AI goals (`Enrich with visual AI`, `Full re-scan`) when no API key is set,
    while keeping offline File & Folder rules enabled by default.
  - 3-step workflow configuration: Goal selection, Scope selection with live candidate counters
    (`X designs`, `X unverified · Y verified`), "Exclude human-verified designs" toggle,
    and Merge strategy (`Add new tags only` vs `Complete reset`).
  - Advanced options drawer (stitching tags, image regeneration, colour/stitch counts, hoop recalculation).
  - Pre-flight confirmation modal (`Ready to Retag`) summary assertions and clean cancellation.
  - Full end-to-end execution of offline File & Folder rules against real SQLite catalogue designs
    (`Cake 3 - Food.jef`), verifying completion toasts, "Last run summary" metrics, Backfill log entries,
    and tag persistence across `page.reload()`.
  - Mock Google API key handling: testing UI enablement and dynamic AI scope attachment
    (`Designs missing Visual AI analysis`, `Visual AI found no match`, `Re-analyze`) by saving a dummy key
    in Settings and verifying time estimates in the confirmation modal without exposing real secrets
    in the repository.
- `batch-maintenance.spec.ts` covers the **Maintenance & File Processing** workflow on the Batch
  Operations admin page (`#/admin/batch-operations`, reached via top nav *Batch Operations* -> *Maintenance & File Processing* tab):
  - Sub-tab navigation and tab active state assertions (`aria-selected="true"`).
  - Target scope selection (Step 1): "Entire catalogue" (`scope = "all"`) vs "Designs missing preview images only" (`scope = "missing_previews"`), including live missing preview count badge population (seeded `ZZ-broken*.pes`).
  - Maintenance tasks form validation (Step 2): verifying "Review & Start Maintenance" is disabled when no tasks are checked, and dynamically enabled when checking "Generate preview images", "Recalculate colour / stitch counts", or "Recalculate hoops / dimensions".
  - Pre-flight confirmation modal (`Ready to Run Maintenance`) summary assertions (target scope and bulleted list of tasks) and cancellation.
  - Full end-to-end execution of maintenance passes against SQLite catalogue designs, verifying completion toasts (`Maintenance complete: X operations, 0 errors.`), "Last run summary" card metrics and task breakdown (`Tasks run: color_counts, hoop_dimensions`), and Backfill log entries.
  - Missing previews thumbnail pass & "Needs attention" handling: running preview generation on broken fixtures (`ZZ-broken*.pes`) and verifying the "Needs attention: X before → Y after" summary with the "Review these in Browse" deep link leading to Browse Designs with the filter active.
  - Unmatched design files reconciler (`UnmatchedFilesReconciler`): scanning clean directory states (`No unmatched design files found`), detecting newly placed design files on disk (`Unmatched files found` prompt), testing user dismissal, and executing batch unmatched import (`Import X file(s)`).
- `settings.spec.ts` covers the **Settings** tab on the **Application Settings** page (`#/admin/system/settings`, reached via top nav *System*):
  - Top navigation and deep linking (`#/admin/system` defaulting to `#/admin/system/settings`), plus sub-tab switching between *Settings*, *Backup & Restore*, and *Orphaned Files*.
  - Initial clean form state verification: ensuring "Save settings" starts disabled and the unsaved changes indicator (`settings-dirty-hint`) is hidden.
  - Granular dirty state tracking and reversion: modifying any editable field marks the form dirty and enables Save, while restoring original values re-cleans the form.
  - API key password masking/unmasking toggle (`aria-pressed`, `type="password"` vs `type="text"`).
  - Free-tier rate-limit toggle: dynamically adapting worker (2 vs 4) and delay (10s vs 0s) placeholder guidance.
  - Mock Gemini API key lifecycle: entering a mock key dynamically enables the Gemini model dropdown, Refresh button, and Test model action without requiring real secrets or incurring external API costs.
  - Full settings persistence across `page.reload()`: saving batch sizes, commit intervals, workers, delay, idle check intervals, and mock API keys, verifying SQLite round-trip preservation.
  - Database Maintenance & Diagnostics: inspecting storage usage badges (database size, recoverable space) and executing manual compaction (`Optimize & Compact Database`), asserting reclaimed page counts and updated stats.
  - System storage locations inspection and in-page documentation cross-links (Help and Batch Operations).
  - Clean test restoration: resetting configuration back to empty/default states at test completion.
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
- `help.spec.ts` covers the Help system (`#/help`), including in-page Table of
  Contents section jumps (`#/help?section=...`), contextual entry points from
  other views ("Search help" on Browse, "Import help" on Import, and "Learn more"
  on Projects), and outbound cross-links to app pages and About document guides.
  External reference links (e.g. Google AI Studio) are asserted with
  `toHaveAttribute("href", ...)` rather than clicked, avoiding navigating WebView2
  outside the desktop app.
- `projects.spec.ts` covers the Projects workflows (list / new / detail / print). The
  shared catalogue starts with **no projects**, so the spec creates them through the New
  Project form and links designs from Design Detail; it also proves that deleting a
  project preserves the design records. It declares its tests with
  `test.describe.serial` because later tests reuse projects created earlier, and its
  empty-state case runs against its own throwaway empty root (`app-fixture` +
  `prepareEmptyDataRoot`). The **Print** trigger is verified by stubbing `window.print()`
  so the native OS print dialog never opens.

## Shared wizard state between tests

The app instance is long-lived (one per worker) and the import wizard mirrors its state
into a module-level store, so a spec that asserts on step 1's default layout must reset
it first. Helpers live in `tests/e2e/helpers.ts`:

- `resetImportView(page)` — navigate to `#/import` and click **Reset** (falling back to
  clearing the input when Reset is disabled because no path is set).
- `runImportToPrecheck(page, folder)` — drive folder -> scan -> review -> step 3.
- `folderRows(page)` — the step 1 folder rows (one per source folder).

## Shared routing history between tests

The app instance is long-lived within each worker, so in-memory routing state
(such as `previousRoute` used by the shell's context-aware `← Back` button)
persists across test cases.

- When testing a cold-launch or deep-link scenario where the `← Back` button
  should be hidden (e.g. landing on `#/help` directly with no prior history),
  call `await gotoRoute(page, "#/help")` followed by `await page.reload()` to
  re-mount the frontend and reset in-memory route history.

## Shared catalogue state between tests

Every spec in a worker shares one long-lived app instance and therefore one SQLite
database, so a spec that creates rows leaves them behind for the specs that follow.
Specs are executed in file order for that reason.

- Reference data specs (`admin-designers.spec.ts`, `admin-tags.spec.ts`,
  `admin-sources.spec.ts`, `admin-hoops.spec.ts`) use timestamped entity names
  (e.g. `Playwright Source ${Date.now()}`, `Playwright Hoop ${Date.now()}`)
  for newly created or renamed items to avoid unique constraint collisions across test runs.
- `projects.spec.ts` is the main example: the seed catalogue has no projects, so it
  creates its own through the New Project form and declares its tests with
  `test.describe.serial` (later tests reuse projects created earlier). If a spec needs a
  known-empty or otherwise specific catalogue, give it its own data root via
  `app-fixture.ts` + `prepareEmptyDataRoot()` instead of assuming anything about the
  shared one.

## Troubleshooting

- Debug executable not found -> run `npm run e2e:build`.
- App shows the setup wizard -> the copied DB is missing
  `initial_setup_completed = TRUE`; check `global-setup.ts`.
- Title is `localhost` / can-not-reach-this-page -> the exe was built with
  plain `cargo build`; rebuild with `npm run e2e:build`.
- Changes not reflected -> tests run against the built app, so re-run
  `npm run e2e:build` after any Rust or frontend change.
- **`e2e:build` fails with `Access is denied. (os error 5)`** while removing
  `target/debug/embroidery-catalogue.exe` -> a previously launched app instance is
  still running and holds the binary. Close the app window (or
  `Stop-Process -Name embroidery-catalogue -Force`) and rebuild.
