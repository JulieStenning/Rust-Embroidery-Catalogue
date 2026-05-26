# Design Detail UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-26
- Backend companion: [docs/Specs/design-detail-backend-spec.md](docs/Specs/design-detail-backend-spec.md)
- User guide companion: [docs/User-Facing-Guidance/DESIGN_DETAIL.md](docs/User-Facing-Guidance/DESIGN_DETAIL.md)

## Purpose
Define the Design Detail UI contract for structure, controls, form contracts, states, and interactions.

## Scope
In scope:
- Detail-page anatomy and visual state behavior.
- Form actions, field naming, and interaction contracts.
- Print and no-print presentation behaviors.
- Navigation affordances specific to detail context.

Out of scope:
- Service-layer implementation and persistence internals.
- Browse-page filter contracts.
- Cross-feature workflows outside design detail and print.

## Source of Truth
- [templates/designs/detail.html](templates/designs/detail.html)
- [templates/designs/print.html](templates/designs/print.html)
- [src/routes/designs.py](src/routes/designs.py)
- [tests/test_routes.py](tests/test_routes.py)
- [docs/feature-inventory.md](docs/feature-inventory.md)

## Route Surface (UI-bound)
- Primary page: `GET /designs/{design_id}`
- Print page: `GET /designs/{design_id}/print`

Interactive action endpoints bound from detail UI:
- `POST /designs/{design_id}/render-3d-preview`
- `GET /designs/{design_id}/open-in-editor`
- `GET /designs/{design_id}/open-in-explorer`
- `POST /designs/{design_id}/toggle-tags-checked`
- `POST /designs/{design_id}/rate`
- `POST /designs/{design_id}/toggle-stitched`
- `POST /designs/{design_id}/edit`
- `POST /designs/{design_id}/set-tags`
- `POST /designs/{design_id}/add-to-project`
- `POST /designs/{design_id}/remove-from-project/{project_id}`
- `POST /designs/{design_id}/delete`

