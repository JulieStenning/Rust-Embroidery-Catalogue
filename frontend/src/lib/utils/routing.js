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
  "#/admin/designers",
  "#/admin/tags",
  "#/admin/sources",
  "#/admin/hoops",
  "#/admin/settings",
  "#/admin/maintenance/backup",
  "#/admin/tagging-actions",
  "#/admin/orphans",
  "#/about",
];

export const ROUTE_UI_KIND = {
  "#/designs": "browse",
  "#/import": "import",
  "#/projects": "projects-list",
  "#/help": "help",
  "#/admin/designers": "admin-list",
  "#/admin/tags": "admin-list",
  "#/admin/sources": "admin-list",
  "#/admin/hoops": "admin-list",
  "#/admin/settings": "settings",
  "#/admin/maintenance/backup": "backup",
  "#/admin/tagging-actions": "tagging-actions",
  "#/admin/orphans": "orphans",
  "#/about": "about",
};

export const HELP_SECTION_IDS = new Set([
  "search",
  "importing",
  "storage",
  "ai-tagging",
  "tagging-actions",
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
  return ROUTE_UI_KIND[/** @type {keyof typeof ROUTE_UI_KIND} */ (route)] || null;
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
