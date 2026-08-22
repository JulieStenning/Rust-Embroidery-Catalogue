import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import App from "../App.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter — the new App.svelte calls checkInitialSetup() from
// ../lib/api/commandAdapter (no disclaimer gate).
// ---------------------------------------------------------------------------
const checkInitialSetupMock = vi.hoisted(() => vi.fn());
const getDatabaseStatusMock = vi.hoisted(() => vi.fn());
vi.mock("../lib/api/commandAdapter", () => ({
  checkInitialSetup: checkInitialSetupMock,
  getDatabaseStatus: getDatabaseStatusMock,
}));

// Mock the Tauri invoke bridge and event API so hasTauriInvoke()/listeners work.
const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const listenMock = vi.hoisted(() => vi.fn(() => Promise.resolve(() => {})));
vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

// Mock child views so App's gating logic can be tested in isolation.
vi.mock("../lib/InitialSetupView.svelte", async () => {
  const { default: InitialSetupView } = await import("./__mocks__/InitialSetupView.svelte");
  return { default: InitialSetupView };
});

vi.mock("../lib/MainView.svelte", async () => {
  const { default: MainView } = await import("./__mocks__/MainView.svelte");
  return { default: MainView };
});

vi.mock("../lib/components/ToastContainer.svelte", async () => {
  const { default: ToastContainer } = await import("./__mocks__/ToastContainer.svelte");
  return { default: ToastContainer };
});

vi.mock("../lib/DatabaseRecoveryView.svelte", async () => {
  const { default: DatabaseRecoveryView } = await import("./__mocks__/DatabaseRecoveryView.svelte");
  return { default: DatabaseRecoveryView };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Minimal shape used by the Tauri injected bridge. */
interface TauriInternalsBridge {
  invoke: unknown;
  [key: string]: unknown;
}

/** Window augmented with the optional Tauri bridge property. */
interface WindowWithTauriBridge extends Window {
  __TAURI_INTERNALS__?: TauriInternalsBridge;
}

const winWithTauriBridge = window as unknown as WindowWithTauriBridge;

/** Installs a fake Tauri internals bridge on window. */
function installTauriBridge() {
  winWithTauriBridge.__TAURI_INTERNALS__ = { invoke: invokeMock };
}

/** Removes the fake Tauri bridge, simulating plain browser dev mode. */
function removeTauriBridge() {
  delete winWithTauriBridge.__TAURI_INTERNALS__;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("App.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    removeTauriBridge();
    checkInitialSetupMock.mockReset();
    // Default: the configured database is healthy/connected so the setup gate
    // proceeds to checkInitialSetup.
    getDatabaseStatusMock.mockReset();
    getDatabaseStatusMock.mockResolvedValue({
      source: "rust",
      status: { status: "connected" },
    });
  });

  afterEach(() => {
    removeTauriBridge();
  });

  it("shows the loading splash while check_initial_setup is pending", async () => {
    installTauriBridge();
    checkInitialSetupMock.mockReturnValue(new Promise(() => {}));

    render(App);

    await waitFor(() => {
      expect(screen.getByText("Loading Embroidery Catalogue…")).toBeInTheDocument();
    });
    expect(checkInitialSetupMock).toHaveBeenCalled();
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
  });

  it("skips the setup gate in plain browser mode (no Tauri bridge)", async () => {
    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(checkInitialSetupMock).not.toHaveBeenCalled();
    expect(screen.queryByText("Loading Embroidery Catalogue…")).not.toBeInTheDocument();
  });

  it("renders the main app when initial setup is already completed", async () => {
    installTauriBridge();
    checkInitialSetupMock.mockResolvedValue(true);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(screen.queryByTestId("initial-setup-view")).not.toBeInTheDocument();
    expect(checkInitialSetupMock).toHaveBeenCalledTimes(1);
  });

  it("renders the database recovery view when the configured database is missing", async () => {
    installTauriBridge();
    getDatabaseStatusMock.mockResolvedValue({
      source: "rust",
      status: {
        status: "missing",
        configured_data_root: "D:/EmbroideryCatalogue",
        database_path: "D:/EmbroideryCatalogue/Database/EmbroideryCatalogue.db",
        embroidery_dir: "D:/EmbroideryCatalogue/MachineEmbroideryDesigns",
        data_root_missing: true,
      },
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("database-recovery-view")).toBeInTheDocument();
    });
    expect(checkInitialSetupMock).not.toHaveBeenCalled();
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("initial-setup-view")).not.toBeInTheDocument();
  });

  it("renders the setup wizard when setup is required", async () => {
    installTauriBridge();
    checkInitialSetupMock.mockResolvedValue(false);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("initial-setup-view")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("toast-container")).not.toBeInTheDocument();
  });

  it("renders the startup error state when check_initial_setup rejects", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    installTauriBridge();
    checkInitialSetupMock.mockRejectedValue(new Error("db locked"));

    render(App);

    await waitFor(() => {
      expect(screen.getByText("Startup Error")).toBeInTheDocument();
    });
    expect(screen.getByText("Could not verify setup status: Error: db locked")).toBeInTheDocument();
    expect(screen.getByText(/Try restarting the application\./)).toBeInTheDocument();
    expect(consoleError).toHaveBeenCalledWith("check_initial_setup failed:", expect.any(Error));
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("initial-setup-view")).not.toBeInTheDocument();

    consoleError.mockRestore();
  });

  it("transitions to the main app when the user accepts the initial setup", async () => {
    installTauriBridge();
    checkInitialSetupMock.mockResolvedValue(false);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("initial-setup-view")).toBeInTheDocument();
    });

    // The mock stub exposes a "complete" button that calls onInitialSetupCompleted.
    await fireEvent.click(screen.getByTestId("complete-initial-setup"));

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(screen.queryByTestId("initial-setup-view")).not.toBeInTheDocument();
  });
});
