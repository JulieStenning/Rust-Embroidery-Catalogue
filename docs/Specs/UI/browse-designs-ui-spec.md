# Browse Designs UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-25

## Purpose
Define the Browse Designs user interface contract for layout, controls, states, and accessibility so UI changes can be implemented consistently without mixing in backend semantics.

## Scope
In scope:
- Browse page structure and visual hierarchy.
- Search/filter controls and user-visible state behavior.
- Result grid/card presentation and selection UX.
- Pagination and bulk-action surfaces.
- Accessibility and responsive behavior.

Out of scope:
- Backend query implementation and optimization.
- Route/service-level contract details.
- Data-model or storage semantics.

## Source of Truth
- [templates/designs/browse.html](templates/designs/browse.html)
- [src/routes/designs.py](src/routes/designs.py)
- [src/services/designs.py](src/services/designs.py)
- [tests/test_routes.py](tests/test_routes.py)
- [docs/feature-inventory.md](docs/feature-inventory.md)

## Page Anatomy

### 1. Header and Context
- Page title: `Browse Designs`.
- Search guidance is available via contextual help affordance.

### 2. Search and Filter Area
Core controls:
- General search input.
- Unverified-only checkbox.
- Expandable additional filters area.
- Scope toggles for where text search applies.
- Sort-by and sort-direction controls.
- Search action and Reset action.

Additional filters include:
- All words
- Exact phrase
- Any words
- None words
- Filename pattern
- Designer
- Tags (with untagged option)
- Hoop
- Source
- Rating
- Stitched state

### 3. Results Summary Area
- Total match count.
- Current page indicator.
- Page-level select-all control.

### 4. Design Grid Area
Each card presents:
- Selection checkbox/selector.
- Preview image (or placeholder when missing).
- Filename/title text.
- Verification marker.
- Hoop badge when available.
- Tag badges.
- Rating display.
- Stitched status badge when applicable.

### 5. Pagination Area
- Previous and Next controls.
- Numbered page links with current-page emphasis.

### 6. Bulk Action Surfaces
- Sticky bulk action bar appears when one or more items are selected.
- Bulk tag modal for assigning tags to selected designs.
- Additional bulk actions include verify and add-to-project.

## Visual Design Contract

### Layout and Hierarchy
- Search and filter controls appear before result content.
- Results are shown as a responsive card grid.
- Bulk action controls remain visibly connected to current selection state.

### Visual Cues and Status Encoding
- Verification and stitched states are visually distinct.
- Tags are rendered as compact badges.
- Rating is displayed in a recognizable star-based pattern.
- Empty-state messaging is prominent and actionable.

### Density and Readability
- Card content prioritizes quick scan: image first, identity second, metadata third.
- Control labels are concise and maintain legibility at common viewport sizes.

## State Model (UI)

### Default State
- No active selection.
- Sticky bulk action bar hidden.
- Reset control disabled if no active criteria.

### Filtered State
- Active criteria reflected in control values.
- Reset control enabled when criteria differ from defaults.
- Result set and pagination update to filtered context.

### Empty Results State
- Clear `No designs found` style message.
- No grid cards rendered.
- Bulk action surfaces remain hidden.

### Selection Active State
- One or more cards selected.
- Sticky bulk action bar visible.
- Bulk actions operate on selected IDs only.

### Modal Active State (Bulk Tags)
- Modal surface is visible above page content.
- Background interaction is visually de-emphasized.
- Modal provides explicit apply and cancel actions.

## Interaction Contract

### Search and Filter Interactions
- Search submits current criteria as a single coherent state.
- Additional filter controls can be expanded/collapsed without losing values.
- Untagged option in tag filtering behaves as a distinct selection case.

### Selection Interactions
- Individual selection can be toggled per card.
- Select-all affects visible result set.
- Selection count drives visibility of sticky bulk action bar.

### Bulk Tagging Interactions
- Opening modal is only possible when selection exists.
- Tag options are grouped and scannable.
- Applying tags submits selected IDs plus selected tag IDs.

### Pagination Interactions
- Pagination preserves current filter/sort state.
- Current page is visually highlighted.

## Responsive Behavior
- Grid column count scales by viewport width.
- Controls remain accessible without overlap at smaller widths.
- Sticky bulk action bar remains usable on both desktop and mobile layouts.
- Modal content remains reachable without requiring horizontal scrolling.

## Accessibility Requirements
- All form controls have programmatically associated labels.
- Keyboard users can:
  - navigate search and filter controls,
  - select cards,
  - open and close modal,
  - execute bulk actions,
  - paginate results.
- Focus-visible indicators are clear across controls, cards, action bar, and modal actions.
- Modal semantics include accessible name and proper focus handling.
- Status indicators (verification, stitched, rating) are not conveyed by color alone.

## Non-Goals and Deferred Specifications
- Backend specification for browse search/filter execution, persistence, and service boundaries will be authored later as a dedicated backend spec.
- This document intentionally defines only user-visible UI behavior and interaction.

## Change Management Notes
- Treat this as the canonical UI baseline for Browse Designs.
- If controls/states/layout are changed, update this spec as part of the same change set.
