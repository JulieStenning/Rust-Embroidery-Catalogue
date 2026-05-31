# UI Global Standards

## Status
- Type: Canonical cross-page UI contract
- Audience: Agents and maintainers
- Last validated: 2026-05-31

## Purpose
Define shared UI standards across pages so Rust implementation follows measurable rules. Page-level UI specs may add local behavior, but must not conflict with this document.

## Contract Hierarchy
1. This global standards document.
2. Page-level UI spec in docs/Specs/UI.
3. Implementation in frontend and templates.

When visual conflicts occur, update standards first, then implementation.

## Layout and Spacing
- Content container max width: 80rem (max-w-7xl).
- Primary page horizontal padding: 1rem.
- Primary section vertical rhythm: 1rem minimum gap between major blocks.
- Card/grid baseline gap: 1rem unless page spec defines a stricter value.

## Responsive Breakpoints
Use the following viewport breakpoints for layout shifts:
- Base: < 640px
- SM: >= 640px
- MD: >= 768px
- LG: >= 1024px
- XL: >= 1280px

Page specs can define element-specific behavior on these breakpoints but should not introduce new global breakpoints.

## Typography
- Page title: 1.875rem, semibold/bold.
- Shared implementation class: use .ui-page-title in frontend/src/app.css, with values derived from Tailwind scale tokens.
- Section label: 0.875rem, medium weight.
- Control text: 0.875rem.
- Metadata text inside dense cards: 0.75rem to 0.6875rem.
- Keep line-height between 1.2 and 1.35 for dense metadata rows.

## Section Surfaces and Fields
- Use a rounded gray section shell for grouped controls (background #f3f4f6, 1px gray border, rounded corners).
- Section labels (for example, General search) use sentence case with shared section-label typography and muted dark gray text.
- Field labels (for example, Unverified only) use shared field-label typography and muted dark gray text.
- Top-aligned text-entry field labels must use the shared Browse pattern: outer label class `ui-field-label text-sm` with inner label text in `span.block.font-medium.mb-1`.
- Text entry controls use white background, dark text, and gray placeholders.
- Text entry controls use compact vertical padding with equal top/bottom breathing room.
- Dropdown/select controls use a dedicated class with slightly larger vertical padding for readability.
- Stacked field labels should use a block label with a small bottom gap before the control (for example, `mb-1`), matching the Browse page spacing.
- Dropdown/select controls and multi-select summary controls must use the same left text inset so displayed values (for example, "Any") align consistently across fields.
- Text-entry controls must use the same shared left text inset class as dropdown/select controls to keep value alignment consistent across field types.
- In dense filter grids, controls must share the same rendered height and full-width sizing within each column.
- Multi-column filter layouts must define explicit column and row gaps (not implicit utility defaults) for consistent spacing.
- Checkboxes use a visible gray border and white background.
- Selected checkboxes use a blue background with a white checkmark (for parity with Search in controls).
- Radio controls in grouped filters use a shared class and selected blue indicator styling.
- Help notes use shared help-note typography and muted gray text.
- In-app links in control/help text remain blue, non-underlined, and must not switch to visited purple.

## Shared Browse-Oriented UI Classes
- .ui-section-shell
- .ui-section-label
- .ui-field-label
- .ui-help-note
- .ui-app-link
- .ui-text-input
- .ui-select-input
- .ui-control-text-inset
- .ui-multi-dropdown-summary
- .ui-control-caret
- .ui-checkbox
- .ui-radio
- .ui-action-button
- .ui-action-button-primary

## Controls
- Primary button height target: 2rem to 2.25rem.
- Input/select height target: 2rem to 2.25rem.
- Control border radius baseline: 0.25rem.
- Button labels use sentence case, including action buttons and secondary controls.
- Keep button copy concise and task-focused; use title case only for proper nouns or product-specific names.
- Shared button classes (`ui-action-button`, `ui-action-button-primary`, and existing page aliases) should resolve to the same filled button treatment, size, and shape across the app.
- Import folder rows use a small horizontal gap between the text box, browse button, and remove button, plus a small vertical gap between stacked rows.
- Checkbox hit area should remain easily clickable and keyboard focus-visible.
- Disabled buttons must use explicit disabled styling via shared button classes, not utility-only styling.
- Action button groups (primary + secondary buttons side by side) must use the `.ui-action-button-group` container class, which provides `flex`, `flex-wrap`, `gap: 1rem`, and `align-items: center`.
- Horizontal spacing between adjacent action buttons in the same row is a global standard of `1rem`; do not use page-specific left/right margin overrides to change that spacing.
- Browse search action buttons use dedicated semantic classes (.browse-search-submit-button and .browse-search-reset-button).
- In Browse, Search is the authoritative refresh action; do not add a separate Refresh button beside Search/Reset.

## Card Contract
Applies to dense browse/project card grids unless overridden by page spec.

- Outer card dimensions must be uniform within a visible grid row.
- Card media must preserve aspect ratio and show the full image when source data is available.
- Media viewport may be fixed; rendered image can vary inside viewport based on intrinsic ratio.
- Metadata overflow must be truncated using deterministic rules (single-line ellipsis or max line clamp).
- Card information order: media, identity (filename/title), validation/status cues, contextual metadata, rating/action cues.

## States and Feedback
- Required states: default, hover, focus-visible, active/pressed, disabled.
- Required data states: loading, empty, error, success/notice.
- Status cues must not rely on color alone; include iconography or text labels.

## Accessibility
- All form controls require associated labels.
- Focus-visible indicators must be clear across links, controls, cards, and modal actions.
- Modal/dialog UI must expose accessible name and keyboard dismissal path.
- Interactive elements must be reachable in logical keyboard order.

## Verification Checklist
1. Layout and spacing follow global scale and breakpoints.
2. Controls follow sizing and state conventions.
3. Card behavior follows uniform-size + full-image-fit rules.
4. Overflow handling is deterministic and documented.
5. Loading/empty/error/notice states are present and readable.
6. Keyboard and focus-visible behavior is verified.

## Change Management
- Any UI change that modifies spacing, sizing, card contract, or state behavior must update the affected spec document in the same change set.
- Screenshot comparison with Python UI is validation, not source-of-truth authoring.
