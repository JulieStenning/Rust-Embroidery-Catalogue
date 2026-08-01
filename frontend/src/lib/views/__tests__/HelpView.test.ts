import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/svelte";
import HelpView from "../HelpView.svelte";

// ---------------------------------------------------------------------------
// HelpView.svelte — a purely presentational component with no props, no
// external dependencies, and no Tauri IPC calls, so no mocks are required.
//
// Behaviour under test:
//   - Renders the page title and subtitle.
//   - Renders all 7 in-page navigation links with the correct hrefs.
//   - Renders all 7 help sections with their headings and key content.
// ---------------------------------------------------------------------------

/** Type-guard helper so querySelector results can be used as HTMLElements. */
function element<T extends Element>(
  value: T | null | undefined,
  message?: string
): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

/**
 * Creates a text matcher that compares an element's normalized textContent
 * (whitespace collapsed) against an expected string. This is required for
 * paragraphs whose text is broken up by inline elements such as <a>, <strong>
 * or <code>, where the default getByText() string matcher (which only looks at
 * direct text nodes) cannot find the full sentence.
 */
function normalizedText(expected: string) {
  return (_content: string, node: Element | null) =>
    node !== null &&
    (node.textContent ?? "").replace(/\s+/g, " ").trim() === expected;
}

/** Returns a `within`-scoped query API for the given <section id>. */
function sectionQueries(container: HTMLElement, id: string) {
  return within(
    element(container.querySelector(`#${id}`), `Expected section #${id}.`)
  );
}

