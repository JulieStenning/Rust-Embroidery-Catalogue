import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import type { DesignDetail } from "../../types/ipc";
import DesignPrintView from "../DesignPrintView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter module — this prevents real Tauri `invoke` calls
// from being executed during tests. DesignPrintView only uses getDesignDetail.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getDesignDetail: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/** A fully-populated DesignDetail returned by the mocked adapter. */
const baseDetail = {
  id: 42,
  filename: "rose-border-01.pes",
  filepath: "C:/designs/rose-border-01.pes",
  imageType: null,
  imageDataUrl: "data:image/png;base64,AAAA",
  widthMm: 120,
  heightMm: 80,
  stitchCount: 10000,
  colorCount: 5,
  colorChangeCount: 12,
  designer: "Rose Studio",
  designerId: 7,
  source: "Imported",
  sourceId: 3,
  hoop: "Hoop A",
  hoopId: 1,
  notes: "Pretty floral border with satin stitches.",
  rating: 4,
  isStitched: true,
  imageTagsVerified: true,
  stitchingTagsVerified: true,
  taggingMode: "ai_vision",
  dateAdded: "2026-05-01",
  tags: [
    { id: 11, description: "Floral", tag_group: "image" },
    { id: 12, description: "Satin Stitch", tag_group: "stitching" },
    { id: 13, description: "Blue", tag_group: null },
  ],
  projects: [],
  availableProjects: [],
  allTags: [],
  designers: [],
  sources: [],
  hoops: [],
} satisfies DesignDetail;

/** Helper that constructs an AdapterItemResponse for getDesignDetail. */
function detailResponse(overrides: Partial<DesignDetail> = {}) {
  return { source: "rust", item: { ...baseDetail, ...overrides } };
}

/** Helper to render the component with sensible default props. */
function renderPrintView(props: Record<string, unknown> = {}) {
  return render(DesignPrintView, {
    props: {
      printDesignId: 42,
      navigateTo: () => {},
      ...props,
    },
  });
}

