"""Admin maintenance routes — find and remove orphaned design records."""

from __future__ import annotations

import logging
import os
import subprocess

from fastapi import APIRouter, Depends, Form, Query, Request
from fastapi.responses import HTMLResponse, JSONResponse, RedirectResponse
from sqlalchemy.orm import Session

from src.config import DATABASE_URL, DESIGNS_BASE_PATH, external_launches_disabled
from src.database import get_db
from src.services import backup_service
from src.services import designs as svc
from src.services import validation as val
from src.services.folder_picker import FolderPickerUnavailableError, pick_folder
from src.services.preview import _read_art_metadata
from src.services.settings_service import (
    SETTING_BACKUP_DB_DESTINATION,
    SETTING_BACKUP_DESIGNS_DESTINATION,
    get_designs_base_path,
    get_setting,
    set_setting,
)
from src.templating import templates

router = APIRouter(prefix="/admin/maintenance", tags=["maintenance"])
logger = logging.getLogger(__name__)


@router.get("/orphans/scan", response_class=JSONResponse)
def scan_orphans(db: Session = Depends(get_db)):
    base_path = get_designs_base_path(db)
    result = svc.scan_orphaned(db, base_path)
    return result


@router.get("/browse-path")
def browse_path(
    filepath: str = Query(...),
    db: Session = Depends(get_db),
):
    """Open Explorer to the deepest existing ancestor folder of a missing file, with validation."""
    base_path = get_designs_base_path(db)
    full_path = os.path.normpath(base_path.rstrip("/\\") + filepath.replace("\\", os.sep))
    # Start from the directory of the (missing) file and walk upward
    folder = os.path.dirname(full_path)
    try:
        while folder:
            try:
                val.validate_is_directory(folder, "Target folder")
                break
            except Exception:
                parent = os.path.dirname(folder)
                if parent == folder:
                    folder = base_path
                    break
                folder = parent
        # Final safety net
        val.validate_is_directory(folder, "Target folder")
    except Exception as exc:
        logger.warning("browse_path: invalid folder %r (%s)", folder, exc)
        folder = base_path
    logger.info("browse_path: opening Explorer at %r", folder)
    if external_launches_disabled():
        logger.info("browse_path: suppressed Explorer launch at %r", folder)
        return {"ok": True, "opened": folder}
    try:
        subprocess.Popen(["explorer", folder])
    except OSError:
        logger.exception("browse_path: failed to open Explorer at %r", folder)
    return {"ok": True, "opened": folder}


