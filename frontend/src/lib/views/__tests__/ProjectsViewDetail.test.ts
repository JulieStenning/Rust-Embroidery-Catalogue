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

const detailItem = {
  project: { id: 1, name: "Wedding Collection", description: "Bridesmaid gifts." },
  designs: [
    { id: 101, filename: "rose-border.pes", filepath: "C:/designs/rose-border.pes", image_data_url: "data:image/png;base64,AAAA", designer_name: "Rose Studio" },
    { id: 102, filename: "tulip.pes", filepath: "C:/designs/tulip.pes", image_data_url: null, has_image: true, designer_name: "Tulip Co" },
  ],
};

const detailResponse = () => ({ source: "rust", item: detailItem, error: undefined });

const persistedUpdate = { source: "rust", persisted: true, project_id: 1, message: "Project updated." };
const persistedDelete = { source: "rust", persisted: true, project_id: 1, message: "Project deleted." };
const persistedRemove = { source: "rust", persisted: true, project_id: 1, design_id: 101, message: "Design removed from project." };

function renderProjects(props: Record<string, unknown> = {}) {
  return render(ProjectsView, {
    props: { currentUiKind: "project-detail", projectDetailId: 1, projectPrintId: null, navigateTo: () => {}, ...props },
  });
}

function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) throw new Error(message ?? "Expected element to exist.");
  return value;
}

describe("ProjectsView detail view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMock.getProjectsList.mockResolvedValue({ source: "rust", items: [], error: undefined });
    adapterMock.getProjectDetail.mockResolvedValue(detailResponse());
    adapterMock.updateProject.mockResolvedValue(persistedUpdate);
    adapterMock.deleteProject.mockResolvedValue(persistedDelete);
    adapterMock.removeDesignFromProjectDetail.mockResolvedValue(persistedRemove);
    adapterMock.getProjectPrintView.mockResolvedValue({ source: "rust", item: null, error: undefined });
  });

  it("shows the loading message while getProjectDetail is pending", async () => {
    adapterMock.getProjectDetail.mockReturnValue(new Promise(() => {}));
    renderProjects();
    expect(screen.getByText("Loading project...")).toBeInTheDocument();
    await Promise.resolve();
  });

  it("renders the adapter error when the detail item is null", async () => {
    adapterMock.getProjectDetail.mockResolvedValue({ source: "rust", item: null, error: "Project 999 was deleted." });
    renderProjects({ projectDetailId: 999 });
    await waitFor(() => expect(screen.getByText("Project 999 was deleted.")).toBeInTheDocument());
  });

  it("renders an error message when getProjectDetail rejects", async () => {
    adapterMock.getProjectDetail.mockRejectedValue(new Error("network down"));
    renderProjects();
    await waitFor(() => expect(screen.getByText(/Could not load project detail: Error: network down/)).toBeInTheDocument());
  });

  it("renders project details and design cards", async () => {
    const { container } = renderProjects();
    await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());

    const nameInput = container.querySelector<HTMLInputElement>(".projects-title-input");
    expect(nameInput?.value).toBe("Wedding Collection");
    const descriptionInput = container.querySelector<HTMLTextAreaElement>(".projects-textarea");
    expect(descriptionInput?.value).toBe("Bridesmaid gifts.");

    expect(screen.getByText("tulip.pes")).toBeInTheDocument();
    expect(screen.getByText("Rose Studio")).toBeInTheDocument();
    expect(screen.getByText("Tulip Co")).toBeInTheDocument();
    const roseImage = screen.getByRole("img", { name: "rose-border.pes" });
    expect(roseImage).toHaveAttribute("src", "data:image/png;base64,AAAA");
    expect(screen.getByText("Image unavailable")).toBeInTheDocument();
  });

  it("enables Save and Undo after an edit and reverts on Undo", async () => {
    const { container } = renderProjects();
    await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());

    const saveButton = screen.getByRole("button", { name: "Save" });
    const undoButton = screen.getByRole("button", { name: "Undo" });
    expect(saveButton).toBeDisabled();
    expect(undoButton).toBeDisabled();

    const nameInput = container.querySelector<HTMLInputElement>(".projects-title-input");
    await fireEvent.input(element(nameInput), { target: { value: "Wedding Collection v2" } });
    await waitFor(() => expect(saveButton).not.toBeDisabled());
    expect(undoButton).not.toBeDisabled();

    await fireEvent.click(undoButton);
    await waitFor(() => expect(saveButton).toBeDisabled());
    const revertedInput = container.querySelector<HTMLInputElement>(".projects-title-input");
    expect(revertedInput?.value).toBe("Wedding Collection");
  });

  it("saves edits via updateProject and refreshes the detail view", async () => {
    const { container } = renderProjects();
    await waitFor(() => expect(adapterMock.getProjectDetail).toHaveBeenCalledWith(1));
    await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());

    const nameInput = container.querySelector<HTMLInputElement>(".projects-title-input");
    await fireEvent.input(element(nameInput), { target: { value: "Wedding Collection v2" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(adapterMock.updateProject).toHaveBeenCalledWith(1, "Wedding Collection v2", "Bridesmaid gifts.");
      expect(toastMock.addToast).toHaveBeenCalledWith("Project updated.", "success");
    });
    await waitFor(() => expect(adapterMock.getProjectDetail).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(screen.getByRole("button", { name: "Save" })).toBeDisabled());
  });

  it("removes a design and reloads the detail", async () => {
    renderProjects();
    await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());

    await fireEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);

    await waitFor(() => {
      expect(adapterMock.removeDesignFromProjectDetail).toHaveBeenCalledWith(1, 101);
      expect(toastMock.addToast).toHaveBeenCalledWith("Design removed from project.", "success");
    });
    await waitFor(() => expect(adapterMock.getProjectDetail).toHaveBeenCalledTimes(2));
  });

  it("does not delete the project when the confirmation is cancelled", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    try {
      renderProjects();
      await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());
      await fireEvent.click(screen.getByRole("button", { name: "Delete Project" }));
      expect(adapterMock.deleteProject).not.toHaveBeenCalled();
    } finally {
      confirmSpy.mockRestore();
    }
  });

  it("deletes the project after confirmation and navigates to #/projects", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const navigateTo = vi.fn();
    try {
      renderProjects({ navigateTo });
      await waitFor(() => expect(screen.getByText("rose-border.pes")).toBeInTheDocument());
      await fireEvent.click(screen.getByRole("button", { name: "Delete Project" }));
      await waitFor(() => {
        expect(adapterMock.deleteProject).toHaveBeenCalledWith(1);
        expect(toastMock.addToast).toHaveBeenCalledWith("Project deleted.", "success");
      });
      expect(navigateTo).toHaveBeenCalledWith("#/projects");
    } finally {
      confirmSpy.mockRestore();
    }
  });
});