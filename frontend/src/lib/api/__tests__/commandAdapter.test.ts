import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  addDesignToProject,
  browseBackupFolder,
  browseImportFolder,
  browseOrphanPath,
  browseSettingsDataRoot,
  bulkAddDesignsToProject,
  bulkDeleteDesigns,
  bulkSetTagsForDesigns,
  bulkVerifyDesigns,
  cancelCatalogueStorageMigration,
  checkInitialSetup,
  compactDatabase,
  completeInitialSetup,
  configureFreshDataRoot,
  createDesigner,
  createHoop,
  createProject,
  createSource,
  createTag,
  deleteAllOrphans,
  deleteDesigner,
  deleteHoop,
  deleteOrphans,
  deleteProject,
  deleteSource,
  deleteTag,
  getAboutDocument,
  getAboutDocuments,
  getAppStatus,
  getBackfillLogEntries,
  getBackupViewModel,
  getBrowseDesignPreviews,
  getBrowseDesigns,
  getBrowseProjects,
  getBrowseTags,
  getDbStats,
  getDesignDetail,
  getDesignImageDataUrl,
  getGoogleApiKey,
  getOrphansPage,
  getProjectDetail,
  getProjectPrintView,
  getProjectsList,
  getSettingsViewModel,
  getTaggingActionsViewModel,
  listDesigners,
  listHoops,
  listSources,
  listTags,
  openDesignInEditor,
  openDesignInExplorer,
  precheckImportWire,
  previewImportFromRoot,
  previewImportFromRoots,
  removeDesignFromProject,
  removeDesignFromProjectDetail,
  removeDesignTag,
  renderDesign3dPreview,
  reparseDesignFile,
  requestStopBulkImport,
  runBothBackups,
  runDatabaseBackup,
  runDesignsBackup,
  runPrecheckAction,
  runStitchingBackfill,
  runUnifiedBackfill,
  saveBackupSettings,
  saveImportLastBrowseFolder,
  saveSettings,
  scanOrphans,
  startCatalogueStorageMigration,
  setDesignRating,
  setGoogleApiKey,
  setDesignStitched,
  setDesignTags,
  setDesignVerification,
  setTagGroup,
  stopUnifiedBackfill,
  updateDesignMetadata,
  updateDesigner,
  updateHoop,
  updateProject,
  updateSource,
  updateTag,
} from "../commandAdapter";

// Mock the Tauri invoke used by the adapter so we can assert the exact wire payload.
const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));


// Make the mocked invoke reject once so the adapter handles the failure
// through the same promise-rejection path it uses for unavailable commands.
function mockReject(error: Error) {
  invokeMock.mockRejectedValueOnce(error);
}

const DESIGN_DETAIL_WIRE = {
  id: 1,
  filename: "rose-border-01.pes",
  filepath: "C:/designs/rose-border-01.pes",
  image_type: "png",
  image_data_url: "data:image/png;base64,abc",
  width_mm: 100,
  height_mm: 120,
  stitch_count: 5000,
  color_count: 4,
  color_change_count: 3,
  designer: "Mock Designer",
  designer_id: 7,
  source: "Mock Source",
  source_id: 9,
  hoop: "Hoop A",
  hoop_id: 1,
  is_stitched: true,
  image_tags_verified: false,
  stitching_tags_verified: true,
  notes: "Some notes",
  rating: 4,
  tagging_tier: 2,
  date_added: "2026-01-01",
  tags: [
    { id: 1, description: "Flowers", tag_group: null },
    { id: 2, description: "Borders", tag_group: null },
  ],
  projects: [{ id: 1, name: "Project A" }],
  available_projects: [{ id: 2, name: "Project B" }],
  all_tags: [{ id: 1, description: "Flowers", tag_group: null, is_system: false }],
  designers: [{ id: 7, name: "Mock Designer" }],
  sources: [{ id: 9, name: "Mock Source" }],
  hoops: [{ id: 1, name: "Hoop A" }],
};

const REPARSE_WIRE = {
  design_id: 5,
  width_mm: 200,
  height_mm: 180,
  stitch_count: 9000,
  color_count: 6,
  color_change_count: 5,
  hoop_id: 2,
  hoop: "Hoop B",
  message: "Recalculated.",
};

const APP_STATUS = {
  execution_mode: "portable",
  data_root: "C:/data",
  embroidery_dir: "C:/data/embroidery",
  database_path: "C:/data/catalogue.db",
};

describe("commandAdapter getBrowseDesigns", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps a Rust payload into normalized browse items", async () => {
    invokeMock.mockResolvedValue([
      {
        id: 10,
        filename: "a.pes",
        filepath: "/a/a.pes",
        designer: "D1",
        source: "S1",
        projects: ["P1"],
        tags: ["Flowers"],
        image_tags: ["image-tag"],
        stitching_tags: ["stitch-tag"],
        hoop: "Hoop A",
        rating: "4",
        is_stitched: true,
        image_tags_verified: true,
        stitching_tags_verified: true,
      },
      {
        id: 11,
        name: "b.pes",
        project_names: "P1, P2",
        tags: [],
        rating: 9,
        is_stitched: false,
      },
    ]);

    const result = await getBrowseDesigns();

    expect(result.source).toBe("rust");
    expect(invokeMock).toHaveBeenCalledWith("get_designs", { payload: undefined });
    expect(result.items).toHaveLength(2);
    expect(result.items[0]).toMatchObject({
      id: 10,
      filename: "a.pes",
      filepath: "/a/a.pes",
      designer: "D1",
      source: "S1",
      projects: ["P1"],
      tags: ["Flowers"],
      image_tags: ["image-tag"],
      stitching_tags: ["stitch-tag"],
      hoop: "Hoop A",
      rating: 4,
      is_stitched: true,
      image_tags_verified: true,
      stitching_tags_verified: true,
    });
    // project_names string split, empty tags, rating clamped to 5
    expect(result.items[1].projects).toEqual(["P1", "P2"]);
    expect(result.items[1].tags).toEqual([]);
    expect(result.items[1].rating).toBe(5);
  });

  it("falls back to mock data when the command rejects", async () => {
    mockReject(new Error("not available"));

    const result = await getBrowseDesigns();

    expect(result.source).toBe("mock");
    expect(result.items).toHaveLength(3);
    // Mock items use seeded tags.
    expect(result.items[0].tags.length).toBeGreaterThan(0);
  });

  it("falls back to mock data when the command returns a non-array", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getBrowseDesigns();

    expect(result.source).toBe("mock");
    expect(result.items).toHaveLength(3);
  });
});

describe("commandAdapter getDesignDetail", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns an error for an invalid design id", async () => {
    const result = await getDesignDetail("abc");

    expect(result).toEqual({ item: null, source: "mock", error: "Invalid design id: abc" });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("returns mapped detail from Rust", async () => {
    invokeMock.mockResolvedValue(DESIGN_DETAIL_WIRE);

    const result = await getDesignDetail(1);

    expect(result.source).toBe("rust");
    expect(invokeMock).toHaveBeenCalledWith("get_design_detail", { designId: 1 });
    expect(result.item?.id).toBe(1);
    expect(result.item?.stitchCount).toBe(5000);
    expect(result.item?.hoopId).toBe(1);
    expect(result.item?.projects).toEqual([{ id: 1, name: "Project A" }]);
  });

  it("falls back to mock detail when both invokes fail", async () => {
    mockReject(new Error("rust missing"));

    const result = await getDesignDetail(2);

    expect(invokeMock).toHaveBeenCalledTimes(2);
    expect(result.source).toBe("mock");
    expect(result.error).toContain("rust missing");
    expect(result.item?.filename).toBe("holiday-tree.vp3");
    expect(result.item?.hoop).toBe("Hoop B");
    expect(result.item?.hoopId).toBe(2);
  });

  it("returns null mock item when id is not in mock data and invokes fail", async () => {
    mockReject(new Error("rust missing"));

    const result = await getDesignDetail(999);

    expect(result.source).toBe("mock");
    expect(result.item).toBeNull();
    expect(result.error).toContain("rust missing");
  });

  it("falls back to mock detail when invokes resolve with null", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getDesignDetail(1);

    expect(invokeMock).toHaveBeenCalledTimes(2);
    expect(result.source).toBe("mock");
    // no error field on the non-error fallback path
    expect(result.error).toBeUndefined();
    expect(result.item?.id).toBe(1);
  });
});

describe("commandAdapter getDesignImageDataUrl", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns an empty mock item for an invalid design id", async () => {
    const result = await getDesignImageDataUrl(0);

    expect(result).toEqual({ item: null, source: "mock" });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("returns image data from Rust", async () => {
    invokeMock.mockResolvedValue({ design_id: 1, image_type: "png", data_url: "data:... " });

    const result = await getDesignImageDataUrl(1);

    expect(result.source).toBe("rust");
    expect(result.item).toEqual({ design_id: 1, image_type: "png", data_url: "data:... " });
  });

  it("falls back to null mock image on error", async () => {
    mockReject(new Error("boom"));

    const result = await getDesignImageDataUrl(1);

    expect(result).toEqual({ item: null, source: "mock" });
  });
});

