import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import SettingsView from "../SettingsView.svelte";
import type {
  SettingsViewModel,
  DbStats,
} from "../../types/ipc";

// ---------------------------------------------------------------------------
// Mock the command adapter and toast store so all logic branches can be driven
// from the test.
// ---------------------------------------------------------------------------
const getSettingsViewModelMock = vi.hoisted(() => vi.fn());
const saveSettingsMock = vi.hoisted(() => vi.fn());
const browseSettingsDataRootMock = vi.hoisted(() => vi.fn());
const setConfiguredDataRootMock = vi.hoisted(() => vi.fn());
const restartApplicationMock = vi.hoisted(() => vi.fn());
const startCatalogueStorageMigrationMock = vi.hoisted(() => vi.fn());
const cancelCatalogueStorageMigrationMock = vi.hoisted(() => vi.fn());
const listenCatalogueStorageMigrationProgressMock = vi.hoisted(() => vi.fn());
const getDbStatsMock = vi.hoisted(() => vi.fn());
const compactDatabaseMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../../api/commandAdapter", () => ({
  getSettingsViewModel: getSettingsViewModelMock,
  saveSettings: saveSettingsMock,
  browseSettingsDataRoot: browseSettingsDataRootMock,
  setConfiguredDataRoot: setConfiguredDataRootMock,
  restartApplication: restartApplicationMock,
  startCatalogueStorageMigration: startCatalogueStorageMigrationMock,
  cancelCatalogueStorageMigration: cancelCatalogueStorageMigrationMock,
  listenCatalogueStorageMigrationProgress: listenCatalogueStorageMigrationProgressMock,
  getDbStats: getDbStatsMock,
  compactDatabase: compactDatabaseMock,
}));