describe("HelpView", () => {
  describe("page chrome", () => {
    it("renders the page heading 'Help'", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "Help" })
      ).toBeInTheDocument();
    });

    it("renders the subtitle describing the help page", () => {
      render(HelpView);

      expect(
        screen.getByText("Quick guidance for using the Embroidery Catalogue.")
      ).toBeInTheDocument();
    });
  });

  describe("navigation links", () => {
    it("renders all 7 section navigation links with correct hrefs", () => {
      render(HelpView);

      const expectedLinks = [
        { text: "Search", href: "#/help?section=search", emoji: "🔍" },
        { text: "Importing", href: "#/help?section=importing", emoji: "📥" },
        { text: "AI Tagging", href: "#/help?section=ai-tagging", emoji: "🤖" },
        {
          text: "Tagging Actions",
          href: "#/help?section=tagging-actions",
          emoji: "🏷",
        },
        { text: "Projects", href: "#/help?section=projects", emoji: "📁" },
        {
          text: "Maintenance",
          href: "#/help?section=maintenance",
          emoji: "🛠",
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
    it("renders the Search heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "🔍 Search" })
      ).toBeInTheDocument();
    });

    it("renders the search section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "search");

      // Intro paragraph — text is split by the inline Browse link and contains
      // a second sentence, so match against the full normalized textContent.
      expect(
        section.getByText(
          normalizedText(
            "Use the general search box on the Browse page to find designs by keyword. The search checks the filename, notes, designer, source, and tags."
          )
        )
      ).toBeInTheDocument();

      // Internal Browse link
      const browseLink = section.getByRole("link", { name: "Browse" });
      expect(browseLink).toHaveAttribute("href", "#/designs");

      // Example search syntax hints
      expect(section.getByText("Quoted phrases:")).toBeInTheDocument();
      expect(section.getByText('"cross stitch"')).toBeInTheDocument();
      expect(section.getByText("Exclude terms:")).toBeInTheDocument();
      expect(section.getByText("-rose -applique")).toBeInTheDocument();
      expect(section.getByText("OR searches:")).toBeInTheDocument();
      expect(section.getByText("rose OR tulip")).toBeInTheDocument();
      expect(section.getByText("Filename wildcards:")).toBeInTheDocument();
      expect(section.getByText("rose*.jef")).toBeInTheDocument();
      expect(section.getByText("Unverified only:")).toBeInTheDocument();
      expect(
        section.getByText("Quick search vs. filters:")
      ).toBeInTheDocument();
    });

    it("renders the search section with a section id of 'search'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#search");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("importing section", () => {
    it("renders the Importing heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "📥 Importing" })
      ).toBeInTheDocument();
    });

    it("renders the importing section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "importing");

      // Intro paragraph — text is split by the inline Import link.
      expect(
        section.getByText(
          normalizedText(
            "Use Import to scan one or more folders and their sub-folders."
          )
        )
      ).toBeInTheDocument();

      const importLink = section.getByRole("link", { name: "Import" });
      expect(importLink).toHaveAttribute("href", "#/import");

      expect(section.getByText("Choosing folders:")).toBeInTheDocument();
      expect(section.getByText("Review and metadata:")).toBeInTheDocument();
      expect(section.getByText("Tag check before import:")).toBeInTheDocument();
      expect(section.getByText("AI tagging notice:")).toBeInTheDocument();
      expect(
        section.getByText("Error files and large scans:")
      ).toBeInTheDocument();
    });

    it("renders the importing section with a section id of 'importing'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#importing");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("ai-tagging section", () => {
    it("renders the AI Tagging heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "🤖 AI Tagging" })
      ).toBeInTheDocument();
    });

    it("renders the ai-tagging section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "ai-tagging");

      expect(
        section.getByText(
          "Optional Google AI tagging can suggest design-type tags for imported embroidery files."
        )
      ).toBeInTheDocument();

      // External links
      const studioLink = section.getByRole("link", {
        name: "Google AI Studio",
      });
      expect(studioLink).toHaveAttribute(
        "href",
        "https://aistudio.google.com/"
      );

      const pricingLink = section.getByRole("link", {
        name: "current pricing",
      });
      expect(pricingLink).toHaveAttribute("href", "https://ai.google.dev/pricing");

      // Internal links
      const settingsLink = section.getByRole("link", { name: "Settings" });
      expect(settingsLink).toHaveAttribute("href", "#/admin/settings");

      // "Admin → Tagging Actions" also appears in the tagging-actions section
      // further down the page, so scope this query to the ai-tagging section.
      const taggingActionsLink = section.getByRole("link", {
        name: "Admin → Tagging Actions",
      });
      expect(taggingActionsLink).toHaveAttribute(
        "href",
        "#/admin/tagging-actions"
      );

      const guideLink = section.getByRole("link", {
        name: "AI Tagging Guide",
      });
      expect(guideLink).toHaveAttribute("href", "#/about/document/ai-tagging");

      // Content bullets
      expect(section.getByText("Get an API key:")).toBeInTheDocument();
      expect(section.getByText("Enable tiers:")).toBeInTheDocument();
      expect(section.getByText("Batch size and delay:")).toBeInTheDocument();
      expect(section.getByText("In-app actions:")).toBeInTheDocument();
      expect(section.getByText("Potential costs:")).toBeInTheDocument();
      expect(section.getByText("Full guide:")).toBeInTheDocument();
    });

    it("renders the ai-tagging section with a section id of 'ai-tagging'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#ai-tagging");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("tagging-actions section", () => {
    it("renders the Tagging Actions heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "🏷 Tagging Actions" })
      ).toBeInTheDocument();
    });

    it("renders the tagging-actions section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "tagging-actions");

      // Intro paragraph — text is split by the inline Admin → Tagging Actions link.
      expect(
        section.getByText(
          normalizedText(
            "Run AI tagging on existing designs from Admin → Tagging Actions."
          )
        )
      ).toBeInTheDocument();

      const adminLink = section.getByRole("link", {
        name: "Admin → Tagging Actions",
      });
      expect(adminLink).toHaveAttribute("href", "#/admin/tagging-actions");

      expect(
        section.getByText("Tag only untagged designs:")
      ).toBeInTheDocument();
      expect(
        section.getByText("Tag untagged and unverified designs:")
      ).toBeInTheDocument();
      expect(section.getByText("Re-tag ALL designs:")).toBeInTheDocument();
      expect(
        section.getByText("Local stitching backfill:")
      ).toBeInTheDocument();
    });

    it("renders the tagging-actions section with a section id of 'tagging-actions'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#tagging-actions");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("projects section", () => {
    it("renders the Projects heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "📁 Projects" })
      ).toBeInTheDocument();
    });

    it("renders the projects section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "projects");

      // Intro paragraph — text is split by the inline Projects link.
      expect(
        section.getByText(
          normalizedText(
            "Projects let you group designs for planned embroidery tasks."
          )
        )
      ).toBeInTheDocument();

      const projectsLink = section.getByRole("link", { name: "Projects" });
      expect(projectsLink).toHaveAttribute("href", "#/projects");

      expect(
        section.getByText("What projects are for:")
      ).toBeInTheDocument();
      expect(section.getByText("Adding designs:")).toBeInTheDocument();
      expect(section.getByText("Bulk add:")).toBeInTheDocument();
      expect(section.getByText("Printing:")).toBeInTheDocument();
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

      expect(
        screen.getByRole("heading", { name: "🛠 Maintenance" })
      ).toBeInTheDocument();
    });

    it("renders the maintenance section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "maintenance");

      // Intro paragraph — text is split by the inline Orphans link.
      expect(
        section.getByText(
          normalizedText(
            "Use Orphans to find records whose files are missing."
          )
        )
      ).toBeInTheDocument();

      const orphansLink = section.getByRole("link", { name: "Orphans" });
      expect(orphansLink).toHaveAttribute("href", "#/admin/orphans");

      expect(
        section.getByText("What orphaned records are:")
      ).toBeInTheDocument();
      expect(section.getByText("Deleting orphans:")).toBeInTheDocument();
      expect(section.getByText("Use carefully:")).toBeInTheDocument();
    });

    it("renders the maintenance section with a section id of 'maintenance'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#maintenance");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("troubleshooting section", () => {
    it("renders the Troubleshooting heading", () => {
      render(HelpView);

      expect(
        screen.getByRole("heading", { name: "🔧 Troubleshooting" })
      ).toBeInTheDocument();
    });

    it("renders the troubleshooting section content", () => {
      const { container } = render(HelpView);
      const section = sectionQueries(container, "troubleshooting");

      expect(
        section.getByText("Missing folder / changed drive letter:")
      ).toBeInTheDocument();
      expect(
        section.getByText("Import scan finds nothing:")
      ).toBeInTheDocument();
      expect(
        section.getByText("Files missing after import:")
      ).toBeInTheDocument();
      expect(
        section.getByText("Managed storage location:")
      ).toBeInTheDocument();
      expect(section.getByText("Still stuck:")).toBeInTheDocument();
    });

    it("renders the troubleshooting section with a section id of 'troubleshooting'", () => {
      const { container } = render(HelpView);

      const section = container.querySelector("#troubleshooting");
      expect(section).not.toBeNull();
      expect(section?.tagName).toBe("SECTION");
    });
  });

  describe("structure", () => {
    it("renders exactly 7 help sections with the expected ids", () => {
      const { container } = render(HelpView);

      const sections = Array.from(container.querySelectorAll("section"));
      const ids = sections.map((section) => section.id);

      expect(sections).toHaveLength(7);
      expect(ids).toEqual([
        "search",
        "importing",
        "ai-tagging",
        "tagging-actions",
        "projects",
        "maintenance",
        "troubleshooting",
      ]);
    });

    it("counts the same number of navigation links as sections", () => {
      const { container } = render(HelpView);

      const links = Array.from(container.querySelectorAll("a"));
      const sections = container.querySelectorAll("section");

      expect(links.length).toBeGreaterThanOrEqual(7);
      expect(sections).toHaveLength(7);

      // Exactly 7 of the links are the in-page section navigators.
      const sectionNavigators = links.filter((link) =>
        (link.getAttribute("href") || "").startsWith("#/help?section=")
      );
      expect(sectionNavigators).toHaveLength(7);
    });
  });
});