describe("commandAdapter design mutations", () => {
  beforeEach(() => invokeMock.mockReset());

  const wireResult = { design_id: 42, message: "ok" };

  it("updateDesignMetadata succeeds from Rust", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const result = await updateDesignMetadata(42, { notes: "note" });

    expect(invokeMock).toHaveBeenCalledWith("update_design_metadata", {
      designId: 42,
      request: { notes: "note" },
    });
    expect(result).toEqual({
      source: "rust",
      persisted: true,
      design_id: 42,
      message: "ok",
    });
  });

  it("updateDesignMetadata falls back to mock on error", async () => {
    mockReject(new Error("failed"));

    const result = await updateDesignMetadata(2, {});

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.design_id).toBe(2);
    expect(result.error).toContain("failed");
  });

  it("setDesignRating passes a null rating through", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const result = await setDesignRating(42, null);

    expect(invokeMock).toHaveBeenCalledWith("set_design_rating", {
      designId: 42,
      request: { rating: null },
    });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
  });

  it("setDesignRating passes a numeric rating and falls back to mock on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await setDesignRating(3, 5);
    expect(invokeMock).toHaveBeenCalledWith("set_design_rating", {
      designId: 3,
      request: { rating: 5 },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("nope"));
    const bad = await setDesignRating(3, 1);
    expect(bad.source).toBe("mock");
    expect(bad.persisted).toBe(false);
    expect(bad.error).toContain("nope");
  });

  it("setDesignStitched sends is_stitched and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await setDesignStitched(5, true);
    expect(invokeMock).toHaveBeenCalledWith("set_design_stitched", {
      designId: 5,
      request: { is_stitched: true },
    });
    expect(ok.source).toBe("rust");

    mockReject(new Error("err"));
    const bad = await setDesignStitched(5, false);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("err");
  });

  it("setDesignVerification sends camelCase id + snake_case verification flags and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await setDesignVerification(7, {
      imageTagsVerified: true,
      stitchingTagsVerified: false,
    });
    // Assert the EXACT wire payload Tauri expects (see .clinerules):
    // invoke keys are camelCase, but the nested request struct is deserialized
    // by serde using snake_case field names.
    expect(invokeMock).toHaveBeenCalledWith("set_design_verification", {
      designId: 7,
      request: {
        image_tags_verified: true,
        stitching_tags_verified: false,
      },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("boom"));
    const bad = await setDesignVerification(7, {
      imageTagsVerified: true,
      stitchingTagsVerified: false,
    });
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("boom");
  });

  it("setDesignTags deduplicates and filters tag ids", async () => {
    invokeMock.mockResolvedValue(wireResult);

    await setDesignTags(1, ["2", 2, 3, 0, -1, "abc", 3]);

    expect(invokeMock).toHaveBeenCalledWith("set_design_tags", {
      designId: 1,
      request: {
        tag_ids: [2, 3],
        image_tags_verified: null,
        stitching_tags_verified: null,
      },
    });
  });

  it("setDesignTags handles a non-array input and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await setDesignTags(1, []);
    expect(ok.source).toBe("rust");

    mockReject(new Error("x"));
    const bad = await setDesignTags(1, [1]);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("x");
  });

  it("removeDesignTag passes tagId and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await removeDesignTag(1, 9);
    expect(invokeMock).toHaveBeenCalledWith("remove_design_tag", {
      designId: 1,
      tagId: 9,
    });
    expect(ok.source).toBe("rust");

    mockReject(new Error("e"));
    const bad = await removeDesignTag(1, 9);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("e");
  });

  it("addDesignToProject sends project_id and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await addDesignToProject(1, 3);
    expect(invokeMock).toHaveBeenCalledWith("add_design_to_project", {
      designId: 1,
      request: { project_id: 3 },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("e"));
    const bad = await addDesignToProject(1, 3);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("e");
  });

  it("removeDesignFromProject passes projectId and falls back on error", async () => {
    invokeMock.mockResolvedValue(wireResult);

    const ok = await removeDesignFromProject(1, 3);
    expect(invokeMock).toHaveBeenCalledWith("remove_design_from_project", {
      designId: 1,
      projectId: 3,
    });
    expect(ok.source).toBe("rust");

    mockReject(new Error("e"));
    const bad = await removeDesignFromProject(1, 3);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("e");
  });
});

describe("commandAdapter bulkDeleteDesigns", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns a zero-result mock when the id list is empty", async () => {
    const result = await bulkDeleteDesigns([]);

    expect(result).toEqual({
      source: "mock",
      persisted: false,
      deleted_count: 0,
      files_trashed: 0,
      errors: [],
    });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("normalizes ids and sends the delete_files flag on success", async () => {
    invokeMock.mockResolvedValue({ deleted_count: 2, files_trashed: 1, errors: ["warn"] });

    const result = await bulkDeleteDesigns(["1", 0, 7, 7, "abc", 2], true);

    expect(invokeMock).toHaveBeenCalledWith("bulk_delete_designs", {
      request: { design_ids: [1, 7, 2], delete_files: true },
    });
    expect(result).toEqual({
      source: "rust",
      persisted: true,
      deleted_count: 2,
      files_trashed: 1,
      errors: ["warn"],
    });
  });

  it("falls back to mock on error, surfacing the error string", async () => {
    mockReject(new Error("cannot delete"));

    const result = await bulkDeleteDesigns([1, 2]);

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    // String(error) includes the "Error: " prefix.
    expect(result.errors).toEqual(["Error: cannot delete"]);
  });
});

describe("commandAdapter openDesignInEditor / openDesignInExplorer / renderDesign3dPreview", () => {
  beforeEach(() => invokeMock.mockReset());

  it("openDesignInEditor succeeds and falls back on error", async () => {
    invokeMock.mockResolvedValue({ message: "opened" });

    const ok = await openDesignInEditor(1);
    expect(invokeMock).toHaveBeenCalledWith("open_design_in_editor", { designId: 1 });
    expect(ok.source).toBe("rust");
    expect(ok.persisted).toBe(true);
    expect(ok.message).toBe("opened");

    mockReject(new Error("e"));
    const bad = await openDesignInEditor(1);
    expect(bad.source).toBe("mock");
    expect(bad.result).toBeNull();
    expect(bad.error).toContain("e");
  });

  it("openDesignInExplorer succeeds and falls back on error", async () => {
    invokeMock.mockResolvedValue({ message: "shown" });

    const ok = await openDesignInExplorer(2);
    expect(invokeMock).toHaveBeenCalledWith("open_design_in_explorer", { designId: 2 });
    expect(ok.source).toBe("rust");

    mockReject(new Error("e"));
    const bad = await openDesignInExplorer(2);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("e");
  });

  it("renderDesign3dPreview sends preview_3d true by default", async () => {
    invokeMock.mockResolvedValue({ message: "rendered" });

    await renderDesign3dPreview(1);
    expect(invokeMock).toHaveBeenCalledWith("render_design_3d_preview", {
      designId: 1,
      request: { preview_3d: true },
    });
  });

  it("renderDesign3dPreview sends preview_3d false for 2D and falls back on error", async () => {
    invokeMock.mockResolvedValue({});

    await renderDesign3dPreview(1, false);
    expect(invokeMock).toHaveBeenCalledWith("render_design_3d_preview", {
      designId: 1,
      request: { preview_3d: false },
    });

    mockReject(new Error("e"));
    const bad = await renderDesign3dPreview(1);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("e");
  });
});

describe("commandAdapter reparseDesignFile", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps the wire result from Rust", async () => {
    invokeMock.mockResolvedValue(REPARSE_WIRE);

    const result = await reparseDesignFile(5);

    expect(invokeMock).toHaveBeenCalledWith("reparse_design_file", { designId: 5 });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
    expect(result.result).toEqual({
      designId: 5,
      widthMm: 200,
      heightMm: 180,
      stitchCount: 9000,
      colorCount: 6,
      colorChangeCount: 5,
      hoopId: 2,
      hoop: "Hoop B",
      message: "Recalculated.",
    });
    expect(result.message).toBe("Recalculated.");
  });

  it("returns null result when the invoke resolves with null", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await reparseDesignFile(5);

    expect(result.source).toBe("rust");
    expect(result.result).toBeNull();
  });

  it("falls back to mock on error", async () => {
    mockReject(new Error("reparse failed"));

    const result = await reparseDesignFile(5);

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.result).toBeNull();
    expect(result.error).toContain("reparse failed");
  });
});

describe("commandAdapter previewImportFromRoots", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns an invalid_root preview for empty roots", async () => {
    const result = await previewImportFromRoots([]);

    expect(result.source).toBe("mock");
    expect(result.preview.invalid_root).toBe(true);
    expect(result.preview.discovered_count).toBe(0);
    expect(result.message).toContain("Enter at least one folder path");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("maps a Rust preview payload", async () => {
    invokeMock.mockResolvedValue({
      discovered_count: 10,
      selected_count: 8,
      scanned_files: ["a.pes"],
      resolved_assignments: [{ x: 1 }],
      missing_root: true,
      no_supported_files: false,
      invalid_root: false,
    });

    const result = await previewImportFromRoots(["C:/a", "C:/b"]);

    expect(invokeMock).toHaveBeenCalledWith("preview_bulk_import", {
      request: {
        root_path: "C:/a",
        root_paths: ["C:/a", "C:/b"],
        fallback_designer_id: null,
        fallback_source_id: null,
      },
    });
    expect(result.source).toBe("rust");
    expect(result.preview.discovered_count).toBe(10);
    expect(result.preview.folder_count).toBe(2);
  });

  it("falls back to a mock preview on error", async () => {
    mockReject(new Error("preview failed"));

    const result = await previewImportFromRoots(["C:/a"]);

    expect(result.source).toBe("mock");
    expect(result.preview.no_supported_files).toBe(true);
    expect(result.preview.folder_count).toBe(1);
    expect(result.message).toContain("preview failed");
  });

  it("previewImportFromRoot delegates an empty string to empty roots", async () => {
    const result = await previewImportFromRoot("   ");

    expect(result.source).toBe("mock");
    expect(result.preview.invalid_root).toBe(true);
  });

  it("previewImportFromRoot delegates a single root", async () => {
    invokeMock.mockResolvedValue({ discovered_count: 3 });

    const result = await previewImportFromRoot(" C:/single ");

    expect(invokeMock).toHaveBeenCalledWith("preview_bulk_import", {
      request: {
        root_path: "C:/single",
        root_paths: ["C:/single"],
        fallback_designer_id: null,
        fallback_source_id: null,
      },
    });
    expect(result.source).toBe("rust");
    expect(result.preview.discovered_count).toBe(3);
  });
});

