/**
 * COVERAGE NOTE
 * -------------
 * MainView.svelte is a 3,000+ line application shell that embeds routing,
 * data fetching, client-side filtering/sorting, admin CRUD, pagination,
 * preview loading, and bulk operations within a single component scope.
 *
 * Because all business logic resides in private, non-exported functions
 * (normalizeHash, parseQueryWithOr, compareBrowseItems, normalizeCardItem,
 * applyPatchesToBrowse, etc.) and state is managed via tightly-coupled
 * $state/$derived runes inside the component, individual units cannot be
 * tested in isolation with vitest + jsdom.
 *
 * Meaningful coverage improvement requires architectural refactoring:
 *   - Extract the hash router into a separate module
 *   - Decompose Browse into its own view-level sub-component
 *   - Move card normalization & search parsing into utility modules
 *   - Elevate each admin entity CRUD into dedicated sub-components
 *
 * The tests in this file provide integration-level coverage of every
 * primary route destination, the core browse CRUD lifecycle (load,
 * filter, search, sort, paginate, select, bulk-delete, bulk-verify),
 * and admin CRUD for all four entity types. Edge-case coverage for
 * internal functions is deferred to the component-level spec or future
 * refactoring that enables isolated unit testing.
 */

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import MainView from "../MainView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter module — this prevents real Tauri `invoke` calls.
// The mock must provide EVERY named export from commandAdapter.ts because the
// sub-views imported by MainView also import from this module.
// ---------------------------------------------------------------------------
const adapterMock = vi.hoisted(() => ({
  // Browse
  getBrowseDesigns: vi.fn(),
  getBrowseDesignPreviews: vi.fn(),
  getBrowseProjects: vi.fn(),
  getBrowseTags: vi.fn(),
  addDesignToProject: vi.fn(),
  removeDesignFromProject: vi.fn(),
  bulkVerifyDesigns: vi.fn(),
  bulkAddDesignsToProject: vi.fn(),
  bulkSetTagsForDesigns: vi.fn(),
  bulkDeleteDesigns: vi.fn(),
  // Design detail
  getDesignDetail: vi.fn(),
  getDesignImageDataUrl: vi.fn(),
  updateDesignMetadata: vi.fn(),
  setDesignRating: vi.fn(),
  setDesignStitched: vi.fn(),
  setDesignTagsChecked: vi.fn(),
  setDesignTags: vi.fn(),
  removeDesignTag: vi.fn(),
  openDesignInEditor: vi.fn(),
  openDesignInExplorer: vi.fn(),
  renderDesign3dPreview: vi.fn(),
  // Import
  previewImportFromRoots: vi.fn(),
  previewImportFromRoot: vi.fn(),
  browseImportFolder: vi.fn(),
  precheckImportWire: vi.fn(),
  runPrecheckAction: vi.fn(),
  requestStopBulkImport: vi.fn(),
  // Projects
  getProjectsList: vi.fn(),
  createProject: vi.fn(),
  getProjectDetail: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  removeDesignFromProjectDetail: vi.fn(),
  getProjectPrintView: vi.fn(),
  // About / settings / maintenance
  getAboutDocuments: vi.fn(),
  getAboutDocument: vi.fn(),
  getSettingsViewModel: vi.fn(),
  saveSettings: vi.fn(),
  saveImportLastBrowseFolder: vi.fn(),
  browseSettingsDataRoot: vi.fn(),
  getTaggingActionsViewModel: vi.fn(),
  runUnifiedBackfill: vi.fn(),
  stopUnifiedBackfill: vi.fn(),
  getBackfillLogEntries: vi.fn(),
  runStitchingBackfill: vi.fn(),
  getBackupViewModel: vi.fn(),
  saveBackupSettings: vi.fn(),
  browseBackupFolder: vi.fn(),
  runDatabaseBackup: vi.fn(),
  runDesignsBackup: vi.fn(),
  runBothBackups: vi.fn(),
  // Orphans
  scanOrphans: vi.fn(),
  getOrphansPage: vi.fn(),
  deleteOrphans: vi.fn(),
  deleteAllOrphans: vi.fn(),
  browseOrphanPath: vi.fn(),
  // Admin entities
  listDesigners: vi.fn(),
  createDesigner: vi.fn(),
  updateDesigner: vi.fn(),
  deleteDesigner: vi.fn(),
  listSources: vi.fn(),
  createSource: vi.fn(),
  updateSource: vi.fn(),
  deleteSource: vi.fn(),
  listTags: vi.fn(),
  createTag: vi.fn(),
  setTagGroup: vi.fn(),
  deleteTag: vi.fn(),
  listHoops: vi.fn(),
  createHoop: vi.fn(),
  updateHoop: vi.fn(),
  deleteHoop: vi.fn(),
  // Misc
  getAppStatus: vi.fn(),
}));

