# Backfill Performance Optimization Plan

## Current Performance Profile

Based on code analysis of [`src/services/unified_backfill.py`](src/services/unified_backfill.py), [`src/services/preview.py`](src/services/preview.py), and [`src/services/stitch_identifier.py`](src/services/stitch_identifier.py):

| Operation | Time per design | 100,000 designs |
|---|---|---|
| Preview image (3D render) | ~8s | ~222 hrs (9.3 days) |
| Stitch type analysis + color counts | ~2s | ~55 hrs (2.3 days) |
| **Combined (sequential)** | **~10s** | **~278 hrs (11.6 days)** |

## Root Causes

1. **Strictly sequential processing** — The [`unified_backfill()`](src/services/unified_backfill.py:281) loop processes one design at a time in a single thread. No parallelism whatsoever.

2. **3D preview rendering is expensive** — [`_render_preview()`](src/services/preview.py:304) uses `pyembroidery.PngWriter.write` with `{"3d": True}`. The 3D simulation (stitch relief/shading) is computationally heavy.

3. **File I/O is a hidden bottleneck** — Each design file is read from disk via `pyembroidery.read()`. For 100K files scattered across folders, the OS overhead of opening/closing handles adds up significantly.

4. **SQLite write contention** — The database is SQLite with WAL mode ([`src/database.py`](src/database.py:82)), which supports concurrent reads but serializes writes. Commits every 100 designs create a sync bottleneck.

## Optimization Strategies (Ordered by Impact)

### Strategy 1: Parallel Processing with `multiprocessing.Pool` (HIGHEST IMPACT)

**Estimated speedup: 4-8x on a typical multi-core machine**

The single-threaded loop in [`unified_backfill()`](src/services/unified_backfill.py:408) is the #1 bottleneck. Embroidery file parsing (`pyembroidery.read`) and preview rendering (`PngWriter.write`) are CPU-bound operations that release the GIL (they're implemented in C/Cython). This makes them ideal for multiprocessing.

**Implementation approach:**

- Create a worker pool using `multiprocessing.Pool` (or `concurrent.futures.ProcessPoolExecutor`).
- Split the design IDs into N chunks (where N = CPU core count, e.g. 8-16).
- Each worker process:
  - Opens its own SQLite database connection (SQLite WAL mode supports concurrent readers).
  - Reads the design file, renders preview, runs stitch analysis.
  - Returns results as serializable dicts (not ORM objects).
- The main process collects results and writes them to the DB in batches.

**Key considerations:**
- SQLite in WAL mode supports multiple concurrent readers but only one writer. Workers must NOT write — they return data to the main process which handles all writes.
- The `pyembroidery.EmbPattern` object is not picklable, so file reading must happen inside the worker process.
- Need to handle the stop-signal mechanism across processes (use a `multiprocessing.Event` or a sentinel file).

**Estimated time reduction:** 11.6 days → **~1.5-3 days** (with 8 workers)

---

### Strategy 2: Optional 2D Preview Mode (MEDIUM IMPACT)

**Estimated speedup: 2-4x on the preview step alone**

The 3D rendering in [`_render_preview()`](src/services/preview.py:313) is the heaviest single operation. Adding an option to render a flat 2D preview instead would dramatically speed up the initial backfill.

**Implementation approach:**

- Add a `settings={"3d": False}` option to `PngWriter.write`.
- Add a `preview_mode` parameter to the backfill UI and CLI: `"3d"` (default, current behavior) or `"2d"` (fast).
- The 2D mode renders a flat stitch map without the 3D relief simulation.

**Trade-off:** 2D previews are less visually appealing but perfectly adequate for catalogue browsing. Users can selectively re-render specific designs in 3D later.

**Estimated time reduction:** Preview step from 8s → **~2-3s per design**

---

### Strategy 3: Batched File Reading (LOW-MEDIUM IMPACT)

**Estimated speedup: 1.2-1.5x on the file I/O portion**

Currently, each design's file is read individually. If multiple designs share the same parent directory, the OS directory traversal overhead is repeated.

**Implementation approach:**

- Group designs by their parent directory before processing.
- Use `os.scandir()` or `pathlib.Path.iterdir()` to batch-read directory listings.
- Not a huge win since `pyembroidery.read()` is the dominant cost, not directory traversal.

---

### Strategy 4: SQLite Write Optimization (LOW IMPACT)

**Estimated speedup: 1.1-1.2x**

Current commit frequency is every 100 designs. SQLite's transaction overhead for individual writes adds up.

**Implementation approach:**

- Increase commit batch size to 500-1000 for the backfill (the UI already has a `commit_every` parameter).
- Use `PRAGMA synchronous = OFF` and `PRAGMA cache_size = -80000` during backfill (trade durability for speed — acceptable for a batch operation that can be re-run).
- Use bulk INSERT/UPDATE via `db.bulk_update_mappings()` instead of ORM attribute assignment for the image_data blob column.

**Note:** The `image_data` column is `LargeBinary` — storing 50-200KB PNG blobs per row. Bulk operations on blobs are tricky with SQLite. This is a secondary optimization.

---

### Strategy 5: Skip Already-Processed Designs More Aggressively (ALREADY DONE)

The current code already skips designs where `image_data IS NOT NULL` (for images) or where stitch/color counts are already populated. No change needed here.

