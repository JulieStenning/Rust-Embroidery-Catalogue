// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, expectMainView } from "./helpers";

/**
 * Navigation coverage for the application shell. These assert the real
 * hash-router wiring (nav links and deep links) against the live Tauri app.
 */
test.describe("main navigation", () => {
  test("boots into the main app on the Browse view", async ({ page }) => {
    await expectMainView(page);
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
  });

  test("top-level nav links route to their views", async ({ page }) => {
    await expectMainView(page);

    await clickNav(page, "Projects");
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();

    await clickNav(page, "Help");
    await expect(page.getByRole("heading", { name: "Help" })).toBeVisible();

    await clickNav(page, "Import");
    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();

    await clickNav(page, "Browse");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
  });

  test("admin hub links route to their default views", async ({ page }) => {
    await expectMainView(page);

    await clickNav(page, "Manage Data");
    await expect(
      page.getByRole("heading", { name: "Manage Designers" }),
    ).toBeVisible();

    await clickNav(page, "System");
    await expect(
      page.getByRole("heading", { name: "Application Settings" }),
    ).toBeVisible();

    await clickNav(page, "Batch Operations");
    await expect(
      page.getByRole("heading", { name: "Batch Operations" }),
    ).toBeVisible();
  });

  test("deep links render the target view", async ({ page }) => {
    await gotoRoute(page, "#/admin/system/backup");
    await expect(
      page.getByRole("heading", { name: "Backup & Restore" }),
    ).toBeVisible();

    await gotoRoute(page, "#/admin/data/tags");
    await expect(
      page.getByRole("heading", { name: "Manage Tags" }),
    ).toBeVisible();

    await gotoRoute(page, "#/projects");
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();

    await gotoRoute(page, "#/about/licence");
    await expect(page.getByTestId("licence-application-tab")).toBeVisible();

    await page.getByRole("button", { name: "Rust Dependencies" }).click();
    await expect(page.getByTestId("licence-rust-tab")).toBeVisible();
    await expect(page.getByTestId("dependency-search-input")).toBeVisible();
    await expect(page.getByTestId("dependency-count-summary")).toBeVisible();

    await page.getByRole("button", { name: "Frontend Dependencies" }).click();
    await expect(page.getByTestId("licence-frontend-tab")).toBeVisible();
    const searchInput = page.getByTestId("dependency-search-input");
    await searchInput.fill("svelte");
    await expect(page.getByTestId("dependency-count-summary")).toContainText(
      "packages",
    );
  });

  test("suppresses browser context menu on chrome and displays tailored text context menu on inputs", async ({
    page,
  }) => {
    await expectMainView(page);

    // Right-clicking the window body/chrome must prevent default browser menu and keep menu closed
    const isChromePrevented = await page.evaluate(() => {
      const event = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
        clientX: 100,
        clientY: 100,
      });
      document.body.dispatchEvent(event);
      return event.defaultPrevented;
    });
    expect(isChromePrevented).toBe(true);
    await expect(page.getByTestId("text-context-menu")).not.toBeVisible();

    // Find a search input on the main view
    const searchInput = page
      .locator("input[type='text'], input[type='search'], input")
      .first();
    await expect(searchInput).toBeVisible();

    // Right-click the input field
    await searchInput.click({ button: "right" });

    // Custom desktop context menu must appear
    const contextMenu = page.getByTestId("text-context-menu");
    await expect(contextMenu).toBeVisible();

    // Must contain Cut, Copy, Paste, and Select All
    await expect(page.getByTestId("context-menu-cut")).toBeVisible();
    await expect(page.getByTestId("context-menu-copy")).toBeVisible();
    await expect(page.getByTestId("context-menu-paste")).toBeVisible();
    await expect(page.getByTestId("context-menu-select-all")).toBeVisible();

    // Must NOT contain browser artefacts
    await expect(contextMenu).not.toContainText("Import passwords");
    await expect(contextMenu).not.toContainText("Undo");
    await expect(contextMenu).not.toContainText("Writing direction");
    await expect(contextMenu).not.toContainText("More tools");
    await expect(contextMenu).not.toContainText("Inspect");

    // Pressing Escape closes the context menu
    await page.keyboard.press("Escape");
    await expect(contextMenu).not.toBeVisible();
  });
});
