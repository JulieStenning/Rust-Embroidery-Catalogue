# Browse Bulk Actions UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-28
- Backend companion: [docs/Specs/browse-bulk-actions-backend-spec.md](docs/Specs/browse-bulk-actions-backend-spec.md)
- Companion checklist: [docs/Specs/browse-bulk-actions-refactor-checklist.md](docs/Specs/browse-bulk-actions-refactor-checklist.md)

## Purpose
Define the Browse-page bottom-banner UI contract for multi-selection actions so UI changes can be implemented consistently without mixing backend internals into UI requirements.

## Scope
In scope:
- Bottom-banner controls, layout role, and visibility rules.
- Bulk tag modal interaction contract.
- Selection interactions tied to card, row, and page-level controls.
- Pagination and selection-state behavior.
- Accessibility and responsive behavior specific to the bottom-banner surface.

Out of scope:
- Backend route/service implementation details.
- Browse search/filter query semantics outside selection behavior.

## Source of Truth
- [templates/designs/browse.html](templates/designs/browse.html)
- [src/routes/designs.py](src/routes/designs.py)
- [tests/test_routes.py](tests/test_routes.py)
- [docs/feature-inventory.md](docs/feature-inventory.md)

## UI Anatomy

### 1. Activation Context
- The bottom banner is hidden by default.
- The banner appears once one or more design checkboxes are selected.
- The selected count is displayed in banner text.

Anchors:
- banner container: [templates/designs/browse.html#L449](templates/designs/browse.html#L449)
- selection count update: [templates/designs/browse.html#L712](templates/designs/browse.html#L712)

### 2. Banner Controls
The bottom banner includes, in display order:
- selected count text
- Choose tags button
- Verify selected button
- project dropdown
- Add to project button
- Clear selection button

Anchors:
- Choose tags: [templates/designs/browse.html#L452](templates/designs/browse.html#L452)
- Verify selected: [templates/designs/browse.html#L456](templates/designs/browse.html#L456)
- project dropdown: [templates/designs/browse.html#L461](templates/designs/browse.html#L461)
- Add to project: [templates/designs/browse.html#L473](templates/designs/browse.html#L473)
- Clear selection: [templates/designs/browse.html#L479](templates/designs/browse.html#L479)

### 3. Bulk Tag Modal Surface
- Choose tags opens an in-page modal (not route navigation).
- Modal title reflects current selected count.
- Tag choices are grouped and scrollable.
- Apply tags and Cancel actions are explicit.

Anchors:
- modal container: [templates/designs/browse.html#L486](templates/designs/browse.html#L486)
- open modal function: [templates/designs/browse.html#L728](templates/designs/browse.html#L728)
- apply action: [templates/designs/browse.html#L825](templates/designs/browse.html#L825)

## State Model (UI)

### Default State
- No card selected.
- Bottom banner hidden.
- Modal hidden.

### Selection Active State
- One or more cards selected.
- Bottom banner visible.
- Selected count updates live.

### Project-Unavailable State
- Project dropdown is disabled when no projects exist.
- Add-to-project button is disabled in the same state.

Anchor:
- disabled dropdown/button rendering: [templates/designs/browse.html#L461](templates/designs/browse.html#L461)

### Modal Active State
- Modal overlays page with backdrop.
- Backdrop click closes modal.
- Escape closes modal.
- Modal close button and cancel button close modal.

Anchors:
- close handlers: [templates/designs/browse.html#L818](templates/designs/browse.html#L818)

## Interaction Contract

### Selection Interactions
- Card-level checkbox toggles per design.
- Select-all toggles all visible cards on the current page.
- Row selectors (injected via JS) toggle all cards in visual row.
- Clear selection unchecks all card checkboxes.

Anchors:
- card selection checkbox: [templates/designs/browse.html#L342](templates/designs/browse.html#L342)
- select-all checkbox: [templates/designs/browse.html#L328](templates/designs/browse.html#L328)
- row selector build: [templates/designs/browse.html#L606](templates/designs/browse.html#L606)
- clear selection handler: [templates/designs/browse.html#L848](templates/designs/browse.html#L848)

### Choose Tags Interactions
- Opening modal requires selection.
- Modal pre-checks tag boxes that are common to all selected designs (intersection behavior).
- Applying tags submits selected IDs and chosen tag IDs.

Note:
Current behavior is replace-set tagging for selected designs, not additive tagging.

Anchors:
- intersection computation: [templates/designs/browse.html#L728](templates/designs/browse.html#L728)
- hidden form submission target: [templates/designs/browse.html#L427](templates/designs/browse.html#L427)

### Verify Selected Interactions
- Verify selected submits selected IDs to verify endpoint.
- Action does not require modal.

Anchors:
- verify button: [templates/designs/browse.html#L456](templates/designs/browse.html#L456)
- verify submit assembly: [templates/designs/browse.html#L783](templates/designs/browse.html#L783)

### Add to Project Interactions
- Action requires selection plus non-empty project selection.
- If no project is selected, focus returns to project dropdown.
- Selected IDs are submitted with `project_id`.

Anchors:
- add handler and project-id guard: [templates/designs/browse.html#L796](templates/designs/browse.html#L796)
- hidden project form target: [templates/designs/browse.html#L442](templates/designs/browse.html#L442)

### Pagination and Selection Interactions
- Pagination preserves browse filter/sort state in links.
- Checkbox selection is not persisted across page navigation.
- Banner state resets on fresh page load unless new selections are made.

Anchors:
- pagination links with filter params: [templates/designs/browse.html#L410](templates/designs/browse.html#L410)
- selection state source (checked inputs on page): [templates/designs/browse.html#L708](templates/designs/browse.html#L708)

## Responsive Behavior
- Banner uses wrapped flex layout so controls reflow on smaller widths.
- The page adds bottom padding to avoid last-row obstruction by sticky banner.
- Modal max height keeps content reachable on smaller viewports.

Anchors:
- banner responsive classes: [templates/designs/browse.html#L449](templates/designs/browse.html#L449)
- body bottom padding: [templates/designs/browse.html#L856](templates/designs/browse.html#L856)
- modal max-height style: [templates/designs/browse.html#L490](templates/designs/browse.html#L490)

## Accessibility Requirements
- Banner and modal controls must remain keyboard reachable.
- Modal must support keyboard close via Escape.
- Form controls must keep associated labels and visible action text.
- Project-unavailable state must expose disabled affordance, not hidden action.
- Selection status must be conveyed in text (`N selected`), not by color-only cues.

## Non-Goals and Deferred Specifications
- This document does not define browse search/filter backend semantics.
- This document does not define route/service error payload formats for bulk actions.

## Change Management Notes
- Treat this as the canonical UI baseline for Browse bottom-banner bulk actions.
- If labels, control order, selection scope, or modal semantics change, update this spec in the same change set.