---

## Combined Projection

| Strategy | Alone | Cumulative |
|---|---|---|
| Current (sequential, 3D) | 11.6 days | 11.6 days |
| + Parallel (8 workers) | ~1.5-3 days | ~1.5-3 days |
| + 2D preview mode | ~3-4 days (sequential) | ~12-18 hours (parallel + 2D) |
| + SQLite tuning | ~10 days (sequential) | ~10-15 hours (all combined) |

---

## Implementation Plan (Actionable Steps)

### Step 1: Add parallel processing to `unified_backfill()`

**Files to modify:**
- [`src/services/unified_backfill.py`](src/services/unified_backfill.py)

**Changes:**
1. Create a new function `_process_design_batch(design_ids, actions, db_url, ...)` that:
   - Opens its own DB connection using the `DATABASE_URL`.
   - Queries designs by the provided IDs.
   - For each design, reads the file, renders preview, runs stitch analysis.
   - Returns a list of result dicts (serializable, no ORM objects).
2. Modify `unified_backfill()` to:
   - Split `design_ids` into chunks (one per CPU core).
   - Use `concurrent.futures.ProcessPoolExecutor` to process chunks in parallel.
   - Collect results and write them to the DB in the main process.
3. Handle the stop signal via a `multiprocessing.Event` or a temporary sentinel file that workers check.

**Acceptance criteria:**
- All four action types (tagging, stitching, images, color_counts) work in parallel mode.
- The stop signal still works.
- Error logging still works.
- No data corruption — each design is processed exactly once.

### Step 2: Add 2D preview option

**Files to modify:**
- [`src/services/preview.py`](src/services/preview.py)
- [`src/services/unified_backfill.py`](src/services/unified_backfill.py)
- [`templates/admin/tagging_actions.html`](templates/admin/tagging_actions.html)

**Changes:**
1. In [`_render_preview()`](src/services/preview.py:304), accept a `preview_3d=True` parameter. When `False`, use `settings={"3d": False}`.
2. In [`unified_backfill()`](src/services/unified_backfill.py:281), accept a `preview_3d` option in the `actions["images"]` dict.
3. In the UI template, add a checkbox for "Fast 2D preview (skip 3D simulation)" under the Images options.

**Acceptance criteria:**
- 2D mode renders ~2-3x faster than 3D mode.
- 2D previews are visually distinguishable from 3D (flat vs. shaded).
- The setting is optional and defaults to 3D (backward compatible).

### Step 3: Optimize SQLite write performance

**Files to modify:**
- [`src/services/unified_backfill.py`](src/services/unified_backfill.py)

**Changes:**
1. Increase default `commit_every` from 100 to 500.
2. Add a context manager that temporarily sets SQLite pragmas for bulk operations:
   - `PRAGMA synchronous = OFF`
   - `PRAGMA cache_size = -80000` (80MB cache)
   - `PRAGMA temp_store = MEMORY`
3. Restore original pragmas after the backfill completes.

**Acceptance criteria:**
- Write throughput increases measurably.
- No data loss if the process is killed (the operation is idempotent and can be re-run).

### Step 4: Update CLI scripts for parallel mode

**Files to modify:**
- [`backfill_images.py`](backfill_images.py)
- [`backfill_stitching_tags.py`](backfill_stitching_tags.py)
- [`backfill_color_counts.py`](backfill_color_counts.py) (if exists)

**Changes:**
1. Add `--parallel` / `--workers` CLI arguments.
2. When `--parallel` is specified, use the same `ProcessPoolExecutor` approach.
3. Default to sequential for backward compatibility.

**Acceptance criteria:**
- CLI scripts accept `--workers N` argument.
- Parallel mode works correctly.
- Dry-run mode works in parallel.

### Step 5: Update UI for parallel options

**Files to modify:**
- [`templates/admin/tagging_actions.html`](templates/admin/tagging_actions.html)
- [`src/routes/tagging_actions.py`](src/routes/tagging_actions.py)

**Changes:**
1. Add a "Workers" number input to the unified backfill form (default: 1 = sequential).
2. Add the "Fast 2D preview" checkbox.
3. Pass these options through to `unified_backfill()`.

**Acceptance criteria:**
- UI controls are present and functional.
- Worker count defaults to 1 (safe, sequential).
- Tooltip explains that more workers = faster but uses more CPU/RAM.

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| SQLite concurrent read issues | Low | WAL mode already enabled; each worker opens its own connection |
| Memory exhaustion with many workers | Medium | Cap workers at `cpu_count()`; each worker processes a chunk, not all designs |
| pyembroidery thread safety | Low | Workers are separate processes (not threads), so no GIL/shared-state issues |
| Stop signal not working across processes | Medium | Use `multiprocessing.Event` or a file-based sentinel checked by workers |
| Image data blobs causing memory pressure | Low | Workers return results as dicts; main process writes in batches |

---

## Quick Wins (Can Be Done First)

1. **Increase `commit_every` from 100 to 500** — Simple one-line change, immediate ~10% write speedup.
2. **Add `--workers` CLI argument to `backfill_images.py`** — The standalone script is simpler than the unified backfill and can serve as a proof of concept for the parallel approach.
3. **Add 2D preview option** — Simple parameter change in `_render_preview()`, big impact on render time.
