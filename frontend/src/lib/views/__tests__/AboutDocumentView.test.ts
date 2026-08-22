import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { tick } from "svelte";
import AboutDocumentView from "../AboutDocumentView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter module — this prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMock = vi.hoisted(() => ({
  getAboutDocument: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMock);

// ---------------------------------------------------------------------------
// Mock the statically-imported licence assets so tests are self-contained and
// deterministic (and so behaviour is covered even when the assets are empty,
// satisfying the "no build/lint errors if assets are empty" requirement).
// The fixture values live inside vi.hoisted() because vi.mock factories are
// hoisted above any top-level const declarations in this file.
// ---------------------------------------------------------------------------
const assetMocks = vi.hoisted(() => ({
  WHIPPED_LICENCE_HTML:
    '<details class="license-card"><summary class="license-header"><span class="license-name">Apache License 2.0</span><span class="license-used-by">Crates using this license:</span></summary><div class="license-crates"><h4>Used by:</h4><ul><li><strong>tokio</strong> (v1.0) — <a href="https://github.com/tokio-rs/tokio" target="_blank" rel="noopener">https://github.com/tokio-rs/tokio</a></li></ul></div><div class="license-text"><pre>Apache license text.</pre></div></details>',
  WHIPPED_APP_LICENCE:
    "GNU AFFERO GENERAL PUBLIC LICENSE\nVersion 3, 19 November 2007\n\nCopyright (c) 2026",
  WHIPPED_NPM_LICENCES: {
    "@esbuild/win32-x64@0.25.12": {
      licenses: "MIT",
      repository: "https://github.com/evanw/esbuild",
      path: "C:\\node_modules\\@esbuild\\win32-x64",
      licenseFile: "C:\\node_modules\\@esbuild\\win32-x64\\README.md",
    },
    "svelte@5.56.9": {
      licenses: "MIT",
      repository: "https://github.com/sveltejs/svelte",
      publisher: "Rich Harris",
      path: "C:\\node_modules\\svelte",
      licenseFile: "C:\\node_modules\\svelte\\LICENSE",
    },
  },
}));

