"""
Unified backfill service for batch processing tagging, stitching, image, and color count actions.

Supports parallel processing via ``multiprocessing`` for CPU-bound operations
(preview rendering, stitch analysis) and sequential mode for API-bound operations
(tagging with Gemini).
"""

from __future__ import annotations

import logging
import multiprocessing
import os as _os
import traceback
from concurrent.futures import CancelledError, ProcessPoolExecutor, as_completed
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

from sqlalchemy import false as sql_false
from sqlalchemy import text as sql_text
from sqlalchemy.orm import Session

from src.config import DESIGNS_BASE_PATH
from src.models import Design, Tag, design_tags
from src.services.auto_tagging import (
    run_stitching_backfill_action,
    run_tagging_action,
)
from src.services.hoops import select_hoop_for_dimensions
from src.services.pattern_analysis import analyze_pattern
from src.services.tagging import _unique_tags_from_descriptions

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Per-design action metadata — tells workers what work to do for each design
# ---------------------------------------------------------------------------


@dataclass
class DesignWorkItem:
    """Metadata for a single design, indicating which actions are needed.

    Workers read the embroidery file ONCE and check these booleans to decide
    what work to perform.  The main process builds this list by merging per-action
    database queries.
    """

    id: int
    filename: str
    filepath: str

    # Action flags — set by the merge logic in unified_backfill()
    needs_images: bool = False
    needs_color_counts: bool = False
    needs_stitching: bool = False

    # Pre-loaded values from the database (used to decide if work is needed
    # and to carry forward existing data that doesn't need re-processing).
    image_data: bytes | None = None
    image_type: str | None = None
    width_mm: float | None = None
    height_mm: float | None = None
    hoop_id: int | None = None
    stitch_count: int | None = None
    color_count: int | None = None
    color_change_count: int | None = None


ERROR_LOG_PATH = Path("logs/backfill_errors.log")
INFO_LOG_PATH = Path("logs/backfill_info.log")

# In-memory stop flag — set to True to request the running backfill to stop.
# Using an in-memory flag rather than a file-based signal avoids filesystem
# timing issues and is immediately visible to all running threads.
_backfill_stop_requested: bool = False

# Multiprocessing stop event — shared across worker processes.
# Initialised when a parallel backfill starts; checked by workers.
_backfill_stop_event: multiprocessing.synchronize.Event | None = None


def request_stop() -> None:
    """Signal the running backfill to stop. Outstanding changes will be committed."""
    global _backfill_stop_requested
    _backfill_stop_requested = True
    # Also signal any parallel workers
    event = _backfill_stop_event
    if event is not None:
        event.set()
    # Create the stop sentinel file so worker processes (which cannot see
    # the in-memory flag/event) detect the stop signal promptly.
    _signal_stop_sentinel()
    logger.info("Stop signal requested — backfill will stop at next checkpoint.")


def clear_stop_signal() -> None:
    """Clear the stop flag and remove the sentinel file (called at the start of a backfill run)."""
    global _backfill_stop_requested, _backfill_stop_event
    _backfill_stop_requested = False
    _backfill_stop_event = None
    _clear_stop_sentinel()


def is_stop_requested() -> bool:
    """Return True if a stop has been requested."""
    if _backfill_stop_requested:
        return True
    event = _backfill_stop_event
    if event is not None and event.is_set():
        return True
    return False


# --- Logging helpers ---


def log_error(
    filename: str,
    stage: str,
    message: str,
    action: str = "",
    **extra: str,
) -> None:
    """Append a tab-separated error entry to the persistent error log.

    The log line has at least six tab-separated columns:

        timestamp  filename  action  stage  exc_type  message

    Callers pass ``stage`` as the second positional argument (e.g.
    ``log_error(fn, "stitching", msg, exc_type="…")``).  Any additional
    keyword arguments are appended as ``key=value`` pairs after the
    standard columns.
    """
    try:
        ERROR_LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
        exc_type = extra.pop("exc_type", "")
        extra_str = " ".join(f"{k}={v}" for k, v in extra.items())
        with open(str(ERROR_LOG_PATH), "a", encoding="utf-8") as fh:
            fh.write(
                f"{datetime.now():%Y-%m-%d %H:%M:%S}"
                f"\t{filename}"
                f"\t{action}"
                f"\t{stage}"
                f"\t{exc_type}"
                f"\t{message}"
            )
            if extra_str:
                fh.write(f"\t({extra_str})")
            fh.write("\n")
    except OSError:
        pass


def _resolve_design_filepath(design_filepath: str) -> str | None:
    """Resolve a design's stored filepath to an absolute filesystem path."""
    rel = design_filepath.lstrip("/\\")
    full = _os.path.normpath(_os.path.join(DESIGNS_BASE_PATH, rel))
    if _os.path.isfile(full):
        return full
    logger.info(
        "Pattern file not found at %r (stored filepath: %r, DESIGNS_BASE_PATH=%r)",
        full,
        design_filepath,
        DESIGNS_BASE_PATH,
    )
    return None


