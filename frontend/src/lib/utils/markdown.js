// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { marked } from "marked";
import DOMPurify from "dompurify";

/**
 * Map of user guidance filename (lowercased) to in-app about document slug.
 * Allows relative markdown links (e.g. `[Import Workflow](IMPORT_WORKFLOW.md)`)
 * to navigate seamlessly to `#/about/document/<slug>` within the desktop app.
 * @type {Record<string, string>}
 */
export const DOCUMENT_SLUG_BY_FILENAME = {
  "app installer.md": "app-installer",
  "backup_restore.md": "backup-restore",
  "batch_operations_backfill.md": "ai-tagging",
  "browse_bulk_actions.md": "browse-bulk-actions",
  "colour_counts.md": "colour-counts",
  "data_storage_guide.md": "data-storage",
  "design_detail.md": "design-detail",
  "first_import_actions.md": "first-import-actions",
  "getting_started.md": "getting-started",
  "image_generation.md": "image-generation",
  "importing_a_large_library.md": "importing-a-large-library",
  "import_folder_assignment.md": "import-folder-assignment",
  "import_workflow.md": "import-workflow",
  "projects.md": "projects-guide",
  "settings.md": "settings-guide",
  "stitch_types.md": "stitch-types",
  "supported_formats.md": "supported-formats",
  "tag_word_matches.md": "tag-word-matches",
};

/**
 * Resolve relative Markdown links to in-app about document routes where supported,
 * preserving absolute URLs, in-page '#' anchors, and root '/' paths.
 * Unrecognised relative file links that do not map to an embedded document are
 * neutralized to plain text so broken links are not rendered.
 * @param {string} text
 * @returns {string}
 */
export function resolveDocumentLinks(text) {
  return String(text || "").replace(/\[([^\]]+)\]\(([^)]*)\)/g, (match, label, href) => {
    const target = String(href || "").trim();
    if (
      !target ||
      target.startsWith("#") ||
      target.startsWith("/") ||
      /^[a-z][a-z0-9+.-]*:/i.test(target)
    ) {
      return match;
    }

    const basename = target.split("/").pop()?.split("\\").pop()?.trim().toLowerCase() || "";
    const matchedSlug = DOCUMENT_SLUG_BY_FILENAME[basename];
    if (matchedSlug) {
      return `[${label}](#/about/document/${matchedSlug})`;
    }

    // Unrecognised relative file link — keep label, drop link.
    return label;
  });
}

/**
 * Render Markdown text to a safe HTML string for use with {@html}.
 * @param {string} text
 * @returns {string}
 */
export function renderMarkdown(text) {
  const rawHtml = marked.parse(resolveDocumentLinks(text), { async: false });
  return DOMPurify.sanitize(rawHtml, { USE_PROFILES: { html: true } });
}
