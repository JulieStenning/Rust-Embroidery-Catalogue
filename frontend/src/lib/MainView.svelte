<script>
  import { tick } from "svelte";
  import HelpView from "./views/HelpView.svelte";
  import AboutView from "./views/AboutView.svelte";
  import AboutDocumentView from "./views/AboutDocumentView.svelte";
  import SettingsView from "./views/SettingsView.svelte";
  import BackupView from "./views/BackupView.svelte";
  import TaggingActionsView from "./views/TaggingActionsView.svelte";
  import OrphansView from "./views/OrphansView.svelte";
  import ProjectsView from "./views/ProjectsView.svelte";
  import DesignDetailView from "./views/DesignDetailView.svelte";
  import DesignPrintView from "./views/DesignPrintView.svelte";
  import ImportView from "./views/ImportView.svelte";
  import TagsView from "./views/TagsView.svelte";
  import BrowseView from "./views/BrowseView.svelte";
  import AdminDesignersView from "./views/AdminDesignersView.svelte";
  import AdminSourcesView from "./views/AdminSourcesView.svelte";
  import AdminHoopsView from "./views/AdminHoopsView.svelte";
  import {
    normalizeHash,
    parseDesignDetailId,
    parseDesignPrintId,
    parseProjectDetailId,
    parseProjectPrintId,
    parseAboutDocumentSlug,
    parseImportWizardStep,
    resolveCurrentUiKind,
    ORDERED_ROUTE_HINTS,
    HELP_SECTION_IDS,
  } from "./utils/routing.js";

  let currentRoute = $state("");
  let previousRoute = $state("");
  let currentUiKind = $derived(resolveCurrentUiKind(currentRoute));

  // Utility/reference pages (About, Licensing/AI-Tagging docs, Help, Settings)
  // are cross-linked from many places. Show a context-aware "Back" button that
  // returns to the page the user actually came from (e.g. Import step 3), and
  // hide it when there is no previous route (e.g. app launched directly here).
  const UTILITY_UI_KINDS_WITH_BACK = new Set(["settings", "about", "about-document", "help"]);
  let showBackButton = $derived(
    UTILITY_UI_KINDS_WITH_BACK.has(currentUiKind) &&
    Boolean(previousRoute) &&
    previousRoute !== currentRoute
  );

  /** Return to the page the user came from. */
  function goBack() {
    if (previousRoute) {
      navigateTo(previousRoute);
    }
  }
  let detailDesignId = $derived(parseDesignDetailId(currentRoute));
  let printDesignId = $derived(parseDesignPrintId(currentRoute));
  let projectDetailId = $derived(parseProjectDetailId(currentRoute));
  let projectPrintId = $derived(parseProjectPrintId(currentRoute));
  let aboutDocumentSlug = $derived(parseAboutDocumentSlug(currentRoute));

  let browseNeedsRefresh = $state(false);

  // Detail navigation browse context (bridged to BrowseView via bindable props)
  /** @type {number[]} */
  let detailBrowseIds = $state([]);
  let detailBrowseIndex = $state(-1);

  /** @param {number} importedCount */
  function handleImportCompleted(importedCount) {
    if (importedCount >= 1) {
      browseNeedsRefresh = true;
    }
  }

  function syncRouteFromHash() {
    const newHash = window.location.hash || "#/designs";
    const nextRoute = normalizeHash(newHash);
    if (nextRoute !== currentRoute) {
      previousRoute = currentRoute;
      currentRoute = nextRoute;
    }

    const questionIndex = newHash.indexOf("?");
    if (nextRoute === "#/help" && questionIndex !== -1) {
      const queryParams = new URLSearchParams(newHash.slice(questionIndex));
      const section = queryParams.get("section");
      if (section && HELP_SECTION_IDS.has(section)) {
        tick().then(() => {
          setTimeout(() => {
            const el = document.getElementById(section);
            if (el) {
              el.scrollIntoView({ behavior: "smooth", block: "start" });
            }
          }, 150);
        });
      }
    }
  }

  /** @param {string} target */
  function navigateTo(target) {
    window.location.hash = target;
  }

  /** @param {string} target */
  function linkClass(target) {
    const isActive = currentRoute === target || (target === "#/import" && currentUiKind === "import");
    return `menu-link ${isActive ? "menu-link-active" : ""}`;
  }

  /** @param {string} target */
  function adminLinkClass(target) {
    const isActive = currentRoute === target;
    return `menu-link menu-link-admin ${isActive ? "menu-link-active" : ""}`;
  }

  syncRouteFromHash();
</script>

<svelte:window onhashchange={syncRouteFromHash} />

