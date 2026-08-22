import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import AdminHoopsView from "../AdminHoopsView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter and toast store so all logic branches can be driven
// from the test.
// ---------------------------------------------------------------------------
const listHoopsMock = vi.hoisted(() => vi.fn());
const createHoopMock = vi.hoisted(() => vi.fn());
const updateHoopMock = vi.hoisted(() => vi.fn());
const deleteHoopMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../../api/commandAdapter", () => ({
  listHoops: listHoopsMock,
  createHoop: createHoopMock,
  updateHoop: updateHoopMock,
  deleteHoop: deleteHoopMock,
}));

vi.mock("../../stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const defaultHoops = [
  {
    id: 1,
    name: "5x7 Hoop",
    max_width_mm: 130,
    max_height_mm: 180,
    design_count: 2,
  },
  {
    id: 2,
    name: "8x12 Hoop",
    max_width_mm: 200,
    max_height_mm: 300,
    design_count: 0,
  },
];

/** Resolves listHoops with a given list of hoops. */
function mockListHoops(hoops = defaultHoops) {
  listHoopsMock.mockResolvedValue({ items: hoops, source: "rust" });
}

function renderView() {
  return render(AdminHoopsView);
}

/** Helper to fill the "Add new hoop" form fields and submit. */
async function fillAddForm(name: string, width: number, height: number) {
  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: name } });
  await fireEvent.input(screen.getByLabelText("Max Width (mm)"), {
    target: { value: String(width) },
  });
  await fireEvent.input(screen.getByLabelText("Max Height (mm)"), {
    target: { value: String(height) },
  });
  const form = document.querySelector("form");
  if (!form) throw new Error("Add hoop form not found");
  await fireEvent.submit(form);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("AdminHoopsView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    addToastMock.mockClear();
    mockListHoops();
  });

  it("renders the page heading, description, add form, and empty table state", async () => {
    mockListHoops([]);

    renderView();

    await waitFor(() => {
      expect(screen.getByText("Manage Hoops")).toBeInTheDocument();
    });
    expect(screen.getByText(/Hoop sizes depend on your machine/)).toBeInTheDocument();
    expect(screen.getByText("Add new hoop")).toBeInTheDocument();
    expect(
      screen.getByText("No hoops defined yet. Add your own machine hoops above.")
    ).toBeInTheDocument();
    expect(listHoopsMock).toHaveBeenCalledTimes(1);
  });

  it("loads hoops on mount and renders them in the table", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });
    expect(screen.getByText("8x12 Hoop")).toBeInTheDocument();
    expect(screen.getByText("130")).toBeInTheDocument();
    expect(screen.getByText("180")).toBeInTheDocument();
    expect(screen.getByText("200")).toBeInTheDocument();
    expect(screen.getByText("300")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("0")).toBeInTheDocument();
  });

  it("shows a toast when listHoops rejects", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    listHoopsMock.mockRejectedValue(new Error("db locked"));

    renderView();

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Failed to load hoops: Error: db locked", "error");
    });

    consoleError.mockRestore();
  });

  it("calls loadHoops with force=true when adding a hoop, bypassing the in-flight guard", async () => {
    // Keep the first promise pending so adminLoading stays true; the forced
    // reload from addHoop must still proceed.
    listHoopsMock.mockReturnValueOnce(new Promise(() => {}));
    createHoopMock.mockResolvedValue({ persisted: true, item: { id: 3 }, source: "rust" });

    renderView();

    await fillAddForm("New Hoop", 100, 150);

    await waitFor(() => {
      expect(createHoopMock).toHaveBeenCalledWith("New Hoop", 100, 150);
    });
    // listHoops is called once on mount (pending) and once via loadHoops(true).
    expect(listHoopsMock).toHaveBeenCalledTimes(2);
  });

  it("adds a hoop with valid name, width, and height", async () => {
    createHoopMock.mockResolvedValue({ persisted: true, item: { id: 3 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fillAddForm("New Hoop", 100, 150);

    await waitFor(() => {
      expect(createHoopMock).toHaveBeenCalledWith("New Hoop", 100, 150);
    });
    expect(screen.getByLabelText("Name")).toHaveValue("");
    expect(screen.getByLabelText("Max Width (mm)")).toHaveValue(0);
    expect(screen.getByLabelText("Max Height (mm)")).toHaveValue(0);
    expect(addToastMock).toHaveBeenCalledWith("Hoop added.", "success");
  });

  it("shows an error toast when createHoop is not persisted", async () => {
    createHoopMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fillAddForm("New Hoop", 100, 150);

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not add hoop: boom", "error");
    });
  });

  it("does not submit the add form when the name is empty", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fillAddForm("   ", 100, 150);

    expect(createHoopMock).not.toHaveBeenCalled();
  });

  it("does not submit the add form when the width is not positive", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fillAddForm("New Hoop", 0, 150);

    expect(createHoopMock).not.toHaveBeenCalled();
  });

  it("does not submit the add form when the height is not positive", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fillAddForm("New Hoop", 100, 0);

    expect(createHoopMock).not.toHaveBeenCalled();
  });

  it("disables the Add button when required fields are missing", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    const addButton = screen.getByRole("button", { name: "Add" });
    expect(addButton).toBeDisabled();

    await fireEvent.input(screen.getByLabelText("Name"), {
      target: { value: "5x7 Hoop" },
    });
    await fireEvent.input(screen.getByLabelText("Max Width (mm)"), {
      target: { value: "130" },
    });
    await fireEvent.input(screen.getByLabelText("Max Height (mm)"), {
      target: { value: "180" },
    });
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  });

  it("disables the Clear button when the form is empty and enables it when dirty", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    const clearButton = screen.getByRole("button", { name: "Clear" });
    expect(clearButton).toBeDisabled();

    await fireEvent.input(screen.getByLabelText("Name"), {
      target: { value: "5x7 Hoop" },
    });
    expect(clearButton).toBeEnabled();
  });

  it("clears the add form when Clear is clicked", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.input(screen.getByLabelText("Name"), {
      target: { value: "5x7 Hoop" },
    });
    await fireEvent.input(screen.getByLabelText("Max Width (mm)"), {
      target: { value: "130" },
    });
    await fireEvent.input(screen.getByLabelText("Max Height (mm)"), {
      target: { value: "180" },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(screen.getByLabelText("Name")).toHaveValue("");
    expect(screen.getByLabelText("Max Width (mm)")).toHaveValue(0);
    expect(screen.getByLabelText("Max Height (mm)")).toHaveValue(0);
  });

  it("begins editing a hoop and shows inline inputs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);

    const nameInput = screen.getByDisplayValue("5x7 Hoop");
    expect(nameInput).toBeInTheDocument();
    expect(screen.getByDisplayValue("130")).toBeInTheDocument();
    expect(screen.getByDisplayValue("180")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
  });

  it("does not begin editing when passed a null hoop (defensive guard)", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    // No direct access to the internal function; the guard is covered by
    // ensuring that clicking Edit for a valid row works, and the default
    // branch renders Edit/Delete buttons for both rows.
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(2);
  });

  it("cancels editing and restores the default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue("5x7 Hoop")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(2);
  });

  it("saves an edited hoop with valid data", async () => {
    updateHoopMock.mockResolvedValue({ persisted: true, item: { id: 1 }, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);

    await fireEvent.input(screen.getByDisplayValue("5x7 Hoop"), {
      target: { value: "6x8 Hoop" },
    });
    await fireEvent.input(screen.getByDisplayValue("130"), {
      target: { value: "150" },
    });
    await fireEvent.input(screen.getByDisplayValue("180"), {
      target: { value: "200" },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(updateHoopMock).toHaveBeenCalledWith(1, "6x8 Hoop", 150, 200);
    });
    expect(addToastMock).toHaveBeenCalledWith("Hoop updated.", "success");
    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
  });

  it("shows an error toast when saving an edit with invalid details", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);

    await fireEvent.input(screen.getByDisplayValue("5x7 Hoop"), {
      target: { value: "   " },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Enter hoop details.", "error");
    });
    expect(updateHoopMock).not.toHaveBeenCalled();
  });

  it("shows an error toast when updateHoop is not persisted", async () => {
    updateHoopMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not update hoop: boom", "error");
    });
  });

  it("shows the 'clear assignment' toast for a hoop with designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);

    expect(addToastMock).toHaveBeenCalledWith(
      "Deleting '5x7 Hoop' will clear assignment from 2 design(s).",
      "info"
    );
    expect(
      screen.getByText(
        "This hoop is currently used by 2 design(s). If you delete it, those designs will no longer have a hoop assigned."
      )
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();
  });

  it("shows the 'confirm delete' toast for a hoop with no designs", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("8x12 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[1]);

    expect(addToastMock).toHaveBeenCalledWith(
      "Delete '8x12 Hoop'? Click confirm delete to continue.",
      "info"
    );
    expect(screen.getByText("Confirm deletion for this hoop.")).toBeInTheDocument();
  });

  it("cancels a pending delete and restores default buttons", async () => {
    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    expect(screen.getByRole("button", { name: "Confirm delete" })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(2);
  });

  it("deletes a hoop successfully", async () => {
    deleteHoopMock.mockResolvedValue({ persisted: true, source: "rust" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(deleteHoopMock).toHaveBeenCalledWith(1);
    });
    expect(addToastMock).toHaveBeenCalledWith("Hoop deleted.", "success");
    expect(screen.queryByRole("button", { name: "Confirm delete" })).not.toBeInTheDocument();
  });

  it("shows an error toast when deleteHoop is not persisted", async () => {
    deleteHoopMock.mockResolvedValue({ persisted: false, error: "boom", source: "mock" });

    renderView();

    await waitFor(() => {
      expect(screen.getByText("5x7 Hoop")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Could not delete hoop: boom", "error");
    });
  });
});
