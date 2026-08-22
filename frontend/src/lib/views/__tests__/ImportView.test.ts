import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import ImportView from "../ImportView.svelte";
import ImportTestHarness from "./ImportTestHarness.svelte";
import { importSessionStore } from "../../stores/importSessionStore";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

/** Mock the command adapter so no real Tauri `invoke` calls are made. */
const adapterMocks = vi.hoisted(() => ({
  listDesigners: vi.fn(),
  listSources: vi.fn(),
  previewImportFromRoots: vi.fn(),
  precheckImportWire: vi.fn(),
  runPrecheckAction: vi.fn(),
  requestStopBulkImport: vi.fn(),
  browseImportFolder: vi.fn(),
  saveImportLastBrowseFolder: vi.fn(),
  getSettingsViewModel: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

/** Mock the toast store. */
const toastMocks = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMocks);

/** Mock the Tauri event module used for bulk-import progress updates. */
const eventMocks = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => eventMocks);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const scannedFiles = [
  { full_path: "C:/Designs/Rose Studio/rose.pes" },
  { full_path: "C:/Designs/Rose Studio/border.pes" },
  { full_path: "C:/Designs/Winter/snow.vp3" },
];

const settingsResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  model: {
    preview_3d_profile: "balanced",
    google_api_key: "",
    has_google_api_key: false,
    ai_tier2_auto: false,
    ai_tier3_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    import_commit_batch_size: "",
    import_last_browse_folder: "",
    can_configure_data_root: false,
    data_root: "",
    database_path: "",
    log_folder: "",
    app_mode: "development",
    ai_tagging_help_url: "#/help",
    ...o,
  },
});

const listResponse = (items: unknown[] = []) => ({ source: "rust", items });

const defaultDesigners = () => [
  { id: 1, name: "Rose Studio", design_count: 2 },
  { id: 2, name: "Mock Designer", design_count: 5 },
];

const defaultSources = () => [
  { id: 1, name: "Imported", design_count: 3 },
  { id: 2, name: "Purchased", design_count: 4 },
];

const previewResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  preview: {
    discovered_count: 3,
    selected_count: 3,
    folder_count: 2,
    scanned_files: scannedFiles,
    resolved_assignments: [],
    missing_root: false,
    no_supported_files: false,
    invalid_root: false,
    ...o,
  },
  message: "Preview loaded from Rust command.",
});

const precheckResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  precheck: {
    context_token: "tok-123",
    context_token_present: true,
    ready_for_confirm: true,
    is_first_import: false,
    needs_hoop_setup: false,
    root_path_count: 1,
    selected_file_count: 3,
    resolved_assignments: [],
    ...o,
  },
  message: "Precheck loaded from Rust command.",
});

const actionResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  actionResult: {
    action: "import_now",
    context_token_present: true,
    consumed_context: true,
    requires_skip_hoops_confirmation: false,
    next_route: "/designs",
    confirm_result: { persisted_design_count: 3 },
    ...o,
  },
  message: "Import complete.",
  ...o,
});

const browseResponse = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  path: "",
  paths: [],
  message: "Folder selection cancelled.",
  ...o,
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

function asRecord(value: unknown): Record<string, unknown> {
  return (value ?? {}) as Record<string, unknown>;
}

/**
 * Render the wizard behind the test harness so `navigateTo` genuinely advances
 * the route from step 1 -> 2 -> 3 as the user would experience it.
 */
function renderHarness(initialRoute = "#/import") {
  const onNavigate = vi.fn();
  const onImportCompleted = vi.fn();
  const view = render(ImportTestHarness, {
    props: { initialRoute, onImportCompleted, onNavigate },
  });
  return { onNavigate, onImportCompleted, container: view.container, view };
}

/**
 * Render ImportView directly with a fixed route and a no-op navigateTo.  This
 * is useful for asserting static chrome on a given step, or for observing
 * state that would normally be hidden by an immediate navigation.
 */
function renderStatic(route: string) {
  const navigateTo = vi.fn();
  const onImportCompleted = vi.fn();
  const view = render(ImportView, {
    props: { currentRoute: route, navigateTo, onImportCompleted },
  });
  return { navigateTo, onImportCompleted, view, container: view.container };
}

/** Type a path into the step 1 folder input and submit the scan form. */
async function scanFolder(container: HTMLElement, path = "C:\\Designs") {
  const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
  await fireEvent.input(input, { target: { value: path } });
  const form = element(container.querySelector<HTMLFormElement>("#importScanForm"));
  await fireEvent.submit(form);
}

/** Complete the scan step and wait for the review screen. */
async function gotoStep2(container: HTMLElement, path = "C:\\Designs") {
  await scanFolder(container, path);
  await waitFor(() => {
    expect(screen.getByText("Review scanned files")).toBeInTheDocument();
  });
}

