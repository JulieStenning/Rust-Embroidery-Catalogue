import { describe, it, expect, vi, beforeEach } from "vitest";
import { runUnifiedBackfill, runStitchingBackfill } from "../commandAdapter";

// Mock the Tauri invoke used by the adapter so we can assert the exact wire payload.
const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("commandAdapter runUnifiedBackfill wire translation", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({});
  });

  it("maps an image-only run to nested actions with tagging disabled", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_tier2: false,
      run_tier3: false,
      run_images: true,
      image_redo: false,
      run_color_counts: false,
      commit_every: 50,
      batch_size: 33,
      workers: 2,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: null,
          stitching: null,
          images: { enabled: true, redo: false },
          color_counts: null,
          fingerprinting: null,
        },
        batch_size: 33,
        commit_every: 50,
        workers: 2,
      },
    });
  });

  it("maps tag_all to retag_all and includes the tier list when tagging is enabled", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_all",
      run_tier2: true,
      run_tier3: false,
      run_images: true,
      image_redo: true,
      run_color_counts: true,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: { action: "retag_all", tiers: [1, 2], enabled: true },
          stitching: null,
          images: { enabled: true, redo: true },
          color_counts: { enabled: true },
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });

  it("does not include the images action when run_images is false", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_tier2: true,
      run_tier3: false,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: { action: "tag_untagged", tiers: [1, 2], enabled: true },
          stitching: null,
          images: null,
          color_counts: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });
});

describe("commandAdapter runStitchingBackfill payload", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({});
  });

  it("maps snake_case options to the Rust command arguments", async () => {
    await runStitchingBackfill({
      clear_stitching_mode: "all",
      batch_size: 25,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_stitching_backfill", {
      clearStitchingMode: "all",
      batchSize: 25,
    });
  });
});