describe("commandAdapter browseImportFolder", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps a Rust folder result", async () => {
    invokeMock.mockResolvedValue({ path: "C:/picked", paths: ["C:/picked", "C:/other"] });

    const result = await browseImportFolder("C:/start");

    expect(invokeMock).toHaveBeenCalledWith("browse_import_folder", {
      request: { start_dir: "C:/start", allow_multi: true },
    });
    expect(result.source).toBe("rust");
    expect(result.path).toBe("C:/picked");
    expect(result.paths).toEqual(["C:/picked", "C:/other"]);
    expect(result.message).toBe("Folder selected.");
  });

  it("treats a null path as a cancelled selection", async () => {
    invokeMock.mockResolvedValue({ path: null, paths: [] });

    const result = await browseImportFolder();

    expect(result.path).toBe("");
    expect(result.message).toBe("Folder selection cancelled.");
  });

  it("falls back to mock browse on error", async () => {
    mockReject(new Error("no picker"));

    const result = await browseImportFolder("C:/start");

    expect(result.source).toBe("mock");
    expect(result.path).toBe("C:/start");
    expect(result.paths).toEqual([]);
    expect(result.message).toContain("not available");
  });
});

describe("commandAdapter precheckImportWire", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns a mock precheck for a missing wire payload", async () => {
    const result = await precheckImportWire(null);

    expect(result.source).toBe("mock");
    expect(result.precheck.context_token_present).toBe(false);
    expect(result.precheck.ready_for_confirm).toBe(false);
    expect(result.message).toContain("Missing confirm wire payload");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("passes the wire through to Rust on success", async () => {
    const wire = { context_token: "tok", selected_count: 3 };
    invokeMock.mockResolvedValue({
      context_token: "tok",
      context_token_present: true,
      ready_for_confirm: true,
      is_first_import: true,
      needs_hoop_setup: false,
      root_path_count: 1,
      selected_file_count: 3,
      resolved_assignments: [],
    });

    const result = await precheckImportWire(wire);

    expect(invokeMock).toHaveBeenCalledWith("precheck_bulk_import_wire", {
      confirmWire: wire,
    });
    expect(result.source).toBe("rust");
    expect(result.precheck.ready_for_confirm).toBe(true);
  });

  it("rethrows a wrapped error when the command fails", async () => {
    mockReject(new Error("backend down"));

    await expect(precheckImportWire({ context_token: "tok" })).rejects.toThrow(
      "Precheck failed: Error: backend down"
    );
  });
});

describe("commandAdapter runPrecheckAction", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns a mock missing-payload result when inputs are blank", async () => {
    const result = await runPrecheckAction({ contextToken: "", action: "" });

    expect(result.source).toBe("mock");
    expect(result.actionResult.action).toBe("");
    expect(result.actionResult.context_token_present).toBe(false);
    expect(result.message).toContain("Missing precheck action payload");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("returns a mock missing-payload result when only token is blank", async () => {
    const result = await runPrecheckAction({ contextToken: "", action: "cancel" });

    expect(result.source).toBe("mock");
    expect(result.actionResult.action).toBe("cancel");
  });

  it("passes the normalized request to Rust on success", async () => {
    invokeMock.mockResolvedValue({
      action: "import_now",
      context_token_present: true,
      consumed_context: true,
      requires_skip_hoops_confirmation: false,
      next_route: "/import/",
      confirm_result: null,
    });

    const result = await runPrecheckAction({
      contextToken: " tok ",
      action: " import_now ",
      confirmSkipHoops: true,
    });

    expect(invokeMock).toHaveBeenCalledWith("precheck_bulk_import_action_wire", {
      request: {
        context_token: "tok",
        action: "import_now",
        confirm_skip_hoops: true,
      },
    });
    expect(result.source).toBe("rust");
    expect(result.actionResult.next_route).toBe("/import/");
    expect(result.message).toBe("");
  });

  it("falls back to a cancel mock result on error", async () => {
    mockReject(new Error("x"));

    const result = await runPrecheckAction({ contextToken: "tok", action: "cancel" });

    expect(result.source).toBe("mock");
    expect(result.actionResult.context_token_present).toBe(false);
    expect(result.actionResult.consumed_context).toBe(true);
    expect(result.actionResult.next_route).toBe("/import/");
  });

  it("falls back to an import_now mock result on error", async () => {
    mockReject(new Error("x"));

    const result = await runPrecheckAction({ contextToken: "tok", action: "import_now" });

    expect(result.source).toBe("mock");
    expect(result.actionResult.context_token_present).toBe(true);
    expect(result.actionResult.consumed_context).toBe(false);
    expect(result.actionResult.next_route).toBeNull();
  });
});

describe("commandAdapter requestStopBulkImport", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns stopRequested from Rust", async () => {
    invokeMock.mockResolvedValue({ stop_requested: true });

    const result = await requestStopBulkImport();

    expect(result.source).toBe("rust");
    expect(result.stopRequested).toBe(true);
  });

  it("falls back to a true mock stop on error", async () => {
    mockReject(new Error("x"));

    const result = await requestStopBulkImport();

    expect(result.source).toBe("mock");
    expect(result.stopRequested).toBe(true);
  });
});

describe("commandAdapter bulkVerifyDesigns", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns a zero-result mock when no valid ids", async () => {
    const result = await bulkVerifyDesigns([]);

    expect(result).toEqual({
      source: "mock",
      requested_count: 0,
      verified_count: 0,
      persisted: false,
    });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("maps a Rust result", async () => {
    invokeMock.mockResolvedValue({ requested_count: 2, verified_count: 2 });

    const result = await bulkVerifyDesigns(["1", 2, 0, "x"]);

    expect(invokeMock).toHaveBeenCalledWith("bulk_verify_designs", { designIds: [1, 2] });
    expect(result).toEqual({
      source: "rust",
      requested_count: 2,
      verified_count: 2,
      persisted: true,
    });
  });

  it("falls back to local verify on error", async () => {
    mockReject(new Error("x"));

    const result = await bulkVerifyDesigns([1, 2]);

    expect(result.source).toBe("mock");
    expect(result.verified_count).toBe(2);
    expect(result.persisted).toBe(false);
  });
});