/** Complete scan + continue, then wait for the import step 3 screen. */
async function gotoStep3(container: HTMLElement, path = "C:\\Designs") {
  await gotoStep2(container, path);
  await fireEvent.click(screen.getByRole("button", { name: /Continue with \d+ designs/ }));
  await waitFor(() => {
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeInTheDocument();
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  importSessionStore.clear();

  adapterMocks.listDesigners.mockResolvedValue(listResponse(defaultDesigners()));
  adapterMocks.listSources.mockResolvedValue(listResponse(defaultSources()));
  adapterMocks.getSettingsViewModel.mockResolvedValue(settingsResponse());
  adapterMocks.previewImportFromRoots.mockResolvedValue(previewResponse());
  adapterMocks.precheckImportWire.mockResolvedValue(precheckResponse());
  adapterMocks.runPrecheckAction.mockResolvedValue(actionResponse());
  adapterMocks.requestStopBulkImport.mockResolvedValue({
    source: "rust",
    stopRequested: true,
    message: "Stop requested for the running import.",
  });
  adapterMocks.browseImportFolder.mockResolvedValue(browseResponse());
  adapterMocks.saveImportLastBrowseFolder.mockResolvedValue({ source: "rust", persisted: true });
  eventMocks.listen.mockResolvedValue(() => {});
});

// ---------------------------------------------------------------------------
// Page chrome and step rendering
// ---------------------------------------------------------------------------
describe("ImportView page chrome and step rendering", () => {
  it("renders the page heading 'Bulk Import'", () => {
    renderStatic("#/import");
    expect(screen.getByRole("heading", { name: "Bulk Import" })).toBeInTheDocument();
  });

  it("renders the step 1 folder scan chrome", async () => {
    renderStatic("#/import");

    expect(screen.getByLabelText("Source folder path 1")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Enter path to your embroidery designs folder/)).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /Browse/ })).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Scan folder(s)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reset" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add another folder" })).toBeInTheDocument();
    expect(screen.getAllByRole("link", { name: "Import help" })).toHaveLength(1);
  });

  it("shows the step 2 fallback when no preview has been run", () => {
    renderStatic("#/import/step2");
    expect(screen.getByText("Step 2 needs a completed preview first.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Back to Step 1" })).toBeInTheDocument();
  });

  it("shows the step 3 fallback when no precheck has been run", () => {
    renderStatic("#/import/step3");
    expect(screen.getByText("Step 3 needs precheck to be completed first.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Go to previous step" })).toBeInTheDocument();
  });

  it("renders nothing but the heading for an unknown route", () => {
    renderStatic("#/designs");
    expect(screen.getByRole("heading", { name: "Bulk Import" })).toBeInTheDocument();
    expect(screen.queryByLabelText("Source folder path 1")).not.toBeInTheDocument();
    expect(screen.queryByText("Review scanned files")).not.toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Initial data loading
// ---------------------------------------------------------------------------
describe("ImportView initial data loading", () => {
  it("loads designers, sources, and settings on mount", async () => {
    renderHarness("#/import");

    await waitFor(() => {
      expect(adapterMocks.listDesigners).toHaveBeenCalled();
    });
    expect(adapterMocks.listSources).toHaveBeenCalled();
    expect(adapterMocks.getSettingsViewModel).toHaveBeenCalled();
  });

  it("loads reference data only once across the whole wizard", async () => {
    const { container } = renderHarness("#/import");

    await waitFor(() => {
      expect(adapterMocks.listDesigners).toHaveBeenCalled();
    });

    await gotoStep3(container);

    expect(adapterMocks.listDesigners).toHaveBeenCalledTimes(1);
    expect(adapterMocks.listSources).toHaveBeenCalledTimes(1);
    expect(adapterMocks.getSettingsViewModel).toHaveBeenCalledTimes(1);
  });

  it("tolerates reference data failures and continues", async () => {
    const spy = vi.spyOn(console, "info").mockImplementation(() => {});
    adapterMocks.listDesigners.mockRejectedValue(new Error("db down"));
    adapterMocks.listSources.mockRejectedValue(new Error("db down"));

    renderHarness("#/import");

    await waitFor(() => {
      expect(adapterMocks.listDesigners).toHaveBeenCalled();
    });
    // The global selects still render with only the placeholder option.
    const { container } = renderHarness("#/import/step2");
    await waitFor(() => expect(spy).toHaveBeenCalled());
    expect(container).toBeTruthy();
    spy.mockRestore();
  });

  it("logs an error when settings fail to load but still renders", async () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    adapterMocks.getSettingsViewModel.mockRejectedValue(new Error("settings down"));

    renderHarness("#/import");

    await waitFor(() => expect(spy).toHaveBeenCalled());
    expect(screen.getByRole("heading", { name: "Bulk Import" })).toBeInTheDocument();
    spy.mockRestore();
  });
});

// ---------------------------------------------------------------------------
// Step 1: folder path management
// ---------------------------------------------------------------------------
describe("ImportView step 1 folder path management", () => {
  it("binds the typed folder path to the input", () => {
    const { container } = renderStatic("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    expect(input.value).toBe("");
  });

  it("adds another folder row via the Add another folder button", async () => {
    const { container } = renderStatic("#/import");

    const addButton = screen.getByRole("button", { name: "Add another folder" });
    expect(addButton).toBeDisabled(); // no typed path yet

    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:\\Designs" } });
    expect(addButton).toBeEnabled();

    await fireEvent.click(addButton);
    expect(screen.getByLabelText("Source folder path 2")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Browse…" })).toHaveLength(2);
  });

  it("removes an extra folder row", async () => {
    const { container } = renderStatic("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:\\Designs" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add another folder" }));
    expect(screen.getByLabelText("Source folder path 2")).toBeInTheDocument();

    // The second row has its own Remove button (the last one on screen).
    const removeButtons = screen.getAllByRole("button", { name: "Remove" });
    await fireEvent.click(removeButtons[removeButtons.length - 1]);
    expect(screen.queryByLabelText("Source folder path 2")).not.toBeInTheDocument();
  });

  it("shifts the next folder up into the primary row when its Remove button is pressed", async () => {
    const { container } = renderStatic("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:\\Designs" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add another folder" }));

    // Extra rows are readonly; fill the second row with a real value via browse.
    adapterMocks.browseImportFolder.mockResolvedValue(browseResponse({ paths: ["D:/Extra"] }));
    await fireEvent.click(screen.getAllByRole("button", { name: "Browse…" })[1]);
    await waitFor(() =>
      expect(
        container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 2"]')?.value
      ).toBe("D:/Extra")
    );

    const removeButtons = screen.getAllByRole("button", { name: "Remove" });
    await fireEvent.click(removeButtons[0]);
    // The primary row now holds the next folder and the extra row is gone.
    expect(input.value).toBe("D:/Extra");
    expect(screen.queryByLabelText("Source folder path 2")).not.toBeInTheDocument();
  });

  it("shifts the primary row up when several folders are selected via multi-browse", async () => {
    const { container } = renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ paths: ["D:/A", "D:/B", "D:/C"] })
    );

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() =>
      expect(screen.getByLabelText("Source folder path 3")).toBeInTheDocument()
    );

    // Remove the primary (row 1); row 2 shifts up into the primary slot.
    const removeButtons = screen.getAllByRole("button", { name: "Remove" });
    await fireEvent.click(removeButtons[0]);
    expect(container.querySelector<HTMLInputElement>("#import-root-path")?.value).toBe("D:/B");
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 2"]')?.value).toBe("D:/C");
    expect(screen.queryByLabelText("Source folder path 3")).not.toBeInTheDocument();
  });

  it("clears the primary row when it is the only folder and Remove is pressed", async () => {
    const { container } = renderStatic("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:\\Designs" } });
    expect(input.value).toBe("C:\\Designs");

    const removeButtons = screen.getAllByRole("button", { name: "Remove" });
    await fireEvent.click(removeButtons[0]);
    // No extra folders to shift up, so the single entry point just clears.
    expect(input.value).toBe("");
    expect(screen.queryByLabelText("Source folder path 2")).not.toBeInTheDocument();
  });

  it("disabled Scan and Reset until at least one folder is selected", async () => {
    const { container } = renderStatic("#/import");

    const scanButton = screen.getByRole("button", { name: "Scan folder(s)" });
    const resetButton = screen.getByRole("button", { name: "Reset" });

    // No folder selected yet.
    expect(scanButton).toBeDisabled();
    expect(resetButton).toBeDisabled();

    // Type a folder into the primary row.
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:\\Designs" } });
    await tick();
    expect(scanButton).toBeEnabled();
    expect(resetButton).toBeEnabled();

    // Clear the folder again.
    await fireEvent.input(input, { target: { value: "" } });
    await tick();
    expect(scanButton).toBeDisabled();
    expect(resetButton).toBeDisabled();
  });

  it("normalises backslashes, trailing slashes, and drive-letter roots on submit", async () => {
    const { container, navigateTo } = renderStatic("#/import");

    await scanFolder(container, "C:\\Designs\\");
    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenCalled());
    expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["C:/Designs"]);
    expect(navigateTo).toHaveBeenCalledWith("#/import/step2");
  });

  it("normalises doubled separators and UNC paths", async () => {
    const { container } = renderStatic("#/import");

    await scanFolder(container, "C:/Designs//sub");
    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenCalled());
    expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["C:/Designs/sub"]);

    await scanFolder(container, "\\\\server\\share\\designs\\");
    await waitFor(() =>
      expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["//server/share/designs"])
    );

    await scanFolder(container, "C:");
    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["C:/"]));
  });

  it("deduplicates root paths case-insensitively", async () => {
    const { container } = renderStatic("#/import");

    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:/Designs" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add another folder" }));

    // The extra row is set to the same path in different casing via browse.
    adapterMocks.browseImportFolder.mockResolvedValue(browseResponse({ paths: ["c:/designs"] }));
    await fireEvent.click(screen.getAllByRole("button", { name: "Browse…" })[1]);

    await waitFor(() =>
      expect(adapterMocks.saveImportLastBrowseFolder).toHaveBeenCalledWith("c:/designs")
    );

    const form = element(container.querySelector<HTMLFormElement>("#importScanForm"));
    await fireEvent.submit(form);
    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenCalled());
    // The case-insensitive duplicate is folded into a single root.
    expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["C:/Designs"]);
  });
});

