// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/svelte";
import HelpView from "../HelpView.svelte";

// ---------------------------------------------------------------------------
// HelpView.svelte — a presentational component following John Carroll's
// Minimalism principles. Provides quick action guidance and links to the
// full in-app documentation guides.
// ---------------------------------------------------------------------------

/** Type-guard helper so querySelector results can be used as HTMLElements. */
function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

/** Returns a `within`-scoped query API for the given <section id>. */
function sectionQueries(container: HTMLElement, id: string) {
  return within(element(container.querySelector(`#${id}`), `Expected section #${id}.`));
}

describe("HelpView", () => {
  describe("page chrome", () => {
    it("renders the page heading 'Help'", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "Help" })).toBeInTheDocument();
    });

    it("renders the subtitle describing the help page", () => {
      render(HelpView);

      expect(
        screen.getByText("Quick guidance for using the Embroidery Catalogue.")
      ).toBeInTheDocument();
    });
  });

  describe("navigation links", () => {
    it("renders all 12 section navigation links with correct hrefs", () => {
      render(HelpView);

      const expectedLinks = [
        { text: "Browse & Search", href: "#/help?section=search", emoji: "🔍" },
        { text: "Design Detail", href: "#/help?section=design-detail", emoji: "🎨" },
        {
          text: "Designers, Sources & Hoops",
          href: "#/help?section=designers-sources",
          emoji: "🏷️",
        },
        { text: "Importing", href: "#/help?section=importing", emoji: "📥" },
        {
          text: "Tag Word Matches",
          href: "#/help?section=word-matches",
          emoji: "🔤",
        },
        {
          text: "Data Storage & External Drives",
          href: "#/help?section=storage",
          emoji: "💾",
        },
        { text: "AI Tagging", href: "#/help?section=ai-tagging", emoji: "🤖" },
        {
          text: "Batch Operations",
          href: "#/help?section=batch-operations",
          emoji: "🏷",
        },
        { text: "Projects", href: "#/help?section=projects", emoji: "📁" },
        {
          text: "Maintenance",
          href: "#/help?section=maintenance",
          emoji: "🛠",
        },
        {
          text: "Settings",
          href: "#/help?section=settings",
          emoji: "⚙️",
        },
        {
          text: "Troubleshooting",
          href: "#/help?section=troubleshooting",
          emoji: "🔧",
        },
      ];

      for (const { text, href, emoji } of expectedLinks) {
        const link = screen.getByRole("link", { name: `${emoji} ${text}` });
        expect(link).toBeInTheDocument();
        expect(link).toHaveAttribute("href", href);
      }
    });
  });

  describe("search section", () => {
    it("renders the Browse & Search heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🔍 Browse & Search" })).toBeInTheDocument();
    });

    it("renders the search section content and guide links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "search");

      const browseLink = section.getByRole("link", { name: "Browse" });
      expect(browseLink).toHaveAttribute("href", "#/designs");

      expect(section.getByText("Search operators:")).toBeInTheDocument();
      expect(section.getByText("Verification status:")).toBeInTheDocument();
      expect(section.getByText("Quick search vs. filters:")).toBeInTheDocument();
      expect(section.getByText("Active filter chips:")).toBeInTheDocument();
      expect(section.getByText("Sorting:")).toBeInTheDocument();
      expect(section.getByText("Fast tag verification:")).toBeInTheDocument();

      const searchGuideLink = section.getByRole("link", {
        name: "Read guide → Search & Filtering Guide",
      });
      expect(searchGuideLink).toHaveAttribute("href", "#/about/document/search-and-filtering");

      const bulkGuideLink = section.getByRole("link", {
        name: "Read guide → Bulk Tagging Guide",
      });
      expect(bulkGuideLink).toHaveAttribute("href", "#/about/document/browse-bulk-actions");
    });

    it("renders the search section with a section id of 'search'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#search");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("design-detail section", () => {
    it("renders the Design Detail heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🎨 Design Detail" })).toBeInTheDocument();
    });

    it("renders the design-detail section content and full guide link", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "design-detail");

      const browseLink = section.getByRole("link", { name: "Browse" });
      expect(browseLink).toHaveAttribute("href", "#/designs");

      expect(section.getByText("Open a design:")).toBeInTheDocument();
      expect(section.getByText("Actual size (1:1) & zoom:")).toBeInTheDocument();
      expect(section.getByText("Thread colours & stops:")).toBeInTheDocument();
      expect(section.getByText("Open in embroidery software:")).toBeInTheDocument();
      expect(section.getByText("Find on disk:")).toBeInTheDocument();
      expect(section.getByText("Edit metadata:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read guide → Design Detail Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/design-detail");
    });

    it("renders the design-detail section with a section id of 'design-detail'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#design-detail");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("designers-sources section", () => {
    it("renders the Designers, Sources & Hoops heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "🏷️ Designers, Sources & Hoops" })
      ).toBeInTheDocument();
    });

    it("renders the designers-sources section content and guide links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "designers-sources");

      const designersLink = section.getByRole("link", { name: "Designers" });
      expect(designersLink).toHaveAttribute("href", "#/admin/data/designers");

      const sourcesLink = section.getByRole("link", { name: "Sources" });
      expect(sourcesLink).toHaveAttribute("href", "#/admin/data/sources");

      const hoopsLink = section.getByRole("link", { name: "Hoops" });
      expect(hoopsLink).toHaveAttribute("href", "#/admin/data/hoops");

      expect(section.getByText("First import setup:")).toBeInTheDocument();
      expect(section.getByText("Assign per folder:")).toBeInTheDocument();
      expect(section.getByText("Manage creators & suppliers:")).toBeInTheDocument();
      expect(section.getByText("Machine hoop compatibility:")).toBeInTheDocument();

      const firstImportGuideLink = section.getByRole("link", {
        name: "Read guide → First Import Actions Guide",
      });
      expect(firstImportGuideLink).toHaveAttribute(
        "href",
        "#/about/document/first-import-actions"
      );

      const folderAssignmentGuideLink = section.getByRole("link", {
        name: "Read guide → Import Folder Assignment Guide",
      });
      expect(folderAssignmentGuideLink).toHaveAttribute(
        "href",
        "#/about/document/import-folder-assignment"
      );
    });

    it("renders the designers-sources section with a section id of 'designers-sources'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#designers-sources");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("importing section", () => {
    it("renders the Importing heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "📥 Importing" })).toBeInTheDocument();
    });

    it("renders the importing section content and full guide link", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "importing");

      const importLink = section.getByRole("link", { name: "Import" });
      expect(importLink).toHaveAttribute("href", "#/import");

      expect(section.getByText("Choosing folders:")).toBeInTheDocument();
      expect(section.getByText("Live scan progress:")).toBeInTheDocument();
      expect(section.getByText("Review and metadata:")).toBeInTheDocument();
      expect(section.getByText("Tag check before import:")).toBeInTheDocument();
      expect(section.getByText("Tagging during import:")).toBeInTheDocument();
      expect(section.getByText("Alphabet & font compression:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Import Workflow Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/import-workflow");
    });

    it("renders the importing section with a section id of 'importing'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#importing");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("word-matches section", () => {
    it("renders the Tag Word Matches heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🔤 Tag Word Matches" })).toBeInTheDocument();
    });

    it("renders the word-matches section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "word-matches");

      expect(section.getByText("Offline & automatic:")).toBeInTheDocument();
      expect(section.getByText("Smart matching:")).toBeInTheDocument();
      expect(section.getByText("Folder hierarchies & supplier masking:")).toBeInTheDocument();
      expect(section.getByText("Manage matches:")).toBeInTheDocument();

      const manageLink = section.getByRole("link", { name: "Word Matches" });
      expect(manageLink).toHaveAttribute("href", "#/admin/data/tag-matches");

      const tagsLink = section.getByRole("link", { name: "Tags" });
      expect(tagsLink).toHaveAttribute("href", "#/admin/data/tags");

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Tag Word Matches Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/tag-word-matches");
    });

    it("renders the word-matches section with a section id of 'word-matches'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#word-matches");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("storage section", () => {
    it("renders the Data Storage & External Drives heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "💾 Data Storage & External Drives" })
      ).toBeInTheDocument();
    });

    it("renders the storage section content and guide link", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "storage");

      expect(section.getByText("Original files untouched:")).toBeInTheDocument();
      expect(section.getByText("SD card recommendation:")).toBeInTheDocument();
      expect(section.getByText("Filesystem formatting:")).toBeInTheDocument();
      expect(section.getByText("Keep drives connected:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Data Storage Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/data-storage");
    });

    it("renders the storage section with a section id of 'storage'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#storage");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("ai-tagging section", () => {
    it("renders the AI Tagging heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🤖 AI Tagging" })).toBeInTheDocument();
    });

    it("renders the ai-tagging section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "ai-tagging");

      const studioLink = section.getByRole("link", { name: "Google AI Studio" });
      expect(studioLink).toHaveAttribute("href", "https://aistudio.google.com/");

      const pricingLink = section.getByRole("link", { name: "pricing" });
      expect(pricingLink).toHaveAttribute("href", "https://ai.google.dev/pricing");

      const settingsLink = section.getByRole("link", { name: "Settings" });
      expect(settingsLink).toHaveAttribute("href", "#/admin/system/settings");

      const batchOperationsLink = section.getByRole("link", { name: "Batch Operations" });
      expect(batchOperationsLink).toHaveAttribute("href", "#/admin/batch-operations");

      const guideLink = section.getByRole("link", {
        name: "Read full guide → AI Tagging & Batch Operations Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/ai-tagging");

      expect(section.getByText("Get an API key:")).toBeInTheDocument();
      expect(section.getByText("Free tier protection:")).toBeInTheDocument();
      expect(section.getByText("In-app actions:")).toBeInTheDocument();
    });

    it("renders the ai-tagging section with a section id of 'ai-tagging'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#ai-tagging");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("batch-operations section", () => {
    it("renders the Batch Operations heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🏷 Batch Operations" })).toBeInTheDocument();
    });

    it("renders the batch-operations section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "batch-operations");

      const adminLink = section.getByRole("link", {
        name: "Admin → Batch Operations",
      });
      expect(adminLink).toHaveAttribute("href", "#/admin/batch-operations");

      expect(section.getByText("Tagging scopes:")).toBeInTheDocument();
      expect(section.getByText("Local stitching backfill:")).toBeInTheDocument();
      expect(section.getByText("File maintenance:")).toBeInTheDocument();
      expect(section.getByText("Tag verification workflow:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Batch Operations Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/ai-tagging");
    });

    it("renders the batch-operations section with a section id of 'batch-operations'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#batch-operations");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("projects section", () => {
    it("renders the Projects heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "📁 Projects" })).toBeInTheDocument();
    });

    it("renders the projects section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "projects");

      const projectsLink = section.getByRole("link", { name: "Projects" });
      expect(projectsLink).toHaveAttribute("href", "#/projects");

      expect(section.getByText("What projects are for:")).toBeInTheDocument();
      expect(section.getByText("Adding designs:")).toBeInTheDocument();
      expect(section.getByText("Printing:")).toBeInTheDocument();
      expect(section.getByText("Safe removal:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Projects Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/projects-guide");
    });

    it("renders the projects section with a section id of 'projects'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#projects");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("maintenance section", () => {
    it("renders the Maintenance heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🛠 Maintenance" })).toBeInTheDocument();
    });

    it("renders the maintenance section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "maintenance");

      const discrepanciesLink = section.getByRole("link", {
        name: "Library Discrepancies",
      });
      expect(discrepanciesLink).toHaveAttribute("href", "#/admin/system/discrepancies");

      expect(section.getByText("Find Unmatched Design Files:")).toBeInTheDocument();
      expect(section.getByText("Missing Design Files (Orphans):")).toBeInTheDocument();
      expect(section.getByText("Deleting orphans:")).toBeInTheDocument();
      expect(section.getByText("Backups & Restore:")).toBeInTheDocument();

      const backupLink = section.getByRole("link", { name: "Backup" });
      expect(backupLink).toHaveAttribute("href", "#/admin/system/backup");

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Backup & Restore Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/backup-restore");
    });

    it("renders the maintenance section with a section id of 'maintenance'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#maintenance");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("settings section", () => {
    it("renders the Settings heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "⚙️ Settings & Configuration" })
      ).toBeInTheDocument();
    });

    it("renders the settings section content and links", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "settings");

      const settingsLink = section.getByRole("link", { name: "Admin → Settings" });
      expect(settingsLink).toHaveAttribute("href", "#/admin/system/settings");

      expect(section.getByText("Appearance:")).toBeInTheDocument();
      expect(section.getByText("Google Gemini & AI Tagging:")).toBeInTheDocument();
      expect(section.getByText("Database Maintenance:")).toBeInTheDocument();
      expect(section.getByText("Catalogue Storage:")).toBeInTheDocument();
      expect(section.getByText("Digitising & Format Rules:")).toBeInTheDocument();
      expect(section.getByText("Product licence:")).toBeInTheDocument();

      const aboutLink = section.getByRole("link", { name: "About" });
      expect(aboutLink).toHaveAttribute("href", "#/about");

      const licenceLink = section.getByRole("link", { name: "Licence & Dependencies" });
      expect(licenceLink).toHaveAttribute("href", "#/about/licence");

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Settings Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/settings-guide");
    });

    it("renders the settings section with a section id of 'settings'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#settings");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("troubleshooting section", () => {
    it("renders the Troubleshooting heading", () => {
      render(HelpView);

      expect(screen.getByRole("heading", { name: "🔧 Troubleshooting" })).toBeInTheDocument();
    });

    it("renders the troubleshooting section content and guide link", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "troubleshooting");

      expect(section.getByText("Missing folder / changed drive letter:")).toBeInTheDocument();
      expect(section.getByText("Database preparation on startup:")).toBeInTheDocument();
      expect(section.getByText("Database full / 4 GB limit:")).toBeInTheDocument();
      expect(section.getByText("AI tagging errors (429):")).toBeInTheDocument();
      expect(section.getByText("Import scan finds nothing:")).toBeInTheDocument();

      const guideLink = section.getByRole("link", {
        name: "Read full guide → Getting Started & Setup Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/getting-started");
    });

    it("renders the troubleshooting section with a section id of 'troubleshooting'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#troubleshooting");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("structure", () => {
    it("renders exactly 12 help sections with the expected ids", () => {
      const { container } = render(HelpView);

      const sections = Array.from(container.querySelectorAll("section"));
      const ids = sections.map((section) => section.id);

      expect(sections).toHaveLength(12);
      expect(ids).toEqual([
        "search",
        "design-detail",
        "designers-sources",
        "importing",
        "word-matches",
        "storage",
        "ai-tagging",
        "batch-operations",
        "projects",
        "maintenance",
        "settings",
        "troubleshooting",
      ]);
    });

    it("counts the same number of navigation links as sections", () => {
      const { container } = render(HelpView);

      const links = Array.from(container.querySelectorAll("a"));
      const sections = container.querySelectorAll("section");

      expect(links.length).toBeGreaterThanOrEqual(12);
      expect(sections).toHaveLength(12);

      // Exactly 12 of the links are the in-page section navigators.
      const sectionNavigators = links.filter((link) =>
        (link.getAttribute("href") || "").startsWith("#/help?section=")
      );
      expect(sectionNavigators).toHaveLength(12);
    });
  });
});
