// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type {
  AdapterProjectDesignMutationResponse,
  AdapterProjectDetailResponse,
  AdapterProjectListResponse,
  AdapterProjectMutationResponse,
  AdapterListResponse,
  AdapterMutationResponse,
  DesignCommandResult,
  ProjectDetailView,
  ProjectMutationResult,
  ProjectListItem,
  ProjectSummary,
  RemoveProjectDesignResult,
} from "../types/ipc";

/**
 * @param {number | string} designId
 * @param {number | string} projectId
 */
export async function addDesignToProject(
  designId: number | string,
  projectId: number | string
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<DesignCommandResult>("add_design_to_project", {
      designId: normalizedId,
      request: { project_id: normalizedProjectId },
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design added to project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not add design to project: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} designId
 * @param {number | string} projectId
 */
export async function removeDesignFromProject(
  designId: number | string,
  projectId: number | string
): Promise<AdapterMutationResponse> {
  const normalizedId = Number(designId);
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<DesignCommandResult>("remove_design_from_project", {
      designId: normalizedId,
      projectId: normalizedProjectId,
    });
    return {
      source: "rust",
      persisted: true,
      design_id: Number(result?.design_id ?? normalizedId),
      message: String(result?.message || "Design removed from project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      design_id: normalizedId,
      message: `Could not remove design from project: ${error}`,
      error: String(error),
    };
  }
}

export async function getBrowseProjects(): Promise<AdapterListResponse<ProjectListItem>> {
  try {
    const projects = await invokeLoose<ProjectListItem[]>("get_projects_for_browse");
    if (Array.isArray(projects)) {
      return { items: projects, source: "rust" };
    }
  } catch (error) {
    console.info("get_projects_for_browse unavailable, using mock projects.", error);
  }

  return {
    items: [
      { id: 1, name: "Project A" },
      { id: 2, name: "Project B" },
      { id: 3, name: "Project C" },
    ],
    source: "mock",
  };
}

export async function getProjectsList(
  requestTimeoutMs = 15000
): Promise<AdapterProjectListResponse> {
  const REQUEST_TIMEOUT_MS = requestTimeoutMs;

  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => {
      reject(new Error(`Timed out loading projects after ${REQUEST_TIMEOUT_MS / 1000}s.`));
    }, REQUEST_TIMEOUT_MS);
  });

  try {
    const projects = await Promise.race([
      invokeLoose<ProjectSummary[]>("get_projects_list"),
      timeoutPromise,
    ]);
    if (Array.isArray(projects)) {
      return { items: projects, source: "rust" };
    }
  } catch (error) {
    console.info("get_projects_list unavailable or timed out, using empty fallback.", error);
    return {
      items: [],
      source: "mock",
      error: `Could not load projects: ${String(error)}`,
    };
  }

  return { items: [], source: "mock" };
}

/**
 * @param {string} name
 * @param {string} description
 */
export async function createProject(
  name: string,
  description: string
): Promise<AdapterProjectMutationResponse> {
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invokeLoose<ProjectMutationResult>("create_project", { request: payload });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || 0),
      message: String(result?.message || "Project created."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: 0,
      message: `Could not create project: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} projectId
 */
export async function getProjectDetail(
  projectId: number | string
): Promise<AdapterProjectDetailResponse> {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const detail = await invokeLoose<ProjectDetailView>("get_project_detail", {
      projectId: normalizedProjectId,
    });
    if (detail && typeof detail === "object") {
      return { item: detail, source: "rust" };
    }
  } catch (error) {
    return {
      item: null,
      source: "mock",
      error: `Could not load project detail: ${error}`,
    };
  }

  return { item: null, source: "mock", error: "Project detail was empty." };
}

/**
 * @param {number | string} projectId
 * @param {string} name
 * @param {string} description
 */
export async function updateProject(
  projectId: number | string,
  name: string,
  description: string
): Promise<AdapterProjectMutationResponse> {
  const normalizedProjectId = Number(projectId);
  const payload = {
    name: String(name || "").trim(),
    description: String(description || "").trim() || null,
  };

  try {
    const result = await invokeLoose<ProjectMutationResult>("update_project", {
      projectId: normalizedProjectId,
      request: payload,
    });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      message: String(result?.message || "Project updated."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      message: `Could not update project: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} projectId
 */
