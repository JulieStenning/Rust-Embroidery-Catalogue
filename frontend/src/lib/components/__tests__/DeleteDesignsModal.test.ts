import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import DeleteDesignsModal from "../DeleteDesignsModal.svelte";
import { bulkDeleteDesigns } from "../../api/commandAdapter";

// ---------------------------------------------------------------------------
// DeleteDesignsModal.svelte — shared unified deletion modal used by the Browse
// page and the Design Detail page.
//
// Behaviour under test:
//   - Renders nothing when `open` is false.
//   - Renders the dialog, singular/plural labels and selected-count message.
//   - Collapsible preview list is hidden when <= 1 design selected.
//   - "deleteFile" radio toggling reveals/hides the recycle-bin warning.
//   - Cancel / backdrop / Escape call `onClose` (and are suppressed while busy).
//   - The delete button is disabled for empty design lists and while busy.
//   - confirmDelete calls `bulkDeleteDesigns(designIds, deleteFile)` and
//     forwards the result (or a synthesized error result) to `onDeleted`.
//   - Preview items render filenames, filepaths, thumbnails and placeholders.
// ---------------------------------------------------------------------------

vi.mock("../../api/commandAdapter", () => ({
  bulkDeleteDesigns: vi.fn(),
}));

const mockedBulkDeleteDesigns = vi.mocked(bulkDeleteDesigns);