Evidence:
- [templates/designs/detail.html#L50](templates/designs/detail.html#L50)
- [templates/designs/detail.html#L72](templates/designs/detail.html#L72)
- [templates/designs/detail.html#L78](templates/designs/detail.html#L78)
- [templates/designs/detail.html#L115](templates/designs/detail.html#L115)
- [templates/designs/detail.html#L130](templates/designs/detail.html#L130)
- [templates/designs/detail.html#L146](templates/designs/detail.html#L146)
- [templates/designs/detail.html#L159](templates/designs/detail.html#L159)
- [templates/designs/detail.html#L276](templates/designs/detail.html#L276)
- [templates/designs/detail.html#L229](templates/designs/detail.html#L229)
- [templates/designs/detail.html#L219](templates/designs/detail.html#L219)
- [templates/designs/detail.html#L206](templates/designs/detail.html#L206)

## Page Anatomy

### 1. Context Navigation Strip
- Back-to-browse link shown at top.
- Session-based previous/next micro-navigation uses `sessionStorage.browse_ids` and current design id.
- Large fixed previous/next arrows are injected when browse context exists.

Evidence:
- [templates/designs/detail.html#L5](templates/designs/detail.html#L5)
- [templates/designs/detail.html#L15](templates/designs/detail.html#L15)
- [templates/designs/detail.html#L333](templates/designs/detail.html#L333)

### 2. Two-Column Main Region
- Responsive layout uses one column on smaller screens, two columns from medium breakpoint.
- Left panel: preview image actions.
- Right panel: metadata and action controls.

Evidence:
- [templates/designs/detail.html#L43](templates/designs/detail.html#L43)

### 3. Preview Panel
- If image exists, display embedded image and 3D re-render button.
- If no image exists, show placeholder text and generate button.
- 3D button text changes based on current image type (`✓ 3D Preview` vs `Render 3D Preview`).

Evidence:
- [templates/designs/detail.html#L50](templates/designs/detail.html#L50)
- [templates/designs/detail.html#L53](templates/designs/detail.html#L53)
- [templates/designs/detail.html#L58](templates/designs/detail.html#L58)
- [templates/designs/detail.html#L61](templates/designs/detail.html#L61)

### 4. Metadata Panel (Read + Edit)
Contains:
- Filename heading.
- Filepath display with launch actions.
- Dimension/hoop/stitch/colour badges.
- Designer/source/date-added text rows.
- Assigned tags chip row (when tags exist) with verify/unverify button.
- Rating control row.
- Stitched toggle row.
- Metadata edit form (notes, designer, source, hoop) with save.
- Actions row (print + delete).
- Projects membership list and add/remove controls.

Evidence:
- [templates/designs/detail.html#L67](templates/designs/detail.html#L67)
- [templates/designs/detail.html#L72](templates/designs/detail.html#L72)
- [templates/designs/detail.html#L84](templates/designs/detail.html#L84)
- [templates/designs/detail.html#L101](templates/designs/detail.html#L101)
- [templates/designs/detail.html#L110](templates/designs/detail.html#L110)
- [templates/designs/detail.html#L130](templates/designs/detail.html#L130)
- [templates/designs/detail.html#L146](templates/designs/detail.html#L146)
- [templates/designs/detail.html#L159](templates/designs/detail.html#L159)
- [templates/designs/detail.html#L204](templates/designs/detail.html#L204)
- [templates/designs/detail.html#L211](templates/designs/detail.html#L211)

### 5. Collapsible Tags Editor (Full Width)
- Rendered as a `<details>` block below main region.
- Summary shows assigned-tag count/details and verified state text.
- Tag options grouped by `image`, `stitching`, and unclassified tags.
- Save button posts selected `tag_ids`.
- Inline hint states that save marks design verified.

Evidence:
- [templates/designs/detail.html#L244](templates/designs/detail.html#L244)
- [templates/designs/detail.html#L276](templates/designs/detail.html#L276)
- [templates/designs/detail.html#L324](templates/designs/detail.html#L324)

## Form and Field Contract

### Rating Form
- Action: `/designs/{id}/rate`
- Field: `rating`
- Values:
  - `1..5` for set.
  - empty string for clear.

Evidence:
- [templates/designs/detail.html#L130](templates/designs/detail.html#L130)
- [templates/designs/detail.html#L138](templates/designs/detail.html#L138)

### Stitched Form
- Action: `/designs/{id}/toggle-stitched`
- Hidden field: `is_stitched`
- Value flips current state (`true` when currently false, else `false`).

Evidence:
- [templates/designs/detail.html#L146](templates/designs/detail.html#L146)

### Verification Toggle Form
- Action: `/designs/{id}/toggle-tags-checked`
- Hidden field: `tags_checked`
- Value contract:
  - `true` to mark verified.
  - empty value to mark unverified.
- Rendered only when `design.tags` exists.

Evidence:
- [templates/designs/detail.html#L110](templates/designs/detail.html#L110)
- [templates/designs/detail.html#L115](templates/designs/detail.html#L115)
- [templates/designs/detail.html#L117](templates/designs/detail.html#L117)
- [templates/designs/detail.html#L121](templates/designs/detail.html#L121)

### Metadata Edit Form
- Action: `/designs/{id}/edit`
- Hidden pass-through fields: `filename`, `filepath`, `rating`, `is_stitched`.
- Editable fields: `notes`, `designer_id`, `source_id`, `hoop_id`.

Evidence:
- [templates/designs/detail.html#L159](templates/designs/detail.html#L159)
- [templates/designs/detail.html#L160](templates/designs/detail.html#L160)
- [templates/designs/detail.html#L171](templates/designs/detail.html#L171)
- [templates/designs/detail.html#L180](templates/designs/detail.html#L180)
- [templates/designs/detail.html#L189](templates/designs/detail.html#L189)

### Tags Set Form
- Action: `/designs/{id}/set-tags`
- Checkbox field: `tag_ids` (multi-value)
- Groups are visual-only categories; payload is flat `tag_ids` list.

Evidence:
- [templates/designs/detail.html#L276](templates/designs/detail.html#L276)

### Project Membership Forms
- Add action: `/designs/{id}/add-to-project` with `project_id` select.
- Remove action: `/designs/{id}/remove-from-project/{project_id}` per listed project.

Evidence:
- [templates/designs/detail.html#L229](templates/designs/detail.html#L229)
- [templates/designs/detail.html#L219](templates/designs/detail.html#L219)

### Delete Form
- Action: `/designs/{id}/delete`
- Includes native confirm prompt string.

Evidence:
- [templates/designs/detail.html#L206](templates/designs/detail.html#L206)

## UI State Model

### Preview States
- `has-image`: image shown with render button.
- `no-image`: placeholder text + generate button.

### Verification States
- `verified`: `✓ Verified` text/button variant.
- `unverified`: `⚠ Verify` text/button variant.

### Rating States
- `unrated`: all stars muted, no clear button.
- `rated`: stars filled to current value, clear control shown, print-only stars shown.

### Stitched States
- `not-stitched`: button says `Mark as Stitched`.
- `stitched`: button says `✓ Mark as Not Stitched`, print-only stitched line shown.

### Projects States
- `no-project-membership`: informational empty text.
- `has-project-membership`: removable membership list.
- `has-available-projects`: add-to-project form visible.

Evidence:
- [templates/designs/detail.html#L53](templates/designs/detail.html#L53)
- [templates/designs/detail.html#L58](templates/designs/detail.html#L58)
- [templates/designs/detail.html#L116](templates/designs/detail.html#L116)
- [templates/designs/detail.html#L120](templates/designs/detail.html#L120)
- [templates/designs/detail.html#L138](templates/designs/detail.html#L138)
- [templates/designs/detail.html#L142](templates/designs/detail.html#L142)
- [templates/designs/detail.html#L151](templates/designs/detail.html#L151)
- [templates/designs/detail.html#L155](templates/designs/detail.html#L155)
- [templates/designs/detail.html#L226](templates/designs/detail.html#L226)

## Print Surface Contract
- Dedicated printable route presents summary metadata and optional image.
- Detail page also exposes print-only snippets for rating/stitched/notes when using browser print.

Evidence:
- [src/routes/designs.py#L657](src/routes/designs.py#L657)
- [templates/designs/print.html#L21](templates/designs/print.html#L21)
- [templates/designs/print.html#L48](templates/designs/print.html#L48)
- [templates/designs/print.html#L51](templates/designs/print.html#L51)
- [templates/designs/print.html#L57](templates/designs/print.html#L57)
- [templates/designs/detail.html#L142](templates/designs/detail.html#L142)
- [templates/designs/detail.html#L155](templates/designs/detail.html#L155)
- [templates/designs/detail.html#L200](templates/designs/detail.html#L200)

## Accessibility and Responsive Expectations
- Forms and controls remain keyboard reachable.
- Core labels and control text remain visible and understandable in compact layouts.
- No-print/print-only classes preserve clean print behavior.
- Fixed prev/next arrows should not obstruct core content on common viewport widths.

## Non-Goals and Deferred Work
- Backend validation/error normalization is defined in backend spec, not here.
- This UI spec does not define redesign goals, only current interaction contract.
- Any move away from hidden pass-through fields in edit form requires coordinated backend contract update.

## Change Management Notes
- Treat this as canonical UI contract for design detail.
- Update this document in the same change set when action endpoints, field names, or state behavior changes.