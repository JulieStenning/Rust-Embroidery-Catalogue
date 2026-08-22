import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import TagSelectionModal from "../TagSelectionModal.svelte";
import { setDesignTags, createTag } from "../../api/commandAdapter";
import { designSessionStore } from "../../stores/designSessionStore";

// ---------------------------------------------------------------------------
// TagSelectionModal.svelte — tag picker used by the Design Detail page and the
// tagging workflow.
//
// Behaviour under test:
//   - Renders nothing when `open` is false; renders a modal dialog when open.
//   - Groups tags by `tag_group` (image / stitching / unclassified).
//   - Reflects `selectedTagIds` as checked checkboxes.
//   - Toggling a tag auto-saves (debounced) via `setDesignTags`.
//   - Search filters tags case-insensitively and reveals a create-tag button
//     for query text that does not exactly match an existing tag.
//   - Create flows call `createTag(description, group)` and surface errors.
//   - Done / backdrop / Escape flush any pending save and call `onClose`.
//   - Successful saves call `designSessionStore.trackMutation` with grouped
//     tag descriptions.
// ---------------------------------------------------------------------------

vi.mock("../../api/commandAdapter", () => ({
  setDesignTags: vi.fn(),
  createTag: vi.fn(),
}));

vi.mock("../../stores/designSessionStore", () => ({
  designSessionStore: { trackMutation: vi.fn() },
}));

const mockedSetDesignTags = vi.mocked(setDesignTags);
const mockedCreateTag = vi.mocked(createTag);
const mockedTrackMutation = vi.mocked(designSessionStore.trackMutation);

const persistedResult: Awaited<ReturnType<typeof setDesignTags>> = {
  source: "rust",
  persisted: true,
  design_id: 5,
  message: "Design tags updated.",
};

const sampleTags = [
  { id: 10, description: "Floral", tag_group: "image" },
  { id: 11, description: "Satin Stitch", tag_group: "stitching" },
  { id: 12, description: "Vintage", tag_group: null },
  { id: 13, description: "Animals", tag_group: "image" },
  { id: 14, description: "Applique", tag_group: "stitching" },
];

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

async function renderModal(props: Record<string, unknown>) {
  const result = render(TagSelectionModal, { props });
  await tick();
  return result;
}

