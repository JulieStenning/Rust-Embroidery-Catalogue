"""CRUD service for Designs."""

from __future__ import annotations

import base64
import logging
import os
from datetime import date
from typing import Any

from sqlalchemy import func, nulls_last
from sqlalchemy.orm import Session, joinedload

from src.models import Design, Tag
from src.services.search import ParsedQuery, build_search_filters
from src.services.validation import validate_positive_number, validate_rating

logger = logging.getLogger(__name__)

PAGE_SIZE = 50


def _wildcard_to_ilike(pattern: str) -> str:
    """Convert user wildcard pattern (* ?) to a SQL ILIKE pattern (% _).

    If the pattern contains no wildcards, wraps it in % for a substring match.
    """
    if "*" in pattern or "?" in pattern:
        # Only replace * and ? for SQL wildcards, do not escape _ or % for SQLite
        return pattern.replace("*", "%").replace("?", "_")
    # plain text → substring search, do not escape _ or %
    return f"%{pattern}%"


def _apply_standard_filters(
    q,
    *,
    filename: str | None = None,
    designer_id: int | None = None,
    tag_ids: list[int] | None = None,
    hoop_id: int | None = None,
    source_id: int | None = None,
    rating: int | None = None,
    is_stitched: bool | None = None,
    unverified: bool | None = None,
):
    """Apply the browse page's standard filters to a design query."""
    if filename:
        ilike_pattern = _wildcard_to_ilike(filename)
        # Match against the stored filename stem OR the full name derived from
        # filepath (which always includes the extension, e.g. ".jef").
        q = q.filter(
            Design.filename.ilike(ilike_pattern)
            | (Design.filename + func.file_extension(Design.filepath)).ilike(ilike_pattern)
        )
    if designer_id is not None:
        q = q.filter(Design.designer_id == designer_id)
    if hoop_id is not None:
        q = q.filter(Design.hoop_id == hoop_id)
    if source_id is not None:
        q = q.filter(Design.source_id == source_id)
    if rating is not None:
        q = q.filter(Design.rating == rating)
    if is_stitched is not None:
        q = q.filter(Design.is_stitched == is_stitched)
    if tag_ids:
        if -1 in tag_ids:
            q = q.filter(~Design.tags.any())
        else:
            for tag_id in tag_ids:
                q = q.filter(Design.tags.any(Tag.id == tag_id))
    if unverified:
        q = q.filter(Design.tags_checked == False)  # noqa: E712
    return q


def get_all(
    db: Session,
    *,
    filename: str | None = None,
    designer_id: int | None = None,
    tag_ids: list[int] | None = None,
    hoop_id: int | None = None,
    source_id: int | None = None,
    rating: int | None = None,
    is_stitched: bool | None = None,
    unverified: bool | None = None,
    sort_by: str = "name",
    sort_dir: str = "asc",
    limit: int = PAGE_SIZE,
    offset: int = 0,
) -> tuple[list[Design], int]:
    """Return (page_of_designs, total_count)."""
    # Build the sort expression
    # "folder" = the immediate parent directory of the file
    # e.g. \DVDs\...\pes\01.pes  →  "pes"
    # Uses a custom SQLite function registered in `database.py`.
    _folder_expr = func.parent_folder(Design.filepath)
    _sort_col = {
        "name": Design.filename,
        "folder": _folder_expr,
        "date_added": Design.date_added,
    }.get(sort_by, Design.filename)

    if sort_dir == "desc":
        _order = nulls_last(_sort_col.desc())
    else:
        _order = nulls_last(_sort_col.asc())
    """Return (page_of_designs, total_count)."""
    q = (
        db.query(Design)
        .options(
            joinedload(Design.designer),
            joinedload(Design.source),
            joinedload(Design.hoop),
            joinedload(Design.tags),
        )
        .order_by(_order)
    )
    q = _apply_standard_filters(
        q,
        filename=filename,
        designer_id=designer_id,
        tag_ids=tag_ids,
        hoop_id=hoop_id,
        source_id=source_id,
        rating=rating,
        is_stitched=is_stitched,
        unverified=unverified,
    )
    total = q.count()
    items = q.offset(offset).limit(limit).all()
    return items, total