def log_info(filename: str, stage: str, message: str, **extra: str) -> None:
    """Append an info entry to the persistent info log.

    Callers pass ``stage`` as the second positional argument (e.g.
    ``log_info(fn, "tagging", msg, stage="run_tagging_action")``).
    """
    try:
        INFO_LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
        extra_str = " ".join(f"{k}={v}" for k, v in extra.items())
        with open(str(INFO_LOG_PATH), "a", encoding="utf-8") as fh:
            fh.write(f"[{datetime.now():%Y-%m-%d %H:%M:%S}] {filename} | {stage} | {message}")
            if extra_str:
                fh.write(f" ({extra_str})")
            fh.write("\n")
    except OSError:
        pass


# --- Modularized action runners ---
def run_tagging_action_runner(
    db,
    design,
    tag_opts,
    api_key,
    delay,
    vision_delay,
    batch_size: int = 1,
    action: str = "tagging",
    design_ids: list[int] | None = None,
):
    """Run the tagging backfill for a single design.

    ``batch_size`` is passed through to :func:`run_tagging_action` so that the
    inner query respects the user's configured batch size.

    ``design_ids`` is an optional pre-filtered list of design IDs to process.
    When provided, :func:`run_tagging_action` skips its own design selection
    query and uses these IDs directly — avoiding a redundant database query
    for every design in the unified backfill loop.
    """
    try:
        run_tagging_action(
            db=db,
            action=tag_opts.get("action", "tag_untagged"),
            tiers=tag_opts.get("tiers", [1]),
            api_key=api_key,
            batch_size=batch_size,
            delay=delay,
            vision_delay=vision_delay,
            overwrite_verified=tag_opts.get("overwrite_verified", False),
            dry_run=False,
            design_ids=design_ids,
        )
        log_info(design.filename, "run_tagging_action", "Tagging action completed")
        return None
    except Exception as e:
        exc_type = type(e).__name__
        tb = traceback.format_exc()
        log_error(design.filename, "run_tagging_action", f"{e}", action=action, exc_type=exc_type)
        log_info(design.filename, "run_tagging_action", f"Exception: {exc_type}\n{tb}")
        return f"{exc_type}: {e}"


def run_stitching_action_runner(db, design, st_opts, batch_size: int = 1, pattern=None):
    """Run the stitching backfill for a single design.

    When called from the unified loop (``pattern`` is provided), the
    embroidery file has already been read — use ``analyze_pattern``
    with the pre-read pattern to avoid a redundant file read.

    When called standalone (``pattern`` is ``None``), delegates to
    :func:`run_stitching_backfill_action` which has its own query-and-read loop.
    """
    logger.info("Starting stitch analysis for design %d (%s)", design.id, design.filename)
    # Check stop signal before potentially expensive stitch analysis
    if is_stop_requested():
        return None
    try:
        if pattern is not None:
            # Pre-read pattern available — apply stitching detection directly.
            from src.models import Tag

            all_tags: list[Tag] = db.query(Tag).order_by(Tag.description).all()
            desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in all_tags}

            pattern_path = _resolve_design_filepath(design.filepath)

            result = analyze_pattern(
                pattern,
                needs_stitching=True,
                filename=design.filename,
                filepath=design.filepath,
                pattern_path=pattern_path or "",
                desc_to_tag=desc_to_tag,
                clear_existing_stitching=st_opts.get("clear_existing_stitching", False),
            )

            # Check stop signal after stitch analysis (may have been slow)
            if is_stop_requested():
                return None

            st_descriptions = result.stitching_tag_descriptions
            if st_descriptions is not None:
                non_stitching_tags = [
                    tag for tag in design.tags if getattr(tag, "tag_group", None) != "stitching"
                ]
                if st_descriptions:
                    matched_tags = _unique_tags_from_descriptions(st_descriptions, desc_to_tag)
                    replacement_tags = non_stitching_tags + [
                        tag for tag in matched_tags if tag not in non_stitching_tags
                    ]
                else:
                    replacement_tags = non_stitching_tags
                design.tags = replacement_tags
                design.tags_checked = False
                design.tagging_tier = 1 if st_descriptions else None
        else:
            # No pre-read pattern — delegate to the standalone batch runner.
            # Check stop signal before delegating to the standalone runner
            if is_stop_requested():
                return None
            run_stitching_backfill_action(
                db=db,
                batch_size=batch_size,
                dry_run=False,
                clear_existing_stitching=st_opts.get("clear_existing_stitching", False),
            )
        return None
    except Exception as e:
        return str(e)


def run_images_action_runner(db, design, img_opts, pattern):
    if pattern is None:
        logger.debug(
            "run_images_action_runner: pattern is None for design %d — skipping", design.id
        )
        return None
    logger.info("Starting image render for design %d (%s)", design.id, design.filename)
    # Check stop signal before potentially expensive preview render
    if is_stop_requested():
        return None
    try:
        result = analyze_pattern(
            pattern,
            needs_images=True,
            preview_3d=img_opts.get("preview_3d", True),
            redo=img_opts.get("redo", False),
            upgrade_2d_to_3d=img_opts.get("upgrade_2d_to_3d", False),
            existing_image_data=design.image_data,
            existing_image_type=design.image_type,
            existing_width_mm=design.width_mm,
            existing_height_mm=design.height_mm,
        )

        # Check stop signal after preview render (may have been slow)
        if is_stop_requested():
            return None

        # Write results to the ORM object
        if result.image_data is not None:
            design.image_data = result.image_data
        if result.image_type is not None:
            design.image_type = result.image_type
        if result.width_mm is not None:
            design.width_mm = result.width_mm
            design.height_mm = result.height_mm

        # Hoop auto-lookup (requires DB — stays here)
        if design.hoop_id is None and design.width_mm and design.height_mm:
            hoop = select_hoop_for_dimensions(db, design.width_mm, design.height_mm)
            if hoop:
                design.hoop_id = hoop.id
        return None
    except Exception as e:
        logger.error(
            "run_images_action_runner failed for design %d (%s): %s",
            design.id,
            design.filename,
            e,
        )
        return str(e)


