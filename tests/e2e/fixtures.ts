import { execSync, spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  test as base,
  expect,
  type Browser,
  type BrowserContext,
  type Page,
} from "@playwright/test";
import { CDP_PORT_BASE, DATA_ROOT_PATH, EXE_PATH, REPO_ROOT } from "./paths";

/**
 * Wait until the WebView2 remote-debugging endpoint answers.
 *
 * WebView2 exposes the CDP HTTP endpoint at `/json/version` as soon as the
 * `--remote-debugging-port` flag is active, so it is a reliable readiness
 * probe (more robust than parsing the app's stdout).
 */
async function waitForCdpEndpoint(
  url: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: unknown = null;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error(
    `WebView2 CDP endpoint ${url} did not become ready within ${timeoutMs}ms. ` +
      `Last error: ${String(lastError)}`,
  );
}

/** Return the first page target WebView2 exposes, waiting for it to exist. */
async function firstPage(
  context: BrowserContext,
  timeoutMs = 30_000,
): Promise<Page> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const [existing] = context.pages();
    if (existing) {
      return existing;
    }
    await context.waitForEvent("page", { timeout: 500 }).catch(() => undefined);
  }
  throw new Error("WebView2 exposed no page target.");
}

/** Force-kill a process and its children (Windows desktop app). */
function killProcessTree(child: ChildProcess): void {
  if (!child.pid) {
    return;
  }
  try {
    execSync(`taskkill /pid ${child.pid} /T /F`, { stdio: "ignore" });
  } catch {
    // Process already exited — nothing to clean up.
  }
}

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
  browser: async ({ playwright }, use, testInfo) => {
    if (!fs.existsSync(EXE_PATH)) {
      throw new Error(
        `Debug executable not found at:\n  ${EXE_PATH}\n` +
          "Build it once with:  npm run e2e:build",
      );
    }

    const cdpPort = CDP_PORT_BASE + testInfo.workerIndex;
    const userDataDir = path.join(
      fs.realpathSync.native(os.tmpdir()),
      "embroidery-catalogue-e2e",
      `user-data-${testInfo.workerIndex}`,
    );
    try {
      fs.rmSync(userDataDir, { recursive: true, force: true });
    } catch {
      // A locked leftover profile from a crashed run is harmless.
    }
    fs.mkdirSync(userDataDir, { recursive: true });

    const app = spawn(EXE_PATH, [], {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        // Enable the CDP endpoint on this worker's port.
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${cdpPort}`,
        // Keep each worker's WebView2 profile isolated.
        WEBVIEW2_USER_DATA_FOLDER: userDataDir,
        // Point the debug app at the throwaway test catalogue. Honoured only
        // by debug builds (see src/paths.rs).
        EMBROIDERY_DATA_ROOT: DATA_ROOT_PATH,
      },
      stdio: "ignore",
    });

    let browser: Browser | undefined;
    try {
      await waitForCdpEndpoint(`http://127.0.0.1:${cdpPort}/json/version`);
      browser = await playwright.chromium.connectOverCDP(
        `http://127.0.0.1:${cdpPort}`,
      );
      await use(browser);
    } finally {
      if (browser) {
        await browser.close().catch(() => undefined);
      }
      killProcessTree(app);
      // Cleanup must never fail the test: WebView2 can still hold file handles
      // for a moment after the process tree is killed.
      try {
        fs.rmSync(userDataDir, {
          recursive: true,
          force: true,
          maxRetries: 5,
          retryDelay: 200,
        });
      } catch {
        // Best-effort: a leftover temp profile is harmless.
      }
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

    // The WebView2 opens on about:blank and navigates to the application URL
    // once the frontend is served. Wait for the real document before handing
    // the page to a test so nothing races the initial navigation.
    await page.waitForSelector("#app", { state: "attached", timeout: 30_000 });

    await use(page);

    if (testInfo.status !== testInfo.expectedStatus) {
      await page
        .screenshot({ path: testInfo.outputPath("failure.png") })
        .catch(() => undefined);
    }
  },
});

export { expect };
