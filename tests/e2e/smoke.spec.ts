import { test, expect } from "./fixtures";

/**
 * Smoke tests: prove Playwright is attached to the real Tauri WebView2 and
 * that the backend-served UI renders. These intentionally avoid depending on a
 * specific catalogue state so they stay valid as the app evolves.
 */
test.describe("application shell", () => {
  test("runs inside the real Tauri webview", async ({ page }) => {
    const hasBridge = await page.evaluate(
      () =>
        typeof (
          window as unknown as {
            __TAURI_INTERNALS__?: { invoke?: unknown };
          }
        ).__TAURI_INTERNALS__?.invoke === "function",
    );
    expect(hasBridge).toBe(true);
  });

  test("renders the title and clears the startup splash", async ({ page }) => {
    await expect(page).toHaveTitle(/Embroidery Catalogue/);
    await expect(page.locator("#app")).toBeVisible();

    // The startup splash is replaced once the backend status resolves.
    await expect(page.getByText("Loading Embroidery Catalogue…")).toBeHidden();
    await expect(page.locator("#app > *").first()).toBeVisible();
  });
});
