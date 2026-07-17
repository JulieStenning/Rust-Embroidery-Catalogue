<script>
  import { onMount, untrack } from "svelte";
  import {
    getProjectsList,
    createProject,
    getProjectDetail,
    updateProject,
    deleteProject,
    removeDesignFromProjectDetail,
    getProjectPrintView
  } from "../api/commandAdapter.js";

  let { currentUiKind, projectDetailId, projectPrintId, navigateTo } = $props();

  /** @type {any[]} */
  let projectsItems = $state([]);
  let projectsSource = $state("mock");
  let projectsLoading = $state(false);
  let projectsLoaded = $state(false);
  let projectsError = $state("");
  let projectsLoadRequestToken = 0;
  let projectsActionMessage = $state("");
  let projectsActionIsError = $state(false);

  let projectNewName = $state("");
  let projectNewDescription = $state("");
  let projectNewSaving = $state(false);

  /** @type {{ project: { id: number, name: string, description?: string }, designs: any[] } | null} */
  let projectDetail = $state(null);
  let projectDetailSource = $state("mock");
  let projectDetailLoading = $state(false);
  let projectDetailError = $state("");
  let projectDetailSaving = $state(false);
  let projectDetailName = $state("");
  let projectDetailDescription = $state("");
  let projectDetailOriginalName = $state("");
  let projectDetailOriginalDescription = $state("");

  /** @type {{ project: { name: string, description?: string }, designs: any[] } | null} */
  let projectPrint = $state(null);
  let projectPrintSource = $state("mock");
  let projectPrintLoading = $state(false);
  let projectPrintError = $state("");

  let projectDetailHasChanges = $derived(
    projectDetailName !== projectDetailOriginalName ||
    projectDetailDescription !== projectDetailOriginalDescription
  );

  /**
   * @param {string} message
   * @param {boolean} [isError]
   */
  function setProjectsNotice(message, isError = false) {
    projectsActionMessage = message;
    projectsActionIsError = isError;
  }

  async function loadProjects(force = false) {
    if (projectsLoading && !force) return;

    const requestToken = ++projectsLoadRequestToken;
    const timeoutMs = 15000;

    projectsLoading = true;
    projectsError = "";

    try {
      const timeoutResult = new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            items: [],
            source: "mock",
            error: `Could not load projects: Timed out loading projects after ${timeoutMs / 1000}s.`,
          });
        }, timeoutMs);
      });

      const result = await Promise.race([getProjectsList(), timeoutResult]);

      if (requestToken !== projectsLoadRequestToken) return;

      projectsItems = Array.isArray(result?.items) ? result.items : [];
      projectsSource = result?.source || "mock";
      if (result?.error) {
        projectsError = String(result.error);
      }
      projectsLoaded = true;
    } catch (error) {
      if (requestToken !== projectsLoadRequestToken) return;
      projectsItems = [];
      projectsSource = "mock";
      projectsError = `Could not load projects: ${error}`;
    } finally {
      if (requestToken === projectsLoadRequestToken) {
        projectsLoading = false;
      }
    }
  }

  async function submitNewProject() {
    if (projectNewSaving) return;

    const name = String(projectNewName || "").trim();
    if (!name) {
      setProjectsNotice("Project name is required.", true);
      return;
    }

    projectNewSaving = true;
    const result = await createProject(name, projectNewDescription);
    projectNewSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      projectNewName = "";
      projectNewDescription = "";
      await loadProjects(true);
      navigateTo("#/projects");
    }
  }

  /**
   * @param {number | string} projectId
   */
  async function loadProjectDetailView(projectId) {
    if (projectId == null) return;

    projectDetailLoading = true;
    projectDetailError = "";

    try {
      const result = await getProjectDetail(projectId);
      if (projectId !== projectDetailId) return;

      projectDetail = result?.item || null;
      projectDetailSource = result?.source || "mock";

      if (!projectDetail) {
        projectDetailError = result?.error || `Could not load project ${projectId}.`;
        projectDetailName = "";
        projectDetailDescription = "";
        projectDetailOriginalName = "";
        projectDetailOriginalDescription = "";
      } else {
        projectDetailName = String(projectDetail?.project?.name || "");
        projectDetailDescription = String(projectDetail?.project?.description || "");
        projectDetailOriginalName = projectDetailName;
        projectDetailOriginalDescription = projectDetailDescription;
      }
    } catch (error) {
      if (projectId !== projectDetailId) return;
      projectDetail = null;
      projectDetailSource = "mock";
      projectDetailError = `Could not load project detail: ${error}`;
      projectDetailName = "";
      projectDetailDescription = "";
    } finally {
      if (projectId === projectDetailId) {
        projectDetailLoading = false;
      }
    }
  }

  async function refreshProjectDetailView() {
    if (!projectDetailId) return;
    await loadProjectDetailView(projectDetailId);
    await loadProjects(true);
  }

  function undoProjectDetailChanges() {
    projectDetailName = projectDetailOriginalName;
    projectDetailDescription = projectDetailOriginalDescription;
  }

  async function saveProjectDetail() {
    if (!projectDetail?.project?.id || projectDetailSaving) return;

    const name = String(projectDetailName || "").trim();
    if (!name) {
      setProjectsNotice("Project name is required.", true);
      return;
    }

    projectDetailSaving = true;
    const result = await updateProject(projectDetail.project.id, name, projectDetailDescription);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      projectDetailOriginalName = name;
      projectDetailOriginalDescription = projectDetailDescription;
      await refreshProjectDetailView();
    }
  }

  async function confirmDeleteProject() {
    if (!projectDetail?.project?.id || projectDetailSaving) return;

    const confirmed = window.confirm(
      `Delete project "${projectDetail.project.name || ""}"? This cannot be undone.`
    );
    if (!confirmed) return;

    projectDetailSaving = true;
    const result = await deleteProject(projectDetail.project.id);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      await loadProjects(true);
      navigateTo("#/projects");
    }
  }

  /**
   * @param {number | string} designId
   */
  async function removeDesignFromProjectMembership(designId) {
    if (!projectDetail?.project?.id || !designId || projectDetailSaving) return;

    projectDetailSaving = true;
    const result = await removeDesignFromProjectDetail(projectDetail.project.id, designId);
    projectDetailSaving = false;

    setProjectsNotice(result.message, !result.persisted);
    if (result.persisted) {
      await refreshProjectDetailView();
    }
  }

  /**
   * @param {number | string} projectId
   */
  async function loadProjectPrint(projectId) {
    if (projectId == null) return;

    projectPrintLoading = true;
    projectPrintError = "";

    try {
      const result = await getProjectPrintView(projectId);
      if (projectId !== projectPrintId) return;

      projectPrint = result?.item || null;
      projectPrintSource = result?.source || "mock";
      if (!projectPrint) {
        projectPrintError = result?.error || `Could not load project print view for id ${projectId}.`;
      }
    } catch (error) {
      if (projectId !== projectPrintId) return;
      projectPrint = null;
      projectPrintSource = "mock";
      projectPrintError = `Could not load project print view: ${error}`;
    } finally {
      if (projectId === projectPrintId) {
        projectPrintLoading = false;
      }
    }
  }

  function printCurrentView() {
    window.print();
  }

  /**
   * @param {number | string | null | undefined} rating
   */
  function ratingToStars(rating) {
    const numeric = Number(rating);
    if (!Number.isFinite(numeric) || numeric <= 0) return "";
    const clamped = Math.min(5, Math.max(0, numeric));
    return `${"★".repeat(clamped)}${"☆".repeat(5 - clamped)}`;
  }

  // Reactive loaders based on routing
  $effect(() => {
    if (currentUiKind === "projects-list") {
      untrack(() => {
        loadProjects(true);
      });
    }
  });

  $effect(() => {
    if (currentUiKind === "project-detail" && projectDetailId !== null) {
      untrack(() => {
        loadProjectDetailView(projectDetailId);
      });
    }
  });

  $effect(() => {
    if (currentUiKind === "project-print" && projectPrintId !== null) {
      untrack(() => {
        loadProjectPrint(projectPrintId);
      });
    }
  });
