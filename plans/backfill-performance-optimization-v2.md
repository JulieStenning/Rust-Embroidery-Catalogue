# Backfill Performance Optimization — Revised Architecture (v2)

## Design Decisions (Confirmed with User)

| Decision | Choice | Rationale |
|---|---|---|
| Chunk size | `commit_every` (small chunks) | Progressive commits, responsive stop, at most `workers × commit_every` designs processed after stop |
| Commit trigger | Count-based | Simple, predictable, worked well in testing |
| Design selection | Option B — combined list with per-design action metadata | One file read per design, only do needed work |
| Stitching query | Designs without ANY stitching tag (`NOT IN` subquery) | Correct semantics — if no stitching tag exists, run analysis |
| `batch_size` | Remove from non-tagging paths | Only relevant for Gemini API batching |
| Writer pattern | Single writer (main process) | Avoids SQLite locking contention |
| Stop signal | Sentinel file + `shutdown(wait=False, cancel_futures=True)` | Workers detect file, queued futures cancelled immediately |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Main Process                          │
│                                                         │
│  1. Query designs needing each action                   │
│  2. Merge into combined list with per-design metadata   │
│  3. Split into chunks of size = commit_every            │
│  4. Submit chunks to ProcessPoolExecutor                │
│  5. Collect results via as_completed                    │
│  6. Write results to DB sequentially (single writer)    │
│  7. Commit every commit_every designs                   │
│  8. Check stop signal after each chunk                  │
└──────────────┬──────────────────────────────┬───────────┘
               │ submit chunks                │ results
               ▼                              ▲
┌─────────────────────────────────────────────────────────┐
│              Worker Processes (N workers)                │
│                                                         │
│  For each design in chunk:                              │
│    1. Check stop sentinel → exit if set                 │
│    2. Read file with pyembroidery.read() (once)         │
│    3. If design needs images → render preview           │
│    4. If design needs color_counts → extract counts     │
│    5. If design needs stitching → run stitch_identifier │
│    6. Return result dict (no DB writes)                 │
└─────────────────────────────────────────────────────────┘
```

## Design Selection Queries

Each action type has its own query to find which designs need processing. These are merged into a single deduplicated list with per-design action metadata.

### Images Query
```sql
SELECT id, filename, filepath, image_data, width_mm, height_mm, hoop_id
FROM designs
WHERE image_data IS NULL
```
(If `redo=True`, omit the `WHERE` clause — all designs need images.)

### Color Counts Query
```sql
SELECT id, filename, filepath, stitch_count, color_count, color_change_count
FROM designs
WHERE stitch_count IS NULL
   OR color_count IS NULL
   OR color_change_count IS NULL
```
(If `redo=True`, omit the `WHERE` clause — all designs need color counts.)

### Stitching Query
```sql
SELECT id, filename, filepath
FROM designs
WHERE id NOT IN (
    SELECT design_id FROM design_tags
    JOIN tags ON tags.id = design_tags.tag_id
    WHERE tags.tag_group = 'stitching'
)
```
(If "Clear existing stitching tags for unverified designs first" is checked, the clear happens BEFORE this query, so unverified designs will have no stitching tags and will be picked up.)

### Merge Logic

```python
design_map: dict[int, DesignWorkItem] = {}

# Images
for row in images_query:
    design_map.setdefault(row.id, DesignWorkItem(id=row.id, filename=row.filename, filepath=row.filepath))
    design_map[row.id].needs_images = True
    design_map[row.id].image_data = row.image_data
    design_map[row.id].width_mm = row.width_mm
    design_map[row.id].height_mm = row.height_mm
    design_map[row.id].hoop_id = row.hoop_id

# Color counts
for row in color_counts_query:
    design_map.setdefault(row.id, DesignWorkItem(...))
    design_map[row.id].needs_color_counts = True
    design_map[row.id].stitch_count = row.stitch_count
    design_map[row.id].color_count = row.color_count
    design_map[row.id].color_change_count = row.color_change_count

# Stitching
for row in stitching_query:
    design_map.setdefault(row.id, DesignWorkItem(...))
    design_map[row.id].needs_stitching = True