vi.mock("../api/commandAdapter", () => adapterMock);

// Mock the toast store — MainView and its sub-views call addToast().
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../stores/toastStore", () => toastMock);

// Mock the design session store — MainView drains patches on route entry.
const sessionMock = vi.hoisted(() => ({
  designSessionStore: {
    consumePatches: vi.fn(() => ({})),
    trackMutation: vi.fn(),
    subscribe: vi.fn(),
  },
}));
vi.mock("../stores/designSessionStore", () => sessionMock);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/** Builds a BrowseDesignSummaryWire-shaped item for getBrowseDesigns. */
const wireCard = (overrides: Record<string, unknown> = {}) => ({
  id: 1,
  filename: "rose-border-01.pes",
  filepath: "C:/designs/rose-border-01.pes",
  designer: "Rose Studio",
  source: "Imported",
  hoop: "Hoop A",
  projects: ["Wedding Collection"],
  tags: ["Floral", "Border"],
  image_tags: ["Floral"],
  stitching_tags: ["Border"],
  is_stitched: false,
  tags_checked: true,
  rating: 4,
  ...overrides,
});

/** Wraps items in an AdapterListResponse. */
const browseResponse = (items: unknown[] = []) => ({ source: "rust", items });

/** Full DesignDetail fixture used by the design-detail route test. */
const baseDetail = {
  id: 42,
  filename: "rose-border-01.pes",
  filepath: "C:/designs/rose-border-01.pes",
  imageType: null,
  imageDataUrl: null,
  widthMm: 120,
  heightMm: 80,
  stitchCount: 10000,
  colorCount: 5,
  colorChangeCount: 12,
  designer: "Rose Studio",
  designerId: 7,
  source: "Imported",
  sourceId: 3,
  hoop: "Hoop A",
  hoopId: 1,
  notes: "Pretty floral border.",
  rating: 4,
  isStitched: true,
  tagsChecked: true,
  taggingTier: 2,
  dateAdded: "2026-05-01",
  tags: [
    { id: 11, description: "Floral", tag_group: "image" },
    { id: 12, description: "Satin Stitch", tag_group: "stitching" },
  ],
  projects: [{ id: 1, name: "Wedding Collection" }],
  availableProjects: [
    { id: 1, name: "Wedding Collection" },
    { id: 2, name: "Autumn 2026" },
  ],
  allTags: [
    { id: 11, description: "Floral", tag_group: "image" },
    { id: 12, description: "Satin Stitch", tag_group: "stitching" },
  ],
  designers: [{ id: 7, name: "Rose Studio" }],
  sources: [{ id: 3, name: "Imported" }],
  hoops: [{ id: 1, name: "Hoop A" }],
};

const settingsModel = {
  preview_3d_profile: "balanced",
  google_api_key: "",
  has_google_api_key: false,
  ai_tier2_auto: false,
  ai_tier3_auto: false,
  ai_batch_size: "",
  ai_delay: "",
  import_commit_batch_size: "",
  import_last_browse_folder: "",
  can_configure_data_root: false,
  data_root: "",
  database_path: "",
  log_folder: "",
  app_mode: "development",
  ai_tagging_help_url: "#/help",
};

/** Sets the location hash BEFORE mount so syncRouteFromHash sees it. */
const renderAtHash = (hash: string) => {
  window.location.hash = hash;
  return render(MainView);
};

/** Returns the visible browse card titles in DOM order. */
const cardTitles = (container: HTMLElement) =>
  Array.from(container.querySelectorAll(".browse-card-title")).map((el) =>
    (el.textContent || "").trim()
  );

