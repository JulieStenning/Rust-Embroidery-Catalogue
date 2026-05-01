"""
Embroidery Catalogue — FastAPI application entry point.
Run with:  uvicorn src.main:app --reload
"""

import logging
from urllib.parse import quote

from fastapi import FastAPI, Request
from fastapi.responses import RedirectResponse
from fastapi.staticfiles import StaticFiles

from src import models  # noqa: F401 — ensures models are registered with Base.metadata
from src.database import get_db
from src.routes import (
    about,
    bulk_import,
    designers,
    designs,
    disclaimer,
    hoops,
    info,
    maintenance,
    projects,
    settings,
    sources,
    tagging_actions,
    tags,
)
from src.services import settings_service
from src.templating import templates  # noqa: F401 — shared Jinja2 instance with custom filters

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-8s %(name)s — %(message)s",
    datefmt="%H:%M:%S",
)

app = FastAPI(title="Embroidery Catalogue", version="0.1.0")

# ---------------------------------------------------------------------------
# Static files (Tailwind-compiled CSS, any JS, favicons, etc.)
# ---------------------------------------------------------------------------
app.mount("/static", StaticFiles(directory="static"), name="static")

# ---------------------------------------------------------------------------
# Routers
# ---------------------------------------------------------------------------
app.include_router(about.router)
app.include_router(disclaimer.router)
app.include_router(designers.router)
app.include_router(tags.router)
app.include_router(sources.router)
app.include_router(hoops.router)
app.include_router(designs.router)
app.include_router(projects.router)
app.include_router(bulk_import.router)
app.include_router(settings.router)
app.include_router(maintenance.router)
app.include_router(tagging_actions.router)
app.include_router(info.router)

_DISCLAIMER_EXEMPT_PREFIXES = (
    "/about",
    "/help",
    "/disclaimer",
    "/static",
    "/health",
    "/docs",
    "/redoc",
    "/openapi.json",
    "/favicon.ico",
)


def _is_disclaimer_exempt(path: str) -> bool:
    return any(
        path == prefix or path.startswith(prefix + "/") for prefix in _DISCLAIMER_EXEMPT_PREFIXES
    )


@app.middleware("http")
async def require_disclaimer_acceptance(request: Request, call_next):
    if _is_disclaimer_exempt(request.url.path):
        return await call_next(request)

    try:
        if settings_service.DISCLAIMER_ACK_FILE.exists():
            return await call_next(request)
    except OSError:
        pass

    db_gen = None
    try:
        dependency = app.dependency_overrides.get(get_db, get_db)
        db_gen = dependency()
        db = next(db_gen)
        if not settings_service.is_disclaimer_accepted(db):
            next_path = request.url.path
            if request.url.query:
                next_path = f"{next_path}?{request.url.query}"
            quoted_next = quote(next_path, safe="/?=&-._~")
            return RedirectResponse(f"/disclaimer?next={quoted_next}", status_code=303)
    except Exception:
        logging.getLogger(__name__).exception(
            "Failed to evaluate disclaimer acceptance; allowing request to continue."
        )
    finally:
        if db_gen is not None:
            try:
                db_gen.close()
            except Exception:
                pass

    return await call_next(request)


@app.get("/", include_in_schema=False)
def root():
    return RedirectResponse("/designs/")


@app.get("/favicon.ico", include_in_schema=False)
def favicon():
    return RedirectResponse(url="/static/icons/favicon.ico", status_code=307)


@app.get("/health")
def health_check():
    return {"status": "ok"}