def advanced_search(
    db: Session,
    pq: ParsedQuery,
    *,
    filename: str | None = None,
    designer_id: int | None = None,
    tag_ids: list[int] | None = None,
    hoop_id: int | None = None,
    source_id: int | None = None,
    rating: int | None = None,
    is_stitched: bool | None = None,
    unverified: bool | None = None,
    search_filename: bool = True,
    search_tags: bool = True,
    search_folder: bool = True,
    sort_by: str = "name",
    sort_dir: str = "asc",
    limit: int = PAGE_SIZE,
    offset: int = 0,
) -> tuple[list[Design], int]:
    """Return (page_of_designs, total_count) matching *pq*.

    Field checkboxes control which columns are searched.
    """
    _folder_expr = func.parent_folder(Design.filepath)
    _sort_col = {
        "name": Design.filename,
        "folder": _folder_expr,
        "date_added": Design.date_added,
    }.get(sort_by, Design.filename)

    if sort_dir == "desc":
        _order = nulls_last(_sort_col.desc())
    else:
        _order = nulls_last(_sort_col.asc())

    q = (
        db.query(Design)
        .options(
            joinedload(Design.designer),
            joinedload(Design.source),
            joinedload(Design.hoop),
            joinedload(Design.tags),
        )
        .order_by(_order)
    )

    q = _apply_standard_filters(
        q,
        filename=filename,
        designer_id=designer_id,
        tag_ids=tag_ids,
        hoop_id=hoop_id,
        source_id=source_id,
        rating=rating,
        is_stitched=is_stitched,
        unverified=unverified,
    )

    for condition in build_search_filters(
        pq,
        search_filename=search_filename,
        search_tags=search_tags,
        search_folder=search_folder,
    ):
        q = q.filter(condition)

    total = q.count()
    items = q.offset(offset).limit(limit).all()
    return items, total


def get_by_id(db: Session, design_id: int) -> Design | None:
    return (
        db.query(Design)
        .options(
            joinedload(Design.designer),
            joinedload(Design.source),
            joinedload(Design.hoop),
            joinedload(Design.tags),
            joinedload(Design.projects),
        )
        .filter(Design.id == design_id)
        .first()
    )


def create(db: Session, data: dict[str, Any]) -> Design:
    _validate_design_data(data)
    tag_ids = data.pop("tag_ids", None) or []
    # Explicitly set tags_checked if provided, otherwise let default apply
    tags_checked = data.pop("tags_checked", None)
    design = Design(**data)
    if tags_checked is not None:
        design.tags_checked = tags_checked
    if not design.date_added:
        design.date_added = date.today()
    if tag_ids:
        design.tags = db.query(Tag).filter(Tag.id.in_(tag_ids)).all()
    db.add(design)
    db.commit()
    db.refresh(design)

    # Auto-backfill all actions for new designs
    import pyembroidery

    from src.config import DESIGNS_BASE_PATH
    from src.services.auto_tagging import run_stitching_backfill_action, run_tagging_action
    from src.services.preview import _render_preview

    # Tagging (Tier 1 only, no API key)
    run_tagging_action(
        db,
        action="tag_untagged",
        tiers=[1],
        api_key="",
        batch_size=1,
        delay=0,
        vision_delay=0,
        overwrite_verified=False,
        dry_run=False,
    )
    # Stitching
    run_stitching_backfill_action(
        db, batch_size=1, dry_run=False, allowed_descriptions=None, clear_existing_stitching=False
    )
    # Image and color counts
    rel = design.filepath.replace("/", "\\")
    full_path = DESIGNS_BASE_PATH.rstrip("/\\") + rel
    try:
        pattern = pyembroidery.read(full_path)
        if pattern is not None:
            if design.image_data is None:
                design.image_data = _render_preview(pattern)
                design.image_type = "3d"
            bounds = pattern.bounds()
            if bounds and design.width_mm is None:
                min_x, min_y, max_x, max_y = bounds
                design.width_mm = round((max_x - min_x) / 10.0, 2)
                design.height_mm = round((max_y - min_y) / 10.0, 2)
            if design.hoop_id is None and design.width_mm and design.height_mm:
                from src.services.hoops import select_hoop_for_dimensions

                hoop = select_hoop_for_dimensions(db, design.width_mm, design.height_mm)
                if hoop:
                    design.hoop_id = hoop.id
            if design.stitch_count is None:
                design.stitch_count = pattern.count_stitches()
            if design.color_count is None:
                design.color_count = pattern.count_threads()
            if design.color_change_count is None:
                design.color_change_count = pattern.count_color_changes()
    except Exception as e:
        logger.warning(
            f"Auto-backfill (image/color counts) failed for design {design.filename}: {e}"
        )
    db.commit()

    return design


