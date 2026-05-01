"""Routes for designer admin pages."""

import re
from urllib.parse import urlencode

from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import HTMLResponse, RedirectResponse
from sqlalchemy.orm import Session

from src.database import get_db
from src.services import designers as svc
from src.templating import templates

router = APIRouter(prefix="/admin/designers", tags=["designers"])

_UUID_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")


def _safe_import_token(token: str) -> str:
    """Return the token if it is a valid UUID v4, otherwise return an empty string."""
    return token if _UUID_RE.match(token or "") else ""


def _render_designer_list(request: Request, db: Session, import_token: str = "") -> HTMLResponse:
    items = svc.get_all(db)
    return templates.TemplateResponse(
        request,
        "admin/designers.html",
        {"designers": items, "import_token": _safe_import_token(import_token)},
    )


def _redirect_to_designers(import_token: str = "") -> RedirectResponse:
    """Return a redirect to the designers list, preserving import mode if active."""
    safe_token = _safe_import_token(import_token)
    url = "/admin/designers/"
    if safe_token:
        url = f"{url}?{urlencode({'import_token': safe_token})}"
    return RedirectResponse(url, status_code=303)


@router.get("/", response_class=HTMLResponse)
def list_designers(request: Request, import_token: str = "", db: Session = Depends(get_db)):
    return _render_designer_list(request, db, import_token=import_token)


@router.post("/", response_class=RedirectResponse)
def create_designer(
    name: str = Form(...),
    import_token: str = Form(default=""),
    db: Session = Depends(get_db),
):
    try:
        svc.create(db, name)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return _redirect_to_designers(import_token)


@router.post("/{designer_id}/delete", response_class=RedirectResponse)
def delete_designer(
    designer_id: int,
    import_token: str = Form(default=""),
    db: Session = Depends(get_db),
):
    try:
        svc.delete(db, designer_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return _redirect_to_designers(import_token)
