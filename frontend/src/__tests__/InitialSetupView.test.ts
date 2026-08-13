import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import InitialSetupView from "../lib/InitialSetupView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter so we control all setup flows.
// ---------------------------------------------------------------------------
const completeInitialSetupMock = vi.hoisted(() => vi.fn());
const getAppStatusMock = vi.hoisted(() => vi.fn());
const getConfiguredDataRootMock = vi.hoisted(() => vi.fn());
const setConfiguredDataRootMock = vi.hoisted(() => vi.fn());
const browseDataRootFolderMock = vi.hoisted(() => vi.fn());
const restartApplicationMock = vi.hoisted(() => vi.fn());
const getGoogleApiKeyMock = vi.hoisted(() => vi.fn());
const setGoogleApiKeyMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/api/commandAdapter", () => ({
  completeInitialSetup: completeInitialSetupMock,
  getAppStatus: getAppStatusMock,
  getConfiguredDataRoot: getConfiguredDataRootMock,
  setConfiguredDataRoot: setConfiguredDataRootMock,
  browseDataRootFolder: browseDataRootFolderMock,
  restartApplication: restartApplicationMock,
  getGoogleApiKey: getGoogleApiKeyMock,
  setGoogleApiKey: setGoogleApiKeyMock,
}));

vi.mock("../lib/stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

// Mock the embedded admin views so the wizard's step gating can be tested in
// isolation. Each stub exposes a data-testid so we can assert which step is
// currently visible.
vi.mock("../lib/views/AdminDesignersView.svelte", async () => {
  const { default: AdminDesignersView } = await import(
    "./__mocks__/AdminDesignersView.svelte"
  );
  return { default: AdminDesignersView };
});

vi.mock("../lib/views/AdminSourcesView.svelte", async () => {
  const { default: AdminSourcesView } = await import(
    "./__mocks__/AdminSourcesView.svelte"
  );
  return { default: AdminSourcesView };
});

vi.mock("../lib/views/AdminHoopsView.svelte", async () => {
  const { default: AdminHoopsView } = await import(
    "./__mocks__/AdminHoopsView.svelte"
  );
  return { default: AdminHoopsView };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Default non-installed mode so the data step is hidden. */
function mockDevMode() {
  getAppStatusMock.mockResolvedValue({
    source: "rust",
    status: {
      execution_mode: "dev",
      data_root: "D:/dev_data",
      embroidery_dir: "D:/dev_data/MachineEmbroideryDesigns",
      database_path: "D:/dev_data/Database/EmbroideryCatalogue.db",
      data_root_missing: false,
    },
  });
}

/** Installed mode with no persisted data root yet (first run, data-first). */
function mockInstalledNoConfig() {
  getAppStatusMock.mockResolvedValue({
    source: "rust",
    status: {
      execution_mode: "installed",
      data_root: "C:/Users/test/AppData/Roaming/EmbroideryCatalogue",
      embroidery_dir:
        "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/MachineEmbroideryDesigns",
      database_path:
        "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database/EmbroideryCatalogue.db",
      data_root_missing: false,
    },
  });
  getConfiguredDataRootMock.mockResolvedValue({ source: "rust", path: null });
  setConfiguredDataRootMock.mockResolvedValue({ source: "rust", persisted: true });
  browseDataRootFolderMock.mockResolvedValue({ source: "rust", path: null });
  restartApplicationMock.mockResolvedValue({ source: "rust", restarted: true });
}

/** Installed mode where the configured data root is no longer reachable
 *  (data-first recovery flow). */
function mockInstalledDataRootMissing() {
  getAppStatusMock.mockResolvedValue({
    source: "rust",
    status: {
      execution_mode: "installed",
      data_root: "C:/Users/test/AppData/Roaming/EmbroideryCatalogue",
      embroidery_dir:
        "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/MachineEmbroideryDesigns",
      database_path:
        "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database/EmbroideryCatalogue.db",
      data_root_missing: true,
    },
  });
  getConfiguredDataRootMock.mockResolvedValue({
    source: "rust",
    path: "G:/OldPortableData",
  });
  setConfiguredDataRootMock.mockResolvedValue({ source: "rust", persisted: true });
  browseDataRootFolderMock.mockResolvedValue({ source: "rust", path: null });
  restartApplicationMock.mockResolvedValue({ source: "rust", restarted: true });
}

/** Installed mode where a valid configured data root already exists
 *  (data step is skipped entirely — Designers → Sources → Hoops → API Key). */
function mockInstalledWithConfig() {
  getAppStatusMock.mockResolvedValue({
    source: "rust",
    status: {
      execution_mode: "installed",
      data_root: "D:/ExistingData",
      embroidery_dir:
        "D:/ExistingData/MachineEmbroideryDesigns",
      database_path:
        "D:/ExistingData/Database/EmbroideryCatalogue.db",
      data_root_missing: false,
    },
  });
  getConfiguredDataRootMock.mockResolvedValue({
    source: "rust",
    path: "D:/ExistingData",
  });
  setConfiguredDataRootMock.mockResolvedValue({ source: "rust", persisted: true });
  browseDataRootFolderMock.mockResolvedValue({ source: "rust", path: null });
  restartApplicationMock.mockResolvedValue({ source: "rust", restarted: true });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("InitialSetupView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    completeInitialSetupMock.mockResolvedValue(undefined);
    restartApplicationMock.mockResolvedValue({ source: "rust", restarted: true });
    getGoogleApiKeyMock.mockResolvedValue({ source: "rust", key: "" });
    setGoogleApiKeyMock.mockResolvedValue({ source: "rust", persisted: true });
    mockDevMode();
  });

  // -----------------------------------------------------------------------
  // Dev mode — no data step, Designers → Sources → Hoops → API Key
  // -----------------------------------------------------------------------

  it("renders the Designers step by default", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    expect(
      screen.getByText("Welcome to Embroidery Catalogue!")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Let's set up your catalogue")
    ).toBeInTheDocument();
    expect(screen.getByText("Step 1 of 4 — Designers")).toBeInTheDocument();
    expect(screen.getByText("What are Designers?")).toBeInTheDocument();
    expect(
      screen.getByText(/Designers are the digitizers or creators/)
    ).toBeInTheDocument();
    expect(screen.getByText(/Why do this now\?/)).toBeInTheDocument();
    expect(
      screen.getByText(/Setting up your frequent designers now/)
    ).toBeInTheDocument();
    expect(screen.getByText("Are they mandatory?")).toBeInTheDocument();
    expect(
      screen.getByText(/Not at all! This step is completely optional/)
    ).toBeInTheDocument();

    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-sources-view")).not.toBeInTheDocument();

    const button = screen.getByRole("button", { name: "Continue →" });
    expect(button).toBeEnabled();
    // Back is hidden on the first step.
    const backButton = screen.getByRole("button", { name: "← Back" });
    expect(backButton).toBeDisabled();
  });

  it("advances to the Sources step when Continue is clicked", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();

    expect(screen.getByText("Step 2 of 4 — Sources")).toBeInTheDocument();
    expect(screen.getByText("What are Sources?")).toBeInTheDocument();
    expect(
      screen.getByText(/Sources describe where your embroidery designs came from/)
    ).toBeInTheDocument();
    expect(screen.getByText(/Setting up your common sources now/)).toBeInTheDocument();
    expect(
      screen.getByText(/Not at all! This step is completely optional/)
    ).toBeInTheDocument();

    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-designers-view")).not.toBeInTheDocument();

    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("advances to the Hoops step from Sources", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();

    expect(screen.getByText("Step 3 of 4 — Hoops")).toBeInTheDocument();
    expect(screen.getByText("What are Hoops?")).toBeInTheDocument();
    expect(
      screen.getByText(/Hoops are the frames your embroidery machine uses/)
    ).toBeInTheDocument();
    expect(screen.getByTestId("admin-hoops-view")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-sources-view")).not.toBeInTheDocument();
  });

  it("advances to the Google API Key step from Hoops", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    expect(screen.getByText("Step 4 of 4 — Google API Key")).toBeInTheDocument();
    expect(screen.getByText("Would you like automated tagging?")).toBeInTheDocument();
    expect(screen.getByTestId("initial-setup-api-key-input")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Finish" })).toBeEnabled();
  });

  it("completes setup and invokes the callback on the final step", async () => {
    const onInitialSetupCompleted = vi.fn();
    render(InitialSetupView, { props: { onInitialSetupCompleted } });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(onInitialSetupCompleted).toHaveBeenCalledTimes(1);
    });
  });

  it("saves a non-blank API key when finishing", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    const apiInput = screen.getByTestId("initial-setup-api-key-input");
    await fireEvent.input(apiInput, { target: { value: "AIza-Setup-Key" } });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));

    await waitFor(() => {
      expect(setGoogleApiKeyMock).toHaveBeenCalledWith("AIza-Setup-Key");
    });
    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
  });

  it("does not save an empty API key when finishing", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));

    await waitFor(() => {
      expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    });
    expect(setGoogleApiKeyMock).not.toHaveBeenCalled();
  });

  it("saves the API key when navigating back from the API Key step", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    const apiInput = screen.getByTestId("initial-setup-api-key-input");
    await fireEvent.input(apiInput, { target: { value: "AIza-Back-Key" } });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "← Back" }));
    await tick();

    await waitFor(() => {
      expect(setGoogleApiKeyMock).toHaveBeenCalledWith("AIza-Back-Key");
    });
    expect(screen.getByText("Step 3 of 4 — Hoops")).toBeInTheDocument();
  });

  it("back navigates through the steps", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }
    expect(screen.getByText("Step 4 of 4 — Google API Key")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "← Back" }));
    await tick();
    expect(screen.getByText("Step 3 of 4 — Hoops")).toBeInTheDocument();
    expect(screen.getByTestId("admin-hoops-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "← Back" }));
    await tick();
    expect(screen.getByText("Step 2 of 4 — Sources")).toBeInTheDocument();
    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "← Back" }));
    await tick();
    expect(screen.getByText("Step 1 of 4 — Designers")).toBeInTheDocument();
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
    const backButton = screen.getByRole("button", { name: "← Back" });
    expect(backButton).toBeDisabled();

    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("loads an existing API key on mount", async () => {
    getGoogleApiKeyMock.mockResolvedValue({ source: "rust", key: "existing-key" });
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    await waitFor(() => {
      expect(screen.getByTestId("initial-setup-api-key-input")).toHaveValue("existing-key");
    });
  });

  it("disables the button and shows 'Saving…' while finishing setup", async () => {
    let resolveSetup: (() => void) | undefined;
    completeInitialSetupMock.mockReturnValue(
      new Promise<void>((resolve) => {
        resolveSetup = resolve;
      })
    );

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    const finishButton = screen.getByRole("button", { name: "Finish" });
    await fireEvent.click(finishButton);

    await waitFor(() => {
      expect(screen.getByText("Saving…")).toBeInTheDocument();
    });
    const savingButton = screen.getByRole("button", { name: "Saving…" });
    expect(savingButton).toBeDisabled();

    resolveSetup?.();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Finish" })).toBeEnabled();
    });
  });

  it("ignores a second continue while already finishing setup", async () => {
    completeInitialSetupMock.mockReturnValue(new Promise<void>(() => {}));

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }

    const finishButton = screen.getByRole("button", { name: "Finish" });
    await fireEvent.click(finishButton);
    await fireEvent.click(finishButton);

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    const savingButton = screen.getByRole("button", { name: "Saving…" });
    expect(savingButton).toBeDisabled();
  });

  it("shows an error message when setup completion fails", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    completeInitialSetupMock.mockRejectedValue(new Error("db locked"));

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    for (let i = 0; i < 3; i += 1) {
      await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
      await tick();
    }
    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));

    await waitFor(() => {
      expect(
        screen.getByText(
          "Failed to save setup status: Error: db locked. Please try again."
        )
      ).toBeInTheDocument();
    });
    expect(consoleError).toHaveBeenCalledWith(
      "initial setup failed:",
      expect.any(Error)
    );
    expect(screen.getByText("Finish")).toBeInTheDocument();
    expect(screen.queryByText("Saving…")).not.toBeInTheDocument();

    consoleError.mockRestore();
  });

  it("does not show an error message initially", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    expect(
      screen.queryByText(/Failed to save setup status/)
    ).not.toBeInTheDocument();
  });

  // -----------------------------------------------------------------------
  // Installed mode, first run — data root first, then restart
  // -----------------------------------------------------------------------

  it("shows a five-step wizard with Data Location first on first run", async () => {
    mockInstalledNoConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });
    expect(screen.getByTestId("data-root-input")).toBeInTheDocument();
    expect(screen.getByTestId("data-root-browse")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-designers-view")).not.toBeInTheDocument();
  });

  it("saves the data root then shows the restart confirmation on first run", async () => {
    mockInstalledNoConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });

    const input = screen.getByTestId("data-root-input");
    await fireEvent.input(input, {
      target: { value: "D:/EmbroideryCatalogue/Data" },
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    await waitFor(() => {
      expect(setConfiguredDataRootMock).toHaveBeenCalledWith(
        "D:/EmbroideryCatalogue/Data"
      );
    });
    expect(screen.getByTestId("restart-dialog")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-designers-view")).not.toBeInTheDocument();
    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("launches the restart when the user confirms", async () => {
    mockInstalledNoConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });

    const input = screen.getByTestId("data-root-input");
    await fireEvent.input(input, {
      target: { value: "D:/EmbroideryCatalogue/Data" },
    });
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    await waitFor(() => {
      expect(screen.getByTestId("restart-dialog")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("restart-now"));
    await waitFor(() => {
      expect(restartApplicationMock).toHaveBeenCalledTimes(1);
    });
    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("shows an error and closes the dialog when restart fails", async () => {
    mockInstalledNoConfig();
    restartApplicationMock.mockResolvedValue({
      source: "rust",
      restarted: false,
      error: "spawn failed",
    });
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });

    const input = screen.getByTestId("data-root-input");
    await fireEvent.input(input, {
      target: { value: "D:/EmbroideryCatalogue/Data" },
    });
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    await waitFor(() => {
      expect(screen.getByTestId("restart-dialog")).toBeInTheDocument();
    });
    await fireEvent.click(screen.getByTestId("restart-now"));

    await waitFor(() => {
      expect(
        screen.getByText(/Could not restart the application.*spawn failed/)
      ).toBeInTheDocument();
    });
    expect(restartApplicationMock).toHaveBeenCalledTimes(1);
  });

  it("validates that a data root is provided before continuing", async () => {
    mockInstalledNoConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });

    const finishButton = screen.getByRole("button", { name: "Continue →" });
    await fireEvent.click(finishButton);

    await tick();
    expect(
      screen.getByText("Please enter a data location or choose a folder.")
    ).toBeInTheDocument();
    expect(setConfiguredDataRootMock).not.toHaveBeenCalled();
    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("jumps to the data step and shows a recovery notice when the configured root is missing", async () => {
    mockInstalledDataRootMissing();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 5 — Data Location")).toBeInTheDocument();
    });
    expect(screen.getByTestId("data-root-missing-notice")).toBeInTheDocument();
    expect(screen.getByTestId("data-root-input")).toHaveValue("G:/OldPortableData");
  });

  // -----------------------------------------------------------------------
  // Installed mode with a valid configured root — skip the data step
  // -----------------------------------------------------------------------

  it("skips the data step when a valid configured data root already exists", async () => {
    mockInstalledWithConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    await waitFor(() => {
      expect(screen.getByText("Step 1 of 4 — Designers")).toBeInTheDocument();
    });
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
    expect(screen.queryByTestId("data-root-input")).not.toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();
    expect(screen.getByText("Step 2 of 4 — Sources")).toBeInTheDocument();
    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();
    expect(screen.getByText("Step 3 of 4 — Hoops")).toBeInTheDocument();
    expect(screen.getByTestId("admin-hoops-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();
    expect(screen.getByText("Step 4 of 4 — Google API Key")).toBeInTheDocument();
    expect(screen.getByTestId("initial-setup-api-key-input")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));
    expect(setConfiguredDataRootMock).not.toHaveBeenCalled();
    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
  });
});