def run_color_counts_action_runner(db, design, cc_opts, pattern):
    if pattern is None:
        # Pattern could not be read (e.g. .ART thumbnails) — skip silently.
        return None
    logger.info("Starting colour count for design %d (%s)", design.id, design.filename)
    if is_stop_requested():
        return None
    try:
        result = analyze_pattern(
            pattern,
            needs_color_counts=True,
            existing_stitch_count=design.stitch_count,
            existing_color_count=design.color_count,
            existing_color_change_count=design.color_change_count,
        )

        if is_stop_requested():
            return None

        # Write results to the ORM object
        if result.stitch_count is not None:
            design.stitch_count = result.stitch_count
        if result.color_count is not None:
            design.color_count = result.color_count
        if result.color_change_count is not None:
            design.color_change_count = result.color_change_count
        return None
    except Exception as e:
        return str(e)


# ---------------------------------------------------------------------------
# Parallel worker — runs in a subprocess
# ---------------------------------------------------------------------------
# This module-level function is pickled and sent to worker processes.  It
# opens its own database connection and returns serializable result dicts.

# Sentinel file path used to signal stop to worker processes.
# Workers check for the existence of this file between designs.
_STOP_SENTINEL_PATH = str(Path("logs/_backfill_stop.sentinel"))


def _clear_stop_sentinel() -> None:
    """Remove the stop sentinel file if it exists."""
    try:
        p = Path(_STOP_SENTINEL_PATH)
        if p.exists():
            p.unlink()
    except OSError:
        pass


def _signal_stop_sentinel() -> None:
    """Create the stop sentinel file to signal workers to stop."""
    try:
        Path(_STOP_SENTINEL_PATH).parent.mkdir(parents=True, exist_ok=True)
        Path(_STOP_SENTINEL_PATH).touch()
    except OSError:
        pass


def _stop_sentinel_exists() -> bool:
    """Return True if the stop sentinel file exists."""
    return _os.path.exists(_STOP_SENTINEL_PATH)


