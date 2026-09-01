import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  runUnifiedBackfill,
  runStitchingBackfill,
  countTaggingCandidates,
  browseTaggingFolder,
} from "../commandAdapter";

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
      run_vision: false,
      run_images: true,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
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
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 33,
        commit_every: 50,
        workers: 2,
      },
    });
  });

  it("maps tag_all to retag_all and includes the mode list when tagging is enabled", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_all",
      run_vision: true,
      run_images: true,
      image_redo: true,
      run_color_counts: true,
      run_hoop_dimensions: true,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: { action: "retag_all", modes: ["path_rule", "ai_vision"], enabled: true },
          stitching: null,
          images: { enabled: true, redo: true },
          color_counts: { enabled: true },
          hoop_dimensions: { enabled: true },
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
      run_vision: true,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: { action: "tag_untagged", modes: ["path_rule", "ai_vision"], enabled: true },
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });

  it("maps run_hoop_dimensions to the hoop_dimensions action when only it is enabled", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_vision: false,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: true,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: null,
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: { enabled: true },
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

describe("commandAdapter countTaggingCandidates payload", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({ total_count: 1200, unverified_count: 700, verified_count: 500 });
  });

  it("invokes count_tagging_candidates with the camelCase action key", async () => {
    const result = await countTaggingCandidates("retag_all_unverified");

    expect(invokeMock).toHaveBeenCalledWith("count_tagging_candidates", {
      action: "retag_all_unverified",
    });
    expect(result).toEqual({
      source: "rust",
      action: "retag_all_unverified",
      counts: { total_count: 1200, unverified_count: 700, verified_count: 500 },
    });
  });

  it("falls back to a mock response on error", async () => {
    invokeMock.mockRejectedValue(new Error("boom"));

    const result = await countTaggingCandidates("retag_all");

    expect(result).toEqual({
      source: "mock",
      action: "retag_all",
      counts: { total_count: 0, unverified_count: 0, verified_count: 0 },
      error: "Error: boom",
    });
  });

  it("passes folder_path and include_subfolders to the count command", async () => {
    await countTaggingCandidates("retag_all", "C:/library/Flowers", false);

    expect(invokeMock).toHaveBeenCalledWith("count_tagging_candidates", {
      action: "retag_all",
      folderPath: "C:/library/Flowers",
      includeSubfolders: false,
    });
  });
});

describe("commandAdapter browseTaggingFolder payload", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({ path: "C:/library/Flowers", relative_path: "Flowers" });
  });

  it("invokes browse_tagging_folder with the startDir key", async () => {
    const result = await browseTaggingFolder("C:/library");

    expect(invokeMock).toHaveBeenCalledWith("browse_tagging_folder", { startDir: "C:/library" });
    expect(result).toEqual({
      path: "C:/library/Flowers",
      relative_path: "Flowers",
      error: undefined,
    });
  });

  it("passes a null startDir when none is provided", async () => {
    await browseTaggingFolder();

    expect(invokeMock).toHaveBeenCalledWith("browse_tagging_folder", { startDir: null });
  });
});

describe("commandAdapter merge_mode forwarding", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({});
  });

  it("forwards an explicit merge_mode and mode list into the tagging action", async () => {
    await runUnifiedBackfill({
      action_mode: "retag_all_unverified",
      modes: ["ai_vision"],
      merge_mode: "add",
      run_vision: true,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: {
            action: "retag_all_unverified",
            modes: ["ai_vision"],
            merge_mode: "add",
            enabled: true,
          },
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });

  it("omits merge_mode when not provided, preserving legacy behaviour", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_all",
      run_vision: true,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: { action: "retag_all", modes: ["path_rule", "ai_vision"], enabled: true },
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });

  it("forwards exclude_verified into the tagging action when provided", async () => {
    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      modes: ["path_rule"],
      merge_mode: "add",
      exclude_verified: false,
      run_vision: false,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: {
            action: "tag_untagged",
            modes: ["path_rule"],
            merge_mode: "add",
            exclude_verified: false,
            enabled: true,
          },
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });
  it("forwards folder_path and include_subfolders into the tagging action", async () => {
    await runUnifiedBackfill({
      action_mode: "retag_all",
      modes: ["path_rule"],
      merge_mode: "add",
      exclude_verified: false,
      folder_path: "C:/library/Flowers",
      include_subfolders: false,
      run_vision: false,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      run_hoop_dimensions: false,
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: {
            action: "retag_all",
            modes: ["path_rule"],
            merge_mode: "add",
            exclude_verified: false,
            folder_path: "C:/library/Flowers",
            include_subfolders: false,
            enabled: true,
          },
          stitching: null,
          images: null,
          color_counts: null,
          hoop_dimensions: null,
          fingerprinting: null,
        },
        batch_size: 100,
        commit_every: 100,
        workers: 4,
      },
    });
  });

});
