"""
Pattern analysis — pure computation functions for embroidery pattern data.

Extracts preview rendering, colour/stitch counts, and stitching-type detection
into a single shared module so that the same logic is not duplicated across:

* ``preview.py:_process_file()``        — import scanning
* ``unified_backfill.py`` sequential     — backfill sequential runners
* ``unified_backfill.py`` parallel       — backfill parallel worker

All functions in this module are **pure computation** — they take a
``pyembroidery.EmbPattern`` (and supporting data) and return plain dataclasses.
No database access, no ORM objects.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any

import pyembroidery

from src.services.preview import _render_preview

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Result type — pure data, no DB objects
# ---------------------------------------------------------------------------


@dataclass
class PatternAnalysisResult:
    """Computed pattern data returned by :func:`analyze_pattern`.

    Every field is ``None`` when the corresponding action was not requested
    or the data could not be extracted.
    """

    image_data: bytes | None = None
    image_type: str | None = None
    width_mm: float | None = None
    height_mm: float | None = None
    stitch_count: int | None = None
    color_count: int | None = None
    color_change_count: int | None = None
    stitching_tag_descriptions: list[str] | None = None


# ---------------------------------------------------------------------------
# Internal helpers (extracted from duplicated code)
# ---------------------------------------------------------------------------


def _render_preview_and_bounds(
    pattern: pyembroidery.EmbPattern,
    *,
    preview_3d: bool = True,
    redo: bool = False,
    upgrade_2d_to_3d: bool = False,
    existing_image_data: bytes | None = None,
    existing_image_type: str | None = None,
    existing_width_mm: float | None = None,
    existing_height_mm: float | None = None,
) -> tuple[bytes | None, str | None, float | None, float | None]:
    """Render a preview image and extract dimensions from pattern bounds.

    Returns ``(image_data, image_type, width_mm, height_mm)``.

    This consolidates logic that previously lived in both
    ``run_images_action_runner`` and ``_process_design_batch_worker``.
    """
    # Determine if we should render: redo, no existing image, or upgrade from 2D to 3D
    should_render = (
        redo
        or existing_image_data is None
        or (upgrade_2d_to_3d and existing_image_type in ("2d", None))
    )

    image_data: bytes | None = None
    image_type: str | None = None

    if should_render:
        image_data = _render_preview(pattern, preview_3d=preview_3d)
        image_type = "2d" if not preview_3d else "3d"

    # Extract bounds — only if not already known (or redo forces re-extraction)
    width_mm = existing_width_mm
    height_mm = existing_height_mm
    if redo or (width_mm is None):
        bounds = pattern.bounds()
        if bounds:
            min_x, min_y, max_x, max_y = bounds
            width_mm = round((max_x - min_x) / 10.0, 2)
            height_mm = round((max_y - min_y) / 10.0, 2)

    return image_data, image_type, width_mm, height_mm


def _extract_color_counts(
    pattern: pyembroidery.EmbPattern,
    *,
    existing_stitch_count: int | None = None,
    existing_color_count: int | None = None,
    existing_color_change_count: int | None = None,
) -> tuple[int | None, int | None, int | None]:
    """Extract stitch/colour/colour-change counts from a pattern.

    Only fills values that are currently ``None`` (thread/colour data never
    changes for a given file).

    Returns ``(stitch_count, color_count, color_change_count)``.
    """
    stitch_count = existing_stitch_count
    color_count = existing_color_count
    color_change_count = existing_color_change_count

    if stitch_count is None:
        try:
            stitch_count = pattern.count_stitches()
        except Exception:
            stitch_count = None

    if color_count is None:
        try:
            color_count = pattern.count_threads()
        except Exception:
            color_count = None

    if color_change_count is None:
        try:
            color_change_count = pattern.count_color_changes()
        except Exception:
            color_change_count = None

    return stitch_count, color_count, color_change_count


def _detect_stitching_tags(
    pattern: pyembroidery.EmbPattern,
    *,
    filename: str = "",
    filepath: str = "",
    pattern_path: str = "",
    desc_to_tag: dict[str, Any] | None = None,
    clear_existing_stitching: bool = False,
) -> list[str] | None:
    """Detect stitching-type tags from pattern geometry.

    Returns a sorted list of tag descriptions (e.g. ``["Filled", "Satin Stitch"]``)
    or ``None`` if no stitching detection was performed.

    When ``clear_existing_stitching`` is ``True`` and no stitch types are detected,
    returns an empty list (meaning "clear existing stitching tags").
    When ``clear_existing_stitching`` is ``False`` and nothing is detected,
    returns ``None`` (meaning "leave existing stitching tags alone").
    """
    from src.services.auto_tagging import suggest_stitching_from_pattern

    matched_descriptions = suggest_stitching_from_pattern(
        pattern_path=pattern_path,
        filename=filename,
        filepath=filepath,
        desc_to_tag=desc_to_tag or {},
        pattern=pattern,
    )

    if matched_descriptions:
        return sorted(matched_descriptions)

    if clear_existing_stitching:
        return []  # signal: clear existing stitching tags

    return None  # signal: leave existing tags alone


# ---------------------------------------------------------------------------
# Public API — single entry point
# ---------------------------------------------------------------------------


def analyze_pattern(
    pattern: pyembroidery.EmbPattern,
    *,
    needs_images: bool = False,
    needs_color_counts: bool = False,
    needs_stitching: bool = False,
    # Image options
    preview_3d: bool = True,
    redo: bool = False,
    upgrade_2d_to_3d: bool = False,
    existing_image_data: bytes | None = None,
    existing_image_type: str | None = None,
    existing_width_mm: float | None = None,
    existing_height_mm: float | None = None,
    # Colour-count options
    existing_stitch_count: int | None = None,
    existing_color_count: int | None = None,
    existing_color_change_count: int | None = None,
    # Stitching options
    filename: str = "",
    filepath: str = "",
    pattern_path: str = "",
    desc_to_tag: dict[str, Any] | None = None,
    clear_existing_stitching: bool = False,
) -> PatternAnalysisResult:
    """Analyse an embroidery pattern and return computed data.

    This is the single entry point for all pattern-analysis computation.
    Callers specify which analyses they need via the ``needs_*`` boolean flags.

    Args:
        pattern: A successfully-read ``pyembroidery.EmbPattern``.
        needs_images: Extract preview image and dimensions.
        needs_color_counts: Extract stitch/colour/colour-change counts.
        needs_stitching: Detect stitching-type tags from pattern geometry.
        preview_3d: Use 3D stitch simulation (slower, prettier).
        redo: Force re-rendering / re-extraction even if data exists.
        upgrade_2d_to_3d: Only re-render if existing image is 2D (or absent).
        existing_image_data: Previously-stored image bytes (to skip render).
        existing_image_type: Previously-stored image type (``"2d"`` or ``"3d"``).
        existing_width_mm: Previously-stored width.
        existing_height_mm: Previously-stored height.
        existing_stitch_count: Previously-stored stitch count.
        existing_color_count: Previously-stored colour count.
        existing_color_change_count: Previously-stored colour-change count.
        filename: Design filename (for name-based stitching detection).
        filepath: Design stored filepath (for folder-based stitching detection).
        pattern_path: Full filesystem path to the pattern file.
        desc_to_tag: Mapping of tag description → Tag (for stitching detection).
        clear_existing_stitching: If True and no stitch types detected, return
            empty list to signal "clear existing stitching tags".

    Returns:
        A :class:`PatternAnalysisResult` with fields populated according to
        which ``needs_*`` flags were set.
    """
    result = PatternAnalysisResult()

    # --- Images action ---
    if needs_images:
        img_data, img_type, w_mm, h_mm = _render_preview_and_bounds(
            pattern,
            preview_3d=preview_3d,
            redo=redo,
            upgrade_2d_to_3d=upgrade_2d_to_3d,
            existing_image_data=existing_image_data,
            existing_image_type=existing_image_type,
            existing_width_mm=existing_width_mm,
            existing_height_mm=existing_height_mm,
        )
        result.image_data = img_data
        result.image_type = img_type
        result.width_mm = w_mm
        result.height_mm = h_mm

    # --- Color counts action ---
    if needs_color_counts:
        sc, cc, ccc = _extract_color_counts(
            pattern,
            existing_stitch_count=existing_stitch_count,
            existing_color_count=existing_color_count,
            existing_color_change_count=existing_color_change_count,
        )
        result.stitch_count = sc
        result.color_count = cc
        result.color_change_count = ccc

    # --- Stitching action ---
    if needs_stitching:
        st_descriptions = _detect_stitching_tags(
            pattern,
            filename=filename,
            filepath=filepath,
            pattern_path=pattern_path,
            desc_to_tag=desc_to_tag,
            clear_existing_stitching=clear_existing_stitching,
        )
        result.stitching_tag_descriptions = st_descriptions

    return result
