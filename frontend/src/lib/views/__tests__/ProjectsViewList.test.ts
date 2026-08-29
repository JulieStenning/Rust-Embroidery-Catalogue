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

const projectsResponse = (items: unknown[] = []) => ({ source: "rust", items, error: undefined });

const listItems = [
  {
    id: 1,
    name: "Wedding Collection",
    description: "Bridesmaid gifts.",
    design_count: 4,
    date_created: "2026-05-01",
  },
  { id: 2, name: "Autumn 2026", description: null, design_count: 1 },
];

const emptyStateMatcher = (content: string) => content.includes("No projects yet.");

function renderProjects(props: Record<string, unknown> = {}) {
  return render(ProjectsView, {
    props: {
      currentUiKind: "projects-list",
      projectDetailId: null,
      projectPrintId: null,
      navigateTo: () => {},
      ...props,
    },
  });
}

describe("ProjectsView list view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMock.getProjectsList.mockResolvedValue(projectsResponse(listItems));
  });

  it("shows the loading message while getProjectsList is pending", async () => {
    adapterMock.getProjectsList.mockReturnValue(new Promise(() => {}));
    renderProjects();
    expect(screen.getByText("Loading projects...")).toBeInTheDocument();
    await Promise.resolve();
  });

  it("renders project titles, descriptions and design counts", async () => {
    renderProjects();
    await waitFor(() => expect(screen.getByText("Wedding Collection")).toBeInTheDocument());
    expect(screen.getByText("Bridesmaid gifts.")).toBeInTheDocument();
    expect(screen.getByText("Autumn 2026")).toBeInTheDocument();
    expect(screen.getByText("4 designs")).toBeInTheDocument();
    expect(screen.getByText("1 design")).toBeInTheDocument();
  });

  it("renders the adapter error message when getProjectsList returns an error", async () => {
    adapterMock.getProjectsList.mockResolvedValue({
      source: "mock",
      items: [],
      error: "Could not load projects: backend down",
    });
    renderProjects();
    await waitFor(() =>
      expect(screen.getByText(/Could not load projects: backend down/)).toBeInTheDocument()
    );
  });

  it("renders an error message when getProjectsList rejects", async () => {
    adapterMock.getProjectsList.mockRejectedValue(new Error("network down"));
    renderProjects();
    await waitFor(() =>
      expect(screen.getByText(/Could not load projects: Error: network down/)).toBeInTheDocument()
    );
  });

  it("shows the empty state with a Create one link when the list is empty", async () => {
    adapterMock.getProjectsList.mockResolvedValue(projectsResponse([]));
    renderProjects();
    await waitFor(() => expect(screen.getByText(emptyStateMatcher)).toBeInTheDocument());
    expect(screen.getByRole("button", { name: "Create one" })).toBeInTheDocument();
  });

  it("navigates to #/projects/new via the + New Project button", async () => {
    const navigateTo = vi.fn();
    renderProjects({ navigateTo });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "+ New Project" })).toBeInTheDocument()
    );
    await fireEvent.click(screen.getByRole("button", { name: "+ New Project" }));
    expect(navigateTo).toHaveBeenCalledWith("#/projects/new");
  });

  it("navigates to #/projects/new via the empty-state Create one button", async () => {
    adapterMock.getProjectsList.mockResolvedValue(projectsResponse([]));
    const navigateTo = vi.fn();
    renderProjects({ navigateTo });
    await waitFor(() => expect(screen.getByText(emptyStateMatcher)).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: "Create one" }));
    expect(navigateTo).toHaveBeenCalledWith("#/projects/new");
  });

  it("renders a project tile linking to the project detail route", async () => {
    renderProjects();
    await waitFor(() => expect(screen.getByText("Wedding Collection")).toBeInTheDocument());
    const tile = screen.getByRole("link", { name: "Open project Wedding Collection" });
    expect(tile).toHaveAttribute("href", "#/projects/1");
  });

  it("defaults to an empty list when the response has no items array", async () => {
    adapterMock.getProjectsList.mockResolvedValue({ source: "rust", error: undefined });
    renderProjects();
    await waitFor(() => expect(screen.getByText(emptyStateMatcher)).toBeInTheDocument());
  });

  it("defaults a missing design count to zero", async () => {
    adapterMock.getProjectsList.mockResolvedValue(
      projectsResponse([{ id: 3, name: "Sparse", description: null, design_count: null }])
    );
    renderProjects();
    await waitFor(() => expect(screen.getByText("Sparse")).toBeInTheDocument());
    expect(screen.getByText("0 designs")).toBeInTheDocument();
  });
});
