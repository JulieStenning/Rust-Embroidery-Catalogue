## Implementation Plan: DesignDetailView Two-Column Layout Refactor

### 1. CSS Grid / Flex Layout Structure

**Overall page container (replacing the outer `<div class="space-y-4 font-sans">`):**

```svelte
<div class="detail-page font-sans h-[calc(100vh-3.5rem)] flex flex-col">
  <!-- Top: navigation breadcrumb bar (stays as-is but compacted) -->
  <div class="flex flex-wrap gap-2 px-4 pt-4 pb-2">
    ... buttons ...
    ... action notice ...
  </div>
  
  <!-- Main: two-column flex area filling remaining height -->
  <div class="flex-1 flex flex-col lg:flex-row gap-0 min-h-0">
    <!-- LEFT: sticky preview column -->
    <div class="lg:w-5/12 xl:w-2/5 lg:sticky lg:top-0 lg:self-start lg:max-h-[calc(100vh-3.5rem)] flex flex-col p-4 overflow-y-auto border-r border-gray-200">
      ... preview image + action buttons ...
    </div>
    
    <!-- RIGHT: scrollable content column -->
    <div class="lg:w-7/12 xl:w-3/5 flex-1 overflow-y-auto p-4">
      ... metadata, rating, tags, notes, projects ...
    </div>
  </div>
</div>
```

**Key CSS decisions:**
- `h-[calc(100vh-3.5rem)]` — The `3.5rem` (~56px) subtracts the top navigation bar height (`.menu-shell-inner` has `py-3` = 24px + line height ≈ 56px).
- `min-h-0` on the flex container is **critical** — without it, flex children won't shrink below their content height, breaking the scroll.
- Left column uses `lg:sticky lg:top-0 lg:self-start` so it stays visible while the right column scrolls independently.
- Left column gets `overflow-y-auto` as a safety net for very small viewports.
- Right column gets `overflow-y-auto` for independent scrolling.
- `border-r border-gray-200` on the left column provides a clean visual divider.

**Alternative approach if `h-[calc(100vh-3.5rem)]` proves fragile with dynamic nav height:**
Use a CSS custom property set via JavaScript or a simpler `min-h-screen` approach with `overflow-hidden` on the parent. But the calc approach is simpler and the nav height is static, so I'll start with it.

### 2. Component Breakdown

**New component: `TechnicalDataGrid.svelte`** at `frontend/src/lib/components/TechnicalDataGrid.svelte`

This component replaces the 8 read-only `.route-card` divs currently in a `grid sm:grid-cols-2 gap-3`.

**Props interface:**
```typescript
interface TechnicalDataItem {
  label: string;
  value: string | number;
}

let { items = [] }: { items: TechnicalDataItem[] } = $props();
```

**Rendering approach — compact key-value badge grid:**
```svelte
<div class="grid grid-cols-2 sm:grid-cols-4 gap-x-3 gap-y-2">
  {#each items as item}
    <div class="flex flex-col bg-gray-50 rounded border border-gray-200 px-2.5 py-1.5">
      <span class="text-[10px] font-semibold text-gray-400 uppercase tracking-wide">{item.label}</span>
      <span class="text-sm font-medium text-gray-800">{item.value ?? "?"}</span>
    </div>
  {/each}
</div>
```

This collapses 8 full-width cards into a compact 2×4 or 4×2 grid of badge-like cells with tiny uppercase labels and prominent values — far less vertical space.

**Data passed from parent:**
```javascript
let technicalItems = $derived([
  { label: "Hoop", value: detailItem?.hoop || "Unknown" },
  { label: "Date Added", value: detailItem?.date_added || "Unknown" },
  { label: "Dimensions", value: `${detailItem?.width_mm ?? "?"} × ${detailItem?.height_mm ?? "?"} mm` },
  { label: "Stitches", value: detailItem?.stitch_count ?? "?" },
  { label: "Colours", value: detailItem?.color_count ?? "?" },
  { label: "Colour Changes", value: detailItem?.color_change_count ?? "?" },
]);
```

