/**
 * Shared mutable holder so tests can override the result passed to
 * `onDeleted` when the DeleteDesignsModal mock's Confirm button is clicked.
 * BrowseView.test.ts imports this same module and sets `value` before
 * clicking Confirm to exercise alternate branches of `handleBulkDeleteResult`.
 */
/** @type {{ value: any }} */
export const deleteResultHolder = {
  value: null,
};