describe("commandAdapter projects", () => {
  beforeEach(() => invokeMock.mockReset());

  it("getBrowseProjects returns Rust items", async () => {
    invokeMock.mockResolvedValue([
      { id: 1, name: "A" },
      { id: 2, name: "B" },
    ]);

    const result = await getBrowseProjects();

    expect(result.source).toBe("rust");
    expect(result.items).toHaveLength(2);
  });

  it("getBrowseProjects falls back to mock projects", async () => {
    mockReject(new Error("x"));

    const result = await getBrowseProjects();

    expect(result.source).toBe("mock");
    expect(result.items).toHaveLength(3);
    expect(result.items[0].name).toBe("Project A");
  });

  it("getProjectsList returns Rust items", async () => {
    invokeMock.mockResolvedValue([{ id: 1, name: "A", design_count: 3 }]);

    const result = await getProjectsList();

    expect(result.source).toBe("rust");
    expect(result.items[0].design_count).toBe(3);
  });

  it("getProjectsList falls back to empty mock on error", async () => {
    mockReject(new Error("load failed"));

    const result = await getProjectsList();

    expect(result.source).toBe("mock");
    expect(result.items).toEqual([]);
    expect(result.error).toContain("load failed");
  });

  it("getProjectsList times out and falls back to empty mock", async () => {
    vi.useFakeTimers();

    try {
      // The invoke settles LATE (100ms) — after the 25ms timeout — so the
      // timeout wins the Promise.race. Settling both sides of the race means
      // no dangling promise is left behind for vitest to wait on.
      invokeMock.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(null), 100))
      );

      const pending = getProjectsList(25);
      // Fire the 25ms timeout (which rejects and settles `pending`) and then
      // the 100ms late resolve so both race participants settle.
      vi.advanceTimersByTime(100);

      const result = await pending;

      expect(result.source).toBe("mock");
      expect(result.items).toEqual([]);
      expect(result.error).toContain("Timed out loading projects");
    } finally {
      vi.useRealTimers();
    }
  });

  it("createProject sends normalized payload and maps the Rust result", async () => {
    invokeMock.mockResolvedValue({ project_id: 9, message: "Created." });

    const result = await createProject("  My Project  ", "  ");

    expect(invokeMock).toHaveBeenCalledWith("create_project", {
      request: { name: "My Project", description: null },
    });
    expect(result).toEqual({
      source: "rust",
      persisted: true,
      project_id: 9,
      message: "Created.",
    });
  });

  it("createProject falls back to mock on error", async () => {
    mockReject(new Error("create failed"));

    const result = await createProject("A", "B");

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.project_id).toBe(0);
    expect(result.error).toContain("create failed");
  });

  it("getProjectDetail returns an error for an invalid id", async () => {
    const result = await getProjectDetail(-1);

    expect(result).toEqual({ item: null, source: "mock", error: "Invalid project id: -1" });
  });

  it("getProjectDetail returns Rust detail", async () => {
    invokeMock.mockResolvedValue({ project: { id: 1, name: "P", description: null }, designs: [] });

    const result = await getProjectDetail(1);

    expect(invokeMock).toHaveBeenCalledWith("get_project_detail", { projectId: 1 });
    expect(result.source).toBe("rust");
    expect(result.item?.project?.name).toBe("P");
  });

  it("getProjectDetail falls back to mock error on failure", async () => {
    mockReject(new Error("detail failed"));

    const result = await getProjectDetail(1);

    expect(result.source).toBe("mock");
    expect(result.item).toBeNull();
    expect(result.error).toContain("detail failed");
  });

  it("getProjectDetail returns an error when the payload is empty", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getProjectDetail(1);

    expect(result.source).toBe("mock");
    expect(result.item).toBeNull();
    expect(result.error).toBe("Project detail was empty.");
  });

  it("updateProject sends projectId and payload, maps the Rust result", async () => {
    invokeMock.mockResolvedValue({ project_id: 4, message: "Updated." });

    const result = await updateProject(4, "New", "Desc");

    expect(invokeMock).toHaveBeenCalledWith("update_project", {
      projectId: 4,
      request: { name: "New", description: "Desc" },
    });
    expect(result.source).toBe("rust");
    expect(result.project_id).toBe(4);
  });

  it("updateProject falls back to mock on error", async () => {
    mockReject(new Error("update failed"));

    const result = await updateProject(4, "New", "Desc");

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("update failed");
  });

  it("deleteProject sends projectId and maps the Rust result", async () => {
    invokeMock.mockResolvedValue({ project_id: 6, message: "Deleted." });

    const result = await deleteProject(6);

    expect(invokeMock).toHaveBeenCalledWith("delete_project", { projectId: 6 });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
  });

  it("deleteProject falls back to mock on error", async () => {
    mockReject(new Error("delete failed"));

    const result = await deleteProject(6);

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("delete failed");
  });

  it("removeDesignFromProjectDetail sends ids and maps the Rust result", async () => {
    invokeMock.mockResolvedValue({ project_id: 1, design_id: 2, message: "Removed." });

    const result = await removeDesignFromProjectDetail(1, 2);

    expect(invokeMock).toHaveBeenCalledWith("remove_design_from_project_detail", {
      projectId: 1,
      designId: 2,
    });
    expect(result.source).toBe("rust");
    expect(result.project_id).toBe(1);
    expect(result.design_id).toBe(2);
  });

  it("removeDesignFromProjectDetail falls back to mock on error", async () => {
    mockReject(new Error("remove failed"));

    const result = await removeDesignFromProjectDetail(1, 2);

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("remove failed");
  });

  it("getProjectPrintView returns Rust view and a mock error fallback", async () => {
    invokeMock.mockResolvedValue({ project: { id: 1, name: "P", description: null }, designs: [] });

    const ok = await getProjectPrintView(1);
    expect(ok.source).toBe("rust");
    expect(invokeMock).toHaveBeenCalledWith("get_project_print_view", { projectId: 1 });

    mockReject(new Error("print failed"));
    const bad = await getProjectPrintView(1);
    expect(bad.source).toBe("mock");
    expect(bad.item).toBeNull();
    expect(bad.error).toContain("print failed");
  });

  it("getProjectPrintView returns an empty error for a null view", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getProjectPrintView(1);

    expect(result.source).toBe("mock");
    expect(result.error).toBe("Project print view was empty.");
  });

  it("bulkAddDesignsToProject returns a mock zero-result for invalid inputs", async () => {
    const result = await bulkAddDesignsToProject(0, []);

    expect(result).toEqual({
      source: "mock",
      project_id: 0,
      requested_count: 0,
      added_count: 0,
      persisted: false,
    });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("bulkAddDesignsToProject maps a Rust result", async () => {
    invokeMock.mockResolvedValue({ project_id: 3, requested_count: 2, added_count: 2 });

    const result = await bulkAddDesignsToProject(3, ["1", 2, 0, "x"]);

    expect(invokeMock).toHaveBeenCalledWith("bulk_add_designs_to_project", {
      projectId: 3,
      designIds: [1, 2],
    });
    expect(result.source).toBe("rust");
    expect(result.added_count).toBe(2);
  });

  it("bulkAddDesignsToProject falls back to mock on error", async () => {
    mockReject(new Error("bulk add failed"));

    const result = await bulkAddDesignsToProject(3, [1]);

    expect(result.source).toBe("mock");
    expect(result.added_count).toBe(0);
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("bulk add failed");
  });
});

describe("commandAdapter tags & previews", () => {
  beforeEach(() => invokeMock.mockReset());

  it("getBrowseTags maps Rust tag options", async () => {
    invokeMock.mockResolvedValue([
      { id: 1, description: "Floral", tag_group: "Theme", is_system: true },
      { id: 2, description: "X", tag_group: null, is_system: false },
    ]);

    const result = await getBrowseTags();

    expect(result.source).toBe("rust");
    expect(result.items[0]).toEqual({ id: 1, description: "Floral", tag_group: "Theme", is_system: true });
    expect(result.items[1].tag_group).toBeNull();
  });

  it("getBrowseTags reports an unexpected payload", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getBrowseTags();

    expect(result.source).toBe("rust");
    expect(result.items).toEqual([]);
    expect(result.error).toContain("unexpected payload");
  });

  it("getBrowseTags falls back to mock on error", async () => {
    mockReject(new Error("tags failed"));

    const result = await getBrowseTags();

    expect(result.source).toBe("mock");
    expect(result.items).toEqual([]);
    expect(result.error).toContain("tags failed");
  });

  it("bulkSetTagsForDesigns returns mock zero-result for empty design ids", async () => {
    const result = await bulkSetTagsForDesigns([], [1]);

    expect(result).toEqual({ source: "mock", requested_count: 0, updated_count: 0, persisted: false });
  });

  it("bulkSetTagsForDesigns sends snake_case request fields for the Rust struct", async () => {
    invokeMock.mockResolvedValue({ requested_count: 2, updated_count: 2 });

    const result = await bulkSetTagsForDesigns([1, 2], ["2", 2, 3], [4, "4", 5]);

    // Assert the EXACT wire payload Tauri expects (see .clinerules):
    // top-level invoke keys are camelCase, but the nested `request` struct is
    // deserialized by serde using snake_case field names.
    expect(invokeMock).toHaveBeenCalledWith("bulk_set_tags_for_designs", {
      designIds: [1, 2],
      request: {
        tags_to_add: [2, 3],
        tags_to_remove: [4, 5],
        clear_all_tags: false,
        image_tags_verified: null,
        stitching_tags_verified: null,
      },
    });
    expect(result.source).toBe("rust");
    expect(result.updated_count).toBe(2);
  });

  it("bulkSetTagsForDesigns passes clearAllTags through", async () => {
    invokeMock.mockResolvedValue({ requested_count: 1, updated_count: 1 });

    await bulkSetTagsForDesigns([1], [], [], true);

    expect(invokeMock).toHaveBeenCalledWith("bulk_set_tags_for_designs", {
      designIds: [1],
      request: {
        tags_to_add: [],
        tags_to_remove: [],
        clear_all_tags: true,
        image_tags_verified: null,
        stitching_tags_verified: null,
      },
    });
  });

  it("bulkSetTagsForDesigns defaults to empty remove list and clearAllTags=false", async () => {
    invokeMock.mockResolvedValue({ requested_count: 1, updated_count: 1 });

    await bulkSetTagsForDesigns([1], [7]);

    expect(invokeMock).toHaveBeenCalledWith("bulk_set_tags_for_designs", {
      designIds: [1],
      request: {
        tags_to_add: [7],
        tags_to_remove: [],
        clear_all_tags: false,
        image_tags_verified: null,
        stitching_tags_verified: null,
      },
    });
  });

  it("bulkSetTagsForDesigns falls back to local full-verify mock on error", async () => {
    mockReject(new Error("x"));

    const result = await bulkSetTagsForDesigns([1, 2], [3]);

    expect(result.source).toBe("mock");
    expect(result.updated_count).toBe(2);
    expect(result.persisted).toBe(false);
  });

  it("getBrowseDesignPreviews returns mock empty for no ids", async () => {
    const result = await getBrowseDesignPreviews([]);

    expect(result).toEqual({ items: [], source: "mock" });
  });

  it("getBrowseDesignPreviews maps Rust previews", async () => {
    invokeMock.mockResolvedValue([
      { id: 1, data_url: "data:png;base64,aa" },
      { id: 2, data_url: null },
      { id: 3 },
    ]);

    const result = await getBrowseDesignPreviews(["1", 2, 3, 0, 3]);

    expect(invokeMock).toHaveBeenCalledWith("get_design_previews_for_browse", {
      designIds: [1, 2, 3],
    });
    expect(result.source).toBe("rust");
    expect(result.items).toEqual([
      { id: 1, data_url: "data:png;base64,aa" },
      { id: 2, data_url: null },
      { id: 3, data_url: null },
    ]);
  });

  it("getBrowseDesignPreviews falls back to null-data mock previews", async () => {
    mockReject(new Error("no previews"));

    const result = await getBrowseDesignPreviews([1, 2]);

    expect(result.source).toBe("mock");
    expect(result.items).toEqual([
      { id: 1, data_url: null },
      { id: 2, data_url: null },
    ]);
  });
});

