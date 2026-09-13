import { execSync, spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type {
  Browser,
  BrowserContext,
  BrowserType,
  Page,
} from "@playwright/test";
import { CDP_PORT_BASE, EXE_PATH, REPO_ROOT } from "./paths";

/**
 * Launch the built Tauri debug executable with WebView2 remote debugging
 * enabled and attach to it over the Chrome DevTools Protocol.
 *
 * Playwright cannot launch a Tauri desktop binary as a browser, so both the
 * shared `browser` fixture and the per-data-root fixture in `app-fixture.ts`
 * use this single launcher (see https://playwright.dev/docs/webview2).
 */

/**
 * Wait until the WebView2 remote-debugging endpoint answers.
 *
 * WebView2 exposes the CDP HTTP endpoint at `/json/version` as soon as the
 * `--remote-debugging-port` flag is active, so it is a reliable readiness
 * probe (more robust than parsing the app's stdout).
 */
export async function waitForCdpEndpoint(
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
export async function firstPage(
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

export interface LaunchOptions {
  /** The `playwright` fixture (only `chromium` is used). */
  playwright: { chromium: BrowserType };
  /** Absolute data root the app under test should use (`EMBROIDERY_DATA_ROOT`). */
  dataRoot: string;
  /** Worker index, used to keep ports/profiles unique per worker. */
  workerIndex: number;
  /**
   * Port/profile offset. Allows a spec to run its own app alongside the shared
   * worker app without a CDP port or profile collision.
   */
  portOffset?: number;
}

export interface LaunchedApp {
  browser: Browser;
  context: BrowserContext;
  page: Page;
  /** Close the connection, kill the app process and remove its temp profile. */
  dispose: () => Promise<void>;
}

export async function launchDebugApp({
  playwright,
  dataRoot,
  workerIndex,
  portOffset = 0,
}: LaunchOptions): Promise<LaunchedApp> {
  if (!fs.existsSync(EXE_PATH)) {
    throw new Error(
      `Debug executable not found at:\n  ${EXE_PATH}\n` +
        "Build it once with:  npm run e2e:build",
    );
  }

  const cdpPort = CDP_PORT_BASE + portOffset + workerIndex;
  const userDataDir = path.join(
    fs.realpathSync.native(os.tmpdir()),
    "embroidery-catalogue-e2e",
    `user-data-${portOffset}-${workerIndex}`,
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
      // Keep each app instance's WebView2 profile isolated.
      WEBVIEW2_USER_DATA_FOLDER: userDataDir,
      // Point the debug app at the throwaway test catalogue. Honoured only by
      // debug builds (see src/paths.rs).
      EMBROIDERY_DATA_ROOT: dataRoot,
    },
    stdio: "ignore",
  });

  const cleanup = async (): Promise<void> => {
    killProcessTree(app);
    // Cleanup must never fail: WebView2 can still hold file handles for a moment
    // after the process tree is killed.
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
  };

  let browser: Browser | undefined;
  try {
    await waitForCdpEndpoint(`http://127.0.0.1:${cdpPort}/json/version`);
    browser = await playwright.chromium.connectOverCDP(
      `http://127.0.0.1:${cdpPort}`,
    );

    const [context] = browser.contexts();
    if (!context) {
      throw new Error("WebView2 exposed no browser context.");
    }

    const page = await firstPage(context);
    // The WebView2 opens on about:blank and navigates to the application URL
    // once the frontend is served. Wait for the real document before handing the
    // page out so nothing races the initial navigation.
    await page.waitForSelector("#app", { state: "attached", timeout: 30_000 });

    return {
      browser,
      context,
      page,
      dispose: async () => {
        await browser?.close().catch(() => undefined);
        await cleanup();
      },
    };
  } catch (error) {
    // Never leak the app process or profile when startup fails part-way.
    await browser?.close().catch(() => undefined);
    await cleanup();
    throw error;
  }
}
