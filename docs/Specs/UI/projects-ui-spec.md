# Projects UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-29
- Backend companion: [docs/Specs/projects-backend-spec.md](docs/Specs/projects-backend-spec.md)
- User guide companion: [docs/User-Facing-Guidance/PROJECTS.md](docs/User-Facing-Guidance/PROJECTS.md)

## Purpose
Define the Projects UI contract for page structure, controls, form contracts, states, and interactions.

## Scope
In scope:
- Projects list page (`/projects/`).
- New project form page (`/projects/new`).
- Project detail page (`/projects/{project_id}`).
- Project print page (`/projects/{project_id}/print`).
- Design Detail project-membership controls that interact with project routes.

Out of scope:
- Browse bottom-banner tag/verify control details (covered by browse bulk-actions UI spec).
- Service internals and persistence implementation.

## Source of Truth
- [templates/projects/list.html](templates/projects/list.html)
- [templates/projects/form.html](templates/projects/form.html)
- [templates/projects/detail.html](templates/projects/detail.html)
- [templates/projects/print.html](templates/projects/print.html)
- [templates/designs/detail.html](templates/designs/detail.html)
- [src/routes/projects.py](src/routes/projects.py)
- [src/routes/designs.py](src/routes/designs.py)
- [tests/test_routes.py](tests/test_routes.py)

## Route Surface (UI-bound)
Primary Projects pages:
- `GET /projects/`
- `GET /projects/new`
- `GET /projects/{project_id}`
- `GET /projects/{project_id}/print`

Project actions bound from Projects pages:
- `POST /projects/`
- `POST /projects/{project_id}/edit`
- `POST /projects/{project_id}/delete`
- `POST /projects/{project_id}/remove-design/{design_id}`

Project actions bound from Design Detail:
- `POST /designs/{design_id}/add-to-project`
- `POST /designs/{design_id}/remove-from-project/{project_id}`

Related Browse integration (cross-surface contract):
- `POST /designs/bulk-add-to-project`

## Page Anatomy

### 1. Projects List (`/projects/`)
- Title + primary action row (`Projects`, `+ New Project`).
- Intro/help text with link to Help Projects anchor.
- Responsive card grid when projects exist.
- Empty-state text + create link when no projects exist.

Evidence:
- [templates/projects/list.html#L5](templates/projects/list.html#L5)
- [templates/projects/list.html#L7](templates/projects/list.html#L7)
- [templates/projects/list.html#L9](templates/projects/list.html#L9)
- [templates/projects/list.html#L13](templates/projects/list.html#L13)
- [templates/projects/list.html#L28](templates/projects/list.html#L28)

### 2. New Project Form (`/projects/new`)
- Back link to Projects list.
- Name (required) + description (optional) fields.
- Submit button (`Create Project`).
- Inline help link to Projects section in Help.

Evidence:
- [templates/projects/form.html#L6](templates/projects/form.html#L6)
- [templates/projects/form.html#L13](templates/projects/form.html#L13)
- [templates/projects/form.html#L15](templates/projects/form.html#L15)
- [templates/projects/form.html#L21](templates/projects/form.html#L21)

### 3. Project Detail (`/projects/{project_id}`)
- Top navigation strip:
  - back to projects,
  - print-sheet action,
  - delete project action with browser confirm prompt.
- Editable project header area:
  - name input,
  - description textarea,
  - save button.
- Design membership region:
  - heading with count,
  - responsive design-card grid with thumbnail/fallback,
  - filename/designer text,
  - remove button per design membership,
  - empty-state message when no designs.

Evidence:
- [templates/projects/detail.html#L5](templates/projects/detail.html#L5)
- [templates/projects/detail.html#L8](templates/projects/detail.html#L8)
- [templates/projects/detail.html#L9](templates/projects/detail.html#L9)
- [templates/projects/detail.html#L16](templates/projects/detail.html#L16)
- [templates/projects/detail.html#L34](templates/projects/detail.html#L34)
- [templates/projects/detail.html#L37](templates/projects/detail.html#L37)
- [templates/projects/detail.html#L47](templates/projects/detail.html#L47)
- [templates/projects/detail.html#L56](templates/projects/detail.html#L56)

### 4. Project Print (`/projects/{project_id}/print`)
- Standalone HTML document optimized for print.
- Project name and optional description header.
- Repeating design cards with preview/fallback and optional metadata rows.
- Uses print stylesheet with mm units and `page-break-inside: avoid` on cards.

Evidence:
- [templates/projects/print.html](templates/projects/print.html)

### 5. Design Detail Project Controls
- Membership section lists current projects for the design with per-project remove controls.
- Add-to-project selector is shown when at least one project is available.
- Project links navigate to project detail.

Evidence:
- [templates/designs/detail.html#L211](templates/designs/detail.html#L211)
- [templates/designs/detail.html#L216](templates/designs/detail.html#L216)
- [templates/designs/detail.html#L219](templates/designs/detail.html#L219)
- [templates/designs/detail.html#L228](templates/designs/detail.html#L228)

## Form and Field Contract

### Create Project Form
- Action: `/projects/`
- Fields:
  - `name` (required)
  - `description` (optional)

### Edit Project Form
- Action: `/projects/{project_id}/edit`
- Fields:
  - `name` (required)
  - `description` (optional)

### Delete Project Form
- Action: `/projects/{project_id}/delete`
- Contract: native confirm prompt before submit.

### Remove Design (Project Detail)
- Action: `/projects/{project_id}/remove-design/{design_id}`
- Contract: remove membership only.

### Add/Remove from Design Detail
- Add action: `/designs/{design_id}/add-to-project` with `project_id`.
- Remove action: `/designs/{design_id}/remove-from-project/{project_id}`.

## UI State Model

### Projects List States
- `has-projects`: grid of cards rendered.
- `empty`: no projects message + create link.

### Project Detail States
- `has-designs`: design cards rendered with remove controls.
- `empty-project`: empty membership message shown.

### Project Print States
- `has-image`: per-design base64 image rendered.
- `no-image`: placeholder tile rendered.
- `optional-metadata`: each metadata line appears only when value exists.

### Design Detail Membership States
- `has-memberships`: linked list of project memberships.
- `no-memberships`: informational text shown.
- `has-available-projects`: add form shown.

## Interaction Contract
- Project cards on list navigate to project detail.
- Project create/edit/delete flows are form + redirect driven (no AJAX).
- Project detail remove buttons are immediate POST actions.
- Print action opens a dedicated print-friendly route in a new tab from project detail.
- Design Detail add/remove controls route through project membership endpoints and return to design detail.

## Responsive and Print Expectations
- Projects list and detail design grids are responsive across small and large viewports.
- Project detail includes no-print controls and print-only project header text for browser print use.
- Dedicated print route remains the canonical printable sheet for project planning.

## Accessibility Expectations
- Primary controls remain keyboard reachable.
- Visible text labels remain present for create/edit/delete/remove actions.
- Empty states provide actionable navigation where possible.
- Confirm dialog is used for destructive delete action.

## Non-Goals and Cross-Spec Notes
- Browse bottom-banner project-assignment behavior is canonical in:
  - [docs/Specs/UI/browse-bulk-actions-ui-spec.md](docs/Specs/UI/browse-bulk-actions-ui-spec.md)
  - [docs/Specs/browse-bulk-actions-backend-spec.md](docs/Specs/browse-bulk-actions-backend-spec.md)
- This UI spec does not redefine browse selection semantics.

## Change Management Notes
- Treat this as canonical UI baseline for Projects pages and Design Detail membership controls.
- Update this document when route bindings, control labels, page states, or print output structure changes.