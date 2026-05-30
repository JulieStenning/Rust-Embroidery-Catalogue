# Help UI Specification

## Status
- Type: Current behavior baseline (UI only)
- Audience: Agents
- Last validated: 2026-05-29
- Backend companion: [docs/Specs/help-backend-spec.md](docs/Specs/help-backend-spec.md)
- User guide companion: [docs/User-Facing-Guidance/HELP.md](docs/User-Facing-Guidance/HELP.md)
- Refactor checklist companion: [docs/Specs/help-refactor-checklist.md](docs/Specs/help-refactor-checklist.md)

## Purpose
Define the Help/About UI contract for migration to a Svelte implementation with strict visible-behavior parity.

## Scope
In scope:
- Help hub page (`/help`) structure, section anchors, labels, and quick-jump behavior.
- About hub page (`/about`) document-list behavior.
- About document page (`/about/document/{slug}`) rendering contract.
- Contextual help entry links from key workflow screens.

Out of scope:
- Backend/service internals beyond route-bound UI behavior.
- Full legal/policy document authoring.
- Styling redesign beyond behavior-preserving parity.

## Source of Truth
- [templates/info/help.html](templates/info/help.html)
- [templates/about.html](templates/about.html)
- [templates/about_document.html](templates/about_document.html)
- [templates/base.html](templates/base.html)
- [src/routes/info.py](src/routes/info.py)
- [src/routes/about.py](src/routes/about.py)
- [tests/test_routes.py](tests/test_routes.py)

## Route Surface (UI-bound)
- `GET /help`
- `GET /about`
- `GET /about/document/{slug}`

## Page Anatomy

### 1. Help Hub (`/help`)
- Title block:
  - `Help`
  - `Quick guidance for using the Embroidery Catalogue.`
- Quick-jump nav with hash links (in this exact order and labels):
  - `🔍 Search` -> `#search`
  - `📥 Importing` -> `#importing`
  - `🤖 AI Tagging` -> `#ai-tagging`
  - `🏷 Tagging Actions` -> `#tagging-actions`
  - `📁 Projects` -> `#projects`
  - `🛠 Maintenance` -> `#maintenance`
  - `🔧 Troubleshooting` -> `#troubleshooting`
- Section blocks in the same exact order as quick-jump nav:
  - `🔍 Search`
  - `📥 Importing`
  - `🤖 AI Tagging`
  - `🏷 Tagging Actions`
  - `📁 Projects`
  - `🛠 Maintenance`
  - `🔧 Troubleshooting`

Strict parity notes:
- Keep section IDs unchanged (`search`, `importing`, `ai-tagging`, `tagging-actions`, `projects`, `maintenance`, `troubleshooting`).
- Keep heading labels unchanged (including emoji and spelling).
- Keep same user-facing guidance intent and workflow references.

### 2. About Hub (`/about`)
- Intro card with app summary and AI/settings caveat text.
- Licence highlight panel with `Open licence text` action to `/about/document/licence`.
- Document-list cards driven by route-provided document map.
- Each listed document row shows:
  - title,
  - description,
  - `Open` button when available,
  - `Not found` status when unavailable.

### 3. About Document (`/about/document/{slug}`)
- Header includes document title and source filename.
- Back link to `/about`.
- Document body rendered as plain pre-wrapped text in a readable container.

## Link Contract

### Required links inside Help content
- Internal app routes:
  - `/designs/`
  - `/import/`
  - `/admin/settings/`
  - `/admin/tagging-actions/`
  - `/projects/`
  - `/admin/maintenance/orphans`
  - `/about/document/ai-tagging`
- External links:
  - `https://aistudio.google.com/`
  - `https://ai.google.dev/pricing`

### Global discoverability links that impact Help/About usage
- Global top-nav Help link in base layout.
- Footer About link in base layout.

### Contextual workflow links to Help anchors
- Browse page -> `/help#search`
- Import pages -> `/help#importing`
- Projects pages -> `/help#projects`
- Orphans page -> `/help#maintenance` and `/help#troubleshooting`

## UI State Model

### Help Hub States
- `default`: all sections rendered, quick-jump links active.
- `anchor-navigation`: loading with `#section-id` scrolls to matching section.

### About Hub States
- `document-available`: row renders `Open` action.
- `document-missing`: row renders `Not found` label.

### About Document States
- `valid-slug-and-file`: rendered content page.
- `invalid-slug`: route returns 404 page.
- `missing-file`: route returns 404 page.

## Accessibility and Responsive Expectations
- Quick-jump links and document actions remain keyboard reachable.
- Heading hierarchy remains clear (`h1` page title, `h2` section headings).
- Content remains readable on small and large screens.
- Print behavior preserves readability and hides quick-jump controls (`no-print`).

## Svelte Migration Parity Contract
- Preserve route paths and hash anchors exactly.
- Preserve section order and labels exactly.
- Preserve About slug destinations and visible document titles.
- Preserve visible fallback states (`Not found` in About list, 404 route behavior for invalid/missing document requests).
- Preserve contextual inbound links from existing workflow pages.

## Test Anchors
- Help route returns 200 and contains core headings: [tests/test_routes.py](tests/test_routes.py)
- Help page exposes About link via base layout content: [tests/test_routes.py](tests/test_routes.py)
- About page and selected about-document routes return expected content: [tests/test_routes.py](tests/test_routes.py)

## Change Management Notes
- Treat this document as the canonical UI baseline for Help/About migration.
- Update this spec whenever Help labels/anchors, About slugs, or contextual link targets change.