vi.mock("../../stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

// ---------------------------------------------------------------------------
// Helpers & Fixtures
// ---------------------------------------------------------------------------
const defaultModel: SettingsViewModel = {
  preview_3d_profile: "balanced",
  google_api_key: "AIza-SY-Key",
  has_google_api_key: true,
  ai_tier2_auto: true,
  ai_tier3_auto: false,
  ai_batch_size: "100",
  ai_delay: "6.0",
  import_commit_batch_size: "10",
  import_last_browse_folder: "",
  can_configure_data_root: true,
  data_root: "D:\\EmbroideryData",
  database_path: "D:\\EmbroideryData\\catalogue.db",
  log_folder: "D:\\EmbroideryData\\logs",
  app_mode: "development",
  ai_tagging_help_url: "#/help",
  db_idle_check_interval_secs: "1800",
};

const defaultStats: DbStats = {
  file_size_bytes: 5000000,
  page_count: 500,
  freelist_count: 100,
  page_size: 4096,
  free_ratio: 0.2,
  reclaimable_bytes: 1000000,
};

function mockSettings(model: Partial<SettingsViewModel> = defaultModel) {
  getSettingsViewModelMock.mockResolvedValue({ model, source: "rust" });
}

function mockDbStats(stats: DbStats | null = defaultStats) {
  getDbStatsMock.mockResolvedValue({ stats, source: "rust" });
}

function renderView() {
  return render(SettingsView);
}

async function waitForSettingsLoaded() {
  await waitFor(() => {
    expect(getSettingsViewModelMock).toHaveBeenCalled();
  });
  await tick();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("SettingsView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    addToastMock.mockClear();
    mockSettings();
    mockDbStats();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // -- Loading -------------------------------------------------------------

  it("renders the loading indicator while settings are still loading", () => {
    getSettingsViewModelMock.mockReturnValue(new Promise(() => {}));

    renderView();

    expect(screen.getByText("Loading settings...")).toBeInTheDocument();
    expect(screen.getByText("Application Settings")).toBeInTheDocument();
  });

  it("loads settings on mount and renders every model field", async () => {
    renderView();

    await waitForSettingsLoaded();

    expect(screen.queryByText("Loading settings...")).not.toBeInTheDocument();

    expect(screen.getByLabelText("API key")).toHaveValue("AIza-SY-Key");
    // Number inputs report numeric values in jsdom.
    expect(screen.getByLabelText(/AI tagging batch size/)).toHaveValue(100);
    expect(screen.getByLabelText(/Delay between Gemini calls/)).toHaveValue(6.0);
    expect(screen.getByLabelText(/Import database commit batch size/)).toHaveValue(10);
    expect(screen.getByLabelText(/Database health check interval/)).toHaveValue(1800);
    expect(screen.getByLabelText(/Tier 2/)).toBeChecked();
    expect(screen.getByLabelText(/Tier 3/)).not.toBeChecked();

    const dataRootInput = screen.getByLabelText("Catalogue data location");
    expect(dataRootInput).toHaveValue("D:\\EmbroideryData");
    expect(screen.getByRole("button", { name: "Browse…" })).toBeInTheDocument();
    expect(screen.getByText("D:\\EmbroideryData\\logs")).toBeInTheDocument();
    expect(screen.getByText("D:\\EmbroideryData\\catalogue.db")).toBeInTheDocument();
    expect(screen.getByText("D:\\EmbroideryData")).toBeInTheDocument();

    // Storage grid rendered from db stats.
    expect(screen.getByText("4.8 MB")).toBeInTheDocument();
    expect(screen.getByText("977 KB")).toBeInTheDocument();
  });

  it("shows an error toast when loading settings fails", async () => {
    getSettingsViewModelMock.mockRejectedValue(new Error("boom"));

    renderView();

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not load settings: Error: boom",
        "error"
      );
    });
  });

  it("logs to console when database stats fail to load", async () => {
    const infoSpy = vi
      .spyOn(console, "info")
      .mockImplementation(() => {});
    getDbStatsMock.mockRejectedValue(new Error("db busy"));

    renderView();

    await waitFor(() => {
      expect(infoSpy).toHaveBeenCalled();
    });

    infoSpy.mockRestore();
  });

  it("applies fallback defaults and hides data-root controls when the model is empty", async () => {
    mockSettings({});
    mockDbStats(null);

    renderView();

    await waitForSettingsLoaded();

    // Fallback defaults applied from the empty model.
    expect(screen.getByLabelText("API key")).toHaveValue("");
    expect(screen.getByLabelText(/Database health check interval/)).toHaveValue(1800);
    expect(screen.getByLabelText(/Tier 2/)).not.toBeChecked();
    expect(screen.getByLabelText(/Tier 3/)).not.toBeChecked();

    // can_configure_data_root is false → browse input hidden, dev-mode note shown.
    expect(
      screen.queryByRole("button", { name: "Browse…" })
    ).not.toBeInTheDocument();
    expect(screen.getByText(/In development mode this location follows/)).toBeInTheDocument();

    // No API key → blue notice + "leave blank" hint.
    expect(screen.getByText(/No API key is saved/)).toBeInTheDocument();
    expect(
      screen.getByText(/Leave this blank if you only want keyword-based tagging/)
    ).toBeInTheDocument();

    // dbStats null → "unavailable" message.
    expect(
      screen.getByText("Database statistics unavailable.")
    ).toBeInTheDocument();
  });

  // -- API key visibility & notices ----------------------------------------

  it("toggles the API key input between password and text", async () => {
    renderView();

    await waitForSettingsLoaded();

    const toggle = screen.getByRole("button", { name: "Show or hide API key" });
    const input = screen.getByLabelText("API key");

    expect(input).toHaveAttribute("type", "password");
    expect(toggle).toHaveAttribute("aria-pressed", "false");
    expect(toggle).toHaveTextContent("👁");

    await fireEvent.click(toggle);
    await tick();

    expect(input).toHaveAttribute("type", "text");
    expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(toggle).toHaveTextContent("🙈");

    await fireEvent.click(toggle);
    await tick();

    expect(input).toHaveAttribute("type", "password");
    expect(toggle).toHaveAttribute("aria-pressed", "false");
  });

  it("shows the cost notice and saved-key hint when an API key is configured", async () => {
    renderView();

    await waitForSettingsLoaded();

    expect(screen.getByText(/A key is currently saved in/)).toBeInTheDocument();
    expect(
      screen.queryByText(/Leave this blank if you only want keyword-based tagging/)
    ).not.toBeInTheDocument();
    expect(screen.getByText("⚠ Cost notice:")).toBeInTheDocument();
    expect(screen.queryByText(/No API key is saved/)).not.toBeInTheDocument();
  });

  it("treats a whitespace-only API key as empty", async () => {
    mockSettings({ ...defaultModel, google_api_key: "   " });

    renderView();

    await waitForSettingsLoaded();

    expect(
      screen.getByText(/Leave this blank if you only want keyword-based tagging/)
    ).toBeInTheDocument();
    expect(screen.getByText(/No API key is saved/)).toBeInTheDocument();
    expect(screen.queryByText("⚠ Cost notice:")).not.toBeInTheDocument();
  });

  // -- Save settings -------------------------------------------------------

  it("saves settings successfully with the current form values", async () => {
    saveSettingsMock.mockResolvedValue({
      source: "rust",
      saved: true,
      message: "All good.",
      persisted: true,
    });

    renderView();

    await waitForSettingsLoaded();

    const form = document.querySelector("form");
    if (!form) throw new Error("Settings form not found");
    await fireEvent.submit(form);

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        google_api_key: "AIza-SY-Key",
        ai_tier2_auto: true,
        ai_tier3_auto: false,
        ai_batch_size: "100",
        ai_delay: "6.0",
        import_commit_batch_size: "10",
        data_root: "D:\\EmbroideryData",
        db_idle_check_interval_secs: "1800",
      });
    });
    expect(addToastMock).toHaveBeenCalledWith("All good.", "success");
  });

  it("shows an error toast when save reports failure", async () => {
    saveSettingsMock.mockResolvedValue({
      source: "mock",
      saved: false,
      message: "Nope",
      persisted: false,
    });

    renderView();

    await waitForSettingsLoaded();

    const form = document.querySelector("form");
    if (!form) throw new Error("Settings form not found");
    await fireEvent.submit(form);

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Nope", "error");
    });
  });

  it("shows an error toast when save throws", async () => {
    saveSettingsMock.mockRejectedValue(new Error("boom"));

    renderView();

    await waitForSettingsLoaded();

    const form = document.querySelector("form");
    if (!form) throw new Error("Settings form not found");
    await fireEvent.submit(form);

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not save settings: Error: boom",
        "error"
      );
    });
  });

  it("disables the submit button and shows 'Saving...' while saving", async () => {
    saveSettingsMock.mockReturnValue(new Promise(() => {}));

    renderView();

    await waitForSettingsLoaded();

    const form = document.querySelector("form");
    if (!form) throw new Error("Settings form not found");
    await fireEvent.submit(form);
    await tick();

    const savingButton = screen.getByRole("button", { name: "Saving..." });
    expect(savingButton).toBeDisabled();
  });

  // -- Data root browsing --------------------------------------------------

  it("starts the catalogue migration for the picked folder and shows the restart dialog on success", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    listenCatalogueStorageMigrationProgressMock.mockResolvedValue(() => {});
    startCatalogueStorageMigrationMock.mockResolvedValue({
      source: "rust",
      summary: {
        success: true,
        source_root: "D:\\EmbroideryData",
        target_root: "E:\\NewData",
        database_bytes: 10,
        asset_items: 3,
        asset_bytes: 40,
        requires_restart: true,
      },
    });
    restartApplicationMock.mockResolvedValue({ source: "rust", restarted: true });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() => {
      expect(browseSettingsDataRootMock).toHaveBeenCalledWith("D:\\EmbroideryData");
    });
    expect(listenCatalogueStorageMigrationProgressMock).toHaveBeenCalledTimes(1);
    expect(startCatalogueStorageMigrationMock).toHaveBeenCalledWith("E:\\NewData");
    expect(screen.getByLabelText("Catalogue data location")).toHaveValue("E:\\NewData");
    expect(screen.getByTestId("settings-restart-dialog")).toBeInTheDocument();
  });

  it("shows an error toast and retains the old path when the migration cannot start", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    listenCatalogueStorageMigrationProgressMock.mockResolvedValue(() => {});
    startCatalogueStorageMigrationMock.mockResolvedValue({
      source: "mock",
      summary: null,
      error: "insufficient free space",
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not start catalogue migration: insufficient free space",
        "error"
      );
    });
    expect(screen.getByLabelText("Catalogue data location")).toHaveValue("D:\\EmbroideryData");
    expect(screen.queryByTestId("settings-restart-dialog")).not.toBeInTheDocument();
    // The migration dialog stays visible to surface the error message.
    expect(screen.getByTestId("catalogue-migration-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("catalogue-migration-error")).toHaveTextContent("insufficient free space");
  });

  it("provides a Close button that dismisses the terminal-error migration dialog", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    listenCatalogueStorageMigrationProgressMock.mockResolvedValue(() => {});
    startCatalogueStorageMigrationMock.mockResolvedValue({
      source: "mock",
      summary: null,
      error: "disk full",
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() => {
      expect(screen.getByTestId("catalogue-migration-dialog")).toBeInTheDocument();
    });

    // Terminal error state must offer a dismiss control (regression: previously
    // the modal had no button and the only escape was quitting the app).
    const closeButton = screen.getByTestId("close-catalogue-migration");
    expect(closeButton).toHaveTextContent("Close");

    await fireEvent.click(closeButton);
    await tick();

    expect(screen.queryByTestId("catalogue-migration-dialog")).not.toBeInTheDocument();
    expect(
      screen.getByLabelText("Catalogue data location")
    ).toHaveValue("D:\\EmbroideryData");
  });

  it("shows an error toast when restarting fails after the migration succeeds", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    listenCatalogueStorageMigrationProgressMock.mockResolvedValue(() => {});
    startCatalogueStorageMigrationMock.mockResolvedValue({
      source: "rust",
      summary: {
        success: true,
        source_root: "D:\\EmbroideryData",
        target_root: "E:\\NewData",
        database_bytes: 10,
        asset_items: 0,
        asset_bytes: 0,
        requires_restart: true,
      },
    });
    restartApplicationMock.mockResolvedValue({
      source: "mock",
      restarted: false,
      error: "restart boom",
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    // Restart dialog appears after the migration completes.
    await waitFor(() => {
      expect(screen.getByTestId("settings-restart-dialog")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("settings-restart-now"));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not restart the application: restart boom. Please close and reopen it manually so your new data location takes effect.",
        "error"
      );
    });
    expect(screen.queryByTestId("settings-restart-dialog")).not.toBeInTheDocument();
  });

  it("renders the migration progress dialog while migrating", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    let capturedCallback: ((progress: unknown) => void) | null = null;
    listenCatalogueStorageMigrationProgressMock.mockImplementation(
      (callback: (progress: unknown) => void) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      }
    );
    startCatalogueStorageMigrationMock.mockResolvedValue(new Promise(() => {})); // stay migrating

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    // The picker returns a path; migration starts and the modal appears.
    await waitFor(() => {
      expect(screen.getByTestId("catalogue-migration-dialog")).toBeInTheDocument();
    });

    // Simulate a progress event from the backend.
    (capturedCallback as ((progress: unknown) => void) | null)?.({
      current_phase: "assets",
      items_copied: 2,
      total_items: 10,
      bytes_copied: 50,
      total_bytes: 100,
      status_message: "Copying files…",
      percent: 0.5,
      error: null,
    });
    await tick();

    expect(screen.getByTestId("catalogue-migration-status")).toHaveTextContent("Copying files…");
    expect(screen.getByTestId("catalogue-migration-counts")).toHaveTextContent("2 of 10 files");
  });

  it("cancels the running migration when the Cancel button is clicked", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: "E:\\NewData",
      error: null,
    });
    listenCatalogueStorageMigrationProgressMock.mockResolvedValue(() => {});
    startCatalogueStorageMigrationMock.mockResolvedValue(new Promise(() => {})); // stay migrating
    cancelCatalogueStorageMigrationMock.mockResolvedValue({ source: "rust", cancelled: true });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() => {
      expect(screen.getByTestId("catalogue-migration-dialog")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("cancel-catalogue-migration"));

    await waitFor(() => {
      expect(cancelCatalogueStorageMigrationMock).toHaveBeenCalledTimes(1);
    });
  });

  it("shows an error toast when the folder picker fails", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "mock",
      path: null,
      error: "Picker exploded",
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("Picker exploded", "error");
    });
  });

  it("does nothing when the folder picker returns no path and no error", async () => {
    browseSettingsDataRootMock.mockResolvedValue({
      source: "mock",
      path: null,
      error: null,
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(screen.getByRole("button", { name: "Browse…" }));

    await tick();
    expect(addToastMock).not.toHaveBeenCalled();
    expect(screen.getByLabelText("Catalogue data location")).toHaveValue("D:\\EmbroideryData");
  });

  // -- Database compaction -------------------------------------------------

  it("compacts the database and shows a success toast, then reloads stats", async () => {
    compactDatabaseMock.mockResolvedValue({
      source: "rust",
      result: {
        file_size_before: 2048,
        file_size_after: 1024,
        pages_reclaimed: 42,
        duration_ms: 5,
      },
      message: "ok",
    });

    renderView();

    await waitForSettingsLoaded();
    expect(getDbStatsMock).toHaveBeenCalledTimes(1);

    await fireEvent.click(
      screen.getByRole("button", { name: "Optimize & Compact Database" })
    );

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Database compacted — 42 pages reclaimed (2.0 KB → 1.0 KB)",
        "success"
      );
    });
    expect(getDbStatsMock).toHaveBeenCalledTimes(2);
  });

  it("shows the compaction error message when no result is returned", async () => {
    compactDatabaseMock.mockResolvedValue({
      source: "mock",
      result: null,
      message: "fail",
      error: "disk full",
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(
      screen.getByRole("button", { name: "Optimize & Compact Database" })
    );

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith("disk full", "error");
    });
  });

  it("falls back to a generic message when compaction fails without an error", async () => {
    compactDatabaseMock.mockResolvedValue({
      source: "mock",
      result: null,
      message: "fail",
      error: null,
    });

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(
      screen.getByRole("button", { name: "Optimize & Compact Database" })
    );

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not compact database.",
        "error"
      );
    });
  });

  it("shows an error toast when compaction throws", async () => {
    compactDatabaseMock.mockRejectedValue(new Error("boom"));

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(
      screen.getByRole("button", { name: "Optimize & Compact Database" })
    );

    await waitFor(() => {
      expect(addToastMock).toHaveBeenCalledWith(
        "Could not compact database: Error: boom",
        "error"
      );
    });
  });

  it("disables the compact button and shows 'Compacting…' while running", async () => {
    compactDatabaseMock.mockReturnValue(new Promise(() => {}));

    renderView();

    await waitForSettingsLoaded();

    await fireEvent.click(
      screen.getByRole("button", { name: "Optimize & Compact Database" })
    );
    await tick();

    const compactingButton = screen.getByRole("button", { name: "Compacting…" });
    expect(compactingButton).toBeDisabled();
  });

  // -- formatBytes branches ------------------------------------------------

  it("formats byte sizes with unit suffixes", async () => {
    mockDbStats({
      ...defaultStats,
      file_size_bytes: 1099511627776, // 1 TB
      reclaimable_bytes: 1024, // 1 KB
    });

    renderView();

    await waitForSettingsLoaded();

    expect(screen.getByText("1.0 TB")).toBeInTheDocument();
    expect(screen.getByText("1.0 KB")).toBeInTheDocument();
  });

  it("renders '0 B' for zero and NaN byte values", async () => {
    mockDbStats({
      ...defaultStats,
      file_size_bytes: 0,
      reclaimable_bytes: NaN,
    });

    renderView();

    await waitForSettingsLoaded();

    expect(screen.getAllByText("0 B")).toHaveLength(2);
  });

  it("renders whole numbers for small byte values and large KB values", async () => {
    mockDbStats({
      ...defaultStats,
      file_size_bytes: 50,
      reclaimable_bytes: 150000, // 146.48 KB → rounds to 146 KB
    });

    renderView();

    await waitForSettingsLoaded();

    expect(screen.getByText("50 B")).toBeInTheDocument();
    expect(screen.getByText("146 KB")).toBeInTheDocument();
  });

  it("renders the 'Database statistics unavailable' message when stats are missing", async () => {
    mockDbStats(null);

    renderView();

    await waitForSettingsLoaded();

    expect(
      screen.getByText("Database statistics unavailable.")
    ).toBeInTheDocument();
  });

  // -- Help link -----------------------------------------------------------

  it("links the help button to the configured help URL", async () => {
    mockSettings({ ...defaultModel, ai_tagging_help_url: "#/help/settings" });

    renderView();

    await waitForSettingsLoaded();

    const link = screen.getByText("Press here for more information.");
    expect(link).toHaveAttribute("href", "#/help/settings");
  });
});