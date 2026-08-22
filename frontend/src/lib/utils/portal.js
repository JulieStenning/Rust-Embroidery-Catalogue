/**
 * Svelte action that portals a node to document.body.
 * Extracted from MainView.svelte.
 *
 * @param {HTMLElement} node
 */
export function portalToBody(node) {
  if (typeof document === "undefined") return {};
  const host = document.body;
  const parent = node.parentNode;
  const marker = document.createComment("main-modal-portal");
  if (parent) parent.insertBefore(marker, node);
  host.appendChild(node);
  return {
    destroy() {
      if (node.parentNode === host) host.removeChild(node);
      if (marker.parentNode) marker.parentNode.removeChild(marker);
    },
  };
}