const successResult = {
  source: "rust",
  persisted: true,
  deleted_count: 2,
  files_trashed: 0,
  errors: [],
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

describe("DeleteDesignsModal", () => {
  beforeEach(() => {
    mockedBulkDeleteDesigns.mockReset();
    mockedBulkDeleteDesigns.mockResolvedValue(successResult);
  });

  describe("rendering", () => {
    it("renders nothing when open is false", () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2], open: false } });

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("renders the dialog when open is true", () => {
      render(DeleteDesignsModal, { props: { designIds: [1], open: true } });

      expect(screen.getByRole("dialog")).toBeInTheDocument();
      expect(screen.getByRole("dialog")).toHaveAttribute("aria-modal", "true");
    });

    it("uses a singular heading for one design", () => {
      render(DeleteDesignsModal, { props: { designIds: [7], open: true } });

      expect(screen.getByRole("heading", { name: "Delete selected design?" })).toBeInTheDocument();
    });

    it("uses a plural heading for multiple designs", () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2, 3], open: true } });

      expect(screen.getByRole("heading", { name: "Delete selected designs?" })).toBeInTheDocument();
    });

    it("shows the singular selected-count message", () => {
      render(DeleteDesignsModal, { props: { designIds: [5], open: true } });

      expect(screen.getByText("1 design selected.")).toBeInTheDocument();
    });

    it("shows the plural selected-count message", () => {
      render(DeleteDesignsModal, { props: { designIds: [5, 6], open: true } });

      expect(screen.getByText("2 designs selected.")).toBeInTheDocument();
    });
  });

  describe("preview visibility", () => {
    it("hides the collapsible preview for a single design", () => {
      render(DeleteDesignsModal, {
        props: {
          designIds: [1],
          previewItems: [{ id: 1, filename: "flower.jef", filepath: "C:\\flower.jef" }],
          open: true,
        },
      });

      expect(screen.queryByText(/Review selected designs/)).not.toBeInTheDocument();
    });

    it("shows the collapsible preview and count for multiple designs", () => {
      render(DeleteDesignsModal, {
        props: {
          designIds: [1, 2],
          previewItems: [
            { id: 1, filename: "flower.jef", filepath: "C:\\flower.jef" },
            { id: 2, filename: "heart.pes", filepath: "C:\\heart.pes" },
          ],
          open: true,
        },
      });

      expect(screen.getByText(/Review selected designs \(2\)/)).toBeInTheDocument();
    });
  });

  describe("file action radio", () => {
    it("defaults to keeping files on disk and hides the recycle-bin warning", () => {
      render(DeleteDesignsModal, { props: { designIds: [1], open: true } });

      const radios = screen.getAllByRole("radio");
      expect(radios).toHaveLength(2);
      expect(radios[0]).toBeChecked();
      expect(radios[1]).not.toBeChecked();
      expect(screen.queryByText(/will be moved to the system recycle bin/)).not.toBeInTheDocument();
    });

    it("checks the recycle-bin option and shows the warning when selected", async () => {
      render(DeleteDesignsModal, { props: { designIds: [1], open: true } });

      const radios = screen.getAllByRole("radio");
      await fireEvent.click(radios[1]);

      expect(radios[1]).toBeChecked();
      expect(radios[0]).not.toBeChecked();
      expect(screen.getByText(/will be moved to the system recycle bin/)).toBeInTheDocument();
    });

    it("hides the warning again when switching back to keep-files", async () => {
      render(DeleteDesignsModal, { props: { designIds: [1], open: true } });

      const radios = screen.getAllByRole("radio");
      await fireEvent.click(radios[1]);
      expect(screen.getByText(/will be moved to the system recycle bin/)).toBeInTheDocument();

      await fireEvent.click(radios[0]);

      expect(radios[0]).toBeChecked();
      expect(screen.queryByText(/will be moved to the system recycle bin/)).not.toBeInTheDocument();
    });
  });

  describe("cancel", () => {
    it("calls onClose when Cancel is clicked", async () => {
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("resets the deleteFile state when cancelled", async () => {
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.click(screen.getAllByRole("radio")[1]);
      expect(screen.getByText(/will be moved to the system recycle bin/)).toBeInTheDocument();

      await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

      expect(onClose).toHaveBeenCalledTimes(1);
      expect(screen.queryByText(/will be moved to the system recycle bin/)).not.toBeInTheDocument();
    });
  });

  describe("backdrop", () => {
    it("calls onClose when the backdrop is clicked", async () => {
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.click(screen.getByLabelText("Close delete confirmation"));

      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("ignores backdrop clicks while a deletion is in progress", async () => {
      const { promise, resolve } = deferred<typeof successResult>();
      mockedBulkDeleteDesigns.mockReturnValue(promise);
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));
      await fireEvent.click(screen.getByLabelText("Close delete confirmation"));

      expect(onClose).not.toHaveBeenCalled();
      resolve(successResult);
    });
  });

  describe("keyboard", () => {
    it("closes the modal when Escape is pressed", async () => {
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });

      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("ignores Escape while a deletion is in progress", async () => {
      const { promise, resolve } = deferred<typeof successResult>();
      mockedBulkDeleteDesigns.mockReturnValue(promise);
      const onClose = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));
      await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });

      expect(onClose).not.toHaveBeenCalled();
      resolve(successResult);
    });
  });

  describe("delete button", () => {
    it("is disabled when no designs are selected", () => {
      render(DeleteDesignsModal, { props: { designIds: [], open: true } });

      expect(screen.getByRole("button", { name: "Delete 0 designs" })).toBeDisabled();
    });

    it("is enabled when designs are selected", () => {
      render(DeleteDesignsModal, { props: { designIds: [1], open: true } });

      expect(screen.getByRole("button", { name: "Delete 1 design" })).toBeEnabled();
    });

    it("is disabled and shows 'Deleting...' while a deletion is in progress", async () => {
      const { promise, resolve } = deferred<typeof successResult>();
      mockedBulkDeleteDesigns.mockReturnValue(promise);
      render(DeleteDesignsModal, { props: { designIds: [1, 2], open: true } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 2 designs" }));

      const deletingButton = screen.getByRole("button", { name: "Deleting..." });
      expect(deletingButton).toBeDisabled();
      expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
      resolve(successResult);
    });
  });

  describe("confirmDelete", () => {
    it("calls bulkDeleteDesigns with the design ids and deleteFiles=false by default", async () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2], open: true } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 2 designs" }));

      expect(mockedBulkDeleteDesigns).toHaveBeenCalledWith([1, 2], false);
    });

    it("calls bulkDeleteDesigns with deleteFiles=true when recycle-bin is selected", async () => {
      render(DeleteDesignsModal, { props: { designIds: [42], open: true } });

      await fireEvent.click(screen.getAllByRole("radio")[1]);
      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));

      expect(mockedBulkDeleteDesigns).toHaveBeenCalledWith([42], true);
    });

    it("passes the persisted result to onDeleted", async () => {
      const onDeleted = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1, 2], open: true, onDeleted } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 2 designs" }));

      await waitFor(() => {
        expect(onDeleted).toHaveBeenCalledWith(successResult);
      });
    });

    it("fires onDeleted even when the result is not fully persisted", async () => {
      mockedBulkDeleteDesigns.mockResolvedValue({
        source: "mock",
        persisted: false,
        deleted_count: 0,
        files_trashed: 0,
        errors: [],
      });
      const onDeleted = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [9], open: true, onDeleted } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));

      await waitFor(() => {
        expect(onDeleted).toHaveBeenCalledWith({
          source: "mock",
          persisted: false,
          deleted_count: 0,
          files_trashed: 0,
          errors: [],
        });
      });
    });

    it("does not call onClose after a successful deletion", async () => {
      const onClose = vi.fn();
      const onDeleted = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [1], open: true, onClose, onDeleted } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));

      await waitFor(() => {
        expect(onDeleted).toHaveBeenCalled();
      });
      expect(onClose).not.toHaveBeenCalled();
    });

    it("calls onDeleted with an error result when bulkDeleteDesigns throws", async () => {
      mockedBulkDeleteDesigns.mockRejectedValue(new Error("backend unreachable"));
      const onDeleted = vi.fn();
      render(DeleteDesignsModal, { props: { designIds: [5], open: true, onDeleted } });

      await fireEvent.click(screen.getByRole("button", { name: "Delete 1 design" }));

      await waitFor(() => {
        expect(onDeleted).toHaveBeenCalledWith({
          source: "mock",
          persisted: false,
          deleted_count: 0,
          files_trashed: 0,
          errors: ["Error: backend unreachable"],
        });
      });
    });
  });

  describe("preview items", () => {
    const previewItems = [
      {
        id: 1,
        filename: "flower.jef",
        filepath: "C:\\designs\\flower.jef",
        dataUrl: "data:image/png;base64,Zm9v",
      },
      {
        id: 2,
        filename: "heart.pes",
        filepath: "C:\\designs\\heart.pes",
        dataUrl: null,
      },
    ];

    it("shows filenames and filepaths for each preview item", () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2], previewItems, open: true } });

      expect(screen.getByText("flower.jef")).toBeInTheDocument();
      expect(screen.getByText("C:\\designs\\flower.jef")).toBeInTheDocument();
      expect(screen.getByText("heart.pes")).toBeInTheDocument();
      expect(screen.getByText("C:\\designs\\heart.pes")).toBeInTheDocument();
    });

    it("shows a thumbnail image for preview items with a dataUrl", () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2], previewItems, open: true } });

      const img = screen.getByAltText("flower.jef");
      expect(img).toHaveAttribute("src", "data:image/png;base64,Zm9v");
    });

    it("shows a placeholder for preview items without a dataUrl", () => {
      render(DeleteDesignsModal, { props: { designIds: [1, 2], previewItems, open: true } });

      expect(screen.getByText("?")).toBeInTheDocument();
    });
  });
});
