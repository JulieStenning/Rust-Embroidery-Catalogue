import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import TagsView from "../TagsView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  listTags: vi.fn(),
  createTag: vi.fn(),
  updateTag: vi.fn(),
  deleteTag: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast().
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

// Mock the tag change store — TagTable flags tag mutations here.
const tagChangeMock = vi.hoisted(() => ({
  tagChangeStore: {
    flagTagDeleted: vi.fn(),
    flagTagRenamed: vi.fn(),
    consumeFlags: vi.fn(() => ({ tagsNeedRefresh: false, designsNeedRefresh: false })),
    subscribe: vi.fn(),
  },
}));
vi.mock("../../stores/tagChangeStore.js", () => tagChangeMock);

/** Wraps items in an AdapterListResponse. */
const listResponse = (items: unknown[] = []) => ({ source: "rust", items });

describe("TagsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.localStorage.clear();
    toastMock.addToast.mockResolvedValue(undefined);
    adapterMocks.listTags.mockResolvedValue(listResponse([]));
    adapterMocks.createTag.mockResolvedValue({ persisted: true });
    adapterMocks.updateTag.mockResolvedValue({ persisted: true });
    adapterMocks.deleteTag.mockResolvedValue({ persisted: true });
  });

  it("renders the Manage Tags page title", async () => {
    render(TagsView);

    expect(await screen.findByRole("heading", { name: "Manage Tags" })).toBeInTheDocument();
  });

  it("renders image and stitching tag sections with group-split tags", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([
        { id: 1, description: "Floral", tag_group: "image", design_count: 2 },
        { id: 2, description: "Satin", tag_group: "stitching", design_count: 0 },
      ])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Image Tags" })).toBeInTheDocument();
    });
    expect(screen.getByRole("heading", { name: "Stitching Tags" })).toBeInTheDocument();

    expect(screen.getByText("Floral")).toBeInTheDocument();
    expect(screen.getByText("Satin")).toBeInTheDocument();
  });

  it("does NOT render an Unclassified Tags section", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([
        { id: 1, description: "Floral", tag_group: "image", design_count: 0 },
        { id: 2, description: "Satin", tag_group: "stitching", design_count: 0 },
        { id: 3, description: "Sparkle", tag_group: "", design_count: 0 },
      ])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Image Tags" })).toBeInTheDocument();
    });

    expect(screen.queryByRole("heading", { name: "Unclassified Tags" })).not.toBeInTheDocument();
    expect(screen.queryByText("Sparkle")).not.toBeInTheDocument();
  });

  it("shows empty-state messages when no tags exist", async () => {
    render(TagsView);

    expect(await screen.findByText("No image tags yet.")).toBeInTheDocument();
    expect(screen.getByText("No stitching tags yet.")).toBeInTheDocument();
  });

  it("shows used-by counts for each tag", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 1, description: "Floral", tag_group: "image", design_count: 4 }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("4")).toBeInTheDocument();
    });
  });

  it("adds a new tag with the default image group", async () => {
    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "Bees" } });

    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMocks.createTag).toHaveBeenCalledWith("Bees", "image");
    });
  });

  it("adds a new tag with the stitching group when selected", async () => {
    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "Satin stitch" } });

    const groupSelect = screen.getByLabelText("Group");
    await fireEvent.change(groupSelect, { target: { value: "stitching" } });

    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMocks.createTag).toHaveBeenCalledWith("Satin stitch", "stitching");
    });
  });

  it("deletes a tag from the table after confirmation", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 7, description: "Floral", tag_group: "image", design_count: 0 }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Floral")).toBeInTheDocument();
    });

    const deleteButton = screen.getAllByRole("button", { name: "Delete" })[0];
    await fireEvent.click(deleteButton);

    // No deletion until the user confirms.
    expect(adapterMocks.deleteTag).not.toHaveBeenCalled();

    const confirmButton = screen.getByRole("button", { name: "Confirm delete" });
    await fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(adapterMocks.deleteTag).toHaveBeenCalledWith(7);
    });

    // The tag-change store must be flagged so the browse page refreshes.
    expect(tagChangeMock.tagChangeStore.flagTagDeleted).toHaveBeenCalledTimes(1);
  });

  it("renames a tag via inline edit", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 5, description: "Old", tag_group: "image", design_count: 0 }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Old")).toBeInTheDocument();
    });

    const editButton = screen.getAllByRole("button", { name: "Edit" })[0];
    await fireEvent.click(editButton);

    const input = screen.getByDisplayValue("Old");
    await fireEvent.input(input, { target: { value: "New Name" } });

    const saveButton = screen.getByRole("button", { name: "Save" });
    await fireEvent.click(saveButton);

    await waitFor(() => {
      expect(adapterMocks.updateTag).toHaveBeenCalledWith(5, "New Name");
    });

    // Tag has no designs, so only the tag filter options need refreshing.
    expect(tagChangeMock.tagChangeStore.flagTagRenamed).toHaveBeenCalledWith(false);
  });

  it("flags the tag-change store with hasDesigns=true when renaming a used tag", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 5, description: "Old", tag_group: "image", design_count: 3 }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Old")).toBeInTheDocument();
    });

    const editButton = screen.getAllByRole("button", { name: "Edit" })[0];
    await fireEvent.click(editButton);

    const input = screen.getByDisplayValue("Old");
    await fireEvent.input(input, { target: { value: "New Name" } });

    const saveButton = screen.getByRole("button", { name: "Save" });
    await fireEvent.click(saveButton);

    await waitFor(() => {
      expect(adapterMocks.updateTag).toHaveBeenCalledWith(5, "New Name");
    });

    // Tag is used by designs, so cards must be refreshed as well.
    expect(tagChangeMock.tagChangeStore.flagTagRenamed).toHaveBeenCalledWith(true);
  });

  it("cancels an inline edit without saving", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 5, description: "Old", tag_group: "image", design_count: 0 }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Old")).toBeInTheDocument();
    });

    const editButton = screen.getAllByRole("button", { name: "Edit" })[0];
    await fireEvent.click(editButton);

    const saveButton = screen.getByRole("button", { name: "Save" });
    expect(saveButton).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    await waitFor(() => {
      expect(adapterMocks.updateTag).not.toHaveBeenCalled();
    });
    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
  });

  it("locks edit/delete for system stitching tags with no badge", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([
        {
          id: 1,
          description: "Cross Stitch",
          tag_group: "stitching",
          design_count: 2,
          is_system: true,
        },
        {
          id: 2,
          description: "Custom Stitch",
          tag_group: "stitching",
          design_count: 0,
          is_system: false,
        },
        { id: 3, description: "Floral", tag_group: "image", design_count: 1, is_system: false },
      ])
    );

    render(TagsView);

    // No "System" badge is rendered next to the tag name.
    await waitFor(() => {
      expect(screen.getByText("Cross Stitch")).toBeInTheDocument();
    });
    expect(screen.queryByText("System")).not.toBeInTheDocument();

    // The system tag row shows "Locked" instead of Edit/Delete.
    expect(screen.getByText("Locked")).toBeInTheDocument();

    // The custom stitching tag keeps full CRUD actions.
    expect(screen.getAllByRole("button", { name: "Edit" }).length).toBe(2); // image + custom stitching
    expect(screen.getAllByRole("button", { name: "Delete" }).length).toBe(2);

    // No Edit or Delete button should exist within the system tag row.
    const crossStitchRow = screen.getByText("Cross Stitch").closest("tr");
    expect(crossStitchRow).not.toBeNull();
    expect(crossStitchRow!.querySelectorAll("button").length).toBe(0);
    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
  });

  it("keeps full CRUD actions for image tags even when is_system is truthy", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([
        {
          id: 10,
          description: "Special Image",
          tag_group: "image",
          design_count: 0,
          is_system: true,
        },
      ])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Special Image")).toBeInTheDocument();
    });

    // Image tags never show the System/Locked state regardless of is_system.
    expect(screen.queryByText("System")).not.toBeInTheDocument();
    expect(screen.queryByText("Locked")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Edit" }).length).toBe(1);
    expect(screen.getAllByRole("button", { name: "Delete" }).length).toBe(1);
  });

  it("persists collapsible panel state to localStorage on toggle", async () => {
    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const imageHeading = screen.getByRole("heading", { name: "Image Tags" });
    const imageDetails = imageHeading.closest("details");
    expect(imageDetails).not.toBeNull();

    // jsdom does not implement native <details> toggling, so simulate the
    // toggle by removing the open attribute and dispatching a bubbling event.
    imageDetails!.removeAttribute("open");
    fireEvent(imageDetails!, new Event("toggle", { bubbles: true }));

    await waitFor(() => {
      expect(window.localStorage.getItem("admin.tags.collapsible.image")).toBe("closed");
    });
  });

  it("reloads the tag list after adding a tag", async () => {
    adapterMocks.listTags.mockResolvedValue(listResponse([]));

    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });
    expect(adapterMocks.listTags).toHaveBeenCalledTimes(1);

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "Bees" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMocks.listTags).toHaveBeenCalledTimes(2);
    });
  });

  it("does not call createTag when the description is blank or whitespace", async () => {
    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "   " } });
    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    // The whitespace-only description fails the `if (!desc) return;` guard.
    expect(adapterMocks.createTag).not.toHaveBeenCalled();
  });

  it("shows an error toast when createTag fails to persist", async () => {
    adapterMocks.createTag.mockResolvedValue({ persisted: false, error: "boom" });

    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "Bees" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith("Could not add tag: boom", "error");
    });
  });

  it("shows an error toast and clears tags when listTags returns an error", async () => {
    adapterMocks.listTags.mockResolvedValue({ error: "database unavailable" });

    render(TagsView);

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Failed to load tags: database unavailable",
        "error"
      );
    });

    // Both lists are cleared on a failed load.
    expect(screen.queryByText("No image tags yet.")).toBeInTheDocument();
    expect(screen.queryByText("No stitching tags yet.")).toBeInTheDocument();
  });

  it("shows an error toast when listTags throws", async () => {
    adapterMocks.listTags.mockRejectedValue(new Error("network down"));

    render(TagsView);

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        expect.stringContaining("Failed to load tags"),
        "error"
      );
    });
  });

  it("hydrates saved collapsible panel states from localStorage on mount", async () => {
    window.localStorage.setItem("admin.tags.collapsible.image", "closed");
    window.localStorage.setItem("admin.tags.collapsible.stitching", "open");

    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const imageDetails = screen.getByRole("heading", { name: "Image Tags" }).closest("details");
    const stitchingDetails = screen
      .getByRole("heading", { name: "Stitching Tags" })
      .closest("details");
    expect(imageDetails).not.toBeNull();
    expect(stitchingDetails).not.toBeNull();

    // "closed" removes the open attribute; "open" keeps it.
    expect(imageDetails!.hasAttribute("open")).toBe(false);
    expect(stitchingDetails!.hasAttribute("open")).toBe(true);
  });

  it("persists the stitching panel toggle state to localStorage", async () => {
    render(TagsView);

    await screen.findByRole("heading", { name: "Manage Tags" });

    const stitchingHeading = screen.getByRole("heading", { name: "Stitching Tags" });
    const stitchingDetails = stitchingHeading.closest("details");
    expect(stitchingDetails).not.toBeNull();

    // jsdom does not implement native <details> toggling, so simulate the
    // toggle by removing the open attribute and dispatching a bubbling event.
    stitchingDetails!.removeAttribute("open");
    fireEvent(stitchingDetails!, new Event("toggle", { bubbles: true }));

    await waitFor(() => {
      expect(window.localStorage.getItem("admin.tags.collapsible.stitching")).toBe("closed");
    });
  });

  it("defensively maps tags with missing fields without crashing", async () => {
    adapterMocks.listTags.mockResolvedValue(listResponse([{ id: 9 }]));

    render(TagsView);

    // The minimal tag has no description/group/count/is_system; the defensive
    // ternaries must fall back to safe defaults rather than crash.
    await screen.findByRole("heading", { name: "Manage Tags" });
    expect(adapterMocks.listTags).toHaveBeenCalled();
  });

});