</script>

{#if currentUiKind === "projects-list"}
  <section class="projects-page space-y-4">
    <div class="projects-header flex items-center justify-between gap-3 font-sans">
      <h1 class="ui-page-title projects-title text-2xl font-bold text-gray-800">Projects</h1>
      <button class="menu-button-primary" onclick={() => navigateTo("#/projects/new")}>+ New Project</button>
    </div>

    <p class="projects-intro text-sm text-gray-500">
      <br />
      Group designs for a planned embroidery task - for example a seasonal set or a quilt block series.
      <a href="#/help?section=projects" class="text-indigo-600 hover:underline">Learn more</a>
    </p>

    {#if projectsActionMessage}
      <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
        {projectsActionMessage}
      </div>
    {/if}

    <div class="projects-shell">
      {#if projectsLoading}
        <p>Loading projects...</p>
      {:else if projectsError}
        <p class="text-red-600">{projectsError}</p>
      {:else if !projectsItems || projectsItems.length === 0}
        <p class="projects-empty text-gray-500">
          No projects yet.
          <button class="text-indigo-600 hover:underline" onclick={() => navigateTo("#/projects/new")}>Create one</button>.
        </p>
      {:else}
        <div class="projects-grid">
          {#each projectsItems as project}
            <a
              class="projects-tile text-left block"
              href={`#/projects/${project.id}`}
              aria-label={`Open project ${project.name}`}
            >
              <div class="projects-tile-top flex items-start justify-between gap-3">
                <h2 class="projects-tile-title font-semibold text-gray-800">{project.name}</h2>
                <span class="projects-count-badge">{Number(project.design_count || 0)} design{Number(project.design_count || 0) === 1 ? "" : "s"}</span>
              </div>
              {#if project.description}
                <p class="projects-tile-description text-sm text-gray-500 mt-1">{project.description}</p>
              {/if}
              {#if project.date_created}
                <p class="projects-tile-meta text-xs text-gray-400 mt-2">Created {project.date_created}</p>
              {/if}
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </section>
{:else if currentUiKind === "project-new"}
  <section class="projects-page space-y-4 font-sans">
    <div>
      <button class="projects-back-link text-indigo-600 text-sm hover:underline" onclick={() => navigateTo("#/projects")}>← Projects</button>
    </div>

    <div class="projects-form-card space-y-3 bg-white rounded shadow p-6 max-w-xl">
      <h2 class="projects-subtitle text-2xl font-bold text-gray-800">New Project</h2>
      <p class="projects-intro text-sm text-gray-500 font-sans">
        Projects let you group designs for a planned embroidery task.
        <a href="#/help?section=projects" class="text-indigo-600 hover:underline font-medium">Help</a>
      </p>

      {#if projectsActionMessage}
        <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
          {projectsActionMessage}
        </div>
      {/if}

      <form
        class="space-y-3"
        onsubmit={(event) => {
          event.preventDefault();
          submitNewProject();
        }}
      >
        <label class="projects-label block text-sm text-gray-700">
          <span class="block font-medium mb-1">Name *</span>
          <input
            type="text"
            class="projects-input w-full border rounded px-3 py-1.5 text-sm"
            bind:value={projectNewName}
            required
            placeholder="e.g. Christmas Stockings 2024"
          />
        </label>
        <label class="projects-label block text-sm text-gray-700">
          <span class="block font-medium mb-1">Description</span>
          <textarea
            rows="3"
            class="projects-input projects-textarea w-full border rounded px-3 py-1.5 text-sm"
            bind:value={projectNewDescription}
            placeholder="Optional notes, goals, or deadline"
          ></textarea>
        </label>
        <button type="submit" class="menu-button-primary" disabled={projectNewSaving || !String(projectNewName || "").trim()}>
          {projectNewSaving ? "Creating..." : "Create Project"}
        </button>
      </form>
    </div>
  </section>
{:else if currentUiKind === "project-detail"}
  <section class="projects-page space-y-4 font-sans">
    <div class="projects-detail-top flex items-center justify-between gap-3 no-print">
      <button class="projects-back-link text-indigo-600 text-sm hover:underline" onclick={() => navigateTo("#/projects")}>← Projects</button>
      <div class="flex flex-wrap gap-3">
        {#if projectDetail?.project?.id}
          <button class="projects-action-link text-sm text-gray-600 hover:underline" onclick={() => navigateTo(`#/projects/${projectDetail?.project?.id}/print`)}>Print Sheet</button>
        {/if}
        <button class="projects-danger-link text-sm text-red-500 hover:underline" onclick={confirmDeleteProject} disabled={projectDetailSaving || !projectDetail?.project?.id}>Delete Project</button>
      </div>
    </div>

    {#if projectsActionMessage}
      <div class={`rounded border px-3 py-2 text-sm ${projectsActionIsError ? "bg-red-50 border-red-200 text-red-700" : "bg-green-50 border-green-200 text-green-700"}`}>
        {projectsActionMessage}
      </div>
    {/if}

    <div class="projects-form-card bg-white rounded shadow p-6">
      {#if projectDetailLoading}
        <p>Loading project...</p>
      {:else if projectDetailError}
        <p class="text-red-600">{projectDetailError}</p>
      {:else if !projectDetail?.project}
        <p>Project not found.</p>
      {:else}
        <form
          class="space-y-3"
          onsubmit={(event) => {
            event.preventDefault();
            saveProjectDetail();
          }}
        >
          <input
            type="text"
            class="projects-title-input text-2xl font-bold border-b w-full focus:outline-none py-1 text-gray-800"
            bind:value={projectDetailName}
            required
          />
          <textarea
            rows="2"
            class="projects-input projects-textarea w-full border rounded px-2 py-1 text-sm focus:outline-none text-gray-700"
            bind:value={projectDetailDescription}
            placeholder="Description..."
          ></textarea>
          <button type="submit" class="menu-button-primary font-medium" disabled={projectDetailSaving || !projectDetailHasChanges}>
            {projectDetailSaving ? "Saving..." : "Save"}
          </button>
          <button
            type="button"
            class="menu-button-secondary font-medium"
            disabled={projectDetailSaving || !projectDetailHasChanges}
            onclick={undoProjectDetailChanges}
          >
            Undo
          </button>
        </form>
      {/if}
    </div>

    {#if projectDetail?.project}
      <div class="space-y-3">
        <h2 class="text-lg font-semibold text-gray-800">Designs ({Array.isArray(projectDetail?.designs) ? projectDetail.designs.length : 0})</h2>
        {#if Array.isArray(projectDetail.designs) && projectDetail.designs.length > 0}
          <div class="projects-design-grid grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mb-6">
            {#each projectDetail.designs as design}
              <div class="projects-design-card bg-white rounded shadow overflow-hidden flex flex-col hover:shadow-md transition">
                <a class="projects-design-link text-left block" href={`#/designs/${design.id}`} aria-label={`Open design ${design.filename}`}>
                  {#if design.image_data_url}
                    <img src={design.image_data_url} alt={design.filename} class="projects-design-image w-full h-32 object-contain bg-gray-50" loading="lazy" />
                  {:else if design.has_image}
                    <div class="projects-design-preview w-full h-32 bg-gray-100 flex items-center justify-center text-gray-700 text-xs">Image unavailable</div>
                  {:else}
                    <div class="projects-design-preview-empty w-full h-32 bg-gray-200 flex items-center justify-center text-gray-400 text-xs">No image</div>
                  {/if}
                </a>
                <div class="projects-design-meta p-2 flex-1 flex flex-col">
                  <a class="projects-design-title-link text-xs font-semibold text-gray-800 truncate hover:text-indigo-600" href={`#/designs/${design.id}`}>
                    {design.filename}
                  </a>
                  {#if design.designer_name}
                    <p class="text-[11px] text-gray-500">{design.designer_name}</p>
                  {/if}
                  <button class="text-xs text-red-400 hover:text-red-600 hover:underline mt-auto pt-2 text-left font-medium" onclick={() => removeDesignFromProjectMembership(design.id)} disabled={projectDetailSaving}>Remove</button>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-gray-500 mb-6 font-sans">No designs in this project yet.</p>
        {/if}
      </div>
    {/if}
  </section>
{:else if currentUiKind === "project-print"}
  <section class="projects-page space-y-3 projects-print-page font-sans">
    <div class="flex flex-wrap gap-2 no-print">
      <button class="menu-button-secondary font-medium" onclick={() => navigateTo(`#/projects/${projectPrintId}`)}>Back to Project</button>
      <button class="menu-button-primary font-medium" onclick={printCurrentView}>Print</button>
    </div>

    <div class="projects-print-shell print:p-0 print:shadow-none print:border-none">
      {#if projectPrintLoading}
        <p>Loading printable project sheet...</p>
      {:else if projectPrintError}
        <p class="text-red-600">{projectPrintError}</p>
      {:else if !projectPrint?.project}
        <p>Project not found.</p>
      {:else}
        <div class="space-y-4">
          <h2 class="text-2xl font-bold text-gray-800">{projectPrint.project.name}</h2>
          {#if projectPrint.project.description}
            <p class="text-sm text-gray-600">{projectPrint.project.description}</p>
          {/if}

          <div class="space-y-3">
            {#if Array.isArray(projectPrint.designs) && projectPrint.designs.length > 0}
              {#each projectPrint.designs as design}
                <div class="projects-print-card border border-gray-200 rounded p-3 flex gap-4 bg-white print:break-inside-avoid shadow-sm">
                  {#if design.image_data_url}
                    <img src={design.image_data_url} alt={design.filename} class="projects-print-image w-40 h-40 object-contain bg-gray-100 rounded" />
                  {:else}
                    <div class="projects-print-image projects-design-preview-empty w-40 h-40 bg-gray-200 flex items-center justify-center text-gray-400 text-xs rounded">No image</div>
                  {/if}
                  <div class="text-sm space-y-1 flex-1">
                    <h3 class="font-bold text-lg text-gray-800">{design.filename}</h3>
                    {#if design.width_mm != null && design.height_mm != null}
                      <p class="text-gray-700"><strong>Size:</strong> {design.width_mm} x {design.height_mm} mm</p>
                    {/if}
                    {#if design.hoop}
                      <p class="text-gray-700"><strong>Hoop:</strong> {design.hoop}</p>
                    {/if}
                    {#if design.stitch_count != null}
                      <p class="text-gray-700"><strong>Stitches:</strong> {design.stitch_count}</p>
                    {/if}
                    {#if design.color_count != null}
                      <p class="text-gray-700"><strong>Colours:</strong> {design.color_count}</p>
                    {/if}
                    {#if design.color_change_count != null}
                      <p class="text-gray-700"><strong>Colour changes:</strong> {design.color_change_count}</p>
                    {/if}
                    {#if design.designer_name}
                      <p class="text-gray-700"><strong>Designer:</strong> {design.designer_name}</p>
                    {/if}
                    {#if design.rating}
                      <p class="text-gray-700"><strong>Rating:</strong> <span class="text-yellow-500 font-bold">{ratingToStars(design.rating)}</span></p>
                    {/if}
                    {#if design.is_stitched}
                      <p class="text-gray-700"><strong>Stitched:</strong> Yes</p>
                    {/if}
                    {#if design.notes}
                      <p class="italic text-gray-600 bg-gray-50 border-l-2 border-indigo-200 pl-2 py-1">{design.notes}</p>
                    {/if}
                  </div>
                </div>
              {/each}
            {:else}
              <p class="text-gray-500">No designs in this project yet.</p>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </section>
{/if}
