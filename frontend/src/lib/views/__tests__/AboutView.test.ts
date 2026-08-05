import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import AboutView from "../AboutView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter module — this prevents real Tauri `invoke` calls
// from being executed during tests. Only the functions used by AboutView are
// stubbed so they can be asserted against.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getAboutDocuments: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — AboutView calls addToast() in its error path.
const toastMocks = vi.hoisted(() => ({
  addToast: vi.fn(),
}));

vi.mock("../../stores/toastStore.js", () => toastMocks);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/** Shape of a document returned by the mocked getAboutDocuments adapter. */
const mockDocs = [
  {
    slug: "licence",
    title: "Licence",
    description: "The licence terms for the Embroidery Catalogue project itself.",
    filename: "LICENCE",
    available: true,
  },
  {
    slug: "privacy",
    title: "Privacy",
    description: "Explains what data is stored locally.",
    filename: "templates/info/PRIVACY.html",
    available: false,
  },
  {
    slug: "disclaimer",
    title: "Disclaimer",
    description: "Important use-at-your-own-risk information.",
    filename: "DISCLAIMER.html",
    available: true,
  },
];

/** Wraps document items in the AdapterListResponse shape. */
const docsResponse = (items: unknown[] = []) => ({ source: "rust", items });

// ---------------------------------------------------------------------------
// Helpers (mirrors HelpView.test.ts)
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

