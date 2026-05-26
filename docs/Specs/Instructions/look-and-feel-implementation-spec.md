# Look and Feel Implementation Spec

## Status
- Type: UI implementation standard
- Audience: Agents and contributors migrating pages from legacy templates
- Last updated: 2026-05-26

## Why this exists
Use this spec to keep migrated Svelte pages visually consistent with the original Embroidery Catalogue look and feel.

Without a shared standard, each page drifts in spacing, borders, typography, and button treatment.

## Visual direction (legacy parity)
- Overall tone: practical desktop utility, not marketing UI.
- Surfaces: light neutral gray page background, pale gray cards.
- Density: compact forms and short vertical rhythm.
- Typography: clear system font stack and moderate weights.
- Controls: plain borders, low-radius corners, subtle focus ring.
- Primary action: indigo/purple solid button.

## Global baseline
- Keep using the established shell and top menu styles in app.css.
- Keep page content width constrained (do not run full-bleed forms).
- Prefer simple gradients only where legacy used them; otherwise flat color.

## Current Settings page reference
Settings migration now uses a scoped style set in:
- frontend/src/app.css (settings-specific classes)
- frontend/src/lib/MainView.svelte (settings markup)

Primary class hooks:
- settings-page
- settings-title
- settings-layout
- settings-card
- settings-form
- settings-meta
- settings-input
- settings-primary-button
- settings-secondary-button
- settings-code
- settings-alert

## Design tokens for migrated admin forms
Use these values unless a page-specific spec says otherwise.

### Colors
- Page background: #f3f4f6 to #f5f6f8 range
- Card background: #f4f5f7
- Card border: #d6d9e1
- Input border: #d1d5db
- Body text: #111827 to #374151
- Muted text: #6b7280
- Link text: #4f46d8
- Primary button: #4f46d8 (hover #4338ca)

### Radius
- Card radius: 0.4rem
- Input/button radius: 0.3rem to 0.35rem

### Spacing
- Card padding: 1.5rem
- Section gaps inside form card: 1.25rem
- Label-to-input gap: 0.25rem to 0.4rem
- Help text top margin: about 0.25rem

### Typography
- Family: Segoe UI, Tahoma, Arial, sans-serif
- Page title: strong, compact, no extra letter spacing
- Section headings: semibold, small
- Input text size: small to normal utility scale

## Form behavior and styling rules
- Inputs should look plain and stable, not glossy.
- Focus states should be visible but soft.
- Checkboxes/radios should keep indigo focus accents.
- Do not oversize controls for mobile unless needed by usability.
- Keep explanatory copy muted and compact.

## Alerts and notices
- Success/error/info banners should be short and single-purpose.
- Keep warning blocks (for cost or risk) with subtle colored backgrounds and clear border.
- Avoid large paddings that break compact layout.

## Migration checklist for each new page
1. Start from legacy template structure first (sections and order).
2. Match container width and card rhythm before color tuning.
3. Match control density (input height, button height, spacing).
4. Apply shared classes from app.css where possible.
5. Add page-scoped classes only if shared classes are insufficient.
6. Run frontend build and verify no style regressions.
7. Compare against screenshot/template and adjust only deltas.

## Implementation strategy for remaining pages
- Prefer extending shared admin form styles, not creating unique per-page designs.
- Keep new page-specific CSS grouped and clearly prefixed.
- Reuse button/input/card class patterns from Settings.
- Keep dark mode support aligned with existing overrides.

## Non-goals
- Do not redesign pages to modern dashboard patterns.
- Do not replace legacy tone with bold gradients or oversized typography.
- Do not introduce new component libraries for simple forms.

## Suggested next pages to apply this spec
- Import settings-like subforms
- Backup and maintenance forms
- Admin CRUD pages (designers, tags, sources, hoops)
