import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent, within } from "@testing-library/svelte";
import OrphansView from "../OrphansView.svelte";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

/** Mock the command adapter so no real Tauri `invoke` calls are made. */
const adapterMocks = vi.hoisted(() => ({
  getOrphansPage: vi.fn(),
  deleteOrphans: vi.fn(),
  deleteAllOrphans: vi.fn(),
  browseOrphanPath: vi.fn(),
  openDesignInEditor: vi.fn(),
  scanOrphans: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

/** Mock the toast store. */
const toastMocks = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMocks);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const DEFAULT_ORPHAN_ITEMS = [
  {
    id: 1,
    filename: "rose.pes",
    filepath: "C:/Designs/rose.pes",
    designer: "Rose Studio",
    date_added: null,
  },
  {
    id: 2,
    filename: "missing.vp3",
    filepath: "C:/Designs/missing.vp3",
    designer: "Mock Designer",
    date_added: "2026-01-01",
  },
];

const orphanPageResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  page: 1,
  page_size: 100,
  total: 2,
  total_pages: 1,
  items: DEFAULT_ORPHAN_ITEMS,
  ...o,
});

const scanResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  checked: 0,
  found: 0,
  ...o,
});

const deleteResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  persisted: true,
  deleted: 2,
  ...o,
});

const browseResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  ok: true,
  opened: "",
  ...o,
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function mockDefaults() {
  adapterMocks.getOrphansPage.mockResolvedValue(orphanPageResponse());
  adapterMocks.deleteOrphans.mockResolvedValue(deleteResponse());
  adapterMocks.deleteAllOrphans.mockResolvedValue(deleteResponse());
  adapterMocks.browseOrphanPath.mockResolvedValue(browseResponse());
  adapterMocks.openDesignInEditor.mockResolvedValue({
    source: "rust",
    persisted: true,
    message: "Open in editor action completed.",
  });
  adapterMocks.scanOrphans.mockResolvedValue(scanResponse());
}

/** Wait for the initial orphan page load to settle (loading flag cleared). */
async function waitForLoaded() {
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Scan Disk" })).toBeInTheDocument()
  );
}

let confirmSpy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  vi.clearAllMocks();
  mockDefaults();
  confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
});

afterEach(() => {
  confirmSpy.mockRestore();
});

