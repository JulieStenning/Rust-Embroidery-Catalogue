import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import ProjectsView from "../ProjectsView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMock = vi.hoisted(() => ({
  getProjectsList: vi.fn(),
  createProject: vi.fn(),
  getProjectDetail: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  removeDesignFromProjectDetail: vi.fn(),
  getProjectPrintView: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMock);

// Mock the toast store — ProjectsView calls addToast() on every mutation.
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore", () => toastMock);

const printItem = {
  project: { name: "Wedding Collection", description: "Bridesmaid gifts." },
  designs: [
    {
      id: 101,
      filename: "rose-border.pes",
      filepath: "C:/designs/rose-border.pes",
      image_data_url: "data:image/png;base64,AAAA",
      width_mm: 120,
      height_mm: 80,
      hoop: "Hoop A",
      stitch_count: 10000,
      color_count: 5,
      color_change_count: 12,
      designer_name: "Rose Studio",
      rating: 4,
      is_stitched: true,
      notes: "Pretty floral border.",
    },
  ],
};

const printResponse = (overrides: Record<string, unknown> = {}) => ({
  source: "rust",
  item: { ...printItem, ...overrides },
  error: undefined,
});

function renderProjects(props: Record<string, unknown> = {}) {
  return render(ProjectsView, {
    props: { currentUiKind: "project-print", projectDetailId: null, projectPrintId: 1, navigateTo: () => {}, ...props },
  });
}

describe("ProjectsView print view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMock.getProjectsList.mockResolvedValue({ source: "rust", items: [], error: undefined });
    adapterMock.getProjectDetail.mockResolvedValue({ source: "rust", item: null, error: undefined });
    adapterMock.getProjectPrintView.mockResolvedValue(printResponse());
  });

  it("shows the loading message while getProjectPrintView is pending", async () => {
    adapterMock.getProjectPrintView.mockReturnValue(new Promise(() => {}));
    renderProjects();
    expect(screen.getByText("Loading printable project sheet...")).toBeInTheDocument();
    await Promise.resolve();
  });

  it("renders an error message when getProjectPrintView rejects", async () => {
    adapterMock.getProjectPrintView.mockRejectedValue(new Error("network down"));
    renderProjects();
    await waitFor(() => expect(screen.getByText(/Could not load project print view: Error: network down/)).toBeInTheDocument());
  });

  it("renders the project title, description and design print metadata", async () => {
    renderProjects();
    await waitFor(() => expect(screen.getByRole("heading", { name: "Wedding Collection" })).toBeInTheDocument());

    expect(screen.getByText("Bridesmaid gifts.")).toBeInTheDocument();
    expect(screen.getByText("rose-border.pes")).toBeInTheDocument();
    expect(screen.getByText(/120 x 80 mm/)).toBeInTheDocument();
    expect(screen.getByText("Hoop A")).toBeInTheDocument();
    expect(screen.getByText("10000")).toBeInTheDocument();
    expect(screen.getByText("Rose Studio")).toBeInTheDocument();
    expect(screen.getByText("★★★★☆")).toBeInTheDocument();
    expect(screen.getByText("Pretty floral border.")).toBeInTheDocument();
  });

  it("clamps ratings above five to five stars", async () => {
    adapterMock.getProjectPrintView.mockResolvedValue(printResponse({ designs: [{ ...printItem.designs[0], rating: 6 }] }));
    renderProjects();
    await waitFor(() => expect(screen.getByText("★★★★★")).toBeInTheDocument());
  });

  it("shows the empty message when the project has no designs", async () => {
    adapterMock.getProjectPrintView.mockResolvedValue(printResponse({ designs: [] }));
    renderProjects();
    await waitFor(() => expect(screen.getByRole("heading", { name: "Wedding Collection" })).toBeInTheDocument());
    expect(screen.getByText("No designs in this project yet.")).toBeInTheDocument();
  });

  it("navigates back when Back to Project is clicked", async () => {
    const navigateTo = vi.fn();
    renderProjects({ navigateTo });
    await waitFor(() => expect(screen.getByRole("heading", { name: "Wedding Collection" })).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: "Back to Project" }));
    expect(navigateTo).toHaveBeenCalledWith("#/projects/1");
  });

  it("calls window.print when Print is clicked", async () => {
    const printSpy = vi.spyOn(window, "print").mockImplementation(() => {});
    try {
      renderProjects();
      await waitFor(() => expect(screen.getByRole("heading", { name: "Wedding Collection" })).toBeInTheDocument());
      await fireEvent.click(screen.getByRole("button", { name: "Print" }));
      expect(printSpy).toHaveBeenCalledTimes(1);
    } finally {
      printSpy.mockRestore();
    }
  });
});