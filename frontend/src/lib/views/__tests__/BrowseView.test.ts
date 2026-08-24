import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent, within } from "@testing-library/svelte";
import { tick } from "svelte";
import BrowseView from "../BrowseView.svelte";
import { deleteResultHolder } from "./__mocks__/deleteResultHolder.js";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls from running.
// All functions imported by BrowseView are stubbed so they can be asserted on.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getBrowseDesigns: vi.fn(),
  getBrowseDesignPreviews: vi.fn(),
  getBrowseProjects: vi.fn(),
  getBrowseTags: vi.fn(),
  addDesignToProject: vi.fn(),
  removeDesignFromProject: vi.fn(),
  listDesigners: vi.fn(),
  listSources: vi.fn(),
  listHoops: vi.fn(),
  bulkVerifyDesigns: vi.fn(),
  bulkAddDesignsToProject: vi.fn(),
  bulkSetTagsForDesigns: vi.fn(),
  bulkDeleteDesigns: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store.
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

// Mock the design session store.
const sessionMock = vi.hoisted(() => ({
  designSessionStore: {
    consumePatches: vi.fn(() => ({})),
  },
}));
vi.mock("../../stores/designSessionStore.js", () => sessionMock);

// Mock the tag change store.
const tagChangeMock = vi.hoisted(() => ({
  tagChangeStore: {
    consumeFlags: vi.fn(() => ({ designsNeedRefresh: false, tagsNeedRefresh: false })),
  },
}));
vi.mock("../../stores/tagChangeStore.js", () => tagChangeMock);

// Mock the portal action — it moves nodes to document.body which complicates
// DOM queries. A passthrough keeps the nodes in-place for selection.
vi.mock("../../utils/portal.js", () => ({
  portalToBody: (_node: HTMLElement) => ({
    destroy() {},
  }),
}));

// Mock the child components (real tiny Svelte stubs that expose testids).
vi.mock("../../components/DeleteDesignsModal.svelte", async () => {
  const { default: component } = await import("./__mocks__/DeleteDesignsModal.svelte");
  return { default: component };
});
vi.mock("../../components/Pagination.svelte", async () => {
  const { default: component } = await import("./__mocks__/Pagination.svelte");
  return { default: component };
});
vi.mock("../../components/SelectionHeader.svelte", async () => {
  const { default: component } = await import("./__mocks__/SelectionHeader.svelte");
  return { default: component };
});

// ---------------------------------------------------------------------------
// Fixtures & helpers
// ---------------------------------------------------------------------------

interface WireDesign {
  id?: number;
  filename?: string;
  filepath?: string;
  designer?: string;
  source?: string;
  hoop?: string | null;
  projects?: string[] | string;
  tags?: string[];
  image_tags?: string[];
  stitching_tags?: string[];
  is_stitched?: boolean;
  image_tags_verified?: boolean;
  stitching_tags_verified?: boolean;
  rating?: number | null;
  folder?: string;
  date_added?: string;
  project_names?: string[] | string;
}

/** Wrap items in an AdapterListResponse-style object (with pagination fields). */
const listResponse = (items: unknown[] = [], source = "rust") => ({
  source,
  items,
  page: 1,
  page_size: 50,
  total: items.length,
  total_pages: Math.max(1, Math.ceil(items.length / 50)),
});

// ---------------------------------------------------------------------------
// Backend-authority mock for getBrowseDesigns
//
// BrowseView now delegates filtering, sorting, and pagination to the backend
// and simply renders the returned page. These helpers simulate the backend so
// filter/sort tests can verify the UI wiring without a live Rust process.
// ---------------------------------------------------------------------------

function extractFolderFromPath(filepath: string): string {
  const path = String(filepath || "")
    .trim()
    .replace(/\\/g, "/");
  if (!path) return "";
  const segments = path.split("/").filter(Boolean);
  if (segments.length <= 1) return "";
  return segments[segments.length - 2];
}

function parseBackendQuery(q: string): Array<Array<{ text: string; exclude: boolean }>> {
  const query = String(q || "").trim();
  if (!query) return [];
  return query
    .split(/\bOR\b/)
    .map((part) => {
      const terms: Array<{ text: string; exclude: boolean }> = [];
      const regex = /"([^"]+)"|(-?\S+)/g;
      let match;
      while ((match = regex.exec(part)) !== null) {
        if (match[1]) {
          terms.push({ text: match[1], exclude: false });
        } else if (match[2]) {
          const rawTerm = match[2];
          const exclude = rawTerm.startsWith("-");
          const text = exclude ? rawTerm.slice(1) : rawTerm;
          if (text) terms.push({ text, exclude });
        }
      }
      return terms;
    })
    .filter((group) => group.length > 0);
}

function backendFieldMatches(field: string, token: string): boolean {
  const val = String(field || "").toLowerCase();
  const pat = String(token || "").toLowerCase();
  if (pat.includes("*") || pat.includes("?")) {
    const escaped = pat.replace(/[-/\\^$*+?.()|[\]{}]/g, (char) => {
      if (char === "*") return ".*";
      if (char === "?") return ".";
      return "\\" + char;
    });
    return new RegExp("^" + escaped + "$").test(val);
  }
  return val.includes(pat);
}

function applyBackendFilter(
  items: Array<Record<string, unknown>>,
  payload: Record<string, unknown>
): Array<Record<string, unknown>> {
  let result = items;

  const q = String(payload?.q || "").trim();
  if (q) {
    const searchFile = payload.search_file_name !== false;
    const searchTags = payload.search_tags !== false;
    const searchFolder = payload.search_folder_name !== false;
    const groups = parseBackendQuery(q);
    if (groups.length > 0) {
      result = result.filter((item) =>
        groups.some((group) =>
          group.every((token) => {
            const fields: string[] = [];
            if (searchFile) fields.push(String(item.filename || ""));
            if (searchTags) {
              fields.push(...(Array.isArray(item.tags) ? item.tags.map(String) : []));
            }
            if (searchFolder) {
              fields.push(extractFolderFromPath(String(item.filepath || "")));
            }
            const matched = fields.some((field) => backendFieldMatches(field, token.text));
            return token.exclude ? !matched : matched;
          })
        )
      );
    }
  }

  if (payload.unverified_only) {
    result = result.filter(
      (item) => !(item.image_tags_verified && item.stitching_tags_verified)
    );
  }

  const af = (payload.additional_filters || {}) as Record<string, unknown>;
  const designerFilters = Array.isArray(af.designer_filters)
    ? af.designer_filters.map(String)
    : [];
  if (designerFilters.length > 0) {
    const set = new Set(designerFilters.map((d) => d.toLowerCase().trim()));
    result = result.filter((item) =>
      set.has(String(item.designer || "").toLowerCase().trim())
    );
  }

  const imageTagFilters = Array.isArray(af.image_tag_filters)
    ? af.image_tag_filters.map(String)
    : [];
  if (imageTagFilters.length > 0) {
    const set = new Set(imageTagFilters.map((t) => t.toLowerCase().trim()));
    result = result.filter((item) =>
      (Array.isArray(item.image_tags) ? item.image_tags.map(String) : []).some((tag) =>
        set.has(tag.toLowerCase().trim())
      )
    );
  }

  const stitchingTagFilters = Array.isArray(af.stitching_tag_filters)
    ? af.stitching_tag_filters.map(String)
    : [];
  if (stitchingTagFilters.length > 0) {
    const set = new Set(stitchingTagFilters.map((t) => t.toLowerCase().trim()));
    result = result.filter((item) =>
      (Array.isArray(item.stitching_tags) ? item.stitching_tags.map(String) : []).some(
        (tag) => set.has(tag.toLowerCase().trim())
      )
    );
  }

  const sourceFilters = Array.isArray(af.source_filters) ? af.source_filters.map(String) : [];
  if (sourceFilters.length > 0) {
    const set = new Set(sourceFilters.map((s) => s.toLowerCase().trim()));
    result = result.filter((item) =>
      set.has(String(item.source || "").toLowerCase().trim())
    );
  }

  const hoop = String(af.hoop_size || "").trim();
  if (hoop) {
    result = result.filter(
      (item) => String(item.hoop || "").toLowerCase().trim() === hoop.toLowerCase()
    );
  }

  const minRating = Number(af.min_rating ?? 0);
  if (minRating >= 1) {
    result = result.filter((item) => Number(item.rating ?? 0) >= minRating);
  }

  const stitched = String(af.stitched_status || "").trim();
  if (stitched === "yes") result = result.filter((item) => item.is_stitched);
  else if (stitched === "no") result = result.filter((item) => !item.is_stitched);

  return result;
}

