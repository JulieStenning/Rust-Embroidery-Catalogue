import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent, within } from "@testing-library/svelte";
import { tick } from "svelte";
import BackupView from "../BackupView.svelte";

const adapterMocks = vi.hoisted(() => ({
  getBackupViewModel: vi.fn(),
  saveBackupSettings: vi.fn(),
  browseBackupFolder: vi.fn(),
  runDatabaseBackup: vi.fn(),
  runDesignsBackup: vi.fn(),
  runBothBackups: vi.fn(),
  requestCancelBackup: vi.fn(),
  getSettingsViewModel: vi.fn(),
  browseRestoreFile: vi.fn(),
  restoreDatabase: vi.fn(),
  restoreDesignsIncremental: vi.fn(),
  restoreBoth: vi.fn(),
  importUnmatchedDesignFiles: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

const toastMocks = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore", () => toastMocks);

const DB_DEST = "C:\\Backups\\EmbroideryDB";
const DESIGNS_DEST = "C:\\Backups\\EmbroideryDesigns";
const DB_SRC = "C:\\Data\\database\\catalogue.db";
const DESIGNS_SRC = "C:\\Data\\MachineEmbroideryDesigns";

const backupModel = (o: Record<string, unknown> = {}) => ({
  db_destination: DB_DEST,
  designs_destination: DESIGNS_DEST,
  db_source_path: DB_SRC,
  designs_source_path: DESIGNS_SRC,
  ...o,
});

const backupResponse = (model: Record<string, unknown>) => ({
  source: "rust",
  model,
});

const settingsResponse = (dataRoot = "") => ({
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
    data_root: dataRoot,
    database_path: "",
    log_folder: "",
    app_mode: "development",
    ai_tagging_help_url: "#/help",
  },
});

const dbResult = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  success: true,
  backup_path: DB_DEST + "\\catalogue_2026-08-01.db",
  size_bytes: 5242880,
  completed_at: "2026-08-01T21:00:00Z",
  error: "",
  cancelled: false,
  ...o,
});

const designsResult = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  success: true,
  scanned: 100,
  copied: 25,
  updated: 5,
  unchanged: 70,
  archived: 2,
  total_bytes_copied: 1048576,
  completed_at: "2026-08-01T21:00:00Z",
  error: "",
  cancelled: false,
  ...o,
});

const bothResult = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  database: {
    success: true,
    backup_path: "C:\\x.db",
    size_bytes: 1024,
    completed_at: "",
    error: "",
    cancelled: false,
  },
  designs: {
    success: true,
    scanned: 10,
    copied: 2,
    updated: 0,
    unchanged: 8,
    archived: 0,
    total_bytes_copied: 0,
    completed_at: "",
    error: "",
    cancelled: false,
  },
  ...o,
});

const saveResult = (o: Record<string, unknown> = {}) => ({
  source: "rust",
  persisted: true,
  saved: true,
  message: "Backup destinations saved.",
  db_destination: DB_DEST,
  designs_destination: DESIGNS_DEST,
  ...o,
});

function element<T extends Element>(value: T | null | undefined): T {
  if (!value) throw new Error("Expected element to exist.");
  return value;
}

function normalizedText(expected: string) {
  return (_c: string, node: Element | null) =>
    node !== null && (node.textContent ?? "").replace(/\s+/g, " ").trim() === expected;
}

function mockDefaults() {
  adapterMocks.getSettingsViewModel.mockResolvedValue(settingsResponse(""));
  adapterMocks.getBackupViewModel.mockResolvedValue(backupResponse(backupModel()));
  adapterMocks.saveBackupSettings.mockResolvedValue(saveResult());
  adapterMocks.browseBackupFolder.mockResolvedValue({ source: "rust", path: null, error: null });
  adapterMocks.runDatabaseBackup.mockResolvedValue(dbResult());
  adapterMocks.runDesignsBackup.mockResolvedValue(designsResult());
  adapterMocks.runBothBackups.mockResolvedValue(bothResult());
  adapterMocks.requestCancelBackup.mockResolvedValue({
    source: "rust",
    cancel_requested: true,
  });
  adapterMocks.browseRestoreFile.mockResolvedValue({
    source: "rust",
    path: null,
    error: null,
  });
  adapterMocks.restoreDatabase.mockResolvedValue({
    source: "rust",
    success: true,
    restored_path: "C:\\x.db",
    rollback_copy_path: "C:\\x.pre-restore-1.db",
    design_count: 12,
    schema_version_hint: 3,
    previous_schema_version_hint: 3,
    rolled_back: false,
    error: null,
  });
  adapterMocks.restoreDesignsIncremental.mockResolvedValue({
    source: "rust",
    success: true,
    scanned: 20,
    copied: 4,
    updated: 1,
    skipped: 15,
    total_bytes_copied: 1024,
    error: null,
  });
  adapterMocks.restoreBoth.mockResolvedValue({
    source: "rust",
    database: {
      success: true,
      restored_path: "C:\\x.db",
      rollback_copy_path: "C:\\x.pre-restore-1.db",
      design_count: 12,
      schema_version_hint: 3,
      previous_schema_version_hint: 3,
      rolled_back: false,
      error: null,
    },
    designs: {
      success: true,
      scanned: 20,
      copied: 4,
      updated: 1,
      skipped: 15,
      total_bytes_copied: 1024,
      error: null,
    },
    unmatched: null,
  });
  adapterMocks.importUnmatchedDesignFiles.mockResolvedValue({
    source: "rust",
    detected: 0,
    imported: 0,
    failed: 0,
    failed_samples: [],
  });
}

