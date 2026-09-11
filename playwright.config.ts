import { defineConfig } from "@playwright/test";

/**
 * Playwright end-to-end configuration for the Embroidery Catalogue desktop app.
 *
 * The app is a Tauri v2 binary, not a web app, so Playwright does not launch a
 * browser here. The custom `browser` fixture in `tests/e2e/fixtures.ts` spawns
 * the built debug executable with WebView2 remote debugging enabled and
 * attaches over the Chrome DevTools Protocol.
 * See https://playwright.dev/docs/webview2.
 *
 * Every test shares one SQLite database and one desktop window, so the suite
 * runs with a single worker.
 */
export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: "html",
  outputDir: "tests/e2e/test-results/",
  globalSetup: "./tests/e2e/global-setup.ts",
  globalTeardown: "./tests/e2e/global-teardown.ts",
  /* Give the desktop app room to boot and serve its first render. */
  timeout: 60_000,
  expect: { timeout: 10_000 },
});
