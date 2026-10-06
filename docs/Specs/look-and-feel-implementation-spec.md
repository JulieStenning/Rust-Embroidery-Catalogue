# Look and Feel & UI Theme Implementation Specification

## Status
- **Type:** Canonical UI & Theme Implementation Standard
- **Audience:** AI Agents, Developers, and Maintainers
- **Authority:** Authoritative reference for all UI components, views, layouts, and light/dark theme styling across Embroidery Catalogue
- **Last Updated:** October 2026 (Supersedes legacy May 2026 spec)

---

## 1. Executive Summary & Design Philosophy

Embroidery Catalogue is a fast, responsive, offline desktop application for cataloguing, browsing, and inspecting machine embroidery designs.

### Core Visual Principles
1. **Practical Desktop Utility:** Clean, high-density, focused workflow tool — not a flashy marketing website or heavyweight web portal.
2. **Complete Theme Parity (Light, Dark, and System):** Every view, card, modal, form, badge, and table must render with optimal contrast and visual hierarchy in **both Light and Dark modes**, whether following the OS preference (`system`) or forced by user setting (`light` / `dark`).
3. **Consistent Component Vocabulary:** Reuse established component patterns from our gold-standard views (**Browse Designs**, **Choose Tags**, and **Design Details**) rather than inventing ad-hoc styling.
4. **Token-Driven Architecture:** All colors, borders, and surfaces are driven by CSS custom properties defined in `:root`, `:root[data-theme="dark"]`, and scoped dark-mode media queries in `frontend/src/app.css`.

---

## 2. Exemplar Views (The Gold Standards)

When designing or updating any view, agents **MUST** align with the patterns proven in these three gold-standard views:

### 1. Browse Designs (`BrowseView.svelte` + subcomponents)
- **Header:** Clean `h1.ui-page-title` with `text-2xl font-bold text-[var(--text-primary)]`.
- **Search & Filter Shell:** Contained white/dark-card container (`bg-[var(--surface-card)] rounded shadow p-4 border border-[var(--border-default)]`).
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

---

## 4. Specific Regression Guardrails & Rules

### ⚠️ Rule 1: No Un-Gated Dark Mode Media Queries (Prevents "Dark Areas in Light Mode")
**Problem:** If CSS rules in `app.css` are placed inside `@media (prefers-color-scheme: dark) { ... }` without checking `data-theme`, a user whose OS is in Dark Mode who selects **Appearance: Light** will still receive dark styling on tables, cards, or titles.
**Requirement:** Any media-query dark overrides MUST be gated with `:root:not([data-theme="light"])`:
```css
/* ✅ CORRECT: Only applies if the user hasn't explicitly chosen Light */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) .admin-table-shell,
  :root[data-theme="dark"] .admin-table-shell {
    background: #1e293b !important;
    border-color: #334155 !important;
  }
}

/* ❌ INCORRECT: Forces dark background in Light Mode when OS is dark */
@media (prefers-color-scheme: dark) {
  .admin-table-shell {
    background: #1e293b !important;
  }
}
```

### ⚠️ Rule 2: No Hardcoded Opacity Gray Backgrounds on Cards (Prevents "White Glare in Dark Mode")
**Problem:** Using classes like `bg-gray-50/50` or `bg-white/80` creates bright translucent boxes on dark cards because Tailwind's escaped classes (`.bg-gray-50\/50`) bypass plain `.bg-gray-50` dark overrides.
**Requirement:** Use semantic tokens or explicit `dark:` classes for nested container items:
```html
<!-- ✅ CORRECT: Adapts cleanly in both light and dark mode -->
<label class="flex items-start gap-2.5 p-2.5 rounded border border-[var(--border-default)] bg-[var(--surface-card-subtle)] hover:bg-[var(--surface-hover)] cursor-pointer text-sm text-[var(--text-primary)]">
  ...
</label>

<!-- ❌ INCORRECT: Glows white in dark mode -->
<label class="border border-gray-200 bg-gray-50/50 hover:bg-gray-50 text-gray-700">
  ...
</label>
```

### ⚠️ Rule 3: Uniform Page Headings
**Problem:** Some pages have `h1` titles rendered at `text-lg` with `text-gray-500` or washed-out `#e5e7eb` in light mode, while other pages use `text-2xl font-bold text-gray-800`.
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

## 5. UI Component Specifications

### 5.1 Page Headings & Subtitles
- **Root Element:** `<h1 class="ui-page-title text-2xl font-bold text-[var(--text-primary)]">`
- **Subtitle:** `<p class="text-sm text-[var(--text-muted)] mt-1 mb-4">`
- **Section Headings (`h2`):** `<h2 class="text-sm font-semibold text-[var(--text-secondary)] mb-1 flex items-center gap-1.5">`
- **Sub-section / Panel Headers (`h3`):** `<h3 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wide">`

