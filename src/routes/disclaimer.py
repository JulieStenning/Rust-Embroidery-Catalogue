"""One-time disclaimer acknowledgement routes."""

from __future__ import annotations

from pathlib import Path

from fastapi import APIRouter, Depends, Form, Request
from fastapi.responses import HTMLResponse, RedirectResponse
from markupsafe import Markup
from sqlalchemy.orm import Session

from src.database import get_db
from src.services import settings_service as svc
from src.templating import templates

router = APIRouter(tags=["disclaimer"])

_PROJECT_ROOT = Path(__file__).resolve().parents[2]
_DISCLAIMER_HTML = _PROJECT_ROOT / "DISCLAIMER.html"


def _safe_next_path(next_path: str | None) -> str:
    if not next_path or not next_path.startswith("/"):
        return "/designs/"
    return next_path


@router.get("/disclaimer", response_class=HTMLResponse)
def disclaimer_page(
    request: Request,
    next: str = "/designs/",
    db: Session = Depends(get_db),
):
    if svc.is_disclaimer_accepted(db):
        return RedirectResponse(_safe_next_path(next), status_code=303)

    disclaimer_html = (
        _DISCLAIMER_HTML.read_text(encoding="utf-8")
        if _DISCLAIMER_HTML.exists()
        else "<p>DISCLAIMER.html was not found.</p>"
    )
    return templates.TemplateResponse(
        request,
        "disclaimer.html",
        {
            "disclaimer_html": Markup(disclaimer_html),
            "next_path": _safe_next_path(next),
        },
    )


@router.post("/disclaimer/accept", response_class=RedirectResponse)
def accept_disclaimer(
    next: str = Form("/designs/"),
    db: Session = Depends(get_db),
):
    svc.mark_disclaimer_accepted(db)
    return RedirectResponse(_safe_next_path(next), status_code=303)
