# UI Global Standards & Look and Feel Specification

## Status
- **Type:** Canonical Cross-Page UI, Layout, & Theme Contract
- **Audience:** AI Agents, Developers, and Maintainers
- **Authority:** Authoritative reference for all UI components, views, layouts, and light/dark theme styling across Embroidery Catalogue
- **Last Updated:** October 2026 (Consolidated master standard)

---

## 1. Executive Summary & Design Philosophy

Embroidery Catalogue is a fast, responsive, offline desktop application for cataloguing, browsing, and inspecting machine embroidery designs.

### Core Visual Principles
1. **Practical Desktop Utility:** Clean, high-density, focused workflow tool — not a flashy marketing website or heavyweight web portal.
2. **Complete Theme Parity (Light, Dark, and System):** Every view, card, modal, form, badge, and table must render with optimal contrast and visual hierarchy in **both Light and Dark modes**, whether following the OS preference (`system`) or explicitly selected by user setting (`light` / `dark`).
3. **Consistent Component Vocabulary:** Reuse established component patterns from our gold-standard views (**Browse Designs**, **Choose Tags**, and **Design Details**) rather than inventing ad-hoc styling.
4. **Token-Driven Architecture:** All colors, borders, and surfaces are driven by CSS custom properties defined in `:root`, `:root[data-theme="dark"]`, and scoped dark-mode media queries in `frontend/src/app.css`.

---

## 2. Exemplar Views (The Gold Standards)

When designing or updating any view, agents **MUST** align with the patterns proven in these three gold-standard views:

### 1. Browse Designs (`BrowseView.svelte` + subcomponents)
- **Header:** Clean `h1.ui-page-title` with `text-2xl font-bold text-[var(--text-primary)]`.
- **Search & Filter Shell:** Contained card container (`bg-[var(--surface-card)] rounded shadow p-4 border border-[var(--border-default)]`).
- **Cards & Grids:** Uniform card dimensions, thumbnail previews centered in `bg-[var(--surface-preview)]` with subtle borders, deterministic text truncation, and responsive multi-column layouts.
- **Selection & Action Bar:** High-visibility contextual action bar for batch operations.

### 2. Choose Tags (`TagSelectionModal.svelte` / `TagChooserModal.svelte`)
- **Dialog Shell:** Clean modal dialog with crisp header, searchable quick-entry field, auto-save indicator, and discrete category buckets.
- **Categorisation Semantics:** Strict visual separation:
  - **Image Tags:** **Green** theme (`#166534` light text on `#dcfce7` / `#86efac` dark text on `#052e16`).
  - **Stitching Tags:** **Blue** theme (`#1d4ed8` light text on `#dbeafe` / `#93c5fd` dark text on `#172554`).
  - **Unclassified Tags:** Neutral gray theme.
- **Inputs & Checkboxes:** Native-feel, accessible checkbox items with hover states (`var(--surface-hover)`).

### 3. Design Details (`DesignDetailView.svelte`)
- **Two-Column Responsive Split:** Left column with sticky design preview and quick launch actions; right column with scrollable metadata panels.
- **Surface Cards (`.route-card`):** Soft, rounded container panels (`bg-[var(--surface-card)] rounded border border-[var(--border-default)] p-4`).
- **Data Table / Technical Grid (`TechnicalDataGrid.svelte`):** High-density, high-legibility key-value pairs with monospace values for stitch counts, hoop sizes, and dimensions.
- **Action Buttons:** Standard purple/indigo primary button (`menu-button-primary`), neutral ghost buttons (`menu-button-ghost`), and clear feedback toasts.

---

## 3. Theme Architecture & CSS Custom Properties

The application uses `themeStore.ts` to manage three modes: `"system"`, `"light"`, and `"dark"`.
The store attaches `data-theme="light"` or `data-theme="dark"` to `document.documentElement` (`<html data-theme="...">`). In system mode, no `data-theme` attribute is set and OS `prefers-color-scheme` takes effect.

