import {
  test as base,
  expect,
  type Browser,
  type BrowserContext,
  type Page,
} from "@playwright/test";
import { startCoverage, stopCoverage } from "./coverage-helper";
import { DATA_ROOT_PATH } from "./paths";
import { firstPage, launchDebugApp } from "./app-launcher";

/**
 * Playwright fixtures that drive the real Tauri app over WebView2's CDP
 * endpoint.
 *
 * See https://playwright.dev/docs/webview2 — Playwright cannot launch a Tauri
 * desktop binary as a browser, so instead we spawn the debug executable with
 * remote debugging enabled and attach to its WebView2 instance.
 *
 * Note: because these fixtures replace the built-in `browser`/`context`/`page`
 * fixtures, Playwright does not run its own tracing/video capture. A
 * screenshot is attached on failure instead; add tracing here later if needed.
 */
export const test = base.extend<{
  browser: Browser;
  context: BrowserContext;
  page: Page;
}>({
  // Worker-scoped, matching Playwright's built-in `browser` fixture: the app is
  // launched once per worker against the shared throwaway data root. Specs that
  // need their own catalogue state use the test-scoped fixture in
  // `app-fixture.ts` instead (it can read the per-file `dataRoot` option).
  browser: async ({ playwright }, use, testInfo) => {
    const app = await launchDebugApp({
      playwright,
      dataRoot: DATA_ROOT_PATH,
      workerIndex: testInfo.workerIndex,
    });
    try {
      await use(app.browser);
    } finally {
      await app.dispose();
    }
  },

  context: async ({ browser }, use) => {
    const [context] = browser.contexts();
    if (!context) {
      throw new Error("WebView2 exposed no browser context.");
    }
    await use(context);
  },

  page: async ({ context }, use, testInfo) => {
    const page = await firstPage(context);

    await startCoverage(page);

    // The WebView2 opens on about:blank and navigates to the application URL
    // once the frontend is served. Wait for the real document before handing
    // the page to a test so nothing races the initial navigation.
    await page.waitForSelector("#app", { state: "attached", timeout: 30_000 });

    try {
      await use(page);
    } finally {
      await stopCoverage(page);

      if (testInfo.status !== testInfo.expectedStatus) {
        await page
          .screenshot({ path: testInfo.outputPath("failure.png") })
          .catch(() => undefined);
      }
    }
  },
});

export { expect };