<nav class="menu-shell text-white shadow font-sans">
  <div class="menu-shell-inner max-w-7xl mx-auto flex items-center justify-between px-4 py-3">
    <div class="menu-primary-group flex items-center gap-4">
      <a href="#/designs" class="menu-brand flex items-center gap-1.5 font-bold text-lg text-white">
        <span aria-hidden="true">🧵</span>
        <span>Embroidery Catalogue</span>
      </a>
      <a href="#/designs" class={linkClass("#/designs")}>Browse</a>
      <a href="#/import" class={linkClass("#/import")}>Import</a>
      <a href="#/projects" class={linkClass("#/projects")}>Projects</a>
      <a href="#/help" class={linkClass("#/help")}>Help</a>
    </div>

    <div class="menu-admin-group flex items-center gap-3 text-xs text-indigo-200">
      <span class="menu-admin-label opacity-70" aria-hidden="true">Admin:</span>
      <a href="#/admin/designers" class={adminLinkClass("#/admin/designers")}>Designers</a>
      <a href="#/admin/tags" class={adminLinkClass("#/admin/tags")}>Tags</a>
      <a href="#/admin/sources" class={adminLinkClass("#/admin/sources")}>Sources</a>
      <a href="#/admin/hoops" class={adminLinkClass("#/admin/hoops")}>Hoops</a>
      <a href="#/admin/settings" class={adminLinkClass("#/admin/settings")}>Settings</a>
      <a href="#/admin/maintenance/backup" class={adminLinkClass("#/admin/maintenance/backup")}>Backup</a>
      <a href="#/admin/tagging-actions" class={adminLinkClass("#/admin/tagging-actions")}>Tagging Actions</a>
      <a href="#/admin/orphans" class={adminLinkClass("#/admin/orphans")}>Orphans</a>
    </div>
  </div>
</nav>

<main class="max-w-7xl mx-auto px-4 py-6 font-sans">
  {#if showBackButton}
    <div class="ui-action-button-group flex flex-wrap gap-2 mb-4 no-print">
      <button type="button" class="menu-button-secondary ui-action-button" onclick={goBack}>&larr; Back</button>
    </div>
  {/if}
  {#if currentUiKind === "browse"}
    <BrowseView
      {navigateTo}
      {detailDesignId}
      bind:browseNeedsRefresh
      bind:detailBrowseIds
      bind:detailBrowseIndex
    />
  {:else if currentUiKind === "settings"}
    <SettingsView />
  {:else if currentUiKind === "backup"}
    <BackupView />
  {:else if currentUiKind === "tagging-actions"}
    <TaggingActionsView />
  {:else if currentUiKind === "orphans"}
    <OrphansView />
  {:else if currentUiKind === "projects-list" || currentUiKind === "project-new" || currentUiKind === "project-detail" || currentUiKind === "project-print"}
    <ProjectsView
      {currentUiKind}
      {projectDetailId}
      {projectPrintId}
      {navigateTo}
    />
  {:else if currentUiKind === "design-detail"}
    <DesignDetailView
      {detailDesignId}
      {detailBrowseIds}
      {detailBrowseIndex}
      {navigateTo}
      onDesignDeleted={() => { browseNeedsRefresh = true; }}
    />
  {:else if currentUiKind === "design-print"}
    <DesignPrintView
      {printDesignId}
      {navigateTo}
    />
  {:else if currentUiKind === "import"}
    <ImportView
      {currentRoute}
      {navigateTo}
      onImportCompleted={handleImportCompleted}
    />
  {:else if currentUiKind === "about"}
    <AboutView />
  {:else if currentUiKind === "about-document"}
    <AboutDocumentView
      slug={aboutDocumentSlug}
    />
  {:else if currentUiKind === "help"}
    <HelpView />
  {:else if currentUiKind === "admin-list" && currentRoute === "#/admin/designers"}
    <AdminDesignersView />
  {:else if currentUiKind === "admin-list" && currentRoute === "#/admin/tags"}
    <TagsView />
  {:else if currentUiKind === "admin-list" && currentRoute === "#/admin/sources"}
    <AdminSourcesView />
  {:else if currentUiKind === "admin-list" && currentRoute === "#/admin/hoops"}
    <AdminHoopsView />
  {:else}
    <div class="bg-white rounded-xl shadow p-6 space-y-4 border">
      <h1 class="ui-page-title text-2xl font-bold text-gray-800">Route Not Found</h1>
      <p class="text-gray-600">
        The requested route does not exist. Use one of the known placeholders below.
      </p>

      <div class="flex flex-wrap gap-2 pt-2">
        <button class="menu-button-primary" onclick={() => navigateTo("#/designs")}>Go to Browse</button>
      </div>

      <div class="border border-gray-200 rounded-lg p-4 bg-gray-50 text-sm text-gray-700 shadow-inner">
        <p class="font-semibold mb-2">Known routes</p>
        <ul class="space-y-1">
          {#each ORDERED_ROUTE_HINTS as route}
            <li>{route}</li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}
</main>

<footer class="max-w-7xl mx-auto px-4 pb-6 text-xs text-gray-500">
  <div class="border-t border-gray-300 pt-4 flex flex-wrap items-center gap-x-3 gap-y-1">
    <span>Embroidery Catalogue</span>
    <span aria-hidden="true">•</span>
    <a href="#/about" class="hover:underline text-indigo-650 font-medium">About</a>
    <span aria-hidden="true">•</span>
    <a href="#/about/document/licence" class="hover:underline text-indigo-650 font-medium">Licence</a>
  </div>
</footer>