### 3.1 Design Token Reference Table

| Token Name | Light Mode Value | Dark Mode Value | Usage / Semantic Role |
| :--- | :--- | :--- | :--- |
| `--surface-page` | `#f8fafc` (slate-50) | `#0f172a` (slate-900) | App background behind all cards and routes |
| `--surface-card` | `#ffffff` (white) | `#1e293b` (slate-800) | Primary cards, panels, forms, modals, tables |
| `--surface-card-subtle` | `#f8fafc` (slate-50) | `#162032` (slate-950/80) | Nested cards, form groups, code snippets |
| `--surface-card-muted` | `#f1f5f9` (slate-100) | `#334155` (slate-700) | Secondary panels, disabled areas, dividers |
| `--surface-preview` | `#e6e8eb` (gray-200) | `#0f172a` (slate-900) | Background matting behind stitch image thumbnails |
| `--surface-input` | `#ffffff` (white) | `#0f172a` (slate-900) | Input fields, textareas, dropdown selects |
| `--surface-hover` | `#f3f4f6` (gray-100) | `#334155` (slate-700) | Table row hover, list item hover, menu hover |
| `--surface-active` | `#e5e7eb` (gray-200) | `#475569` (slate-600) | Active/pressed state for controls and tabs |
| `--text-primary` | `#111827` (gray-900) | `#f8fafc` (slate-50) | Page titles, primary card headers, form text |
| `--text-secondary` | `#374151` (gray-700) | `#e2e8f0` (slate-200) | Field labels, descriptive text, table content |
| `--text-muted` | `#6b7280` (gray-500) | `#94a3b8` (slate-400) | Subtitles, helper text, disabled hints, badges |
| `--text-dim` | `#9ca3af` (gray-400) | `#64748b` (slate-500) | Placeholder text, empty state messages |
| `--text-brand` | `#4f46e5` (indigo-600) | `#818cf8` (indigo-400) | Primary link text, accent labels, highlights |
| `--text-brand-hover` | `#4338ca` (indigo-700) | `#a5b4fc` (indigo-300) | Link text hover state |
| `--border-default` | `#d1d5db` (gray-300) | `#334155` (slate-700) | Card borders, input borders, divider lines |
| `--border-subtle` | `#e5e7eb` (gray-200) | `#1e293b` (slate-800) | Nested divider lines, table cell separators |
| `--border-strong` | `#9ca3af` (gray-400) | `#475569` (slate-600) | Scrollbar thumb, active control borders |
| `--border-focus` | `#6366f1` (indigo-500) | `#818cf8` (indigo-400) | Focus ring outline for keyboard navigation |
| `--control-accent` | `#4f46e5` (indigo-600) | `#818cf8` (indigo-400) | Checkbox/radio tint, active switches |
| `--notice-info-bg` / `-border` / `-text` | `#eff6ff` / `#93c5fd` / `#1e40af` | `#1e1b4b` / `#3730a3` / `#c7d2fe` | Informational banners & helper callouts |
| `--notice-success-bg` / `-border` / `-text` | `#f0fdf4` / `#86efac` / `#166534` | `#052e16` / `#166534` / `#bbf7d0` | Success notices, positive status badges |
| `--notice-warn-bg` / `-border` / `-text` | `#fffbeb` / `#fcd34d` / `#92400e` | `#422006` / `#92400e` / `#fde68a` | Warning / advisory alerts, rating stars |
| `--notice-error-bg` / `-border` / `-text` | `#fef2f2` / `#fca5a5` / `#991b1b` | `#450a0a` / `#991b1b` / `#fca5a5` | Error alerts, destructive actions, failure indicators |

---

## 4. Layout, Geometry, and Responsive Breakpoints

### 4.1 Content Containers & Spacing Rhythm
- **Content Container Max Width:** `80rem` (`max-w-7xl`).
- **Primary Page Horizontal Padding:** `1rem` (`px-4`).
- **Primary Section Vertical Rhythm:** `1rem` (`gap-4` or `space-y-4`) minimum gap between major blocks.
- **Card / Grid Baseline Gap:** `1rem` unless a sub-spec defines a denser grid.

