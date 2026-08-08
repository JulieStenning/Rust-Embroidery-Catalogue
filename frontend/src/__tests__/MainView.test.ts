import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import MainView from "../lib/MainView.svelte";

// ---------------------------------------------------------------------------
// Mock normalizeHash as a passthrough (minus query strings). The real
// implementation normalizes unknown routes to "#/designs", which makes the
// project-new and Route Not Found branches unreachable in tests. Keeping
// the query-stripping behavior lets help-section scrolling work too.
// ---------------------------------------------------------------------------
vi.mock("../lib/utils/routing.js", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/utils/routing.js")>();
  return {
    ...actual,
    normalizeHash: (hashString: string) => {
      const questionIndex = hashString.indexOf("?");
      return questionIndex !== -1
        ? hashString.slice(0, questionIndex)
        : hashString;
    },
  };
});

// ---------------------------------------------------------------------------
// Mock every child view so MainView's routing shell can be tested in
// isolation. Each mock is a real (tiny) Svelte component that exposes a
// data-testid so we can assert which router branch is rendered.
// ---------------------------------------------------------------------------
vi.mock("../lib/views/HelpView.svelte", async () => {
  const { default: HelpView } = await import("./__mocks__/HelpView.svelte");
  return { default: HelpView };
});
vi.mock("../lib/views/AboutView.svelte", async () => {
  const { default: AboutView } = await import("./__mocks__/AboutView.svelte");
  return { default: AboutView };
});
vi.mock("../lib/views/AboutDocumentView.svelte", async () => {
  const { default: AboutDocumentView } = await import(
    "./__mocks__/AboutDocumentView.svelte"
  );
  return { default: AboutDocumentView };
});
vi.mock("../lib/views/SettingsView.svelte", async () => {
  const { default: SettingsView } = await import(
    "./__mocks__/SettingsView.svelte"
  );
  return { default: SettingsView };
});
vi.mock("../lib/views/BackupView.svelte", async () => {
  const { default: BackupView } = await import("./__mocks__/BackupView.svelte");
  return { default: BackupView };
});
vi.mock("../lib/views/TaggingActionsView.svelte", async () => {
  const { default: TaggingActionsView } = await import(
    "./__mocks__/TaggingActionsView.svelte"
  );
  return { default: TaggingActionsView };
});
vi.mock("../lib/views/OrphansView.svelte", async () => {
  const { default: OrphansView } = await import("./__mocks__/OrphansView.svelte");
  return { default: OrphansView };
});
vi.mock("../lib/views/ProjectsView.svelte", async () => {
  const { default: ProjectsView } = await import(
    "./__mocks__/ProjectsView.svelte"
  );
  return { default: ProjectsView };
});
vi.mock("../lib/views/DesignDetailView.svelte", async () => {
  const { default: DesignDetailView } = await import(
    "./__mocks__/DesignDetailView.svelte"
  );
  return { default: DesignDetailView };
});
vi.mock("../lib/views/DesignPrintView.svelte", async () => {
  const { default: DesignPrintView } = await import(
    "./__mocks__/DesignPrintView.svelte"
  );
  return { default: DesignPrintView };
});
vi.mock("../lib/views/ImportView.svelte", async () => {
  const { default: ImportView } = await import("./__mocks__/ImportView.svelte");
  return { default: ImportView };
});
vi.mock("../lib/views/TagsView.svelte", async () => {
  const { default: TagsView } = await import("./__mocks__/TagsView.svelte");
  return { default: TagsView };
});
vi.mock("../lib/views/BrowseView.svelte", async () => {
  const { default: BrowseView } = await import("./__mocks__/BrowseView.svelte");
  return { default: BrowseView };
});
vi.mock("../lib/views/AdminDesignersView.svelte", async () => {
  const { default: AdminDesignersView } = await import(
    "./__mocks__/AdminDesignersView.svelte"
  );
  return { default: AdminDesignersView };
});
vi.mock("../lib/views/AdminSourcesView.svelte", async () => {
  const { default: AdminSourcesView } = await import(
    "./__mocks__/AdminSourcesView.svelte"
  );
  return { default: AdminSourcesView };
});
vi.mock("../lib/views/AdminHoopsView.svelte", async () => {
  const { default: AdminHoopsView } = await import(
    "./__mocks__/AdminHoopsView.svelte"
  );
  return { default: AdminHoopsView };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Set location.hash and dispatch a hashchange event. */
function setHash(hash: string) {
  window.location.hash = hash;
  window.dispatchEvent(new HashChangeEvent("hashchange"));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("MainView.svelte", () => {
  beforeEach(() => {
    window.location.hash = "#/designs";
  });

  // --- Nav shell & brand ---------------------------------------------------

  it("renders the brand and nav shell", () => {
    render(MainView);

    // The brand appears in both the nav bar and the footer.
    expect(screen.getAllByText("Embroidery Catalogue")).toHaveLength(2);
    expect(screen.getByText("Browse")).toBeInTheDocument();
    expect(screen.getByText("Import")).toBeInTheDocument();
    expect(screen.getByText("Projects")).toBeInTheDocument();
    expect(screen.getByText("Help")).toBeInTheDocument();
  });

  it("renders the admin navigation links", () => {
    render(MainView);

    expect(screen.getByText("Admin:")).toBeInTheDocument();
    expect(screen.getByText("Designers")).toBeInTheDocument();
    expect(screen.getByText("Tags")).toBeInTheDocument();
    expect(screen.getByText("Sources")).toBeInTheDocument();
    expect(screen.getByText("Hoops")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
    expect(screen.getByText("Backup")).toBeInTheDocument();
    expect(screen.getByText("Tagging Actions")).toBeInTheDocument();
    expect(screen.getByText("Orphans")).toBeInTheDocument();
  });

  // --- Main view routing ----------------------------------------------------

  it("renders BrowseView for the default route", () => {
    render(MainView);

    expect(screen.getByTestId("browse-view")).toBeInTheDocument();
    expect(screen.queryByTestId("import-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("projects-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("help-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("settings-view")).not.toBeInTheDocument();
  });

  it("renders ImportView for #/import", () => {
    setHash("#/import");
    render(MainView);

    expect(screen.getByTestId("import-view")).toBeInTheDocument();
    expect(screen.queryByTestId("browse-view")).not.toBeInTheDocument();
  });

  it("renders ProjectsView for #/projects with kind projects-list", () => {
    setHash("#/projects");
    render(MainView);

    const projectsView = screen.getByTestId("projects-view");
    expect(projectsView).toBeInTheDocument();
    expect(projectsView).toHaveAttribute("data-kind", "projects-list");
  });

  it("renders ProjectsView for #/projects/new with kind project-new", () => {
    setHash("#/projects/new");
    render(MainView);

    const projectsView = screen.getByTestId("projects-view");
    expect(projectsView).toHaveAttribute("data-kind", "project-new");
  });

  it("renders ProjectsView for #/projects/123 with kind project-detail and id 123", () => {
    setHash("#/projects/123");
    render(MainView);

    const projectsView = screen.getByTestId("projects-view");
    expect(projectsView).toHaveAttribute("data-kind", "project-detail");
    expect(projectsView).toHaveAttribute("data-detail-id", "123");
  });

  it("renders ProjectsView for #/projects/123/print with kind project-print and id 123", () => {
    setHash("#/projects/123/print");
    render(MainView);

    const projectsView = screen.getByTestId("projects-view");
    expect(projectsView).toHaveAttribute("data-kind", "project-print");
    expect(projectsView).toHaveAttribute("data-print-id", "123");
  });

  it("renders DesignDetailView for #/designs/123", () => {
    setHash("#/designs/123");
    render(MainView);

    const detailView = screen.getByTestId("design-detail-view");
    expect(detailView).toBeInTheDocument();
    expect(detailView).toHaveAttribute("data-design-id", "123");
    expect(screen.queryByTestId("browse-view")).not.toBeInTheDocument();
  });

  it("renders DesignPrintView for #/designs/123/print", () => {
    setHash("#/designs/123/print");
    render(MainView);

    const printView = screen.getByTestId("design-print-view");
    expect(printView).toBeInTheDocument();
    expect(printView).toHaveAttribute("data-print-id", "123");
    expect(screen.queryByTestId("design-detail-view")).not.toBeInTheDocument();
  });

  // --- Admin views ----------------------------------------------------------

  it("renders SettingsView for #/admin/settings", () => {
    setHash("#/admin/settings");
    render(MainView);

    expect(screen.getByTestId("settings-view")).toBeInTheDocument();
    expect(screen.queryByTestId("browse-view")).not.toBeInTheDocument();
  });

  it("renders BackupView for #/admin/maintenance/backup", () => {
    setHash("#/admin/maintenance/backup");
    render(MainView);

    expect(screen.getByTestId("backup-view")).toBeInTheDocument();
  });

  it("renders TaggingActionsView for #/admin/tagging-actions", () => {
    setHash("#/admin/tagging-actions");
    render(MainView);

    expect(screen.getByTestId("tagging-actions-view")).toBeInTheDocument();
  });

  it("renders OrphansView for #/admin/orphans", () => {
    setHash("#/admin/orphans");
    render(MainView);

    expect(screen.getByTestId("orphans-view")).toBeInTheDocument();
  });

  it("renders AdminDesignersView for #/admin/designers", () => {
    setHash("#/admin/designers");
    render(MainView);

    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
  });

  it("renders TagsView for #/admin/tags", () => {
    setHash("#/admin/tags");
    render(MainView);

    expect(screen.getByTestId("tags-view")).toBeInTheDocument();
  });

  it("renders AdminSourcesView for #/admin/sources", () => {
    setHash("#/admin/sources");
    render(MainView);

    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();
  });

  it("renders AdminHoopsView for #/admin/hoops", () => {
    setHash("#/admin/hoops");
    render(MainView);

    expect(screen.getByTestId("admin-hoops-view")).toBeInTheDocument();
  });

  // --- Static info views ------------------------------------------------------

  it("renders HelpView for #/help", () => {
    setHash("#/help");
    render(MainView);

    expect(screen.getByTestId("help-view")).toBeInTheDocument();
  });

  it("auto-scrolls to a recognised help section when ?section is provided", async () => {
    const sectionEl = document.createElement("div");
    sectionEl.id = "search";
    document.body.appendChild(sectionEl);
    const scrollIntoView = vi.fn();
    sectionEl.scrollIntoView = scrollIntoView;

    vi.useFakeTimers();
    setHash("#/help?section=search");
    render(MainView);

    await vi.runAllTimersAsync();

    expect(scrollIntoView).toHaveBeenCalledWith({
      behavior: "smooth",
      block: "start",
    });

    vi.useRealTimers();
    sectionEl.remove();
  });

  it("does not scroll when the help section id is unknown", async () => {
    vi.useFakeTimers();
    setHash("#/help?section=not-a-real-section");
    render(MainView);

    await vi.runAllTimersAsync();

    expect(screen.getByTestId("help-view")).toBeInTheDocument();

    vi.useRealTimers();
  });

  it("does not attempt scrolling for non-help routes with query strings", async () => {
    const scrollIntoView = vi.fn();
    HTMLElement.prototype.scrollIntoView = scrollIntoView;

    setHash("#/designs?page=2");
    render(MainView);

    await new Promise((resolve) => setTimeout(resolve, 200));

    expect(scrollIntoView).not.toHaveBeenCalled();
    expect(screen.getByTestId("browse-view")).toBeInTheDocument();
  });

  it("handles a help section whose DOM element is missing", async () => {
    vi.useFakeTimers();
    setHash("#/help?section=troubleshooting");
    render(MainView);

    await vi.runAllTimersAsync();

    expect(screen.getByTestId("help-view")).toBeInTheDocument();
    vi.useRealTimers();
  });

  it("renders AboutView for #/about", () => {
    setHash("#/about");
    render(MainView);

    expect(screen.getByTestId("about-view")).toBeInTheDocument();
  });

  it("renders AboutDocumentView with the licence slug", () => {
    setHash("#/about/document/licence");
    render(MainView);

    expect(screen.getByTestId("about-document-view")).toBeInTheDocument();
    expect(screen.getByTestId("about-document-slug")).toHaveTextContent(
      "licence"
    );
  });

  // --- Fallback / route not found ---------------------------------------------

  it("renders Route Not Found for an unknown route", () => {
    setHash("#/unknown/route");
    render(MainView);

    expect(screen.getByText("Route Not Found")).toBeInTheDocument();
    expect(
      screen.getByText(
        "The requested route does not exist. Use one of the known placeholders below."
      )
    ).toBeInTheDocument();
    expect(screen.getByText("Known routes")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Go to Browse" })).toBeInTheDocument();
    // A sampling of the known routes should be listed
    expect(screen.getByText("#/designs")).toBeInTheDocument();
    expect(screen.getByText("#/import")).toBeInTheDocument();
    expect(screen.getByText("#/about")).toBeInTheDocument();
  });

  it("navigates to Browse when the fallback Go to Browse button is clicked", async () => {
    setHash("#/unknown/route");
    render(MainView);

    await fireEvent.click(
      screen.getByRole("button", { name: "Go to Browse" })
    );

    expect(window.location.hash).toBe("#/designs");
    await waitFor(() => {
      expect(screen.getByTestId("browse-view")).toBeInTheDocument();
    });
  });

  // --- Footer ------------------------------------------------------------------

  it("renders the footer links", () => {
    render(MainView);

    const licenceLink = screen.getByRole("link", { name: "Licence" });
    expect(licenceLink).toHaveAttribute("href", "#/about/document/licence");
    expect(screen.getByRole("link", { name: "About" })).toHaveAttribute(
      "href",
      "#/about"
    );
  });

  // --- Navigation & link styling -------------------------------------------------

  it.each([
    ["#/designs", "Browse"],
    ["#/import", "Import"],
    ["#/projects", "Projects"],
    ["#/help", "Help"],
  ])("marks nav link %s active when route matches", (route, label) => {
    setHash(route);
    render(MainView);

    const link = screen.getByRole("link", { name: label });
    expect(link).toHaveClass("menu-link-active");
  });

  it.each([
    ["#/admin/designers", "Designers"],
    ["#/admin/tags", "Tags"],
    ["#/admin/sources", "Sources"],
    ["#/admin/hoops", "Hoops"],
    ["#/admin/settings", "Settings"],
    ["#/admin/maintenance/backup", "Backup"],
    ["#/admin/tagging-actions", "Tagging Actions"],
    ["#/admin/orphans", "Orphans"],
  ])("marks admin link %s active when route matches", (route, label) => {
    setHash(route);
    render(MainView);

    const link = screen.getByRole("link", { name: label });
    expect(link).toHaveClass("menu-link-active");
    expect(link).toHaveClass("menu-link-admin");
  });

  it("does not mark links active on a non-matching route", () => {
    setHash("#/designs");
    render(MainView);

    const projectsLink = screen.getByRole("link", { name: "Projects" });
    expect(projectsLink).not.toHaveClass("menu-link-active");
  });

  it("treats #/import/step1 as an active import link", () => {
    setHash("#/import/step1");
    render(MainView);

    const importLink = screen.getByRole("link", { name: "Import" });
    expect(importLink).toHaveClass("menu-link-active");
    expect(screen.getByTestId("import-view")).toBeInTheDocument();
  });

  // --- Import completion callback ----------------------------------------------

  it("sets browseNeedsRefresh when import completes with items", async () => {
    setHash("#/import");
    render(MainView);

    expect(screen.queryByTestId("browse-needs-refresh")).not.toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("import-completed"));

    // Navigate to browse and verify the refresh flag was hoisted.
    setHash("#/designs");
    await waitFor(() => {
      expect(screen.getByTestId("browse-needs-refresh")).toBeInTheDocument();
    });
  });

  it("does not set browseNeedsRefresh when import completes with zero items", async () => {
    setHash("#/import");
    render(MainView);

    await fireEvent.click(screen.getByTestId("import-zero"));

    setHash("#/designs");
    await waitFor(() => {
      expect(screen.getByTestId("browse-view")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("browse-needs-refresh")).not.toBeInTheDocument();
  });

  // --- Design deletion callback ---------------------------------------------------

  it("sets browseNeedsRefresh when a design is deleted", async () => {
    setHash("#/designs/123");
    render(MainView);

    await fireEvent.click(screen.getByTestId("design-deleted"));

    setHash("#/designs");
    await waitFor(() => {
      expect(screen.getByTestId("browse-needs-refresh")).toBeInTheDocument();
    });
  });

  // --- Hash change handling --------------------------------------------------------

  it("reacts to external hashchange events", async () => {
    render(MainView);
    expect(screen.getByTestId("browse-view")).toBeInTheDocument();

    setHash("#/admin/settings");

    await waitFor(() => {
      expect(screen.getByTestId("settings-view")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("browse-view")).not.toBeInTheDocument();
  });

  it("switches from settings back to browse on hashchange", async () => {
    setHash("#/admin/settings");
    render(MainView);
    expect(screen.getByTestId("settings-view")).toBeInTheDocument();

    setHash("#/designs");

    await waitFor(() => {
      expect(screen.getByTestId("browse-view")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("settings-view")).not.toBeInTheDocument();
  });
});