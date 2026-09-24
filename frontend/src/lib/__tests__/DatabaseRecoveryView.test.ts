// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import DatabaseRecoveryView from "../DatabaseRecoveryView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter
// ---------------------------------------------------------------------------
const getDatabaseStatusMock = vi.hoisted(() => vi.fn());
const detectRelocatedDataRootMock = vi.hoisted(() => vi.fn());
const validateDatabasePathMock = vi.hoisted(() => vi.fn());
const browseSettingsDataRootMock = vi.hoisted(() => vi.fn());
const seedDatabaseToDataRootMock = vi.hoisted(() => vi.fn());
const setConfiguredDataRootMock = vi.hoisted(() => vi.fn());
const restartApplicationMock = vi.hoisted(() => vi.fn());

vi.mock("../api/commandAdapter", () => ({
  getDatabaseStatus: getDatabaseStatusMock,
  detectRelocatedDataRoot: detectRelocatedDataRootMock,
  validateDatabasePath: validateDatabasePathMock,
  browseSettingsDataRoot: browseSettingsDataRootMock,
  seedDatabaseToDataRoot: seedDatabaseToDataRootMock,
  setConfiguredDataRoot: setConfiguredDataRootMock,
  restartApplication: restartApplicationMock,
}));

describe("DatabaseRecoveryView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getDatabaseStatusMock.mockResolvedValue({
      source: "rust",
      status: {
        status: "missing",
        configured_data_root: "F:\\OldCatalogue",
        database_path: "F:\\OldCatalogue\\Database\\EmbroideryCatalogue.db",
        embroidery_dir: "F:\\OldCatalogue\\MachineEmbroideryDesigns",
        data_root_missing: true,
      },
    });
    detectRelocatedDataRootMock.mockResolvedValue({
      source: "rust",
      detected: null,
    });
    seedDatabaseToDataRootMock.mockResolvedValue({
      source: "rust",
      persisted: true,
    });
    setConfiguredDataRootMock.mockResolvedValue({
      source: "rust",
      persisted: true,
    });
    validateDatabasePathMock.mockResolvedValue({
      source: "rust",
      validation: {
        valid: true,
        data_root: "D:\\FoundCatalogue",
        database_path: "D:\\FoundCatalogue\\Database\\EmbroideryCatalogue.db",
        embroidery_dir: "D:\\FoundCatalogue\\MachineEmbroideryDesigns",
        embroidery_dir_exists: true,
      },
    });
    browseSettingsDataRootMock.mockResolvedValue({
      source: "rust",
      path: null,
      error: null,
    });
    restartApplicationMock.mockResolvedValue({
      source: "rust",
      restarted: true,
    });
  });

  async function renderAndMount() {
    render(DatabaseRecoveryView);
    await waitFor(() => {
      expect(screen.queryByText("Checking for your catalogue…")).not.toBeInTheDocument();
    });
  }

  it("renders recovery view and displays the missing configured root", async () => {
    await renderAndMount();

    expect(screen.getByText("Your catalogue database could not be found")).toBeInTheDocument();
    expect(screen.getByText("F:\\OldCatalogue")).toBeInTheDocument();
    expect(screen.getByTestId("recovery-browse")).toBeInTheDocument();
    expect(screen.getByTestId("recovery-create-new")).toBeInTheDocument();
  });

  it("opens create new catalogue modal with configuredRoot prefilled", async () => {
    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    expect(
      screen.getByRole("dialog", { name: "Create new catalogue confirmation" })
    ).toBeInTheDocument();
    const input = screen.getByTestId("recovery-new-location-input") as HTMLInputElement;
    expect(input.value).toBe("F:\\OldCatalogue");
    expect(screen.getByTestId("recovery-new-location-browse")).toBeInTheDocument();
    expect(screen.getByTestId("recovery-create-confirm")).toBeInTheDocument();
  });

  it("allows browsing for a new location when creating a catalogue", async () => {
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: "D:\\NewCatalogueFolder",
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-new-location-browse"));
    await tick();

    expect(browseSettingsDataRootMock).toHaveBeenCalledWith("F:\\OldCatalogue");
    const input = screen.getByTestId("recovery-new-location-input") as HTMLInputElement;
    expect(input.value).toBe("D:\\NewCatalogueFolder");
  });

  it("creates a fresh catalogue at a custom location and prompts for restart", async () => {
    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    const input = screen.getByTestId("recovery-new-location-input");
    await fireEvent.input(input, {
      target: { value: "E:\\CustomCatalogueLocation" },
    });
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-create-confirm"));
    await tick();

    expect(seedDatabaseToDataRootMock).toHaveBeenCalledWith("E:\\CustomCatalogueLocation", false);
    expect(setConfiguredDataRootMock).toHaveBeenCalledWith("E:\\CustomCatalogueLocation");

    expect(screen.getByRole("dialog", { name: "Restart required" })).toBeInTheDocument();
    expect(screen.getByTestId("recovery-restart-now")).toBeInTheDocument();
  });

  it("shows an error when seedDatabaseToDataRoot fails", async () => {
    seedDatabaseToDataRootMock.mockResolvedValueOnce({
      source: "rust",
      persisted: false,
      error: "A database already exists at this location.",
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-create-confirm"));
    await tick();

    expect(seedDatabaseToDataRootMock).toHaveBeenCalledWith("F:\\OldCatalogue", false);
    expect(setConfiguredDataRootMock).not.toHaveBeenCalled();

    const errorBox = screen.getByTestId("recovery-create-error");
    expect(errorBox).toHaveTextContent("A database already exists at this location.");
    // Dialog stays open so user can pick another folder
    expect(
      screen.getByRole("dialog", { name: "Create new catalogue confirmation" })
    ).toBeInTheDocument();
  });

  it("cancels creation when cancel button is clicked", async () => {
    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    expect(
      screen.getByRole("dialog", { name: "Create new catalogue confirmation" })
    ).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("recovery-create-cancel"));
    await tick();

    expect(
      screen.queryByRole("dialog", {
        name: "Create new catalogue confirmation",
      })
    ).not.toBeInTheDocument();
  });

  it("re-connects to a relocated root when detected on another drive", async () => {
    detectRelocatedDataRootMock.mockResolvedValue({
      source: "rust",
      detected: {
        data_root: "E:\\OldCatalogue",
        relative_subpath: "OldCatalogue",
      },
    });

    await renderAndMount();

    expect(screen.getByTestId("recovery-reconnect")).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("recovery-reconnect"));
    await tick();

    expect(validateDatabasePathMock).toHaveBeenCalledWith("E:\\OldCatalogue");
    expect(setConfiguredDataRootMock).toHaveBeenCalledWith("D:\\FoundCatalogue");
    expect(screen.getByRole("dialog", { name: "Restart required" })).toBeInTheDocument();
  });

  it("handles browse flow when a valid folder is selected", async () => {
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: "D:\\FoundCatalogue",
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));
    await tick();

    expect(browseSettingsDataRootMock).toHaveBeenCalledWith("F:\\OldCatalogue");
    expect(validateDatabasePathMock).toHaveBeenCalledWith("D:\\FoundCatalogue");
    expect(setConfiguredDataRootMock).toHaveBeenCalledWith("D:\\FoundCatalogue");
    expect(screen.getByRole("dialog", { name: "Restart required" })).toBeInTheDocument();
  });

  it("does nothing when browse is cancelled without selecting a path", async () => {
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: null,
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));
    await tick();

    expect(validateDatabasePathMock).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog", { name: "Restart required" })).not.toBeInTheDocument();
  });

  it("shows an error when browse dialog fails", async () => {
    browseSettingsDataRootMock.mockRejectedValueOnce(new Error("Native picker error"));

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));
    await tick();

    const errorBox = screen.getByTestId("recovery-error");
    expect(errorBox).toHaveTextContent("Error: Native picker error");
  });

  it("shows validation message when selected database path is invalid", async () => {
    validateDatabasePathMock.mockResolvedValueOnce({
      source: "rust",
      validation: {
        valid: false,
        data_root: "D:\\EmptyFolder",
        database_path: "D:\\EmptyFolder\\Database\\EmbroideryCatalogue.db",
        embroidery_dir: "D:\\EmptyFolder\\MachineEmbroideryDesigns",
        embroidery_dir_exists: false,
        error: "No catalogue database found at this location.",
      },
    });
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: "D:\\EmptyFolder",
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));
    await tick();

    const validationBox = screen.getByTestId("recovery-validation");
    expect(validationBox).toHaveTextContent("No catalogue database found at this location.");
    expect(setConfiguredDataRootMock).not.toHaveBeenCalled();
  });

  it("shows warning when database exists but embroidery directory is missing", async () => {
    validateDatabasePathMock.mockResolvedValueOnce({
      source: "rust",
      validation: {
        valid: true,
        data_root: "D:\\MissingEmbroideryDir",
        database_path: "D:\\MissingEmbroideryDir\\Database\\EmbroideryCatalogue.db",
        embroidery_dir: "D:\\MissingEmbroideryDir\\MachineEmbroideryDesigns",
        embroidery_dir_exists: false,
      },
    });
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: "D:\\MissingEmbroideryDir",
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));
    await tick();

    const validationBox = screen.getByTestId("recovery-validation");
    expect(validationBox).toHaveTextContent(
      "Database found, but the MachineEmbroideryDesigns folder is missing"
    );
    expect(setConfiguredDataRootMock).toHaveBeenCalledWith("D:\\MissingEmbroideryDir");
    expect(screen.getByRole("dialog", { name: "Restart required" })).toBeInTheDocument();
  });

  it("shows error when setConfiguredDataRoot fails during validation", async () => {
    setConfiguredDataRootMock.mockResolvedValueOnce({
      source: "rust",
      persisted: false,
      error: "Permission denied writing config.json",
    });
    browseSettingsDataRootMock.mockResolvedValueOnce({
      source: "rust",
      path: "D:\\FoundCatalogue",
      error: null,
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-browse"));

    await waitFor(() => {
      const errorBox = screen.getByTestId("recovery-error");
      expect(errorBox).toHaveTextContent("Permission denied writing config.json");
    });
  });

  it("executes restartApplication when restart button is clicked", async () => {
    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();
    await fireEvent.click(screen.getByTestId("recovery-create-confirm"));
    await tick();

    expect(screen.getByRole("dialog", { name: "Restart required" })).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("recovery-restart-now"));
    await tick();

    expect(restartApplicationMock).toHaveBeenCalledTimes(1);
  });

  it("shows error message when restartApplication fails", async () => {
    restartApplicationMock.mockResolvedValueOnce({
      source: "rust",
      restarted: false,
      error: "Process spawn failed.",
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();
    await fireEvent.click(screen.getByTestId("recovery-create-confirm"));
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-restart-now"));
    await tick();

    expect(screen.queryByRole("dialog", { name: "Restart required" })).not.toBeInTheDocument();
    const errorBox = screen.getByTestId("recovery-error");
    expect(errorBox).toHaveTextContent("Process spawn failed.");
  });

  it("shows error if create location browse throws", async () => {
    browseSettingsDataRootMock.mockRejectedValueOnce(new Error("Picker access denied"));

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-new-location-browse"));
    await tick();

    const errorBox = screen.getByTestId("recovery-create-error");
    expect(errorBox).toHaveTextContent("Error: Picker access denied");
  });

  it("shows error if setConfiguredDataRoot fails after successful seed", async () => {
    setConfiguredDataRootMock.mockResolvedValueOnce({
      source: "rust",
      persisted: false,
      error: "Could not write config file",
    });

    await renderAndMount();

    await fireEvent.click(screen.getByTestId("recovery-create-new"));
    await tick();

    await fireEvent.click(screen.getByTestId("recovery-create-confirm"));
    await tick();

    const errorBox = screen.getByTestId("recovery-create-error");
    expect(errorBox).toHaveTextContent("Could not write config file");
  });
});