def update(db: Session, design_id: int, data: dict[str, Any]) -> Design:
    _validate_design_data(data)
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    tag_ids = data.pop("tag_ids", None)
    for key, value in data.items():
        setattr(design, key, value)
    if tag_ids is not None:
        design.tags = db.query(Tag).filter(Tag.id.in_(tag_ids)).all()
    db.commit()
    db.refresh(design)
    return design


def delete(db: Session, design_id: int) -> None:
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    db.delete(design)
    db.commit()


def set_stitched(db: Session, design_id: int, is_stitched: bool) -> Design:
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    design.is_stitched = is_stitched
    db.commit()
    db.refresh(design)
    return design


def set_rating(db: Session, design_id: int, rating: int | None) -> Design:
    validate_rating(rating)
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    design.rating = rating
    db.commit()
    db.refresh(design)
    return design


def set_tags_checked(db: Session, design_id: int, tags_checked: bool) -> Design:
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    design.tags_checked = tags_checked
    db.commit()
    db.refresh(design)
    return design


def get_image_base64(design: Design) -> str | None:
    """Return the stored image as a base64-encoded PNG string, or None."""
    if not design.image_data:
        return None
    return base64.b64encode(design.image_data).decode("utf-8")


def _full_path(base_path: str, filepath: str) -> str:
    """Resolve a stored filepath (which may use backslashes) against base_path."""
    return os.path.normpath(base_path.rstrip("/\\") + filepath.replace("\\", os.sep))


def scan_orphaned(db: Session, base_path: str) -> dict:
    """Quickly count files checked and orphans found without loading full ORM objects."""
    rows = db.query(Design.filepath).all()
    checked = 0
    found = 0
    for (filepath,) in rows:
        if not filepath:
            continue
        checked += 1
        if not os.path.isfile(_full_path(base_path, filepath)):
            found += 1
    return {"checked": checked, "found": found}


def _find_orphan_ids(db: Session, base_path: str) -> list[int]:
    """Return IDs of all designs whose file is missing, sorted by filepath."""
    rows = db.query(Design.id, Design.filepath).order_by(Design.filepath).all()
    return [
        design_id
        for design_id, filepath in rows
        if filepath and not os.path.isfile(_full_path(base_path, filepath))
    ]


def get_orphaned(
    db: Session, base_path: str, limit: int = 100, offset: int = 0
) -> tuple[list[Design], int]:
    """Return (page_of_orphaned_designs, total_orphan_count).

    Scans only id+filepath columns for the full table, then loads full ORM
    objects only for the requested page slice.
    """
    orphan_ids = _find_orphan_ids(db, base_path)
    total = len(orphan_ids)
    page_ids = orphan_ids[offset : offset + limit]
    if not page_ids:
        return [], total
    designs = (
        db.query(Design)
        .options(joinedload(Design.designer))
        .filter(Design.id.in_(page_ids))
        .order_by(Design.filepath)
        .all()
    )
    return designs, total


def delete_orphaned(db: Session, design_ids: list[int]) -> int:
    """Delete designs by a list of IDs. Returns the count of deleted records."""
    if not design_ids:
        return 0
    from sqlalchemy import delete as _delete

    total = 0
    chunk_size = 500
    for i in range(0, len(design_ids), chunk_size):
        chunk = design_ids[i : i + chunk_size]
        result = db.execute(_delete(Design).where(Design.id.in_(chunk)))
        total += result.rowcount
    db.commit()
    return total


def delete_all_orphaned(db: Session, base_path: str) -> int:
    """Delete every design whose file is missing. Returns the count."""
    from sqlalchemy import delete as _delete

    orphan_ids = _find_orphan_ids(db, base_path)
    if not orphan_ids:
        return 0
    total = 0
    chunk_size = 500
    # For each orphan, check if the file is missing, log a warning if so, but do not raise
    for orphan_id in orphan_ids:
        design = db.get(Design, orphan_id)
        if design is not None:
            full_path = _full_path(base_path, design.filepath)
            if not os.path.isfile(full_path):
                logger.warning(
                    f"Design file missing for orphaned design {design.filename} at {full_path}"
                )
    for i in range(0, len(orphan_ids), chunk_size):
        chunk = orphan_ids[i : i + chunk_size]
        result = db.execute(_delete(Design).where(Design.id.in_(chunk)))
        total += result.rowcount
    db.commit()
    return total


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _validate_design_data(data: dict[str, Any]) -> None:
    validate_positive_number(data.get("width_mm"), "width_mm")
    validate_positive_number(data.get("height_mm"), "height_mm")
    validate_rating(data.get("rating"))