### 4.2 Responsive Breakpoints
Use the following standard viewport breakpoints for layout shifts:
- **Base:** `< 640px`
- **SM:** `>= 640px`
- **MD:** `>= 768px`
- **LG:** `>= 1024px`
- **XL:** `>= 1280px`

Page specs can define element-specific behavior across these breakpoints but should not introduce ad-hoc global breakpoints.

---

## 5. Specific Regression Guardrails & Rules

### ⚠️ Rule 1: No Hardcoded Color Literals in Component Templates
**Problem:** Hardcoded utility classes like `bg-white`, `bg-gray-50`, `text-gray-700`, `text-indigo-600`, `bg-red-50`, or `border-amber-300` break in dark mode or when theme colors change.
**Requirement:** All component styling must use semantic tokens or standardized semantic utility classes (`.notice-*`, `.btn-*`, `.input-field`, `.modal-*`, `.badge-status-*`, `.text-rating-star`, `var(--surface-*)`, `var(--text-*)`, `var(--border-*)`, `var(--control-*)`, `var(--notice-*)`).

### ⚠️ Rule 2: No Un-Gated Dark Mode Media Queries (Prevents "Dark Areas in Light Mode")
**Problem:** If CSS rules in `app.css` are placed inside `@media (prefers-color-scheme: dark) { ... }` without checking `data-theme`, a user whose OS is in Dark Mode who selects **Appearance: Light** will still receive dark styling on tables, cards, or titles.
**Requirement:** Any media-query dark overrides MUST be gated with `:root:not([data-theme="light"])`:
```css
/* ✅ CORRECT: Only applies if the user hasn't explicitly chosen Light */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) .admin-table-shell,
  :root[data-theme="dark"] .admin-table-shell {
    background: var(--surface-card) !important;
    border-color: var(--border-default) !important;
  }
}

/* ❌ INCORRECT: Forces dark background in Light Mode when OS is dark */
@media (prefers-color-scheme: dark) {
  .admin-table-shell {
    background: #1e293b !important;
  }
}
```

### ⚠️ Rule 3: Uniform Page Headings
**Problem:** Some pages have `h1` titles rendered at `text-lg` with washed-out text, while other pages use different ad-hoc sizes.
**Requirement:** All primary view titles must use standard `h1` page heading styling:
```html
<!-- ✅ STANDARD PAGE TITLE -->
<h1 class="ui-page-title text-2xl font-bold text-[var(--text-primary)]">
  Application Settings
</h1>
<p class="text-sm text-[var(--text-muted)] mt-1 mb-4">
  Configure Google Gemini API keys, file formats, and storage locations.
</p>
```

---

## 6. Action Semantics: Cancel vs Stop (ADR 011)

- **Cancel (Atomic Rollback):** Strictly reserved for actions that **discard uncommitted user input** or **roll back in-flight changes** (e.g., dismissing modals/forms without saving, cancelling a storage migration by deleting partial targets, or aborting a setup wizard). When "Cancel" is clicked, zero changes from that operation persist.
- **Stop (Progressive Halt):** Strictly reserved for **halting iterative or batch background processes mid-flight** without rollback (e.g., bulk importing designs, unified backfill/tagging, unmatched files reconciliation, design sync). When "Stop" is clicked, the system halts processing after the current item, and all records/files processed and committed up to that moment **are retained**.
- **Progressive Feedback:** In-flight stopping actions must display active progress feedback: `"Stop"` transitions to `"Stopping..."` and becomes disabled while waiting for the background task to safely finish its current unit of work.

---

## 7. Card Contract & Grid Standards

