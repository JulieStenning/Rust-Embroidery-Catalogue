import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
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
  reparseDesignFile: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — every interaction handler calls addToast().
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore", () => toastMock);

// Mock the design session store — mutation handlers call trackMutation().
const sessionMock = vi.hoisted(() => ({
  designSessionStore: {
    trackMutation: vi.fn(),
  },
}));
vi.mock("../../stores/designSessionStore", () => sessionMock);

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

/** Type-guard helper so querySelector results can be passed to user-event/within. */
function element(value: Element | null | undefined, message?: string): HTMLElement {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value as HTMLElement;
}

describe("DesignDetailView", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    adapterMocks.getDesignDetail.mockResolvedValue(detailResponse());
    adapterMocks.getDesignImageDataUrl.mockResolvedValue({
      source: "rust",
      item: { data_url: "data:image/png;base64,refreshed3d", image_type: "3d" },
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
    adapterMocks.reparseDesignFile.mockResolvedValue({
      source: "rust",
      persisted: true,
      result: {
        designId: 42,
        widthMm: 150,
        heightMm: 100,
        stitchCount: 12345,
        colorCount: 6,
        colorChangeCount: 15,
        hoopId: 2,
        hoop: "Hoop B",
        message: "Design metadata recalculated from file.",
      },
      message: "Design metadata recalculated from file.",
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

  describe("star rating interactions", () => {
    it("calls setDesignRating with the clicked score", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "3 stars" }));

      await waitFor(() => {
        expect(adapterMocks.setDesignRating).toHaveBeenCalledWith(42, 3);
      });
      expect(toastMock.addToast).toHaveBeenCalledWith("Rating updated.", "success");
    });

    it("clears the rating when the active star is clicked", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "4 stars" }));

      await waitFor(() => {
        expect(adapterMocks.setDesignRating).toHaveBeenCalledWith(42, null);
      });
    });

    it("clears the rating via the Clear button", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Clear" }));

      await waitFor(() => {
        expect(adapterMocks.setDesignRating).toHaveBeenCalledWith(42, null);
      });
    });
  });

  describe("status toggle interactions", () => {
    it("toggles the stitched state off", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Stitched/ }));

      await waitFor(() => {
        expect(adapterMocks.setDesignStitched).toHaveBeenCalledWith(42, false);
      });
    });

    it("marks a design as verified", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Verify/ }));

      await waitFor(() => {
        expect(adapterMocks.setDesignTagsChecked).toHaveBeenCalledWith(42, true);
      });
    });
  });

  describe("tag interactions", () => {
    it("opens the tag selection modal via Choose tags...", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Choose tags/ }));

      await waitFor(() => {
        expect(
          screen.getByRole("dialog", { name: "Choose tags for this design" })
        ).toBeInTheDocument();
      });
    });

    it("removes a tag via its × button and tracks the mutation", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Floral")).toBeInTheDocument();
      });

      const floralPill = element(
        screen.getByText("Floral").closest("span.group"),
        "Expected the Floral pill to exist."
      );
      const user = userEvent.setup();
      await user.click(within(floralPill).getByTitle("Remove tag"));

      await waitFor(() => {
        expect(adapterMocks.removeDesignTag).toHaveBeenCalledWith(42, 11);
      });
      expect(toastMock.addToast).toHaveBeenCalledWith("Tag removed.", "success");
      expect(sessionMock.designSessionStore.trackMutation).toHaveBeenCalledWith(
        42,
        expect.objectContaining({ tags: expect.any(Array) })
      );
      // Optimistic UI removes the tag pill from the DOM immediately.
      await waitFor(() => {
        expect(screen.queryByText("Floral")).not.toBeInTheDocument();
      });
    });

    it("rolls back the optimistic tag removal when the backend rejects", async () => {
      adapterMocks.removeDesignTag.mockResolvedValue({
        source: "rust",
        persisted: false,
        design_id: 42,
        message: "Tag is in use by another design.",
      });

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Floral")).toBeInTheDocument();
      });

      const floralPill = element(
        screen.getByText("Floral").closest("span.group"),
        "Expected the Floral pill to exist."
      );
      const user = userEvent.setup();
      await user.click(within(floralPill).getByTitle("Remove tag"));

      // Error toast surfaced.
      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Tag is in use by another design.",
          "error"
        );
      });

      // The tag is restored after refreshDetailAfterAction re-fetches.
      await waitFor(() => {
        expect(screen.getByText("Floral")).toBeInTheDocument();
      });
    });
  });

  describe("designer & source auto-save", () => {
    it("auto-saves when the designer is cleared to None", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.selectOptions(screen.getByLabelText("Designer"), "");

      await waitFor(() => {
        expect(adapterMocks.updateDesignMetadata).toHaveBeenCalledWith(42, {
          designer_id: null,
          source_id: 3,
        });
      });
      expect(toastMock.addToast).toHaveBeenCalledWith("Designer updated", "success");
    });

    it("switches to a second designer from the dropdown", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({
          designers: [
            { id: 7, name: "Rose Studio" },
            { id: 8, name: "Lily Studio" },
          ],
        })
      );

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Lily Studio")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.selectOptions(screen.getByLabelText("Designer"), "8");

      await waitFor(() => {
        expect(adapterMocks.updateDesignMetadata).toHaveBeenCalledWith(42, {
          designer_id: 8,
          source_id: 3,
        });
      });
    });

    it("reverts the designer dropdown when the save fails", async () => {
      adapterMocks.updateDesignMetadata.mockResolvedValue({
        source: "rust",
        persisted: false,
        design_id: 42,
        message: "Failed to update designer",
      });

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const designerSelect = screen.getByLabelText("Designer") as HTMLSelectElement;
      const user = userEvent.setup();
      await user.selectOptions(designerSelect, "");

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Failed to update designer",
          "error"
        );
      });
      // The dropdown must revert to the previously known-good value.
      expect(designerSelect.value).toBe("7");
    });

    it("auto-saves when the source is cleared to None", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.selectOptions(screen.getByLabelText("Source"), "");

      await waitFor(() => {
        expect(adapterMocks.updateDesignMetadata).toHaveBeenCalledWith(42, {
          designer_id: 7,
          source_id: null,
        });
      });
      expect(toastMock.addToast).toHaveBeenCalledWith("Source updated", "success");
    });

    it("reverts the source dropdown when the save fails", async () => {
      adapterMocks.updateDesignMetadata.mockResolvedValue({
        source: "rust",
        persisted: false,
        design_id: 42,
        message: "Failed to update source",
      });

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const sourceSelect = screen.getByLabelText("Source") as HTMLSelectElement;
      const user = userEvent.setup();
      await user.selectOptions(sourceSelect, "");

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Failed to update source",
          "error"
        );
      });
      // The dropdown must revert to the previously known-good value.
      expect(sourceSelect.value).toBe("3");
    });
  });

  describe("notes editing", () => {
    it("saves updated notes via the Save Notes button", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const notesTextarea = screen.getByPlaceholderText(
        "Add notes about this design..."
      );
      const user = userEvent.setup();
      await user.clear(notesTextarea);
      await user.type(notesTextarea, "New notes added in test.");

      await user.click(screen.getByRole("button", { name: "Save Notes" }));

      await waitFor(() => {
        expect(adapterMocks.updateDesignMetadata).toHaveBeenCalledWith(42, {
          notes: "New notes added in test.",
          designer_id: 7,
          source_id: 3,
        });
      });
    });

    it("disables Save Notes when the notes are unchanged", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      expect(screen.getByRole("button", { name: "Save Notes" })).toBeDisabled();
    });
  });

  describe("project management", () => {
    it("adds the design to a selected project", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const projectsSection = element(
        screen.getByText("Projects").closest(".route-card"),
        "Expected the Projects card to exist."
      );
      const projectSelect = within(projectsSection).getByRole("combobox");

      const user = userEvent.setup();
      await user.selectOptions(projectSelect, "2");
      await user.click(within(projectsSection).getByRole("button", { name: "Add" }));

      await waitFor(() => {
        expect(adapterMocks.addDesignToProject).toHaveBeenCalledWith(42, 2);
      });
    });

    it("removes the design from a project via its × button", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getAllByText("Wedding Collection").length).toBeGreaterThan(0);
      });

      const projectsSection = element(
        screen.getByText("Projects").closest(".route-card"),
        "Expected the Projects card to exist."
      );
      const user = userEvent.setup();
      await user.click(within(projectsSection).getByTitle("Remove from project"));

      await waitFor(() => {
        expect(adapterMocks.removeDesignFromProject).toHaveBeenCalledWith(42, 1);
      });
    });

    it("disables the Add button while no project is selected", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const projectsSection = element(
        screen.getByText("Projects").closest(".route-card"),
        "Expected the Projects card to exist."
      );
      expect(
        within(projectsSection).getByRole("button", { name: "Add" })
      ).toBeDisabled();
    });
  });

  describe("action buttons", () => {
    it("opens the design in the platform editor", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Open in Editor")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Open in Editor/ }));

      await waitFor(() => {
        expect(adapterMocks.openDesignInEditor).toHaveBeenCalledWith(42);
      });
    });

    it("shows the design in the file explorer", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Show in Explorer")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Show in Explorer/ }));

      await waitFor(() => {
        expect(adapterMocks.openDesignInExplorer).toHaveBeenCalledWith(42);
      });
    });

    it("generates a 3D preview for a design with no existing image", async () => {
      renderDetail();
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Generate 3D Preview" })
        ).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Generate 3D Preview" }));

      await waitFor(() => {
        expect(adapterMocks.renderDesign3dPreview).toHaveBeenCalledWith(42, true);
      });
      await waitFor(() => {
        expect(adapterMocks.getDesignImageDataUrl).toHaveBeenCalledWith(42);
      });
    });

    it("labels the button as Generate 2D Preview when the image is already 3D", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({
          imageDataUrl: "data:image/png;base64,abc",
          imageType: "3d",
        })
      );

      renderDetail();
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Generate 2D Preview" })
        ).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Generate 2D Preview" }));

      await waitFor(() => {
        expect(adapterMocks.renderDesign3dPreview).toHaveBeenCalledWith(42, false);
      });
    });

    it("labels the button as Generate 3D Preview when the image is already 2D", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({
          imageDataUrl: "data:image/png;base64,abc",
          imageType: "2d",
        })
      );

      renderDetail();
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Generate 3D Preview" })
        ).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Generate 3D Preview" }));

      await waitFor(() => {
        expect(adapterMocks.renderDesign3dPreview).toHaveBeenCalledWith(42, true);
      });
    });

    it("shows an error toast when the file explorer launch fails", async () => {
      adapterMocks.openDesignInExplorer.mockResolvedValue({
        source: "rust",
        persisted: false,
        result: { success: false },
        message: "Explorer unavailable.",
      });

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Show in Explorer")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Show in Explorer/ }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Explorer unavailable.",
          "error"
        );
      });
    });

    it("shows an error toast when the 3D preview generation fails", async () => {
      adapterMocks.renderDesign3dPreview.mockResolvedValue({
        source: "rust",
        persisted: false,
        result: { success: false },
        message: "3D renderer failed.",
      });

      renderDetail();
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Generate 3D Preview" })
        ).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Generate 3D Preview" }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "3D renderer failed.",
          "error"
        );
      });
      // The image refresh must NOT be triggered on failure.
      expect(adapterMocks.getDesignImageDataUrl).not.toHaveBeenCalled();
    });
  });

  describe("recalculate from file", () => {
    it("calls reparseDesignFile and updates the technical data grid", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      // Initial values from the fixture
      expect(screen.getByText("120 × 80 mm")).toBeInTheDocument();
      expect(screen.getByText("10000")).toBeInTheDocument();
      expect(screen.getByText("5")).toBeInTheDocument();
      expect(screen.getByText("12")).toBeInTheDocument();

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Recalculate From File/ }));

      await waitFor(() => {
        expect(adapterMocks.reparseDesignFile).toHaveBeenCalledWith(42);
      });

      // The reactive technical grid updates instantly with fresh values
      await waitFor(() => {
        expect(screen.getByText("150 × 100 mm")).toBeInTheDocument();
      });
      expect(screen.getByText("12345")).toBeInTheDocument();
      expect(screen.getByText("6")).toBeInTheDocument();
      expect(screen.getByText("15")).toBeInTheDocument();
      expect(screen.getByText("Hoop B")).toBeInTheDocument();

      // The toast surfaced the success message
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Design metadata recalculated from file.",
        "success"
      );
    });

    it("tracks the hoop mutation for browse card sync", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Recalculate From File/ }));

      await waitFor(() => {
        expect(sessionMock.designSessionStore.trackMutation).toHaveBeenCalledWith(
          42,
          { hoop: "Hoop B" }
        );
      });
    });

    it("shows an error toast when recalculation fails and keeps old values", async () => {
      adapterMocks.reparseDesignFile.mockResolvedValue({
        source: "rust",
        persisted: false,
        result: null,
        message: "Could not recalculate metadata: Design file not found on disk.",
      });

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Recalculate From File/ }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Could not recalculate metadata: Design file not found on disk.",
          "error"
        );
      });

      // The original technical values remain untouched
      expect(screen.getByText("120 × 80 mm")).toBeInTheDocument();
      expect(screen.getByText("10000")).toBeInTheDocument();
      expect(screen.getByText("5")).toBeInTheDocument();
      expect(screen.getByText("12")).toBeInTheDocument();
      expect(screen.getByText("Hoop A")).toBeInTheDocument();
    });

    it("disables the button while a recalculation is in flight", async () => {
      // Never resolve — the button stays disabled and shows Recalculating…
      adapterMocks.reparseDesignFile.mockReturnValue(new Promise(() => {}));

      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const button = screen.getByRole("button", { name: /Recalculate From File/ });
      const user = userEvent.setup();
      await user.click(button);

      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Recalculating/ })).toBeDisabled();
      });
      expect(adapterMocks.reparseDesignFile).toHaveBeenCalledWith(42);
    });
  });

  describe("navigation interactions", () => {
    it("navigates back to the browse view", async () => {
      const navigateTo = vi.fn();
      renderDetail({ navigateTo });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: /Back to Browse/ }));

      expect(navigateTo).toHaveBeenCalledWith("#/designs");
    });

    it("navigates to the previous browse design", async () => {
      const navigateTo = vi.fn();
      renderDetail({ detailBrowseIndex: 1, navigateTo });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByTitle("Previous design"));

      expect(navigateTo).toHaveBeenCalledWith("#/designs/41");
    });

    it("navigates to the next browse design", async () => {
      const navigateTo = vi.fn();
      renderDetail({ detailBrowseIndex: 1, navigateTo });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByTitle("Next design"));

      expect(navigateTo).toHaveBeenCalledWith("#/designs/43");
    });

    it("opens the print view for the current design", async () => {
      const navigateTo = vi.fn();
      renderDetail({ navigateTo });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Print" }));

      expect(navigateTo).toHaveBeenCalledWith("#/designs/42/print");
    });
  });

  describe("delete flow", () => {
    it("opens the delete confirmation modal", async () => {
      renderDetail();
      await waitFor(() => {
        expect(screen.getByText("Delete design")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Delete design" }));

      await waitFor(() => {
        expect(
          screen.getByRole("heading", { name: "Delete selected design?" })
        ).toBeInTheDocument();
      });
    });

    it("confirms deletion, notifies the parent and navigates to browse", async () => {
      const navigateTo = vi.fn();
      const onDesignDeleted = vi.fn();
      renderDetail({ navigateTo, onDesignDeleted });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Delete design" }));
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Delete 1 design" })
        ).toBeInTheDocument();
      });
      await user.click(screen.getByRole("button", { name: "Delete 1 design" }));

      await waitFor(() => {
        expect(adapterMocks.bulkDeleteDesigns).toHaveBeenCalledWith([42], false);
      });
      expect(onDesignDeleted).toHaveBeenCalled();
      expect(navigateTo).toHaveBeenCalledWith("#/designs");
    });

    it("shows an error toast and does not navigate when deletion fails", async () => {
      const navigateTo = vi.fn();
      adapterMocks.bulkDeleteDesigns.mockResolvedValue({
        source: "rust",
        persisted: false,
        deleted_count: 0,
        files_trashed: 0,
        errors: ["Permission denied"],
      });

      renderDetail({ navigateTo });
      await waitFor(() => {
        expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
      });

      const user = userEvent.setup();
      await user.click(screen.getByRole("button", { name: "Delete design" }));
      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: "Delete 1 design" })
        ).toBeInTheDocument();
      });
      await user.click(screen.getByRole("button", { name: "Delete 1 design" }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Permission denied",
          "error"
        );
      });
      expect(navigateTo).not.toHaveBeenCalled();
    });
  });

  describe("filepath and preview rendering", () => {
    it("shows the 'No preview image saved yet.' placeholder when no data URL exists", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("No preview image saved yet.")).toBeInTheDocument();
      });
    });

    it("renders the preview image from the data URL", async () => {
      adapterMocks.getDesignDetail.mockResolvedValue(
        detailResponse({ imageDataUrl: "data:image/png;base64,abc" })
      );

      renderDetail();

      await waitFor(() => {
        expect(screen.getByAltText("rose-border-01.pes")).toBeInTheDocument();
      });
      expect(screen.getByAltText("rose-border-01.pes").getAttribute("src")).toBe(
        "data:image/png;base64,abc"
      );
    });

    it("renders the collapsible file path details", async () => {
      renderDetail();

      await waitFor(() => {
        expect(screen.getByText("C:/designs/rose-border-01.pes")).toBeInTheDocument();
      });
      expect(screen.getByText("Show file path")).toBeInTheDocument();
    });
  });
});