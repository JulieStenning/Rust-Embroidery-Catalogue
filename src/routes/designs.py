"""Routes for designs — browse, detail, create, edit, delete, rating, stitched toggle."""

from __future__ import annotations

import logging
import os
import subprocess
import sys
from dataclasses import dataclass, field
from urllib.parse import urlencode

from fastapi import APIRouter, Depends, Form, HTTPException, Query, Request
from fastapi.responses import HTMLResponse, RedirectResponse, Response
from sqlalchemy.orm import Session

from src.config import external_launches_disabled
from src.database import get_db
from src.models import Design as _DesignModel
from src.models import Tag as _TagModel
from src.services import designers, hoops, sources, tags
from src.services import designs as svc
from src.services import projects as projects_svc
from src.services import validation as val
from src.services.search import parse_advanced_query
from src.services.settings_service import get_designs_base_path
from src.templating import templates

router = APIRouter(prefix="/designs", tags=["designs"])
logger = logging.getLogger(__name__)

_DEFAULT_SORT_BY = "name"
_DEFAULT_SORT_DIR = "asc"


def _resolve_design_full_path(db: Session, design: _DesignModel) -> str:
    """Build an absolute file path for a stored design record in managed storage."""
    base = get_designs_base_path(db) or ""
    relative_path = (design.filepath or "").replace("/", os.sep).replace("\\", os.sep).lstrip("/\\")
    return (
        os.path.normpath(os.path.join(base, relative_path))
        if relative_path
        else os.path.normpath(base)
    )


def _nearest_existing_folder(path: str, fallback: str) -> str:
    """Return the deepest existing folder for *path*, falling back to *fallback*."""
    candidate = path if os.path.isdir(path) else os.path.dirname(path)
    while candidate:
        if os.path.isdir(candidate):
            return os.path.normpath(candidate)
        parent = os.path.dirname(candidate)
        if parent == candidate:
            break
        candidate = parent
    return os.path.normpath(fallback)


def _open_with_default_app(path: str) -> None:
    """Open a file using the OS default application."""
    if external_launches_disabled():
        logger.info(
            "Skipping default-app launch for %r because external launches are disabled.", path
        )
        return
    if os.name == "nt" and hasattr(os, "startfile"):
        os.startfile(path)
        return
    if sys.platform == "darwin":
        subprocess.Popen(["open", path])
    else:
        subprocess.Popen(["xdg-open", path])


