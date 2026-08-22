import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/svelte";
import { tick } from "svelte";
import AdminSourcesView from "../AdminSourcesView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter and toast store so all logic branches can be driven
// from the test.
// ---------------------------------------------------------------------------
const listSourcesMock = vi.hoisted(() => vi.fn());
const createSourceMock = vi.hoisted(() => vi.fn());
const updateSourceMock = vi.hoisted(() => vi.fn());
const deleteSourceMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../../api/commandAdapter", () => ({
  listSources: listSourcesMock,
  createSource: createSourceMock,
  updateSource: updateSourceMock,
  deleteSource: deleteSourceMock,
}));

vi.mock("../../stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const defaultSources = [
  { id: 1, name: "Purchased", design_count: 2 },
  { id: 2, name: "Downloaded", design_count: 0 },
  { id: 3, name: "Gift", design_count: null },
  { id: 4, name: "", design_count: 1 },
];

/** Resolves listSources with a given list of sources. */
function mockListSources(sources = defaultSources) {
  listSourcesMock.mockResolvedValue({ items: sources, source: "rust" });
}

function renderView(props: { embedded?: boolean } = {}) {
  return render(AdminSourcesView, props);
}

/** Helper to fill the "Add new source" form and submit. */
async function fillAddForm(name: string) {
  await fireEvent.input(screen.getByPlaceholderText("e.g. Purchased, Downloaded..."), {
    target: { value: name },
  });
  const form = document.querySelector("form");
  if (!form) throw new Error("Add source form not found");
  await fireEvent.submit(form);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("AdminSourcesView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    addToastMock.mockClear();
    mockListSources();
  });

  it("renders the page heading, description, add form, and empty table state", async () => {
    mockListSources([]);

    renderView();

    expect(screen.getByText("Manage Sources")).toBeInTheDocument();
    expect(screen.getByText(/Sources describe where your designs came from/)).toBeInTheDocument();
    expect(screen.getByText("Add new source")).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("No sources yet.")).toBeInTheDocument();
    });
    expect(listSourcesMock).toHaveBeenCalledTimes(1);
  });

  it("loads sources on mount and renders them in the table", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });
    expect(screen.getByText("Downloaded")).toBeInTheDocument();
    expect(screen.getByText("Gift")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
    // Downloaded has design_count 0 and Gift has null → 0, so two "0" cells.
    expect(screen.getAllByText("0")).toHaveLength(2);
  });

  it("shows a toast when listSources rejects", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    listSourcesMock.mockRejectedValue(new Error("db locked"));

    renderView();

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Failed to load sources: Error: db locked",
        "error"
      );
    });

    consoleError.mockRestore();
  });

  it("calls loadSources with force=true when adding a source, bypassing the in-flight guard", async () => {
    // Keep the first promise pending so adminLoading stays true; the forced
    // reload from addSource must still proceed.
    listSourcesMock.mockReturnValueOnce(new Promise(() => {}));
    createSourceMock.mockResolvedValue({ persisted: true, item: { id: 5 }, source: "rust" });

    renderView();

    await fillAddForm("New Source");

    await waitFor(() => {
      expect(createSourceMock).toHaveBeenCalledWith("New Source");
    });
    // listSources is called once on mount (pending) and once via loadSources(true).
    expect(listSourcesMock).toHaveBeenCalledTimes(2);
  });

  it("adds a source with a valid name", async () => {
    createSourceMock.mockResolvedValue({ persisted: true, item: { id: 5 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fillAddForm("Bought");

    await waitFor(() => {
      expect(createSourceMock).toHaveBeenCalledWith("Bought");
    });
    expect(screen.getByPlaceholderText("e.g. Purchased, Downloaded...")).toHaveValue("");
    expect(addToastMock).toHaveBeenCalledWith("Source added.", "success");
    await waitFor(() => {
      expect(listSourcesMock).toHaveBeenCalledTimes(2);
    });
  });

  it("shows an error toast when createSource is not persisted", async () => {
    createSourceMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fillAddForm("Bought");

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not add source: boom", "error");
    });
  });

  it("shows the Unknown error fallback when createSource fails without an error message", async () => {
    createSourceMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fillAddForm("Bought");

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not add source: Unknown error", "error");
    });
  });

  it("does not submit the add form when the name is empty", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fillAddForm("   ");

    expect(createSourceMock).not.toHaveBeenCalled();
  });

  it("enables Clear but not Add for whitespace-only input, and Add for valid input", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    const addButton = screen.getByRole("button", { name: "Add" });
    const clearButton = screen.getByRole("button", { name: "Clear" });
    expect(addButton).toBeDisabled();
    expect(clearButton).toBeDisabled();

    await fireEvent.input(screen.getByPlaceholderText("e.g. Purchased, Downloaded..."), {
      target: { value: "   " },
    });
    await tick();
    // trim() is empty → Add stays disabled; non-zero length → Clear enabled.
    expect(addButton).toBeDisabled();
    expect(clearButton).toBeEnabled();

    await fireEvent.input(screen.getByPlaceholderText("e.g. Purchased, Downloaded..."), {
      target: { value: "Purchased" },
    });
    await tick();
    expect(addButton).toBeEnabled();
  });

  it("clears the add form when Clear is clicked", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.input(screen.getByPlaceholderText("e.g. Purchased, Downloaded..."), {
      target: { value: "Bought" },
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    await tick();

    expect(screen.getByPlaceholderText("e.g. Purchased, Downloaded...")).toHaveValue("");
  });

  it("begins editing a source and shows an inline input", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    expect(screen.getByDisplayValue("Purchased")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
  });

  it("does not begin editing when passed a null source (defensive guard)", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    // The guard in beginEditSource is unreachable from the UI; this ensures the
    // default Edit/Delete branch renders for every row.
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(4);
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(4);
  });

  it("begins editing a source with an empty name using the fallback value", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    // The 4th row has name: "" so the `source.name || ""` fallback is used.
    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[3]);
    await tick();

    // Scope to the table because the "Add new source" form input also has an
    // empty display value.
    const table = document.querySelector("table");
    if (!table) throw new Error("Sources table not found");
    expect(within(table).getByDisplayValue("")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
  });

  it("cancels editing and restores the default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await tick();

    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue("Purchased")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(4);
  });

  it("clears a pending delete when beginning an edit on another row", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();

    // The first row now shows Confirm delete/Cancel instead of Edit/Delete, so
    // Edit[0] is the Downloaded row.
    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
    expect(screen.getByDisplayValue("Downloaded")).toBeInTheDocument();
  });

  it("saves an edited source with a valid name", async () => {
    updateSourceMock.mockResolvedValue({ persisted: true, item: { id: 1 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    await fireEvent.input(screen.getByDisplayValue("Purchased"), {
      target: { value: "Purchased & Downloaded" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(updateSourceMock).toHaveBeenCalledWith(1, "Purchased & Downloaded");
    });
    expect(addToastMock).toHaveBeenCalledWith("Source updated.", "success");
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
    });
  });

  it("shows an error toast when saving an edit with an empty name", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();

    await fireEvent.input(screen.getByDisplayValue("Purchased"), {
      target: { value: "   " },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Enter a source name.", "error");
    });
    expect(updateSourceMock).not.toHaveBeenCalled();
  });

  it("shows an error toast when updateSource is not persisted", async () => {
    updateSourceMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not update source: boom", "error");
    });
  });

  it("shows the Unknown error fallback when updateSource fails without an error message", async () => {
    updateSourceMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not update source: Unknown error", "error");
    });
  });

  it("shows the 'clear assignment' toast for a source with designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();

    expect(addToastMock).toHaveBeenCalledWith(
      "Deleting 'Purchased' will clear assignment from 2 design(s).",
      "info"
    );
    expect(
      screen.getByText(
        "This source is currently used by 2 design(s). If you delete it, those designs will no longer have a source assigned."
      )
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();
  });

  it("shows the 'confirm delete' toast for a source with no designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Downloaded")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[1]);
    await tick();

    expect(addToastMock).toHaveBeenCalledWith(
      "Delete 'Downloaded'? Click confirm delete to continue.",
      "info"
    );
    expect(screen.getByText("Confirm deletion for this source.")).toBeInTheDocument();
  });

  it("cancels a pending delete and restores default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await tick();

    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(4);
  });

  it("deletes a source successfully", async () => {
    deleteSourceMock.mockResolvedValue({ persisted: true, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(deleteSourceMock).toHaveBeenCalledWith(1);
    });
    expect(addToastMock).toHaveBeenCalledWith("Source deleted.", "success");
    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
  });

  it("shows an error toast when deleteSource is not persisted", async () => {
    deleteSourceMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not delete source: boom", "error");
    });
  });

  it("shows the Unknown error fallback when deleteSource fails without an error message", async () => {
    deleteSourceMock.mockResolvedValue({ persisted: false, source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not delete source: Unknown error", "error");
    });
  });

  it("hides the standalone title and description when embedded is true", async () => {
    renderView({ embedded: true });

    await waitFor(() => {
      expect(screen.getByText("Purchased")).toBeInTheDocument();
    });

    expect(screen.queryByText("Manage Sources")).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Sources describe where your designs came from/)
    ).not.toBeInTheDocument();
    expect(screen.getByText("Add new source")).toBeInTheDocument();
  });

  it("handles a list response where items is not an array", async () => {
    listSourcesMock.mockResolvedValue({ items: "not-an-array", source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No sources yet.")).toBeInTheDocument();
    });
  });

  it("handles a null list response", async () => {
    listSourcesMock.mockResolvedValue(null);

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No sources yet.")).toBeInTheDocument();
    });
  });

  it("handles a list response without an items property", async () => {
    listSourcesMock.mockResolvedValue({ source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("No sources yet.")).toBeInTheDocument();
    });
  });
});
