# Refactoring Plan: Eliminate Duplicate Preview / Stitching / Colour-Count Logic

## Problem Statement

There are **3-4 copies** of the same computation logic for:
1. Preview image rendering + bounds extraction
2. Thread/colour count extraction (`count_stitches`, `count_threads`, `count_color_changes`)
3. Stitching type detection (`suggest_stitching_from_pattern`)
4. Hoop selection from dimensions

These are scattered across:
- [`src/services/preview.py:_process_file()`](src/services/preview.py:333) — used during import scanning
- [`src/services/unified_backfill.py:run_images_action_runner()`](src/services/unified_backfill.py:310) — backfill sequential
- [`src/services/unified_backfill.py:run_color_counts_action_runner()`](src/services/unified_backfill.py:357) — backfill sequential
- [`src/services/unified_backfill.py:run_stitching_action_runner()`](src/services/unified_backfill.py:238) — backfill sequential
- [`src/services/unified_backfill.py:_process_design_batch_worker()`](src/services/unified_backfill.py:416) — backfill parallel worker
- [`src/services/bulk_import.py:_build_design_records()`](src/services/bulk_import.py:156) — import persistence (stitching only)

Any bug fix or enhancement must be applied in multiple places, and they can easily drift apart.

## Design Constraints

1. **Parallel worker cannot write to DB** — it returns result dicts for the main process to apply.
2. **Sequential runners write to DB directly** — they receive an ORM `Design` object.
3. **Import scanning** uses a `ScannedDesign` dataclass, not an ORM object.
4. **The parallel worker** cannot import the sequential runners (they'd try to write to DB).
5. **The sequential runners** cannot be called from the parallel worker for the same reason.

The solution is to extract **pure computation functions** (no DB access) that all paths can call, then keep the DB-writing logic in each path.

## Proposed Architecture

### New Module: `src/services/pattern_analysis.py`

Extract pure computation functions that take a `pyembroidery.EmbPattern` and return data — no DB access, no ORM objects.

```python
@dataclass
class PatternAnalysisResult:
    """Pure data — no DB objects, no ORM references."""
    image_data: bytes | None = None
    image_type: str | None = None
    width_mm: float | None = None
    height_mm: float | None = None
    stitch_count: int | None = None
    color_count: int | None = None
    color_change_count: int | None = None
    stitching_tag_descriptions: list[str] | None = None


def analyze_pattern(
    pattern: pyembroidery.EmbPattern,
    *,
    needs_images: bool = False,
    needs_color_counts: bool = False,
    needs_stitching: bool = False,
    preview_3d: bool = True,
    redo: bool = False,
    existing_image_data: bytes | None = None,
    existing_image_type: str | None = None,
    existing_width_mm: float | None = None,
    existing_height_mm: float | None = None,
    existing_stitch_count: int | None = None,
    existing_color_count: int | None = None,
    existing_color_change_count: int | None = None,
    filename: str = "",
    filepath: str = "",
    pattern_path: str = "",
    desc_to_tag: dict[str, Tag] | None = None,
    clear_existing_stitching: bool = False,
) -> PatternAnalysisResult:
    """Single function that does all the computation. Called by all three paths."""
    ...
```

### Callers

| Caller | What changes |
|--------|-------------|
| [`preview.py:_process_file()`](src/services/preview.py:333) | Replace inline logic with call to `analyze_pattern(pattern, needs_images=True, needs_color_counts=True, ...)` |
| [`unified_backfill.py:run_images_action_runner()`](src/services/unified_backfill.py:310) | Replace inline logic with call to `analyze_pattern(pattern, needs_images=True, ...)`, then write result to DB |
| [`unified_backfill.py:run_color_counts_action_runner()`](src/services/unified_backfill.py:357) | Replace inline logic with call to `analyze_pattern(pattern, needs_color_counts=True, ...)`, then write result to DB |
| [`unified_backfill.py:run_stitching_action_runner()`](src/services/unified_backfill.py:238) | Replace inline logic with call to `analyze_pattern(pattern, needs_stitching=True, ...)`, then write result to DB |
| [`unified_backfill.py:_process_design_batch_worker()`](src/services/unified_backfill.py:416) | Replace inline logic with call to `analyze_pattern(pattern, needs_images=..., needs_color_counts=..., needs_stitching=...)` — returns result dict directly |
| [`bulk_import.py:_build_design_records()`](src/services/bulk_import.py:156) | Already delegates to `suggest_stitching_from_pattern` — no change needed here (it reads the file separately) |

## Detailed Steps

### Step 1: Create `src/services/pattern_analysis.py`

Extract these pure functions from existing code:

#### 1a. `_render_preview_and_bounds(pattern, preview_3d, redo, existing_image_data, existing_image_type, existing_width_mm, existing_height_mm) -> tuple[bytes | None, str | None, float | None, float | None]`

Source: [`unified_backfill.py:run_images_action_runner()`](src/services/unified_backfill.py:320-341) and [`_process_design_batch_worker()`](src/services/unified_backfill.py:515-541)

Logic:
- Determine `should_render` from `redo`, `existing_image_data`, `upgrade_2d_to_3d`
- If should_render: call `_render_preview(pattern, preview_3d=preview_3d)`, set `image_type`
- Extract bounds from `pattern.bounds()`, compute `width_mm`/`height_mm`
- Return `(image_data, image_type, width_mm, height_mm)`

#### 1b. `_extract_color_counts(pattern, existing_stitch_count, existing_color_count, existing_color_change_count) -> tuple[int | None, int | None, int | None]`

Source: [`unified_backfill.py:run_color_counts_action_runner()`](src/services/unified_backfill.py:366-375) and [`_process_design_batch_worker()`](src/services/unified_backfill.py:545-551)

Logic:
- If `existing_stitch_count is None`: `pattern.count_stitches()`
- If `existing_color_count is None`: `pattern.count_threads()`
- If `existing_color_change_count is None`: `pattern.count_color_changes()`
- Return `(stitch_count, color_count, color_change_count)`

#### 1c. `_detect_stitching_tags(pattern, filename, filepath, pattern_path, desc_to_tag, clear_existing_stitching) -> list[str] | None`

Source: [`unified_backfill.py:run_stitching_action_runner()`](src/services/unified_backfill.py:253-293) and [`_process_design_batch_worker()`](src/services/unified_backfill.py:554-567)

Logic:
- Call `suggest_stitching_from_pattern(pattern_path, filename, filepath, desc_to_tag, pattern=pattern)`
- Return matched descriptions or `[]` if `clear_existing_stitching` is set

#### 1d. `analyze_pattern()` — orchestrator function

Calls 1a, 1b, 1c based on boolean flags and returns a single `PatternAnalysisResult`.

### Step 2: Refactor `unified_backfill.py` sequential runners

Replace the body of:
- [`run_images_action_runner()`](src/services/unified_backfill.py:310) → call `analyze_pattern(needs_images=True)`, write result fields to `design` ORM object, then do hoop selection
- [`run_color_counts_action_runner()`](src/services/unified_backfill.py:357) → call `analyze_pattern(needs_color_counts=True)`, write result fields to `design` ORM object
- [`run_stitching_action_runner()`](src/services/unified_backfill.py:238) → call `analyze_pattern(needs_stitching=True)`, apply stitching tags to `design.tags`

### Step 3: Refactor `unified_backfill.py` parallel worker

Replace lines 514-567 in [`_process_design_batch_worker()`](src/services/unified_backfill.py:416) with a single call to `analyze_pattern(pattern, needs_images=item.needs_images, needs_color_counts=item.needs_color_counts, needs_stitching=item.needs_stitching, ...)` and populate the result dict from the returned `PatternAnalysisResult`.

### Step 4: Refactor `preview.py:_process_file()`

Replace lines 368-407 with calls to `analyze_pattern()` for the parts that operate on a successfully-read pattern. The spider-image fallback and ART-specific logic should remain in `_process_file()` since they're specific to the scanning/import path.

### Step 5: Update imports

- `bulk_import.py` already re-exports from `preview.py` — no change needed if `_process_file` signature stays the same
- `backfill_images.py`, `backfill_color_counts.py`, `backfill_stitching_tags.py` — these delegate to `unified_backfill` for parallel mode, and have their own sequential fallback. The sequential fallbacks could optionally be updated later.

### Step 6: Remove now-unnecessary imports

Once the refactoring is complete, the sequential action runners in `unified_backfill.py` can be simplified. The parallel worker's inline logic is replaced entirely.

## What Does NOT Change

- **`suggest_stitching_from_pattern()`** in [`auto_tagging.py`](src/services/auto_tagging.py:380) — this is already a shared function, it's just called from 3 places. It stays as-is.
- **`_render_preview()`** in [`preview.py`](src/services/preview.py:304) — this is already a shared function. It stays as-is.
- **`_read_art_metadata()`** in [`preview.py`](src/services/preview.py:173) — ART-specific metadata extraction stays in preview.py.
- **`_find_spider_image()`** / **`_decode_art_icon()`** in [`preview.py`](src/services/preview.py:41,150) — these are import-scanning-specific fallbacks, not duplicated.
- **`select_hoop_for_dimensions()`** — this is a DB query, so it stays in each caller (it can't be in a pure computation function). However, the hoop selection logic is duplicated 3 times and could optionally be extracted into a shared helper that takes `(db, width_mm, height_mm)`.

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Breaking the parallel worker's result dict contract | The `PatternAnalysisResult` dataclass maps 1:1 to the existing result dict keys |
| Performance regression from extra function call overhead | The new function is called once per design (same as now) — no extra file reads |
| Missing an edge case in the extraction | Each extracted function has its source lines documented; tests can compare old vs new output |
| Import cycles | `pattern_analysis.py` imports from `preview.py` and `auto_tagging.py` — both are leaf modules with no back-edges to `unified_backfill.py` or `bulk_import.py` |

## Verification

After refactoring, run the existing test suite:
```bash
python -m pytest tests/ -v
```

Key test files:
- [`tests/test_bulk_import_extra.py`](tests/test_bulk_import_extra.py)
- [`tests/test_folder_picker_and_tagging.py`](tests/test_folder_picker_and_tagging.py)

Also run the standalone backfill scripts with `--dry-run` to verify they produce the same results:
```bash
python backfill_images.py --dry-run
python backfill_color_counts.py --dry-run
python backfill_stitching_tags.py --dry-run
```