// ---------------------------------------------------------------------------
// Page chrome
// ---------------------------------------------------------------------------
describe("OrphansView page chrome", () => {
  it("renders the page heading 'Orphans'", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(screen.getByRole("heading", { name: "Orphans" })).toBeInTheDocument();
  });

  it("renders the description paragraph", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(
      screen.getByText("Find and remove database records whose files no longer exist on disk.")
    ).toBeInTheDocument();
  });

  it("renders all control buttons with default counts", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(screen.getByRole("button", { name: "Scan Disk" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Select all" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Delete selected (2)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Delete all (2)" })).toBeInTheDocument();
  });

  it("renders the table column headers", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(screen.getByRole("columnheader", { name: "Select" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "ID" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Filename" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Path" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Actions" })).toBeInTheDocument();
  });

  it("does not render pagination controls when there is a single page", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(
      screen.queryByRole("navigation", { name: "Orphans pagination" })
    ).not.toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Initial data loading
// ---------------------------------------------------------------------------
describe("OrphansView initial data loading", () => {
  it("calls getOrphansPage with page 1 and page size 100 on mount", async () => {
    render(OrphansView);
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1));
    expect(adapterMocks.getOrphansPage).toHaveBeenCalledWith({ page: 1, pageSize: 100 });
  });

  it("renders orphan rows with filename and filepath", async () => {
    render(OrphansView);
    await waitFor(() => expect(screen.getByText("rose.pes")).toBeInTheDocument());
    expect(screen.getByText("missing.vp3")).toBeInTheDocument();
    expect(screen.getByText("C:/Designs/rose.pes")).toBeInTheDocument();
    expect(screen.getByText("C:/Designs/missing.vp3")).toBeInTheDocument();
    expect(screen.getAllByRole("checkbox")).toHaveLength(2);
  });

  it("renders each orphan id in the id column", async () => {
    render(OrphansView);
    await waitFor(() => expect(screen.getAllByRole("row")).toHaveLength(3));

    const rows = screen.getAllByRole("row");
    expect(within(rows[1]).getByText("1")).toBeInTheDocument();
    expect(within(rows[2]).getByText("2")).toBeInTheDocument();
  });

  it("renders the orphan summary line", async () => {
    render(OrphansView);
    await waitForLoaded();
    expect(
      screen.getByText(/2 orphaned record\(s\) total, page 1 of 1, showing 2/)
    ).toBeInTheDocument();
  });

  it("renders 'Unknown' as the filename fallback when a filename is missing", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({
        total: 1,
        items: [
          {
            id: 7,
            filename: "",
            filepath: "C:/Designs/unknown-path.pes",
            designer: "",
            date_added: null,
          },
        ],
      })
    );
    render(OrphansView);
    await waitFor(() => expect(screen.getByText("Unknown")).toBeInTheDocument());
  });
});

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------
describe("OrphansView empty state", () => {
  it("shows the no-orphans message when the page has no items", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({ total: 0, total_pages: 1, items: [] })
    );
    render(OrphansView);
    await waitFor(() =>
      expect(
        screen.getByText("No orphaned records found. Refresh or scan to check.")
      ).toBeInTheDocument()
    );
  });

  it("disables select and delete-all controls when there are no orphans", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({ total: 0, total_pages: 1, items: [] })
    );
    render(OrphansView);
    await waitForLoaded();
    expect(screen.getByRole("button", { name: "Select all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Delete selected (0)" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Delete all (0)" })).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// Error banner
// ---------------------------------------------------------------------------
describe("OrphansView error banner", () => {
  it("shows the error banner and falls back to the empty state when loading fails", async () => {
    adapterMocks.getOrphansPage.mockRejectedValue(new Error("db down"));
    render(OrphansView);

    await waitFor(() =>
      expect(screen.getByText("Could not load orphans: Error: db down")).toBeInTheDocument()
    );
    expect(
      screen.getByText("No orphaned records found. Refresh or scan to check.")
    ).toBeInTheDocument();
    expect(
      screen.getByText(/0 orphaned record\(s\) total, page 1 of 1, showing 0/)
    ).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------
describe("OrphansView selection", () => {
  it("pre-selects all loaded orphans on page load", async () => {
    render(OrphansView);
    await waitForLoaded();

    const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
    expect(checkboxes).toHaveLength(2);
    expect(checkboxes.every((checkbox) => checkbox.checked)).toBe(true);
    expect(screen.getByRole("button", { name: "Delete selected (2)" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Select all" })).toBeEnabled();
  });

  it("toggles a single orphan selection off and updates the count", async () => {
    render(OrphansView);
    await waitForLoaded();

    const first = screen.getAllByRole("checkbox")[0] as HTMLInputElement;
    await fireEvent.click(first);
    expect(first.checked).toBe(false);
    expect(screen.getByRole("button", { name: "Delete selected (1)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeEnabled();
  });

  it("toggles a selection back on", async () => {
    render(OrphansView);
    await waitForLoaded();

    const first = screen.getAllByRole("checkbox")[0] as HTMLInputElement;
    await fireEvent.click(first);
    await fireEvent.click(first);
    expect(first.checked).toBe(true);
    expect(screen.getByRole("button", { name: "Delete selected (2)" })).toBeInTheDocument();
  });

  it("selects and deselects all orphans on the page", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Deselect all" }));
    const unchecked = screen.getAllByRole("checkbox") as HTMLInputElement[];
    expect(unchecked.every((checkbox) => !checkbox.checked)).toBe(true);
    expect(screen.getByRole("button", { name: "Delete selected (0)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeDisabled();

    await fireEvent.click(screen.getByRole("button", { name: "Select all" }));
    const rechecked = screen.getAllByRole("checkbox") as HTMLInputElement[];
    expect(rechecked.every((checkbox) => checkbox.checked)).toBe(true);
    expect(screen.getByRole("button", { name: "Delete selected (2)" })).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Delete selected orphans
// ---------------------------------------------------------------------------
describe("OrphansView delete selected orphans", () => {
  it("deletes the selected orphans and shows a success toast", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete selected (2)" }));
    expect(confirmSpy).toHaveBeenCalledWith(
      "Delete 2 selected record(s)? This cannot be undone."
    );
    await waitFor(() => expect(adapterMocks.deleteOrphans).toHaveBeenCalledWith([1, 2]));
    expect(toastMocks.addToast).toHaveBeenCalledWith("2 record(s) deleted.", "success");
  });

  it("deletes only the ids that remain selected", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getAllByRole("checkbox")[0]);
    await fireEvent.click(screen.getByRole("button", { name: "Delete selected (1)" }));
    await waitFor(() => expect(adapterMocks.deleteOrphans).toHaveBeenCalledWith([2]));
  });

  it("does not delete when the user cancels the confirmation", async () => {
    render(OrphansView);
    await waitForLoaded();

    confirmSpy.mockReturnValue(false);
    await fireEvent.click(screen.getByRole("button", { name: "Delete selected (2)" }));
    expect(adapterMocks.deleteOrphans).not.toHaveBeenCalled();
    expect(toastMocks.addToast).not.toHaveBeenCalled();
  });

  it("shows an error toast when the deletion is not persisted", async () => {
    adapterMocks.deleteOrphans.mockResolvedValue({
      source: "mock",
      persisted: false,
      deleted: 0,
      error: "disk failure",
    });
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete selected (2)" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not delete selected orphans: disk failure",
        "error"
      )
    );
    // No refresh happens when deletion fails.
    expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1);
  });

  it("refreshes the current page after a successful deletion", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete selected (2)" }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(2));
    expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 1, pageSize: 100 });
  });
});

// ---------------------------------------------------------------------------
// Delete all orphans
// ---------------------------------------------------------------------------
describe("OrphansView delete all orphans", () => {
  it("deletes all orphans after confirmation and shows a success toast", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete all (2)" }));
    expect(confirmSpy).toHaveBeenCalledWith(
      "Delete ALL {orphanTotal} orphaned records? This cannot be undone."
    );
    await waitFor(() => expect(adapterMocks.deleteAllOrphans).toHaveBeenCalledTimes(1));
    expect(toastMocks.addToast).toHaveBeenCalledWith("2 record(s) deleted.", "success");
  });

  it("refreshes page 1 after a successful delete-all", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete all (2)" }));
    await waitFor(() =>
      expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 1, pageSize: 100 })
    );
  });

  it("does not delete all when the user cancels the confirmation", async () => {
    render(OrphansView);
    await waitForLoaded();

    confirmSpy.mockReturnValue(false);
    await fireEvent.click(screen.getByRole("button", { name: "Delete all (2)" }));
    expect(adapterMocks.deleteAllOrphans).not.toHaveBeenCalled();
    expect(toastMocks.addToast).not.toHaveBeenCalled();
  });

  it("shows an error toast when delete-all is not persisted", async () => {
    adapterMocks.deleteAllOrphans.mockResolvedValue({
      source: "mock",
      persisted: false,
      deleted: 0,
      error: "db locked",
    });
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Delete all (2)" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not delete all orphans: db locked",
        "error"
      )
    );
    expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1);
  });
});

// ---------------------------------------------------------------------------
// Disk scan
// ---------------------------------------------------------------------------
describe("OrphansView disk scan", () => {
  it("shows an info toast and calls scanOrphans when the scan starts", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Scan Disk" }));
    expect(toastMocks.addToast).toHaveBeenCalledWith(
      "Scanning disk for orphaned records...",
      "info"
    );
    await waitFor(() => expect(adapterMocks.scanOrphans).toHaveBeenCalledTimes(1));
  });

  it("shows a success toast with checked and found counts after a scan", async () => {
    adapterMocks.scanOrphans.mockResolvedValue(scanResponse({ checked: 12, found: 3 }));
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Scan Disk" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Scan complete. Checked 12 file record(s). Found 3 orphan(s).",
        "success"
      )
    );
  });

  it("reports a non-rust scan failure and re-enables the scan button", async () => {
    adapterMocks.scanOrphans.mockResolvedValue(
      scanResponse({ source: "mock", error: "scan unavailable" })
    );
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Scan Disk" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not complete scan: Error: scan unavailable",
        "error"
      )
    );
    await waitFor(() => expect(screen.getByRole("button", { name: "Scan Disk" })).toBeEnabled());
  });

  it("shows an error toast when scanOrphans rejects", async () => {
    adapterMocks.scanOrphans.mockRejectedValue(new Error("scanner crashed"));
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Scan Disk" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not complete scan: Error: scanner crashed",
        "error"
      )
    );
  });

  it("shows 'Scanning...' on the button while the scan is running", async () => {
    let resolveScan!: (v: unknown) => void;
    adapterMocks.scanOrphans.mockReturnValue(
      new Promise((r) => {
        resolveScan = r;
      })
    );
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Scan Disk" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Scanning..." })).toBeInTheDocument()
    );

    resolveScan(scanResponse({ checked: 5, found: 1 }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Scan complete. Checked 5 file record(s). Found 1 orphan(s).",
        "success"
      )
    );
  });
});

