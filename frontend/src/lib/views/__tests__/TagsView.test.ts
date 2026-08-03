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
  setTagGroup: vi.fn(),
  deleteTag: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast().
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

/** Wraps items in an AdapterListResponse. */
const listResponse = (items: unknown[] = []) => ({ source: "rust", items });

describe("TagsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.localStorage.clear();
    toastMock.addToast.mockResolvedValue(undefined);
    adapterMocks.listTags.mockResolvedValue(listResponse([]));
    adapterMocks.createTag.mockResolvedValue({ persisted: true });
    adapterMocks.setTagGroup.mockResolvedValue({ persisted: true });
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
        { id: 1, description: "Floral", tag_group: "image" },
        { id: 2, description: "Satin", tag_group: "stitching" },
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
        { id: 1, description: "Floral", tag_group: "image" },
        { id: 2, description: "Satin", tag_group: "stitching" },
        { id: 3, description: "Sparkle", tag_group: "" },
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

  it("deletes a tag from the table", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 7, description: "Floral", tag_group: "image" }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Floral")).toBeInTheDocument();
    });

    const deleteButton = screen.getAllByRole("button", { name: "Delete" })[0];
    await fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(adapterMocks.deleteTag).toHaveBeenCalledWith(7);
    });
  });

  it("reassigns a tag group via the row dropdown", async () => {
    adapterMocks.listTags.mockResolvedValue(
      listResponse([{ id: 3, description: "Satin", tag_group: "stitching" }])
    );

    render(TagsView);

    await waitFor(() => {
      expect(screen.getByText("Satin")).toBeInTheDocument();
    });

    const stitchingHeading = screen.getByRole("heading", { name: "Stitching Tags" });
    const detailsNode = stitchingHeading.closest("details");
    const rowSelect = detailsNode?.querySelector<HTMLSelectElement>("select");
    expect(rowSelect).not.toBeNull();

    await fireEvent.change(rowSelect!, { target: { value: "image" } });

    await waitFor(() => {
      expect(adapterMocks.setTagGroup).toHaveBeenCalledWith(3, "image");
    });
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