def _process_design_batch_worker(
    design_items: list[DesignWorkItem],
    actions: dict[str, dict[str, Any]],
    db_url: str,
    designs_base_path: str,
) -> list[dict]:
    """Process a batch of designs in a worker process.

    Workers do the CPU-bound work (reading files, rendering previews,
    analysing stitches) and return result dicts.  They do NOT write to
    the database — the main process collects results and writes them
    sequentially, avoiding SQLite locking contention.

    Each ``DesignWorkItem`` carries per-design action metadata (``needs_images``,
    ``needs_color_counts``, ``needs_stitching``) so the worker only does the
    work required for each design, reading the embroidery file only once.

    Returns a list of result dicts — one per design — that the main process
    can use for final accounting and DB writes.

    This function is module-level so it can be pickled by ``multiprocessing``.
    """
    import logging as _logging

    _logging.basicConfig(level=_logging.INFO)
    _log = _logging.getLogger(__name__ + ".worker")

    # Open a read-only DB session to pre-load tag data for stitching detection.
    # Workers do NOT write to the DB — they return results for the main process.
    from sqlalchemy import create_engine
    from sqlalchemy.orm import sessionmaker

    engine = create_engine(db_url, connect_args={"check_same_thread": False})
    SessionLocal = sessionmaker(bind=engine)
    db = SessionLocal()

    # Pre-load tag data for stitching detection
    from src.models import Tag as _Tag

    all_tags: list[_Tag] = db.query(_Tag).order_by(_Tag.description).all()
    desc_to_tag: dict[str, _Tag] = {tag.description: tag for tag in all_tags}

    results: list[dict] = []
    _total = len(design_items)
    _log.info("Worker started: %d designs to process", _total)
    _idx = 0

    try:
        for _idx, item in enumerate(design_items, 1):
            # Check stop sentinel
            if _stop_sentinel_exists():
                _log.info("Worker detected stop sentinel — exiting.")
                break

            # Periodic progress log
            if _idx % 100 == 0 or _idx == _total or _idx == 1:
                _log.info("Progress: %d/%d designs processed", _idx, _total)

            result: dict = {
                "design_id": item.id,
                "filename": item.filename,
                "error": None,
                "image_data": None,
                "image_type": None,
                "width_mm": item.width_mm,
                "height_mm": item.height_mm,
                "hoop_id": item.hoop_id,
                "stitch_count": item.stitch_count,
                "color_count": item.color_count,
                "color_change_count": item.color_change_count,
                "stitching_tag_descriptions": None,
            }

            try:
                # Only read the file if any action needs it
                pattern = None
                if item.needs_images or item.needs_color_counts or item.needs_stitching:
                    rel = item.filepath.replace("/", "\\")
                    full_path = designs_base_path.rstrip("/\\") + rel
                    _ext = _os.path.splitext(rel)[1].lstrip(".").lower()
                    try:
                        import pyembroidery

                        _supported = {
                            e.lower()
                            for f in pyembroidery.supported_formats()
                            if f.get("reader") is not None
                            for e in f.get("extensions", (f.get("extension"),))
                        }
                    except Exception:
                        _supported = set()

                    if _ext in _supported:
                        try:
                            pattern = pyembroidery.read(full_path)
                        except Exception:
                            pattern = None

                # Use the shared analyze_pattern function for all computation
                if pattern is not None:
                    img_opts = actions.get("images", {})
                    st_opts = actions.get("stitching", {})
                    pattern_path = _resolve_design_filepath(item.filepath) or ""

                    analysis = analyze_pattern(
                        pattern,
                        needs_images=item.needs_images,
                        needs_color_counts=item.needs_color_counts,
                        needs_stitching=item.needs_stitching,
                        # Image options
                        preview_3d=img_opts.get("preview_3d", True),
                        redo=img_opts.get("redo", False),
                        upgrade_2d_to_3d=img_opts.get("upgrade_2d_to_3d", False),
                        existing_image_data=item.image_data,
                        existing_image_type=item.image_type,
                        existing_width_mm=item.width_mm,
                        existing_height_mm=item.height_mm,
                        # Colour-count options
                        existing_stitch_count=item.stitch_count,
                        existing_color_count=item.color_count,
                        existing_color_change_count=item.color_change_count,
                        # Stitching options
                        filename=item.filename,
                        filepath=item.filepath,
                        pattern_path=pattern_path,
                        desc_to_tag=desc_to_tag,
                        clear_existing_stitching=st_opts.get("clear_existing_stitching", False),
                    )

                    # Map analysis result back to the result dict
                    if item.needs_images:
                        result["image_data"] = analysis.image_data
                        result["image_type"] = analysis.image_type
                        result["width_mm"] = analysis.width_mm
                        result["height_mm"] = analysis.height_mm
                        if analysis.image_data is not None:
                            _log.info(
                                "Rendered preview for %s: %d bytes",
                                item.filename,
                                len(analysis.image_data),
                            )
                        else:
                            _log.warning("_render_preview returned None for %s", item.filename)

                    if item.needs_color_counts:
                        result["stitch_count"] = analysis.stitch_count
                        result["color_count"] = analysis.color_count
                        result["color_change_count"] = analysis.color_change_count

                    if item.needs_stitching:
                        result["stitching_tag_descriptions"] = analysis.stitching_tag_descriptions

            except Exception as exc:
                _log.exception("Worker error processing %s", item.filename)
                result["error"] = f"{type(exc).__name__}: {exc}"

            results.append(result)

    finally:
        db.close()
        engine.dispose()

    return results


# ---------------------------------------------------------------------------
# SQLite bulk-write optimisation helpers
# ---------------------------------------------------------------------------


def _optimise_sqlite_for_bulk(db: Session) -> None:
    """Temporarily tune SQLite pragmas for faster bulk writes.

    Call before a large backfill operation and restore with
    :func:`_restore_sqlite_after_bulk` afterwards.

    Each PRAGMA is executed as a separate statement because SQLite's
    ``execute()`` only accepts one statement at a time.  Individual
    pragma failures are logged but do not abort the whole batch —
    the backfill still proceeds with whatever settings are available.

    Note: ``journal_mode`` is intentionally left at WAL (set by the
    application on connect) because WAL allows concurrent reads from
    worker processes while the main process writes — critical for the
    parallel backfill path.  Changing it to MEMORY would require an
    exclusive lock and would block workers from reading.
    """
    pragmas = [
        ("synchronous", "OFF"),
        ("cache_size", "-80000"),
        ("temp_store", "MEMORY"),
    ]
    ok_count = 0
    for name, value in pragmas:
        try:
            db.execute(sql_text(f"PRAGMA {name} = {value}"))
            ok_count += 1
        except Exception as exc:
            logger.warning("Could not set PRAGMA %s = %s: %s", name, value, exc)
    if ok_count == len(pragmas):
        logger.info("SQLite tuned for bulk write performance.")
    elif ok_count > 0:
        logger.info("SQLite partially tuned (%d/%d pragmas set).", ok_count, len(pragmas))


def _restore_sqlite_after_bulk(db: Session) -> None:
    """Restore normal SQLite pragmas after a bulk operation."""
    pragmas = [
        ("synchronous", "FULL"),
        ("cache_size", "-2000"),
        ("temp_store", "DEFAULT"),
        ("journal_mode", "WAL"),
    ]
    ok_count = 0
    for name, value in pragmas:
        try:
            db.execute(sql_text(f"PRAGMA {name} = {value}"))
            ok_count += 1
        except Exception as exc:
            logger.warning("Could not restore PRAGMA %s = %s: %s", name, value, exc)
    if ok_count == len(pragmas):
        logger.info("SQLite pragmas restored to normal.")
    elif ok_count > 0:
        logger.info("SQLite pragmas partially restored (%d/%d).", ok_count, len(pragmas))