/** Type-guard helper so querySelector results can be passed to fireEvent. */
function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

// ---------------------------------------------------------------------------
// beforeEach defaults
// ---------------------------------------------------------------------------
beforeEach(() => {
  vi.clearAllMocks();
  window.location.hash = "#/designs";

  // Browse data defaults
  adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse());
  adapterMock.getBrowseDesignPreviews.mockResolvedValue(browseResponse());
  adapterMock.getBrowseTags.mockResolvedValue(browseResponse());
  adapterMock.getBrowseProjects.mockResolvedValue(browseResponse());
  adapterMock.getProjectsList.mockResolvedValue(browseResponse());

  // Admin list defaults
  adapterMock.listDesigners.mockResolvedValue(browseResponse());
  adapterMock.listSources.mockResolvedValue(browseResponse());
  adapterMock.listHoops.mockResolvedValue(browseResponse());
  adapterMock.listTags.mockResolvedValue(browseResponse());

  // Detail / about defaults
  adapterMock.getDesignDetail.mockResolvedValue({ source: "rust", item: null });
  adapterMock.getAboutDocuments.mockResolvedValue(browseResponse());
  adapterMock.getSettingsViewModel.mockResolvedValue({
    source: "rust",
    model: settingsModel,
  });

  // Persisted mutation responses
  const persisted = { source: "rust", persisted: true };
  adapterMock.bulkDeleteDesigns.mockResolvedValue({
    ...persisted,
    deleted_count: 1,
    files_trashed: 0,
    errors: [],
  });
  adapterMock.bulkVerifyDesigns.mockResolvedValue({
    ...persisted,
    requested_count: 1,
    verified_count: 1,
  });
  adapterMock.bulkAddDesignsToProject.mockResolvedValue({
    ...persisted,
    project_id: 1,
    requested_count: 1,
    added_count: 1,
  });
  adapterMock.bulkSetTagsForDesigns.mockResolvedValue({
    ...persisted,
    requested_count: 1,
    updated_count: 1,
  });
  adapterMock.addDesignToProject.mockResolvedValue({
    ...persisted,
    design_id: 1,
    message: "ok",
  });
  adapterMock.removeDesignFromProject.mockResolvedValue({
    ...persisted,
    design_id: 1,
    message: "ok",
  });
  adapterMock.createDesigner.mockResolvedValue({
    ...persisted,
    item: { id: 9, name: "New", design_count: 0 },
  });
  adapterMock.updateDesigner.mockResolvedValue({
    ...persisted,
    item: { id: 1, name: "Renamed", design_count: 1 },
  });
  adapterMock.deleteDesigner.mockResolvedValue(persisted);
  adapterMock.createSource.mockResolvedValue({
    ...persisted,
    item: { id: 9, name: "New", design_count: 0 },
  });
  adapterMock.updateSource.mockResolvedValue({
    ...persisted,
    item: { id: 1, name: "Renamed", design_count: 1 },
  });
  adapterMock.deleteSource.mockResolvedValue(persisted);
  adapterMock.createTag.mockResolvedValue({
    ...persisted,
    item: { id: 9, description: "New Tag", tag_group: "image" },
  });
  adapterMock.setTagGroup.mockResolvedValue({
    ...persisted,
    item: { id: 1, description: "Floral", tag_group: "stitching" },
  });
  adapterMock.deleteTag.mockResolvedValue(persisted);
  adapterMock.createHoop.mockResolvedValue({
    ...persisted,
    item: { id: 9, name: "New Hoop", max_width_mm: 100, max_height_mm: 150, design_count: 0 },
  });
  adapterMock.updateHoop.mockResolvedValue({
    ...persisted,
    item: { id: 1, name: "Renamed", max_width_mm: 100, max_height_mm: 150, design_count: 1 },
  });
  adapterMock.deleteHoop.mockResolvedValue(persisted);

  // Every remaining adapter function gets a benign resolved default so any
  // sub-view that mounts mid-test can never hang on an unresolved mock.
  const genericFnKeys = [
    "getDesignImageDataUrl",
    "updateDesignMetadata",
    "setDesignRating",
    "setDesignStitched",
    "setDesignTagsChecked",
    "setDesignTags",
    "removeDesignTag",
    "openDesignInEditor",
    "openDesignInExplorer",
    "renderDesign3dPreview",
    "previewImportFromRoots",
    "previewImportFromRoot",
    "browseImportFolder",
    "precheckImportWire",
    "runPrecheckAction",
    "requestStopBulkImport",
    "createProject",
    "getProjectDetail",
    "updateProject",
    "deleteProject",
    "removeDesignFromProjectDetail",
    "getProjectPrintView",
    "getAboutDocument",
    "saveSettings",
    "saveImportLastBrowseFolder",
    "browseSettingsDataRoot",
    "getTaggingActionsViewModel",
    "runUnifiedBackfill",
    "stopUnifiedBackfill",
    "getBackfillLogEntries",
    "runStitchingBackfill",
    "getBackupViewModel",
    "saveBackupSettings",
    "browseBackupFolder",
    "runDatabaseBackup",
    "runDesignsBackup",
    "runBothBackups",
    "scanOrphans",
    "getOrphansPage",
    "deleteOrphans",
    "deleteAllOrphans",
    "browseOrphanPath",
    "getAppStatus",
  ] as const;

  for (const key of genericFnKeys) {
    adapterMock[key].mockResolvedValue(persisted);
  }
});