Applies to dense browse/project card grids unless overridden by a specific sub-spec:
- **Uniform Dimensions:** Outer card dimensions must be uniform within a visible grid row.
- **Full Image Fit:** Card media must preserve aspect ratio and show the full image when source data is available. Centered in `bg-[var(--surface-preview)]`.
- **Media Viewport:** Media viewport may be fixed; rendered image can vary inside viewport based on intrinsic ratio.
- **Deterministic Truncation:** Metadata overflow must be truncated using deterministic rules (single-line ellipsis or max line clamp).
- **Information Hierarchy:** Media $\rightarrow$ Identity (filename/title) $\rightarrow$ Validation/status cues $\rightarrow$ Contextual metadata $\rightarrow$ Rating/action cues.

---

## 8. UI Component Specifications

### 8.1 Page Headings & Subtitles
- **Root Element:** `<h1 class="ui-page-title text-2xl font-bold text-[var(--text-primary)]">`
- **Subtitle:** `<p class="text-sm text-[var(--text-muted)] mt-1 mb-4">`
- **Section Headings (`h2`):** `<h2 class="text-sm font-semibold text-[var(--text-secondary)] mb-1 flex items-center gap-1.5">`
- **Sub-section / Panel Headers (`h3`):** `<h3 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wide">`

### 8.2 Cards & Form Containers
- **Main View Cards:**
  ```html
  <div class="route-card bg-[var(--surface-card)] rounded-xl border border-[var(--border-default)] shadow p-6 space-y-4">
    ...
  </div>
  ```

### 8.3 Buttons & Actions
- **Primary Action (Brand solid):**
  - Class: `.btn-primary` or `.menu-button-primary`
  - Style: `var(--control-accent)` background, white text, `0.375rem` radius, `0.5rem 1rem` padding.
- **Secondary Action (Neutral Outline):**
  - Class: `.btn-secondary` or `.menu-button-secondary`
  - Style: `border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] hover:bg-[var(--surface-hover)]`.
- **Destructive Action:**
  - Class: `.btn-danger` or `.menu-button-danger`
  - Style: `background-color: var(--notice-error-border); color: var(--notice-error-text);`.
- **Button Sizing & Rhythm:**
  - Primary button height target: `2rem` to `2.5rem`.
  - Button labels use sentence case.
  - Action button groups use `.ui-action-button-group` (`flex`, `flex-wrap`, `gap: 1rem`, `align-items: center`).

### 8.4 Form Controls & Inputs
- **Text & Number Inputs:**
  - Class: `.input-field`, `.settings-input`
  - Attributes: `background: var(--surface-input); color: var(--text-primary); border: 1px solid var(--border-default); border-radius: 0.375rem; padding: 0.5rem 0.75rem; font-size: 0.875rem;`
  - Focus state: `border-color: var(--border-focus); box-shadow: 0 0 0 1px var(--border-focus);`
- **Checkboxes & Radios:**
  - Class: `accent-[var(--text-brand)]`
  - Visible border with `var(--border-default)` in unselected states.
- **Monospace Code / Path Display:**
  - `font-mono text-xs text-[var(--text-secondary)] bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)] px-2.5 py-1.5 break-all`

### 8.5 Tables & Data Grids
- **Container:** `.admin-table-shell overflow-auto border border-[var(--border-default)] rounded shadow bg-[var(--surface-card)]`
- **Table Header (`thead tr`):** `bg-[var(--surface-card-subtle)] border-b border-[var(--border-default)] text-[var(--text-secondary)] font-semibold text-xs uppercase tracking-wider`
- **Table Body Rows (`tbody tr`):** `border-b border-[var(--border-subtle)] hover:bg-[var(--surface-hover)] text-sm text-[var(--text-primary)]`
- **Empty State:** `px-4 py-8 text-center text-[var(--text-dim)] italic`

### 8.6 Alerts, Banners & Notices
- **Info Notice:**
  - Class: `.notice-info` (uses `var(--notice-info-bg)`, `var(--notice-info-border)`, `var(--notice-info-text)`)
