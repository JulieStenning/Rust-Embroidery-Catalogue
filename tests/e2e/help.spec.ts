// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, expectMainView, mainMenu } from "./helpers";

/**
 * Help functionality coverage for the application.
 * Tests direct routing, in-page section navigation, context-aware Back button,
 * contextual help links from other views, and outbound cross-links against
 * the live desktop application.
 */
test.describe("help functionality", () => {
  test("renders Help page via top navigation and direct deep link", async ({
    page,
  }) => {
    await expectMainView(page);

    // Navigate via top menu
    await clickNav(page, "Help");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText("Quick guidance for using the Embroidery Catalogue."),
    ).toBeVisible();

    // The top navbar link is highlighted
    const helpNavLink = mainMenu(page).getByRole("link", {
      name: "Help",
      exact: true,
    });
    await expect(helpNavLink).toHaveClass(/menu-link-active/);

    // Navigate via direct route
    await gotoRoute(page, "#/designs");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    await gotoRoute(page, "#/help");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
  });

  test("renders all 8 table-of-contents navigation links and all 8 sections", async ({
    page,
  }) => {
    await gotoRoute(page, "#/help");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();

    const expectedSections = [
      { id: "search", label: "🔍 Search", heading: "🔍 Search" },
      { id: "importing", label: "📥 Importing", heading: "📥 Importing" },
      {
        id: "storage",
        label: "💾 Data Storage & External Drives",
        heading: "💾 Data Storage & External Drives",
      },
      { id: "ai-tagging", label: "🤖 AI Tagging", heading: "🤖 AI Tagging" },
      {
        id: "batch-operations",
        label: "🏷 Batch Operations",
        heading: "🏷 Batch Operations",
      },
      { id: "projects", label: "📁 Projects", heading: "📁 Projects" },
      { id: "maintenance", label: "🛠 Maintenance", heading: "🛠 Maintenance" },
      {
        id: "troubleshooting",
        label: "🔧 Troubleshooting",
        heading: "🔧 Troubleshooting",
      },
    ];

    for (const sec of expectedSections) {
      // In-page navigation pill
      const tocLink = page.getByRole("link", { name: sec.label });
      await expect(tocLink).toBeVisible();
      await expect(tocLink).toHaveAttribute("href", `#/help?section=${sec.id}`);

      // Corresponding section container and heading
      const sectionEl = page.locator(`section#${sec.id}`);
      await expect(sectionEl).toBeVisible();
      await expect(
        sectionEl.getByRole("heading", { name: sec.heading }),
      ).toBeVisible();
    }
  });

  test("clicking in-page section links updates the URL hash", async ({
    page,
  }) => {
    await gotoRoute(page, "#/help");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();

    // Click "🤖 AI Tagging" TOC pill
    await page.getByRole("link", { name: "🤖 AI Tagging" }).click();
    await expect(page).toHaveURL(/#\/help\?section=ai-tagging$/);

    // Click "🔧 Troubleshooting" TOC pill
    await page.getByRole("link", { name: "🔧 Troubleshooting" }).click();
    await expect(page).toHaveURL(/#\/help\?section=troubleshooting$/);
  });

  test("deep-links directly to specific help sections", async ({ page }) => {
    await gotoRoute(page, "#/help?section=storage");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
    await expect(page.locator("section#storage")).toBeVisible();
    await expect(
      page.locator("section#storage").getByRole("heading", {
        name: "💾 Data Storage & External Drives",
      }),
    ).toBeVisible();
  });

  test("displays context-aware Back button when navigating from Browse and returns correctly", async ({
    page,
  }) => {
    // Start at Browse
    await gotoRoute(page, "#/designs");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    // Navigate to Help
    await clickNav(page, "Help");
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();

    // Back button should be visible in the shell
    const backButton = page.getByRole("button", { name: /← Back/ });
    await expect(backButton).toBeVisible();

    // Click Back to return to Browse
    await backButton.click();
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
  });

  test("does not display Back button on cold deep-link to Help", async ({
    page,
  }) => {
    // Deep link directly to Help and reload to clear in-memory routing history
    await gotoRoute(page, "#/help");
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();

    // On cold deep link / fresh mount, no prior route exists, so Back button is hidden
    await expect(page.getByRole("button", { name: /← Back/ })).toBeHidden();
  });

  test("contextual help links in Browse, Projects, and Import views route directly to the relevant help section", async ({
    page,
  }) => {
    // 1. Browse view -> Search help link
    await gotoRoute(page, "#/designs");
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();
    const searchHelpLink = page.getByRole("link", { name: "Search help" });
    await expect(searchHelpLink).toBeVisible();
    await searchHelpLink.click();

    await expect(page).toHaveURL(/#\/help\?section=search$/);
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
    await expect(page.locator("section#search")).toBeVisible();

    // 2. Import view -> Import help link
    await gotoRoute(page, "#/import");
    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();
    const importHelpLink = page.getByRole("link", { name: "Import help" });
    await expect(importHelpLink).toBeVisible();
    await importHelpLink.click();

    await expect(page).toHaveURL(/#\/help\?section=importing$/);
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
    await expect(page.locator("section#importing")).toBeVisible();

    // 3. Projects view -> Learn more link
    await gotoRoute(page, "#/projects");
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();
    const projectsLearnMoreLink = page.getByRole("link", {
      name: "Learn more",
    });
    await expect(projectsLearnMoreLink).toBeVisible();
    await projectsLearnMoreLink.click();

    await expect(page).toHaveURL(/#\/help\?section=projects$/);
    await expect(
      page.getByRole("heading", { name: "Help", exact: true }),
    ).toBeVisible();
    await expect(page.locator("section#projects")).toBeVisible();
  });

  test("internal links in Help content route to target application views", async ({
    page,
  }) => {
    // Search section -> Browse link
    await gotoRoute(page, "#/help");
    await page
      .locator("section#search")
      .getByRole("link", { name: "Browse" })
      .click();
    await expect(
      page.getByRole("heading", { name: "Browse Designs" }),
    ).toBeVisible();

    // Importing section -> Import link
    await gotoRoute(page, "#/help");
    await page
      .locator("section#importing")
      .getByRole("link", { name: "Import" })
      .click();
    await expect(
      page.getByRole("heading", { name: "Bulk Import" }),
    ).toBeVisible();

    // Projects section -> Projects link
    await gotoRoute(page, "#/help");
    await page
      .locator("section#projects")
      .getByRole("link", { name: "Projects" })
      .click();
    await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();

    // Storage section -> Data Storage & External Drives Guide
    await gotoRoute(page, "#/help");
    await page
      .locator("section#storage")
      .getByRole("link", { name: "Data Storage & External Drives Guide" })
      .click();
    await expect(
      page.getByRole("heading", {
        name: "Data Storage & External Drives Guide",
      }),
    ).toBeVisible();

    // AI Tagging section -> AI Tagging & Batch Operations Guide
    await gotoRoute(page, "#/help");
    await page
      .locator("section#ai-tagging")
      .getByRole("link", { name: "AI Tagging & Batch Operations Guide" })
      .click();
    await expect(
      page.getByRole("heading", {
        name: "Batch Operations and Backfill (with AI Tagging)",
      }),
    ).toBeVisible();

    // Maintenance section -> Orphans link
    await gotoRoute(page, "#/help");
    await page
      .locator("section#maintenance")
      .getByRole("link", { name: "Orphans" })
      .click();
    await expect(
      page.getByRole("heading", { name: "Orphans", exact: true }),
    ).toBeVisible();
  });

  test("external reference links in Help content point to expected external URLs", async ({
    page,
  }) => {
    await gotoRoute(page, "#/help");
    const aiSection = page.locator("section#ai-tagging");

    const studioLink = aiSection.getByRole("link", {
      name: "Google AI Studio",
    });
    await expect(studioLink).toHaveAttribute(
      "href",
      "https://aistudio.google.com/",
    );

    const pricingLink = aiSection.getByRole("link", {
      name: "current pricing",
    });
    await expect(pricingLink).toHaveAttribute(
      "href",
      "https://ai.google.dev/pricing",
    );
  });
});