// ---------------------------------------------------------------------------
// Routing & navigation
// ---------------------------------------------------------------------------
describe("routing and navigation", () => {
  it("renders the browse view by default when no hash is present", async () => {
    window.location.hash = "";
    render(MainView);

    await waitFor(() => {
      expect(screen.getByText("Browse Designs")).toBeInTheDocument();
    });
  });

  it("redirects unknown hashes to the browse view", async () => {
    renderAtHash("#/does-not-exist");

    await waitFor(() => {
      expect(screen.getByText("Browse Designs")).toBeInTheDocument();
    });
  });

  it("normalises route casing to the canonical hash", async () => {
    renderAtHash("#/HELP");

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Help" })).toBeInTheDocument();
    });
  });

  it("renders the Import view for #/import", async () => {
    renderAtHash("#/import");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Bulk Import" })
      ).toBeInTheDocument();
    });
  });

  it("renders the Projects view for #/projects", async () => {
    adapterMock.getProjectsList.mockResolvedValue({
      source: "rust",
      items: [{ id: 1, name: "Wedding Collection", design_count: 4 }],
    });
    renderAtHash("#/projects");

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Projects" })).toBeInTheDocument();
    });
    expect(screen.getByText("Wedding Collection")).toBeInTheDocument();
  });

  it("renders the Help view for #/help", async () => {
    renderAtHash("#/help");

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Help" })).toBeInTheDocument();
    });
  });

  it("renders the About view for #/about", async () => {
    renderAtHash("#/about");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "About Embroidery Catalogue" })
      ).toBeInTheDocument();
    });
  });

  it("renders the Settings view for #/admin/settings", async () => {
    renderAtHash("#/admin/settings");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Application Settings" })
      ).toBeInTheDocument();
    });
  });

  it("renders the Backup view for #/admin/maintenance/backup", async () => {
    renderAtHash("#/admin/maintenance/backup");

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Backup" })).toBeInTheDocument();
    });
  });

  it("renders the Tagging Actions view for #/admin/tagging-actions", async () => {
    renderAtHash("#/admin/tagging-actions");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Tagging Actions" })
      ).toBeInTheDocument();
    });
  });

  it("renders the Orphans view for #/admin/orphans", async () => {
    renderAtHash("#/admin/orphans");

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Orphans" })).toBeInTheDocument();
    });
  });

  it("renders the DesignDetail view for #/designs/42", async () => {
    adapterMock.getDesignDetail.mockResolvedValue({
      source: "rust",
      item: baseDetail,
    });
    renderAtHash("#/designs/42");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });
    expect(screen.getByText("Open in Editor")).toBeInTheDocument();
  });

  it("reacts to hashchange events after mount", async () => {
    renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("Browse Designs")).toBeInTheDocument();
    });

    window.location.hash = "#/help";
    fireEvent(window, new HashChangeEvent("hashchange"));

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Help" })).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// Browse: loading, empty, and error states
// ---------------------------------------------------------------------------
describe("browse loading, empty and error states", () => {
  it("shows the loading message while getBrowseDesigns is pending", async () => {
    adapterMock.getBrowseDesigns.mockReturnValue(new Promise(() => {}));

    renderAtHash("#/designs");

    expect(screen.getByText("Loading designs...")).toBeInTheDocument();
    await Promise.resolve();
  });

  it("shows the empty state when no designs match", async () => {
    renderAtHash("#/designs");

    await waitFor(() => {
      expect(
        screen.getByText("No designs match your filters.")
      ).toBeInTheDocument();
    });
  });

  it("renders the empty state when the adapter rejects", async () => {
    adapterMock.getBrowseDesigns.mockRejectedValue(new Error("backend down"));

    renderAtHash("#/designs");

    await waitFor(() => {
      expect(
        screen.getByText("No designs match your filters.")
      ).toBeInTheDocument();
    });
    expect(adapterMock.getBrowseDesigns).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Browse: card rendering
// ---------------------------------------------------------------------------
describe("browse card rendering", () => {
  it("renders a design card with filename, hoop, tags, rating and verified badge", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));

    renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });
    expect(screen.getByText("Hoop A")).toBeInTheDocument();
    // Card tags are rendered joined with ", " so assert the joined string.
    expect(screen.getByText("Floral, Border")).toBeInTheDocument();
    expect(screen.getByLabelText("Rating 4 out of 5")).toBeInTheDocument();
    expect(screen.getByLabelText("Verified")).toBeInTheDocument();
  });

  it("shows the 'No preview image' placeholder once previews have loaded", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));
    adapterMock.getBrowseDesignPreviews.mockResolvedValue(
      browseResponse([{ id: 1, data_url: null }])
    );

    renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("No preview image")).toBeInTheDocument();
    });
  });

  it("renders an <img> preview when a data URL is returned", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));
    adapterMock.getBrowseDesignPreviews.mockResolvedValue(
      browseResponse([{ id: 1, data_url: "data:image/png;base64,abc" }])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(container.querySelector(".browse-card-image")).not.toBeNull();
    });
    expect(
      container.querySelector(".browse-card-image")?.getAttribute("src")
    ).toBe("data:image/png;base64,abc");
  });
});

