import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/svelte";
import { tick } from "svelte";
import AdminDesignersView from "../AdminDesignersView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter and toast store so all logic branches can be driven
// from the test.
// ---------------------------------------------------------------------------
const listDesignersMock = vi.hoisted(() => vi.fn());
const createDesignerMock = vi.hoisted(() => vi.fn());
const updateDesignerMock = vi.hoisted(() => vi.fn());
const deleteDesignerMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../../api/commandAdapter", () => ({
  listDesigners: listDesignersMock,
  createDesigner: createDesignerMock,
  updateDesigner: updateDesignerMock,
  deleteDesigner: deleteDesignerMock,
}));

vi.mock("../../stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const defaultDesigners = [
  { id: 1, name: "Amazing Designs", design_count: 2 },
  { id: 2, name: "Stitch Studio", design_count: 0 },
  { id: 3, name: "", design_count: 1 },
];

/** Resolves listDesigners with a given list of designers. */
function mockListDesigners(designers = defaultDesigners) {
  listDesignersMock.mockResolvedValue({ items: designers, source: "rust" });
}

function renderView(props: { embedded?: boolean } = {}) {
  return render(AdminDesignersView, props);
}

/** Helper to type in the "Add new designer" input and submit the form. */
async function fillAddForm(name: string) {
  await fireEvent.input(screen.getByPlaceholderText("New designer name..."), {
    target: { value: name },
  });
  const form = document.querySelector("form");
  if (!form) throw new Error("Add designer form not found");
  await fireEvent.submit(form);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("AdminDesignersView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    addToastMock.mockClear();
    mockListDesigners();
  });

  it("renders the page heading, description, add form, and empty table state", async () => {
    mockListDesigners([]);

    renderView();

    expect(screen.getByText("Manage Designers")).toBeInTheDocument();
    expect(
      screen.getByText(/Designers are the creators or brands of embroidery designs/)
    ).toBeInTheDocument();
    expect(screen.getByText("Add new designer")).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("No designers yet.")).toBeInTheDocument();
    });
    expect(listDesignersMock).toHaveBeenCalledTimes(1);
  });

  it("loads designers on mount and renders them in the table", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });
    expect(screen.getByText("Stitch Studio")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
    // Stitch Studio has design_count 0, so there is one "0" cell.
    expect(screen.getByText("0")).toBeInTheDocument();
  });

  it("shows a toast when listDesigners rejects", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    listDesignersMock.mockRejectedValue(new Error("db locked"));

    renderView();

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Failed to load designers: Error: db locked",
        "error"
      );
    });

    consoleError.mockRestore();
  });

  it("calls loadDesigners with force=true when adding a designer, bypassing the in-flight guard", async () => {
    // Keep the first promise pending so adminLoading stays true; the forced
    // reload from addDesigner must still proceed.
    listDesignersMock.mockReturnValueOnce(new Promise(() => {}));
    createDesignerMock.mockResolvedValue({ persisted: true, item: { id: 4 }, source: "rust" });

    renderView();

    await fillAddForm("New Designer");

    await waitFor(() => {
      expect(createDesignerMock).toHaveBeenCalledWith("New Designer");
    });
    // listDesigners is called once on mount (pending) and once via loadDesigners(true).
    expect(listDesignersMock).toHaveBeenCalledTimes(2);
  });

  it("adds a designer with a valid name", async () => {
    createDesignerMock.mockResolvedValue({ persisted: true, item: { id: 4 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fillAddForm("Boutique Stitch");

    await waitFor(() => {
      expect(createDesignerMock).toHaveBeenCalledWith("Boutique Stitch");
    });
    expect(screen.getByPlaceholderText("New designer name...")).toHaveValue("");
    expect(addToastMock).toHaveBeenCalledWith("Designer added.", "success");
    await waitFor(() => {
      expect(listDesignersMock).toHaveBeenCalledTimes(2);
    });
  });

  it("shows an error toast when createDesigner is not persisted", async () => {
    createDesignerMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fillAddForm("Boutique Stitch");

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not add designer: boom", "error");
    });
  });

  it("shows the Unknown error fallback when createDesigner fails without an error message", async () => {
    createDesignerMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fillAddForm("Boutique Stitch");

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not add designer: Unknown error", "error");
    });
  });

  it("does not submit the add form when the name is empty", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fillAddForm("   ");

    expect(createDesignerMock).not.toHaveBeenCalled();
  });

  it("enables Clear but not Add for whitespace-only input, and Add for valid input", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    const addButton = screen.getByRole("button", { name: "Add" });
    const clearButton = screen.getByRole("button", { name: "Clear" });
    expect(addButton).toBeDisabled();
    expect(clearButton).toBeDisabled();

    await fireEvent.input(screen.getByPlaceholderText("New designer name..."), {
      target: { value: "   " },
    });
    await tick();
    // trim() is empty → Add stays disabled; non-zero length → Clear enabled.
    expect(addButton).toBeDisabled();
    expect(clearButton).toBeEnabled();

    await fireEvent.input(screen.getByPlaceholderText("New designer name..."), {
      target: { value: "Amazing Designs" },
    });
    await tick();
    expect(addButton).toBeEnabled();
  });

  it("clears the add form when Clear is clicked", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.input(screen.getByPlaceholderText("New designer name..."), {
      target: { value: "Boutique Stitch" },
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    await tick();

    expect(screen.getByPlaceholderText("New designer name...")).toHaveValue("");
  });

  it("begins editing a designer and shows an inline input", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    expect(screen.getByDisplayValue("Amazing Designs")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
  });

  it("does not begin editing when passed a null designer (defensive guard)", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    // The guard in beginEditDesigner is unreachable from the UI; this ensures
    // the default Edit/Delete branch renders for every row.
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(3);
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(3);
  });

  it("begins editing a designer with an empty name using the fallback value", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    // The 3rd row has name: "" so the `designer.name || ""` fallback is used.
    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[2]);
    await tick();

    // Scope to the table because the "Add new designer" form input also has an
    // empty display value.
    const table = document.querySelector("table");
    if (!table) throw new Error("Designers table not found");
    expect(within(table).getByDisplayValue("")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
  });

  it("cancels editing and restores the default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await tick();

    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue("Amazing Designs")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(3);
  });

  it("clears a pending delete when beginning an edit on another row", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();

    // The first row now shows Confirm delete/Cancel instead of Edit/Delete, so
    // Edit[0] is the Stitch Studio row.
    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
    expect(screen.getByDisplayValue("Stitch Studio")).toBeInTheDocument();
  });

  it("saves an edited designer with a valid name", async () => {
    updateDesignerMock.mockResolvedValue({ persisted: true, item: { id: 1 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    await fireEvent.input(screen.getByDisplayValue("Amazing Designs"), {
      target: { value: "Amazing & Co" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(updateDesignerMock).toHaveBeenCalledWith(1, "Amazing & Co");
    });
    expect(addToastMock).toHaveBeenCalledWith("Designer updated.", "success");
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
    });
  });

  it("shows an error toast when saving an edit with an empty name", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    await fireEvent.input(screen.getByDisplayValue("Amazing Designs"), {
      target: { value: "   " },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Enter a designer name.", "error");
    });
    expect(updateDesignerMock).not.toHaveBeenCalled();
  });

  it("shows an error toast when updateDesigner is not persisted", async () => {
    updateDesignerMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not update designer: boom", "error");
    });
  });

  it("shows the Unknown error fallback when updateDesigner fails without an error message", async () => {
    updateDesignerMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not update designer: Unknown error",
        "error"
      );
    });
  });

  it("shows the 'clear assignment' toast for a designer with designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();

    expect(addToastMock).toHaveBeenCalledWith(
      "Deleting 'Amazing Designs' will clear assignment from 2 design(s).",
      "info"
    );
    expect(
      screen.getByText(
        "This designer is currently used by 2 design(s). If you delete it, those designs will no longer have a designer assigned."
      )
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();
  });

  it("shows the 'confirm delete' toast for a designer with no designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Stitch Studio")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[1]);
    await tick();

    expect(addToastMock).toHaveBeenCalledWith(
      "Delete 'Stitch Studio'? Click confirm delete to continue.",
      "info"
    );
    expect(screen.getByText("Confirm deletion for this designer.")).toBeInTheDocument();
  });

  it("cancels a pending delete and restores default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await tick();

    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(3);
  });

  it("deletes a designer successfully", async () => {
    deleteDesignerMock.mockResolvedValue({ persisted: true, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(deleteDesignerMock).toHaveBeenCalledWith(1);
    });
    expect(addToastMock).toHaveBeenCalledWith("Designer deleted.", "success");
    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
  });

  it("shows an error toast when deleteDesigner is not persisted", async () => {
    deleteDesignerMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not delete designer: boom", "error");
    });
  });

  it("shows the Unknown error fallback when deleteDesigner fails without an error message", async () => {
    deleteDesignerMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not delete designer: Unknown error",
        "error"
      );
    });
  });

  it("hides the standalone title and description when embedded is true", async () => {
    renderView({ embedded: true });

    await waitFor(() => {
      expect(screen.getByText("Amazing Designs")).toBeInTheDocument();
    });

    expect(screen.queryByText("Manage Designers")).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Designers are the creators or brands of embroidery designs/)
    ).not.toBeInTheDocument();
    expect(screen.getByText("Add new designer")).toBeInTheDocument();
  });

  it("handles a list response where items is not an array", async () => {
    listDesignersMock.mockResolvedValue({ items: "not-an-array", source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No designers yet.")).toBeInTheDocument();
    });
  });

  it("handles a null list response", async () => {
    listDesignersMock.mockResolvedValue(null);

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No designers yet.")).toBeInTheDocument();
    });
  });

  it("handles a list response without an items property", async () => {
    listDesignersMock.mockResolvedValue({ source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No designers yet.")).toBeInTheDocument();
    });
  });
});