// ---------------------------------------------------------------------------
// Step 1: browse flows
// ---------------------------------------------------------------------------
describe("ImportView browse flows", () => {
  it("updates the folder input when a single path is selected", async () => {
    const { container } = renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ path: "E:\\New Designs", paths: ["E:\\New Designs"] })
    );

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() =>
      expect(adapterMocks.browseImportFolder).toHaveBeenCalledWith("")
    );

    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await waitFor(() => expect(input.value).toBe("E:/New Designs"));
    expect(adapterMocks.saveImportLastBrowseFolder).toHaveBeenCalledWith("E:/New Designs");
  });

  it("seeds the picker from the persisted last browse folder when the row is empty", async () => {
    // Simulate a previous session where multiple folders were picked: the
    // persisted value is the last folder picked (e.g. e:/designs/patterns).
    adapterMocks.getSettingsViewModel.mockResolvedValue(
      settingsResponse({ import_last_browse_folder: "e:/designs/patterns" })
    );
    renderStatic("#/import");
    // Wait for settings to load so the persisted folder seeds the picker.
    await waitFor(() => expect(adapterMocks.getSettingsViewModel).toHaveBeenCalled());

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() => expect(adapterMocks.browseImportFolder).toHaveBeenCalled());
    // The picker should open in the parent of the persisted folder (e:/designs),
    // not the last folder picked (e:/designs/patterns).
    expect(adapterMocks.browseImportFolder).toHaveBeenLastCalledWith("e:/designs");
  });

  it("uses the persisted parent for a single previously browsed folder", async () => {
    adapterMocks.getSettingsViewModel.mockResolvedValue(
      settingsResponse({ import_last_browse_folder: "e:/designs/faces" })
    );
    renderStatic("#/import");
    await waitFor(() => expect(adapterMocks.getSettingsViewModel).toHaveBeenCalled());

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() => expect(adapterMocks.browseImportFolder).toHaveBeenCalled());
    expect(adapterMocks.browseImportFolder).toHaveBeenLastCalledWith("e:/designs");
  });

  it("prefers the current row's parent over the persisted folder when the row is populated", async () => {
    const { container } = renderStatic("#/import");
    adapterMocks.getSettingsViewModel.mockResolvedValue(
      settingsResponse({ import_last_browse_folder: "e:/designs/patterns" })
    );
    await waitFor(() => expect(adapterMocks.getSettingsViewModel).toHaveBeenCalled());

    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:/MyDesigns/Sub" } });

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() => expect(adapterMocks.browseImportFolder).toHaveBeenCalled());
    // The populated row wins: it opens in the parent of the typed value.
    expect(adapterMocks.browseImportFolder).toHaveBeenLastCalledWith("C:/MyDesigns");
  });

  it("fills additional rows when multi-selection returns extra paths", async () => {
    const { container } = renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ paths: ["D:/A", "D:/B", "D:/C"] })
    );

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    // Row 0 becomes D:/A, and D:/B / D:/C are appended as new rows.
    await waitFor(() =>
      expect(screen.getByLabelText("Source folder path 2")).toBeInTheDocument()
    );
    expect(screen.getByLabelText("Source folder path 3")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Browse…" })).toHaveLength(3);

    // Every row must retain its selected folder in order — the primary row must
    // not be clobbered by the last appended folder.
    expect(container.querySelector<HTMLInputElement>("#import-root-path")?.value).toBe("D:/A");
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 2"]')?.value).toBe("D:/B");
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 3"]')?.value).toBe("D:/C");
    expect(adapterMocks.saveImportLastBrowseFolder).toHaveBeenCalledWith("D:/A");
  });

  it("keeps all four folders in order with no duplication when four are selected", async () => {
    const { container, navigateTo } = renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ paths: ["D:/Folder1", "D:/Folder2", "D:/Folder3", "D:/Folder4"] })
    );

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    // All four selected folders are represented: primary + 3 appended rows.
    await waitFor(() =>
      expect(screen.getByLabelText("Source folder path 4")).toBeInTheDocument()
    );
    expect(screen.getAllByRole("button", { name: "Browse…" })).toHaveLength(4);

    const inputs = [
      container.querySelector<HTMLInputElement>("#import-root-path"),
      container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 2"]'),
      container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 3"]'),
      container.querySelector<HTMLInputElement>('input[aria-label="Source folder path 4"]'),
    ];
    const values = inputs.map((input) => input?.value ?? "");
    expect(values).toEqual(["D:/Folder1", "D:/Folder2", "D:/Folder3", "D:/Folder4"]);
    // No duplicate and no missing folder.
    expect(new Set(values).size).toBe(4);

    const form = element(container.querySelector<HTMLFormElement>("#importScanForm"));
    await fireEvent.submit(form);
    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenCalled());
    expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith([
      "D:/Folder1",
      "D:/Folder2",
      "D:/Folder3",
      "D:/Folder4",
    ]);
    expect(navigateTo).toHaveBeenCalledWith("#/import/step2");
  });

  it("updates an extra row when its browse button is used", async () => {
    const { container } = renderStatic("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:/Designs" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add another folder" }));

    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ paths: ["D:/Extra"] })
    );
    await fireEvent.click(screen.getAllByRole("button", { name: "Browse…" })[1]);
    await waitFor(() => expect(screen.getByLabelText("Source folder path 2")).toBeInTheDocument());

    const form = element(container.querySelector<HTMLFormElement>("#importScanForm"));
    await fireEvent.submit(form);
    await waitFor(() =>
      expect(adapterMocks.previewImportFromRoots).toHaveBeenLastCalledWith(["C:/Designs", "D:/Extra"])
    );
  });

  it("does nothing when the folder picker is cancelled", async () => {
    renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(browseResponse());

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() => expect(adapterMocks.browseImportFolder).toHaveBeenCalled());
    expect(adapterMocks.saveImportLastBrowseFolder).not.toHaveBeenCalled();
  });

  it("shows an error toast when the folder picker throws", async () => {
    renderStatic("#/import");
    adapterMocks.browseImportFolder.mockRejectedValue(new Error("picker down"));

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Folder browse failed: Error: picker down",
        "error"
      )
    );
  });

  it("swallows persistence failures from saveImportLastBrowseFolder", async () => {
    const spy = vi.spyOn(console, "info").mockImplementation(() => {});
    const { container } = renderStatic("#/import");
    adapterMocks.browseImportFolder.mockResolvedValue(
      browseResponse({ paths: ["E:/New Designs"] })
    );
    adapterMocks.saveImportLastBrowseFolder.mockRejectedValue(new Error("persist down"));

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await waitFor(() => expect(input.value).toBe("E:/New Designs"));
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });

  it("shows Browsing... while the folder picker is pending", async () => {
    let resolveBrowse!: (v: unknown) => void;
    adapterMocks.browseImportFolder.mockReturnValue(
      new Promise((r) => {
        resolveBrowse = r;
      })
    );

    renderStatic("#/import");
    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Browsing…" })).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: "Scan folder(s)" })).toBeDisabled();

    resolveBrowse(browseResponse({ paths: ["E:/New Designs"] }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Browse…" })).toBeInTheDocument()
    );
  });
});