@dataclass
class BrowseFilterState:
    """Centralises browse-page filter/query parameters, active-filter detection,
    and pagination query-string construction.  Keeping this logic in one place
    eliminates duplication between the route handler and any future helpers that
    need the same state.
    """

    q: str = ""
    all_words: str = ""
    exact_phrase: str = ""
    any_words: str = ""
    none_words: str = ""
    filename: str | None = None
    designer_id: int | None = None
    tag_ids: list[int] = field(default_factory=list)
    hoop_id: int | None = None
    source_id: int | None = None
    rating: int | None = None
    is_stitched: bool | None = None
    unverified: bool | None = None
    search_filename: bool = True
    search_tags: bool = True
    search_folder: bool = True
    sort_by: str = _DEFAULT_SORT_BY
    sort_dir: str = _DEFAULT_SORT_DIR

    @property
    def has_active_filters(self) -> bool:
        """Return True when any filter or search criterion is non-default."""
        return any(
            [
                bool(self.q),
                bool(self.all_words),
                bool(self.exact_phrase),
                bool(self.any_words),
                bool(self.none_words),
                bool(self.filename),
                self.designer_id is not None,
                bool(self.tag_ids),
                self.hoop_id is not None,
                self.source_id is not None,
                self.rating is not None,
                self.is_stitched is not None,
                self.unverified,
                not self.search_filename,
                not self.search_tags,
                not self.search_folder,
                self.sort_by != _DEFAULT_SORT_BY,
                self.sort_dir != _DEFAULT_SORT_DIR,
            ]
        )

    def to_query_pairs(self) -> list[tuple[str, str]]:
        """Build a list of (name, value) pairs suitable for ``urlencode``.

        Only non-default values are emitted so that generated URLs stay clean.
        The boolean search-scope flags are always included so that the next page
        respects the user's choices.
        """
        pairs: list[tuple[str, str]] = []
        if self.q:
            pairs.append(("q", self.q))
        if self.all_words:
            pairs.append(("all_words", self.all_words))
        if self.exact_phrase:
            pairs.append(("exact_phrase", self.exact_phrase))
        if self.any_words:
            pairs.append(("any_words", self.any_words))
        if self.none_words:
            pairs.append(("none_words", self.none_words))
        if self.filename:
            pairs.append(("filename", self.filename))
        if self.designer_id is not None:
            pairs.append(("designer_id", str(self.designer_id)))
        pairs.extend(("tag_ids", str(tid)) for tid in self.tag_ids)
        if self.hoop_id is not None:
            pairs.append(("hoop_id", str(self.hoop_id)))
        if self.source_id is not None:
            pairs.append(("source_id", str(self.source_id)))
        if self.rating is not None:
            pairs.append(("rating", str(self.rating)))
        if self.is_stitched is not None:
            pairs.append(("is_stitched", "true" if self.is_stitched else "false"))
        if self.unverified:
            pairs.append(("unverified", "true"))
        pairs.append(("search_filename", "true" if self.search_filename else "false"))
        pairs.append(("search_tags", "true" if self.search_tags else "false"))
        pairs.append(("search_folder", "true" if self.search_folder else "false"))
        if self.sort_by != _DEFAULT_SORT_BY:
            pairs.append(("sort_by", self.sort_by))
        if self.sort_dir != _DEFAULT_SORT_DIR:
            pairs.append(("sort_dir", self.sort_dir))
        return pairs

    def as_template_dict(self) -> dict:
        """Return a dict suitable for passing to the ``filters`` template context key."""
        return {
            "q": self.q,
            "all_words": self.all_words,
            "exact_phrase": self.exact_phrase,
            "any_words": self.any_words,
            "none_words": self.none_words,
            "filename": self.filename,
            "designer_id": self.designer_id,
            "tag_ids": self.tag_ids,
            "hoop_id": self.hoop_id,
            "source_id": self.source_id,
            "rating": self.rating,
            "is_stitched": self.is_stitched,
            "unverified": self.unverified,
            "search_filename": self.search_filename,
            "search_tags": self.search_tags,
            "search_folder": self.search_folder,
            "sort_by": self.sort_by,
            "sort_dir": self.sort_dir,
            "has_active_filters": self.has_active_filters,
        }


# ---------------------------------------------------------------------------
# Browse
# ---------------------------------------------------------------------------


