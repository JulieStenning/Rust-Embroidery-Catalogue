# Top-Level Menu UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-25

## Purpose
Define the user interface contract for the top-level application menu so future changes remain visually and structurally consistent.

## Scope
In scope:
- Header/navigation information architecture.
- Visual structure, hierarchy, and responsive behavior.
- User-visible interaction and feedback states in the menu area.
- Accessibility expectations for keyboard and screen-reader use.

Out of scope:
- Functional behavior specifications for destination pages linked from the menu.
- Business logic or permission rules for menu items.
- Backend contracts for endpoints behind each link.

## Source of Truth
- [templates/base.html](templates/base.html)
- [src/main.py](src/main.py)
- [docs/feature-inventory.md](docs/feature-inventory.md)

## Information Architecture

### Header Regions
The top-level navigation is a single horizontal bar with two regions:
- Left region:
  - Application brand/title link: `Embroidery Catalogue`.
  - Primary navigation links intended for routine workflows.
- Right region:
  - Admin/maintenance links for setup and catalog operations.

### Primary Navigation (Left)
Visible link set:
- Browse
- Import
- Projects
- Help

Expected behavior:
- Links are always visible in desktop layout.
- Link text is concise and action-oriented.

### Admin Navigation (Right)
Visible link set:
- Designers
- Tags
- Sources
- Hoops
- Settings
- Backup
- Tagging Actions
- Orphans

Expected behavior:
- Grouped together as a utility/admin cluster.
- Equal visual weight within the admin group.
- Orphans entry behaves as an action trigger that opens status feedback (not a passive static label).

## Visual Design Contract

### Layout
- Header spans full content width and sits above page content.
- Navigation content is horizontally aligned with balanced spacing.
- Left and right link groups remain visually separated.

### Color and Contrast
- Dark, high-contrast header background.
- Light text/icons for all nav labels.
- Hover/focus states provide a clear, perceivable visual change.

### Typography
- Brand/title uses stronger emphasis than individual links.
- Link text remains readable at standard desktop and laptop scaling.

### Spacing and Density
- Horizontal spacing between links is consistent within each group.
- Click/tap targets remain comfortably interactive.

## Interaction States

### Link States
Each top-level link supports:
- Default
- Hover
- Focus-visible (keyboard)
- Active/pressed

State expectations:
- Hover and focus must be distinguishable from default.
- Focus-visible must not rely on color alone.

### Orphans Action Feedback
When the Orphans action is invoked:
- A modal/status surface appears.
- User receives immediate progress feedback (loading state).
- User receives terminal state feedback (success or error text).
- User can dismiss the feedback surface once complete.

## Responsive Behavior
- On reduced widths, navigation preserves usability and avoids overlapping labels.
- Header continues to provide access to both primary and admin groups.
- If wrapping occurs, link order remains stable and predictable.

## Accessibility Requirements
- All interactive menu items are reachable via keyboard tab order.
- Focus indicator is visible on every interactive item.
- Link text remains descriptive when read out of context.
- Any modal/status UI triggered from the menu:
  - moves initial focus appropriately,
  - exposes role/label semantics,
  - supports keyboard dismissal,
  - returns focus to invoking control on close.

## Non-Goals and Deferred Specifications
- Detailed functionality specifications for each top-menu destination will be documented later in separate functional specs.
- This document intentionally defines only UI presentation and interaction expectations.

## Change Management Notes
- Treat this as the baseline for visual and structural menu behavior.
- If menu items are added/removed/reordered, update this spec first, then implement UI changes.
