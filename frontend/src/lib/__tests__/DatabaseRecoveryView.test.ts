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
});
