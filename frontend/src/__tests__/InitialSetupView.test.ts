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

vi.mock("../lib/api/commandAdapter", () => ({
  completeInitialSetup: completeInitialSetupMock,
  getAppStatus: getAppStatusMock,
  getConfiguredDataRoot: getConfiguredDataRootMock,
  setConfiguredDataRoot: setConfiguredDataRootMock,
  browseDataRootFolder: browseDataRootFolderMock,
  restartApplication: restartApplicationMock,
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
 *  (data step is skipped entirely — Designers → Sources). */
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
    mockDevMode();
  });

  // -----------------------------------------------------------------------
  // Dev mode — no data step, Designers → Sources
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
    expect(screen.getByText("Step 1 of 2 — Designers")).toBeInTheDocument();
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
  });

  it("advances to the Sources step when Continue is clicked", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();

    expect(screen.getByText("Step 2 of 2 — Sources")).toBeInTheDocument();
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

  it("completes setup and invokes the callback on the final step", async () => {
    const onInitialSetupCompleted = vi.fn();
    render(InitialSetupView, { props: { onInitialSetupCompleted } });
    await tick();

    // Move from Designers to Sources.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    // Finish setup on the Sources step (the last step shows "Finish").
    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(onInitialSetupCompleted).toHaveBeenCalledTimes(1);
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

    // Move to the final step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    const finishButton = screen.getByRole("button", { name: "Finish" });
    await fireEvent.click(finishButton);

    await waitFor(() => {
      expect(screen.getByText("Saving…")).toBeInTheDocument();
    });
    const savingButton = screen.getByRole("button", { name: "Saving…" });
    expect(savingButton).toBeDisabled();

    // Resolve the pending setup promise and confirm we return to normal state.
    resolveSetup?.();
    await waitFor(() => {
      // Still on the final step, so the enabled button reads "Finish".
      expect(screen.getByRole("button", { name: "Finish" })).toBeEnabled();
    });
  });

  it("ignores a second continue while already finishing setup", async () => {
    // Keep the promise pending so `finishing` stays true between clicks. A
    // resolved promise would clear `finishing` before the second click fires.
    completeInitialSetupMock.mockReturnValue(new Promise<void>(() => {}));

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    // Move to the final step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    // Fire two clicks while the setup promise is still pending. The second
    // invocation must be ignored by the `finishing` guard so
    // completeInitialSetup is called exactly once.
    const finishButton = screen.getByRole("button", { name: "Finish" });
    await fireEvent.click(finishButton);
    await fireEvent.click(finishButton);

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    // The button should remain disabled and show "Saving…" because the setup
    // promise is still pending.
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

    // Move to the final step and attempt to finish.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
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

  it("shows a three-step wizard with Data Location first on first run", async () => {
    mockInstalledNoConfig();
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    await tick();

    // First step is Data Location (Step 1 of 3).
    await waitFor(() => {
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
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
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
    });

    // Enter a data root and continue.
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
    // The restart confirmation dialog appears; the app must NOT advance to
    // Designers until after restart.
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
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
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
    // Override AFTER the helper so the failure outcome actually takes effect.
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
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
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
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
    });

    // Leave the data root empty and attempt to continue.
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

    // Because the configured root is unreachable, the wizard goes straight to
    // the Data Location step (Step 1 of 3) and shows the recovery notice.
    await waitFor(() => {
      expect(screen.getByText("Step 1 of 3 — Data Location")).toBeInTheDocument();
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

    // No Data Location step — the wizard starts directly at Designers.
    await waitFor(() => {
      expect(screen.getByText("Step 1 of 2 — Designers")).toBeInTheDocument();
    });
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
    expect(screen.queryByTestId("data-root-input")).not.toBeInTheDocument();

    // And continues to Sources as the final step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await tick();
    expect(screen.getByText("Step 2 of 2 — Sources")).toBeInTheDocument();
    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();

    // Finishing does not touch the data root.
    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));
    expect(setConfiguredDataRootMock).not.toHaveBeenCalled();
    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
  });
});