// ---------------------------------------------------------------------------
// Step 1: preview submission
// ---------------------------------------------------------------------------
describe("ImportView step 1 preview submission", () => {
  it("submits the normalised roots to previewImportFromRoots", async () => {
    const { container } = renderHarness("#/import");
    await scanFolder(container, "C:/Designs");

    await waitFor(() => expect(adapterMocks.previewImportFromRoots).toHaveBeenCalledWith(["C:/Designs"]));
    await waitFor(() => expect(screen.getByText("Review scanned files")).toBeInTheDocument());
  });

  it("shows Running... on the submit button while preview is pending", async () => {
    let resolvePreview!: (v: unknown) => void;
    adapterMocks.previewImportFromRoots.mockReturnValue(
      new Promise((r) => {
        resolvePreview = r;
      })
    );

    const { container } = renderHarness("#/import");
    const input = element(container.querySelector<HTMLInputElement>("#import-root-path"));
    await fireEvent.input(input, { target: { value: "C:/Designs" } });
    await fireEvent.submit(element(container.querySelector<HTMLFormElement>("#importScanForm")));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Running…" })).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: "Reset" })).toBeDisabled();

    resolvePreview(previewResponse());
    await waitFor(() => expect(screen.getByText("Review scanned files")).toBeInTheDocument());
  });

  it("shows an error toast and stays on step 1 when preview fails", async () => {
    adapterMocks.previewImportFromRoots.mockRejectedValue(new Error("scan failed"));
    const { container } = renderHarness("#/import");

    await scanFolder(container, "C:/Designs");

    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Import preview failed: Error: scan failed",
        "error"
      )
    );
    // Still on step 1.
    expect(screen.getByLabelText("Source folder path 1")).toBeInTheDocument();
  });

  it("navigates to step 2 and shows folder/file summary after a successful scan", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container, "C:/Designs");

    expect(
      screen.getByText(/2 folder\(s\) scanned - 3 file\(s\) found\./)
    ).toBeInTheDocument();
  });

  it("shows the invalid_root explanation in the step 2 empty state", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        scanned_files: [],
        invalid_root: true,
        missing_root: false,
        no_supported_files: false,
      })
    );
    const { container } = renderHarness("#/import");
    await scanFolder(container, "C:/Designs");

    await waitFor(() =>
      expect(screen.getByText("No supported files discovered in this preview.")).toBeInTheDocument()
    );
    expect(
      screen.getByText("Enter at least one folder path to preview import.")
    ).toBeInTheDocument();
  });

  it("shows the missing_root explanation in the step 2 empty state", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        scanned_files: [],
        invalid_root: false,
        missing_root: true,
        no_supported_files: false,
      })
    );
    const { container } = renderHarness("#/import");
    await scanFolder(container, "C:/Designs");

    await waitFor(() =>
      expect(
        screen.getByText(
          "The selected folder(s) could not be found on disk. Check that the path is correct and the drive is available."
        )
      ).toBeInTheDocument()
    );
  });

  it("shows the no_supported_files explanation in the step 2 empty state", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        scanned_files: [],
        invalid_root: false,
        missing_root: false,
        no_supported_files: true,
      })
    );
    const { container } = renderHarness("#/import");
    await scanFolder(container, "C:/Designs");

    await waitFor(() =>
      expect(
        screen.getByText(
          "No supported embroidery files (JEF, PES, HUS, DST, EXP, VP3) were found in the selected folder(s)."
        )
      ).toBeInTheDocument()
    );
  });

  it("shows the empty state without a message when no diagnostic flag is set", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        discovered_count: 0,
        selected_count: 0,
        folder_count: 0,
        scanned_files: [],
        invalid_root: false,
        missing_root: false,
        no_supported_files: false,
      })
    );
    const { container } = renderHarness("#/import");
    await scanFolder(container, "C:/Designs");

    await waitFor(() =>
      expect(screen.getByText("No supported files discovered in this preview.")).toBeInTheDocument()
    );
    expect(
      screen.queryByText("Enter at least one folder path to preview import.")
    ).not.toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Step 2: file review and selection
