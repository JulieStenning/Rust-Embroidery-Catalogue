"""Routes for the in-app Help page."""

from fastapi import APIRouter, Request
from fastapi.responses import HTMLResponse

from src.templating import templates

router = APIRouter(tags=["info"])


@router.get("/help", response_class=HTMLResponse)
def help_page(request: Request):
    return templates.TemplateResponse(
        request,
        "info/help.html",
        {},
    )