export async function deleteProject(
  projectId: number | string
): Promise<AdapterProjectMutationResponse> {
  const normalizedProjectId = Number(projectId);

  try {
    const result = await invokeLoose<ProjectMutationResult>("delete_project", {
      projectId: normalizedProjectId,
    });
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      message: String(result?.message || "Project deleted."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      message: `Could not delete project: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} projectId
 * @param {number | string} designId
 */
export async function removeDesignFromProjectDetail(
  projectId: number | string,
  designId: number | string
): Promise<AdapterProjectDesignMutationResponse> {
  const normalizedProjectId = Number(projectId);
  const normalizedDesignId = Number(designId);

  try {
    const result = await invokeLoose<RemoveProjectDesignResult>(
      "remove_design_from_project_detail",
      {
        projectId: normalizedProjectId,
        designId: normalizedDesignId,
      }
    );
    return {
      source: "rust",
      persisted: true,
      project_id: Number(result?.project_id || normalizedProjectId),
      design_id: Number(result?.design_id || normalizedDesignId),
      message: String(result?.message || "Design removed from project."),
    };
  } catch (error) {
    return {
      source: "mock",
      persisted: false,
      project_id: normalizedProjectId,
      design_id: normalizedDesignId,
      message: `Could not remove design from project: ${error}`,
      error: String(error),
    };
  }
}

/**
 * @param {number | string} projectId
 */
export async function getProjectPrintView(
  projectId: number | string
): Promise<AdapterProjectDetailResponse> {
  const normalizedProjectId = Number(projectId);
  if (!Number.isFinite(normalizedProjectId) || normalizedProjectId <= 0) {
    return { item: null, source: "mock", error: `Invalid project id: ${projectId}` };
  }

  try {
    const view = await invokeLoose<ProjectDetailView>("get_project_print_view", {
      projectId: normalizedProjectId,
    });
    if (view && typeof view === "object") {
      return { item: view, source: "rust" };
    }
  } catch (error) {
    return {
      item: null,
      source: "mock",
      error: `Could not load project print view: ${error}`,
    };
  }

  return { item: null, source: "mock", error: "Project print view was empty." };
}

/**
 * Add selected designs to a project in Rust backend.
 * Falls back to local-only behavior while route wiring is in progress.
 * @param {number | string} projectId
 * @param {Array<number | string>} designIds
 */
export async function bulkAddDesignsToProject(
  projectId: number | string,
  designIds: Array<number | string>
) {
  const normalizedProjectId = Number(projectId);
  const normalizedIds =
    designIds && typeof designIds[Symbol.iterator] === "function"
      ? Array.from(designIds)
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
      : [];

  if (
    !Number.isFinite(normalizedProjectId) ||
    normalizedProjectId <= 0 ||
    normalizedIds.length === 0
  ) {
    return {
      source: "mock",
      project_id: normalizedProjectId,
      requested_count: normalizedIds.length,
      added_count: 0,
      persisted: false,
    };
  }

  try {
    const result = await invokeLoose("bulk_add_designs_to_project", {
      projectId: normalizedProjectId,
      designIds: normalizedIds,
    });
    return {
      source: "rust",
      project_id: Number(result?.project_id ?? normalizedProjectId),
      requested_count: Number(result?.requested_count ?? normalizedIds.length),
      added_count: Number(result?.added_count ?? 0),
      persisted: true,
    };
  } catch (error) {
    console.info("bulk_add_designs_to_project unavailable or failed, using local fallback.", error);
    return {
      source: "mock",
      project_id: normalizedProjectId,
      requested_count: normalizedIds.length,
      added_count: 0,
      persisted: false,
      error: String(error),
    };
  }
}