describe("commandAdapter about documents", () => {
  beforeEach(() => invokeMock.mockReset());

  it("getAboutDocuments maps Rust docs", async () => {
    invokeMock.mockResolvedValue([
      { slug: "privacy", title: "Privacy", description: "desc", filename: "PRIVACY.html", available: true },
    ]);

    const result = await getAboutDocuments();

    expect(result.source).toBe("rust");
    expect(result.items[0].available).toBe(true);
  });

  it("getAboutDocuments falls back to the mock document list", async () => {
    mockReject(new Error("docs unavailable"));

    const result = await getAboutDocuments();

    expect(result.source).toBe("mock");
    expect(result.items.length).toBeGreaterThan(0);
    expect(result.items[0].slug).toBe("disclaimer");
  });

  it("getAboutDocument returns mock missing for a blank slug", async () => {
    const result = await getAboutDocument("   ");

    expect(result).toEqual({ item: null, source: "mock", error: "Document not found." });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("getAboutDocument maps a Rust document", async () => {
    invokeMock.mockResolvedValue({
      slug: "privacy",
      title: "Privacy",
      description: "desc",
      filename: "PRIVACY.html",
      document_text: "long text",
    });

    const result = await getAboutDocument(" PRIVACY ");

    expect(invokeMock).toHaveBeenCalledWith("get_about_document", { slug: "privacy" });
    expect(result.source).toBe("rust");
    expect(result.item?.document_text).toBe("long text");
  });

  it("getAboutDocument falls back to mock null on error", async () => {
    mockReject(new Error("doc failed"));

    const result = await getAboutDocument("privacy");

    expect(result.source).toBe("mock");
    expect(result.item).toBeNull();
    expect(result.error).toContain("doc failed");
  });

  it("getAboutDocument reports not found when payload is empty", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getAboutDocument("privacy");

    expect(result.source).toBe("mock");
    expect(result.item).toBeNull();
    expect(result.error).toBe("Document not found.");
  });
});

describe("commandAdapter settings", () => {
  beforeEach(() => invokeMock.mockReset());

  const settingsModel = {
    preview_3d_profile: "balanced",
    google_api_key: "",
    has_google_api_key: false,
    ai_tier2_auto: true,
    ai_tier3_auto: false,
    ai_batch_size: "50",
    ai_delay: "100",
    import_commit_batch_size: "200",
    import_last_browse_folder: "",
    can_configure_data_root: true,
    data_root: "C:/data",
    database_path: "C:/data/db.sqlite",
    log_folder: "C:/data/logs",
    app_mode: "portable",
    ai_tagging_help_url: "#/help",
    db_idle_check_interval_secs: "1800",
  };

  it("getSettingsViewModel returns a Rust model", async () => {
    invokeMock.mockResolvedValue(settingsModel);

    const result = await getSettingsViewModel();

    expect(result.source).toBe("rust");
    expect(result.model).toEqual(settingsModel);
  });

  it("getSettingsViewModel falls back to the mock model on error", async () => {
    mockReject(new Error("settings failed"));

    const result = await getSettingsViewModel();

    expect(result.source).toBe("mock");
    expect(result.model.app_mode).toBe("development");
    expect(result.model.has_google_api_key).toBe(false);
  });

  it("saveSettings maps Rust saved result", async () => {
    invokeMock.mockResolvedValue({ saved: true, message: "Saved." });

    const request = { google_api_key: "k", ai_tier2_auto: true, ai_tier3_auto: false, ai_batch_size: "1", ai_delay: "2", import_commit_batch_size: "3", data_root: "C:/x", preview_3d_profile: "balanced" };
    const result = await saveSettings(request);

    expect(invokeMock).toHaveBeenCalledWith("save_settings_view_model", { request });
    expect(result).toEqual({
      source: "rust",
      saved: true,
      message: "Saved.",
      persisted: true,
    });
  });

  it("saveSettings falls back to mock on error", async () => {
    mockReject(new Error("save failed"));

    const request = { google_api_key: "k", ai_tier2_auto: true, ai_tier3_auto: false, ai_batch_size: "1", ai_delay: "2", import_commit_batch_size: "3", data_root: "C:/x" };
    const result = await saveSettings(request);

    expect(result.source).toBe("mock");
    expect(result.saved).toBe(false);
    expect(result.persisted).toBe(false);
    expect(result.message).toContain("save failed");
  });

  it("saveImportLastBrowseFolder maps Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ saved: true, path: "C:/last" });

    const ok = await saveImportLastBrowseFolder("C:/last");
    expect(invokeMock).toHaveBeenCalledWith("save_import_last_browse_folder", { path: "C:/last" });
    expect(ok.source).toBe("rust");
    expect(ok.path).toBe("C:/last");

    mockReject(new Error("e"));
    const bad = await saveImportLastBrowseFolder("C:/last");
    expect(bad.source).toBe("mock");
    expect(bad.saved).toBe(false);
    expect(bad.error).toContain("e");
  });

  it("browseSettingsDataRoot maps Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ path: "C:/pick", error: null });

    const ok = await browseSettingsDataRoot("C:/start");
    expect(invokeMock).toHaveBeenCalledWith("browse_settings_data_root", { startDir: "C:/start" });
    expect(ok).toEqual({ source: "rust", path: "C:/pick", error: null });

    mockReject(new Error("picker failed"));
    const bad = await browseSettingsDataRoot("C:/start");
    expect(bad.source).toBe("mock");
    expect(bad.path).toBeNull();
    expect(bad.error).toContain("picker failed");
  });

  it("browseSettingsDataRoot surfaces a Rust-supplied error", async () => {
    invokeMock.mockResolvedValue({ path: null, error: "cancelled" });

    const result = await browseSettingsDataRoot("C:/start");

    expect(result.source).toBe("rust");
    expect(result.path).toBeNull();
    expect(result.error).toBe("cancelled");
  });

  it("getGoogleApiKey returns the key from Rust with the exact command name", async () => {
    invokeMock.mockResolvedValue("AIza-key");

    const result = await getGoogleApiKey();

    expect(invokeMock).toHaveBeenCalledWith("get_google_api_key");
    expect(result.source).toBe("rust");
    expect(result.key).toBe("AIza-key");
  });

  it("getGoogleApiKey returns an empty key when Rust has none", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getGoogleApiKey();

    expect(result.source).toBe("rust");
    expect(result.key).toBe("");
  });

  it("getGoogleApiKey falls back to mock empty key on error", async () => {
    mockReject(new Error("key failed"));

    const result = await getGoogleApiKey();

    expect(result.source).toBe("mock");
    expect(result.key).toBe("");
    expect(result.error).toContain("key failed");
  });

  it("setGoogleApiKey sends the camelCase apiKey wire key and trims the value", async () => {
    invokeMock.mockResolvedValue(true);

    const result = await setGoogleApiKey("  AIza-trimmed  ");

    expect(invokeMock).toHaveBeenCalledWith("set_google_api_key", {
      apiKey: "AIza-trimmed",
    });
    expect(result).toEqual({ source: "rust", persisted: true });
  });

  it("setGoogleApiKey persists an empty string to clear the key", async () => {
    invokeMock.mockResolvedValue(true);

    const result = await setGoogleApiKey("");

    expect(invokeMock).toHaveBeenCalledWith("set_google_api_key", { apiKey: "" });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
  });

  it("setGoogleApiKey falls back to mock on error", async () => {
    mockReject(new Error("save key failed"));

    const result = await setGoogleApiKey("key");

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("save key failed");
  });

  it("getTaggingActionsViewModel maps the Rust view model", async () => {
    invokeMock.mockResolvedValue({
      has_google_api_key: true,
      ai_tier2_auto: true,
      ai_tier3_auto: false,
      ai_batch_size: "10",
      ai_delay: "20",
      import_commit_batch_size: "30",
      default_batch_size: 50,
      default_commit_every: 60,
      default_workers: 2,
    });

    const result = await getTaggingActionsViewModel();

    expect(result.source).toBe("rust");
    expect(result.model.default_batch_size).toBe(50);
    expect(result.model.default_workers).toBe(2);
  });

  it("getTaggingActionsViewModel falls back to the default mock model on error", async () => {
    mockReject(new Error("vm failed"));

    const result = await getTaggingActionsViewModel();

    expect(result.source).toBe("mock");
    expect(result.model).toEqual({
      has_google_api_key: false,
      ai_tier2_auto: false,
      ai_tier3_auto: false,
      ai_batch_size: "",
      ai_delay: "",
      import_commit_batch_size: "",
      default_batch_size: 100,
      default_commit_every: 100,
      default_workers: 4,
    });
    expect(result.error).toContain("vm failed");
  });
});