### 5.2 Cards & Form Containers
- **Main View Cards:**
  ```html
  <div class="settings-card bg-[var(--surface-card)] rounded border border-[var(--border-default)] shadow p-6 space-y-5">
    ...
  </div>
  ```
- **Sticky Form Headers (Settings / Wizards):**
  ```html
  <div class="sticky top-0 z-20 flex items-center justify-between gap-3 rounded-t border-b border-[var(--border-default)] bg-[var(--surface-card)] px-6 py-4 backdrop-blur">
    <h1 class="text-lg font-bold text-[var(--text-primary)]">Page Title</h1>
    <button type="submit" class="settings-primary-button menu-button-primary">Save Changes</button>
  </div>
  ```

### 5.3 Buttons & Actions
- **Primary Action (Purple/Indigo solid):**
  - Class: `.menu-button-primary` or `.settings-primary-button`
  - Style: `#4f46e5` / `#4338ca` background, `#ffffff` text, `0.35rem` radius, `0.5rem 1rem` padding.
- **Secondary Action (Neutral Outline):**
  - Class: `.menu-button-secondary`
  - Style: `border border-[var(--border-default)] bg-[var(--surface-card)] text-[var(--text-primary)] hover:bg-[var(--surface-hover)]`.
- **Ghost Action (Toolbar / Icons):**
  - Class: `.menu-button-ghost`
  - Style: subtle border or borderless, hover surface highlight.
- **Destructive Action:**
  - Class: `menu-button-secondary text-red-600 border-red-200 hover:bg-red-50 dark:text-red-400 dark:border-red-900/50 dark:hover:bg-red-950/30`.

### 5.4 Form Controls & Inputs
- **Text & Number Inputs:**
  - Class: `.ui-text-input`, `.settings-input`
  - Attributes: `background: var(--surface-input); color: var(--text-primary); border: 1px solid var(--border-default); border-radius: 0.35rem; padding: 0.45rem 0.75rem; font-size: 0.875rem;`
  - Focus state: `outline: none; border-color: var(--border-focus); box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);`
- **Dropdown Selects:**
  - Class: `.ui-select-input`
  - Consistent padding and matching height (`2rem` to `2.25rem`) to align with inputs.
- **Monospace Code / Path Display:**
  - `font-mono text-xs text-[var(--text-secondary)] bg-[var(--surface-card-subtle)] rounded border border-[var(--border-default)] px-2.5 py-1.5 break-all`

### 5.5 Tables & Data Grids
- **Container:** `.admin-table-shell overflow-auto border border-[var(--border-default)] rounded shadow bg-[var(--surface-card)]`
- **Table Header (`thead tr`):** `bg-[var(--surface-card-subtle)] border-b border-[var(--border-default)] text-[var(--text-secondary)] font-semibold text-xs uppercase tracking-wider`
- **Table Body Rows (`tbody tr`):** `border-b border-[var(--border-subtle)] hover:bg-[var(--surface-hover)] text-sm text-[var(--text-primary)]`
- **Empty State:** `px-4 py-8 text-center text-[var(--text-dim)] italic`

### 5.6 Alerts, Banners & Notices
- **Info Notice:**
  - Light: `bg-blue-50 border border-blue-200 text-blue-800`
  - Dark: `dark:bg-blue-950/40 dark:border-blue-800 dark:text-blue-300`
- **Success Notice:**
  - Light: `bg-green-50 border border-green-200 text-green-800`
  - Dark: `dark:bg-green-950/40 dark:border-green-800 dark:text-green-300`
- **Warning / Advisory Notice:**
  - Light: `bg-amber-50 border border-amber-300 text-amber-900`
  - Dark: `dark:bg-amber-950/40 dark:border-amber-800 dark:text-amber-300`
- **Error / Failure Notice:**
  - Light: `bg-red-50 border border-red-300 text-red-800`
  - Dark: `dark:bg-red-950/40 dark:border-red-800 dark:text-red-300`

### 5.7 Semantic Tag Pills & Badges
- **Image Tags (Subject / Theme):**
  - Light: `bg-green-100 text-green-800 border border-green-200`
  - Dark: `dark:bg-green-900/40 dark:text-green-300 dark:border-green-800`
- **Stitching Tags (Technical / Density):**
  - Light: `bg-blue-100 text-blue-800 border border-blue-200`
  - Dark: `dark:bg-blue-900/40 dark:text-blue-300 dark:border-blue-800`
- **Verified Badge:**
  - Light: `bg-emerald-100 text-emerald-800 border border-emerald-300`
  - Dark: `dark:bg-emerald-950/50 dark:text-emerald-300 dark:border-emerald-700`
- **Unverified Badge:**
  - Light: `bg-amber-100 text-amber-800 border border-amber-300`
  - Dark: `dark:bg-amber-950/50 dark:text-amber-300 dark:border-amber-700`

---

## 6. Svelte 5 Implementation Checklist for New & Migrated Views

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