describe("AboutView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getAboutDocuments.mockResolvedValue(docsResponse(mockDocs));
  });

  describe("page chrome", () => {
    beforeEach(() => {
      // Use an empty document list so the static "Licence" label does not
      // collide with a dynamically-rendered document title of the same name.
      adapterMocks.getAboutDocuments.mockResolvedValue(docsResponse());
    });

    it("renders the page heading 'About Embroidery Catalogue'", () => {
      render(AboutView);

      expect(
        screen.getByRole("heading", { name: "About Embroidery Catalogue" })
      ).toBeInTheDocument();
    });

    it("renders all four information section labels", () => {
      render(AboutView);

      expect(screen.getByText("What this app is")).toBeInTheDocument();
      expect(screen.getByText("Where data is stored")).toBeInTheDocument();
      expect(screen.getByText("AI / Gemini features")).toBeInTheDocument();
      expect(screen.getByText("A note on accuracy")).toBeInTheDocument();
    });

    it("renders the app description section content", () => {
      render(AboutView);

      expect(
        screen.getByText(
          normalizedText(
            "Embroidery Catalogue is a local, offline tool for cataloguing and browsing an embroidery design collection. It reads a broad range of embroidery formats directly via built-in readers, including .jef, .pes, .hus, .vp3, .dst, .exp, .sew, .u01, and many more, with limited support for .art. It stores all data in a local database file — no internet connection is required for normal use."
          )
        )
      ).toBeInTheDocument();
    });

    it("renders the data storage section content", () => {
      render(AboutView);

      expect(
        screen.getByText(
          normalizedText(
            "The catalogue database and any generated preview images are stored locally on your machine. Your embroidery files are not moved or modified — the catalogue only reads them to extract metadata and generate thumbnail previews."
          )
        )
      ).toBeInTheDocument();
    });

    it("renders the AI / Gemini section content with a link to Settings", () => {
      render(AboutView);

      expect(
        screen.getByText(
          normalizedText(
            "Some optional features use the Google Gemini API for automatic tagging and description generation. These features require an internet connection and a valid API key configured in Settings. Tier 2 (text AI) and Tier 3 (vision AI) tagging during import can be enabled or disabled independently in Settings. They are entirely optional and the catalogue works fully without them using Tier 1 keyword tagging."
          )
        )
      ).toBeInTheDocument();

      const settingsLink = screen.getByRole("link", { name: "Settings" });
      expect(settingsLink).toHaveAttribute("href", "#/admin/settings");
    });

    it("renders the accuracy section content", () => {
      render(AboutView);

      expect(
        screen.getByText(
          normalizedText(
            "Automatically generated tags and metadata should be treated as suggestions. Always verify results before relying on them, especially for important cataloguing decisions."
          )
        )
      ).toBeInTheDocument();
    });

    it("renders the licence notice box with AGPL text and an open link", () => {
      render(AboutView);

      // The "Licence" label in the notice box is a <p>, not a heading.
      expect(screen.getByText("Licence")).toBeInTheDocument();

      // The strong tag splits the sentence, so match normalized textContent.
      expect(
        screen.getByText(
          normalizedText("This repository is licensed under AGPL-3.0-or-later.")
        )
      ).toBeInTheDocument();

      const openLicenceLink = screen.getByRole("link", {
        name: /Open licence text/,
      });
      expect(openLicenceLink).toHaveAttribute("href", "#/about/document/licence");
    });

    it("renders the documents helper paragraph", () => {
      render(AboutView);

      expect(
        screen.getByText(
          "This page also provides quick access to the app's legal, privacy, security, and licensing documents."
        )
      ).toBeInTheDocument();
    });
  });

  describe("loading state", () => {
    it("shows the loading message while getAboutDocuments is pending", async () => {
      // Never resolve — the loader stays visible.
      adapterMocks.getAboutDocuments.mockReturnValue(new Promise(() => {}));

      render(AboutView);

      expect(screen.getByText("Loading documents...")).toBeInTheDocument();
      expect(screen.queryByText("No about documents are configured.")).not.toBeInTheDocument();
      // Wait one tick so any synchronous flush doesn't leave stray pending state.
      await Promise.resolve();
    });
  });

  describe("documents list", () => {
    it("shows the empty message when no documents are returned", async () => {
      adapterMocks.getAboutDocuments.mockResolvedValue(docsResponse());

      render(AboutView);
      expect(screen.getByText("Loading documents...")).toBeInTheDocument();

      await waitFor(() => {
        expect(
          screen.getByText("No about documents are configured.")
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading documents...")).not.toBeInTheDocument();
      expect(adapterMocks.getAboutDocuments).toHaveBeenCalledTimes(1);
    });

    it("treats a null response as an empty document list", async () => {
      adapterMocks.getAboutDocuments.mockResolvedValue(null);

      render(AboutView);

      await waitFor(() => {
        expect(
          screen.getByText("No about documents are configured.")
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading documents...")).not.toBeInTheDocument();
    });

    it("treats a non-array items field as an empty document list", async () => {
      adapterMocks.getAboutDocuments.mockResolvedValue({
        source: "rust",
        items: "not-an-array",
      });

      render(AboutView);

      await waitFor(() => {
        expect(
          screen.getByText("No about documents are configured.")
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading documents...")).not.toBeInTheDocument();
    });

    it("renders document titles, descriptions, Open links, and Not found labels", async () => {
      render(AboutView);

      // Document titles render as <h2> headings.
      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "Licence" })).toBeInTheDocument();
      });
      expect(screen.getByRole("heading", { name: "Privacy" })).toBeInTheDocument();
      expect(screen.getByRole("heading", { name: "Disclaimer" })).toBeInTheDocument();

      // Descriptions
      expect(
        screen.getByText("The licence terms for the Embroidery Catalogue project itself.")
      ).toBeInTheDocument();
      expect(screen.getByText("Explains what data is stored locally.")).toBeInTheDocument();
      expect(
        screen.getByText("Important use-at-your-own-risk information.")
      ).toBeInTheDocument();

      // Available docs get an "Open" link with the correct slash-prefixed path.
      const openLinks = screen.getAllByRole("link", { name: "Open" });
      expect(openLinks).toHaveLength(2);
      expect(openLinks[0]).toHaveAttribute("href", "#/about/document/licence");
      expect(openLinks[1]).toHaveAttribute("href", "#/about/document/disclaimer");

      // Unavailable docs get a "Not found" label instead of a link.
      expect(screen.getByText("Not found")).toBeInTheDocument();
      expect(screen.queryByText("Loading documents...")).not.toBeInTheDocument();
    });

    it("renders Not found when a document has no available flag", async () => {
      adapterMocks.getAboutDocuments.mockResolvedValue(
        docsResponse([
          {
            slug: "privacy",
            title: "Privacy",
            description: "Data details.",
            filename: "templates/info/PRIVACY.html",
          },
        ])
      );

      render(AboutView);

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "Privacy" })).toBeInTheDocument();
      });
      expect(screen.getByText("Data details.")).toBeInTheDocument();
      expect(screen.getByText("Not found")).toBeInTheDocument();
      expect(screen.queryByRole("link", { name: "Open" })).not.toBeInTheDocument();
    });
  });

  describe("error handling", () => {
    it("calls addToast with the error message and falls back to the empty state", async () => {
      adapterMocks.getAboutDocuments.mockRejectedValue(new Error("network down"));

      render(AboutView);
      expect(screen.getByText("Loading documents...")).toBeInTheDocument();

      await waitFor(() => {
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Could not load about documents: Error: network down",
          "error"
        );
      });

      expect(
        screen.getByText("No about documents are configured.")
      ).toBeInTheDocument();
      expect(screen.queryByText("Loading documents...")).not.toBeInTheDocument();
      expect(adapterMocks.getAboutDocuments).toHaveBeenCalledTimes(1);
    });
  });
});