**No other new components needed** — the existing pattern of inlining sections (rating, notes, projects, tags) within `.route-panel` divs works well; they just need spacing reductions.

**Filepath display:** Currently a full-width monospace block. Move it to a collapsed/inline area below the preview image or as a `<details>` element to save space.

### 3. Spacing & Padding Reductions

All `space-y-4` → `space-y-2.5` or `space-y-3` within the right column. Specific changes:

| Current | Proposed | Location |
|---------|----------|----------|
| `space-y-4` (outer wrapper) | Removed (new layout) | Top-level div |
| `p-6` on route-panel | `p-3` or removed entirely | Main content card |
| `p-4` on section panels | `p-3` | Rating, Metadata, Projects, Tags |
| `gap-3` on metadata grid | Replaced by TechnicalDataGrid | Read-only metadata |
| `space-y-3` within panels | `space-y-2` | All sub-sections |
| `pt-2` spacers between sections | `pt-1.5` | Visual breaks |

### 4. Sticky Left Column — Technical Details

```svelte
<!-- Inside the right side of the flex row -->
<div class="lg:w-7/12 xl:w-3/5 flex-1 overflow-y-auto p-4 space-y-3">
```

The `overflow-y-auto` on the right column combined with `min-h-0` on the parent flex container is what enables independent scrolling. The key CSS chain is:

```
.detail-two-column (flex, h-[calc(100vh-3.5rem)])  
  → .detail-two-column-body (flex-1, flex, min-h-0)  
    → left (sticky, max-h constrained)  
    → right (flex-1, overflow-y-auto) ← scrolls
```

### 5. Section Reordering (Right Column)

To ensure the most important editable controls appear above the fold on 1080p:

1. **Editable metadata** (Designer + Source dropdowns) — top of right column
2. **TechnicalDataGrid** (read-only) — immediately below
3. **Rating & Stitched status** — Compact inline row
4. **Tags** (with current assigned tags as pills + "Choose tags" button)
5. **Notes** textarea
6. **Projects** section
7. **Delete** button — bottom-right

### 6. File-by-File Changes

| File | Action |
|------|--------|
| `frontend/src/lib/components/TechnicalDataGrid.svelte` | **CREATE** — new compact metadata badge grid |
| `frontend/src/lib/views/DesignDetailView.svelte` | **REFACTOR** — restructure into two-column layout, replace read-only metadata with TechnicalDataGrid, reduce spacing throughout |
| `frontend/src/app.css` | **MINIMAL ADDITION** — add `.detail-page` class and `.detail-two-column` helper if needed (or just use Tailwind utilities) |

### 7. Risk Mitigation

- **The `h-[calc(100vh-3.5rem)]` approach:** If the nav bar height varies (e.g., wraps on narrow screens), the layout could break. Fallback: use `h-screen` with a negative top-margin and padding-top, or add a small JS-based resize observer. For this app, the nav uses `white-space: nowrap` → it won't wrap → the height is stable at ~3.5rem.
- **Dark mode:** Already handled via the existing `@media (prefers-color-scheme: dark)` rules in `app.css` which override Tailwind utility classes. The TechnicalDataGrid and new layout elements use the same `bg-gray-50`, `text-gray-800` etc. classes already covered by dark mode overrides.
- **Responsiveness below lg breakpoint:** At `<1024px`, the layout collapses to a single column (same as current behavior with `lg:grid-cols-2`). The `lg:` prefix on all flex classes ensures this.
- **Tag chooser modal:** Already uses `portalToBody` with `position: fixed`, so it's unaffected by the layout change.
- **Delete modal:** Same — portal-based, unaffected.

### 8. Verification Steps After Implementation

1. `cd frontend && npx svelte-check` — must pass with zero type errors
2. `cd frontend && npm run build` (or `npx vite build`) — must produce no errors
3. Manual visual check at 1080p and 1440p — preview pane should stay visible while scrolling the right column
4. Test all interactive controls (rating, stitched toggle, tag save, project add/remove, metadata save, delete) still function correctly
5. Verify dark mode appearance