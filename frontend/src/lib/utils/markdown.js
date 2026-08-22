import { marked } from "marked";
import DOMPurify from "dompurify";

/**
 * Strip relative Markdown links so we don't render dead links to sibling
 * .md files that don't exist in the app. Absolute URL links, "#" anchors,
 * and "/" root paths are preserved.
 * @param {string} text
 * @returns {string}
 */
function neutralizeRelativeLinks(text) {
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
    // Relative path (e.g. STITCH_TYPES.md, ../TROUBLESHOOTING.md) — keep label, drop link.
    return label;
  });
}

/**
 * Render Markdown text to a safe HTML string for use with {@html}.
 * @param {string} text
 * @returns {string}
 */
export function renderMarkdown(text) {
  const rawHtml = marked.parse(neutralizeRelativeLinks(text), { async: false });
  return DOMPurify.sanitize(rawHtml, { USE_PROFILES: { html: true } });
}
