import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import AboutDocumentView from "../AboutDocumentView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter module — this prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMock = vi.hoisted(() => ({
  getAboutDocument: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMock);

// ---------------------------------------------------------------------------
// Helpers
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

describe("AboutDocumentView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("loading state", () => {
    it("shows the loading message while getAboutDocument is pending", async () => {
      // Keep promise pending so it stays in loading state
      adapterMock.getAboutDocument.mockReturnValue(new Promise(() => {}));

      render(AboutDocumentView, { props: { slug: "licence" } });

      expect(screen.getByText("Loading document...")).toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
      
      // Wait one tick
      await Promise.resolve();
    });
  });

  describe("successful document loading", () => {
    it("renders document content as plain text in a pre element by default", async () => {
      const mockDoc = {
        slug: "licence",
        title: "Licence",
        description: "The licence terms.",
        filename: "LICENCE",
        document_text: "Copyright (c) 2026 Rust Embroidery Catalogue Developers",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const pre = element(container.querySelector("pre"));
      expect(pre).toHaveTextContent("Copyright (c) 2026 Rust Embroidery Catalogue Developers");
      expect(pre).toHaveClass("whitespace-pre-wrap", "font-mono");
    });

    it("renders document content as HTML in a div when slug is disclaimer", async () => {
      const mockDoc = {
        slug: "disclaimer",
        title: "Disclaimer",
        description: "Warning message.",
        filename: "DISCLAIMER",
        document_text: "<strong>Warning:</strong> Use at your own risk.",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "disclaimer" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const div = element(container.querySelector("div.text-sm.text-gray-700.bg-gray-50"));
      expect(div.innerHTML).toContain("<strong>Warning:</strong> Use at your own risk.");
      expect(container.querySelector("pre")).not.toBeInTheDocument();
    });

    it("renders document content as HTML in a div when filename ends with .html", async () => {
      const mockDoc = {
        slug: "privacy",
        title: "Privacy Policy",
        description: "Privacy details.",
        filename: "privacy.html",
        document_text: "<p>We do not track you.</p>",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const div = element(container.querySelector("div.text-sm.text-gray-700.bg-gray-50"));
      expect(div.innerHTML).toContain("<p>We do not track you.</p>");
      expect(container.querySelector("pre")).not.toBeInTheDocument();
    });

    it("renders Markdown content as styled HTML and strips relative links for the ai-tagging slug", async () => {
      const mockDoc = {
        slug: "ai-tagging",
        title: "AI Tagging Guide",
        description: "Enable optional AI tagging.",
        filename: "docs/User-Facing-Guidance/AI_TAGGING.md",
        document_text:
          "# AI-Assisted Auto-Tagging\n\nSee [STITCH_TYPES.md](STITCH_TYPES.md) and " +
          "[current pricing](https://ai.google.dev/pricing).",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "ai-tagging" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const div = element(container.querySelector("div.prose"));
      expect(div.querySelector("h1")).toHaveTextContent("AI-Assisted Auto-Tagging");
      // Relative .md links are neutralized to plain text (no dead links).
      expect(div.querySelector('a[href="STITCH_TYPES.md"]')).not.toBeInTheDocument();
      expect(screen.getByText(/STITCH_TYPES\.md/)).toBeInTheDocument();
      // Absolute URLs remain as links.
      expect(div.querySelector('a[href="https://ai.google.dev/pricing"]')).toBeInTheDocument();
      expect(container.querySelector("pre")).not.toBeInTheDocument();
    });
  });

  describe("error handling and empty states", () => {
    it("renders document unavailable text when document_text is missing", async () => {
      const mockDoc = {
        slug: "licence",
        title: "Licence",
        description: "The licence terms.",
        filename: "LICENCE",
        document_text: null,
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.getByText("Document content is unavailable.")).toBeInTheDocument();
      });
    });

    it("renders API error message when the getAboutDocument returns an error field", async () => {
      adapterMock.getAboutDocument.mockResolvedValue({
        error: "Database error reading licence.",
        item: null,
      });

      render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.getByText("Database error reading licence.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders not-found fallback when getAboutDocument returns an empty payload", async () => {
      adapterMock.getAboutDocument.mockResolvedValue({ item: null });

      render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.getByText("Document not found.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders not-found fallback when getAboutDocument resolves to null", async () => {
      adapterMock.getAboutDocument.mockResolvedValue(null);

      render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.getByText("Document not found.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders fallback error message when the getAboutDocument promise rejects", async () => {
      adapterMock.getAboutDocument.mockRejectedValue(new Error("Connection refused"));

      render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.getByText("Could not load document: Error: Connection refused")).toBeInTheDocument();
      });
    });
  });

  describe("shouldRenderAsHtml fallback branches", () => {
    it("renders as HTML when slug is missing but filename ends with .html", async () => {
      const mockDoc = {
        title: "Privacy",
        description: "Privacy details.",
        filename: "privacy.html",
        document_text: "<p>We do not track you.</p>",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const div = element(container.querySelector("div.text-sm.text-gray-700.bg-gray-50"));
      expect(div.innerHTML).toContain("<p>We do not track you.</p>");
      expect(container.querySelector("pre")).not.toBeInTheDocument();
    });

    it("renders as plain text when filename is missing", async () => {
      const mockDoc = {
        slug: "licence",
        title: "Licence",
        description: "The licence terms.",
        document_text: "Copyright (c) 2026 Rust Embroidery Catalogue Developers",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "licence" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const pre = element(container.querySelector("pre"));
      expect(pre).toHaveTextContent("Copyright (c) 2026 Rust Embroidery Catalogue Developers");
      expect(pre).toHaveClass("whitespace-pre-wrap", "font-mono");
    });
  });

  describe("whitespace-only slug handling", () => {
    it("renders document-not-found error when slug is whitespace-only", async () => {
      render(AboutDocumentView, { props: { slug: "   " } });

      // The onMount guard `if (slug)` passes because "   " is truthy,
      // so loadAboutDocumentView("   ") runs and its internal guard
      // normalizes the slug to "" and sets the not-found error.
      await waitFor(() => {
        expect(screen.getByText("Document not found.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(adapterMock.getAboutDocument).not.toHaveBeenCalled();
    });
  });

  describe("race conditions / parameter updates", () => {
    it("ignores results from older requests if the slug prop changes before resolution", async () => {
      let resolveFirst: (value: any) => void = () => {};
      const firstPromise = new Promise((resolve) => {
        resolveFirst = resolve;
      });

      adapterMock.getAboutDocument.mockImplementation((slugArg: string) => {
        if (slugArg === "first") {
          return firstPromise;
        }
        return Promise.resolve({
          item: {
            slug: "second",
            title: "Second",
            filename: "second",
            document_text: "Content of the second document",
          },
        });
      });

      // Render with first slug. This calls loadAboutDocumentView("first") in onMount.
      const { rerender } = render(AboutDocumentView, { props: { slug: "first" } });

      // Change slug to "second". This updates the reactive `slug` prop to "second",
      // but does not auto-trigger loadAboutDocumentView since it only runs onMount.
      await rerender({ slug: "second" });

      // Now resolve the first one which should be ignored because its normalizedSlug ("first")
      // does not match the current slug prop ("second").
      resolveFirst({
        item: {
          slug: "first",
          title: "First",
          filename: "first",
          document_text: "Content of the first document",
        },
      });

      // Wait for the first promise's resolution flow to finish
      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      // It should NOT render the first document's content
      expect(screen.queryByText("Content of the first document")).not.toBeInTheDocument();
      expect(screen.getByText("Document content is unavailable.")).toBeInTheDocument();
    });

    it("does not trigger loader or request when slug is empty", async () => {
      render(AboutDocumentView, { props: { slug: "" } });

      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(screen.getByText("Document content is unavailable.")).toBeInTheDocument();
      expect(adapterMock.getAboutDocument).not.toHaveBeenCalled();
    });
  });
});