function applyBackendSort(
  items: Array<Record<string, unknown>>,
  sortBy: string,
  sortDir: string
): Array<Record<string, unknown>> {
  const dir = sortDir === "desc" ? -1 : 1;
  return [...items].sort((a, b) => {
    if (sortBy === "rating") {
      const x = Number(a.rating ?? -1);
      const y = Number(b.rating ?? -1);
      if (x !== y) return (x - y) * dir;
    }
    if (sortBy === "stitched") {
      const x = a.is_stitched ? 1 : 0;
      const y = b.is_stitched ? 1 : 0;
      if (x !== y) return (x - y) * dir;
    }
    if (sortBy === "folder") {
      const x = extractFolderFromPath(String(a.filepath || ""));
      const y = extractFolderFromPath(String(b.filepath || ""));
      const c = x.localeCompare(y, undefined, { sensitivity: "base" });
      if (c !== 0) return c * dir;
    }
    if (sortBy === "date_added") {
      const x = String(a.date_added || "");
      const y = String(b.date_added || "");
      const c = x.localeCompare(y);
      if (c !== 0) return c * dir;
    }
    return (
      String(a.filename || "").localeCompare(String(b.filename || ""), undefined, {
        sensitivity: "base",
      }) * dir
    );
  });
}

/** Configure getBrowseDesigns to behave like the paginated Rust backend. */
function mockBackendDesigns(all: Array<Record<string, unknown>>) {
  adapterMocks.getBrowseDesigns.mockImplementation(
    async (payload: Record<string, unknown> = {}) => {
      const filtered = applyBackendFilter(all, payload);
      const sorted = applyBackendSort(
        filtered,
        String(payload.sort_by || "name"),
        String(payload.sort_dir || "asc")
      );
      const pageSize = Math.max(1, Number(payload.page_size ?? 50));
      const page = Math.max(1, Number(payload.page ?? 1));
      const total = sorted.length;
      const totalPages = Math.max(1, Math.ceil(total / pageSize));
      const normalizedPage = Math.min(page, totalPages);
      const start = (normalizedPage - 1) * pageSize;
      return {
        source: "rust",
        page: normalizedPage,
        page_size: pageSize,
        total,
        total_pages: totalPages,
        items: sorted.slice(start, start + pageSize),
      };
    }
  );
}

/** Build a normalized browse-design wire fixture. */
const design = (overrides: Partial<WireDesign> = {}): Record<string, unknown> => ({
  id: 1,
  filename: "rose.pes",
  filepath: "C:/designs/rose.pes",
  designer: "Rose Studio",
  source: "Imported",
  hoop: "Hoop A",
  projects: [],
  tags: [],
  image_tags: [],
  stitching_tags: [],
  is_stitched: false,
  image_tags_verified: false,
  stitching_tags_verified: false,
  rating: null,
  ...overrides,
});

const tagOption = (id: number, description: string, tag_group: string | null = null) => ({
  id,
  description,
  tag_group,
  is_system: false,
});

const project = (id: number, name: string) => ({ id, name });

const entity = (id: number, name: string) => ({ id, name });

const hoopEntity = (id: number, name: string) => ({ id, name });

/** Default render with navigateTo that records calls. */
function renderBrowse(overrides: Record<string, unknown> = {}) {
  const navigateTo = vi.fn();
  const utils = render(BrowseView, {
    props: {
      navigateTo,
      browseNeedsRefresh: false,
      detailBrowseIds: [],
      detailBrowseIndex: -1,
      detailDesignId: null,
      ...overrides,
    },
  });
  return { ...utils, navigateTo };
}

/**
 * Wait for the default mock data to settle (effects fire on tick).
 * All adapter mocks resolve immediately, so once the initial effect has
 * invoked the adapter a single `tick()` flushes the whole reactive chain
 * deterministically (matching the .clinerules guidance: prefer `tick()`
 * over `waitFor` polling for synchronous mock data).
 */
async function settle() {
  await waitFor(() => {
    expect(adapterMocks.getBrowseDesigns).toHaveBeenCalled();
  });
  await tick();
}

/**
 * Helper to dispatch a window resize while faking innerWidth.
 */
async function setWindowWidth(width: number) {
  Object.defineProperty(window, "innerWidth", {
    configurable: true,
    writable: true,
    value: width,
  });
  window.dispatchEvent(new Event("resize"));
  // Flush Svelte's reactive chain rather than waiting uncounted microtasks.
  await tick();
  await tick();
}