// ---------------------------------------------------------------------------
// Browse: filtering, search and sorting
// ---------------------------------------------------------------------------
describe("browse filtering and search", () => {
  it("filters the grid client-side as the user types in the general search box", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 1, filename: "rose.pes", hoop: "Hoop A" }),
        wireCard({ id: 2, filename: "tulip.pes", hoop: "Hoop B" }),
      ])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose.pes")).toBeInTheDocument();
    });

    const searchInput = container.querySelector<HTMLInputElement>("#browse-q");
    await fireEvent.input(element(searchInput), { target: { value: "tulip" } });

    await waitFor(() => {
      expect(screen.getByText("tulip.pes")).toBeInTheDocument();
    });
    expect(screen.queryByText("rose.pes")).not.toBeInTheDocument();
  });

  it("applies the unverified-only filter when the checkbox is toggled", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 1, filename: "verified.pes", tags_checked: true }),
        wireCard({ id: 2, filename: "unverified.pes", tags_checked: false }),
      ])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("verified.pes")).toBeInTheDocument();
    });

    const unverifiedCheckbox = container.querySelector<HTMLInputElement>(
      ".browse-unverified-checkbox"
    );
    await fireEvent.change(element(unverifiedCheckbox), { target: { checked: true } });

    await waitFor(() => {
      expect(screen.getByText("unverified.pes")).toBeInTheDocument();
    });
    expect(screen.queryByText("verified.pes")).not.toBeInTheDocument();
  });

  it("sorts ascending by default and descends when direction changes", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 3, filename: "cherry.pes", hoop: "Hoop" }),
        wireCard({ id: 1, filename: "apple.pes", hoop: "Hoop" }),
        wireCard({ id: 2, filename: "bee.pes", hoop: "Hoop" }),
      ])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("apple.pes")).toBeInTheDocument();
    });
    expect(cardTitles(container)).toEqual([
      "apple.pes",
      "bee.pes",
      "cherry.pes",
    ]);

    const directionSelect = screen.getByLabelText(/Direction:/, {
      selector: "select",
    });
    await fireEvent.change(directionSelect, { target: { value: "desc" } });

    await waitFor(() => {
      expect(cardTitles(container)).toEqual([
        "cherry.pes",
        "bee.pes",
        "apple.pes",
      ]);
    });
  });

  it("filters by minimum rating using the rating select", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 1, filename: "four.pes", hoop: "Hoop", rating: 4 }),
        wireCard({ id: 2, filename: "two.pes", hoop: "Hoop", rating: 2 }),
      ])
    );

    renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("four.pes")).toBeInTheDocument();
    });

    const ratingSelect = screen.getByLabelText(/Minimum rating/, {
      selector: "select",
    });
    await fireEvent.change(ratingSelect, { target: { value: "3" } });

    await waitFor(() => {
      expect(screen.queryByText("two.pes")).not.toBeInTheDocument();
    });
    expect(screen.getByText("four.pes")).toBeInTheDocument();
  });

  it("filters by designer via the additional-filters checkbox list", async () => {
    adapterMock.listDesigners.mockResolvedValue(
      browseResponse([{ id: 1, name: "Rose Studio", design_count: 1 }])
    );
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 1, filename: "rose.pes", designer: "Rose Studio", hoop: "Hoop" }),
        wireCard({ id: 2, filename: "tulip.pes", designer: "Mock", hoop: "Hoop" }),
      ])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose.pes")).toBeInTheDocument();
    });

    // Wait until the designer filter option has loaded.
    await waitFor(() => {
      const label = Array.from(
        container.querySelectorAll(".browse-additional-filters label")
      ).find((el) => (el.textContent || "").includes("Rose Studio"));
      expect(label).toBeDefined();
    });

    const designerLabel = Array.from(
      container.querySelectorAll(".browse-additional-filters label")
    ).find((el) => (el.textContent || "").includes("Rose Studio"));
    const checkbox = element(
      designerLabel?.querySelector<HTMLInputElement>("input[type='checkbox']")
    );
    await fireEvent.click(checkbox);

    await waitFor(() => {
      expect(screen.queryByText("tulip.pes")).not.toBeInTheDocument();
    });
    expect(screen.getByText("rose.pes")).toBeInTheDocument();
  });

  it("re-calls getBrowseDesigns with the search payload when the form is submitted", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([wireCard({ id: 1, filename: "rose.pes", hoop: "Hoop" })])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose.pes")).toBeInTheDocument();
    });

    const searchInput = container.querySelector<HTMLInputElement>("#browse-q");
    await fireEvent.input(element(searchInput), { target: { value: "rose" } });

    const form = container.querySelector<HTMLFormElement>(
      ".browse-search-shell"
    );
    await fireEvent.submit(element(form));

    await waitFor(() => {
      // Mount already triggered one call; the submit must add another.
      expect(adapterMock.getBrowseDesigns.mock.calls.length).toBeGreaterThan(1);
    });

    const lastPayload = adapterMock.getBrowseDesigns.mock.calls.at(-1)?.[0] as
      | Record<string, unknown>
      | undefined;
    expect(lastPayload?.q).toBe("rose");
    expect(lastPayload?.search_file_name).toBe(true);
    expect(lastPayload?.search_tags).toBe(true);
    expect(lastPayload?.search_folder_name).toBe(true);
  });

  it("enables and uses the Reset filters button", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(
      browseResponse([
        wireCard({ id: 1, filename: "a.pes", tags_checked: true, hoop: "Hoop" }),
        wireCard({ id: 2, filename: "b.pes", tags_checked: false, hoop: "Hoop" }),
      ])
    );

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("a.pes")).toBeInTheDocument();
    });

    const resetButton = screen.getByRole("button", { name: "Reset filters" });
    expect(resetButton).toBeDisabled();

    const unverifiedCheckbox = container.querySelector<HTMLInputElement>(
      ".browse-unverified-checkbox"
    );
    await fireEvent.change(element(unverifiedCheckbox), { target: { checked: true } });

    await waitFor(() => {
      expect(resetButton).not.toBeDisabled();
    });

    await fireEvent.click(resetButton);

    await waitFor(() => {
      expect(resetButton).toBeDisabled();
    });
    expect(screen.getByText("a.pes")).toBeInTheDocument();
    expect(screen.getByText("b.pes")).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Browse: pagination
