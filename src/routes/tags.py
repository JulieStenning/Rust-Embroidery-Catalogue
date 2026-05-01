"""Routes for tag admin pages."""

import re
from urllib.parse import urlencode

from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import HTMLResponse, RedirectResponse
from sqlalchemy.orm import Session

from src.database import get_db
from src.services import tags as svc
from src.templating import templates

router = APIRouter(prefix="/admin/tags", tags=["tags"])

_UUID_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")


def _safe_import_token(token: str) -> str:
    """Return the token if it is a valid UUID v4, otherwise return an empty string."""
    return token if _UUID_RE.match(token or "") else ""


def _render_tag_list(request: Request, db: Session, import_token: str = "") -> HTMLResponse:
    items = svc.get_all(db)
    return templates.TemplateResponse(
        request,
        "admin/tags.html",
        {"tags": items, "import_token": _safe_import_token(import_token)},
    )


def _redirect_to_tags(import_token: str = "") -> RedirectResponse:
    """Return a redirect to the tags list, preserving import mode if active."""
    safe_token = _safe_import_token(import_token)
    url = "/admin/tags/"
    if safe_token:
        url = f"{url}?{urlencode({'import_token': safe_token})}"
    return RedirectResponse(url, status_code=303)


def _create_tag(
    description: str, tag_group: str, db: Session, import_token: str = ""
) -> RedirectResponse:
    try:
        svc.create(db, description, tag_group)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return _redirect_to_tags(import_token)


def _set_tag_group(
    tag_id: int, tag_group: str, db: Session, import_token: str = ""
) -> RedirectResponse:
    try:
        svc.set_group(db, tag_id, tag_group)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return _redirect_to_tags(import_token)


def _delete_tag(tag_id: int, db: Session, import_token: str = "") -> RedirectResponse:
    try:
        svc.delete(db, tag_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return _redirect_to_tags(import_token)


@router.get("/", response_class=HTMLResponse)
def list_tags(request: Request, import_token: str = "", db: Session = Depends(get_db)):
    return _render_tag_list(request, db, import_token=import_token)


@router.post("/", response_class=RedirectResponse)
def create_tag(
    description: str = Form(...),
    tag_group: str = Form(...),
    import_token: str = Form(default=""),
    db: Session = Depends(get_db),
):
    return _create_tag(description, tag_group, db, import_token=import_token)


@router.post("/{tag_id}/set-group", response_class=RedirectResponse)
def set_tag_group(
    tag_id: int,
    tag_group: str = Form(...),
    import_token: str = Form(default=""),
    db: Session = Depends(get_db),
):
    return _set_tag_group(tag_id, tag_group, db, import_token=import_token)


@router.post("/{tag_id}/delete", response_class=RedirectResponse)
def delete_tag(tag_id: int, import_token: str = Form(default=""), db: Session = Depends(get_db)):
    return _delete_tag(tag_id, db, import_token=import_token)
