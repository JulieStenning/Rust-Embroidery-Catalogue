import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import App from "../App.svelte";

// ---------------------------------------------------------------------------
// Mock the Tauri invoke bridge — App.svelte calls invoke("check_disclaimer").
// ---------------------------------------------------------------------------
const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

// Mock the child views so App's gating logic can be tested in isolation.
// Each stub is a real (tiny) Svelte component so it renders like a normal view.
vi.mock("../lib/DisclaimerView.svelte", async () => {
  const { default: DisclaimerView } = await import(
    "./__mocks__/DisclaimerView.svelte"
  );
  return { default: DisclaimerView };
});

vi.mock("../lib/MainView.svelte", async () => {
  const { default: MainView } = await import("./__mocks__/MainView.svelte");
  return { default: MainView };
});

vi.mock("../lib/components/ToastContainer.svelte", async () => {
  const { default: ToastContainer } = await import(
    "./__mocks__/ToastContainer.svelte"
  );
  return { default: ToastContainer };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Installs a fake Tauri internals bridge on window. */
function installTauriBridge() {
  (window as any).__TAURI_INTERNALS__ = { invoke: invokeMock };
}

/** Removes the fake Tauri bridge, simulating plain browser dev mode. */
function removeTauriBridge() {
  delete (window as any).__TAURI_INTERNALS__;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("App.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    removeTauriBridge();
  });

  afterEach(() => {
    removeTauriBridge();
  });

  it("shows the loading splash while check_disclaimer is pending", async () => {
    installTauriBridge();
    invokeMock.mockReturnValue(new Promise(() => {}));

    render(App);

    await waitFor(() => {
      expect(
        screen.getByText("Loading Embroidery Catalogue…")
      ).toBeInTheDocument();
    });
    expect(invokeMock).toHaveBeenCalledWith("check_disclaimer");
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
  });

  it("skips the disclaimer gate in plain browser mode (no Tauri bridge)", async () => {
    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(screen.queryByTestId("disclaimer-view")).not.toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalled();
    expect(
      screen.queryByText("Loading Embroidery Catalogue…")
    ).not.toBeInTheDocument();
  });

  it("renders the main app when the disclaimer has already been accepted", async () => {
    installTauriBridge();
    invokeMock.mockResolvedValue(true);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(screen.queryByTestId("disclaimer-view")).not.toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("check_disclaimer");
  });

  it("renders the disclaimer view when acceptance is still required", async () => {
    installTauriBridge();
    invokeMock.mockResolvedValue(false);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("disclaimer-view")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("toast-container")).not.toBeInTheDocument();
  });

  it("renders the startup error state when check_disclaimer rejects", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    installTauriBridge();
    invokeMock.mockRejectedValue(new Error("db locked"));

    render(App);

    await waitFor(() => {
      expect(screen.getByText("Startup Error")).toBeInTheDocument();
    });
    expect(
      screen.getByText("Could not verify disclaimer status: Error: db locked")
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Try restarting the application\./)
    ).toBeInTheDocument();
    expect(consoleError).toHaveBeenCalledWith(
      "check_disclaimer failed:",
      expect.any(Error)
    );
    expect(screen.queryByTestId("main-view")).not.toBeInTheDocument();
    expect(screen.queryByTestId("disclaimer-view")).not.toBeInTheDocument();

    consoleError.mockRestore();
  });

  it("transitions to the main app when the user accepts the disclaimer", async () => {
    installTauriBridge();
    invokeMock.mockResolvedValue(false);

    render(App);

    await waitFor(() => {
      expect(screen.getByTestId("disclaimer-view")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByTestId("accept-disclaimer"));

    await waitFor(() => {
      expect(screen.getByTestId("main-view")).toBeInTheDocument();
    });
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
    expect(screen.queryByTestId("disclaimer-view")).not.toBeInTheDocument();
  });
});