- **Success Notice:**
  - Class: `.notice-success` (uses `var(--notice-success-bg)`, `var(--notice-success-border)`, `var(--notice-success-text)`)
- **Warning / Advisory Notice:**
  - Class: `.notice-warn` (uses `var(--notice-warn-bg)`, `var(--notice-warn-border)`, `var(--notice-warn-text)`)
- **Error / Failure Notice:**
  - Class: `.notice-error` (uses `var(--notice-error-bg)`, `var(--notice-error-border)`, `var(--notice-error-text)`)

### 8.7 Semantic Badges & Status Indicators
- **Verified Status Badge:**
  - Class: `.badge-status-verified` (uses `var(--notice-success-border)` background, `var(--notice-success-text)` text)
- **Partial / Warning Status Badge:**
  - Class: `.badge-status-warn` (uses `var(--notice-warn-border)` background, `var(--notice-warn-text)` text)
- **Unverified / Error Status Badge:**
  - Class: `.badge-status-error` (uses `var(--notice-error-border)` background, `var(--notice-error-text)` text)
- **Star Ratings:**
  - Class: `.text-rating-star` (uses `var(--notice-warn-text)`)

---

## 9. Modals and Confirmation Dialogs (ADR 010)

- **Absolute Ban on Native Webview Dialogs:** Never use browser-native `window.confirm()`, `window.alert()`, or `window.prompt()`. Synchronous webview popups freeze the WebView2 UI thread, break theme styling, and stall headless Playwright e2e test runs.
- **Theme & Surface Parity:** All modals must use shared modal container classes (`.modal-overlay`, `.modal-backdrop`, `.modal-dialog`, `.modal-header`, `.modal-body`, `.modal-footer`) with CSS custom properties (`--surface-card`, `--text-primary`, `--border-default`) for seamless light and dark mode support.
- **Accessibility & Focus:**
  - Modals must expose `role="dialog"`, `aria-modal="true"`, and `aria-labelledby` referencing the dialog title element.
  - Modals must support keyboard dismissal via `Escape` (`onkeydown`) and backdrop dismissal (disabled only while background operations are actively executing).
  - Dialog elements must portal to `document.body` to avoid parent stacking or clipping issues.
- **Button Standards:**
  - Footer buttons must use sentence case (*"Cancel"*, *"Deactivate licence"*, *"Delete record"*).
  - Destructive confirmations must style the confirmation button with `.menu-button-danger` and the cancellation action with `.menu-button-secondary`.
- **E2E & Component Testing:**
  - Modals and their interactive buttons must expose clear `data-testid` attributes (for example, `confirm-...-modal`, `confirm-...-button`, `cancel-...-button`).

---

## 10. Svelte 5 Implementation & Verification Checklist

When creating or modifying any view or component in `frontend/src/lib/`:

1. **Heading Standard:** Ensure the main view title is `<h1 class="ui-page-title text-2xl font-bold text-[var(--text-primary)]">`.
2. **Surfaces & Cards:** Use `bg-[var(--surface-card)]` with `border-[var(--border-default)]` for cards, modals, and panels.
3. **Sub-item Grids:** Use `bg-[var(--surface-card-subtle)]` and `hover:bg-[var(--surface-hover)]` for selectable tiles (e.g. Master formats, Hoop selector, Category tiles).
4. **Theme-Safe Classes:** Avoid plain `bg-white` or `bg-gray-50/50` without verifying dark mode rendering. Prefer CSS token variables (`var(--...)`) or explicit `dark:` classes.
5. **No Direct `invoke()` Calls:** Abstract all Tauri IPC calls into `src/lib/api/commandAdapter.ts` or dedicated service modules.
6. **Component Size Limit:** Keep components strictly under 500 lines by decomposing complex UIs into modular subcomponents under `src/lib/components/<domain>/`.
7. **Verification Gates:**
   - Type-check: `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"`
   - Test suite: `npm test` from the repository root.
   - Format check: `npm --prefix frontend run format:check` and `npx prettier --check .`