vi.mock("../../../LICENSE?raw", () => ({
  default: assetMocks.WHIPPED_APP_LICENCE,
}));
vi.mock("../../assets/licences.html?raw", () => ({
  default: assetMocks.WHIPPED_LICENCE_HTML,
}));
vi.mock("../../assets/npm-licences.json", () => ({
  default: assetMocks.WHIPPED_NPM_LICENCES,
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Type-guard helper so querySelector results can be used as HTMLElements. */
function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

describe("AboutDocumentView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ---------------------------------------------------------------------
  // Dedicated Licence view (slug "licence")
  // ---------------------------------------------------------------------
  describe("dedicated licence view", () => {
    it("does not call getAboutDocument for the licence slug", () => {
      render(AboutDocumentView, { props: { slug: "licence" } });

      expect(adapterMock.getAboutDocument).not.toHaveBeenCalled();
    });

    it("renders the three licence tab buttons", () => {
      render(AboutDocumentView, { props: { slug: "licence" } });

      expect(screen.getByRole("button", { name: "Application Licence" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Rust Dependencies" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Frontend Dependencies" })).toBeInTheDocument();
    });

    it("shows the Application Licence tab by default with the primary licence text", () => {
      render(AboutDocumentView, { props: { slug: "licence" } });

      expect(screen.getByTestId("licence-application-tab")).toBeInTheDocument();

      const pre = element(
        document.querySelector("pre.licence-primary-text"),
        "Expected the primary licence <pre> block."
      );
      expect(pre.textContent).toContain("GNU AFFERO GENERAL PUBLIC LICENSE");
    });

    it("switches to the Rust Dependencies tab and renders the sanitized rust licence HTML", async () => {
      render(AboutDocumentView, { props: { slug: "licence" } });

      await fireEvent.click(screen.getByRole("button", { name: "Rust Dependencies" }));
      await tick();

      expect(screen.getByTestId("licence-rust-tab")).toBeInTheDocument();
      expect(screen.queryByTestId("licence-application-tab")).not.toBeInTheDocument();

      // The injected {@html} is not Svelte-scoped, so query the raw DOM.
      const rustList = element(
        document.querySelector(".licence-rust-list"),
        "Expected the rust licence list container."
      );
      expect(rustList.textContent).toContain("Apache License 2.0");
      expect(rustList.textContent).toContain("tokio");
      expect(rustList.querySelector("a")).toHaveAttribute(
        "href",
        "https://github.com/tokio-rs/tokio"
      );
    });

    it("switches to the Frontend Dependencies tab and renders package details cards", async () => {
      render(AboutDocumentView, { props: { slug: "licence" } });

      await fireEvent.click(screen.getByRole("button", { name: "Frontend Dependencies" }));
      await tick();

      const tab = screen.getByTestId("licence-frontend-tab");
      expect(tab).toBeInTheDocument();

      const svelteSummary = element(
        within(tab).getByText("svelte", { selector: "span.font-medium" }).closest("summary"),
        "Expected the svelte licence summary."
      );
      expect(svelteSummary.textContent).toContain("v5.56.9");

      const esbuildSummary = element(
        within(tab)
          .getByText("@esbuild/win32-x64", { selector: "span.font-medium" })
          .closest("summary"),
        "Expected the esbuild licence summary."
      );
      expect(esbuildSummary.textContent).toContain("v0.25.12");

      // license badges (one badge span per package)
      expect(within(tab).getAllByText("MIT", { selector: "span" })).toHaveLength(2);

      // repository links (target=_blank)
      const repoLinks = within(tab).getAllByRole("link", { name: /github.com/ });
      expect(repoLinks).toHaveLength(2);
      expect(repoLinks[0]).toHaveAttribute("href", "https://github.com/evanw/esbuild");
      expect(repoLinks[1]).toHaveAttribute("href", "https://github.com/sveltejs/svelte");
    });

    it("renders the Rust empty fallback when the rust HTML asset is empty", async () => {
      // The rows are computed at module scope, so re-import the component with
      // an empty HTML asset to exercise the guarded fallback. `vi.resetModules()`
      // re-evaluates the `svelte` runtime, so we must also re-import the
      // test harness pieces (render + tick) from the same module graph to avoid
      // the Svelte 5 `effect_orphan` dual-instance error.
      vi.resetModules();
      vi.doMock("../../assets/licences.html?raw", () => ({ default: "" }));

      const { render: renderLazy } = await import("@testing-library/svelte");
      const { tick: tickLazy } = await import("svelte");
      const { default: AboutDocumentViewLazy } = await import("../AboutDocumentView.svelte");
      const view = renderLazy(AboutDocumentViewLazy, { props: { slug: "licence" } });

      await fireEvent.click(screen.getByRole("button", { name: "Rust Dependencies" }));
      await tickLazy();

      expect(screen.getByTestId("licence-rust-tab")).toBeInTheDocument();
      expect(screen.getByText("No Rust licence data is available.")).toBeInTheDocument();

      view.unmount();
    });

    it("renders the frontend empty fallback when the npm JSON asset is empty", async () => {
      vi.resetModules();
      vi.doMock("../../assets/npm-licences.json", () => ({ default: {} }));

      const { render: renderLazy } = await import("@testing-library/svelte");
      const { tick: tickLazy } = await import("svelte");
      const { default: AboutDocumentViewLazy } = await import("../AboutDocumentView.svelte");
      const view = renderLazy(AboutDocumentViewLazy, { props: { slug: "licence" } });

      await fireEvent.click(screen.getByRole("button", { name: "Frontend Dependencies" }));
      await tickLazy();

      expect(screen.getByTestId("licence-frontend-tab")).toBeInTheDocument();
      expect(screen.getByTestId("frontend-licences-empty")).toHaveTextContent(
        "No frontend packages recorded."
      );

      view.unmount();
    });
  });

  // ---------------------------------------------------------------------
  // Generic document loading (non-licence slugs)
  // ---------------------------------------------------------------------
  describe("loading state", () => {
    it("shows the loading message while getAboutDocument is pending", async () => {
      // Keep promise pending so it stays in loading state
      adapterMock.getAboutDocument.mockReturnValue(new Promise(() => {}));

      render(AboutDocumentView, { props: { slug: "privacy" } });

      expect(screen.getByText("Loading document...")).toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();

      // Wait one tick
      await Promise.resolve();
    });
  });

  describe("successful document loading", () => {
    it("renders document content as plain text in a pre element by default", async () => {
      const mockDoc = {
        slug: "privacy",
        title: "Privacy",
        description: "The privacy terms.",
        filename: "PRIVACY",
        document_text: "We store your catalogue data locally.",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const pre = element(container.querySelector("pre"));
      expect(pre).toHaveTextContent("We store your catalogue data locally.");
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
        slug: "privacy",
        title: "Privacy",
        description: "The privacy terms.",
        filename: "PRIVACY",
        document_text: null,
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.getByText("Document content is unavailable.")).toBeInTheDocument();
      });
    });

    it("renders API error message when the getAboutDocument returns an error field", async () => {
      adapterMock.getAboutDocument.mockResolvedValue({
        error: "Database error reading privacy document.",
        item: null,
      });

      render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.getByText("Database error reading privacy document.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders not-found fallback when getAboutDocument returns an empty payload", async () => {
      adapterMock.getAboutDocument.mockResolvedValue({ item: null });

      render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.getByText("Document not found.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders not-found fallback when getAboutDocument resolves to null", async () => {
      adapterMock.getAboutDocument.mockResolvedValue(null);

      render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.getByText("Document not found.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      expect(screen.queryByText("Document content is unavailable.")).not.toBeInTheDocument();
    });

    it("renders fallback error message when the getAboutDocument promise rejects", async () => {
      adapterMock.getAboutDocument.mockRejectedValue(new Error("Connection refused"));

      render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(
          screen.getByText("Could not load document: Error: Connection refused")
        ).toBeInTheDocument();
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
        slug: "privacy",
        title: "Privacy",
        description: "The privacy terms.",
        document_text: "We store your catalogue data locally.",
      };
      adapterMock.getAboutDocument.mockResolvedValue({ item: mockDoc });

      const { container } = render(AboutDocumentView, { props: { slug: "privacy" } });

      await waitFor(() => {
        expect(screen.queryByText("Loading document...")).not.toBeInTheDocument();
      });

      const pre = element(container.querySelector("pre"));
      expect(pre).toHaveTextContent("We store your catalogue data locally.");
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
      let resolveFirst: (value: unknown) => void = () => {};
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
