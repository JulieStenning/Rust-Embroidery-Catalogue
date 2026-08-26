import "@testing-library/jest-dom/vitest";
import { describe, it, expect, afterEach } from "vitest";
import { render, screen, cleanup, within } from "@testing-library/svelte";
import RestoreProgressPanel from "../RestoreProgressPanel.svelte";
import { restoreProgressStore, resetRestoreProgress } from "../../stores/restoreProgressStore";

describe("RestoreProgressPanel", () => {
  afterEach(() => {
    resetRestoreProgress();
    cleanup();
  });

  it("renders nothing when no restore is active", () => {
    restoreProgressStore.set({
      active: false,
      phase: "",
      dbStatus: "",
      scanned: 0,
      copied: 0,
      skipped: 0,
      totalBytes: 0,
      percent: 0,
      error: null,
    });
    render(RestoreProgressPanel);
    expect(screen.queryByTestId("restore-progress-panel")).not.toBeInTheDocument();
  });

  it("renders the live copy/skip metrics and percent during a designs sync", () => {
    restoreProgressStore.set({
      active: true,
      phase: "designs",
      dbStatus: "syncing",
      scanned: 10,
      copied: 3,
      skipped: 7,
      totalBytes: 0,
      percent: 0.4,
      error: null,
    });
    render(RestoreProgressPanel);
    const panel = screen.getByTestId("restore-progress-panel");
    expect(within(panel).getByText("Sync design files")).toBeInTheDocument();
    expect(within(panel).getByText("Syncing files…")).toBeInTheDocument();
    expect(within(panel).getByText(/Copied 3/)).toBeInTheDocument();
    expect(within(panel).getByText(/Skipped 7/)).toBeInTheDocument();
    const bar = panel.querySelector(".bg-indigo-600") as HTMLElement | null;
    expect(bar?.style.width).toBe("40%");
  });

  it("maps the db-swap phase and rolled-back status labels", () => {
    restoreProgressStore.set({
      active: true,
      phase: "db-swap",
      dbStatus: "rolled-back",
      scanned: 0,
      copied: 0,
      skipped: 0,
      totalBytes: 0,
      percent: 1,
      error: null,
    });
    render(RestoreProgressPanel);
    expect(screen.getByText("Database swap")).toBeInTheDocument();
    expect(screen.getByText("Rolled back to previous database")).toBeInTheDocument();
  });

  it("renders the error line when the restore fails", () => {
    restoreProgressStore.set({
      active: true,
      phase: "db-swap",
      dbStatus: "starting",
      scanned: 0,
      copied: 0,
      skipped: 0,
      totalBytes: 0,
      percent: 0,
      error: "Restore failed",
    });
    render(RestoreProgressPanel);
    expect(screen.getByText("Restore failed")).toBeInTheDocument();
  });
});