// ---------------------------------------------------------------------------
describe("browse pagination", () => {
  it("paginates a large design set and navigates to the next page", async () => {
    const many = Array.from({ length: 60 }, (_, index) =>
      wireCard({
        id: index + 1,
        filename: `design-${String(index + 1).padStart(3, "0")}.pes`,
        hoop: "Hoop",
      })
    );
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse(many));

    renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("design-001.pes")).toBeInTheDocument();
    });
    expect(screen.queryByText("design-051.pes")).not.toBeInTheDocument();

    const nextButton = screen.getByRole("button", { name: /Next/ });
    await fireEvent.click(nextButton);

    await waitFor(() => {
      expect(screen.getByText("design-051.pes")).toBeInTheDocument();
    });
    expect(screen.queryByText("design-001.pes")).not.toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Browse: bulk selection and actions
// ---------------------------------------------------------------------------
describe("browse bulk selection and actions", () => {
  it("shows the bulk bar with a count once a design is selected", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });

    const checkbox = container.querySelector<HTMLInputElement>(
      ".browse-design-checkbox"
    );
    await fireEvent.input(element(checkbox));

    await waitFor(() => {
      expect(screen.getByText("1 design selected")).toBeInTheDocument();
    });
  });

  it("clears the selection and hides the bulk bar via Clear selection", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });

    const checkbox = container.querySelector<HTMLInputElement>(
      ".browse-design-checkbox"
    );
    await fireEvent.input(element(checkbox));
    await waitFor(() => {
      expect(screen.getByText("1 design selected")).toBeInTheDocument();
    });

    await fireEvent.click(
      screen.getByRole("button", { name: "Clear selection" })
    );

    await waitFor(() => {
      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
    });
  });

  it("runs bulk verification for the selected designs", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });

    const checkbox = container.querySelector<HTMLInputElement>(
      ".browse-design-checkbox"
    );
    await fireEvent.input(element(checkbox));
    await waitFor(() => {
      expect(screen.getByText("1 design selected")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole("button", { name: "Verify tags" }));

    await waitFor(() => {
      expect(adapterMock.bulkVerifyDesigns).toHaveBeenCalledWith([1]);
    });
    expect(toastMock.addToast).toHaveBeenCalledWith(
      expect.stringContaining("marked verified"),
      "success"
    );
  });

  it("opens the delete modal and deletes the selected designs", async () => {
    adapterMock.getBrowseDesigns.mockResolvedValue(browseResponse([wireCard()]));

    const { container } = renderAtHash("#/designs");

    await waitFor(() => {
      expect(screen.getByText("rose-border-01.pes")).toBeInTheDocument();
    });

    const checkbox = container.querySelector<HTMLInputElement>(
      ".browse-design-checkbox"
    );
    await fireEvent.input(element(checkbox));
    await waitFor(() => {
      expect(screen.getByText("1 design selected")).toBeInTheDocument();
    });

    await fireEvent.click(
      screen.getByRole("button", { name: "Delete selected" })
    );

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Delete selected design?" })
      ).toBeInTheDocument();
    });

    await fireEvent.click(
      screen.getByRole("button", { name: "Delete 1 design" })
    );

    await waitFor(() => {
      expect(adapterMock.bulkDeleteDesigns).toHaveBeenCalledWith([1], false);
    });
    expect(toastMock.addToast).toHaveBeenCalledWith(
      expect.stringContaining("1 design(s) deleted from catalogue."),
      "success"
    );
  });
});

