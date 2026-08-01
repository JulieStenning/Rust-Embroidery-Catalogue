import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import DesignDetailView from "../DesignDetailView.svelte";

// Mock the command adapter module — this prevents real Tauri `invoke` calls
// from being executed during tests. All functions used by the component are
// stubbed so they can be asserted against.
const adapterMocks = vi.hoisted(() => ({
  getDesignDetail: vi.fn(),
  getDesignImageDataUrl: vi.fn(),
  updateDesignMetadata: vi.fn(),
  setDesignRating: vi.fn(),
  setDesignStitched: vi.fn(),
  setDesignTagsChecked: vi.fn(),
  setDesignTags: vi.fn(),
  removeDesignTag: vi.fn(),
  addDesignToProject: vi.fn(),
  removeDesignFromProject: vi.fn(),
  bulkDeleteDesigns: vi.fn(),
  openDesignInEditor: vi.fn(),
  openDesignInExplorer: vi.fn(),
  renderDesign3dPreview: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// ---------------------------------------------------------------------------
// Fixture: a fully-populated DesignDetail returned by the mocked adapter
// ---------------------------------------------------------------------------
const baseDetail = {
  id: 42,
  filename: "rose-border-01.pes",
  filepath: "C:/designs/rose-border-01.pes",
  imageType: null,
  imageDataUrl: null,
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
  tagsChecked: false,
  taggingTier: 2,
  dateAdded: "2026-05-01",
  tags: [
    { id: 11, description: "Floral", tag_group: "image" },
    { id: 12, description: "Satin Stitch", tag_group: "stitching" },
    { id: 13, description: "Blue", tag_group: null },
  ],
  projects: [{ id: 1, name: "Wedding Collection" }],
  availableProjects: [
    { id: 1, name: "Wedding Collection" },
    { id: 2, name: "Autumn 2026" },
  ],
  allTags: [
    { id: 11, description: "Floral", tag_group: "image" },
    { id: 12, description: "Satin Stitch", tag_group: "stitching" },
    { id: 13, description: "Blue", tag_group: null },
  ],
  designers: [{ id: 7, name: "Rose Studio" }],
  sources: [{ id: 3, name: "Imported" }],
  hoops: [{ id: 1, name: "Hoop A" }],
};

/** Helper that constructs an AdapterItemResponse for getDesignDetail. */
const detailResponse = (overrides = {}) => ({
  source: "rust",
  item: { ...baseDetail, ...overrides },
});

/** Helper to render the component with sensible defaults. */
const renderDetail = (props = {}) =>
  render(DesignDetailView, {
    props: {
      detailDesignId: 42,
      detailBrowseIds: [41, 42, 43],
      detailBrowseIndex: 1,
      navigateTo: () => {},
      onDesignDeleted: () => {},
      ...props,
    },
  });

describe("DesignDetailView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    adapterMocks.getDesignDetail.mockResolvedValue(detailResponse());
    adapterMocks.getDesignImageDataUrl.mockResolvedValue({
      source: "rust",
      item: null,
    });
    adapterMocks.updateDesignMetadata.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Metadata updated.",
    });
    adapterMocks.setDesignRating.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Rating updated.",
    });
    adapterMocks.setDesignStitched.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Stitched state updated.",
    });
    adapterMocks.setDesignTagsChecked.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Verification state updated.",
    });
    adapterMocks.setDesignTags.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Tags updated.",
    });
    adapterMocks.removeDesignTag.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Tag removed.",
    });
    adapterMocks.addDesignToProject.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Added to project.",
    });
    adapterMocks.removeDesignFromProject.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "Removed from project.",
    });
    adapterMocks.bulkDeleteDesigns.mockResolvedValue({
      source: "rust",
      persisted: true,
      deleted_count: 1,
      files_trashed: 0,
      errors: [],
    });
    adapterMocks.openDesignInEditor.mockResolvedValue({
      source: "rust",
      persisted: true,
      result: { success: true },
      message: "Opened in editor.",
    });
    adapterMocks.openDesignInExplorer.mockResolvedValue({
      source: "rust",
      persisted: true,
      result: { success: true },
      message: "Opened in explorer.",
    });
    adapterMocks.renderDesign3dPreview.mockResolvedValue({
      source: "rust",
      persisted: true,
      result: { success: true },
      message: "3D preview rendered.",
    });
  });

  describe("loading state", () => {
    it("shows the loading message while getDesignDetail is pending", async () => {
      // Never resolve — the loader stays visible.
      adapterMocks.getDesignDetail.mockReturnValue(new Promise(() => {}));

      renderDetail();

      expect(screen.getByText("Loading design detail...")).toBeInTheDocument();
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

      renderDetail({ detailDesignId: 999 });
      expect(screen.getByText("Loading design detail...")).toBeInTheDocument();

      await waitFor(() => {
        expect(
          screen.getByText(/Could not load design detail from Rust backend/)
        ).toBeInTheDocument();
      });
      expect(
        screen.getByText(
          /Could not load design detail from Rust backend: Design 999 was deleted or is missing\./
        )
      ).toBeInTheDocument();
    });

    it("renders a generic error when the adapter throws", async () => {
      adapterMocks.getDesignDetail.mockRejectedValue(new Error("network down"));

      renderDetail();

      await waitFor(() => {
        expect(
          screen.getByText(/Could not load design detail: Error: network down/)
        ).toBeInTheDocument();
      });
      expect(screen.queryByText("Loading design detail...")).not.toBeInTheDocument();
    });
  });

  describe("empty / null design", () => {
    it("shows 'No design found for id X' when the adapter returns null without error", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue({
        source: "rust",
        item: null,
        error: undefined,
      });

      renderDetail({ detailDesignId: 123 });

      await waitFor(() => {
        expect(screen.getByText("No design found for id 123.")).toBeInTheDocument();
      });
    });
  });

  describe("successful metadata rendering", () => {
    it("renders the filename in the left column", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });
    });

    it("renders the technical data grid with hoop, date, dimensions, stitches, colours, and colour changes", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("Hoop")).toBeInTheDocument();
      });

      // Hoop
      expect(screen.getByText("Hoop A")).toBeInTheDocument();
      // Date Added
      expect(screen.getByText("2026-05-01")).toBeInTheDocument();
      // Dimensions — rendered as "120 × 80 mm"
      expect(screen.getByText("120 × 80 mm")).toBeInTheDocument();
      // Stitches
      expect(screen.getByText("10000")).toBeInTheDocument();
      // Colours
      expect(screen.getByText("5")).toBeInTheDocument();
      // Colour Changes
      expect(screen.getByText("12")).toBeInTheDocument();
    });

    it("renders unrated designs with the 'Unrated' badge", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({ rating: null })
      );

      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("Rating: Unrated")).toBeInTheDocument();
      });
    });

    it("renders the rating stars and badge for a rated design", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText(/Rating: ★ 4 \/ 5/)).toBeInTheDocument();
      });
    });

    it("renders tag pills with the correct group-specific styles", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("Floral")).toBeInTheDocument();
      });
      expect(screen.getByText("Satin Stitch")).toBeInTheDocument();
      expect(screen.getByText("Blue")).toBeInTheDocument();

      // Tag pills are <span> elements; find the nearest parent pill.
      const floralPill = screen.getByText("Floral").closest("span.group");
      const satinPill = screen.getByText("Satin Stitch").closest("span.group");
      const bluePill = screen.getByText("Blue").closest("span.group");

      expect(floralPill).not.toBeNull();
      expect(satinPill).not.toBeNull();
      expect(bluePill).not.toBeNull();

      // Tag types:
      //  - "image" group  → bg-green-100 text-green-700
      //  - "stitching"    → bg-blue-100 text-blue-700
      //  - null / other   → bg-gray-100 text-gray-700
      expect(floralPill).toHaveClass("bg-green-100", "text-green-700");
      expect(satinPill).toHaveClass("bg-blue-100", "text-blue-700");
      expect(bluePill).toHaveClass("bg-gray-100", "text-gray-700");
    });

    it("renders the projects list", async () => {
      renderDetail();

      // "Wedding Collection" appears both as a project pill AND as an
      // option in the Add-to-project dropdown, hence getAllByText.
      await waitFor(() => {
        expect(screen.getAllByText("Wedding Collection").length).toBeGreaterThan(0);
      });
      expect(
        screen.queryByText("Not assigned to any projects.")
      ).not.toBeInTheDocument();
      // The "Add" dropdown should list available projects.
      expect(screen.getByText("Autumn 2026")).toBeInTheDocument();
    });

    it("renders the Open in Editor and Show in Explorer buttons", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("Open in Editor")).toBeInTheDocument();
      });
      expect(screen.getByText("Show in Explorer")).toBeInTheDocument();
    });
  });

  describe("navigation buttons", () => {
    it("disables Previous at the first browse index", async () => {
      renderDetail({ detailBrowseIndex: 0 });

      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const prevButton = screen.getByTitle("Previous design");
      expect(prevButton).toBeDisabled();
    });

    it("disables Next at the last browse index", async () => {
      renderDetail({ detailBrowseIndex: 2 });

      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const nextButton = screen.getByTitle("Next design");
      expect(nextButton).toBeDisabled();
    });
  });
});