// ---------------------------------------------------------------------------
describe("ImportView step 2 file review and selection", () => {
  it("groups scanned files by folder with folder labels and filenames", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const folderLabels = Array.from(
      container.querySelectorAll<HTMLElement>(".import-step2-folder-label")
    ).map((el) => (el.textContent || "").trim());
    expect(folderLabels).toEqual(expect.arrayContaining(["Rose Studio", "Winter"]));

    expect(screen.getByText("rose.pes")).toBeInTheDocument();
    expect(screen.getByText("border.pes")).toBeInTheDocument();
    expect(screen.getByText("snow.vp3")).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "rose.pes" })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "snow.vp3" })).toBeChecked();
  });

  it("selects all files by default and shows the correct continue label", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);
    expect(screen.getByRole("button", { name: "Continue with 3 designs" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Select all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeEnabled();
  });

  it("updates the selected count when a checkbox is toggled", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("checkbox", { name: "rose.pes" }));
    expect(screen.getByRole("button", { name: "Continue with 2 designs" })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "rose.pes" })).not.toBeChecked();
  });

  it("re-selects a file after it was unchecked", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const roseCheckbox = screen.getByRole("checkbox", { name: "rose.pes" });
    await fireEvent.click(roseCheckbox);
    await fireEvent.click(roseCheckbox);
    expect(screen.getByRole("button", { name: "Continue with 3 designs" })).toBeInTheDocument();
  });

  it("deselects and re-selects all files with the bulk buttons", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Deselect all" }));
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Select all" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Deselect all" })).toBeDisabled();

    await fireEvent.click(screen.getByRole("button", { name: "Select all" }));
    expect(screen.getByRole("button", { name: "Continue with 3 designs" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Select all" })).toBeDisabled();
  });

  it("continues with only the selected files in the confirm wire", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("checkbox", { name: "border.pes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Continue with 2 designs" }));

    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalled());

    const wire = asRecord(adapterMocks.precheckImportWire.mock.calls.at(-1)?.[0]);
    const innerWire = asRecord(wire.wire);
    expect(innerWire.create_on_import).toBe(true);
    expect(innerWire.selected_files).toEqual([
      "C:/Designs/Rose Studio/rose.pes",
      "C:/Designs/Winter/snow.vp3",
    ]);
  });

  it("handles paths with a missing folder gracefully", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        scanned_files: [{ full_path: "orphan.pes" }, { full_path: "" }, {}],
        discovered_count: 3,
        folder_count: 1,
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    // "Unknown folder" appears as both the folder label and the folder path.
    expect(screen.getAllByText("Unknown folder").length).toBeGreaterThan(0);
    expect(screen.getByText("orphan.pes")).toBeInTheDocument();
    // Files without a full path are skipped entirely during grouping.
    expect(screen.queryByText("Unknown file")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue with 1 design" })).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Step 2: reference data and per-folder overrides
// ---------------------------------------------------------------------------
describe("ImportView step 2 reference data and overrides", () => {
  it("renders global designer and source selects with loaded options", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const globalShell = element(container.querySelector<HTMLElement>(".import-step2-global-shell"));
    const designerSelect = element(globalShell.querySelector<HTMLSelectElement>("select"));
    const sourceSelect = element(
      globalShell.querySelectorAll<HTMLSelectElement>("select")[1] ?? null
    );

    await waitFor(() => expect(designerSelect.options.length).toBeGreaterThan(1));
    expect(Array.from(designerSelect.options).map((o) => o.text)).toEqual(
      expect.arrayContaining(["Rose Studio", "Mock Designer"])
    );
    expect(Array.from(sourceSelect.options).map((o) => o.text)).toEqual(
      expect.arrayContaining(["Imported", "Purchased"])
    );
  });

  it("shows the inferred designer label from path-based suggestion", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    // Folder "C:/Designs/Rose Studio" matches the "Rose Studio" designer.
    await waitFor(() => {
      const option = screen.getByRole("option", { name: "Keep inferred (Rose Studio)" });
      expect(option).toBeInTheDocument();
    });
    expect(container).toBeTruthy();
  });

  it("shows the inferred designer label from resolved assignments", async () => {
    adapterMocks.previewImportFromRoots.mockResolvedValue(
      previewResponse({
        resolved_assignments: [
          {
            folder_path: "C:/Designs/Winter",
            inferred_designer_id: 2,
            inferred_source_id: 1,
          },
        ],
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    // The Winter folder is sorted last; the resolved assignment targets it.
    const overrideShells = container.querySelectorAll<HTMLElement>(".import-step2-folder-overrides");
    const winterShell = element(overrideShells[overrideShells.length - 1]);
    const designerSelect = element(winterShell.querySelector<HTMLSelectElement>("select"));
    const sourceSelect = element(
      winterShell.querySelectorAll<HTMLSelectElement>("select")[1] ?? null
    );

    await waitFor(() =>
      expect(designerSelect.options[0].text).toBe("Keep inferred (Mock Designer)")
    );
    expect(sourceSelect.options[0].text).toBe("Keep inferred (Imported)");
  });

  it("shows plain 'Keep inferred' when no match exists", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const overrideShells = container.querySelectorAll<HTMLElement>(".import-step2-folder-overrides");
    const winterShell = overrideShells[overrideShells.length - 1];
    const designerSelect = element(winterShell.querySelector<HTMLSelectElement>("select"));

    await waitFor(() =>
      expect(designerSelect.options[0].text).toBe("Keep inferred")
    );
  });

  it("sets a per-folder designer override and includes it in the confirm wire", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const overrideShell = element(container.querySelector<HTMLElement>(".import-step2-folder-overrides"));
    const designerSelect = element(overrideShell.querySelector<HTMLSelectElement>("select"));
    await fireEvent.change(designerSelect, { target: { value: "2" } });
    expect(designerSelect.value).toBe("2");

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalled());

    const wire = asRecord(adapterMocks.precheckImportWire.mock.calls.at(-1)?.[0]);
    const innerWire = asRecord(wire.wire);
    const perFolder = innerWire.per_folder_assignments as Array<Record<string, unknown>>;
    const roseFolder = perFolder.find((f) => f.folder_path === "C:/Designs/Rose Studio");
    expect(roseFolder?.designer_id).toBe(2);
  });

  it("sets a global designer override and includes its id in the confirm wire", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const globalShell = element(container.querySelector<HTMLElement>(".import-step2-global-shell"));
    const designerSelect = element(globalShell.querySelector<HTMLSelectElement>("select"));
    await fireEvent.change(designerSelect, { target: { value: "2" } });

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalled());

    const wire = asRecord(adapterMocks.precheckImportWire.mock.calls.at(-1)?.[0]);
    const innerWire = asRecord(wire.wire);
    expect(innerWire.global_designer_id).toBe(2);
    expect(innerWire.global_source_id).toBe(null);
  });

  it("includes inferred designer/source ids from matching folder names in the wire", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalled());

    const wire = asRecord(adapterMocks.precheckImportWire.mock.calls.at(-1)?.[0]);
    const innerWire = asRecord(wire.wire);
    const perFolder = innerWire.per_folder_assignments as Array<Record<string, unknown>>;
    const roseFolder = perFolder.find((f) => f.folder_path === "C:/Designs/Rose Studio");
    expect(roseFolder?.inferred_designer_id).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// Step 2: precheck flow