// ---------------------------------------------------------------------------
// Admin pages
// ---------------------------------------------------------------------------
describe("admin designers", () => {
  it("renders the designers list with usage counts", async () => {
    adapterMock.listDesigners.mockResolvedValue(
      browseResponse([{ id: 1, name: "Rose Studio", design_count: 2 }])
    );

    renderAtHash("#/admin/designers");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Designers" })
      ).toBeInTheDocument();
    });
    expect(screen.getByText("Rose Studio")).toBeInTheDocument();
    expect(screen.getAllByText("2").length).toBeGreaterThan(0);
  });

  it("creates a new designer via the add form", async () => {
    renderAtHash("#/admin/designers");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Designers" })
      ).toBeInTheDocument();
    });

    const nameInput = screen.getByPlaceholderText("New designer name...");
    await fireEvent.input(nameInput, { target: { value: "New Studio" } });

    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMock.createDesigner).toHaveBeenCalledWith("New Studio");
    });
  });

  it("edits an existing designer", async () => {
    adapterMock.listDesigners.mockResolvedValue(
      browseResponse([{ id: 1, name: "Rose Studio", design_count: 0 }])
    );

    renderAtHash("#/admin/designers");

    await waitFor(() => {
      expect(screen.getByText("Rose Studio")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

    const editInput = screen.getByDisplayValue("Rose Studio") as HTMLInputElement;
    await fireEvent.input(editInput, { target: { value: "Renamed" } });

    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(adapterMock.updateDesigner).toHaveBeenCalledWith(1, "Renamed");
    });
  });

  it("deletes a designer after confirming", async () => {
    adapterMock.listDesigners.mockResolvedValue(
      browseResponse([{ id: 1, name: "Rose Studio", design_count: 0 }])
    );

    renderAtHash("#/admin/designers");

    await waitFor(() => {
      expect(screen.getByText("Rose Studio")).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Confirm delete" })
      ).toBeInTheDocument();
    });

    await fireEvent.click(
      screen.getByRole("button", { name: "Confirm delete" })
    );

    await waitFor(() => {
      expect(adapterMock.deleteDesigner).toHaveBeenCalledWith(1);
    });
  });
});