describe("commandAdapter runUnifiedBackfill edge cases", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps a tier3-only tagging run", async () => {
    invokeMock.mockResolvedValue({
      processed: 3,
      errors: 0,
      stopped: false,
      actions: ["tag"],
      commit_every: 100,
      batch_size: 100,
      workers: 4,
    });

    const result = await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_tier2: false,
      run_tier3: true,
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
          tagging: { action: "tag_untagged", tiers: [1, 3], enabled: true },
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
    expect(result.source).toBe("rust");
    expect(result.processed).toBe(3);
  });

  it("maps a color-counts-only run with no tagging or images", async () => {
    invokeMock.mockResolvedValue({});

    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_tier2: false,
      run_tier3: false,
      run_images: false,
      image_redo: false,
      run_color_counts: true,
      commit_every: 1,
      batch_size: 2,
      workers: 3,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: null,
          stitching: null,
          images: null,
          color_counts: { enabled: true },
          fingerprinting: null,
        },
        batch_size: 2,
        commit_every: 1,
        workers: 3,
      },
    });
  });

  it("applies defaults when numeric fields are missing", async () => {
    invokeMock.mockResolvedValue({});

    await runUnifiedBackfill({
      action_mode: "tag_untagged",
      run_tier2: false,
      run_tier3: false,
      run_images: false,
      image_redo: false,
      run_color_counts: false,
      commit_every: null as unknown as number,
      batch_size: null as unknown as number,
      workers: null as unknown as number,
    });

    expect(invokeMock).toHaveBeenCalledWith("run_unified_backfill", {
      request: {
        actions: {
          tagging: null,
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

  it("falls back to a mock error result when the command fails", async () => {
    mockReject(new Error("backfill failed"));

    const result = await runUnifiedBackfill({
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

    expect(result.source).toBe("mock");
    expect(result.processed).toBe(0);
    expect(result.errors).toBe(1);
    expect(result.error).toContain("backfill failed");
  });
});

describe("commandAdapter stopUnifiedBackfill", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps a Rust status", async () => {
    invokeMock.mockResolvedValue({ status: "stopped" });

    const result = await stopUnifiedBackfill();

    expect(result).toEqual({ source: "rust", status: "stopped" });
  });

  it("falls back to mock stopping on error", async () => {
    mockReject(new Error("x"));

    const result = await stopUnifiedBackfill();

    expect(result.source).toBe("mock");
    expect(result.status).toBe("stopping");
    expect(result.error).toContain("x");
  });
});

describe("commandAdapter getBackfillLogEntries", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps Rust log entries", async () => {
    invokeMock.mockResolvedValue([
      { level: "info", message: "started" },
      { level: "warn", message: "slow" },
      {},
    ]);

    const result = await getBackfillLogEntries(50);

    expect(invokeMock).toHaveBeenCalledWith("get_backfill_log_entries", { limit: 50 });
    expect(result.source).toBe("rust");
    expect(result.entries).toEqual([
      { level: "info", message: "started" },
      { level: "warn", message: "slow" },
      { level: "info", message: "" },
    ]);
  });

  it("falls back to a single error entry when the command fails", async () => {
    mockReject(new Error("log failed"));

    const result = await getBackfillLogEntries();

    expect(result.source).toBe("mock");
    expect(result.entries).toEqual([{ level: "error", message: "Error: log failed" }]);
  });

  it("returns an empty mock list when the payload is not an array", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await getBackfillLogEntries();

    expect(result).toEqual({ source: "mock", entries: [] });
  });
});

describe("commandAdapter backups", () => {
  beforeEach(() => invokeMock.mockReset());

  const backupModel = {
    db_destination: "C:/backup/db",
    designs_destination: "C:/backup/designs",
    db_source_path: "C:/data/db.sqlite",
    designs_source_path: "C:/data/embroidery",
  };

  it("getBackupViewModel maps the Rust model and falls back on error", async () => {
    invokeMock.mockResolvedValue(backupModel);

    const ok = await getBackupViewModel();
    expect(ok.source).toBe("rust");
    expect(ok.model.db_destination).toBe("C:/backup/db");

    mockReject(new Error("backup vm failed"));
    const bad = await getBackupViewModel();
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("backup vm failed");
  });

  it("saveBackupSettings maps the Rust result", async () => {
    invokeMock.mockResolvedValue({
      saved: true,
      message: "Saved.",
      db_destination: "C:/db",
      designs_destination: "C:/des",
    });

    const result = await saveBackupSettings({ dbDestination: "C:/db", designsDestination: "C:/des" });

    expect(invokeMock).toHaveBeenCalledWith("save_backup_settings", {
      request: { db_destination: "C:/db", designs_destination: "C:/des" },
    });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
    expect(result.saved).toBe(true);
    expect(result.db_destination).toBe("C:/db");
  });

  it("saveBackupSettings falls back to mock on error", async () => {
    mockReject(new Error("save backup failed"));

    const result = await saveBackupSettings({ dbDestination: "C:/db", designsDestination: "C:/des" });

    expect(result.source).toBe("mock");
    expect(result.saved).toBe(false);
    expect(result.error).toContain("save backup failed");
  });

  it("browseBackupFolder maps Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ path: "C:/pick", error: null });

    const ok = await browseBackupFolder("C:/start");
    expect(invokeMock).toHaveBeenCalledWith("browse_backup_folder", { startDir: "C:/start" });
    expect(ok).toEqual({ source: "rust", path: "C:/pick", error: null });

    mockReject(new Error("picker failed"));
    const bad = await browseBackupFolder("C:/start");
    expect(bad.source).toBe("mock");
    expect(bad.path).toBeNull();
    expect(bad.error).toContain("picker failed");
  });

  it("browseBackupFolder treats null startDir as null", async () => {
    invokeMock.mockResolvedValue({ path: null, error: null });

    await browseBackupFolder();

    expect(invokeMock).toHaveBeenCalledWith("browse_backup_folder", { startDir: null });
  });

  it("runDatabaseBackup maps the Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({
      success: true,
      backup_path: "C:/bak/db.sqlite",
      size_bytes: 100,
      completed_at: "2026",
      error: "",
    });

    const ok = await runDatabaseBackup();
    expect(ok.source).toBe("rust");
    expect(ok.success).toBe(true);
    expect(ok.backup_path).toBe("C:/bak/db.sqlite");

    mockReject(new Error("backup failed"));
    const bad = await runDatabaseBackup();
    expect(bad.source).toBe("mock");
    expect(bad.success).toBe(false);
    expect(bad.error).toContain("backup failed");
  });

  it("runDesignsBackup maps the Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({
      success: true,
      scanned: 10,
      copied: 5,
      updated: 2,
      unchanged: 3,
      archived: 1,
      total_bytes_copied: 999,
      completed_at: "2026",
      error: "",
    });

    const ok = await runDesignsBackup();
    expect(ok.source).toBe("rust");
    expect(ok.copied).toBe(5);
    expect(ok.total_bytes_copied).toBe(999);

    mockReject(new Error("designs backup failed"));
    const bad = await runDesignsBackup();
    expect(bad.source).toBe("mock");
    expect(bad.success).toBe(false);
    expect(bad.error).toContain("designs backup failed");
  });

  it("runBothBackups maps database and designs results", async () => {
    invokeMock.mockResolvedValue({
      database: { success: true, backup_path: "C:/db", size_bytes: 1, completed_at: "", error: "" },
      designs: { success: true, scanned: 1, copied: 1, updated: 0, unchanged: 0, archived: 0, total_bytes_copied: 1, completed_at: "", error: "" },
    });

    const result = await runBothBackups();

    expect(result.source).toBe("rust");
    expect(result.database?.success).toBe(true);
    expect(result.designs?.scanned).toBe(1);
  });

  it("runBothBackups falls back to null results on error", async () => {
    mockReject(new Error("both failed"));

    const result = await runBothBackups();

    expect(result.source).toBe("mock");
    expect(result.database).toBeNull();
    expect(result.designs).toBeNull();
    expect(result.error).toContain("both failed");
  });
});

describe("commandAdapter scanOrphans", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps a Rust scan result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ checked: 10, found: 2 });

    const ok = await scanOrphans();
    expect(ok).toEqual({ source: "rust", checked: 10, found: 2 });

    mockReject(new Error("scan failed"));
    const bad = await scanOrphans();
    expect(bad.source).toBe("mock");
    expect(bad.checked).toBe(0);
    expect(bad.found).toBe(0);
    expect(bad.error).toContain("scan failed");
  });
});

describe("commandAdapter getDbStats", () => {
  beforeEach(() => invokeMock.mockReset());

  const stats = {
    file_size_bytes: 1000,
    page_count: 50,
    freelist_count: 10,
    page_size: 4096,
    free_ratio: 0.2,
    reclaimable_bytes: 200,
  };

  it("maps a Rust stats payload and falls back to zero stats on error", async () => {
    invokeMock.mockResolvedValue(stats);

    const ok = await getDbStats();
    expect(ok.source).toBe("rust");
    expect(ok.stats).toEqual(stats);

    mockReject(new Error("stats failed"));
    const bad = await getDbStats();
    expect(bad.source).toBe("mock");
    expect(bad.stats).toEqual({
      file_size_bytes: 0,
      page_count: 0,
      freelist_count: 0,
      page_size: 0,
      free_ratio: 0,
      reclaimable_bytes: 0,
    });
    expect(bad.error).toContain("stats failed");
  });
});