# ---------------------------------------------------------------------------
# Main entry point
# ---------------------------------------------------------------------------


def unified_backfill(
    db: Session,
    actions: dict[str, dict[str, Any]],
    batch_size: int = 100,
    commit_every: int = 500,
    api_key: str = "",
    delay: float = 5.0,
    vision_delay: float = 2.0,
    workers: int = 1,
) -> dict:
    """
    Run selected backfill actions in a single batch, processing each design once.

    Parameters
    ----------
    db : Session
        SQLAlchemy database session.
    actions : dict
        Dict of action_name -> options dict.
        Supported actions: ``'tagging'``, ``'stitching'``, ``'images'``, ``'color_counts'``.
    batch_size : int
        Number of designs to fetch per database query for Gemini tagging (default 100).
        Only relevant when the ``'tagging'`` action is included.  Images, stitching,
        and color-counts paths ignore this parameter — they use ``commit_every``
        as the chunk size.
    commit_every : int
        Commit to the database after this many designs (default 500).
    api_key : str
        Google AI API key (required for tagging tiers 2 and 3).
    delay : float
        Seconds between Gemini text API calls (tier 2).
    vision_delay : float
        Seconds between Gemini vision API calls (tier 3).
    workers : int
        Number of parallel worker processes.  ``1`` (default) runs sequentially
        in-process.  Values > 1 use ``multiprocessing`` to process designs in
        parallel.  Tagging actions (Gemini API calls) always run sequentially
        in the main process regardless of this setting.

    Returns
    -------
    dict
        Summary with keys ``processed``, ``errors``, ``stopped``, ``actions``.
    """
    # -----------------------------------------------------------------------
    # Design selection — per-action queries merged into DesignWorkItem list
    # -----------------------------------------------------------------------
    # Each action type queries the database independently to find designs that
    # need that action.  Results are merged into a single deduplicated list of
    # DesignWorkItem objects, each carrying per-design action metadata.
    # Workers use this metadata to decide what work to do for each design,
    # reading the embroidery file only once.

    design_map: dict[int, DesignWorkItem] = {}
    tagging_design_ids: list[int] | None = None

    if "tagging" in actions:
        tag_opts = actions["tagging"]
        action = tag_opts.get("action", "tag_untagged")
        q = db.query(Design)
        if action == "tag_untagged":
            # Only designs that have no tags in the "image" tag group
            # (they may have "stitching" tags — those are unrelated to AI keyword tagging)
            image_tag_ids = [t.id for t in db.query(Tag).filter(Tag.tag_group == "image").all()]
            if image_tag_ids:
                q = q.filter(~Design.tags.any(Tag.id.in_(image_tag_ids)))
            else:
                # No image tags exist — no designs match
                q = q.filter(sql_false())
        elif action == "retag_all":
            # All designs — overwrite everything, including verified
            pass
        elif action == "retag_all_unverified":
            # All unverified designs — overwrite their tags
            q = q.filter(Design.tags_checked.is_(False) | Design.tags_checked.is_(None))
        tagging_ids = [d.id for d in q.all()]
        tagging_design_ids = tagging_ids if tagging_ids else None
        for d_id in tagging_ids:
            if d_id not in design_map:
                design_map[d_id] = DesignWorkItem(id=d_id, filename="", filepath="")

    if "stitching" in actions:
        # Designs that do NOT have any stitching tag
        stitching_subq = (
            db.query(design_tags.c.design_id)
            .join(Tag, Tag.id == design_tags.c.tag_id)
            .filter(Tag.tag_group == "stitching")
        ).subquery()
        q = db.query(Design).filter(~Design.id.in_(db.query(stitching_subq.c.design_id)))
        for d in q.all():
            if d.id not in design_map:
                design_map[d.id] = DesignWorkItem(id=d.id, filename=d.filename, filepath=d.filepath)
            design_map[d.id].needs_stitching = True

    if "images" in actions:
        img_opts = actions["images"]
        redo = img_opts.get("redo", False)
        upgrade_2d_to_3d = img_opts.get("upgrade_2d_to_3d", False)
        q = db.query(Design)
        if redo:
            # Re-process all designs regardless of existing image
            pass
        elif upgrade_2d_to_3d:
            # Only designs that have a 2D image (or legacy image_data without image_type)
            q = q.filter(
                Design.image_data.isnot(None),
                (Design.image_type == "2d") | (Design.image_type.is_(None)),
            )
        else:
            # Default: only designs with no image at all
            q = q.filter(Design.image_data.is_(None))
        for d in q.all():
            if d.id not in design_map:
                design_map[d.id] = DesignWorkItem(id=d.id, filename=d.filename, filepath=d.filepath)
            design_map[d.id].needs_images = True
            design_map[d.id].image_data = d.image_data
            design_map[d.id].image_type = d.image_type
            design_map[d.id].width_mm = d.width_mm
            design_map[d.id].height_mm = d.height_mm
            design_map[d.id].hoop_id = d.hoop_id

    if "color_counts" in actions:
        # Thread/colour data never changes — only add to designs where it's missing.
        q = db.query(Design).filter(
            (Design.stitch_count.is_(None))
            | (Design.color_count.is_(None))
            | (Design.color_change_count.is_(None))
        )
        for d in q.all():
            if d.id not in design_map:
                design_map[d.id] = DesignWorkItem(id=d.id, filename=d.filename, filepath=d.filepath)
            design_map[d.id].needs_color_counts = True
            design_map[d.id].stitch_count = d.stitch_count
            design_map[d.id].color_count = d.color_count
            design_map[d.id].color_change_count = d.color_change_count

    # Convert the merged map to a sorted list for deterministic processing order
    design_items: list[DesignWorkItem] = sorted(design_map.values(), key=lambda x: x.id)

    def chunked(seq, size):
        items = list(seq)
        for i in range(0, len(items), size):
            yield items[i : i + size]

    results = {"processed": 0, "errors": 0, "stopped": False, "actions": list(actions.keys())}
    count_since_commit = 0
    # Clear logs and stop flag at the start of the run
    for log_path in (ERROR_LOG_PATH, INFO_LOG_PATH):
        if log_path.exists():
            log_path.unlink()
    clear_stop_signal()

    # Bulk clear: if "clear existing stitching tags for unverified designs first"
    # is checked, remove all stitching tags from unverified designs before processing.
    if "stitching" in actions and actions["stitching"].get("clear_existing_stitching", False):
        from sqlalchemy import exists, select

        stitching_tag_ids = [
            tag.id for tag in db.query(Tag).all() if getattr(tag, "tag_group", None) == "stitching"
        ]
        if stitching_tag_ids:
            # Use a subquery-based DELETE to avoid SQLite's 999-variable limit.
            subq_tags = select(Tag.id).where(
                Tag.id == design_tags.c.tag_id, Tag.id.in_(stitching_tag_ids)
            )
            subq_designs = select(Design.id).where(
                Design.id == design_tags.c.design_id,
                Design.tags_checked.is_(False) | Design.tags_checked.is_(None),
            )
            deleted = (
                db.query(design_tags)
                .filter(exists(subq_tags), exists(subq_designs))
                .delete(synchronize_session="fetch")
            )
            logger.info(
                "Bulk clear: removed %d stitching tag assignments from unverified designs",
                deleted,
            )
            # Also reset tagging_tier on unverified designs
            db.query(Design).filter(
                Design.tags_checked.is_(False) | Design.tags_checked.is_(None)
            ).update(
                {Design.tagging_tier: None},
                synchronize_session="fetch",
            )
            db.commit()
            logger.info("Bulk clear committed — proceeding with detection loop.")

    # Determine which actions can run in parallel workers.
    # Tagging (Gemini API calls) is I/O-bound and rate-limited — always sequential.
    # Stitching, images, and color_counts are CPU-bound — can be parallelised.
    parallel_actions = {k: v for k, v in actions.items() if k != "tagging"}
    has_parallel_work = bool(parallel_actions) and workers > 1 and len(design_items) > 0
    has_tagging = "tagging" in actions

    # Optimise SQLite for bulk writes
    _optimise_sqlite_for_bulk(db)

    try:
        if has_parallel_work:
            # --- PARALLEL PATH ---
            logger.info(
                "Starting parallel backfill with %d workers for actions: %s",
                workers,
                list(parallel_actions.keys()),
            )

            # Initialise the stop event for workers
            global _backfill_stop_event
            _backfill_stop_event = multiprocessing.Event()

            # Get the database URL for worker connections
            from src.config import DATABASE_URL

            db_url = DATABASE_URL

            # Use small chunks so results stream back to the main process
            # frequently, allowing progressive commits at the user-requested
            # commit_every interval.  Each chunk is processed by one worker;
            # multiple chunks run in parallel via ProcessPoolExecutor.
            # chunk_size = commit_every ensures each chunk completes quickly
            # and the main process can write+commit results progressively.
            chunk_size = max(1, commit_every)
            chunks = [
                design_items[i : i + chunk_size] for i in range(0, len(design_items), chunk_size)
            ]
            logger.info(
                "Split %d designs into %d chunks (chunk_size=%d, workers=%d)",
                len(design_items),
                len(chunks),
                chunk_size,
                workers,
            )

            worker_errors = 0
            # Track all processed design IDs for the tagging phase
            all_processed_ids: list[int] = []

            # Pre-load tag lookup for stitching tag assignment in the main process
            if "stitching" in parallel_actions:
                all_tags: list[Tag] = db.query(Tag).order_by(Tag.description).all()
                desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in all_tags}
            else:
                desc_to_tag = {}

            with ProcessPoolExecutor(max_workers=workers) as executor:
                futures = {
                    executor.submit(
                        _process_design_batch_worker,
                        chunk,
                        parallel_actions,
                        db_url,
                        DESIGNS_BASE_PATH,
                    ): idx
                    for idx, chunk in enumerate(chunks)
                }

                for future in as_completed(futures):
                    chunk_idx = futures[future]
                    try:
                        batch_results = future.result()
                    except CancelledError:
                        # Chunk was cancelled due to stop signal — skip silently.
                        continue
                    except Exception as exc:
                        logger.error("Chunk %d failed: %s", chunk_idx, exc)
                        worker_errors += 1
                        continue

                    logger.info(
                        "Chunk %d/%d completed (%d designs)",
                        chunk_idx + 1,
                        len(chunks),
                        len(batch_results),
                    )

                    # Workers return results — write them to the DB sequentially
                    # (single writer avoids SQLite locking contention).
                    for r in batch_results:
                        all_processed_ids.append(r["design_id"])
                        if r.get("error"):
                            log_error(
                                r["filename"],
                                "worker",
                                r["error"],
                                action=",".join(parallel_actions.keys()),
                            )
                            results["errors"] += 1
                            results["processed"] += 1
                            continue

                        # Write this result to the DB
                        design_obj = db.query(Design).filter(Design.id == r["design_id"]).first()
                        if design_obj is None:
                            logger.warning("Design %d not found in DB — skipping", r["design_id"])
                            results["processed"] += 1
                            continue

                        try:
                            if "images" in parallel_actions:
                                if r.get("image_data") is not None:
                                    design_obj.image_data = r["image_data"]
                                if r.get("image_type") is not None:
                                    design_obj.image_type = r["image_type"]
                                if r.get("width_mm") is not None:
                                    design_obj.width_mm = r["width_mm"]
                                    design_obj.height_mm = r["height_mm"]
                                # Hoop auto-lookup
                                if (
                                    design_obj.hoop_id is None
                                    and design_obj.width_mm
                                    and design_obj.height_mm
                                ):
                                    _hoop = select_hoop_for_dimensions(
                                        db, design_obj.width_mm, design_obj.height_mm
                                    )
                                    if _hoop is not None:
                                        design_obj.hoop_id = _hoop.id

                            if "color_counts" in parallel_actions:
                                if r.get("stitch_count") is not None:
                                    design_obj.stitch_count = r["stitch_count"]
                                if r.get("color_count") is not None:
                                    design_obj.color_count = r["color_count"]
                                if r.get("color_change_count") is not None:
                                    design_obj.color_change_count = r["color_change_count"]

                            if "stitching" in parallel_actions:
                                st_descriptions = r.get("stitching_tag_descriptions")
                                if st_descriptions is not None:
                                    non_stitching_tags = [
                                        tag
                                        for tag in design_obj.tags
                                        if getattr(tag, "tag_group", None) != "stitching"
                                    ]
                                    if st_descriptions:
                                        matched_tags = _unique_tags_from_descriptions(
                                            st_descriptions, desc_to_tag
                                        )
                                        replacement_tags = non_stitching_tags + [
                                            tag
                                            for tag in matched_tags
                                            if tag not in non_stitching_tags
                                        ]
                                    else:
                                        replacement_tags = non_stitching_tags
                                    design_obj.tags = replacement_tags
                                    design_obj.tags_checked = False
                                    design_obj.tagging_tier = 1 if st_descriptions else None

                        except Exception as write_exc:
                            logger.error(
                                "Error writing result for design %d: %s",
                                r["design_id"],
                                write_exc,
                            )
                            results["errors"] += 1

                        results["processed"] += 1
                        count_since_commit += 1

                        # Commit at the requested interval within this chunk
                        if count_since_commit >= commit_every:
                            try:
                                db.commit()
                                logger.info(
                                    "Committed batch of %d designs (chunk %d/%d)",
                                    commit_every,
                                    chunk_idx + 1,
                                    len(chunks),
                                )
                                count_since_commit = 0
                            except Exception as commit_exc:
                                logger.error(
                                    "Commit failed at batch %d (chunk %d/%d): %s",
                                    commit_every,
                                    chunk_idx + 1,
                                    len(chunks),
                                    commit_exc,
                                )
                                results["errors"] += 1

                    # Final commit for this chunk (any remaining designs)
                    if count_since_commit > 0:
                        try:
                            db.commit()
                            logger.info(
                                "Committed final batch of %d designs (chunk %d/%d)",
                                count_since_commit,
                                chunk_idx + 1,
                                len(chunks),
                            )
                            count_since_commit = 0
                        except Exception as commit_exc:
                            logger.error(
                                "Final commit failed for chunk %d/%d: %s",
                                chunk_idx + 1,
                                len(chunks),
                                commit_exc,
                            )
                            results["errors"] += 1

                    if is_stop_requested():
                        logger.info("Stop signal detected — shutting down workers.")
                        results["stopped"] = True
                        # Shut down the executor without waiting for pending
                        # futures, so the main process can proceed to write
                        # any remaining results and exit cleanly.  Workers
                        # will detect the sentinel file and stop promptly.
                        # cancel_futures=True ensures queued (not-yet-started)
                        # chunks are cancelled immediately so as_completed
                        # does not block waiting for them indefinitely.
                        executor.shutdown(wait=False, cancel_futures=True)
                        break

            results["errors"] += worker_errors
            processed_ids = all_processed_ids
            logger.info(
                "Parallel phase complete: %d processed, %d errors",
                results["processed"],
                results["errors"],
            )

            # Run tagging (always sequential, after parallel work)
            if has_tagging:
                logger.info("Starting sequential tagging phase...")
                tag_opts = actions["tagging"]
                # Re-query designs that still need tagging
                # (parallel work may have changed tags_checked state)
                _TAG_FETCH_BATCH = min(batch_size if batch_size > 0 else 100, 500)
                for id_batch in chunked(processed_ids, _TAG_FETCH_BATCH):
                    designs = (
                        db.query(Design).filter(Design.id.in_(id_batch)).order_by(Design.id).all()
                    )
                    for design in designs:
                        if is_stop_requested():
                            logger.info("Stop signal detected — committing and exiting.")
                            results["stopped"] = True
                            break

                        err = run_tagging_action_runner(
                            db,
                            design,
                            tag_opts,
                            api_key,
                            delay,
                            vision_delay,
                            batch_size=batch_size,
                            action="tagging",
                            design_ids=tagging_design_ids,
                        )
                        if err:
                            results["errors"] += 1
                        results["processed"] += 1
                        count_since_commit += 1

                        if count_since_commit >= commit_every:
                            logger.info("Committing batch of %d designs", commit_every)
                            db.commit()
                            count_since_commit = 0

                    if results.get("stopped"):
                        break

                logger.info(
                    "Sequential tagging phase complete: %d processed, %d errors",
                    results["processed"],
                    results["errors"],
                )

        # --- SEQUENTIAL FALLBACK PATH (workers <= 1 or no parallel work) ---
        else:
            logger.info("Starting sequential backfill for actions: %s", list(actions.keys()))

            # Pre-load tag lookup for stitching tag assignment
            if "stitching" in actions:
                _all_tags: list[Tag] = db.query(Tag).order_by(Tag.description).all()
                _desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in _all_tags}
            else:
                _desc_to_tag = {}

            for item in design_items:
                if is_stop_requested():
                    logger.info("Stop signal detected — committing and exiting.")
                    results["stopped"] = True
                    break

                # Fetch the ORM object for this design
                design = db.query(Design).filter(Design.id == item.id).first()
                if design is None:
                    logger.warning("Design %d not found in DB — skipping", item.id)
                    results["processed"] += 1
                    continue

                # Read the pattern file once if any action needs it
                pattern = None
                if item.needs_images or item.needs_color_counts or item.needs_stitching:
                    filepath = _resolve_design_filepath(design.filepath)
                    if not filepath:
                        logger.info(
                            "Pattern file not found for design %d (stored: %r) — skipping actions",
                            item.id,
                            design.filepath,
                        )
                    else:
                        ext = _os.path.splitext(filepath)[1].lstrip(".").lower()
                        try:
                            import pyembroidery

                            _supported = {
                                e.lower()
                                for f in pyembroidery.supported_formats()
                                if f.get("reader") is not None
                                for e in f.get("extensions", (f.get("extension"),))
                            }
                        except Exception:
                            _supported = set()
                        if ext not in _supported:
                            logger.info(
                                "Unsupported extension %r for design %d (%s) — skipping actions",
                                ext,
                                item.id,
                                item.filename,
                            )
                        else:
                            try:
                                pattern = pyembroidery.read(filepath)
                                if pattern is None:
                                    logger.info(
                                        "pyembroidery.read returned None for design %d (%s)",
                                        item.id,
                                        item.filename,
                                    )
                            except Exception as exc:
                                logger.info(
                                    "pyembroidery.read failed for design %d (%s): %s",
                                    item.id,
                                    item.filename,
                                    exc,
                                )
                                pattern = None

                # Run each action for this design based on per-design metadata
                if "tagging" in actions:
                    err = run_tagging_action_runner(
                        db,
                        design,
                        actions["tagging"],
                        api_key,
                        delay,
                        vision_delay,
                        batch_size=batch_size,
                        action="tagging",
                        design_ids=tagging_design_ids,
                    )
                    if err:
                        results["errors"] += 1

                if item.needs_stitching:
                    err = run_stitching_action_runner(
                        db,
                        design,
                        actions.get("stitching", {}),
                        batch_size=batch_size,
                        pattern=pattern,
                    )
                    if err:
                        results["errors"] += 1

                if item.needs_images:
                    err = run_images_action_runner(db, design, actions.get("images", {}), pattern)
                    if err:
                        results["errors"] += 1

                if item.needs_color_counts:
                    err = run_color_counts_action_runner(
                        db, design, actions.get("color_counts", {}), pattern
                    )
                    if err:
                        results["errors"] += 1

                results["processed"] += 1
                count_since_commit += 1

                if count_since_commit >= commit_every:
                    db.commit()
                    count_since_commit = 0

    finally:
        # Final commit BEFORE restoring pragmas — PRAGMA journal_mode implicitly
        # commits any pending transaction in SQLite, which can cause data loss
        # when synchronous=OFF was active during the bulk write.
        if count_since_commit > 0:
            try:
                db.commit()
                logger.info(
                    "Committed final batch of %d designs before restoring pragmas",
                    count_since_commit,
                )
                count_since_commit = 0
            except Exception as exc:
                logger.error("Final commit before pragma restore failed: %s", exc)

        # Restore normal SQLite pragmas regardless of how we exited
        _restore_sqlite_after_bulk(db)

    logger.info(
        "Backfill complete: %d processed, %d errors, stopped=%s",
        results["processed"],
        results["errors"],
        results["stopped"],
    )

    return results