// ---------------------------------------------------------------------------
describe("ImportView precheck flow", () => {
  it("navigates to step 3 with a context token after precheck", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    expect(adapterMocks.precheckImportWire).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Before You Import")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeEnabled();
  });

  it("blocks precheck when no files are selected", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Deselect all" }));
    await fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(adapterMocks.precheckImportWire).not.toHaveBeenCalled();
    expect(toastMocks.addToast).toHaveBeenCalledWith(
      "Select at least one file before continuing.",
      "error"
    );
  });

  it("shows the confirm wire with root paths on precheck", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep2(container, "C:\\Sub\\Designs");
    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));

    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalled());

    const call = asRecord(adapterMocks.precheckImportWire.mock.calls.at(-1)?.[0]);
    const wire = asRecord(call.wire);
    expect(call.context_token).toBe(null);
    expect(call.canonical_confirm).toBe(false);
    expect(wire.root_paths).toEqual(["C:/Sub/Designs"]);
    expect(wire.create_on_import).toBe(true);
  });

  it("shows an error toast and stays on step 2 when precheck throws", async () => {
    adapterMocks.precheckImportWire.mockRejectedValue(new Error("precheck down"));
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Import precheck failed: Error: precheck down",
        "error"
      )
    );
    expect(screen.getByText("Review scanned files")).toBeInTheDocument();
  });

  it("shows Running... on the continue button while precheck is pending", async () => {
    let resolvePrecheck!: (v: unknown) => void;
    adapterMocks.precheckImportWire.mockReturnValue(
      new Promise((r) => {
        resolvePrecheck = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Running…" })).toBeInTheDocument()
    );

    resolvePrecheck(precheckResponse());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Import Designs" })).toBeInTheDocument()
    );
  });

  it("disables step 3 actions when the precheck returns an empty context token", async () => {
    adapterMocks.precheckImportWire.mockResolvedValue(precheckResponse({ context_token: "" }));
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() =>
      expect(screen.getByText("Before You Import")).toBeInTheDocument()
    );

    expect(screen.getByRole("button", { name: "Import Designs" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// Step 3: actions and completion
// ---------------------------------------------------------------------------
describe("ImportView step 3 actions", () => {
  it("renders the precheck summary and the AI notice for a non-configured key", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    expect(screen.getByText("Before You Import")).toBeInTheDocument();
    expect(screen.getByText("Google AI tagging is not configured.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
  });

  it("shows the amber banner when a Google API key is configured", async () => {
    adapterMocks.getSettingsViewModel.mockResolvedValue(
      settingsResponse({ google_api_key: "abc123", has_google_api_key: true })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    expect(screen.getByText("Google AI tagging is enabled for this installation.")).toBeInTheDocument();
  });

  it("completes an import and calls onImportCompleted with the persisted count", async () => {
    const { container, onImportCompleted, onNavigate } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(onImportCompleted).toHaveBeenCalledWith(3));

    // resetImportWizard first lands on step 1, then the designs route is applied.
    expect(onNavigate).toHaveBeenCalledWith("#/import/step1");
    expect(onNavigate).toHaveBeenLastCalledWith("#/designs");
  });

  it("does not call onImportCompleted when nothing was persisted", async () => {
    adapterMocks.runPrecheckAction.mockResolvedValue(
      actionResponse({
        confirm_result: { persisted_design_count: 0 },
        next_route: "/designs",
      })
    );
    const { container, onImportCompleted } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(adapterMocks.runPrecheckAction).toHaveBeenCalled());
    expect(onImportCompleted).not.toHaveBeenCalled();
  });

  it("shows an error toast when an import_now action has an unmapped route", async () => {
    adapterMocks.runPrecheckAction.mockResolvedValue(
      actionResponse({
        next_route: "/unknown",
        confirm_result: null,
        message: "Import failed: no route.",
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Import failed: no route.",
        "error"
      )
    );
  });

  it("shows an error toast when the precheck action throws", async () => {
    adapterMocks.runPrecheckAction.mockRejectedValue(new Error("action down"));
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() =>
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Import action failed: Error: action down",
        "error"
      )
    );
  });

  it("cancels and resets the wizard back to step 1", async () => {
    adapterMocks.runPrecheckAction.mockResolvedValue(
      actionResponse({
        action: "cancel",
        next_route: "/import/",
        consumed_context: true,
        confirm_result: null,
      })
    );
    const { container, onNavigate } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(onNavigate).toHaveBeenCalledWith("#/import/step1"));
    expect(screen.getByLabelText("Source folder path 1")).toBeInTheDocument();
  });

  it("shows the skip-hoops confirmation and re-runs the action with confirmation", async () => {
    adapterMocks.runPrecheckAction.mockResolvedValue(
      actionResponse({
        action: "import_now",
        requires_skip_hoops_confirmation: true,
        consumed_context: false,
        next_route: "/import/step3",
        confirm_result: null,
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() =>
      expect(
        screen.getByText("Hoops are not configured for a first import. Confirm to continue anyway.")
      ).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import without hoop setup" }));
    await waitFor(() =>
      expect(adapterMocks.runPrecheckAction).toHaveBeenLastCalledWith({
        contextToken: "tok-123",
        action: "import_now",
        confirmSkipHoops: true,
      })
    );
  });

});

// ---------------------------------------------------------------------------
// Wizard session survival across route changes
// ---------------------------------------------------------------------------
describe("ImportView wizard session survival", () => {
  it("restores step 3 with all selections after ImportView is unmounted and remounted", async () => {
    // Reach step 3 exactly as the user would.
    const { view, container } = renderHarness("#/import");
    await gotoStep3(container);
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeEnabled();

    // Simulate clicking Admin Settings / AI Tagging Guide / About / Licence:
    // MainView unmounts ImportView, destroying its local $state.
    view.unmount();

    // Return to the wizard on the step 3 hash (e.g. browser Back).  The session
    // store must restore the precheck, selected files, and context token so the
    // "Before You Import" panel renders ready to import again.
    renderHarness("#/import/step3");
    await waitFor(() =>
      expect(screen.getByText("Before You Import")).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeEnabled();
  });

  it("restores step 2 review with selections after unmount and remount", async () => {
    const { view, container } = renderHarness("#/import");
    await gotoStep2(container);
    expect(screen.getByRole("button", { name: "Continue with 3 designs" })).toBeInTheDocument();

    view.unmount();

    // Returning to #/import/step2 after visiting a top-level page restores the
    // scanned preview and the user's file selection.
    renderHarness("#/import/step2");
    await waitFor(() =>
      expect(screen.getByText("Review scanned files")).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: "Continue with 3 designs" })).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Stale backend context token recovery
// ---------------------------------------------------------------------------
describe("ImportView stale context token recovery", () => {
  it("re-runs the precheck and retries the import when the backend token expires", async () => {
    const { container, onImportCompleted, onNavigate } = renderHarness("#/import");
    await gotoStep3(container);

    // First runPrecheckAction call simulates the backend rejecting the token
    // (15-minute TTL elapsed while the user was on a top-level page).  The
    // adapter degrades to a mock result with no next_route and the expired
    // token message.
    adapterMocks.runPrecheckAction.mockResolvedValueOnce({
      source: "mock",
      actionResult: {
        action: "import_now",
        context_token_present: false,
        consumed_context: false,
        requires_skip_hoops_confirmation: false,
        next_route: null,
        confirm_result: null,
      },
      message: "Import action failed: Unknown or expired bulk import context token: tok-123",
    });

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));

    // Recovery re-runs the precheck to mint a fresh token, then retries.
    await waitFor(() => expect(adapterMocks.precheckImportWire).toHaveBeenCalledTimes(2));
    expect(toastMocks.addToast).toHaveBeenCalledWith(
      "Import context expired. Re-checking your selections before retrying...",
      "info"
    );

    // The retried import succeeds and completes normally.
    await waitFor(() => expect(onImportCompleted).toHaveBeenCalledWith(3));
    expect(onNavigate).toHaveBeenLastCalledWith("#/designs");
  });
});

// ---------------------------------------------------------------------------
// Import progress events
// ---------------------------------------------------------------------------
describe("ImportView bulk import progress events", () => {
  it("starts the progress listener when import_now is executed", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    expect(eventMocks.listen).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalledTimes(1));
    expect(eventMocks.listen).toHaveBeenCalledWith(
      "bulk-import-progress",
      expect.any(Function)
    );

    resolveAction(actionResponse());
  });

  it("does not start the listener for a cancel action", async () => {
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(adapterMocks.runPrecheckAction).toHaveBeenCalled());

    expect(eventMocks.listen).not.toHaveBeenCalled();
  });

  it("renders the started stage status on the import button", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;
    handler({ payload: { stage: "started", total_count: 3 } });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... Starting import for 3 files..." })
      ).toBeInTheDocument()
    );
    resolveAction(actionResponse());
  });

  it("renders the processing_file stage with filename and counts", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;
    handler({
      payload: {
        stage: "processing_file",
        processed_count: 1,
        total_count: 3,
        current_file: "C:/Designs/Rose Studio/rose.pes",
      },
    });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... Processing 2/3: rose.pes" })
      ).toBeInTheDocument()
    );
    resolveAction(actionResponse());
  });

  it("renders generating_images and batch_committed stages", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;

    handler({ payload: { stage: "generating_images", processed_count: 2, total_count: 3, persisted_count: 1 } });
    await waitFor(() =>
      expect(
        screen.getByRole("button", {
          name: "Running Import... 2/3 processed (1 imported) - generating preview images...",
        })
      ).toBeInTheDocument()
    );

    handler({ payload: { stage: "batch_committed", processed_count: 3, total_count: 3, committed_count: 3 } });
    await waitFor(() =>
      expect(
        screen.getByRole("button", {
          name: "Running Import... 3/3 processed (3 imported) - saving batch...",
        })
      ).toBeInTheDocument()
    );
    resolveAction(actionResponse());
  });

  it("renders the completed and stopped stages", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;

    handler({ payload: { stage: "completed", processed_count: 3, total_count: 3, committed_count: 3 } });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... Completed 3/3 processed (3 imported)" })
      ).toBeInTheDocument()
    );

    handler({ payload: { stage: "stopped", processed_count: 1, total_count: 3, committed_count: 1 } });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... Stopped after 1/3 processed (1 imported)" })
      ).toBeInTheDocument()
    );
    resolveAction(actionResponse());
  });

  it("falls back to the generic progress message for unknown stages", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;
    handler({ payload: { stage: "weird_stage", processed_count: 1, total_count: 4, committed_count: 1 } });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... 1/4 processed (1 imported)" })
      ).toBeInTheDocument()
    );
    resolveAction(actionResponse());
  });

  it("ignores progress events from a different context token", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() => expect(eventMocks.listen).toHaveBeenCalled());

    const handler = eventMocks.listen.mock.calls[0][1] as (event: {
      payload: Record<string, unknown>;
    }) => void;
    handler({ payload: { stage: "started", total_count: 3, context_token: "tok-123" } });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Running Import... Starting import for 3 files..." })
      ).toBeInTheDocument()
    );

    // A completed event from a different token must not overwrite the status.
    handler({ payload: { stage: "completed", total_count: 0, committed_count: 99, context_token: "other" } });
    await Promise.resolve();
    expect(
      screen.getByRole("button", { name: "Running Import... Starting import for 3 files..." })
    ).toBeInTheDocument();

    resolveAction(actionResponse());
  });

  it("requests a stop for the running import", async () => {
    let resolveAction!: (v: unknown) => void;
    adapterMocks.runPrecheckAction.mockReturnValue(
      new Promise((r) => {
        resolveAction = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep3(container);

    await fireEvent.click(screen.getByRole("button", { name: "Import Designs" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    await waitFor(() => expect(adapterMocks.requestStopBulkImport).toHaveBeenCalledTimes(1));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Stopping..." })).toBeInTheDocument()
    );

    resolveAction(
      actionResponse({ next_route: "/import/step3", consumed_context: false, confirm_result: null })
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Import Designs" })).toBeInTheDocument()
    );
  });
});

// ---------------------------------------------------------------------------
// Loading and disabled states across steps
// ---------------------------------------------------------------------------
describe("ImportView loading and disabled states", () => {
  it("disables file checkboxes while a precheck is running", async () => {
    let resolvePrecheck!: (v: unknown) => void;
    adapterMocks.precheckImportWire.mockReturnValue(
      new Promise((r) => {
        resolvePrecheck = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    await fireEvent.click(screen.getByRole("button", { name: "Continue with 3 designs" }));
    await waitFor(() =>
      expect(screen.getByRole("checkbox", { name: "rose.pes" })).toBeDisabled()
    );
    expect(screen.getByRole("button", { name: "Running…" })).toBeInTheDocument();

    resolvePrecheck(precheckResponse());
    await waitFor(() =>
      expect(screen.getByText("Before You Import")).toBeInTheDocument()
    );
    // After the precheck resolves, the wizard navigates to step 3 where the
    // action buttons become active with the context token available.
    expect(screen.getByRole("button", { name: "Import Designs" })).toBeEnabled();
  });

  it("keeps the global and per-folder selects disabled while reference data loads", async () => {
    let resolveDesigners!: (v: unknown) => void;
    let resolveSources!: (v: unknown) => void;
    adapterMocks.listDesigners.mockReturnValue(
      new Promise((r) => {
        resolveDesigners = r;
      })
    );
    adapterMocks.listSources.mockReturnValue(
      new Promise((r) => {
        resolveSources = r;
      })
    );
    const { container } = renderHarness("#/import");
    await gotoStep2(container);

    const globalShell = element(container.querySelector<HTMLElement>(".import-step2-global-shell"));
    const designerSelect = element(globalShell.querySelector<HTMLSelectElement>("select"));

    await waitFor(() => expect(designerSelect).toBeDisabled());

    resolveDesigners(listResponse(defaultDesigners()));
    resolveSources(listResponse(defaultSources()));
    await waitFor(() => expect(designerSelect).toBeEnabled());
  });
});