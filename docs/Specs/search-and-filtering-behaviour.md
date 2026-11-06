# Search Query & Filtering Engine Specification

Technical reference for the search query parser, filter constraints, and deterministic sorting in Browse (`src/routes/designs.rs`, `frontend/src/lib/views/BrowseView.svelte`).

## Search Query Grammar

The search input in Browse parses queries into structured criteria before generating SQL `WHERE` clauses:

| Syntax | Operator | Parser Action | Example |
|---|---|---|---|
| Single Term | Word | Matches if token appears in selected scopes. | `rose` |
| Quoted Phrase | `"..."` | Exact substring match preserving internal spaces and order. | `"cross stitch"` |
| Negation | `-term` | Excludes records matching the term (`NOT LIKE` / `NOT EXISTS`). | `-applique` |
| Disjunction | `OR` | Matches records meeting either condition (evaluated between terms). | `rose OR tulip` |
| Wildcard | `*` | Translated into SQL `%` wildcard pattern. | `*.pes`, `rose*.jef` |

### Search Scopes
Users can toggle which database columns are evaluated:
- **File name:** Matches against `designs.filename`.
- **Folder name:** Matches against `designs.filepath`.
- **Tags:** Evaluates against linked `tags.name` via `design_tags`.
- **Notes:** Evaluates against `designs.notes`.

## Filter Criteria & SQL Conditions

Browse supports multi-faceted filter parameters passed to `search_designs()`:

### 1. Categories & Attribution
- **Designers:** Array of `designer_id` values $\rightarrow$ `WHERE d.designer_id IN (...)`.
- **Sources:** Array of `source_id` values $\rightarrow$ `WHERE d.source_id IN (...)`.
- **Image Tags:** Array of `tag_id` values requiring subject classification $\rightarrow$ joined via `design_tags`.
- **Stitching Tags:** Array of `tag_id` values requiring technique classification $\rightarrow$ joined via `design_tags`.

### 2. Physical Dimensions & Hoops
- **Dimensions (mm):**
  - Bounded: `d.width_mm BETWEEN :min_w AND :max_w`
  - Open-ended min: `d.width_mm >= :min_w`
  - Open-ended max: `d.width_mm <= :max_w`
- **Hoop Frame:** Filter by assigned `d.hoop_id`, or `hoop_id IS NULL` for uncalculated hoops.

### 3. Verification & Attention States
- **Verification Flags:**
  - `image_tags_verified`: `1` (verified) or `0` (unverified).
  - `stitching_tags_verified`: `1` (verified) or `0` (unverified).
  - Composite modes: **Verified only** (both verified), **Unverified only** (at least one unverified), **Partially verified** (one verified, one unverified).
- **Needs Attention Filter:**
  - `cannot_be_drawn`: `d.preview_error IS NOT NULL`
  - `exceeds_hoop`: `d.width_mm > h.width_mm OR d.height_mm > h.height_mm`
  - `master_only`: `d.is_master = 1 AND NOT EXISTS (SELECT 1 FROM designs s WHERE s.filepath = REPLACE(d.filepath, ...) AND s.is_master = 0)`

## Sorting & Deterministic Ordering

| Sort Key | Primary Sort SQL | Secondary Tie-Break |
|---|---|---|
| `date_added` | `d.date_added ASC / DESC` | `d.id ASC` |
| `date_updated` | `d.date_updated ASC / DESC` | `d.id ASC` |
| `name` | `d.filename ASC / DESC` | `d.id ASC` |
| `rating` | `d.rating ASC / DESC` | `d.filename ASC` |
| `stitched` | `d.stitched ASC / DESC` | `d.filename ASC` |
| `folder` | `folder_path ASC / DESC` | `d.filename ASC` (Always ASC) |

### Folder Sort Secondary Invariant
When sorting by `folder` in **Descending** order, the directory paths are sorted `Z` to `A`, but the designs inside each folder remain strictly ordered alphabetically (`filename ASC`). This prevents numbered design sequences within a folder from reversing.