// ---------------------------------------------------------------------------
// Browse orphan path
// ---------------------------------------------------------------------------
describe("OrphansView browse orphan path", () => {
  it("shows a success toast when the folder opens", async () => {
    adapterMocks.browseOrphanPath.mockResolvedValue(
      browseResponse({ opened: "C:\\Designs" })
    );
    render(OrphansView);
    await waitFor(() => expect(screen.getAllByText("Locate Folder")).toHaveLength(2));

    await fireEvent.click(screen.getAllByText("Locate Folder")[0]);
    await waitFor(() =>
      expect(adapterMocks.browseOrphanPath).toHaveBeenCalledWith("C:/Designs/rose.pes")
    );
    expect(toastMocks.addToast).toHaveBeenCalledWith("Opened: C:\\Designs", "success");
  });

  it("shows an error toast when the folder cannot be opened", async () => {
    adapterMocks.browseOrphanPath.mockResolvedValue(
      browseResponse({ source: "mock", ok: false, error: "No such path" })
    );
    render(OrphansView);
    await waitFor(() => expect(screen.getAllByText("Locate Folder")).toHaveLength(2));

    await fireEvent.click(screen.getAllByText("Locate Folder")[0]);
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not open folder: No such path",
        "error"
      )
    );
  });

  it("uses a fallback message when no error is returned", async () => {
    adapterMocks.browseOrphanPath.mockResolvedValue(
      browseResponse({ source: "mock", ok: false, opened: "" })
    );
    render(OrphansView);
    await waitFor(() => expect(screen.getAllByText("Locate Folder")).toHaveLength(2));

    await fireEvent.click(screen.getAllByText("Locate Folder")[0]);
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Could not open folder: Unknown error",
        "error"
      )
    );
  });
});