describe("BackupView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockDefaults();
  });

  describe("page chrome", () => {
    it("renders the page heading 'Backup & Restore'", () => {
      render(BackupView);
      expect(screen.getByRole("heading", { name: "Backup & Restore" })).toBeInTheDocument();
    });

    it("renders the description paragraph", () => {
      render(BackupView);
      expect(
        screen.getByText(
          normalizedText(
            "Back up your catalogue database and embroidery design files to folders of your choice, or restore them from an earlier snapshot. The database backup saves your catalogue data, settings, tags, and projects; the designs backup saves the actual embroidery files."
          )
        )
      ).toBeInTheDocument();
    });

    it("renders the important notice box", () => {
      render(BackupView);
      expect(screen.getByText("Important")).toBeInTheDocument();
      expect(
        screen.getByText(
          normalizedText(
            "Ensure backup folders reside on a separate drive from your library."
          )
        )
      ).toBeInTheDocument();
    });

    it("renders all four card section headings", () => {
      render(BackupView);
      expect(screen.getByRole("heading", { name: "Backup Destinations" })).toBeInTheDocument();
      expect(screen.getByRole("heading", { name: "Database Backup" })).toBeInTheDocument();
      expect(screen.getByRole("heading", { name: "Designs Backup" })).toBeInTheDocument();
      expect(screen.getByRole("heading", { name: "Backup Everything Now" })).toBeInTheDocument();
    });

    it("renders the destination input labels", () => {
      render(BackupView);
      expect(screen.getByLabelText("Database backup folder")).toBeInTheDocument();
      expect(screen.getByLabelText("Designs backup folder")).toBeInTheDocument();
    });

    it("renders the destination placeholder text", () => {
      render(BackupView);
      // Svelte keeps double backslashes literally in static attributes.
      expect(screen.getByPlaceholderText(/e\.g\. .*EmbroideryDB/)).toBeInTheDocument();
      expect(screen.getByPlaceholderText(/e\.g\. .*EmbroideryDesigns/)).toBeInTheDocument();
    });

    it("renders two Browse buttons and Save button", () => {
      render(BackupView);
      expect(screen.getAllByRole("button", { name: /Browse/ })).toHaveLength(2);
      expect(screen.getByRole("button", { name: "Save destinations" })).toBeInTheDocument();
    });

    it("renders the three backup action buttons", () => {
      render(BackupView);
      expect(screen.getByRole("button", { name: "Backup Database Now" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Run incremental backup" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Backup Everything Now" })).toBeInTheDocument();
    });
  });

  describe("initial load", () => {
    it("calls getSettingsViewModel and getBackupViewModel on mount", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getSettingsViewModel).toHaveBeenCalledTimes(1));
      expect(adapterMocks.getBackupViewModel).toHaveBeenCalledTimes(1);
    });

    it("populates destination inputs from the loaded model", async () => {
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      const designsInput = screen.getByLabelText("Designs backup folder") as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      expect(designsInput.value).toBe(DESIGNS_DEST);
    });

    it("renders the source paths from the model", async () => {
      render(BackupView);
      await waitFor(() => expect(screen.getByText(DB_SRC)).toBeInTheDocument());
      expect(screen.getByText(DESIGNS_SRC)).toBeInTheDocument();
    });

    it("renders '(not set)' labels when destinations are empty", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_destination: "", designs_destination: "" }))
      );
      render(BackupView);
      await waitFor(() => expect(screen.getAllByText("(not set)")).toHaveLength(2));
    });

    it("handles null response by defaulting to empty destinations", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(null);
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(""));
      expect(screen.getAllByText("(not set)")).toHaveLength(2);
    });

    it("uses data_root fallback for source paths when model has no source path", async () => {
      adapterMocks.getSettingsViewModel.mockResolvedValue(settingsResponse("C:\\MyData"));
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_source_path: "", designs_source_path: "" }))
      );
      render(BackupView);
      await waitFor(() =>
        expect(screen.getByText("C:\\MyData\\database\\catalogue.db")).toBeInTheDocument()
      );
      expect(screen.getByText("C:\\MyData\\MachineEmbroideryDesigns")).toBeInTheDocument();
    });

    it("shows '(not available yet)' when no fallback is known", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_source_path: "", designs_source_path: "" }))
      );
      render(BackupView);
      await waitFor(() => expect(screen.getAllByText("(not available yet)")).toHaveLength(2));
    });

    it("logs and continues when getSettingsViewModel rejects", async () => {
      const spy = vi.spyOn(console, "error").mockImplementation(() => {});
      adapterMocks.getSettingsViewModel.mockRejectedValue(new Error("settings down"));
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_source_path: "", designs_source_path: "" }))
      );
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalledTimes(1));
      expect(spy).toHaveBeenCalled();
      expect(screen.getAllByText("(not available yet)")).toHaveLength(2);
      spy.mockRestore();
    });

    it("shows an error toast when getBackupViewModel rejects", async () => {
      adapterMocks.getBackupViewModel.mockRejectedValue(new Error("backup down"));
      render(BackupView);
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Could not load backup settings: Error: backup down",
          "error"
        )
      );
    });
  });

  describe("backup destinations form", () => {
    it("disables Save when there are no unsaved changes", async () => {
      render(BackupView);
      const btn = screen.getByRole("button", { name: "Save destinations" });
      await waitFor(() => expect(btn).toBeDisabled());
    });

    it("enables Save when a destination input changes", async () => {
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      // Wait for the async load to finish so it doesn't overwrite our typed value.
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      await fireEvent.input(dbInput, { target: { value: "D:\\NewDb" } });
      expect(screen.getByRole("button", { name: "Save destinations" })).toBeEnabled();
    });

    it("shows error toast when submitting with no unsaved changes", async () => {
      render(BackupView);
      const form = element(document.querySelector("form"));
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Save destinations" })).toBeDisabled()
      );
      await fireEvent.submit(form);
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "There are no destination changes to save.",
        "error"
      );
      expect(adapterMocks.saveBackupSettings).not.toHaveBeenCalled();
    });

    it("saves changed destinations and shows a success toast", async () => {
      adapterMocks.saveBackupSettings.mockResolvedValue(
        saveResult({
          db_destination: "D:\\NewDb",
          designs_destination: "D:\\NewDesigns",
        })
      );
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      const designsInput = screen.getByLabelText("Designs backup folder") as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      await fireEvent.input(dbInput, { target: { value: "D:\\NewDb" } });
      await fireEvent.input(designsInput, { target: { value: "D:\\NewDesigns" } });
      await fireEvent.submit(element(document.querySelector("form")));

      await waitFor(() =>
        expect(adapterMocks.saveBackupSettings).toHaveBeenCalledWith({
          dbDestination: "D:\\NewDb",
          designsDestination: "D:\\NewDesigns",
        })
      );
      expect(toastMocks.addToast).toHaveBeenCalledWith("Backup destinations saved.", "success");
      expect(dbInput.value).toBe("D:\\NewDb");
      expect(screen.getByRole("button", { name: "Save destinations" })).toBeDisabled();
    });

    it("handles a failed save with an error toast", async () => {
      adapterMocks.saveBackupSettings.mockResolvedValue({
        source: "mock",
        persisted: false,
        saved: false,
        message: "Could not save backup destinations: disk full",
      });
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      await fireEvent.input(dbInput, { target: { value: "D:\\NewDb" } });
      await fireEvent.submit(element(document.querySelector("form")));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Could not save backup destinations: disk full",
          "error"
        )
      );
    });

    it("uses the response's normalized destinations when provided", async () => {
      adapterMocks.saveBackupSettings.mockResolvedValue(
        saveResult({
          message: "Saved.",
          db_destination: "X:\\TrimmedDb",
          designs_destination: "X:\\TrimmedDesigns",
        })
      );
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      await fireEvent.input(dbInput, { target: { value: "X:\\TrimmedDb" } });
      await fireEvent.submit(element(document.querySelector("form")));
      await waitFor(() => expect(toastMocks.addToast).toHaveBeenCalledWith("Saved.", "success"));
      expect(dbInput.value).toBe("X:\\TrimmedDb");
    });
  });

  describe("browse buttons", () => {
    it("calls browseBackupFolder with DB destination", async () => {
      render(BackupView);
      await screen.findByLabelText("Database backup folder");
      await fireEvent.click(screen.getAllByRole("button", { name: /Browse/ })[0]);
      expect(adapterMocks.browseBackupFolder).toHaveBeenCalledWith(DB_DEST);
    });

    it("calls browseBackupFolder with designs destination", async () => {
      render(BackupView);
      await screen.findByLabelText("Database backup folder");
      await fireEvent.click(screen.getAllByRole("button", { name: /Browse/ })[1]);
      expect(adapterMocks.browseBackupFolder).toHaveBeenCalledWith(DESIGNS_DEST);
    });

    it("updates DB destination when a path is returned", async () => {
      adapterMocks.browseBackupFolder.mockResolvedValue({
        source: "rust",
        path: "E:\\NewDbFolder",
        error: null,
      });
      render(BackupView);
      const dbInput = (await screen.findByLabelText("Database backup folder")) as HTMLInputElement;
      await waitFor(() => expect(dbInput.value).toBe(DB_DEST));
      // Auto-save echoes the picked destination back so it stays in the input.
      adapterMocks.saveBackupSettings.mockResolvedValue({
        saved: true,
        message: "Backup destinations saved.",
        db_destination: "E:\\NewDbFolder",
        designs_destination: DESIGNS_DEST,
      });
      await fireEvent.click(screen.getAllByRole("button", { name: /Browse/ })[0]);
      await waitFor(() => expect(dbInput.value).toBe("E:\\NewDbFolder"));
    });

    it("updates designs destination when a path is returned", async () => {
      adapterMocks.browseBackupFolder.mockResolvedValue({
        source: "rust",
        path: "E:\\NewDesignsFolder",
        error: null,
      });
      render(BackupView);
      const designsInput = screen.getByLabelText("Designs backup folder") as HTMLInputElement;
      await waitFor(() => expect(designsInput.value).toBe(DESIGNS_DEST));
      adapterMocks.saveBackupSettings.mockResolvedValue({
        saved: true,
        message: "Backup destinations saved.",
        db_destination: DB_DEST,
        designs_destination: "E:\\NewDesignsFolder",
      });
      await fireEvent.click(screen.getAllByRole("button", { name: /Browse/ })[1]);
      await waitFor(() => expect(designsInput.value).toBe("E:\\NewDesignsFolder"));
    });

    it("shows error toast when browse returns an error", async () => {
      adapterMocks.browseBackupFolder.mockResolvedValue({
        source: "mock",
        path: null,
        error: "Folder picker unavailable.",
      });
      render(BackupView);
      await screen.findByLabelText("Database backup folder");
      await fireEvent.click(screen.getAllByRole("button", { name: /Browse/ })[0]);
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith("Folder picker unavailable.", "error")
      );
    });
  });

  describe("database backup card", () => {
    async function dbButton() {
      const heading = screen.getByRole("heading", { name: "Database Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Database Now" }));
      return card.getByRole("button", { name: "Backup Database Now" }) as HTMLButtonElement;
    }

    it("disables the button when no DB destination is saved", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_destination: "" }))
      );
      render(BackupView);
      const button = await dbButton();
      expect(button).toBeDisabled();
      expect(button).toHaveAttribute("title", "Set a database backup destination first");
    });

    it("enables the button when a DB destination is saved", async () => {
      render(BackupView);
      const button = await dbButton();
      expect(button).toBeEnabled();
    });

    it("runs backup and shows success toast with MB", async () => {
      render(BackupView);
      await fireEvent.click(await dbButton());
      await waitFor(() => expect(adapterMocks.runDatabaseBackup).toHaveBeenCalledTimes(1));
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Database backup created: " + DB_DEST + "\\catalogue_2026-08-01.db (5.00 MB).",
        "success"
      );
    });

    it("shows error toast when backup fails", async () => {
      adapterMocks.runDatabaseBackup.mockResolvedValue(
        dbResult({ success: false, error: "Disk full" })
      );
      render(BackupView);
      await fireEvent.click(await dbButton());
      await waitFor(() => expect(toastMocks.addToast).toHaveBeenCalledWith("Disk full", "error"));
    });

    it("uses fallback error message when no error returned", async () => {
      adapterMocks.runDatabaseBackup.mockResolvedValue(dbResult({ success: false, error: "" }));
      render(BackupView);
      await fireEvent.click(await dbButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith("Database backup failed.", "error")
      );
    });
  });

  describe("designs backup card", () => {
    async function designsButton() {
      const heading = screen.getByRole("heading", { name: "Designs Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Run incremental backup" }));
      return card.getByRole("button", { name: "Run incremental backup" }) as HTMLButtonElement;
    }

    it("disables the button when no designs destination is saved", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ designs_destination: "" }))
      );
      render(BackupView);
      const button = await designsButton();
      expect(button).toBeDisabled();
      expect(button).toHaveAttribute("title", "Set a designs backup destination first");
    });

    it("enables the button when a designs destination is saved", async () => {
      render(BackupView);
      expect(await designsButton()).toBeEnabled();
    });

    it("runs backup and shows success toast with counts", async () => {
      render(BackupView);
      await fireEvent.click(await designsButton());
      await waitFor(() => expect(adapterMocks.runDesignsBackup).toHaveBeenCalledTimes(1));
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Designs backup complete: scanned 100, copied 25, updated 5, unchanged 70, archived 2.",
        "success"
      );
    });

    it("shows error toast when backup fails", async () => {
      adapterMocks.runDesignsBackup.mockResolvedValue(
        designsResult({ success: false, error: "Permission denied" })
      );
      render(BackupView);
      await fireEvent.click(await designsButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith("Permission denied", "error")
      );
    });

    it("uses fallback error message when no error returned", async () => {
      adapterMocks.runDesignsBackup.mockResolvedValue(designsResult({ success: false, error: "" }));
      render(BackupView);
      await fireEvent.click(await designsButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith("Designs backup failed.", "error")
      );
    });
  });

  describe("backup both card", () => {
    async function bothButton() {
      const heading = screen.getByRole("heading", { name: "Backup Everything Now" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Everything Now" }));
      return card.getByRole("button", { name: "Backup Everything Now" }) as HTMLButtonElement;
    }

    it("disables the button when either destination is missing", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ designs_destination: "" }))
      );
      render(BackupView);
      const button = await bothButton();
      expect(button).toBeDisabled();
      expect(button).toHaveAttribute("title", "Set both backup destinations first");
    });

    it("enables the button when both destinations are saved", async () => {
      render(BackupView);
      expect(await bothButton()).toBeEnabled();
    });

    it("runs both and shows success toast when both succeed", async () => {
      render(BackupView);
      await fireEvent.click(await bothButton());
      await waitFor(() => expect(adapterMocks.runBothBackups).toHaveBeenCalledTimes(1));
      expect(toastMocks.addToast).toHaveBeenCalledWith(
        "Both backups completed successfully.",
        "success"
      );
    });

    it("shows error toast when one backup fails", async () => {
      adapterMocks.runBothBackups.mockResolvedValue(
        bothResult({
          database: {
            success: true,
            backup_path: "C:\\x.db",
            size_bytes: 1024,
            completed_at: "",
            error: "",
          },
          designs: {
            success: false,
            scanned: 0,
            copied: 0,
            updated: 0,
            unchanged: 0,
            archived: 0,
            total_bytes_copied: 0,
            completed_at: "",
            error: "Source folder missing",
          },
        })
      );
      render(BackupView);
      await fireEvent.click(await bothButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Backup results: database ok; designs failed (Source folder missing).",
          "error"
        )
      );
    });

    it("shows error toast when both fail", async () => {
      adapterMocks.runBothBackups.mockResolvedValue(
        bothResult({
          database: {
            success: false,
            backup_path: "",
            size_bytes: 0,
            completed_at: "",
            error: "Disk full",
          },
          designs: {
            success: false,
            scanned: 0,
            copied: 0,
            updated: 0,
            unchanged: 0,
            archived: 0,
            total_bytes_copied: 0,
            completed_at: "",
            error: "Source folder missing",
          },
        })
      );
      render(BackupView);
      await fireEvent.click(await bothButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Backup results: database failed (Disk full); designs failed (Source folder missing).",
          "error"
        )
      );
    });

    it("shows generic error toast when both results are null", async () => {
      adapterMocks.runBothBackups.mockResolvedValue({
        source: "mock",
        database: null,
        designs: null,
      });
      render(BackupView);
      await fireEvent.click(await bothButton());
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Backup results: database failed; designs failed.",
          "error"
        )
      );
    });
  });

  describe("validation guards", () => {
    it("blocks DB backup when no destination is saved", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ db_destination: "" }))
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Database Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Database Now" }));
      await fireEvent.click(card.getByRole("button", { name: "Backup Database Now" }));
      expect(adapterMocks.runDatabaseBackup).not.toHaveBeenCalled();
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "No database backup destination is configured. Please set one below and save destinations.",
          "error"
        )
      );
    });

    it("blocks designs backup when no destination is saved", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ designs_destination: "" }))
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Designs Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Run incremental backup" }));
      await fireEvent.click(card.getByRole("button", { name: "Run incremental backup" }));
      expect(adapterMocks.runDesignsBackup).not.toHaveBeenCalled();
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "No designs backup destination is configured. Please set one below and save destinations.",
          "error"
        )
      );
    });

    it("blocks both backups when either destination is missing", async () => {
      adapterMocks.getBackupViewModel.mockResolvedValue(
        backupResponse(backupModel({ designs_destination: "" }))
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Backup Everything Now" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Everything Now" }));
      await fireEvent.click(card.getByRole("button", { name: "Backup Everything Now" }));
      expect(adapterMocks.runBothBackups).not.toHaveBeenCalled();
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Both backup destinations must be configured before you can run both backups.",
          "error"
        )
      );
    });
  });

  describe("running state and concurrency", () => {
    it("shows 'Backing up database...' while running and restores after", async () => {
      let resolve!: (v: unknown) => void;
      adapterMocks.runDatabaseBackup.mockReturnValue(
        new Promise((r) => {
          resolve = r;
        })
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Database Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Database Now" }));
      await fireEvent.click(card.getByRole("button", { name: "Backup Database Now" }));
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Backing up database..." })).toBeInTheDocument()
      );
      resolve(dbResult());
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Backup Database Now" })).toBeInTheDocument()
      );
    });

    it("shows 'Running incremental backup...' while running", async () => {
      let resolve!: (v: unknown) => void;
      adapterMocks.runDesignsBackup.mockReturnValue(
        new Promise((r) => {
          resolve = r;
        })
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Designs Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Run incremental backup" }));
      await fireEvent.click(card.getByRole("button", { name: "Run incremental backup" }));
      await waitFor(() =>
        expect(
          screen.getByRole("button", { name: "Running incremental backup..." })
        ).toBeInTheDocument()
      );
      resolve(designsResult());
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Run incremental backup" })).toBeInTheDocument()
      );
    });

    it("shows 'Cancel Backup' while any backup is running", async () => {
      let resolve!: (v: unknown) => void;
      adapterMocks.runBothBackups.mockReturnValue(
        new Promise((r) => {
          resolve = r;
        })
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Backup Everything Now" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Everything Now" }));
      await fireEvent.click(card.getByRole("button", { name: "Backup Everything Now" }));
      await waitFor(() =>
        expect(
          screen.getByRole("button", { name: "Cancel Backup" })
        ).toBeInTheDocument()
      );

      // The per-card run buttons are replaced while a backup is running.
      expect(screen.getByRole("button", { name: "Backing up database..." })).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "Running incremental backup..." })
      ).toBeInTheDocument();

      resolve(bothResult());
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Backup Everything Now" })).toBeInTheDocument()
      );
    });

    it("prevents starting a second backup while one is running", async () => {
      let resolveDatabase!: (v: unknown) => void;
      adapterMocks.runDatabaseBackup.mockReturnValue(
        new Promise((r) => {
          resolveDatabase = r;
        })
      );
      render(BackupView);
      const heading = screen.getByRole("heading", { name: "Database Backup" });
      const card = within(element(heading.closest(".settings-card")));
      await waitFor(() => card.getByRole("button", { name: "Backup Database Now" }));
      await fireEvent.click(card.getByRole("button", { name: "Backup Database Now" }));

      // While the database backup is running, the other per-card run buttons
      // are replaced by disabled idle labels, and the "Backup Both" card shows
      // the "Cancel Backup" control instead of "Backup Everything Now".
      const designsBtn = screen.getByRole("button", {
        name: "Designs backup idle",
      }) as HTMLButtonElement;
      const bothBtn = screen.getByRole("button", {
        name: "Cancel Backup",
      }) as HTMLButtonElement;
      expect(designsBtn).toBeDisabled();
      expect(bothBtn).toBeEnabled();

      // The disabled designs button does nothing on click.
      await fireEvent.click(designsBtn);
      expect(adapterMocks.runDesignsBackup).not.toHaveBeenCalled();

      resolveDatabase(dbResult());
      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Backup Database Now" })).toBeInTheDocument()
      );
    });
  });

  describe("backup cancellation", () => {
    /**
     * Render the view and start a database backup whose promise we control.
     * Returns the resolve function for the pending backup promise.
     */
    async function startRunningBackup() {
      let resolve!: (v: unknown) => void;
      adapterMocks.runDatabaseBackup.mockReturnValue(
        new Promise((r) => {
          resolve = r;
        })
      );
      render(BackupView);
      const button = await waitFor(() => {
        const found = screen.getByRole("button", { name: "Backup Database Now" });
        return found as HTMLButtonElement;
      });
      await fireEvent.click(button);
      return resolve;
    }

    function cancelButton() {
      return screen.getByTestId("cancel-backup-button") as HTMLButtonElement;
    }

    it("requires confirmation before raising the cancel flag", async () => {
      const resolve = await startRunningBackup();
      await waitFor(() => expect(cancelButton()).toBeInTheDocument());
      await fireEvent.click(cancelButton());

      // Modal appears with the prompt and explanatory text; no backend call yet.
      const dialog = screen.getByRole("dialog");
      expect(
        within(dialog).getByText("Are you sure you want to cancel the backup?")
      ).toBeInTheDocument();

      // The explanatory notes exist both as <p> and inside the modal body
      // container, so assert on the body's normalized textContent (see
      // .clinerules: scope the query, don't fight multi-region text).
      const modalBody = dialog.querySelector(".cancel-backup-modal-body");
      const bodyText = (modalBody?.textContent ?? "").replace(/\s+/g, " ").trim();
      expect(bodyText).toContain(
        "If the database copy is currently running, any partially created database backup file will be aborted and removed."
      );
      // This backup is database-only (activeKind === "database"), so the
      // designs note must NOT appear — proving the conditional rendering.
      expect(bodyText).not.toContain("Any design files already copied");
      expect(adapterMocks.requestCancelBackup).not.toHaveBeenCalled();

      resolve(dbResult());
    });

    it("dismissing the dialog does not request cancellation", async () => {
      const resolve = await startRunningBackup();
      await waitFor(() => expect(cancelButton()).toBeInTheDocument());
      await fireEvent.click(cancelButton());
      await tick();

      await fireEvent.click(screen.getByRole("button", { name: "Continue backup" }));
      await tick();
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
      expect(adapterMocks.requestCancelBackup).not.toHaveBeenCalled();

      resolve(dbResult());
    });

    it("confirming raises the cancel flag and closes the dialog", async () => {
      const resolve = await startRunningBackup();
      await waitFor(() => expect(cancelButton()).toBeInTheDocument());
      await fireEvent.click(cancelButton());
      await tick();

      await fireEvent.click(screen.getByRole("button", { name: "Cancel backup" }));
      await waitFor(() => expect(adapterMocks.requestCancelBackup).toHaveBeenCalledTimes(1));
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

      resolve(dbResult());
    });

    it("shows a warning toast when the running database backup resolves cancelled", async () => {
      render(BackupView);
      adapterMocks.runDatabaseBackup.mockResolvedValue(
        dbResult({ success: false, cancelled: true })
      );
      await waitFor(() => screen.getByRole("button", { name: "Backup Database Now" }));
      await fireEvent.click(screen.getByRole("button", { name: "Backup Database Now" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Database backup cancelled. The partial backup file was removed.",
          "warning"
        )
      );
    });

    it("shows an info toast when the running designs backup resolves cancelled", async () => {
      render(BackupView);
      adapterMocks.runDesignsBackup.mockResolvedValue(
        designsResult({ success: false, cancelled: true })
      );
      await waitFor(() => screen.getByRole("button", { name: "Run incremental backup" }));
      await fireEvent.click(screen.getByRole("button", { name: "Run incremental backup" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Designs backup cancelled. Already copied files were kept.",
          "info"
        )
      );
    });

    it("shows a combined warning toast when both phases are cancelled", async () => {
      render(BackupView);
      adapterMocks.runBothBackups.mockResolvedValue(
        bothResult({
          database: { success: false, cancelled: true },
          designs: { success: false, cancelled: true },
        })
      );
      await waitFor(() => screen.getByRole("button", { name: "Backup Everything Now" }));
      await fireEvent.click(screen.getByRole("button", { name: "Backup Everything Now" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Backup cancelled. Partially created database backup files were removed; already copied design files were kept.",
          "warning"
        )
      );
    });

    it("shows a warning toast when only the database phase is cancelled", async () => {
      render(BackupView);
      adapterMocks.runBothBackups.mockResolvedValue(
        bothResult({
          database: { success: false, cancelled: true },
          designs: { success: true, cancelled: false },
        })
      );
      await waitFor(() => screen.getByRole("button", { name: "Backup Everything Now" }));
      await fireEvent.click(screen.getByRole("button", { name: "Backup Everything Now" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Database backup cancelled. The partial backup file was removed; design files already copied were kept.",
          "warning"
        )
      );
    });

    it("shows a warning toast when only the designs phase is cancelled", async () => {
      render(BackupView);
      adapterMocks.runBothBackups.mockResolvedValue(
        bothResult({
          database: { success: true, cancelled: false },
          designs: { success: false, cancelled: true },
        })
      );
      await waitFor(() => screen.getByRole("button", { name: "Backup Everything Now" }));
      await fireEvent.click(screen.getByRole("button", { name: "Backup Everything Now" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Designs backup cancelled. Already copied design files were kept.",
          "warning"
        )
      );
    });
  });

  describe("restore", () => {
    async function switchToRestoreTab() {
      await fireEvent.click(screen.getByRole("tab", { name: "Restore" }));
      await tick();
    }

    /** Confirm a destructive restore in the confirm modal. @param {string} label */
    async function confirmRestoreModal(label: string) {
      const dialog = screen.getByRole("dialog");
      await fireEvent.click(within(dialog).getByRole("button", { name: label }));
      await tick();
    }

    it("file picker defaults to the configured database backup folder", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\catalogue_2026-08-01.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalledWith(DB_DEST));

      await waitFor(() =>
        expect(screen.getByRole("button", { name: "Restore Database Now" })).toBeEnabled()
      );
    });

    it("restores the database and reports success", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\catalogue_2026-08-01.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());

      await fireEvent.click(screen.getByRole("button", { name: "Restore Database Now" }));
      await confirmRestoreModal("Restore database");
      await waitFor(() =>
        expect(adapterMocks.restoreDatabase).toHaveBeenCalledWith("C:\\backups\\catalogue_2026-08-01.db")
      );
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          expect.stringContaining("Database restored"),
          "success"
        )
      );
    });

    it("syncs designs from the configured backup folder", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      await fireEvent.click(screen.getByRole("button", { name: "Sync designs from backup" }));
      await waitFor(() =>
        expect(adapterMocks.restoreDesignsIncremental).toHaveBeenCalledWith({
          designsSourceDir: DESIGNS_DEST,
        })
      );
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          expect.stringContaining("Designs restored"),
          "success"
        )
      );
    });

    it("shows the unmatched-files prompt after restoring both and imports on request", async () => {
      adapterMocks.restoreBoth.mockResolvedValue({
        source: "rust",
        database: {
          success: true,
          restored_path: "C:\\x.db",
          rollback_copy_path: null,
          design_count: 5,
          schema_version_hint: null,
          previous_schema_version_hint: null,
          rolled_back: false,
          error: null,
        },
        designs: {
          success: true,
          scanned: 5,
          copied: 2,
          updated: 0,
          skipped: 3,
          total_bytes_copied: 0,
          error: null,
        },
        unmatched: {
          checked: 5,
          unmatched: 2,
          sample: ["MachineEmbroideryDesigns/a.pes", "MachineEmbroideryDesigns/b.pes"],
        },
      });
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\catalogue_2026-08-01.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());

      await fireEvent.click(screen.getByRole("button", { name: "Restore Both" }));
      await confirmRestoreModal("Restore both");
      await waitFor(() =>
        expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
      );

      await fireEvent.click(screen.getByRole("button", { name: /Import 2 file/ }));
      await waitFor(() => expect(adapterMocks.importUnmatchedDesignFiles).toHaveBeenCalled());
    });

    it("dismisses the unmatched-files prompt", async () => {
      adapterMocks.restoreBoth.mockResolvedValue({
        source: "rust",
        database: {
          success: true,
          restored_path: "C:\\x.db",
          rollback_copy_path: null,
          design_count: 5,
          schema_version_hint: null,
          previous_schema_version_hint: null,
          rolled_back: false,
          error: null,
        },
        designs: {
          success: true,
          scanned: 5,
          copied: 2,
          updated: 0,
          skipped: 3,
          total_bytes_copied: 0,
          error: null,
        },
        unmatched: { checked: 5, unmatched: 2, sample: [] },
      });
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\catalogue_2026-08-01.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());

      await fireEvent.click(screen.getByRole("button", { name: "Restore Both" }));
      await confirmRestoreModal("Restore both");
      await waitFor(() =>
        expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
      );

      await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
      await tick();
      expect(screen.queryByTestId("unmatched-files-prompt")).not.toBeInTheDocument();
    });

    it("shows no error and keeps restore disabled when the file picker is cancelled", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: null,
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());
      expect(toastMocks.addToast).not.toHaveBeenCalled();
      expect(screen.getByRole("button", { name: "Restore Database Now" })).toBeDisabled();
    });

    it("displays the selected database backup file path", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "H:\\Catalogue Backups\\Database\\EmbroideryCatalogue.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());
      const input = element(document.querySelector("#restore-db-file")) as HTMLInputElement;
      await waitFor(() =>
        expect(input.value).toBe("H:\\Catalogue Backups\\Database\\EmbroideryCatalogue.db")
      );
    });

    it("shows a schema-version warning when the restored database differs", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\catalogue_2026-08-01.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());

      adapterMocks.restoreDatabase.mockResolvedValue({
        source: "rust",
        success: true,
        restored_path: "C:\\x.db",
        rollback_copy_path: null,
        design_count: 5,
        schema_version_hint: 7,
        previous_schema_version_hint: 3,
        rolled_back: false,
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Restore Database Now" }));
      await confirmRestoreModal("Restore database");
      await waitFor(() => screen.getByText("Schema version changed"));
      expect(screen.getByText(/reports schema version 7/)).toBeInTheDocument();
    });

    it("shows a rollback banner and error toast when a corrupt backup is rejected", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.browseRestoreFile.mockResolvedValue({
        source: "rust",
        path: "C:\\backups\\corrupt_test.db",
        error: null,
      });
      await fireEvent.click(screen.getByRole("button", { name: "Choose file…" }));
      await waitFor(() => expect(adapterMocks.browseRestoreFile).toHaveBeenCalled());

      adapterMocks.restoreDatabase.mockResolvedValue({
        source: "rust",
        success: false,
        restored_path: "",
        rollback_copy_path: "C:\\x.pre-restore-1.db",
        design_count: 0,
        schema_version_hint: null,
        previous_schema_version_hint: null,
        rolled_back: true,
        error: "The backup file was corrupt; the database was restored from the safety snapshot.",
      });
      await fireEvent.click(screen.getByRole("button", { name: "Restore Database Now" }));
      await confirmRestoreModal("Restore database");
      await waitFor(() => screen.getByText("Restore rolled back"));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "The backup file was corrupt; the database was restored from the safety snapshot.",
          "error",
          true
        )
      );
    });

    it("shows an error toast when the designs backup folder is invalid", async () => {
      render(BackupView);
      await waitFor(() => expect(adapterMocks.getBackupViewModel).toHaveBeenCalled());
      await switchToRestoreTab();

      adapterMocks.restoreDesignsIncremental.mockResolvedValue({
        source: "rust",
        success: false,
        scanned: 0,
        copied: 0,
        updated: 0,
        skipped: 0,
        total_bytes_copied: 0,
        error: "Designs backup folder not found: Z:\\NonExistentFolder",
      });
      await fireEvent.click(screen.getByRole("button", { name: "Sync designs from backup" }));
      await waitFor(() =>
        expect(toastMocks.addToast).toHaveBeenCalledWith(
          "Designs backup folder not found: Z:\\NonExistentFolder",
          "error"
        )
      );
    });
  });
});