describe("commandAdapter compactDatabase", () => {
  beforeEach(() => invokeMock.mockReset());

  const compactResult = {
    file_size_before: 5000,
    file_size_after: 1000,
    pages_reclaimed: 80,
    duration_ms: 12,
  };

  it("maps a Rust compact result and falls back on error", async () => {
    invokeMock.mockResolvedValue(compactResult);

    const ok = await compactDatabase();
    expect(ok.source).toBe("rust");
    expect(ok.result).toEqual(compactResult);
    expect(ok.message).toContain("compacted");

    mockReject(new Error("compact failed"));
    const bad = await compactDatabase();
    expect(bad.source).toBe("mock");
    expect(bad.result).toBeNull();
    expect(bad.error).toContain("compact failed");
  });
});

describe("commandAdapter orphans pages & actions", () => {
  beforeEach(() => invokeMock.mockReset());

  it("getOrphansPage normalizes page/pageSize and maps Rust rows", async () => {
    invokeMock.mockResolvedValue({
      page: 2,
      page_size: 25,
      total: 60,
      total_pages: 3,
      items: [
        { id: 1, filename: "a.pes", filepath: "C:/a.pes", designer: "D", date_added: "2026" },
        { id: 2, filename: "b.pes", filepath: "C:/b.pes", designer: "", date_added: null },
      ],
    });

    const result = await getOrphansPage({ page: 2, pageSize: 25 });

    expect(invokeMock).toHaveBeenCalledWith("get_orphans_page", {
      request: { page: 2, page_size: 25 },
    });
    expect(result.source).toBe("rust");
    expect(result.items[1].date_added).toBeNull();
  });

  it("getOrphansPage clamps invalid page and pageSize to 1", async () => {
    invokeMock.mockResolvedValue({});

    await getOrphansPage({ page: 0, pageSize: 0 });

    expect(invokeMock).toHaveBeenCalledWith("get_orphans_page", {
      request: { page: 1, page_size: 1 },
    });
  });

  it("getOrphansPage falls back to an empty mock page on error", async () => {
    mockReject(new Error("page failed"));

    const result = await getOrphansPage();

    expect(result.source).toBe("mock");
    expect(result.items).toEqual([]);
    expect(result.total).toBe(0);
    expect(result.error).toContain("page failed");
  });

  it("deleteOrphans filters ids and maps Rust result", async () => {
    invokeMock.mockResolvedValue({ deleted: 2 });

    const result = await deleteOrphans(["1", 2, 0, "x"]);

    expect(invokeMock).toHaveBeenCalledWith("delete_orphans", {
      request: { design_ids: [1, 2] },
    });
    expect(result).toEqual({ source: "rust", persisted: true, deleted: 2 });
  });

  it("deleteOrphans falls back to mock on error", async () => {
    mockReject(new Error("delete failed"));

    const result = await deleteOrphans([1]);

    expect(result.source).toBe("mock");
    expect(result.deleted).toBe(0);
    expect(result.error).toContain("delete failed");
  });

  it("deleteAllOrphans maps Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ deleted: 5 });

    const ok = await deleteAllOrphans();
    expect(ok).toEqual({ source: "rust", persisted: true, deleted: 5 });

    mockReject(new Error("delete all failed"));
    const bad = await deleteAllOrphans();
    expect(bad.source).toBe("mock");
    expect(bad.deleted).toBe(0);
    expect(bad.error).toContain("delete all failed");
  });

  it("browseOrphanPath maps Rust result and falls back on error", async () => {
    invokeMock.mockResolvedValue({ ok: true, opened: "C:/x" });

    const okResult = await browseOrphanPath("C:/x");
    expect(invokeMock).toHaveBeenCalledWith("browse_orphan_path", { filepath: "C:/x" });
    expect(okResult).toEqual({ source: "rust", ok: true, opened: "C:/x" });

    mockReject(new Error("browse failed"));
    const bad = await browseOrphanPath("C:/x");
    expect(bad.source).toBe("mock");
    expect(bad.ok).toBe(false);
    expect(bad.error).toContain("browse failed");
  });
});

describe("commandAdapter admin designers", () => {
  beforeEach(() => invokeMock.mockReset());

  const summary = { id: 1, name: "Amazing Designs", design_count: 4 };

  it("listDesigners returns Rust items and a mock fallback", async () => {
    invokeMock.mockResolvedValue([summary]);

    const ok = await listDesigners();
    expect(ok.source).toBe("rust");
    expect(ok.items[0]).toEqual(summary);

    mockReject(new Error("list failed"));
    const bad = await listDesigners();
    expect(bad.source).toBe("mock");
    expect(bad.items).toHaveLength(3);
  });

  it("createDesigner sends request name and maps item", async () => {
    invokeMock.mockResolvedValue(summary);

    const result = await createDesigner("Amazing Designs");

    expect(invokeMock).toHaveBeenCalledWith("create_designer", { request: { name: "Amazing Designs" } });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
    expect(result.item).toEqual(summary);
  });

  it("createDesigner falls back to mock on error", async () => {
    mockReject(new Error("create failed"));

    const result = await createDesigner("Designer");

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
    expect(result.error).toContain("create failed");
  });

  it("updateDesigner sends designer_id and name, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await updateDesigner(1, "New Name");
    expect(invokeMock).toHaveBeenCalledWith("update_designer", {
      request: { designer_id: 1, name: "New Name" },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("update failed"));
    const bad = await updateDesigner(1, "New Name");
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("update failed");
  });

  it("deleteDesigner sends designerId and falls back on error", async () => {
    invokeMock.mockResolvedValue(undefined);

    const okResult = await deleteDesigner(3);
    expect(invokeMock).toHaveBeenCalledWith("delete_designer", { designerId: 3 });
    expect(okResult).toEqual({ source: "rust", persisted: true });

    mockReject(new Error("delete failed"));
    const bad = await deleteDesigner(3);
    expect(bad.source).toBe("mock");
    expect(bad.persisted).toBe(false);
    expect(bad.error).toContain("delete failed");
  });
});

describe("commandAdapter admin sources", () => {
  beforeEach(() => invokeMock.mockReset());

  const summary = { id: 1, name: "Purchased", design_count: 2 };

  it("listSources returns Rust items and a mock fallback", async () => {
    invokeMock.mockResolvedValue([summary]);

    const ok = await listSources();
    expect(ok.source).toBe("rust");
    expect(ok.items[0].design_count).toBe(2);

    mockReject(new Error("list failed"));
    const bad = await listSources();
    expect(bad.source).toBe("mock");
    expect(bad.items).toHaveLength(3);
  });

  it("createSource sends request name and maps item", async () => {
    invokeMock.mockResolvedValue(summary);

    const result = await createSource("Purchased");

    expect(invokeMock).toHaveBeenCalledWith("create_source", { request: { name: "Purchased" } });
    expect(result.source).toBe("rust");
    expect(result.persisted).toBe(true);
  });

  it("createSource falls back to mock on error", async () => {
    mockReject(new Error("create failed"));

    const result = await createSource("X");

    expect(result.source).toBe("mock");
    expect(result.persisted).toBe(false);
  });

  it("updateSource sends source_id and name, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await updateSource(1, "New Source");
    expect(invokeMock).toHaveBeenCalledWith("update_source", {
      request: { source_id: 1, name: "New Source" },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("update failed"));
    const bad = await updateSource(1, "New Source");
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("update failed");
  });

  it("deleteSource sends sourceId and falls back on error", async () => {
    invokeMock.mockResolvedValue(undefined);

    const okResult = await deleteSource(2);
    expect(invokeMock).toHaveBeenCalledWith("delete_source", { sourceId: 2 });
    expect(okResult).toEqual({ source: "rust", persisted: true });

    mockReject(new Error("delete failed"));
    const bad = await deleteSource(2);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("delete failed");
  });
});

describe("commandAdapter admin tags", () => {
  beforeEach(() => invokeMock.mockReset());

  const summary = { id: 1, description: "Floral", tag_group: "Theme", design_count: 3, is_system: false };

  it("listTags returns Rust items and a mock fallback", async () => {
    invokeMock.mockResolvedValue([summary]);

    const ok = await listTags();
    expect(ok.source).toBe("rust");
    expect(ok.items[0]).toEqual(summary);

    mockReject(new Error("list failed"));
    const bad = await listTags();
    expect(bad.source).toBe("mock");
    expect(bad.items).toEqual([]);
    expect(bad.error).toContain("list failed");
  });

  it("listTags reports an unexpected payload", async () => {
    invokeMock.mockResolvedValue(null);

    const result = await listTags();

    expect(result.source).toBe("rust");
    expect(result.items).toEqual([]);
    expect(result.error).toContain("unexpected payload");
  });

  it("createTag sends description and tag_group, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await createTag("Floral", "Theme");
    expect(invokeMock).toHaveBeenCalledWith("create_tag", {
      request: { description: "Floral", tag_group: "Theme" },
    });
    expect(ok.persisted).toBe(true);
    expect(ok.item?.tag_group).toBe("Theme");

    mockReject(new Error("create failed"));
    const bad = await createTag("Floral", null);
    expect(bad.source).toBe("mock");
    expect(bad.persisted).toBe(false);
  });

  it("setTagGroup sends tag_id and tag_group, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await setTagGroup(1, "New Group");
    expect(invokeMock).toHaveBeenCalledWith("set_tag_group", {
      request: { tag_id: 1, tag_group: "New Group" },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("set failed"));
    const bad = await setTagGroup(1, null);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("set failed");
  });

  it("updateTag sends tag_id and description, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await updateTag(1, "Updated");
    expect(invokeMock).toHaveBeenCalledWith("update_tag", {
      request: { tag_id: 1, description: "Updated" },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("update failed"));
    const bad = await updateTag(1, "Updated");
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("update failed");
  });

  it("deleteTag sends tagId and falls back on error", async () => {
    invokeMock.mockResolvedValue(undefined);

    const okResult = await deleteTag(4);
    expect(invokeMock).toHaveBeenCalledWith("delete_tag", { tagId: 4 });
    expect(okResult).toEqual({ source: "rust", persisted: true });

    mockReject(new Error("delete failed"));
    const bad = await deleteTag(4);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("delete failed");
  });
});