@router.get("/orphans", response_class=HTMLResponse)
def orphans_page(
    request: Request, page: int = Query(default=1, ge=1), db: Session = Depends(get_db)
):
    base_path = get_designs_base_path(db)
    page_size = 100
    offset = (page - 1) * page_size
    orphaned, total = svc.get_orphaned(db, base_path, limit=page_size, offset=offset)
    total_pages = max(1, (total + page_size - 1) // page_size)
    return templates.TemplateResponse(
        request,
        "admin/orphans.html",
        {
            "orphaned": orphaned,
            "base_path": base_path,
            "page": page,
            "total": total,
            "total_pages": total_pages,
            "page_size": page_size,
        },
    )


@router.post("/orphans/delete", response_class=RedirectResponse)
def delete_orphans(
    request: Request,
    design_ids: list[int] = Form(default=[]),
    page: int = Form(default=1),
    db: Session = Depends(get_db),
):
    count = svc.delete_orphaned(db, design_ids)
    logger.info("Deleted %d orphaned design record(s).", count)
    return RedirectResponse(
        f"/admin/maintenance/orphans?page={page}&deleted={count}", status_code=303
    )


@router.post("/orphans/delete-all", response_class=RedirectResponse)
def delete_all_orphans(request: Request, db: Session = Depends(get_db)):
    base_path = get_designs_base_path(db)
    count = svc.delete_all_orphaned(db, base_path)
    logger.info("Deleted all %d orphaned design record(s).", count)
    return RedirectResponse(f"/admin/maintenance/orphans?deleted={count}", status_code=303)


# ---------------------------------------------------------------------------
# Backup routes
# ---------------------------------------------------------------------------


def _resolve_db_path() -> str:
    """Return the filesystem path to the live SQLite database file."""
    # DATABASE_URL is like "sqlite:////<abs_path>" or "sqlite:///<rel_path>"
    url = DATABASE_URL
    if url.startswith("sqlite:///"):
        return url[len("sqlite:///") :]
    return url


def _display_path(path: str) -> str:
    """Return a Windows-friendly display path using backslashes."""
    if not path:
        return ""
    return os.path.normpath(path).replace("/", "\\")


def _normalize_destination_path(path: str) -> str:
    """Normalize a user-entered destination path for saving/display."""
    path = (path or "").strip()
    if not path:
        return ""
    return _display_path(path)


def _resolve_picker_initial_dir(start_dir: str) -> str:
    """Return a safe starting directory for the native folder picker."""
    initial = (start_dir or "").strip()
    if os.path.isabs(initial):
        if os.path.isdir(initial):
            return initial
        parent = os.path.dirname(initial.rstrip("\\/"))
        if parent and os.path.isdir(parent):
            return parent

    initial = os.path.expanduser("~")
    if not os.path.isdir(initial):
        initial = "C:\\"
    return initial


@router.get("/backup/browse-folder", response_class=JSONResponse)
def backup_browse_folder(start_dir: str = "", kind: str = "backup"):
    """Open a native folder picker for selecting a backup destination."""
    initial = _resolve_picker_initial_dir(start_dir)
    titles = {
        "database": "Select database backup folder",
        "designs": "Select designs backup folder",
    }
    title = titles.get(kind, "Select backup folder")

    try:
        path = pick_folder(start_dir=initial, title=title)
        return JSONResponse({"path": _display_path(path or "")})
    except FolderPickerUnavailableError as exc:
        return JSONResponse(
            {
                "error": (
                    "Folder picker is not available on this system. "
                    f"Please enter the path manually. ({exc})"
                )
            },
            status_code=200,
        )


@router.get("/backup", response_class=HTMLResponse)
def backup_page(request: Request, db: Session = Depends(get_db)):
    """Render the backup management page."""
    db_destination = get_setting(db, SETTING_BACKUP_DB_DESTINATION)
    designs_destination = get_setting(db, SETTING_BACKUP_DESIGNS_DESTINATION)

    return templates.TemplateResponse(
        request,
        "admin/backup.html",
        {
            "db_destination": db_destination,
            "designs_destination": designs_destination,
            "db_destination_display": _display_path(db_destination),
            "designs_destination_display": _display_path(designs_destination),
            "db_path": _display_path(_resolve_db_path()),
            "designs_path": _display_path(DESIGNS_BASE_PATH),
        },
    )


@router.post("/backup/save-settings", response_class=RedirectResponse)
def backup_save_settings(
    request: Request,
    db_destination: str = Form(default=""),
    designs_destination: str = Form(default=""),
    db: Session = Depends(get_db),
):
    """Persist the two backup destination paths."""
    db_destination = _normalize_destination_path(db_destination)
    designs_destination = _normalize_destination_path(designs_destination)
    current_db_destination = _normalize_destination_path(
        get_setting(db, SETTING_BACKUP_DB_DESTINATION)
    )
    current_designs_destination = _normalize_destination_path(
        get_setting(db, SETTING_BACKUP_DESIGNS_DESTINATION)
    )

    if (
        db_destination == current_db_destination
        and designs_destination == current_designs_destination
    ):
        return RedirectResponse(
            "/admin/maintenance/backup?error=no_destinations_to_save",
            status_code=303,
        )

    set_setting(db, SETTING_BACKUP_DB_DESTINATION, db_destination)
    set_setting(db, SETTING_BACKUP_DESIGNS_DESTINATION, designs_destination)
    return RedirectResponse("/admin/maintenance/backup?saved=1", status_code=303)


@router.post("/backup/database", response_class=RedirectResponse)
def run_database_backup(request: Request, db: Session = Depends(get_db)):
    """Trigger an immediate database backup."""
    destination = get_setting(db, SETTING_BACKUP_DB_DESTINATION)
    if not destination:
        return RedirectResponse("/admin/maintenance/backup?error=no_db_dest", status_code=303)

    db_path = _resolve_db_path()
    result = backup_service.backup_database(db_path, destination)

    if result.success:
        logger.info("Database backup succeeded: %s", result.backup_path)
        return RedirectResponse(
            f"/admin/maintenance/backup?db_ok=1&db_path={result.backup_path}"
            f"&db_size={result.size_bytes}&db_time={result.completed_at}",
            status_code=303,
        )

    logger.error("Database backup failed: %s", result.error)
    return RedirectResponse(f"/admin/maintenance/backup?db_error={result.error}", status_code=303)


@router.post("/backup/designs", response_class=RedirectResponse)
def run_designs_backup(request: Request, db: Session = Depends(get_db)):
    """Trigger an immediate incremental designs backup."""
    destination = get_setting(db, SETTING_BACKUP_DESIGNS_DESTINATION)
    if not destination:
        return RedirectResponse("/admin/maintenance/backup?error=no_designs_dest", status_code=303)

    result = backup_service.backup_designs(DESIGNS_BASE_PATH, destination)

    if result.success:
        logger.info(
            "Designs backup succeeded — copied %d, updated %d, archived %d",
            result.copied,
            result.updated,
            result.archived,
        )
        return RedirectResponse(
            f"/admin/maintenance/backup"
            f"?designs_ok=1"
            f"&d_scanned={result.scanned}"
            f"&d_copied={result.copied}"
            f"&d_updated={result.updated}"
            f"&d_unchanged={result.unchanged}"
            f"&d_archived={result.archived}"
            f"&d_bytes={result.total_bytes_copied}"
            f"&d_time={result.completed_at}",
            status_code=303,
        )

    logger.error("Designs backup failed: %s", result.error)
    return RedirectResponse(
        f"/admin/maintenance/backup?designs_error={result.error}", status_code=303
    )


@router.post("/backup/both", response_class=RedirectResponse)
def run_both_backups(request: Request, db: Session = Depends(get_db)):
    """Trigger database backup and designs backup in sequence."""
    db_destination = get_setting(db, SETTING_BACKUP_DB_DESTINATION)
    designs_destination = get_setting(db, SETTING_BACKUP_DESIGNS_DESTINATION)

    if not db_destination or not designs_destination:
        return RedirectResponse("/admin/maintenance/backup?error=no_destinations", status_code=303)

    params: list[str] = []

    if db_destination:
        db_path = _resolve_db_path()
        db_result = backup_service.backup_database(db_path, db_destination)
        if db_result.success:
            params += [
                "db_ok=1",
                f"db_path={db_result.backup_path}",
                f"db_size={db_result.size_bytes}",
                f"db_time={db_result.completed_at}",
            ]
        else:
            params.append(f"db_error={db_result.error}")

    if designs_destination:
        des_result = backup_service.backup_designs(DESIGNS_BASE_PATH, designs_destination)
        if des_result.success:
            params += [
                "designs_ok=1",
                f"d_scanned={des_result.scanned}",
                f"d_copied={des_result.copied}",
                f"d_updated={des_result.updated}",
                f"d_unchanged={des_result.unchanged}",
                f"d_archived={des_result.archived}",
                f"d_bytes={des_result.total_bytes_copied}",
                f"d_time={des_result.completed_at}",
            ]
        else:
            params.append(f"designs_error={des_result.error}")

    query = "&".join(params)
    return RedirectResponse(f"/admin/maintenance/backup?{query}", status_code=303)


# ---------------------------------------------------------------------------
# Colour count backfill
# ---------------------------------------------------------------------------


@router.post("/backfill-color-counts", response_class=RedirectResponse)
def run_backfill_color_counts(
    request: Request,
    redo: str | None = Form(default=None),
    db: Session = Depends(get_db),
):
    """Backfill stitch_count, color_count and color_change_count for designs
    where these values are NULL.  Reads each embroidery file via pyembroidery."""
    import pyembroidery

    base_path = get_designs_base_path(db)
    if not base_path:
        return RedirectResponse("/admin/tagging-actions/?error=no_base_path", status_code=303)

    from src.models import Design

    q = db.query(Design).order_by(Design.id)
    if not redo:
        q = q.filter(
            (Design.stitch_count.is_(None))
            | (Design.color_count.is_(None))
            | (Design.color_change_count.is_(None))
        )
    designs: list[Design] = q.all()

    total = len(designs)
    updated = 0
    errors = 0
    BATCH_SIZE = 100

    for i, design in enumerate(designs, 1):
        rel = design.filepath.replace("/", os.sep)
        full_path = base_path.rstrip("\\/") + rel

        if not os.path.isfile(full_path):
            errors += 1
            continue

        try:
            pattern = pyembroidery.read(full_path)
            if pattern is None:
                errors += 1
                continue

            _, ext = os.path.splitext(design.filename)
            ext = ext.lower()

            changed = False

            # For .art files, try Wilcom metadata first
            if ext == ".art":
                meta = _read_art_metadata(full_path)
                if meta.get("stitch_count") is not None and (redo or design.stitch_count is None):
                    design.stitch_count = meta["stitch_count"]
                    changed = True
                if meta.get("color_count") is not None and (redo or design.color_count is None):
                    design.color_count = meta["color_count"]
                    changed = True

            # Fall back to pyembroidery pattern methods
            if redo or design.stitch_count is None:
                try:
                    design.stitch_count = pattern.count_stitches()
                    changed = True
                except Exception:
                    pass
            if redo or design.color_count is None:
                try:
                    design.color_count = pattern.count_threads()
                    changed = True
                except Exception:
                    pass
            if redo or design.color_change_count is None:
                try:
                    design.color_change_count = pattern.count_color_changes()
                    changed = True
                except Exception:
                    pass

            if changed:
                updated += 1

        except Exception:
            errors += 1
            continue

        # Batch commit every BATCH_SIZE designs to avoid long-running transactions
        if updated > 0 and updated % BATCH_SIZE == 0:
            db.commit()

    db.commit()

    logger.info(
        "Colour count backfill complete: %d updated, %d errors out of %d designs",
        updated,
        errors,
        total,
    )

    params = (
        f"done=1"
        f"&action=backfill_color_counts"
        f"&total={total}"
        f"&updated={updated}"
        f"&errors={errors}"
    )
    return RedirectResponse(f"/admin/tagging-actions/?{params}", status_code=303)


# ---------------------------------------------------------------------------
# Image backfill
# ---------------------------------------------------------------------------


@router.post("/clear-images", response_class=RedirectResponse)
def run_clear_images(
    request: Request,
    db: Session = Depends(get_db),
):
    """Clear image_data, width_mm, height_mm and hoop_id for ALL designs.

    This is used before re-processing all images so that the subsequent
    backfill only processes designs with NULL images (fast, incremental).
    """
    from src.models import Design

    count = (
        db.query(Design)
        .filter(Design.image_data.isnot(None))
        .update(
            {
                Design.image_data: None,
                Design.width_mm: None,
                Design.height_mm: None,
                Design.hoop_id: None,
            },
            synchronize_session="fetch",
        )
    )
    db.commit()

    logger.info("Cleared image data for %d design(s).", count)

    params = f"done=1" f"&action=clear_images" f"&cleared={count}"
    return RedirectResponse(f"/admin/tagging-actions/?{params}", status_code=303)


@router.post("/backfill-images", response_class=RedirectResponse)
def run_backfill_images(
    request: Request,
    redo: str | None = Form(default=None),
    db: Session = Depends(get_db),
):
    """Backfill image_data (preview PNG), width_mm, height_mm and hoop_id for
    designs where these values are NULL.  Reads each embroidery file via
    pyembroidery and renders a preview with Pillow."""
    import pyembroidery

    from src.services.hoops import select_hoop_for_dimensions
    from src.services.preview import _render_preview

    base_path = get_designs_base_path(db)
    if not base_path:
        return RedirectResponse("/admin/tagging-actions/?error=no_base_path", status_code=303)

    from src.models import Design

    q = db.query(Design).order_by(Design.id)
    if not redo:
        q = q.filter(Design.image_data.is_(None))
    designs: list[Design] = q.all()

    total = len(designs)
    updated = 0
    errors = 0
    BATCH_SIZE = 100

    for i, design in enumerate(designs, 1):
        rel = design.filepath.replace("/", os.sep)
        full_path = base_path.rstrip("\\/") + rel

        if not os.path.isfile(full_path):
            errors += 1
            continue

        try:
            pattern = pyembroidery.read(full_path)
            if pattern is None:
                errors += 1
                continue

            changed = False

            # Dimensions
            bounds = pattern.bounds()
            if bounds and (redo or design.width_mm is None):
                min_x, min_y, max_x, max_y = bounds
                design.width_mm = round((max_x - min_x) / 10.0, 2)
                design.height_mm = round((max_y - min_y) / 10.0, 2)
                changed = True

            # Hoop
            if design.hoop_id is None and design.width_mm and design.height_mm:
                hoop = select_hoop_for_dimensions(db, design.width_mm, design.height_mm)
                if hoop:
                    design.hoop_id = hoop.id
                    changed = True

            # Preview image
            if redo or design.image_data is None:
                design.image_data = _render_preview(pattern)
                changed = True

            if changed:
                updated += 1

        except Exception:
            errors += 1
            continue

        # Batch commit every BATCH_SIZE designs
        if updated > 0 and updated % BATCH_SIZE == 0:
            db.commit()

    db.commit()

    logger.info(
        "Image backfill complete: %d updated, %d errors out of %d designs",
        updated,
        errors,
        total,
    )

    params = (
        f"done=1"
        f"&action=backfill_images"
        f"&total={total}"
        f"&updated={updated}"
        f"&errors={errors}"
    )
    return RedirectResponse(f"/admin/tagging-actions/?{params}", status_code=303)
