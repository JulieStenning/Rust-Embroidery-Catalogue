## Implementation Plan: Star Rating UI Refactor on Browse Designs

### File to Modify
**`frontend/src/lib/MainView.svelte`** — This single file contains both the `browseStars` helper function and the inline card template that renders the star rating in the Browse Designs grid. There is no separate card component file.

---

### Changes Required

#### 1. Refactor the `browseStars` function (currently at line 1329)

Replace the current 5-star-pattern function with a new "Single Star Badge" helper. The new function will:

- Return a **rated badge** string: `"★ {value}"` (e.g., `"★ 3"`) when a valid rating exists.
- Return an **unrated placeholder** string: `"☆ —"` when no rating exists (instead of an empty string, which currently causes the element to be hidden entirely).

#### 2. Update the card template (currently at lines 2222–2226)

Replace the conditional `{#if item.rating != null && item.rating > 0}` block so that the rating badge is **always rendered**, keeping card height uniform:

- **Remove** the `{#if item.rating != null && item.rating > 0}` wrapper.
- **Always render** the `<p>` with class `browse-card-rating`.
- **Rated state**: Display the single gold star + number (e.g., `★ 3`) in `text-yellow-500 font-bold`.
- **Unrated state**: Display the muted placeholder (`☆ —`) in `text-gray-300` (or `text-gray-400`), matching the "No tags" placeholder style already used on the same cards.

#### 3. CSS adjustments (minimal, inline via Tailwind classes)

- The existing `browse-card-rating` class on the `<p>` tag uses `text-xs` and `mt-1` — these are fine and consistent.
- For the unrated state, apply a muted color (e.g., `text-gray-300` or `text-gray-400`) using a ternary in the class binding, while the rated state uses `text-yellow-500 font-bold` as before.

---

### Specific Edits (summary)

| Edit | Location | Description |
|------|----------|-------------|
| **Function replacement** | `browseStars` (line 1329) | Returns `"★ {value}"` for rated, `"☆ —"` for unrated |
| **Template: remove conditional** | Lines 2222–2226 | Remove `{#if ...}` — always render the rating `<p>` |
| **Template: dynamic classes** | `<p>` tag | Use `item.rating ? 'text-yellow-500 font-bold' : 'text-gray-300'` for color |
| **Template: aria-label** | `<p>` tag | Update to reflect the new single-star badge pattern |

### What Remains Unchanged
- The **Design Detail page** (`DesignDetailView.svelte`): Not touched per requirements.
- The **Projects view** (`ProjectsView.svelte`): Not touched per requirements.
- All other card layout, grid CSS, and card metadata structure remain identical.

Later updates
Task: Update Star Rating Colors on Browse Designs Page

Objective:
Improve text and icon contrast on design cards so ratings are clearly legible against white card backgrounds.

Color Specification Updates:
- Star Icon: Change the filled single star color to deep amber (`#D97706` / Tailwind `amber-600`).
- Rating Number: Change the text color of the rating number next to the star to dark charcoal/slate (`#374151` / Tailwind `gray-700`) to match card typography.
- Unrated Indicator: If applicable, style the unrated icon/dash in a soft neutral gray (`#9CA3AF` / Tailwind `gray-400`).

Requirements:
- Update the Browse Designs card component / CSS styles to reflect these new color values.
- Ensure the single-star badge layout implemented previously is preserved.