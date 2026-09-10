/**
 * Shared route parsing utilities for the application shell.
 * Extracted from MainView.svelte to enable isolated unit testing
 * and reduce MainView's file size.
 */

export const ORDERED_ROUTE_HINTS = [
  "#/designs",
  "#/import",
  "#/projects",
  "#/help",
  // Reference Data hub (Manage Data) — sub-tabs keep their own URLs so each
  // is deep-linkable and survives refresh / Back / Forward.
  "#/admin/data",
  "#/admin/data/designers",
  "#/admin/data/tags",
  "#/admin/data/sources",
  "#/admin/data/hoops",
  // Batch Operations destination (single URL; internally sub-tabbed).
  "#/admin/batch-operations",
  // System / Maintenance hub.
  "#/admin/system",
  "#/admin/system/settings",
  "#/admin/system/backup",
  "#/admin/system/orphans",
  "#/about",
];

export const ROUTE_UI_KIND = {
  "#/designs": "browse",
  "#/import": "import",
  "#/projects": "projects-list",
  "#/help": "help",
  "#/admin/data": "reference-data",
  "#/admin/data/designers": "reference-data",
  "#/admin/data/tags": "reference-data",
  "#/admin/data/sources": "reference-data",
  "#/admin/data/hoops": "reference-data",
  "#/admin/batch-operations": "batch-operations",
  "#/admin/system": "system",
  "#/admin/system/settings": "system",
  "#/admin/system/backup": "system",
  "#/admin/system/orphans": "system",
  "#/about": "about",
};

/** Sub-tabs shown by the Reference Data hub, in display order. */
export const REFERENCE_DATA_TABS = ["designers", "tags", "sources", "hoops"];

/** Sub-tabs shown by the System / Maintenance hub, in display order. */
export const SYSTEM_TABS = ["settings", "backup", "orphans"];

/**
 * Which Reference Data sub-view a route targets.
 * Returns one of `REFERENCE_DATA_TABS`, or `"designers"` for the bare hub
 * root `#/admin/data`, or `null` when the route is not inside this hub.
 * @param {string} route
 * @returns {string | null}
 */
export function parseReferenceDataTab(route) {
  if (route === "#/admin/data") return "designers";
  const match = route.match(/^#\/admin\/data\/(designers|tags|sources|hoops)$/);
  return match ? match[1] : null;
}

/**
 * Which System / Maintenance sub-view a route targets.
 * Returns one of `SYSTEM_TABS`, or `"settings"` for the bare hub root
 * `#/admin/system`, or `null` when the route is not inside this hub.
 * @param {string} route
 * @returns {string | null}
 */
export function parseSystemTab(route) {
  if (route === "#/admin/system") return "settings";
  const match = route.match(/^#\/admin\/system\/(settings|backup|orphans)$/);
  return match ? match[1] : null;
}

export const HELP_SECTION_IDS = new Set([
  "search",
  "importing",
  "storage",
  "ai-tagging",
  "batch-operations",
  "projects",
  "maintenance",
  "troubleshooting",
]);

/** @param {string} route */
export function parseDesignDetailId(route) {
  const match = route.match(/^#\/designs\/(\d+)$/);
  return match ? Number(match[1]) : null;
}

/** @param {string} route */
export function parseDesignPrintId(route) {
  const match = route.match(/^#\/designs\/(\d+)\/print$/);
  return match ? Number(match[1]) : null;
}

/** @param {string} route */
export function parseProjectDetailId(route) {
  const match = route.match(/^#\/projects\/(\d+)$/);
  return match ? Number(match[1]) : null;
}

/** @param {string} route */
export function parseProjectPrintId(route) {
  const match = route.match(/^#\/projects\/(\d+)\/print$/);
  return match ? Number(match[1]) : null;
}

/** @param {string} route */
export function parseAboutDocumentSlug(route) {
  if (route === "#/about/licence") return "licence";
  const match = route.match(/^#\/about\/document\/([a-z0-9-]+)$/);
  return match ? String(match[1]).toLowerCase() : null;
}

/** @param {string} route */
export function parseImportWizardStep(route) {
  if (route === "#/import") return 1;
  const match = route.match(/^#\/import\/step([123])$/);
  return match ? Number(match[1]) : null;
}

/** @param {string} route */
export function resolveCurrentUiKind(route) {
  if (parseProjectPrintId(route) !== null) return "project-print";
  if (route === "#/projects/new") return "project-new";
  if (parseProjectDetailId(route) !== null) return "project-detail";
  if (parseDesignPrintId(route) !== null) return "design-print";
  if (parseDesignDetailId(route) !== null) return "design-detail";
  if (parseAboutDocumentSlug(route) !== null) return "about-document";
  if (parseImportWizardStep(route) !== null) return "import";
  // Admin hubs are recognised by prefix so every sub-tab URL inside a hub
  // maps to the same (stable) ui kind — this keeps the hub shell mounted
  // while the user switches sub-tabs, so only the child swaps.
  if (parseReferenceDataTab(route) !== null) return "reference-data";
  if (parseSystemTab(route) !== null) return "system";
  return ROUTE_UI_KIND[/** @type {keyof typeof ROUTE_UI_KIND} */ (route)] || null;
}

/**
 * Content / reference UI kinds that receive the shared context-aware "Back"
 * button: About (and its documents, e.g. the Licence) and Help. These are the
 * pages you land on from many different places.
 *
 * All Admin hubs are deliberately excluded: Manage Data, Batch Operations and
 * the System hub (all of its tabs share the stable "system" kind) are
 * self-contained destinations with their own internal tab bars. The primary
 * work surfaces are excluded too: browse is the home page, import / projects
 * are top-level destinations, and the design detail / print views provide their
 * own domain-specific Back links ("Back to Browse", which restores the browse
 * session's search results, and "Back to Detail").
 */
export const BACK_UI_KINDS = new Set([
  "about",
  "about-document",
  "help",
]);

/**
 * Whether the shell Back button should render for the current route. Shows on
 * BACK_UI_KINDS pages AND only when the user arrived from a different route -
 * hidden on cold launch / deep link so it never points nowhere.
 * @param {string} currentRoute
 * @param {string} previousRoute
 * @returns {boolean}
 */
export function shouldShowBackButton(currentRoute, previousRoute) {
  const uiKind = resolveCurrentUiKind(currentRoute);
  if (uiKind === null || !BACK_UI_KINDS.has(uiKind)) {
    return false;
  }
  return Boolean(previousRoute) && previousRoute !== currentRoute;
}

/** @param {string} hashString */
export function normalizeHash(hashString) {
  const raw = String(hashString || "").trim();
  if (!raw.startsWith("#")) {
    return "#/designs";
  }

  const questionIndex = raw.indexOf("?");
  const path = questionIndex !== -1 ? raw.slice(0, questionIndex) : raw;
  const pathLower = path.toLowerCase();

  for (const hint of ORDERED_ROUTE_HINTS) {
    if (pathLower === hint.toLowerCase()) {
      return hint;
    }
  }

  if (path === "#/projects/new") {
    return path;
  }

  if (parseDesignDetailId(path) !== null) {
    return path;
  }
  if (parseDesignPrintId(path) !== null) {
    return path;
  }
  if (parseProjectDetailId(path) !== null) {
    return path;
  }
  if (parseProjectPrintId(path) !== null) {
    return path;
  }
  if (parseAboutDocumentSlug(path) !== null) {
    return path;
  }
  if (parseImportWizardStep(path) !== null) {
    return path;
  }

  return "#/designs";
}
