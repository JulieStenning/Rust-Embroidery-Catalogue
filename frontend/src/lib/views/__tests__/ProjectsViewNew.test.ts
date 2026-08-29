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

const persistedCreate = {
  source: "rust",
  persisted: true,
  project_id: 9,
  message: "Project created.",
};

function renderProjects(props: Record<string, unknown> = {}) {
  return render(ProjectsView, {
    props: {
      currentUiKind: "project-new",
      projectDetailId: null,
      projectPrintId: null,
      navigateTo: () => {},
      ...props,
    },
  });
}

describe("ProjectsView new project view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMock.getProjectsList.mockResolvedValue({ source: "rust", items: [], error: undefined });
    adapterMock.createProject.mockResolvedValue(persistedCreate);
  });

  it("renders the Name and Description fields with a disabled Create button", async () => {
    renderProjects();
    expect(screen.getByPlaceholderText("e.g. Christmas Stockings 2024")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Optional notes, goals, or deadline")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Create Project" })).toBeDisabled();
  });

  it("creates a project, shows a success toast, clears the form and navigates to #/projects", async () => {
    const navigateTo = vi.fn();
    renderProjects({ navigateTo });

    const nameInput = screen.getByPlaceholderText("e.g. Christmas Stockings 2024");
    const descriptionInput = screen.getByPlaceholderText("Optional notes, goals, or deadline");
    await fireEvent.input(nameInput, { target: { value: "Christ's" } });
    await fireEvent.input(descriptionInput, { target: { value: "Notes" } });
    await fireEvent.click(screen.getByRole("button", { name: "Create Project" }));

    await waitFor(() => {
      expect(adapterMock.createProject).toHaveBeenCalledWith("Christ's", "Notes");
      expect(toastMock.addToast).toHaveBeenCalledWith("Project created.", "success");
      expect(navigateTo).toHaveBeenCalledWith("#/projects");
    });
    expect((nameInput as HTMLInputElement).value).toBe("");
    expect((descriptionInput as HTMLTextAreaElement).value).toBe("");
  });

  it("shows an error toast and keeps the form intact when the API fails", async () => {
    adapterMock.createProject.mockResolvedValue({
      source: "mock",
      persisted: false,
      project_id: 0,
      message: "Could not create project: backend down",
      error: "backend down",
    });
    const navigateTo = vi.fn();
    renderProjects({ navigateTo });

    const nameInput = screen.getByPlaceholderText("e.g. Christmas Stockings 2024");
    await fireEvent.input(nameInput, { target: { value: "Christ's" } });
    await fireEvent.click(screen.getByRole("button", { name: "Create Project" }));

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith(
        "Could not create project: backend down",
        "error"
      );
    });
    expect((nameInput as HTMLInputElement).value).toBe("Christ's");
    expect(navigateTo).not.toHaveBeenCalled();
  });

  it("shows an error toast when submitting with a blank name", async () => {
    const { container } = renderProjects();

    // The Create button is disabled while the name is blank, so submit the
    // form directly to exercise the submitNewProject validation guard.
    const form = container.querySelector("form");
    expect(form).not.toBeNull();
    await fireEvent.submit(form!);

    await waitFor(() => {
      expect(toastMock.addToast).toHaveBeenCalledWith("Project name is required.", "error");
    });
    expect(adapterMock.createProject).not.toHaveBeenCalled();
  });
});