@router.get("/", response_class=HTMLResponse)
def browse(
    request: Request,
    db: Session = Depends(get_db),
    q: str = "",
    all_words: str = "",
    exact_phrase: str = "",
    any_words: str = "",
    none_words: str = "",
    filename: str | None = None,
    designer_id: int | None = None,
    tag_ids: list[int] = Query(default=[]),
    hoop_id: int | None = None,
    source_id: int | None = None,
    rating: int | None = None,
    is_stitched: bool | None = None,
    unverified: bool | None = None,
    search_filename: bool = True,
    search_tags: bool = True,
    search_folder: bool = True,
    sort_by: str = _DEFAULT_SORT_BY,
    sort_dir: str = _DEFAULT_SORT_DIR,
    page: int = 1,
):
    pq = parse_advanced_query(
        q=q,
        all_words=all_words,
        exact_phrase=exact_phrase,
        any_words=any_words,
        none_words=none_words,
    )

    fs = BrowseFilterState(
        q=q,
        all_words=all_words,
        exact_phrase=exact_phrase,
        any_words=any_words,
        none_words=none_words,
        filename=filename,
        designer_id=designer_id,
        tag_ids=tag_ids,
        hoop_id=hoop_id,
        source_id=source_id,
        rating=rating,
        is_stitched=is_stitched,
        unverified=unverified,
        search_filename=search_filename,
        search_tags=search_tags,
        search_folder=search_folder,
        sort_by=sort_by,
        sort_dir=sort_dir,
    )

    page = max(1, page)
    offset = (page - 1) * svc.PAGE_SIZE
    items, total = svc.advanced_search(
        db,
        pq,
        filename=fs.filename,
        designer_id=fs.designer_id,
        tag_ids=fs.tag_ids or None,
        hoop_id=fs.hoop_id,
        source_id=fs.source_id,
        rating=fs.rating,
        is_stitched=fs.is_stitched,
        unverified=fs.unverified,
        search_filename=fs.search_filename,
        search_tags=fs.search_tags,
        search_folder=fs.search_folder,
        sort_by=fs.sort_by,
        sort_dir=fs.sort_dir,
        limit=svc.PAGE_SIZE,
        offset=offset,
    )
    total_pages = max(1, (total + svc.PAGE_SIZE - 1) // svc.PAGE_SIZE)

    return templates.TemplateResponse(
        request,
        "designs/browse.html",
        {
            "designs": items,
            "designers": designers.get_all(db),
            "tags": tags.get_all(db),
            "hoops": hoops.get_all(db),
            "sources": sources.get_all(db),
            "all_projects": projects_svc.get_all(db),
            "filters": fs.as_template_dict(),
            "page": page,
            "total_pages": total_pages,
            "total": total,
            "filter_params": urlencode(fs.to_query_pairs()),
        },
    )


# ---------------------------------------------------------------------------
# Bulk actions  (must be before /{design_id} routes to avoid 405)
# ---------------------------------------------------------------------------


@router.post("/bulk-verify", response_class=RedirectResponse)
def bulk_verify(
    request: Request,
    design_ids: list[int] = Form(default=[]),
    next: str | None = Form(None),
    db: Session = Depends(get_db),
):
    """Mark selected designs as verified (tags_checked=True) without changing their tags."""
    if design_ids:
        db.query(_DesignModel).filter(_DesignModel.id.in_(design_ids)).update(
            {"tags_checked": True}, synchronize_session=False
        )
        db.commit()
        logger.info("bulk_verify: verified %d designs", len(design_ids))
    return RedirectResponse(next or "/designs/", status_code=303)


@router.post("/bulk-set-tags", response_class=RedirectResponse)
def bulk_set_tags(
    request: Request,
    design_ids: list[int] = Form(default=[]),
    tag_ids: list[int] = Form(default=[]),
    next: str | None = Form(None),
    db: Session = Depends(get_db),
):
    selected_tag_ids = tag_ids
    if design_ids:
        tag_objs = (
            db.query(_TagModel).filter(_TagModel.id.in_(selected_tag_ids)).all()
            if selected_tag_ids
            else []
        )
        for did in design_ids:
            d = db.get(_DesignModel, did)
            if d:
                d.tags = tag_objs
                d.tags_checked = True
        db.commit()
        logger.info(
            "bulk_set_tags: updated %d designs with tag_ids=%s", len(design_ids), selected_tag_ids
        )
    return RedirectResponse(next or "/designs/", status_code=303)


@router.post("/bulk-add-to-project", response_class=RedirectResponse)
def bulk_add_to_project(
    request: Request,
    project_id: int = Form(...),
    design_ids: list[int] = Form(default=[]),
    next: str | None = Form(None),
    db: Session = Depends(get_db),
):
    if design_ids:
        try:
            projects_svc.add_designs(db, project_id, design_ids)
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc))
        logger.info(
            "bulk_add_to_project: added %d designs to project_id=%s", len(design_ids), project_id
        )
    return RedirectResponse(next or "/designs/", status_code=303)


# ---------------------------------------------------------------------------
# Detail
# ---------------------------------------------------------------------------


@router.get("/{design_id}", response_class=HTMLResponse)
def detail(request: Request, design_id: int, db: Session = Depends(get_db)):
    design = svc.get_by_id(db, design_id)
    if not design:
        raise HTTPException(status_code=404, detail="Design not found.")
    image_b64 = svc.get_image_base64(design)
    all_projects = projects_svc.get_all(db)
    design_project_ids = {p.id for p in design.projects}
    available_projects = [p for p in all_projects if p.id not in design_project_ids]
    all_tags = tags.get_all(db)
    return templates.TemplateResponse(
        request,
        "designs/detail.html",
        {
            "design": design,
            "image_b64": image_b64,
            "designers": designers.get_all(db),
            "tags": all_tags,
            "hoops": hoops.get_all(db),
            "sources": sources.get_all(db),
            "available_projects": available_projects,
            "assigned_tag_ids": {tag.id for tag in design.tags},
        },
    )


