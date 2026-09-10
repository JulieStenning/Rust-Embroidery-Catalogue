import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { get } from "svelte/store";
import UnmatchedFilesReconciler from "../UnmatchedFilesReconciler.svelte";
import {
  unmatchedFilesStore,
  resetUnmatchedFiles,
  setUnmatchedFilesDetected,
} from "../../stores/unmatchedFilesStore";

const adapterMocks = vi.hoisted(() => ({
  detectDesignFilesAbsentFromDatabase: vi.fn(),
  importUnmatchedDesignFiles: vi.fn(),
  requestCancelRestore: vi.fn(),
}));
vi.mock("../../api/commandAdapter", () => adapterMocks);

const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

describe("UnmatchedFilesReconciler", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetUnmatchedFiles();
    adapterMocks.detectDesignFilesAbsentFromDatabase.mockResolvedValue({
      source: "rust",
      checked: 0,
      unmatched: 0,
      sample: [],
    });
    adapterMocks.importUnmatchedDesignFiles.mockResolvedValue({
      source: "rust",
      detected: 0,
      imported: 0,
      flagged: 0,
      failed: 0,
      failed_samples: [],
    });
  });

  it("shows the prompt when the scan finds unmatched files", async () => {
    adapterMocks.detectDesignFilesAbsentFromDatabase.mockResolvedValue({
      source: "rust",
      checked: 12,
      unmatched: 3,
      sample: ["a.pes"],
    });
    render(UnmatchedFilesReconciler);

    await fireEvent.click(screen.getByTestId("scan-unmatched-button"));

    await waitFor(() =>
      expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: /Import 3 file/ })).toBeInTheDocument();
  });

  it("reports when the scan finds nothing", async () => {
    render(UnmatchedFilesReconciler);

    await fireEvent.click(screen.getByTestId("scan-unmatched-button"));

    await waitFor(() =>
      expect(toastMock.addToast).toHaveBeenCalledWith(
        expect.stringContaining("No unmatched design files found"),
        "success"
      )
    );
    expect(screen.queryByTestId("unmatched-files-prompt")).not.toBeInTheDocument();
  });

  it("surfaces a scan error", async () => {
    adapterMocks.detectDesignFilesAbsentFromDatabase.mockResolvedValue({
      source: "mock",
      error: "boom",
      checked: 0,
      unmatched: 0,
      sample: [],
    });
    render(UnmatchedFilesReconciler);

    await fireEvent.click(screen.getByTestId("scan-unmatched-button"));

    await waitFor(() =>
      expect(toastMock.addToast).toHaveBeenCalledWith(
        expect.stringContaining("Scan failed"),
        "error"
      )
    );
    expect(screen.queryByTestId("unmatched-files-prompt")).not.toBeInTheDocument();
  });

  it("imports the detected files then dismisses the prompt", async () => {
    setUnmatchedFilesDetected(2, 4, ["a.pes"]);
    adapterMocks.importUnmatchedDesignFiles.mockResolvedValue({
      source: "rust",
      detected: 2,
      imported: 2,
      flagged: 0,
      failed: 0,
      failed_samples: [],
    });
    render(UnmatchedFilesReconciler);

    await waitFor(() =>
      expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
    );
    await fireEvent.click(screen.getByRole("button", { name: /Import 2 file/ }));

    await waitFor(() =>
      expect(toastMock.addToast).toHaveBeenCalledWith(
        expect.stringContaining("Imported 2 unmatched file(s)."),
        "success"
      )
    );
    expect(get(unmatchedFilesStore).showPrompt).toBe(false);
    await waitFor(() =>
      expect(screen.queryByTestId("unmatched-files-prompt")).not.toBeInTheDocument()
    );
  });

  it("dismisses the prompt on Dismiss", async () => {
    setUnmatchedFilesDetected(1, 1, []);
    render(UnmatchedFilesReconciler);

    await waitFor(() =>
      expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
    );
    await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));

    expect(get(unmatchedFilesStore).showPrompt).toBe(false);
  });

  it("notes when the sample list is truncated", async () => {
    const sample = Array.from({ length: 20 }, (_, index) => `file-${index}.pes`);
    setUnmatchedFilesDetected(25, 30, sample);
    render(UnmatchedFilesReconciler);

    const note = await screen.findByTestId("unmatched-sample-note");
    expect(note).toHaveTextContent(
      "Showing the first 20 of 25 unmatched files — all of them will be imported."
    );
  });

  it("omits the truncation note when nothing is omitted", async () => {
    setUnmatchedFilesDetected(3, 12, ["a.pes", "b.pes", "c.pes"]);
    render(UnmatchedFilesReconciler);

    await waitFor(() =>
      expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
    );
    expect(screen.queryByTestId("unmatched-sample-note")).not.toBeInTheDocument();
  });

  it("cancels a running import and reports the partial result", async () => {
    setUnmatchedFilesDetected(2, 4, ["a.pes", "b.pes"]);
    let resolveImport!: (value: unknown) => void;
    adapterMocks.importUnmatchedDesignFiles.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveImport = resolve;
        })
    );
    adapterMocks.requestCancelRestore.mockResolvedValue({
      source: "rust",
      cancel_requested: true,
    });
    render(UnmatchedFilesReconciler);

    await waitFor(() =>
      expect(screen.getByTestId("unmatched-files-prompt")).toBeInTheDocument()
    );
    await fireEvent.click(screen.getByRole("button", { name: /Import 2 file/ }));

    const cancelButton = await screen.findByTestId("cancel-unmatched-import");
    await fireEvent.click(cancelButton);
    await waitFor(() => expect(adapterMocks.requestCancelRestore).toHaveBeenCalledTimes(1));
    expect(cancelButton).toBeDisabled();

    resolveImport({
      source: "rust",
      detected: 2,
      imported: 1,
      flagged: 0,
      failed: 0,
      failed_samples: [],
      cancelled: true,
    });
    await waitFor(() =>
      expect(toastMock.addToast).toHaveBeenCalledWith(
        expect.stringContaining("Import cancelled"),
        "warning"
      )
    );
  });
});