describe("TagSelectionModal", () => {
  beforeEach(() => {
    mockedSetDesignTags.mockReset();
    mockedSetDesignTags.mockResolvedValue(persistedResult);
    mockedCreateTag.mockReset();
    mockedTrackMutation.mockReset();
  });

  describe("rendering", () => {
    it("renders nothing when open is false", async () => {
      renderModal({ designId: 7, allTags: sampleTags, open: false });

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("renders the dialog when open is true", async () => {
      renderModal({ designId: 7, allTags: sampleTags, open: true });

      const dialog = screen.getByRole("dialog");
      expect(dialog).toBeInTheDocument();
      expect(dialog).toHaveAttribute("aria-modal", "true");
    });

    it("shows the design id, search placeholder and Done button", async () => {
      renderModal({ designId: 42, allTags: sampleTags, open: true });

      expect(screen.getByText("Design #42")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("🔍 Search or create tag...")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Done" })).toBeInTheDocument();
    });

    it("does not auto-save when opened without interaction", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });

      expect(mockedSetDesignTags).not.toHaveBeenCalled();
    });
  });

  describe("tag sections", () => {
    it("groups tags by tag_group with the correct headings", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });

      expect(screen.getByText("Image tags")).toBeInTheDocument();
      expect(screen.getByText("Stitching tags")).toBeInTheDocument();
      expect(screen.getByText("Unclassified tags")).toBeInTheDocument();
    });

    it("renders every tag as a labelled checkbox", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });

      for (const tag of sampleTags) {
        expect(screen.getByRole("checkbox", { name: tag.description })).toBeInTheDocument();
      }
    });

    it("only renders sections that contain tags", async () => {
      const stitchingOnly = sampleTags.filter((t) => t.tag_group === "stitching");
      renderModal({ designId: 1, allTags: stitchingOnly, open: true });

      expect(screen.getByText("Stitching tags")).toBeInTheDocument();
      expect(screen.queryByText("Image tags")).not.toBeInTheDocument();
      expect(screen.queryByText("Unclassified tags")).not.toBeInTheDocument();
    });

    it("shows a placeholder message when no tags exist", async () => {
      renderModal({ designId: 1, allTags: [], open: true });

      expect(screen.getByText("No matching tags found.")).toBeInTheDocument();
    });
  });

  describe("checkbox selection", () => {
    it("checks tags included in selectedTagIds", async () => {
      renderModal({
        designId: 1,
        allTags: sampleTags,
        selectedTagIds: [10, 13],
        open: true,
      });

      expect(screen.getByRole("checkbox", { name: "Floral" })).toBeChecked();
      expect(screen.getByRole("checkbox", { name: "Animals" })).toBeChecked();
      expect(screen.getByRole("checkbox", { name: "Satin Stitch" })).not.toBeChecked();
    });

    it("tolerates a non-array allTags prop", async () => {
      renderModal({
        designId: 1,
        allTags: null,
        selectedTagIds: [1],
        open: true,
      });

      expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
      expect(screen.getByText("No matching tags found.")).toBeInTheDocument();
    });
  });

  describe("auto-save", () => {
    it("persists the updated selection when a tag is toggled on", async () => {
      renderModal({
        designId: 5,
        allTags: sampleTags,
        selectedTagIds: [10],
        open: true,
      });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Animals" }));

      await waitFor(() => {
        expect(mockedSetDesignTags).toHaveBeenCalledWith(5, [10, 13], {
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      });
    });

    it("persists the updated selection when a tag is toggled off", async () => {
      renderModal({
        designId: 5,
        allTags: sampleTags,
        selectedTagIds: [10, 13],
        open: true,
      });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Animals" }));

      await waitFor(() => {
        expect(mockedSetDesignTags).toHaveBeenCalledWith(5, [10], {
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      });
    });

    it("persists an empty selection when the last tag is removed", async () => {
      renderModal({
        designId: 5,
        allTags: sampleTags,
        selectedTagIds: [10],
        open: true,
      });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));

      await waitFor(() => {
        expect(mockedSetDesignTags).toHaveBeenCalledWith(5, [], {
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      });
    });

    it("debounces rapid toggles into a single save call", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));
      await fireEvent.click(screen.getByRole("checkbox", { name: "Satin Stitch" }));
      await fireEvent.click(screen.getByRole("checkbox", { name: "Applique" }));

      await waitFor(() => {
        expect(mockedSetDesignTags).toHaveBeenCalledTimes(1);
      });
      expect(mockedSetDesignTags).toHaveBeenCalledWith(5, [10, 11, 14], {
        imageTagsVerified: true,
        stitchingTagsVerified: true,
      });
    });

    it("shows the saving state while persistence is in flight", async () => {
      const { promise, resolve } = deferred<typeof persistedResult>();
      mockedSetDesignTags.mockReturnValue(promise);
      renderModal({ designId: 5, allTags: sampleTags, open: true });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Saving..." })).toBeDisabled();
      });
      expect(screen.getAllByText("Saving...").length).toBeGreaterThan(0);

      resolve(persistedResult);

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Done" })).toBeEnabled();
      });
      expect(screen.queryByText("Saving...")).not.toBeInTheDocument();
    });
  });

  describe("closing", () => {
    it("persists the current selection and closes when Done is clicked", async () => {
      const onClose = vi.fn();
      renderModal({
        designId: 5,
        allTags: sampleTags,
        selectedTagIds: [12],
        open: true,
        onClose,
      });

      await fireEvent.click(screen.getByRole("button", { name: "Done" }));

      await waitFor(() => {
        expect(mockedSetDesignTags).toHaveBeenCalledWith(5, [12], {
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      });
      await waitFor(() => {
        expect(onClose).toHaveBeenCalledTimes(1);
      });
    });

    it("closes when the backdrop is clicked", async () => {
      const onClose = vi.fn();
      renderModal({ designId: 5, allTags: sampleTags, open: true, onClose });

      await fireEvent.click(screen.getByLabelText("Close tag chooser"));

      await waitFor(() => {
        expect(onClose).toHaveBeenCalledTimes(1);
      });
    });

    it("closes when the Escape key is pressed", async () => {
      const onClose = vi.fn();
      renderModal({ designId: 5, allTags: sampleTags, open: true, onClose });

      await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });

      await waitFor(() => {
        expect(onClose).toHaveBeenCalledTimes(1);
      });
    });
  });

  describe("search filtering", () => {
    it("filters tags case-insensitively by description", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "AN" } });

      await waitFor(() => {
        expect(screen.getByRole("checkbox", { name: "Animals" })).toBeInTheDocument();
      });
      expect(screen.queryByRole("checkbox", { name: "Floral" })).not.toBeInTheDocument();
    });

    it("keeps grouped sections intact while filtering", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "applique" } });

      await waitFor(() => {
        expect(screen.getByRole("checkbox", { name: "Applique" })).toBeInTheDocument();
      });
      expect(screen.getByText("Stitching tags")).toBeInTheDocument();
      expect(screen.queryByRole("checkbox", { name: "Floral" })).not.toBeInTheDocument();
    });

    it("hides the create button when the query exactly matches a tag", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Floral" } });

      await waitFor(() => {
        expect(screen.getByRole("checkbox", { name: "Floral" })).toBeInTheDocument();
      });
      expect(screen.queryByRole("button", { name: /Create "Floral"/ })).not.toBeInTheDocument();
    });

    it("shows the create button for a non-matching query and hides the empty message", async () => {
      renderModal({ designId: 1, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "zzz" } });

      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "zzz"/ })).toBeInTheDocument();
      });
      expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
      expect(screen.queryByText("No matching tags found.")).not.toBeInTheDocument();
    });
  });

  describe("create tag", () => {
    beforeEach(() => {
      mockedCreateTag.mockResolvedValue({
        source: "rust",
        persisted: true,
        item: { id: 99, description: "Zebra", tag_group: "image" },
      } as Awaited<ReturnType<typeof createTag>>);
    });

    it("creates a tag with the default image group", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });
      expect(screen.getByRole("combobox")).toHaveValue("image");

      await fireEvent.click(screen.getByRole("button", { name: /Create "Zebra"/ }));

      await waitFor(() => {
        expect(mockedCreateTag).toHaveBeenCalledWith("Zebra", "image");
      });
    });

    it("creates a tag in the selected stitching group", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });

      await fireEvent.change(screen.getByRole("combobox"), {
        target: { value: "stitching" },
      });
      expect(screen.getByRole("combobox")).toHaveValue("stitching");

      await fireEvent.click(screen.getByRole("button", { name: /Create "Zebra"/ }));

      await waitFor(() => {
        expect(mockedCreateTag).toHaveBeenCalledWith("Zebra", "stitching");
      });
    });

    it("inserts the new tag, marks it checked and clears the search box", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });

      await fireEvent.click(screen.getByRole("button", { name: /Create "Zebra"/ }));

      await waitFor(() => {
        const zebra = screen.getByRole("checkbox", { name: "Zebra" });
        expect(zebra).toBeInTheDocument();
        expect(zebra).toBeChecked();
      });
      expect(screen.getByPlaceholderText("🔍 Search or create tag...")).toHaveValue("");
      expect(screen.queryByRole("button", { name: /Create/ })).not.toBeInTheDocument();
    });

    it("creates a tag when Enter is pressed in the search box", async () => {
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });

      await fireEvent.keyDown(input, { key: "Enter" });

      await waitFor(() => {
        expect(mockedCreateTag).toHaveBeenCalledTimes(1);
      });
      expect(mockedCreateTag).toHaveBeenCalledWith("Zebra", "image");
    });

    it("shows an error when creation is rejected by the backend", async () => {
      mockedCreateTag.mockResolvedValue({
        source: "mock",
        persisted: false,
        item: undefined,
        error: "duplicate tag",
      } as unknown as Awaited<ReturnType<typeof createTag>>);
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });

      await fireEvent.click(screen.getByRole("button", { name: /Create "Zebra"/ }));

      await waitFor(() => {
        expect(screen.getByText("Could not create tag: duplicate tag")).toBeInTheDocument();
      });
    });

    it("shows an error when createTag throws", async () => {
      mockedCreateTag.mockRejectedValue(new Error("backend unreachable"));
      renderModal({ designId: 5, allTags: sampleTags, open: true });
      const input = screen.getByPlaceholderText("🔍 Search or create tag...");

      await fireEvent.input(input, { target: { value: "Zebra" } });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Create "Zebra"/ })).toBeInTheDocument();
      });

      await fireEvent.click(screen.getByRole("button", { name: /Create "Zebra"/ }));

      await waitFor(() => {
        expect(
          screen.getByText("Create tag error: Error: backend unreachable")
        ).toBeInTheDocument();
      });
    });
  });

  describe("session store integration", () => {
    it("tracks the mutation with grouped tag data after a successful save", async () => {
      renderModal({
        designId: 5,
        allTags: sampleTags,
        selectedTagIds: [10, 11],
        open: true,
      });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Animals" }));

      await waitFor(() => {
        expect(mockedTrackMutation).toHaveBeenCalledWith(5, {
          tags: ["Floral", "Satin Stitch", "Animals"],
          imageTags: ["Floral", "Animals"],
          stitchingTags: ["Satin Stitch"],
          imageTagsVerified: true,
          stitchingTagsVerified: true,
        });
      });
    });
  });

  describe("error handling", () => {
    it("shows an auto-save failure message when persistence is not confirmed", async () => {
      mockedSetDesignTags.mockResolvedValue({
        source: "mock",
        persisted: false,
        design_id: 5,
        message: "Could not update tags: boom",
        error: "boom",
      });
      renderModal({ designId: 5, allTags: sampleTags, open: true });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));

      await waitFor(() => {
        expect(
          screen.getByText("Auto-save failed: Could not update tags: boom")
        ).toBeInTheDocument();
      });
    });

    it("shows an error when the auto-save request throws", async () => {
      mockedSetDesignTags.mockRejectedValue(new Error("network down"));
      renderModal({ designId: 5, allTags: sampleTags, open: true });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));

      await waitFor(() => {
        expect(screen.getByText("Auto-save error: Error: network down")).toBeInTheDocument();
      });
    });
  });

  describe("edge cases", () => {
    it("skips auto-save when the design id is missing", async () => {
      renderModal({ designId: 0, allTags: sampleTags, open: true });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Floral" }));
      await new Promise((r) => setTimeout(r, 350));

      expect(mockedSetDesignTags).not.toHaveBeenCalled();
    });

    it("ignores toggles for tags with non-finite ids", async () => {
      renderModal({
        designId: 5,
        allTags: [{ id: Number("abc"), description: "Broken", tag_group: "image" }],
        open: true,
      });

      await fireEvent.click(screen.getByRole("checkbox", { name: "Broken" }));
      await new Promise((r) => setTimeout(r, 350));

      expect(mockedSetDesignTags).not.toHaveBeenCalled();
    });
  });
});