# ---------------------------------------------------------------------------
# Image endpoint (serves raw PNG bytes)
# ---------------------------------------------------------------------------


@router.get("/{design_id}/image")
def design_image(design_id: int, db: Session = Depends(get_db)):
    design = svc.get_by_id(db, design_id)
    if not design or not design.image_data:
        raise HTTPException(status_code=404, detail="Image not found.")
    return Response(content=design.image_data, media_type="image/png")


# ---------------------------------------------------------------------------
# Edit
# ---------------------------------------------------------------------------


@router.post("/{design_id}/edit", response_class=RedirectResponse)
def edit_design(
    design_id: int,
    db: Session = Depends(get_db),
    filename: str = Form(...),
    filepath: str = Form(...),
    notes: str | None = Form(None),
    rating: int | None = Form(None),
    is_stitched: bool = Form(False),
    designer_id: int | None = Form(None),
    source_id: int | None = Form(None),
    hoop_id: int | None = Form(None),
):
    try:
        svc.update(
            db,
            design_id,
            {
                "filename": filename,
                "filepath": filepath,
                "notes": notes,
                "rating": rating,
                "is_stitched": is_stitched,
                "designer_id": designer_id,
                "source_id": source_id,
                "hoop_id": hoop_id,
            },
        )
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Open in Editor / Explorer
# ---------------------------------------------------------------------------


@router.get("/{design_id}/open-in-editor", response_class=RedirectResponse)
def open_in_editor(design_id: int, db: Session = Depends(get_db)):
    design = svc.get_by_id(db, design_id)
    if not design or not design.filepath:
        raise HTTPException(status_code=404, detail="Design not found.")

    full_path = _resolve_design_full_path(db, design)
    logger.info("open_in_editor: full_path=%r", full_path)
    if external_launches_disabled():
        logger.info("open_in_editor: suppressed external launch for %r", full_path)
        return RedirectResponse(f"/designs/{design_id}", status_code=303)
    try:
        val.validate_path_exists(full_path, "Design file")
        _open_with_default_app(full_path)
    except Exception as exc:
        logger.warning("open_in_editor: failed to open %r (%s)", full_path, exc)
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


@router.get("/{design_id}/open-in-explorer", response_class=RedirectResponse)
def open_in_explorer(design_id: int, db: Session = Depends(get_db)):
    design = svc.get_by_id(db, design_id)
    if not design or not design.filepath:
        raise HTTPException(status_code=404, detail="Design not found.")

    full_path = _resolve_design_full_path(db, design)
    logger.info("open_in_explorer: full_path=%r", full_path)
    if external_launches_disabled():
        logger.info("open_in_explorer: suppressed external launch for %r", full_path)
        return RedirectResponse(f"/designs/{design_id}", status_code=303)
    try:
        if os.path.isfile(full_path):
            val.validate_path_exists(full_path, "Design file")
            result = subprocess.Popen(["explorer.exe", "/select,", os.path.normpath(full_path)])
        else:
            folder = _nearest_existing_folder(full_path, get_designs_base_path(db))
            try:
                val.validate_is_directory(folder, "Design folder")
            except Exception:
                folder = get_designs_base_path(db)
            logger.warning("open_in_explorer: file not found, opening folder=%r instead", folder)
            result = subprocess.Popen(["explorer.exe", os.path.normpath(folder)])
        logger.info("open_in_explorer: Popen pid=%s", result.pid)
    except Exception as exc:
        logger.warning("open_in_explorer: failed to open Explorer for %r (%s)", full_path, exc)
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Render 3D preview
# ---------------------------------------------------------------------------


