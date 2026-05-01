"""Routes for Projects — list, detail, create, edit, delete, add/remove designs, print."""

from __future__ import annotations

from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import HTMLResponse, RedirectResponse
from sqlalchemy.orm import Session

from src.database import get_db
from src.services import designs as designs_svc
from src.services import projects as svc
from src.templating import templates

router = APIRouter(prefix="/projects", tags=["projects"])


@router.get("/", response_class=HTMLResponse)
def list_projects(request: Request, db: Session = Depends(get_db)):
    items = svc.get_all(db)
    return templates.TemplateResponse(request, "projects/list.html", {"projects": items})


@router.get("/new", response_class=HTMLResponse)
def new_project_form(request: Request):
    return templates.TemplateResponse(request, "projects/form.html", {"project": None})


@router.post("/", response_class=RedirectResponse)
def create_project(
    name: str = Form(...),
    description: str | None = Form(None),
    db: Session = Depends(get_db),
):
    try:
        project = svc.create(db, name, description)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(f"/projects/{project.id}", status_code=303)


@router.get("/{project_id}", response_class=HTMLResponse)
def project_detail(request: Request, project_id: int, db: Session = Depends(get_db)):
    project = svc.get_by_id(db, project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found.")
    return templates.TemplateResponse(
        request,
        "projects/detail.html",
        {"project": project},
    )


@router.post("/{project_id}/edit", response_class=RedirectResponse)
def edit_project(
    project_id: int,
    name: str = Form(...),
    description: str | None = Form(None),
    db: Session = Depends(get_db),
):
    try:
        svc.update(db, project_id, name, description)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return RedirectResponse(f"/projects/{project_id}", status_code=303)


@router.post("/{project_id}/delete", response_class=RedirectResponse)
def delete_project(project_id: int, db: Session = Depends(get_db)):
    try:
        svc.delete(db, project_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    return RedirectResponse("/projects/", status_code=303)


@router.post("/{project_id}/remove-design/{design_id}", response_class=RedirectResponse)
def remove_design(project_id: int, design_id: int, db: Session = Depends(get_db)):
    svc.remove_design(db, project_id, design_id)
    return RedirectResponse(f"/projects/{project_id}", status_code=303)


@router.get("/{project_id}/print", response_class=HTMLResponse)
def print_project(request: Request, project_id: int, db: Session = Depends(get_db)):
    project = svc.get_by_id(db, project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found.")
    designs_with_images = [
        {"design": d, "image_b64": designs_svc.get_image_base64(d)} for d in project.designs
    ]
    return templates.TemplateResponse(
        request,
        "projects/print.html",
        {"project": project, "designs_with_images": designs_with_images},
    )
