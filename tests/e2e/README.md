# End-to-end tests (Playwright -> Tauri WebView2)

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

> `tests/Test Assets/EmbroideryCatalogue.db` is currently untracked in git.
> The harness depends on it, so it must be present (and ideally committed).

## What you will see

A desktop window (the real app) briefly opens and closes for each test. No
browser window opens. Video is not supported over CDP; a screenshot is saved
under `tests/e2e/test-results/` when a test fails.

## Writing tests

Import the harness fixtures, not `@playwright/test`:

```ts
import { test, expect } from './fixtures';
import { clickNav, gotoRoute, expectMainView } from './helpers';
```

See `navigation.spec.ts`, `settings.spec.ts` and `reference-data.spec.ts` for
worked examples.

## Troubleshooting

- Debug executable not found -> run `npm run e2e:build`.
- App shows the setup wizard -> the copied DB is missing
  `initial_setup_completed = TRUE`; check `global-setup.ts`.
- Title is `localhost` / can-not-reach-this-page -> the exe was built with
  plain `cargo build`; rebuild with `npm run e2e:build`.
- Changes not reflected -> tests run against the built app, so re-run
  `npm run e2e:build` after any Rust or frontend change.