// ---------------------------------------------------------------------------
// Open design in editor
// ---------------------------------------------------------------------------
describe("OrphansView open design in editor", () => {
  it("opens the design in the system editor when the filename is clicked", async () => {
    render(OrphansView);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "rose.pes" })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole("button", { name: "rose.pes" }));
    expect(adapterMocks.openDesignInEditor).toHaveBeenCalledWith(1);
  });

  it("does nothing when the orphan id is invalid", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({
        total: 1,
        items: [
          {
            id: -5,
            filename: "bad.pes",
            filepath: "C:/Designs/bad.pes",
            designer: "",
            date_added: null,
          },
        ],
      })
    );
    render(OrphansView);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "bad.pes" })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole("button", { name: "bad.pes" }));
    expect(adapterMocks.openDesignInEditor).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------
describe("OrphansView pagination", () => {
  it("renders pagination controls when there is more than one page", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({ total: 250, total_pages: 3 })
    );
    render(OrphansView);
    await waitFor(() =>
      expect(
        screen.getByRole("navigation", { name: "Orphans pagination" })
      ).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: /First/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Next/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Last/ })).toBeInTheDocument();
  });

  it("shows the current page in the summary line", async () => {
    adapterMocks.getOrphansPage.mockResolvedValue(
      orphanPageResponse({ total: 250, total_pages: 3 })
    );
    render(OrphansView);
    await waitFor(() =>
      expect(
        screen.getByText(/250 orphaned record\(s\) total, page 1 of 3, showing 2/)
      ).toBeInTheDocument()
    );
  });

  it("loads the next page when Next is clicked", async () => {
    adapterMocks.getOrphansPage.mockImplementation(({ page }: { page: number }) =>
      Promise.resolve(orphanPageResponse({ page, total: 250, total_pages: 3 }))
    );
    render(OrphansView);
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1));

    await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(2));
    expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 2, pageSize: 100 });
  });

  it("loads the previous page when Prev is clicked", async () => {
    adapterMocks.getOrphansPage.mockImplementation(({ page }: { page: number }) =>
      Promise.resolve(orphanPageResponse({ page, total: 250, total_pages: 3 }))
    );
    render(OrphansView);
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1));

    await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(2));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /Prev/ })).toBeInTheDocument()
    );
    await fireEvent.click(screen.getByRole("button", { name: /Prev/ }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(3));
    expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 1, pageSize: 100 });
  });

  it("loads the last page when Last is clicked", async () => {
    adapterMocks.getOrphansPage.mockImplementation(({ page }: { page: number }) =>
      Promise.resolve(orphanPageResponse({ page, total: 250, total_pages: 3 }))
    );
    render(OrphansView);
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1));

    await fireEvent.click(screen.getByRole("button", { name: /Last/ }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(2));
    expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 3, pageSize: 100 });
  });
});

// ---------------------------------------------------------------------------
// Refresh button
// ---------------------------------------------------------------------------
describe("OrphansView refresh button", () => {
  it("reloads the current orphan page when Refresh is clicked", async () => {
    render(OrphansView);
    await waitForLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(2));
    expect(adapterMocks.getOrphansPage).toHaveBeenLastCalledWith({ page: 1, pageSize: 100 });
  });
});

// ---------------------------------------------------------------------------
// Loading state
// ---------------------------------------------------------------------------
describe("OrphansView loading state", () => {
  it("disables controls and shows 'Scanning...' while the initial page loads", async () => {
    let resolvePage!: (v: unknown) => void;
    adapterMocks.getOrphansPage.mockReturnValue(
      new Promise((r) => {
        resolvePage = r;
      })
    );
    render(OrphansView);

    await waitFor(() => expect(adapterMocks.getOrphansPage).toHaveBeenCalledTimes(1));
    expect(screen.getByRole("button", { name: "Scanning..." })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Select all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Delete selected (0)" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Delete all (0)" })).toBeDisabled();

    resolvePage(orphanPageResponse());
    await waitFor(() => expect(screen.getByRole("button", { name: "Scan Disk" })).toBeEnabled());
  });
});