describe("DesignPrintView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getDesignDetail.mockResolvedValue(detailResponse());
  });

  describe("loading state", () => {
    it("shows the loading message while getDesignDetail is pending", async () => {
      // Never resolve — the loader stays visible.
      adapterMocks.getDesignDetail.mockReturnValue(new Promise(() => {}));

      renderPrintView();

      expect(screen.getByText("Loading printable design detail...")).toBeInTheDocument();
      // Wait one tick so any synchronous flush doesn't leave stray pending state.
      await Promise.resolve();
    });
  });

  describe("error state", () => {
    it("renders the backend error message when item is null and error is present", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue({
        source: "rust",
        item: null,
        error: "Design 999 was deleted or is missing.",
      });

      renderPrintView({ printDesignId: 999 });

      await waitFor(() => {
        expect(
          screen.getByText(/Could not load design detail: Design 999 was deleted or is missing\./)
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading printable design detail...")).not.toBeInTheDocument();
    });

    it("renders a generic error when the adapter throws", async () => {
      adapterMocks.getDesignDetail.mockRejectedValue(new Error("network down"));

      renderPrintView();

      await waitFor(() => {
        expect(
          screen.getByText(/Could not load design detail: Error: network down/)
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading printable design detail...")).not.toBeInTheDocument();
    });

    it("renders the detail when the adapter returns an item even with an error attached", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue({
        source: "rust",
        item: baseDetail,
        error: "Non-fatal warning",
      });

      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByText(/Could not load design detail/)).not.toBeInTheDocument();
    });
  });

  describe("empty / null design", () => {
    it("shows 'No design found for id X' when the adapter returns null without an error", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue({ source: "rust", item: null });

      renderPrintView({ printDesignId: 123 });

      await waitFor(() => {
        expect(screen.getByText("No design found for id 123.")).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading printable design detail...")).not.toBeInTheDocument();
    });
  });

  describe("detail loading", () => {
    it("requests the design detail for the mounted print id", async () => {
      renderPrintView({ printDesignId: 7 });

      await waitFor(() => {
        expect(adapterMocks.getDesignDetail).toHaveBeenCalledWith(7);
      });
    });

    it("does not request a detail when printDesignId is null", async () => {
      renderPrintView({ printDesignId: null });

      await Promise.resolve();
      expect(adapterMocks.getDesignDetail).not.toHaveBeenCalled();
    });

    it("reloads the detail when printDesignId changes", async () => {
      const view = renderPrintView({ printDesignId: 1 });

      await waitFor(() => {
        expect(adapterMocks.getDesignDetail).toHaveBeenCalledWith(1);
      });

      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({ id: 2, filename: "holiday-tree.vp3" })
      );
      await view.rerender({ printDesignId: 2 });

      await waitFor(() => {
        expect(adapterMocks.getDesignDetail).toHaveBeenCalledWith(2);
      });
      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "holiday-tree.vp3" })).toBeInTheDocument();
      });
      expect(screen.queryByRole("heading", { name: "rose-border-01.pes" })).not.toBeInTheDocument();
    });
  });

  describe("successful rendering", () => {
    it("renders the filename as the print heading", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading printable design detail...")).not.toBeInTheDocument();
    });

    it("renders the preview image when imageDataUrl is present", async () => {
      renderPrintView();

      await waitFor(() => {
        const image = screen.getByRole("img", { name: "rose-border-01.pes" });
        expect(image).toHaveAttribute("src", "data:image/png;base64,AAAA");
      });
    });

    it("renders the file, designer, source, hoop, and date rows", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("File:")).toBeInTheDocument();
      });

      expect(screen.getByText("C:/designs/rose-border-01.pes")).toBeInTheDocument();
      expect(screen.getByText("Designer:")).toBeInTheDocument();
      expect(screen.getByText("Rose Studio")).toBeInTheDocument();
      expect(screen.getByText("Source:")).toBeInTheDocument();
      expect(screen.getByText("Imported")).toBeInTheDocument();
      expect(screen.getByText("Hoop:")).toBeInTheDocument();
      expect(screen.getByText("Hoop A")).toBeInTheDocument();
      expect(screen.getByText("Added:")).toBeInTheDocument();
      expect(screen.getByText("2026-05-01")).toBeInTheDocument();
    });

    it("renders the dimensions, stitches, colours, and colour change rows", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("Dimensions:")).toBeInTheDocument();
      });

      expect(screen.getByText("120 x 80 mm")).toBeInTheDocument();
      expect(screen.getByText("Stitches:")).toBeInTheDocument();
      expect(screen.getByText("10000")).toBeInTheDocument();
      expect(screen.getByText("Colours:")).toBeInTheDocument();
      expect(screen.getByText("5")).toBeInTheDocument();
      expect(screen.getByText("Colour changes:")).toBeInTheDocument();
      expect(screen.getByText("12")).toBeInTheDocument();
    });

    it("renders the Stitched row when the design is stitched", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("Stitched:")).toBeInTheDocument();
      });
      expect(screen.getByText("Yes")).toBeInTheDocument();
    });

    it("renders the rating stars for a rated design", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("Rating:")).toBeInTheDocument();
      });
      expect(screen.getByText("★★★★☆")).toBeInTheDocument();
    });

    it("clamps ratings above five to five stars", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ rating: 6 }));
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("★★★★★")).toBeInTheDocument();
      });
    });

    it("renders the notes section when notes are present", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("Notes")).toBeInTheDocument();
      });
      expect(screen.getByText("Pretty floral border with satin stitches.")).toBeInTheDocument();
    });

    it("renders the tags section with comma-joined descriptions", async () => {
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByText("Tags")).toBeInTheDocument();
      });
      expect(screen.getByText("Floral, Satin Stitch, Blue")).toBeInTheDocument();
    });
  });

  describe("optional / missing fields", () => {
    it("does not render an image when imageDataUrl is missing", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ imageDataUrl: null }));
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByRole("img")).not.toBeInTheDocument();
    });

    it("renders Unknown for missing text fields and ? for missing numeric fields", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({
          filepath: "",
          designer: "",
          source: "",
          hoop: null,
          widthMm: null,
          heightMm: null,
          stitchCount: null,
          colorCount: null,
          colorChangeCount: null,
          dateAdded: null,
        })
      );
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });

      // File, Designer, Source, Hoop, and Added all fall back to "Unknown".
      expect(screen.getAllByText("Unknown")).toHaveLength(5);
      expect(screen.getByText("? x ? mm")).toBeInTheDocument();
      // Stitches, Colours, and Colour changes all fall back to "?".
      expect(screen.getAllByText("?")).toHaveLength(3);
    });

    it.each([null, 0])(
      "does not render a rating section when the rating is %s",
      async (rating: number | null) => {
        adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ rating }));
        renderPrintView();

        await waitFor(() => {
          expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
        });
        expect(screen.queryByText("Rating:")).not.toBeInTheDocument();
      }
    );

    it("does not render the Stitched row when the design is not stitched", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ isStitched: false }));
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByText("Stitched:")).not.toBeInTheDocument();
    });

    it("does not render the notes section when notes are missing", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ notes: null }));
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByText("Notes")).not.toBeInTheDocument();
    });

    it("does not render the tags section when tags are empty", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(detailResponse({ tags: [] }));
      renderPrintView();

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });
      expect(screen.queryByText("Tags")).not.toBeInTheDocument();
    });
  });

  describe("action buttons", () => {
    it("navigates back to the detail page when Back to Detail is clicked", async () => {
      const navigateTo = vi.fn();
      renderPrintView({ navigateTo });

      await waitFor(() => {
        expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
      });

      await fireEvent.click(screen.getByRole("button", { name: "Back to Detail" }));
      expect(navigateTo).toHaveBeenCalledWith("#/designs/42");
    });

    it("calls window.print when Print is clicked", async () => {
      const printSpy = vi.spyOn(window, "print").mockImplementation(() => {});
      try {
        renderPrintView();

        await waitFor(() => {
          expect(screen.getByRole("heading", { name: "rose-border-01.pes" })).toBeInTheDocument();
        });

        await fireEvent.click(screen.getByRole("button", { name: "Print" }));
        expect(printSpy).toHaveBeenCalledTimes(1);
      } finally {
        printSpy.mockRestore();
      }
    });
  });
});