@router.post("/{design_id}/render-3d-preview", response_class=RedirectResponse)
def render_3d_preview(design_id: int, db: Session = Depends(get_db)):
    """Render a 3D preview image for a single design, replacing any existing image."""
    import pyembroidery

    from src.services.preview import _render_preview

    design = svc.get_by_id(db, design_id)
    if not design:
        raise HTTPException(status_code=404, detail="Design not found.")

    full_path = _resolve_design_full_path(db, design)
    if not os.path.isfile(full_path):
        raise HTTPException(status_code=404, detail="Design file not found.")

    try:
        pattern = pyembroidery.read(full_path)
        if pattern is None:
            raise HTTPException(status_code=500, detail="Could not read embroidery file.")

        design.image_data = _render_preview(pattern, preview_3d=True)
        design.image_type = "3d"

        # Also update dimensions if missing
        bounds = pattern.bounds()
        if bounds:
            min_x, min_y, max_x, max_y = bounds
            design.width_mm = round((max_x - min_x) / 10.0, 2)
            design.height_mm = round((max_y - min_y) / 10.0, 2)

        db.commit()
        logger.info("Rendered 3D preview for design %d (%s)", design_id, design.filename)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("Failed to render 3D preview for design %d: %s", design_id, exc)
        raise HTTPException(status_code=500, detail=str(exc))

    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Toggle tags checked
# ---------------------------------------------------------------------------


@router.post("/{design_id}/toggle-tags-checked", response_class=RedirectResponse)
def toggle_tags_checked(
    design_id: int,
    tags_checked: str | None = Form(None),
    db: Session = Depends(get_db),
):
    checked = tags_checked == "true"
    try:
        svc.set_tags_checked(db, design_id, checked)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Set tags
# ---------------------------------------------------------------------------


@router.post("/{design_id}/set-tags", response_class=RedirectResponse)
def set_tags(
    design_id: int,
    tag_ids: list[int] = Form(default=[]),
    db: Session = Depends(get_db),
):
    selected_tag_ids = tag_ids
    try:
        svc.update(db, design_id, {"tag_ids": selected_tag_ids})
        # Saving tags always marks as verified — user reviewed them
        svc.set_tags_checked(db, design_id, True)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Stitched toggle
# ---------------------------------------------------------------------------


@router.post("/{design_id}/toggle-stitched", response_class=RedirectResponse)
def toggle_stitched(design_id: int, is_stitched: bool = Form(...), db: Session = Depends(get_db)):
    try:
        svc.set_stitched(db, design_id, is_stitched)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Rating
# ---------------------------------------------------------------------------


@router.post("/{design_id}/rate", response_class=RedirectResponse)
def rate_design(design_id: int, rating: int | None = Form(None), db: Session = Depends(get_db)):
    try:
        svc.set_rating(db, design_id, rating)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Delete
# ---------------------------------------------------------------------------


@router.post("/{design_id}/delete", response_class=RedirectResponse)
def delete_design(design_id: int, db: Session = Depends(get_db)):
    try:
        svc.delete(db, design_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return RedirectResponse("/designs/", status_code=303)


# ---------------------------------------------------------------------------
# Project membership (add / remove from design detail page)
# ---------------------------------------------------------------------------


@router.post("/{design_id}/add-to-project", response_class=RedirectResponse)
def add_to_project(
    design_id: int,
    project_id: int = Form(...),
    next: str | None = Form(None),
    db: Session = Depends(get_db),
):
    try:
        projects_svc.add_design(db, project_id, design_id)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(next or f"/designs/{design_id}", status_code=303)


@router.post("/{design_id}/remove-from-project/{project_id}", response_class=RedirectResponse)
def remove_from_project(design_id: int, project_id: int, db: Session = Depends(get_db)):
    projects_svc.remove_design(db, project_id, design_id)
    return RedirectResponse(f"/designs/{design_id}", status_code=303)


# ---------------------------------------------------------------------------
# Print view
# ---------------------------------------------------------------------------


@router.get("/{design_id}/print", response_class=HTMLResponse)
def print_design(request: Request, design_id: int, db: Session = Depends(get_db)):
    design = svc.get_by_id(db, design_id)
    if not design:
        raise HTTPException(status_code=404, detail="Design not found.")
    image_b64 = svc.get_image_base64(design)
    return templates.TemplateResponse(
        request,
        "designs/print.html",
        {"design": design, "image_b64": image_b64},
    )