describe("commandAdapter admin hoops", () => {
  beforeEach(() => invokeMock.mockReset());

  const summary = { id: 1, name: "4x4 hoop", max_width_mm: 100, max_height_mm: 100, design_count: 0 };

  it("listHoops returns Rust items and a mock fallback", async () => {
    invokeMock.mockResolvedValue([summary]);

    const ok = await listHoops();
    expect(ok.source).toBe("rust");
    expect(ok.items[0].max_width_mm).toBe(100);

    mockReject(new Error("list failed"));
    const bad = await listHoops();
    expect(bad.source).toBe("mock");
    expect(bad.items).toHaveLength(3);
  });

  it("createHoop sends the request and maps the item, falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await createHoop("4x4 hoop", 100, 100);
    expect(invokeMock).toHaveBeenCalledWith("create_hoop", {
      request: { name: "4x4 hoop", max_width_mm: 100, max_height_mm: 100 },
    });
    expect(ok.persisted).toBe(true);
    expect(ok.item?.max_height_mm).toBe(100);

    mockReject(new Error("create failed"));
    const bad = await createHoop("x", 1, 1);
    expect(bad.source).toBe("mock");
    expect(bad.persisted).toBe(false);
  });

  it("updateHoop sends the request and falls back on error", async () => {
    invokeMock.mockResolvedValue(summary);

    const ok = await updateHoop(1, "New", 50, 60);
    expect(invokeMock).toHaveBeenCalledWith("update_hoop", {
      request: { hoop_id: 1, name: "New", max_width_mm: 50, max_height_mm: 60 },
    });
    expect(ok.persisted).toBe(true);

    mockReject(new Error("update failed"));
    const bad = await updateHoop(1, "New", 50, 60);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("update failed");
  });

  it("deleteHoop sends hoopId and falls back on error", async () => {
    invokeMock.mockResolvedValue(undefined);

    const okResult = await deleteHoop(2);
    expect(invokeMock).toHaveBeenCalledWith("delete_hoop", { hoopId: 2 });
    expect(okResult).toEqual({ source: "rust", persisted: true });

    mockReject(new Error("delete failed"));
    const bad = await deleteHoop(2);
    expect(bad.source).toBe("mock");
    expect(bad.error).toContain("delete failed");
  });
});

describe("commandAdapter runStitchingBackfill additional cases", () => {
  beforeEach(() => invokeMock.mockReset());

  it("applies default options when called with defaults", async () => {
    invokeMock.mockResolvedValue({});

    await runStitchingBackfill();

    expect(invokeMock).toHaveBeenCalledWith("run_stitching_backfill", {
      clearStitchingMode: "none",
      batchSize: 100,
    });
  });

  it("maps a Rust result with actions", async () => {
    invokeMock.mockResolvedValue({
      processed: 7,
      errors: 1,
      stopped: false,
      actions: ["stitching", "color_counts"],
    });

    const result = await runStitchingBackfill({ clear_stitching_mode: "unverified", batch_size: 10 });

    expect(result.source).toBe("rust");
    expect(result.processed).toBe(7);
    expect(result.errors).toBe(1);
    expect(result.actions).toEqual(["stitching", "color_counts"]);
  });

  it("falls back to a mock error result", async () => {
    mockReject(new Error("stitching failed"));

    const result = await runStitchingBackfill();

    expect(result.source).toBe("mock");
    expect(result.processed).toBe(0);
    expect(result.errors).toBe(1);
    expect(result.actions).toEqual(["stitching"]);
    expect(result.error).toContain("stitching failed");
  });
});

describe("commandAdapter initial setup & app status", () => {
  beforeEach(() => invokeMock.mockReset());

  it("checkInitialSetup returns the Rust boolean", async () => {
    invokeMock.mockResolvedValue(false);

    const result = await checkInitialSetup();

    expect(invokeMock).toHaveBeenCalledWith("check_initial_setup");
    expect(result).toBe(false);
  });

  it("checkInitialSetup defaults to true on error", async () => {
    mockReject(new Error("setup check failed"));

    const result = await checkInitialSetup();

    expect(result).toBe(true);
  });

  it("completeInitialSetup invokes the command and swallows errors", async () => {
    invokeMock.mockResolvedValue(undefined);

    await completeInitialSetup();
    expect(invokeMock).toHaveBeenCalledWith("complete_initial_setup");

    mockReject(new Error("complete failed"));
    await expect(completeInitialSetup()).resolves.toBeUndefined();
  });

  it("getAppStatus maps an installed status with the data_root_missing flag", async () => {
    invokeMock.mockResolvedValue(APP_STATUS);

    const result = await getAppStatus();

    expect(result.source).toBe("rust");
    expect(result.status).toEqual({
      execution_mode: "installed",
      data_root: "C:/data",
      embroidery_dir: "C:/data/embroidery",
      database_path: "C:/data/catalogue.db",
      data_root_missing: false,
      database_missing: false,
    });
  });

  it("getAppStatus normalizes unknown execution modes to installed", async () => {
    invokeMock.mockResolvedValue({ ...APP_STATUS, execution_mode: "weird" });

    const result = await getAppStatus();

    expect(result.source).toBe("rust");
    expect(result.status?.execution_mode).toBe("installed");
  });

  it("getAppStatus falls back to a null status on error", async () => {
    mockReject(new Error("status failed"));

    const result = await getAppStatus();

    expect(result.source).toBe("mock");
    expect(result.status).toBeNull();
    expect(result.error).toContain("not available");
  });

  it("startCatalogueStorageMigration invokes with camelCase targetDir and force keys", async () => {
    invokeMock.mockResolvedValue({
      success: true,
      source_root: "D:/data",
      target_root: "E:/new",
      database_bytes: 10,
      asset_items: 3,
      asset_bytes: 40,
      requires_restart: true,
    });

    const result = await startCatalogueStorageMigration("E:/new");

    // The exact camelCase wire keys Tauri expects for `target_dir` and `force`.
    expect(invokeMock).toHaveBeenCalledWith("start_catalogue_storage_migration", {
      targetDir: "E:/new",
      force: true,
    });
    expect(result.source).toBe("rust");
    expect(result.summary?.success).toBe(true);
    expect(result.summary?.target_root).toBe("E:/new");
  });

  it("startCatalogueStorageMigration rejects an empty target without invoking", async () => {
    const result = await startCatalogueStorageMigration("   ");
    expect(result).toEqual({
      source: "mock",
      summary: null,
      error: "Data root cannot be empty.",
    });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("startCatalogueStorageMigration falls back to mock on error", async () => {
    mockReject(new Error("preflight failed"));

    const result = await startCatalogueStorageMigration("E:/new");

    expect(result.source).toBe("mock");
    expect(result.summary).toBeNull();
    expect(result.error).toContain("preflight failed");
  });

  it("cancelCatalogueStorageMigration invokes the command and maps success", async () => {
    invokeMock.mockResolvedValue(undefined);

    const result = await cancelCatalogueStorageMigration();

    expect(invokeMock).toHaveBeenCalledWith("cancel_catalogue_storage_migration");
    expect(result).toEqual({ source: "rust", cancelled: true });
  });

  it("cancelCatalogueStorageMigration falls back to mock on error", async () => {
    mockReject(new Error("cancel failed"));

    const result = await cancelCatalogueStorageMigration();

    expect(result.source).toBe("mock");
    expect(result.cancelled).toBe(false);
    expect(result.error).toContain("cancel failed");
  });

  it("configureFreshDataRoot invokes with the camelCase dataRoot key and maps result", async () => {
    invokeMock.mockResolvedValue({
      data_root: "F:/FreshData",
      existing_database_detected: true,
      database_path: "F:/FreshData/Database/EmbroideryCatalogue.db",
    });

    const result = await configureFreshDataRoot("F:/FreshData");

    // The exact camelCase wire key Tauri expects to map to `data_root`.
    expect(invokeMock).toHaveBeenCalledWith("configure_fresh_data_root", {
      dataRoot: "F:/FreshData",
    });
    expect(result).toEqual({
      source: "rust",
      persisted: true,
      data_root: "F:/FreshData",
      existing_database_detected: true,
      database_path: "F:/FreshData/Database/EmbroideryCatalogue.db",
    });
  });

  it("configureFreshDataRoot rejects an empty data root without invoking", async () => {
    const empty = await configureFreshDataRoot("   ");
    expect(empty).toEqual({
      source: "mock",
      persisted: false,
      error: "Data root cannot be empty.",
    });
    expect(invokeMock).not.toHaveBeenCalled();

    const missing = await configureFreshDataRoot("");
    expect(missing.persisted).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("configureFreshDataRoot falls back to mock on error", async () => {
    mockReject(new Error("config write failed"));

    const result = await configureFreshDataRoot("F:/FreshData");

    expect(result).toEqual({
      source: "mock",
      persisted: false,
      error: expect.stringContaining("config write failed"),
    });
  });
});