describe("admin sources", () => {
  it("renders the sources list and creates a new source", async () => {
    adapterMock.listSources.mockResolvedValue(
      browseResponse([{ id: 1, name: "Imported", design_count: 3 }])
    );

    renderAtHash("#/admin/sources");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Sources" })
      ).toBeInTheDocument();
    });
    expect(screen.getByText("Imported")).toBeInTheDocument();
    expect(screen.getAllByText("3").length).toBeGreaterThan(0);

    const nameInput = screen.getByPlaceholderText("e.g. Purchased, Downloaded...");
    await fireEvent.input(nameInput, { target: { value: "Gifted" } });
    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMock.createSource).toHaveBeenCalledWith("Gifted");
    });
  });
});

describe("admin hoops", () => {
  it("renders hoop rows with dimensions", async () => {
    adapterMock.listHoops.mockResolvedValue(
      browseResponse([
        {
          id: 1,
          name: "5x7 hoop",
          max_width_mm: 130,
          max_height_mm: 180,
          design_count: 2,
        },
      ])
    );

    renderAtHash("#/admin/hoops");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Hoops" })
      ).toBeInTheDocument();
    });
    expect(screen.getByText("5x7 hoop")).toBeInTheDocument();
    expect(screen.getByText("130")).toBeInTheDocument();
    expect(screen.getByText("180")).toBeInTheDocument();
  });
});

describe("admin tags", () => {
  it("renders the TagsView component with image and stitching sections", async () => {
    adapterMock.listTags.mockResolvedValue(
      browseResponse([
        { id: 1, description: "Floral", tag_group: "image" },
        { id: 2, description: "Satin", tag_group: "stitching" },
      ])
    );

    renderAtHash("#/admin/tags");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Tags" })
      ).toBeInTheDocument();
    });
    expect(
      screen.getByRole("heading", { name: "Image Tags" })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Stitching Tags" })
    ).toBeInTheDocument();

    expect(screen.getByText("Floral")).toBeInTheDocument();
    expect(screen.getByText("Satin")).toBeInTheDocument();
  });

  it("adds a new tag via the TagsView add form", async () => {
    renderAtHash("#/admin/tags");

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Manage Tags" })
      ).toBeInTheDocument();
    });

    const descInput = screen.getByPlaceholderText("e.g. Animals, Cross stitch...");
    await fireEvent.input(descInput, { target: { value: "Bees" } });

    await fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(adapterMock.createTag).toHaveBeenCalledWith("Bees", "image");
    });
  });
});
