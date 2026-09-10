<script>
  import { tick } from "svelte";
  import HelpView from "./views/HelpView.svelte";
  import AboutView from "./views/AboutView.svelte";
  import AboutDocumentView from "./views/AboutDocumentView.svelte";
  import ReferenceDataView from "./views/ReferenceDataView.svelte";
  import SystemMaintenanceView from "./views/SystemMaintenanceView.svelte";
  import BatchOperationsView from "./views/BatchOperationsView.svelte";
  import ProjectsView from "./views/ProjectsView.svelte";
  import DesignDetailView from "./views/DesignDetailView.svelte";
  import DesignPrintView from "./views/DesignPrintView.svelte";
  import ImportView from "./views/ImportView.svelte";
  import BrowseView from "./views/BrowseView.svelte";
  import { busyState } from "./stores/busyStore.js";
  import { browseSessionStore } from "./stores/browseSessionStore.js";
  import {
    normalizeHash,
    parseDesignDetailId,
    parseDesignPrintId,
    parseProjectDetailId,
    parseProjectPrintId,
    parseAboutDocumentSlug,
    resolveCurrentUiKind,
    shouldShowBackButton,
    ORDERED_ROUTE_HINTS,
    HELP_SECTION_IDS,
  } from "./utils/routing.js";

  let currentRoute = $state("");
  let previousRoute = $state("");
  let currentUiKind = $derived(resolveCurrentUiKind(currentRoute));

  // Global UI lock: while any long-running task (import, backup/restore,
  // bulk tagging, storage migration, …) is active, navigation and the footer
  // links are disabled so the user cannot route away mid-task.
  let busyActive = $derived($busyState.active);

  /** Prevent a nav/footer link from routing away while a task is running. */
  /** @param {MouseEvent} event */
  function guardNavClick(event) {
    if (busyActive) {
      event.preventDefault();
    }
  }


  // Context-aware "Back" button. Shows on the content pages (About and its
  // documents such as the Licence, and Help) when the user arrived from a
  // different route. It is hidden everywhere else: the Admin hubs (Manage Data,
  // Batch Operations, System) are self-contained destinations with their own
  // tab bars, Browse is the home page, Import / Projects are top-level
  // destinations, and the design detail / print views own their own domain Back
  // links ("Back to Browse" restores the browse search results, "Back to
  // Detail").
  let showBackButton = $derived(shouldShowBackButton(currentRoute, previousRoute));

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

  // Detail navigation browse context. `detailBrowseIds` is the ordered list of
  // design IDs matching the active browse filters (kept in the browse session
  // store and refreshed by BrowseView via `get_design_ids`); the current
  // position is DERIVED from the live route's design id rather than stored
  // separately. The store survives the browse→detail→browse round-trip, so
  // Prev/Next walk the whole filtered result set and the counter tracks the
  // current position.
  let detailBrowseIds = $derived(
    Array.isArray($browseSessionStore.designIds) ? $browseSessionStore.designIds : []
  );
  let detailBrowseIndex = $derived(
    detailDesignId == null ? -1 : detailBrowseIds.indexOf(detailDesignId)
  );

  /**
   * `ImportView` callback prop — invoked when the import wizard finishes.
   * Marks the browse list dirty so it refreshes once the user navigates back.
   *
   * @param {number} importedCount Number of designs successfully imported.
   */
  function handleImportCompleted(importedCount) {
    if (importedCount >= 1) {
      browseNeedsRefresh = true;
    }
  }

  function syncRouteFromHash() {
    const newHash = window.location.hash || "#/designs";
    const nextRoute = normalizeHash(newHash);
    if (nextRoute !== currentRoute) {
      // Navigation is allowed to proceed even while a long-running task is
      // active — this includes the active view's own step-advance / completion
      // transitions (e.g. import scan → step 2), which must NOT be blocked.
      // User-facing routing is disabled at the control layer instead, via
      // `aria-disabled` + `guardNavClick` + the disabled Back button.
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
    // Do NOT block routing while a task is busy: this function is shared with
    // the active view's own step-advance / completion navigation (e.g. the
    // import wizard calling navigateTo("#/import/step2") at the end of a scan).
    // User-facing navigation is disabled at the control layer instead, via
    // `aria-disabled` + `guardNavClick` + the disabled Back button.
    window.location.hash = target;
  }

  /** @param {string} target */
  function linkClass(target) {
    const isActive =
      currentRoute === target || (target === "#/import" && currentUiKind === "import");
    const disabled = busyActive ? " menu-link-disabled" : "";
    return `menu-link ${isActive ? "menu-link-active" : ""}${disabled}`;
  }

  /**
   * Active-highlight class for a consolidated Admin hub link. A hub link is
   * active when the current route is anywhere inside the hub's URL space, so
   * the correct hub stays highlighted as the user moves across its sub-tabs.
   * @param {string[]} hubRoots Hub URL prefixes (e.g. "#/admin/data").
   */
  function hubLinkClass(hubRoots) {
    const isActive = hubRoots.some(
      (root) => currentRoute === root || currentRoute.startsWith(`${root}/`)
    );
    const disabled = busyActive ? " menu-link-disabled" : "";
    return `menu-link menu-link-admin ${isActive ? "menu-link-active" : ""}${disabled}`;
  }

  syncRouteFromHash();
</script>

<!-- Window event — fires on every hash change and drives route state. -->
<svelte:window onhashchange={syncRouteFromHash} />

<nav class="menu-shell text-white shadow font-sans">
  <div class="menu-shell-inner max-w-7xl mx-auto flex items-center justify-between px-4 py-3">
    <div class="menu-primary-group flex items-center gap-4">
      <a
        href="#/designs"
        class="menu-brand flex items-center gap-1.5 font-bold text-lg text-white"
        aria-disabled={busyActive}
        onclick={guardNavClick}
      >
        <span aria-hidden="true">🧵</span>
        <span>Embroidery Catalogue</span>
      </a>
      <a href="#/designs" class={linkClass("#/designs")} aria-disabled={busyActive} onclick={guardNavClick}
        >Browse</a
      >
      <a href="#/import" class={linkClass("#/import")} aria-disabled={busyActive} onclick={guardNavClick}
        >Import</a
      >
      <a href="#/projects" class={linkClass("#/projects")} aria-disabled={busyActive} onclick={guardNavClick}
        >Projects</a
      >
      <a href="#/help" class={linkClass("#/help")} aria-disabled={busyActive} onclick={guardNavClick}>Help</a>
    </div>

    <div class="menu-admin-group flex items-center gap-3 text-xs text-indigo-200">
      <span class="menu-admin-label opacity-70" aria-hidden="true">Admin:</span>
      <a
        href="#/admin/data/designers"
        class={hubLinkClass(["#/admin/data"])}
        aria-disabled={busyActive}
        onclick={guardNavClick}
        >Manage Data</a
      >
      <a
        href="#/admin/batch-operations"
        class={hubLinkClass(["#/admin/batch-operations"])}
        aria-disabled={busyActive}
        onclick={guardNavClick}
        >Batch Operations</a
      >
      <a
        href="#/admin/system/settings"
        class={hubLinkClass(["#/admin/system"])}
        aria-disabled={busyActive}
        onclick={guardNavClick}
        >System</a
      >
    </div>
  </div>
</nav>

<main class="max-w-7xl mx-auto px-4 py-6 font-sans">
  {#if showBackButton}
    <div class="ui-action-button-group flex flex-wrap gap-2 mb-4 no-print">
      <button
        type="button"
        class="menu-button-secondary ui-action-button"
        onclick={goBack}
        disabled={busyActive}
        >&larr; Back</button
      >
    </div>
  {/if}
  {#if currentUiKind === "browse"}
    <BrowseView
      {navigateTo}
      bind:browseNeedsRefresh
    />
  {:else if currentUiKind === "reference-data"}
    <ReferenceDataView />
  {:else if currentUiKind === "batch-operations"}
    <BatchOperationsView />
  {:else if currentUiKind === "system"}
    <SystemMaintenanceView />
  {:else if currentUiKind === "projects-list" || currentUiKind === "project-new" || currentUiKind === "project-detail" || currentUiKind === "project-print"}
    <ProjectsView {currentUiKind} {projectDetailId} {projectPrintId} {navigateTo} />
  {:else if currentUiKind === "design-detail"}
    <!-- Event callback prop — marks the browse list dirty when a design is deleted from the detail view. -->
    <DesignDetailView
      {detailDesignId}
      {detailBrowseIds}
      {detailBrowseIndex}
      {navigateTo}
      onDesignDeleted={() => {
        browseNeedsRefresh = true;
      }}
    />
  {:else if currentUiKind === "design-print"}
    <DesignPrintView {printDesignId} {navigateTo} />
  {:else if currentUiKind === "import"}
    <ImportView {currentRoute} {navigateTo} onImportCompleted={handleImportCompleted} />
  {:else if currentUiKind === "about"}
    <AboutView />
  {:else if currentUiKind === "about-document"}
    <AboutDocumentView slug={aboutDocumentSlug} />
  {:else if currentUiKind === "help"}
    <HelpView />
  {:else}
    <div class="bg-white rounded-xl shadow p-6 space-y-4 border">
      <h1 class="ui-page-title text-2xl font-bold text-gray-800">Route Not Found</h1>
      <p class="text-gray-600">
        The requested route does not exist. Use one of the known placeholders below.
      </p>

      <div class="flex flex-wrap gap-2 pt-2">
        <button class="menu-button-primary" onclick={() => navigateTo("#/designs")}
          >Go to Browse</button
        >
      </div>

      <div
        class="border border-gray-200 rounded-lg p-4 bg-gray-50 text-sm text-gray-700 shadow-inner"
      >
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
    <a
      href="#/about"
      class="hover:underline text-indigo-650 font-medium"
      aria-disabled={busyActive}
      onclick={guardNavClick}
      >About</a
    >
    <span aria-hidden="true">•</span>
    <a
      href="#/about/licence"
      class="hover:underline text-indigo-650 font-medium"
      aria-disabled={busyActive}
      onclick={guardNavClick}
      >Licence</a
    >
  </div>
</footer>
