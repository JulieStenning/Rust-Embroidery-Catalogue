import "@testing-library/jest-dom/vitest";
import { describe, it, expect, afterEach, vi } from "vitest";
import { render, screen, cleanup, within, fireEvent, waitFor } from "@testing-library/svelte";
import RestoreProgressPanel from "../RestoreProgressPanel.svelte";
import {
  restoreProgressStore,
  resetRestoreProgress,
  idleRestoreProgress,
  type RestoreProgressState,
} from "../../stores/restoreProgressStore";

const adapterMocks = vi.hoisted(() => ({ requestCancelRestore: vi.fn() }));
vi.mock("../../api/commandAdapter", () => adapterMocks);

/** Build a store state with sensible defaults for the fields under test. */
function progressState(overrides: Partial<RestoreProgressState> = {}): RestoreProgressState {
  return { ...idleRestoreProgress, active: true, ...overrides };
}

describe("RestoreProgressPanel", () => {
  afterEach(() => {
    resetRestoreProgress();
    cleanup();
  });

  it("renders nothing when no restore is active", () => {
    restoreProgressStore.set({ ...idleRestoreProgress });
    render(RestoreProgressPanel);
    expect(screen.queryByTestId("restore-progress-panel")).not.toBeInTheDocument();
  });

  it("shows a designs-sync summary with live copy/skip metrics", () => {
    restoreProgressStore.set(
      progressState({
        scope: "designs",
        phase: "designs",
        status: "running",
        scanned: 10,
        copied: 3,
        skipped: 7,
        percent: 0.4,
      })
    );
    render(RestoreProgressPanel);
    const panel = screen.getByTestId("restore-progress-panel");
    expect(within(panel).getByText("Restore in progress")).toBeInTheDocument();
    expect(within(panel).getByText(/Syncing design files from backup/)).toBeInTheDocument();
    expect(within(panel).getByText(/Copied 3/)).toBeInTheDocument();
    expect(within(panel).getByText(/Skipped 7/)).toBeInTheDocument();
    const bar = panel.querySelector(".bg-indigo-600") as HTMLElement | null;
    expect(bar?.style.width).toBe("40%");
  });

  it("hides file metrics for a database-only restore", () => {
    restoreProgressStore.set(
      progressState({ scope: "database", phase: "database", status: "running" })
    );
    render(RestoreProgressPanel);
    const panel = screen.getByTestId("restore-progress-panel");
    expect(within(panel).getByText(/Restoring database/)).toBeInTheDocument();
    expect(within(panel).queryByText(/Copied/)).not.toBeInTheDocument();
  });

  it("shows the three-step list only while a combined restore is live", () => {
    restoreProgressStore.set(progressState({ scope: "both", phase: "designs", status: "running" }));
    render(RestoreProgressPanel);
    const steps = screen.getByTestId("restore-steps");
    expect(within(steps).getByText("Database")).toBeInTheDocument();
    expect(within(steps).getByText("Design files")).toBeInTheDocument();
    expect(within(steps).getByText("Reconcile")).toBeInTheDocument();
  });

  it("never claims a database restore when a designs-only sync completes", () => {
    restoreProgressStore.set(
      progressState({
        scope: "designs",
        phase: "completed",
        status: "done",
        terminal: true,
        copied: 5,
        skipped: 2,
        percent: 1,
      })
    );
    render(RestoreProgressPanel);
    expect(screen.getByText("Design sync complete")).toBeInTheDocument();
    expect(screen.queryByText("Database restored")).not.toBeInTheDocument();
    expect(screen.getByTestId("restore-progress-close")).toBeInTheDocument();
  });

  it("shows the rolled-back terminal title, the error line, and a working Close button", async () => {
    const onclose = vi.fn();
    restoreProgressStore.set(
      progressState({
        scope: "database",
        phase: "completed",
        status: "rolled-back",
        terminal: true,
        error: "Restore failed",
      })
    );
    render(RestoreProgressPanel, { props: { onclose } });
    expect(screen.getByText("Database restore rolled back")).toBeInTheDocument();
    expect(screen.getByText("Restore failed")).toBeInTheDocument();

    await fireEvent.click(screen.getByTestId("restore-progress-close"));
    expect(onclose).toHaveBeenCalledTimes(1);
  });

  it("shows a working Cancel button during a live designs sync", async () => {
    adapterMocks.requestCancelRestore.mockResolvedValue({
      source: "rust",
      cancel_requested: true,
    });
    restoreProgressStore.set(
      progressState({ scope: "designs", phase: "designs", status: "running" })
    );
    render(RestoreProgressPanel);

    const cancel = await screen.findByTestId("cancel-restore-button");
    await fireEvent.click(cancel);
    await waitFor(() => expect(adapterMocks.requestCancelRestore).toHaveBeenCalledTimes(1));
  });

  it("disables Cancel during the database phase of a combined restore", () => {
    restoreProgressStore.set(
      progressState({ scope: "both", phase: "database", status: "running" })
    );
    render(RestoreProgressPanel);
    expect(screen.getByTestId("cancel-restore-button")).toBeDisabled();
  });

  it("hides Cancel for an unmatched-import run (the reconciler owns it)", () => {
    restoreProgressStore.set(
      progressState({ scope: "import-unmatched", phase: "import", status: "running" })
    );
    render(RestoreProgressPanel);
    expect(screen.queryByTestId("cancel-restore-button")).not.toBeInTheDocument();
  });

  it("shows a scope-aware cancelled terminal title", () => {
    restoreProgressStore.set(
      progressState({
        scope: "designs",
        phase: "completed",
        status: "cancelled",
        terminal: true,
      })
    );
    render(RestoreProgressPanel);
    expect(screen.getByText("Design sync cancelled")).toBeInTheDocument();
  });
});
