// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test as base, expect, type Page } from "@playwright/test";
import { startCoverage, stopCoverage } from "./coverage-helper";
import { DATA_ROOT_PATH } from "./paths";
import { launchDebugApp } from "./app-launcher";

/**
 * Fixtures for specs that need their own catalogue state.
 *
 * The shared `test` in `fixtures.ts` is worker-scoped (matching Playwright's
 * built-in `browser` fixture) and always points the app at the shared
 * `DATA_ROOT_PATH`; it cannot honour a per-file `dataRoot` option. This fixture
 * launches a *separate* app instance per test on its own CDP port/profile, so a
 * spec can select its data root with `test.use({ dataRoot: … })`.
 *
 * Use only where an isolated catalogue is required (e.g. the first-import hoop
 * gate, which needs an empty catalogue) — the extra app launch is not free.
 */
export const test = base.extend<{
  /** Data root the app under test is pointed at. Override with `test.use`. */
  dataRoot: string;
  page: Page;
}>({
  dataRoot: [DATA_ROOT_PATH, { option: true }],

  page: async ({ playwright, dataRoot }, use, testInfo) => {
    const app = await launchDebugApp({
      playwright,
      dataRoot,
      workerIndex: testInfo.workerIndex,
      // Keep this app's CDP port and WebView2 profile away from the shared
      // worker app so both can run within the same worker.
      portOffset: 100,
    });

    await startCoverage(app.page);

    try {
      await use(app.page);
    } finally {
      await stopCoverage(app.page);
      await app.dispose();
    }
  },
});

export { expect };
