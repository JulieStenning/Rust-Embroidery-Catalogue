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

    expect(
      await screen.findByRole("heading", { name: "Manage Tags" })
    ).toBeInTheDocument();
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
      expect(
        screen.getByRole("heading", { name: "Image Tags" })
      ).toBeInTheDocument();
    });
    expect(
      screen.getByRole("heading", { name: "Stitching Tags" })
    ).toBeInTheDocument();

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
      expect(
        screen.getByRole("heading", { name: "Image Tags" })
      ).toBeInTheDocument();
    });

    expect(
      screen.queryByRole("heading", { name: "Unclassified Tags" })
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Sparkle")).not.toBeInTheDocument();
  });

  it("shows empty-state messages when no tags exist", async () => {
    render(TagsView);

    expect(await screen.findByText("No image tags yet.")).toBeInTheDocument();
    expect(screen.getByText("No stitching tags yet.")).toBeInTheDocument();
  });

  it("shows used-by counts for each tag", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([
        { id: 1, description: "Floral", tag_group: "image", design_count: 4 },
      ])
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
      expect(
        window.localStorage.getItem("admin.tags.collapsible.image")
      ).toBe("closed");
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
});