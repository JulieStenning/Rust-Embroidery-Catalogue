"""Admin route for application settings."""

from __future__ import annotations

from fastapi import APIRouter, Depends, Form, Request
from fastapi.responses import HTMLResponse, JSONResponse, RedirectResponse
from sqlalchemy.orm import Session

from src.database import get_db
from src.services import settings_service as svc
from src.services.folder_picker import FolderPickerUnavailableError, display_path, pick_folder
from src.templating import templates

router = APIRouter(prefix="/admin/settings", tags=["settings"])

_MIN_BATCH_SIZE = 1
_MAX_BATCH_SIZE = 10000


def _normalize_optional_batch_size(raw: str | None) -> str:
    """Return blank or a clamped integer string for batch-size inputs."""
    if raw is None:
        return ""
    value = raw.strip()
    if not value:
        return ""
    try:
        parsed = int(value)
    except ValueError:
        return ""
    if parsed < _MIN_BATCH_SIZE:
        parsed = _MIN_BATCH_SIZE
    if parsed > _MAX_BATCH_SIZE:
        parsed = _MAX_BATCH_SIZE
    return str(parsed)


@router.get("/", response_class=HTMLResponse)
def settings_page(request: Request, db: Session = Depends(get_db)):
    google_api_key = svc.get_google_api_key()
    ai_tier2_auto = svc._is_truthy(svc.get_setting(db, svc.SETTING_AI_TIER2_AUTO))
    ai_tier3_auto = svc._is_truthy(svc.get_setting(db, svc.SETTING_AI_TIER3_AUTO))
    ai_batch_size = svc.get_setting(db, svc.SETTING_AI_BATCH_SIZE)
    ai_delay = svc.get_setting(db, svc.SETTING_AI_DELAY)
    import_commit_batch_size = svc.get_setting(db, svc.SETTING_IMPORT_COMMIT_BATCH_SIZE)
    image_preference = svc.get_setting(db, svc.SETTING_IMAGE_PREFERENCE)
    return templates.TemplateResponse(
        request,
        "admin/settings.html",
        {
            "app_mode": svc.APP_MODE,
            "can_configure_data_root": svc.APP_MODE == "desktop",
            "data_root": svc.get_data_root(db),
            "database_file_path": svc.get_database_file_path(db),
            "log_folder": svc.get_logs_dir(db),
            "managed_designs_path": svc.get_designs_base_path(db),
            "google_api_key": google_api_key,
            "has_google_api_key": bool(google_api_key),
            "ai_tagging_help_url": "/about/document/ai-tagging",
            "ai_tier2_auto": ai_tier2_auto,
            "ai_tier3_auto": ai_tier3_auto,
            "ai_batch_size": ai_batch_size,
            "ai_delay": ai_delay,
            "import_commit_batch_size": import_commit_batch_size,
            "image_preference": image_preference,
        },
    )


@router.get("/browse-data-root", response_class=JSONResponse)
def browse_data_root(start_dir: str = ""):
    try:
        path = pick_folder(start_dir=start_dir, title="Select catalogue home folder")
        return JSONResponse({"path": display_path(path or "")})
    except FolderPickerUnavailableError as exc:
        return JSONResponse({"error": str(exc)}, status_code=200)


@router.post("/", response_class=RedirectResponse)
def save_settings(
    request: Request,
    google_api_key: str | None = Form(None),
    data_root: str | None = Form(None),
    ai_tier2_auto: str | None = Form(None),
    ai_tier3_auto: str | None = Form(None),
    ai_batch_size: str | None = Form(None),
    ai_delay: str | None = Form(None),
    import_commit_batch_size: str | None = Form(None),
    image_preference: str | None = Form(None),
    db: Session = Depends(get_db),
):
    # Managed-only storage no longer exposes a writable designs base path.
    # Keep the POST endpoint so older forms/bookmarks still redirect cleanly.
    if google_api_key is not None:
        try:
            svc.save_google_api_key(google_api_key)
        except OSError:
            return RedirectResponse("/admin/settings/?error=1", status_code=303)

    if svc.APP_MODE == "desktop" and data_root is not None and data_root.strip():
        try:
            svc.save_data_root(data_root)
        except (OSError, ValueError):
            return RedirectResponse("/admin/settings/?error=1", status_code=303)

    # Save AI tagging preferences (checkboxes are absent from POST when unchecked)
    svc.set_setting(db, svc.SETTING_AI_TIER2_AUTO, "true" if ai_tier2_auto else "false")
    svc.set_setting(db, svc.SETTING_AI_TIER3_AUTO, "true" if ai_tier3_auto else "false")
    svc.set_setting(db, svc.SETTING_AI_BATCH_SIZE, _normalize_optional_batch_size(ai_batch_size))
    svc.set_setting(db, svc.SETTING_AI_DELAY, (ai_delay or "").strip())
    svc.set_setting(
        db,
        svc.SETTING_IMPORT_COMMIT_BATCH_SIZE,
        _normalize_optional_batch_size(import_commit_batch_size),
    )
    # Save image preference (2d or 3d)
    if image_preference in ("2d", "3d"):
        svc.set_setting(db, svc.SETTING_IMAGE_PREFERENCE, image_preference)
    return RedirectResponse("/admin/settings/?saved=1", status_code=303)