describe("BrowseView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.localStorage.clear();
    // Default adapter mocks — one design, no projects/tags, no previews.
    adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([design()]));
    adapterMocks.getBrowseDesignPreviews.mockResolvedValue(listResponse([]));
    adapterMocks.getBrowseProjects.mockResolvedValue(listResponse([]));
    adapterMocks.getBrowseTags.mockResolvedValue(listResponse([]));
    adapterMocks.listDesigners.mockResolvedValue(listResponse([]));
    adapterMocks.listSources.mockResolvedValue(listResponse([]));
    adapterMocks.listHoops.mockResolvedValue(listResponse([]));
    adapterMocks.bulkVerifyDesigns.mockResolvedValue({
      source: "rust",
      persisted: true,
      verified_count: 1,
    });
    adapterMocks.bulkAddDesignsToProject.mockResolvedValue({
      source: "rust",
      persisted: true,
      added_count: 1,
    });
    adapterMocks.bulkSetTagsForDesigns.mockResolvedValue({
      source: "rust",
      persisted: true,
      updated_count: 1,
    });
    adapterMocks.addDesignToProject.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 1,
      message: "Added.",
    });
    adapterMocks.removeDesignFromProject.mockResolvedValue({
      source: "rust",
      persisted: true,
      design_id: 1,
      message: "Removed.",
    });
    adapterMocks.bulkDeleteDesigns.mockResolvedValue({
      source: "rust",
      persisted: true,
      deleted_count: 1,
      files_trashed: 0,
      errors: [],
    });
    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      writable: true,
      value: 1200,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // -------------------------------------------------------------------------
  // Rendering & Loading States
  // -------------------------------------------------------------------------

  describe("rendering & loading states", () => {
    it("renders the Browse Designs title and search form", async () => {
      renderBrowse();
      await settle();

      expect(screen.getByRole("heading", { name: "Browse Designs" })).toBeInTheDocument();
      expect(screen.getByText("General search")).toBeInTheDocument();
      expect(screen.getByText("Additional Filters")).toBeInTheDocument();
      expect(screen.getByText("Sort by:")).toBeInTheDocument();
      expect(screen.getByText("Search in:", { exact: false })).toBeInTheDocument();
      expect(screen.getByText("Unverified only")).toBeInTheDocument();
    });

    it("shows 'Loading designs...' while the first load is pending", async () => {
      let resolveLoading: () => void = () => {};
      const pending = new Promise<void>((resolve) => {
        resolveLoading = resolve;
      });
      adapterMocks.getBrowseDesigns.mockReturnValue(pending);
      const { unmount } = renderBrowse();

      expect(screen.getByText("Loading designs...")).toBeInTheDocument();

      // Resolve the promise and unmount to avoid hanging.
      resolveLoading();
      await Promise.resolve();
      await Promise.resolve();
      unmount();
    });

    it("shows 'No designs match your filters' when there are no filtered items", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([]));
      renderBrowse();

      expect(await screen.findByText("No designs match your filters.")).toBeInTheDocument();
    });

    it("renders a design card with filename, hoop, rating, and tags", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({
            id: 1,
            filename: "rose.pes",
            hoop: "Hoop A",
            rating: 4,
            tags: ["Floral"],
            image_tags_verified: true,
            stitching_tags_verified: true,
            is_stitched: true,
            image_tags: ["Floral"],
          }),
        ])
      );

      renderBrowse();

      expect(await screen.findByText("rose.pes")).toBeInTheDocument();
      expect(screen.getByText("Hoop A")).toBeInTheDocument();
      expect(screen.getByText("Floral")).toBeInTheDocument();
      expect(screen.getByText("4")).toBeInTheDocument();
      // Verified badge
      expect(screen.getByLabelText("Verified")).toBeInTheDocument();
    });

    it('shows "Hoop unknown" when hoop is blank', async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([design({ id: 1, hoop: "" })]));

      renderBrowse();

      expect(await screen.findByText("Hoop unknown")).toBeInTheDocument();
    });

    it('renders "No tags" when a card has no tags', async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([design({ id: 1, tags: [] })]));

      renderBrowse();

      expect(await screen.findByText("No tags")).toBeInTheDocument();
    });

    it('renders "Not rated" indicator (☆ —) when rating is null', async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, rating: null })])
      );

      renderBrowse();

      const rating = await screen.findByLabelText("Not rated");
      expect(rating).toBeInTheDocument();
      expect(rating).toHaveTextContent("☆ —");
    });

    it('renders "Rating 4 out of 5" when rated', async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([design({ id: 1, rating: 4 })]));

      renderBrowse();

      expect(await screen.findByLabelText("Rating 4 out of 5")).toBeInTheDocument();
    });

    it("renders projects on the card when assigned", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, projects: ["Wedding Collection"] })])
      );

      renderBrowse();

      expect(await screen.findByText("Wedding Collection")).toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // Search & Query Parsing
  // -------------------------------------------------------------------------

  describe("search & query parsing", () => {
    it("filters designs with an OR-query", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", tags: ["Floral"] }),
        design({ id: 2, filename: "leaf.pes", tags: ["Green"] }),
        design({ id: 3, filename: "tulip.pes", tags: ["Spring"] }),
      ]);

      renderBrowse();

      const q = await screen.findByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "Floral OR Green" } });
      // OR triggers a full reload via updateBrowseFilter? It doesn't auto-apply.
      // Simulate submit to apply the filter locally.
      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.getByText("leaf.pes")).toBeInTheDocument();
      expect(screen.queryByText("tulip.pes")).not.toBeInTheDocument();
    });

    it("filters designs with an exact-phrase query", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", tags: ["cross stitch"] }),
        design({ id: 2, filename: "leaf.pes", tags: ["applique"] }),
      ]);

      renderBrowse();

      const q = await screen.findByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: '"cross stitch"' } });
      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });

    it("filters designs with exclusion terms", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", tags: ["floral"] }),
        design({ id: 2, filename: "leaf.pes", tags: ["applique"] }),
      ]);

      renderBrowse();

      const q = await screen.findByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "applique" } });
      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      await waitFor(() => {
        expect(screen.getByText("leaf.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("rose.pes")).not.toBeInTheDocument();
    });

    it("does not apply search to tags when searchTags is off", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", tags: ["floral"] }),
        design({ id: 2, filename: "leaf.pes", tags: ["green"] }),
      ]);

      renderBrowse();

      // Turn "Search in: Tags" off.
      const tagsCheckbox = await screen.findByLabelText("Tags");
      await fireEvent.click(tagsCheckbox);

      const q = screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "floral" } });
      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      // Since tag search is off, no cards match → empty message.
      expect(await screen.findByText("No designs match your filters.")).toBeInTheDocument();
    });

    it("matches wildcard patterns (* and ?)", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes" }),
        design({ id: 2, filename: "rose2.pes" }),
        design({ id: 3, filename: "leaf.pes" }),
      ]);

      renderBrowse();

      const q = await screen.findByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "rose*.pes" } });
      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.getByText("rose2.pes")).toBeInTheDocument();
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // Filtering: additional filters
  // -------------------------------------------------------------------------

  describe("additional filters", () => {
    async function openFilters() {
      const summary = screen.getByText("Additional Filters");
      await fireEvent.click(summary);
    }

    it("toggles the additional filters section open/closed", async () => {
      renderBrowse();
      await settle();

      // Initially closed (▶)
      expect(screen.getByText("▶")).toBeInTheDocument();
      expect(screen.queryByText("▼")).not.toBeInTheDocument();

      const details = document.querySelector(".browse-additional-filters");
      expect(details).not.toBeNull();
      expect((details as HTMLElement).hasAttribute("open")).toBe(false);

      await openFilters();

      expect(screen.getByText("▼")).toBeInTheDocument();
      expect((details as HTMLElement).hasAttribute("open")).toBe(true);

      // Toggle closed again
      const summary = screen.getByText("Additional Filters");
      await fireEvent.click(summary);
      expect(screen.getByText("▶")).toBeInTheDocument();
      expect((details as HTMLElement).hasAttribute("open")).toBe(false);
    });

    it("filters by designer checkbox", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", designer: "Rose Studio" }),
        design({ id: 2, filename: "leaf.pes", designer: "Nature Co" }),
      ]);
      adapterMocks.listDesigners.mockResolvedValue(
        listResponse([entity(1, "Rose Studio"), entity(2, "Nature Co")])
      );

      renderBrowse();
      await settle();

      // Wait for filter reference to load.
      await waitFor(() => {
        expect(screen.getByText("Rose Studio")).toBeInTheDocument();
      });

      await openFilters();

      // Click the Rose Studio filter checkbox.
      const checkbox = screen.getByText("Rose Studio").closest("label")?.querySelector("input");
      const input = checkbox as HTMLInputElement;
      await fireEvent.click(input);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });

    it("filters by image tag", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", image_tags: ["Floral"] }),
        design({ id: 2, filename: "leaf.pes", image_tags: ["Green"] }),
      ]);
      adapterMocks.getBrowseTags.mockResolvedValue(
        listResponse([tagOption(1, "Floral", "image"), tagOption(2, "Green", "image")])
      );

      renderBrowse();
      await settle();

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });

      await openFilters();

      // Use scoped query: find "Floral" within the Image tags filter box.
      const imageTagSection = screen
        .getByText("Image tags", { exact: false })
        .closest("div") as HTMLElement;
      const checkbox = imageTagSection.querySelector(
        'label input[type="checkbox"]'
      ) as HTMLInputElement;
      await fireEvent.click(checkbox);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });

    it("filters by stitching tag", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", stitching_tags: ["Satin"] }),
        design({ id: 2, filename: "leaf.pes", stitching_tags: ["Cross"] }),
      ]);
      adapterMocks.getBrowseTags.mockResolvedValue(
        listResponse([tagOption(1, "Satin", "stitching"), tagOption(2, "Cross", "stitching")])
      );

      renderBrowse();
      await settle();

      // Wait for the tag options to appear in the DOM.
      const satinLabel = await waitFor(() => {
        const found = Array.from(document.querySelectorAll("label")).find(
          (label) => label.querySelector("span")?.textContent?.trim() === "Satin"
        );
        expect(found).toBeTruthy();
        return found;
      });

      await openFilters();

      const checkbox = (satinLabel as HTMLElement).querySelector("input") as HTMLInputElement;
      expect(checkbox).toBeTruthy();
      await fireEvent.click(checkbox);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });

    it("filters by source", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", source: "Imported" }),
        design({ id: 2, filename: "leaf.pes", source: "Purchased" }),
      ]);
      adapterMocks.listSources.mockResolvedValue(
        listResponse([entity(1, "Imported"), entity(2, "Purchased")])
      );

      renderBrowse();
      await settle();

      await waitFor(() => {
        expect(screen.getByText("Imported")).toBeInTheDocument();
      });

      await openFilters();

      const checkbox = screen.getByText("Imported").closest("label")?.querySelector("input");
      await fireEvent.click(checkbox as HTMLInputElement);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });

    it("filters by hoop size dropdown", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", hoop: "Hoop A" }),
        design({ id: 2, filename: "leaf.pes", hoop: "Hoop B" }),
      ]);
      adapterMocks.listHoops.mockResolvedValue(
        listResponse([hoopEntity(1, "Hoop A"), hoopEntity(2, "Hoop B")])
      );

      renderBrowse();
      await settle();

      // No openFilters() needed — jsdom renders <details> content regardless.
      const hoopSelect = screen.getByText("Hoop size").closest("label")?.querySelector("select");
      await fireEvent.change(hoopSelect as HTMLSelectElement, { target: { value: "Hoop B" } });

      // Flush Svelte's synchronous reactive chain before querying the DOM.
      await tick();

      expect(await screen.findByText("leaf.pes")).toBeInTheDocument();
      expect(screen.queryByText("rose.pes")).not.toBeInTheDocument();
    });

    it("filters by minimum rating", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "five.pes", rating: 5 }),
        design({ id: 2, filename: "two.pes", rating: 2 }),
      ]);

      renderBrowse();
      await settle();

      await openFilters();

      const ratingSelect = screen
        .getByText("Minimum rating")
        .closest("label")
        ?.querySelector("select");
      await fireEvent.change(ratingSelect as HTMLSelectElement, { target: { value: "3" } });

      await waitFor(() => {
        expect(screen.getByText("five.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("two.pes")).not.toBeInTheDocument();
    });

    it("filters by stitched status 'yes'", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "stitched.pes", is_stitched: true }),
        design({ id: 2, filename: "plain.pes", is_stitched: false }),
      ]);

      renderBrowse();
      await settle();

      await openFilters();

      // Scope to the filter's <label class="block"> whose span is exactly "Stitched".
      const stitchedSelect = Array.from(document.querySelectorAll<HTMLLabelElement>("label.block"))
        .find((label) => label.querySelector("span")?.textContent?.trim() === "Stitched")
        ?.querySelector("select");
      await fireEvent.change(stitchedSelect as HTMLSelectElement, { target: { value: "yes" } });

      await waitFor(() => {
        expect(screen.getByText("stitched.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("plain.pes")).not.toBeInTheDocument();
    });

    it("filters by 'Not Stitched' status", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "stitched.pes", is_stitched: true }),
        design({ id: 2, filename: "plain.pes", is_stitched: false }),
      ]);

      renderBrowse();
      await settle();

      await openFilters();

      // Scope to the filter's <label class="block"> whose span is exactly "Stitched".
      const stitchedSelect = Array.from(document.querySelectorAll<HTMLLabelElement>("label.block"))
        .find((label) => label.querySelector("span")?.textContent?.trim() === "Stitched")
        ?.querySelector("select");
      await fireEvent.change(stitchedSelect as HTMLSelectElement, { target: { value: "no" } });

      await waitFor(() => {
        expect(screen.getByText("plain.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("stitched.pes")).not.toBeInTheDocument();
    });

    it("filters by 'unverified only' checkbox", async () => {
      mockBackendDesigns([
        design({
          id: 1,
          filename: "unverified.pes",
          image_tags_verified: false,
          stitching_tags_verified: false,
        }),
        design({
          id: 2,
          filename: "verified.pes",
          image_tags_verified: true,
          stitching_tags_verified: true,
        }),
      ]);

      renderBrowse();
      await settle();

      const unverifiedCheckbox = screen.getByLabelText("Unverified only");
      await fireEvent.click(unverifiedCheckbox);

      await waitFor(() => {
        expect(screen.getByText("unverified.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("verified.pes")).not.toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // Sorting
  // -------------------------------------------------------------------------

  describe("sorting", () => {
    async function getSortSelect() {
      const label = screen.getByText("Sort by:").closest("label");
      return label?.querySelector("select") as HTMLSelectElement;
    }

    async function getDirSelect() {
      const label = screen.getByText("Direction:").closest("label");
      return label?.querySelector("select") as HTMLSelectElement;
    }

    it("sorts by name ascending by default", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "zeta.pes" }),
        design({ id: 2, filename: "alpha.pes" }),
      ]);

      renderBrowse();
      await settle();

      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("alpha.pes");
      expect(cards[1]).toHaveTextContent("zeta.pes");
    });

    it("re-renders card order when the sort dropdown changes", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "zeta.pes", rating: 2 }),
        design({ id: 2, filename: "alpha.pes", rating: 5 }),
      ]);

      renderBrowse();
      await settle();

      // Default sort (name asc): alpha.pes should be first
      let cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("alpha.pes");
      expect(cards[1]).toHaveTextContent("zeta.pes");

      // Change to sort by rating (asc by default)
      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "rating" } });

      // Svelte's reactive chain must be flushed before reading the DOM
      await tick();

      // Rating asc: zeta.pes (rating 5) should now be first
      cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("zeta.pes");
      expect(cards[1]).toHaveTextContent("alpha.pes");
    });

    it("sorts by name descending", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "zeta.pes" }),
        design({ id: 2, filename: "alpha.pes" }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "name" } });

      const dir = await getDirSelect();
      await fireEvent.change(dir, { target: { value: "desc" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("zeta.pes");
      expect(cards[1]).toHaveTextContent("alpha.pes");
    });

    it("sorts by rating", async () => {
      // Data chosen so name-asc order (alpha, zeta) is opposite to rating-asc
      // order (zeta(2) before alpha(5)), proving the sort really changed.
      mockBackendDesigns([
        design({ id: 1, filename: "zeta.pes", rating: 2 }),
        design({ id: 2, filename: "alpha.pes", rating: 5 }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "rating" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("zeta.pes");
      expect(cards[1]).toHaveTextContent("alpha.pes");
    });

    it("sorts by rating descending", async () => {
      // Same opposite-order fixture; desc rating puts alpha(5) first.
      mockBackendDesigns([
        design({ id: 1, filename: "zeta.pes", rating: 2 }),
        design({ id: 2, filename: "alpha.pes", rating: 5 }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "rating" } });

      const dir = await getDirSelect();
      await fireEvent.change(dir, { target: { value: "desc" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("alpha.pes");
      expect(cards[1]).toHaveTextContent("zeta.pes");
    });

    it("sorts by stitched status", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "stitched.pes", is_stitched: true }),
        design({ id: 2, filename: "plain.pes", is_stitched: false }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "stitched" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("plain.pes");
      expect(cards[1]).toHaveTextContent("stitched.pes");
    });

    it("sorts by folder", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "b.pes", filepath: "C:/designs/Zeta/b.pes" }),
        design({ id: 2, filename: "a.pes", filepath: "C:/designs/Alpha/a.pes" }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "folder" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("a.pes");
      expect(cards[1]).toHaveTextContent("b.pes");
    });

    it("sorts by date added", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "old.pes", date_added: "2020-01-01" }),
        design({ id: 2, filename: "new.pes", date_added: "2026-01-01" }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "date_added" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("old.pes");
      expect(cards[1]).toHaveTextContent("new.pes");
    });

    it("sorts by date added descending", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "old.pes", date_added: "2020-01-01" }),
        design({ id: 2, filename: "new.pes", date_added: "2026-01-01" }),
      ]);

      renderBrowse();
      await settle();

      const sort = await getSortSelect();
      await fireEvent.change(sort, { target: { value: "date_added" } });

      const dir = await getDirSelect();
      await fireEvent.change(dir, { target: { value: "desc" } });

      // Svelte's reactive chain is fully flushed; the DOM is settled
      await tick();
      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("new.pes");
      expect(cards[1]).toHaveTextContent("old.pes");
    });
  });

  // -------------------------------------------------------------------------
  // Selection behaviour
  // -------------------------------------------------------------------------

  describe("selection behaviour", () => {
    it("toggles individual card selection and shows the bulk bar", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();

      expect(await screen.findByText("rose.pes")).toBeInTheDocument();

      // No bulk bar initially.
      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();

      const cardCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      expect(cardCheckbox).not.toBeNull();
      await fireEvent.input(cardCheckbox, { target: { checked: true } });

      expect(screen.getByText("1 design selected")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Choose tags" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Verify tags" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Delete selected" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Clear selection" })).toBeInTheDocument();
    });

    it("toggles cell selection off when unchecked", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();

      await screen.findByText("rose.pes");

      const cardCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      await fireEvent.input(cardCheckbox, { target: { checked: true } });
      expect(screen.getByText("1 design selected")).toBeInTheDocument();

      await fireEvent.input(cardCheckbox, { target: { checked: false } });
      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
    });

    it("enforces the BROWSE_BULK_DELETE_MAX cap of 50", async () => {
      // 60 designs (more than the cap of 50) to test the cap on select-all.
      const many = Array.from({ length: 60 }, (_, i) =>
        design({ id: i + 1, filename: `design-${i + 1}.pes` })
      );
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse(many));

      renderBrowse({
        detailDesignId: null,
      });

      // Wait for pagination & selection to appear.
      await waitFor(() => {
        expect(screen.getByText("Select all on page")).toBeInTheDocument();
      });

      // Click "Select all on page".
      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);

      // The cap is 50 - but with only ~40 on page, all of them selected.
      expect(screen.getByText(/designs selected/)).toBeInTheDocument();
    });

    it("clears selection via the 'Clear selection' button", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();

      await screen.findByText("rose.pes");

      const cardCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      await fireEvent.input(cardCheckbox, { target: { checked: true } });
      expect(screen.getByText("1 design selected")).toBeInTheDocument();

      const clearBtn = screen.getByRole("button", { name: "Clear selection" });
      await fireEvent.click(clearBtn);

      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
    });

    it("disables card checkboxes while the delete modal is open", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();

      await screen.findByText("rose.pes");

      const cardCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      await fireEvent.input(cardCheckbox, { target: { checked: true } });

      // Open delete confirm.
      await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));

      // The delete modal mock should have rendered with open=true.
      expect(screen.getByTestId("delete-designs-modal")).toBeInTheDocument();

      // Checkbox should now be disabled.
      const lockedCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      expect(lockedCheckbox.disabled).toBe(true);
    });

    it("uses SelectionHeader to select all on page", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({ id: 1, filename: "rose.pes" }),
          design({ id: 2, filename: "leaf.pes" }),
        ])
      );

      renderBrowse();

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });

      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);

      expect(screen.getByText("2 designs selected")).toBeInTheDocument();
    });

    it("cleanup removes stale selected IDs when items are re-loaded", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();

      await screen.findByText("rose.pes");

      const cardCheckbox = document.querySelector(".browse-design-checkbox") as HTMLInputElement;
      await fireEvent.input(cardCheckbox, { target: { checked: true } });
      expect(screen.getByText("1 design selected")).toBeInTheDocument();

      // Reload without that ID.
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 99, filename: "new.pes" })])
      );

      // Trigger a reload by using the Reset filters button.
      const resetBtn = screen.getByRole("button", { name: "Reset filters" });
      await fireEvent.click(resetBtn);

      await waitFor(() => {
        expect(screen.getByText("new.pes")).toBeInTheDocument();
      });

      // No selection remains.
      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // Bulk actions
  // -------------------------------------------------------------------------

  describe("bulk actions", () => {
    async function selectItems(count = 1) {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse(
          Array.from({ length: count }, (_, i) =>
            design({ id: i + 1, filename: `design-${i + 1}.pes` })
          )
        )
      );
      renderBrowse();

      await waitFor(() => {
        expect(screen.getByText("design-1.pes")).toBeInTheDocument();
      });

      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);
    }

    it("opens the tag chooser modal when clicking 'Choose tags'", async () => {
      adapterMocks.getBrowseTags.mockResolvedValue(
        listResponse([tagOption(1, "Floral", "image"), tagOption(2, "Satin", "stitching")])
      );

      await selectItems(2);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");
      expect(dialog).toBeInTheDocument();
      expect(within(dialog).getByText("Choose tags for selected designs")).toBeInTheDocument();
      expect(within(dialog).getByText("Image tags")).toBeInTheDocument();
      expect(within(dialog).getByText("Stitching tags")).toBeInTheDocument();
      expect(within(dialog).getByText("Floral")).toBeInTheDocument();
      expect(within(dialog).getByText("Satin")).toBeInTheDocument();
    });

    it("applies bulk tags successfully with explicit add lists", async () => {
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));

      await selectItems(2);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");

      // Pick the Floral tag (a tri-state button with aria-checked).
      const floralButton = within(dialog).getByRole("checkbox", { name: /Floral/ });
      await fireEvent.click(floralButton);

      // Clicking cycles [-] / [ ] → [✓] add.
      expect(floralButton).toHaveAttribute("aria-checked", "true");
      expect(floralButton).toHaveTextContent("✓");

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        expect(adapterMocks.bulkSetTagsForDesigns).toHaveBeenCalled();
        const args = adapterMocks.bulkSetTagsForDesigns.mock.calls[0];
        expect(args[0].length).toBe(2);
        // args[1] = tagsToAdd, args[2] = tagsToRemove, args[3] = clearAllTags
        expect(args[1]).toEqual([1]);
        expect(args[2]).toEqual([]);
        expect(args[3]).toBe(false);
      });

      expect(toastMock.addToast).toHaveBeenCalledWith(
        "1 design(s) tag-updated in Rust database.",
        "success"
      );
    });

    it("shows an error toast when bulk tags fail", async () => {
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));
      adapterMocks.bulkSetTagsForDesigns.mockResolvedValue({
        source: "rust",
        persisted: false,
        error: "boom",
      });

      await selectItems(1);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");
      await fireEvent.click(within(dialog).getByRole("checkbox", { name: /Floral/ }));

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith("boom", "error");
      });
    });

    it("clears all tags when 'Untagged' is selected (clearAllTags=true)", async () => {
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));

      await selectItems(2);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");

      // Check Untagged (clear all tags) — a native checkbox.
      const untaggedLabel = within(dialog).getByText("Untagged (clear all tags)").closest("label");
      const untaggedCheckbox = untaggedLabel?.querySelector("input") as HTMLInputElement;
      await fireEvent.click(untaggedCheckbox);

      // The Floral tri-state button should be disabled now.
      const floralButton = within(dialog).getByRole("checkbox", { name: /Floral/ });
      expect((floralButton as HTMLButtonElement).disabled).toBe(true);

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        const args = adapterMocks.bulkSetTagsForDesigns.mock.calls[0];
        expect(args[1]).toEqual([]); // tagsToAdd empty when clearing
        expect(args[2]).toEqual([]); // tagsToRemove empty when clearing
        expect(args[3]).toBe(true); // clearAllTags true
      });
    });

    it("excludes indeterminate (mixed) tags from the save payload", async () => {
      // Two designs; only the first has "Floral" → the tag is mixed ([-]).
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({ id: 1, filename: "design-1.pes", tags: ["Floral"] }),
          design({ id: 2, filename: "design-2.pes", tags: [] }),
        ])
      );
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));

      // Select both designs WITHOUT the selectItems helper (which would
      // overwrite the fixture above with tag-less designs).
      renderBrowse();
      await waitFor(() => {
        expect(screen.getByText("design-1.pes")).toBeInTheDocument();
      });
      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");

      // Floral starts mixed ([-]) — untouched it must NOT be sent at all.
      const floralButton = within(dialog).getByRole("checkbox", { name: /Floral/ });
      expect(floralButton).toHaveAttribute("aria-checked", "mixed");
      expect(floralButton).toHaveTextContent("−");

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        const args = adapterMocks.bulkSetTagsForDesigns.mock.calls[0];
        expect(args[1]).toEqual([]); // tagsToAdd — Floral excluded
        expect(args[2]).toEqual([]); // tagsToRemove — Floral excluded
        expect(args[3]).toBe(false);
      });
    });

    it("cycles a mixed tag [-] → [✓] → [ ] → [✓]", async () => {
      // Both designs have Floral → initial state is checked [✓].
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({ id: 1, filename: "design-1.pes", tags: ["Floral"] }),
          design({ id: 2, filename: "design-2.pes", tags: ["Floral"] }),
        ])
      );
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));

      // Select both designs WITHOUT the selectItems helper (which would
      // overwrite the fixture above with tag-less designs).
      renderBrowse();
      await waitFor(() => {
        expect(screen.getByText("design-1.pes")).toBeInTheDocument();
      });
      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");
      const floralButton = within(dialog).getByRole("checkbox", { name: /Floral/ });

      // Initial: [✓] add
      expect(floralButton).toHaveAttribute("aria-checked", "true");
      expect(floralButton).toHaveTextContent("✓");

      // Click 1: [✓] → [ ] remove
      await fireEvent.click(floralButton);
      expect(floralButton).toHaveAttribute("aria-checked", "false");
      expect(floralButton.querySelector(".tag-chooser-box")).toHaveTextContent("");

      // Click 2: [ ] → [✓] add
      await fireEvent.click(floralButton);
      expect(floralButton).toHaveAttribute("aria-checked", "true");
      expect(floralButton.querySelector(".tag-chooser-box")).toHaveTextContent("✓");

      // Click 3: [✓] → [ ] remove (final state → send to remove list)
      await fireEvent.click(floralButton);
      expect(floralButton).toHaveAttribute("aria-checked", "false");
      expect(floralButton.querySelector(".tag-chooser-box")).toHaveTextContent("");

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        const args = adapterMocks.bulkSetTagsForDesigns.mock.calls[0];
        expect(args[1]).toEqual([]); // no adds
        expect(args[2]).toEqual([1]); // Floral removed
        expect(args[3]).toBe(false);
      });
    });

    it("runs bulk verify on selected designs", async () => {
      await selectItems(2);

      await fireEvent.click(screen.getByRole("button", { name: "Verify tags" }));

      // Verify the API was invoked with the two selected design IDs.
      expect(adapterMocks.bulkVerifyDesigns).toHaveBeenCalledWith([1, 2]);
    });

    it("adds selected designs to a project", async () => {
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection")])
      );

      await selectItems(2);

      // Open the project dropdown.
      await fireEvent.click(screen.getByText("Add to project…"));

      // Scope to the bulk project dropdown (the <details> with the inline-block style)
      // so the "Wedding Collection" text in the in-card project details is ignored.
      const dropdown = document.querySelector(
        "details[style*='display:inline-block']"
      ) as HTMLElement;
      const projectLabel = within(dropdown).getByText("Wedding Collection").closest("label");
      const projectCheckbox = projectLabel?.querySelector("input") as HTMLInputElement;
      await fireEvent.click(projectCheckbox);

      // Let Svelte re-enable the Apply button now that a project is selected.
      await tick();

      // Click Apply.
      await fireEvent.click(screen.getByRole("button", { name: "Apply" }));

      // Verify the API was invoked with the project ID and selected design IDs.
      expect(adapterMocks.bulkAddDesignsToProject).toHaveBeenCalledWith(1, [1, 2]);
    });

    it("shows a warning toast when only some projects succeed", async () => {
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection"), project(2, "Autumn")])
      );
      adapterMocks.bulkAddDesignsToProject.mockImplementation(async (projectId: number) => {
        if (projectId === 1) {
          return { source: "rust", persisted: true, added_count: 1 };
        }
        return { source: "rust", persisted: false, error: "over quota" };
      });

      await selectItems(2);

      await fireEvent.click(screen.getByText("Add to project…"));

      // Scope to the bulk project dropdown so the in-card project details are ignored.
      const dropdown = document.querySelector(
        "details[style*='display:inline-block']"
      ) as HTMLElement;

      // Check both projects.
      const weddingLabel = within(dropdown).getByText("Wedding Collection").closest("label");
      await fireEvent.click(weddingLabel?.querySelector("input") as HTMLInputElement);
      const autumnLabel = within(dropdown).getByText("Autumn").closest("label");
      await fireEvent.click(autumnLabel?.querySelector("input") as HTMLInputElement);

      // Let Svelte re-enable the Apply button now that projects are selected.
      await tick();

      await fireEvent.click(screen.getByRole("button", { name: "Apply" }));

      // The mock resolves immediately, so the warning toast is emitted synchronously.
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Some projects could not be updated. 1 design(s) added to project(s).",
        "warning"
      );
    });

    it("opens the delete confirmation modal with selected previews", async () => {
      adapterMocks.getBrowseDesignPreviews.mockResolvedValue(
        listResponse([{ id: 1, data_url: "data:image/png;base64,abc" }])
      );

      await selectItems(1);

      await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));

      // DeleteDesignsModal mock should be open with the correct props.
      const modal = screen.getByTestId("delete-designs-modal");
      expect(modal).toHaveAttribute("data-open", "true");
      expect(modal).toHaveAttribute("data-design-count", "1");
      expect(modal).toHaveTextContent("design-1.pes");
    });

    it("handles the delete result callback and clears selection", async () => {
      await selectItems(1);

      await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));

      const modal = screen.getByTestId("delete-designs-modal");
      const confirmBtn = within(modal).getByRole("button", { name: "Confirm delete" });
      await fireEvent.click(confirmBtn);

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "1 design(s) deleted from catalogue.",
          "success"
        );
      });

      // Selection cleared and modal closed.
      expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
      expect(modal).toHaveAttribute("data-open", "false");
    });

    it("includes the file-trash count in the delete success toast", async () => {
      deleteResultHolder.value = {
        persisted: true,
        deleted_count: 2,
        files_trashed: 1,
        errors: [],
      };

      try {
        await selectItems(2);
        await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));
        const modal = screen.getByTestId("delete-designs-modal");
        await fireEvent.click(within(modal).getByRole("button", { name: "Confirm delete" }));

        await waitFor(() => {
          expect(toastMock.addToast).toHaveBeenCalledWith(
            "2 design(s) deleted from catalogue. 1 source file(s) moved to recycle bin.",
            "success"
          );
        });
        expect(screen.queryByText("2 designs selected")).not.toBeInTheDocument();
      } finally {
        deleteResultHolder.value = null;
      }
    });

    it("includes file-warning count and console.warn when the delete has errors", async () => {
      const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
      deleteResultHolder.value = {
        persisted: true,
        deleted_count: 1,
        files_trashed: 0,
        errors: ["file locked"],
      };

      try {
        await selectItems(1);
        await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));
        const modal = screen.getByTestId("delete-designs-modal");
        await fireEvent.click(within(modal).getByRole("button", { name: "Confirm delete" }));

        await waitFor(() => {
          expect(toastMock.addToast).toHaveBeenCalledWith(
            "1 design(s) deleted from catalogue. (1 file warning(s) — see console for details)",
            "success"
          );
        });
        expect(warnSpy).toHaveBeenCalledWith("Bulk delete file warnings:", ["file locked"]);
      } finally {
        deleteResultHolder.value = null;
        warnSpy.mockRestore();
      }
    });

    it("shows an error toast when the delete fails to persist", async () => {
      deleteResultHolder.value = {
        persisted: false,
        deleted_count: 0,
        files_trashed: 0,
        errors: ["permission denied"],
      };

      try {
        await selectItems(1);
        await fireEvent.click(screen.getByRole("button", { name: "Delete selected" }));
        const modal = screen.getByTestId("delete-designs-modal");
        await fireEvent.click(within(modal).getByRole("button", { name: "Confirm delete" }));

        await waitFor(() => {
          expect(toastMock.addToast).toHaveBeenCalledWith("permission denied", "error");
        });
        expect(screen.queryByText("1 design selected")).not.toBeInTheDocument();
      } finally {
        deleteResultHolder.value = null;
      }
    });
  });

  // -------------------------------------------------------------------------
  // Pagination
  // -------------------------------------------------------------------------

  describe("pagination", () => {
    // Narrow width → 2 columns × 10 rows = 20 items per page.
    const PAGINATION_WIDTH = 500;
    const PAGE_SIZE = 2 * 10; // browseGridColumns * BROWSE_PAGE_ROWS

    function renderPaginated(count: number) {
      Object.defineProperty(window, "innerWidth", {
        configurable: true,
        writable: true,
        value: PAGINATION_WIDTH,
      });
      const many = Array.from({ length: count }, (_, i) =>
        design({ id: i + 1, filename: `design-${i + 1}.pes` })
      );
      // Backend-paginated response: echo the requested page back and compute
      // total_pages from PAGE_SIZE, since the browse page size is now sent to
      // the backend rather than sliced client-side.
      adapterMocks.getBrowseDesigns.mockImplementation(async (payload: any) => {
        const requestedPage = Math.max(1, Number(payload?.page ?? 1));
        return {
          source: "rust",
          page: requestedPage,
          page_size: PAGE_SIZE,
          total: count,
          total_pages: Math.max(1, Math.ceil(count / PAGE_SIZE)),
          items: many,
        };
      });
      renderBrowse();
    }

    function paginationNode() {
      return screen.getByTestId("pagination-mock") as HTMLElement;
    }

    it("shows pagination when there are more items than fit on one page", async () => {
      renderPaginated(PAGE_SIZE + 1); // 21 items → 2 pages

      await settle();
      await tick();

      expect(paginationNode()).toHaveAttribute("data-total-pages", "2");
    });

    it("changes page via the pagination component", async () => {
      renderPaginated(PAGE_SIZE + 1); // 21 items → 2 pages

      await settle();
      await tick();

      // Initially on page 1.
      expect(paginationNode()).toHaveAttribute("data-current-page", "1");

      // Click Next → page 2.
      await fireEvent.click(screen.getByRole("button", { name: "Next" }));
      await tick();

      expect(paginationNode()).toHaveAttribute("data-current-page", "2");
    });

    it("clamps current page when filtered count shrinks", async () => {
      renderPaginated(PAGE_SIZE + 1); // 21 items → 2 pages

      await settle();
      await tick();

      // Go to page 2.
      await fireEvent.click(screen.getByRole("button", { name: "Next" }));
      await tick();
      expect(paginationNode()).toHaveAttribute("data-current-page", "2");

      // Reload with only 1 item.
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "only.pes" })])
      );

      // Trigger a reload.
      await fireEvent.click(screen.getByRole("button", { name: "Reset filters" }));
      await tick();
      await tick();

      expect(paginationNode()).toHaveAttribute("data-total-pages", "1");
      expect(paginationNode()).toHaveAttribute("data-current-page", "1");
    });
  });

  // -------------------------------------------------------------------------
  // Grid & responsive layout
  // -------------------------------------------------------------------------

  describe("grid & responsive layout", () => {
    it("uses 5 columns on large screens (>= 1024)", async () => {
      await setWindowWidth(1200);
      renderBrowse();
      await settle();

      const row = document.querySelector(".browse-grid-row");
      const style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(5, minmax(0, 1fr))");
    });

    it("uses 4 columns on medium screens (768-1023)", async () => {
      await setWindowWidth(800);
      renderBrowse();
      await settle();

      const row = document.querySelector(".browse-grid-row");
      const style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(4, minmax(0, 1fr))");
    });

    it("uses 3 columns on small screens (640-767)", async () => {
      await setWindowWidth(700);
      renderBrowse();
      await settle();

      const row = document.querySelector(".browse-grid-row");
      const style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(3, minmax(0, 1fr))");
    });

    it("uses 2 columns on very small screens (< 640)", async () => {
      await setWindowWidth(500);
      renderBrowse();
      await settle();

      const row = document.querySelector(".browse-grid-row");
      const style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(2, minmax(0, 1fr))");
    });

    it("recalculates columns when the window resizes", async () => {
      // Start narrow so the initial grid is 2 columns.
      Object.defineProperty(window, "innerWidth", {
        configurable: true,
        writable: true,
        value: 500,
      });
      renderBrowse();
      await settle();
      await tick();

      let row = document.querySelector(".browse-grid-row");
      let style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(2, minmax(0, 1fr))");

      await setWindowWidth(1200);
      await tick();

      row = document.querySelector(".browse-grid-row");
      style = (row as HTMLElement).getAttribute("style");
      expect(style).toContain("repeat(5, minmax(0, 1fr))");
    });
  });

  // -------------------------------------------------------------------------
  // Design card interactions
  // -------------------------------------------------------------------------

  describe("design card interactions", () => {
    it("navigates to design detail when a card is clicked", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 42, filename: "rose.pes" })])
      );

      const { navigateTo } = renderBrowse();

      await screen.findByText("rose.pes");

      // Click the card's link button.
      const cardButton = document.querySelector(".browse-card-link") as HTMLButtonElement;
      await fireEvent.click(cardButton);

      expect(navigateTo).toHaveBeenCalledWith("#/designs/42");
    });

    it("shows 'Loading image...' when previews are loading", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      // Force the preview load to be pending.
      let resolvePreview: () => void = () => {};
      const pending = new Promise<void>((resolve) => {
        resolvePreview = resolve;
      });
      adapterMocks.getBrowseDesignPreviews.mockReturnValue(pending);

      const { unmount } = renderBrowse();

      expect(await screen.findByText("Loading image...")).toBeInTheDocument();

      // Resolve and unmount.
      resolvePreview();
      await Promise.resolve();
      await Promise.resolve();
      unmount();
    });

    it('shows "No preview image" when preview is missing', async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseDesignPreviews.mockResolvedValue(listResponse([]));

      renderBrowse();

      expect(await screen.findByText("No preview image")).toBeInTheDocument();
    });

    it("renders the preview image when data_url is available", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseDesignPreviews.mockResolvedValue(
        listResponse([{ id: 1, data_url: "data:image/png;base64,abc123" }])
      );

      renderBrowse();

      const img = await screen.findByAltText("rose.pes");
      expect(img).toHaveAttribute("src", "data:image/png;base64,abc123");
      expect(screen.queryByText("No preview image")).not.toBeInTheDocument();
    });

    it("extracts folder from filepath when not provided", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({ id: 1, filename: "rose.pes", filepath: "C:/designs/Embroidery/rose.pes" }),
        ])
      );

      renderBrowse();
      await settle();

      const sort = screen
        .getByText("Sort by:")
        .closest("label")
        ?.querySelector("select") as HTMLSelectElement;
      await fireEvent.change(sort, { target: { value: "folder" } });

      // Just ensure the card still renders without errors.
      expect(screen.getByText("rose.pes")).toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // In-card project management
  // -------------------------------------------------------------------------

  describe("in-card project management", () => {
    it("displays 'No projects found. Create one first.' when projects list is empty", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseProjects.mockResolvedValue(listResponse([]));

      renderBrowse();

      await screen.findByText("rose.pes");

      // Open the card project details.
      const summary = screen.getByText("+ Add to project");
      await fireEvent.click(summary);

      expect(await screen.findByText("No projects found. Create one first.")).toBeInTheDocument();
    });

    it("shows project names in the card project dropdown", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection")])
      );

      renderBrowse();

      await settle();

      // All $effect-driven loads have completed — verify synchronously, no polling.
      expect(adapterMocks.getBrowseProjects).toHaveBeenCalled();
      await tick();

      // Open the card project details.
      const summary = screen.getByText("+ Add to project");
      await fireEvent.click(summary);

      // The mock resolves immediately, so a single tick flushes the
      // reactive chain deterministically — no polling needed.
      await tick();
      expect(screen.getByText("Wedding Collection")).toBeInTheDocument();
    });

    it("calls addDesignToProject when a project checkbox is checked", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 7, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection")])
      );

      renderBrowse();

      await settle();

      const summary = screen.getByText("+ Add to project");
      await fireEvent.click(summary);

      // The mock resolves immediately — a tick flushes the reactive chain.
      await tick();
      const weddingLabel = screen.getByText("Wedding Collection").closest("label");
      const checkbox = weddingLabel?.querySelector("input") as HTMLInputElement;
      await fireEvent.change(checkbox, { target: { checked: true } });

      // Mock resolves immediately; callback fired synchronously.
      expect(adapterMocks.addDesignToProject).toHaveBeenCalledWith(7, 1);
    });

    it("calls removeDesignFromProject when a project checkbox is unchecked", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 7, filename: "rose.pes", projects: ["Wedding Collection"] })])
      );
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection")])
      );

      renderBrowse();

      await settle();

      const summary = screen.getByText("+ Add to project");
      await fireEvent.click(summary);

      // The mock resolves immediately — a tick flushes the reactive chain.
      await tick();
      // Scope to the dropdown — the same name also appears in the card's
      // project summary, so `within()` disambiguates (see .clinerules Rule #2).
      const dropdown = document.querySelector(".browse-card-project-details") as HTMLElement;
      const weddingLabel = within(dropdown).getByText("Wedding Collection").closest("label");
      const checkbox = weddingLabel?.querySelector("input") as HTMLInputElement;
      expect(checkbox.checked).toBe(true);

      await fireEvent.change(checkbox, { target: { checked: false } });

      // Mock resolves immediately; callback fired synchronously.
      expect(adapterMocks.removeDesignFromProject).toHaveBeenCalledWith(7, 1);
    });
  });

  // -------------------------------------------------------------------------
  // Reactive effects & store integration
  // -------------------------------------------------------------------------

  describe("reactive effects & store integration", () => {
    it("applies session store patches to browse items", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes", rating: 2 })])
      );
      sessionMock.designSessionStore.consumePatches.mockReturnValue({
        1: { rating: 5, hoop: null },
      });

      renderBrowse();

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });

      // Rating should now show 5 instead of 2.
      expect(screen.getByLabelText("Rating 5 out of 5")).toBeInTheDocument();
    });

    it("handles tagChangeStore 'designsNeedRefresh' flag", async () => {
      tagChangeMock.tagChangeStore.consumeFlags.mockReturnValueOnce({
        designsNeedRefresh: true,
        tagsNeedRefresh: false,
      });

      renderBrowse();

      await waitFor(() => {
        expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(2); // initial + refresh
      });
      expect(adapterMocks.getBrowseTags).toHaveBeenCalled();
    });

    it("handles tagChangeStore 'tagsNeedRefresh' only", async () => {
      tagChangeMock.tagChangeStore.consumeFlags.mockReturnValueOnce({
        designsNeedRefresh: false,
        tagsNeedRefresh: true,
      });

      renderBrowse();

      await waitFor(() => {
        expect(adapterMocks.getBrowseTags).toHaveBeenCalled();
      });
      // The initial design load still happens once.
      expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(1);
    });

    it("reloads items when browseNeedsRefresh changes", async () => {
      const { rerender } = renderBrowse();

      // Initial load.
      await waitFor(() => {
        expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(1);
      });

      // Re-render with browseNeedsRefresh=true.
      await rerender({ browseNeedsRefresh: true });

      await waitFor(() => {
        expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(2);
      });
    });

    it("auto-loads tags when browseTagOptions is empty", async () => {
      renderBrowse();

      await waitFor(() => {
        expect(adapterMocks.getBrowseTags).toHaveBeenCalled();
      });
    });

    it("auto-loads filter reference data when not loaded", async () => {
      renderBrowse();

      await waitFor(() => {
        expect(adapterMocks.listDesigners).toHaveBeenCalled();
        expect(adapterMocks.listSources).toHaveBeenCalled();
        expect(adapterMocks.listHoops).toHaveBeenCalled();
      });
    });

    it("auto-loads projects when browseProjectsLoaded is false", async () => {
      renderBrowse();

      await waitFor(() => {
        expect(adapterMocks.getBrowseProjects).toHaveBeenCalled();
      });
    });
  });

  // -------------------------------------------------------------------------
  // Error handling
  // -------------------------------------------------------------------------

  describe("error handling", () => {
    it("does not retry endlessly when getBrowseDesigns rejects", async () => {
      adapterMocks.getBrowseDesigns.mockRejectedValue(new Error("backend down"));

      renderBrowse();
      await settle();

      // A failed load must settle after one attempt — the reactive $effect
      // must not re-fire loadBrowseItems forever (which made this test hang).
      expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(1);
      expect(screen.queryByText("Loading designs...")).not.toBeInTheDocument();
    });

    it("handles getBrowseTags rejection with a console.info", async () => {
      const spy = vi.spyOn(console, "info").mockImplementation(() => {});
      adapterMocks.getBrowseTags.mockRejectedValue(new Error("tags down"));

      renderBrowse();
      await settle();

      expect(spy).toHaveBeenCalledWith("Could not load browse tags list", expect.any(Error));

      spy.mockRestore();
    });

    it("handles getBrowsePreviews rejection by setting null previews", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );
      adapterMocks.getBrowseDesignPreviews.mockRejectedValue(new Error("preview down"));

      renderBrowse();

      // The preview load fails gracefully; the card still renders.
      expect(await screen.findByText("rose.pes")).toBeInTheDocument();
      expect(await screen.findByText("No preview image")).toBeInTheDocument();
    });

    it("handles filter reference load failure", async () => {
      const spy = vi.spyOn(console, "info").mockImplementation(() => {});
      adapterMocks.listDesigners.mockRejectedValue(new Error("ref down"));

      renderBrowse();
      await settle();

      expect(spy).toHaveBeenCalledWith(
        "Could not load browse filter reference data",
        expect.any(Error)
      );

      spy.mockRestore();
    });

    it("handles bulkAddDesignsToProject rejection", async () => {
      adapterMocks.getBrowseProjects.mockResolvedValue(
        listResponse([project(1, "Wedding Collection")])
      );
      adapterMocks.bulkAddDesignsToProject.mockRejectedValue(new Error("network"));

      await selectAndOpenProject();

      const applyBtn = screen.getByRole("button", { name: "Apply" });
      await fireEvent.click(applyBtn);

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Bulk project add failed: Error: network",
          "error"
        );
      });
    });

    async function selectAndOpenProject() {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "design-1.pes" })])
      );

      renderBrowse();

      await waitFor(() => {
        expect(screen.getByText("design-1.pes")).toBeInTheDocument();
      });

      const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
      await fireEvent.click(allCheckbox);

      await fireEvent.click(screen.getByText("Add to project…"));

      // Scope to the bulk project dropdown — "Wedding Collection" also appears
      // in the in-card project details, so `within()` disambiguates (see
      // .clinerules Rule #2).
      await waitFor(() => {
        const dropdown = document.querySelector(
          "details[style*='display:inline-block']"
        ) as HTMLElement;
        expect(within(dropdown).getByText("Wedding Collection")).toBeInTheDocument();
      });

      const dropdown = document.querySelector(
        "details[style*='display:inline-block']"
      ) as HTMLElement;
      const weddingLabel = within(dropdown).getByText("Wedding Collection").closest("label");
      await fireEvent.click(weddingLabel?.querySelector("input") as HTMLInputElement);
    }

    it("handles bulkSetTagsForDesigns rejection", async () => {
      adapterMocks.getBrowseTags.mockResolvedValue(listResponse([tagOption(1, "Floral", "image")]));
      adapterMocks.bulkSetTagsForDesigns.mockRejectedValue(new Error("network"));

      await selectItemsHelper(1);

      await fireEvent.click(screen.getByRole("button", { name: "Choose tags" }));

      const dialog = screen.getByRole("dialog");
      await fireEvent.click(within(dialog).getByRole("checkbox", { name: /Floral/ }));

      await fireEvent.click(within(dialog).getByRole("button", { name: "Apply tags" }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Bulk tagging failed: Error: network",
          "error"
        );
      });
    });

    it("handles bulkVerify rejection", async () => {
      adapterMocks.bulkVerifyDesigns.mockRejectedValue(new Error("network"));

      await selectItemsHelper(1);

      await fireEvent.click(screen.getByRole("button", { name: "Verify tags" }));

      await waitFor(() => {
        expect(toastMock.addToast).toHaveBeenCalledWith(
          "Verification failed: Error: network",
          "error"
        );
      });
    });
  });

  // -------------------------------------------------------------------------
  // Clear / reset filters
  // -------------------------------------------------------------------------

  describe("clear / reset filters", () => {
    it("disables the Reset button while filters are at defaults", async () => {
      renderBrowse();
      await settle();

      const resetBtn = screen.getByRole("button", { name: "Reset filters" });
      expect(resetBtn).toBeDisabled();
    });

    it("enables the Reset button after changing a filter", async () => {
      renderBrowse();
      await settle();

      const q = screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "rose" } });

      const resetBtn = screen.getByRole("button", { name: "Reset filters" });
      expect(resetBtn).not.toBeDisabled();
    });

    it("resets all filters and reloads when clicked", async () => {
      renderBrowse();
      await settle();

      const q = screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "rose" } });

      await fireEvent.click(screen.getByRole("button", { name: "Reset filters" }));

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus')
        ).toHaveValue("");
      });
      expect(adapterMocks.getBrowseDesigns).toHaveBeenCalled();
    });

    it("submits the search form to apply filters", async () => {
      mockBackendDesigns([
        design({ id: 1, filename: "rose.pes", tags: ["floral"] }),
        design({ id: 2, filename: "leaf.pes", tags: ["green"] }),
      ]);

      renderBrowse();
      await settle();

      const q = screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');
      await fireEvent.input(q, { target: { value: "floral" } });

      await fireEvent.submit(q.closest("form") as HTMLFormElement);

      await waitFor(() => {
        expect(screen.getByText("rose.pes")).toBeInTheDocument();
      });
      expect(screen.queryByText("leaf.pes")).not.toBeInTheDocument();
    });
  });

  // -------------------------------------------------------------------------
  // Normalization edge cases
  // -------------------------------------------------------------------------

  describe("normalizeCardItem edge cases", () => {
    it("handles projects provided as a comma-separated string", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes", projects: "Wedding, Autumn" })])
      );

      renderBrowse();
      await settle();

      expect(screen.getByText("rose.pes")).toBeInTheDocument();
      // Projects are split on "," and trimmed → joined back with ", "
      expect(screen.getByText("Wedding, Autumn")).toBeInTheDocument();
    });

    it("handles project_names as an array when projects is absent", async () => {
      const item = design({ id: 1, filename: "rose.pes" });
      delete item.projects;
      item.project_names = ["Holiday"];
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([item]));

      renderBrowse();
      await settle();

      expect(screen.getByText("Holiday")).toBeInTheDocument();
    });

    it("handles project_names as a comma-separated string when projects is absent", async () => {
      const item = design({ id: 1, filename: "rose.pes" });
      delete item.projects;
      item.project_names = "Spring, Summer";
      adapterMocks.getBrowseDesigns.mockResolvedValue(listResponse([item]));

      renderBrowse();
      await settle();

      expect(screen.getByText("Spring, Summer")).toBeInTheDocument();
    });

    it("uses the explicit folder field over the filepath-derived folder", async () => {
      // The explicit `folder` field differs from what extractFolder would
      // derive from filepath, proving the folder branch wins (folder-asc sort:
      // Alpha folder → a.pes must come before Zeta folder → b.pes).
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([
          design({ id: 1, filename: "a.pes", filepath: "C:/designs/Zeta/a.pes", folder: "Alpha" }),
          design({ id: 2, filename: "b.pes", filepath: "C:/designs/Alpha/b.pes", folder: "Zeta" }),
        ])
      );

      renderBrowse();
      await settle();

      const sort = screen
        .getByText("Sort by:")
        .closest("label")
        ?.querySelector("select") as HTMLSelectElement;
      await fireEvent.change(sort, { target: { value: "folder" } });
      await tick();

      const cards = screen.getAllByRole("article");
      expect(cards[0]).toHaveTextContent("a.pes");
      expect(cards[1]).toHaveTextContent("b.pes");
    });
  });

  // -------------------------------------------------------------------------
  // updateBrowseFilter: empty-q immediate reload
  // -------------------------------------------------------------------------

  describe("updateBrowseFilter q-empty reload", () => {
    it("auto-reloads when the q filter is cleared", async () => {
      adapterMocks.getBrowseDesigns.mockResolvedValue(
        listResponse([design({ id: 1, filename: "rose.pes" })])
      );

      renderBrowse();
      await settle();

      // Initial load.
      expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(1);

      const q = screen.getByPlaceholderText('e.g. rose "cross stitch" -applique or *.hus');

      // Typing a query does NOT auto-apply (only submit does).
      await fireEvent.input(q, { target: { value: "rose" } });
      await tick();
      expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(1);

      // Clearing q triggers the `(key === "q" && !value)` branch → auto-reload.
      await fireEvent.input(q, { target: { value: "" } });
      await waitFor(() => {
        expect(adapterMocks.getBrowseDesigns).toHaveBeenCalledTimes(2);
      });
    });
  });
});

// Helper to select the first page items.
async function selectItemsHelper(count: number) {
  adapterMocks.getBrowseDesigns.mockResolvedValue(
    listResponse(
      Array.from({ length: count }, (_, i) =>
        design({ id: i + 1, filename: `design-${i + 1}.pes` })
      )
    )
  );

  renderBrowse();

  await waitFor(() => {
    expect(screen.getByText("design-1.pes")).toBeInTheDocument();
  });

  const allCheckbox = screen.getByTestId("select-all-page-checkbox") as HTMLInputElement;
  await fireEvent.click(allCheckbox);
}