```

## Worker Function

```python
def _process_design_batch_worker(
    design_rows: list[DesignWorkItem],
    actions: dict,
    designs_base_path: str,
) -> list[dict]:
    """Process a batch of designs.

    Workers do CPU-bound work only (reading files, rendering previews,
    analysing stitches) and return result dicts.  They do NOT write to
    the database — the main process collects results and writes them
    sequentially, avoiding SQLite locking contention.

    Each DesignWorkItem carries metadata about which actions are needed,
    so the worker only does the work required for each design.
    """
    results = []
    for item in design_rows:
        # Check stop sentinel
        if _stop_sentinel_exists():
            break

        result = {
            "design_id": item.id,
            "filename": item.filename,
            "error": None,
            "image_data": None,
            "width_mm": item.width_mm,
            "height_mm": item.height_mm,
            "hoop_id": item.hoop_id,
            "stitch_count": item.stitch_count,
            "color_count": item.color_count,
            "color_change_count": item.color_change_count,
            "stitching_tag_descriptions": None,
        }

        try:
            # Read the file ONCE — only if any action needs it
            pattern = None
            if item.needs_images or item.needs_color_counts or item.needs_stitching:
                pattern = pyembroidery.read(full_path)

            if item.needs_images and pattern is not None:
                result["image_data"] = _render_preview(pattern, preview_3d=...)
                bounds = pattern.bounds()
                if bounds:
                    result["width_mm"] = round((max_x - min_x) / 10.0, 2)
                    result["height_mm"] = round((max_y - min_y) / 10.0, 2)

            if item.needs_color_counts and pattern is not None:
                result["stitch_count"] = pattern.count_stitches()
                result["color_count"] = pattern.count_threads()
                result["color_change_count"] = pattern.count_color_changes()

            if item.needs_stitching and pattern is not None:
                matched = suggest_stitching_from_pattern(...)
                result["stitching_tag_descriptions"] = sorted(matched) if matched else []

        except Exception as exc:
            result["error"] = f"{type(exc).__name__}: {exc}"

        results.append(result)

    return results
```

## Main Process Flow

```
unified_backfill(db, actions, commit_every, workers, ...):
  1. Clear stop signal and sentinel
  2. Run "clear existing stitching tags" if requested (bulk DELETE)
  3. Query designs for each action type → merge into DesignWorkItem list
  4. If no designs match → return early
  5. Optimise SQLite for bulk write (synchronous=OFF, cache_size, temp_store)
  6. Split DesignWorkItem list into chunks of size = commit_every
  7. If workers > 1 and parallel_actions exist:
       a. Submit chunks to ProcessPoolExecutor
       b. For each completed future (as_completed):
          - Write results to DB sequentially
          - Commit every commit_every designs
          - Check stop signal → shutdown(wait=False, cancel_futures=True) → break
  8. Else (sequential fallback):
       a. Process each design in-process
       b. Write results to DB
       c. Commit every commit_every designs
  9. Restore SQLite pragmas
  10. Return summary dict
```

## Key Changes from Current Implementation

1. **Per-design action metadata** — Instead of a flat list of design rows, each `DesignWorkItem` carries `needs_images`, `needs_color_counts`, `needs_stitching` booleans. The worker checks these to decide what work to do.

2. **Correct stitching query** — Changed from `tags_checked.is_(False)` to `id NOT IN (SELECT design_id FROM design_tags JOIN tags ON ... WHERE tags.tag_group = 'stitching')`.

3. **Remove `batch_size` from non-tagging paths** — The `batch_size` parameter is only relevant for Gemini API batching. For images/stitching/color-counts, chunk size is always `commit_every`.

4. **Single file read** — The worker reads the file once with `pyembroidery.read()` and reuses the pattern for all needed actions.

## Files to Modify

| File | Changes |
|---|---|
| `src/services/unified_backfill.py` | Rewrite design selection queries, add `DesignWorkItem` dataclass, update worker to use per-design metadata, fix stitching query, remove `batch_size` from non-tagging paths |
| `src/routes/tagging_actions.py` | Minor — ensure `batch_size` is only passed for tagging actions |
| `templates/admin/tagging_actions.html` | Minor — clarify that `batch_size` only affects Gemini tagging |

## Acceptance Criteria

- [ ] Images-only backfill processes designs without `image_data`
- [ ] Color-counts-only backfill processes designs with null stitch/color/change counts
- [ ] Color-counts with "refresh all" processes ALL designs
- [ ] Stitching-only backfill processes designs without any stitching tag
- [ ] Combined actions process each design once, reading the file once
- [ ] Workers only do the work needed for each design (check per-design metadata)
- [ ] `commit_every` controls both chunk size and commit frequency
- [ ] `batch_size` has no effect on images/stitching/color-counts paths
- [ ] Stop signal stops workers and commits completed results
- [ ] No SQLite locking contention (single writer)
