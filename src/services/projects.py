"""CRUD service for Projects."""

from __future__ import annotations

from datetime import date

from sqlalchemy.orm import Session, joinedload

from src.models import Design, Project
from src.services.validation import validate_non_empty_string


def get_all(db: Session) -> list[Project]:
    return db.query(Project).order_by(Project.name).all()


def get_by_id(db: Session, project_id: int) -> Project | None:
    return (
        db.query(Project)
        .options(joinedload(Project.designs).joinedload(Design.designer))
        .filter(Project.id == project_id)
        .first()
    )


def create(db: Session, name: str, description: str | None = None) -> Project:
    name = validate_non_empty_string(name, "Project name")
    existing = db.query(Project).filter(Project.name == name).first()
    if existing:
        raise ValueError(f"Project '{name}' already exists.")
    project = Project(name=name, description=description, date_created=date.today())
    db.add(project)
    db.commit()
    db.refresh(project)
    return project


def update(db: Session, project_id: int, name: str, description: str | None = None) -> Project:
    name = validate_non_empty_string(name, "Project name")
    project = db.get(Project, project_id)
    if not project:
        raise ValueError(f"Project with id={project_id} not found.")
    project.name = name
    project.description = description
    db.commit()
    db.refresh(project)
    return project


def delete(db: Session, project_id: int) -> None:
    project = db.get(Project, project_id)
    if not project:
        raise ValueError(f"Project with id={project_id} not found.")
    db.delete(project)
    db.commit()


def add_design(db: Session, project_id: int, design_id: int) -> Project:
    project = db.get(Project, project_id)
    if not project:
        raise ValueError(f"Project with id={project_id} not found.")
    design = db.get(Design, design_id)
    if not design:
        raise ValueError(f"Design with id={design_id} not found.")
    if design not in project.designs:
        project.designs.append(design)
        db.commit()
    return project


def add_designs(db: Session, project_id: int, design_ids: list[int]) -> Project:
    project = db.get(Project, project_id)
    if not project:
        raise ValueError(f"Project with id={project_id} not found.")

    unique_ids = list(dict.fromkeys(design_ids))
    if not unique_ids:
        return project

    found_designs = db.query(Design).filter(Design.id.in_(unique_ids)).all()
    found_by_id = {design.id: design for design in found_designs}
    missing_ids = [design_id for design_id in unique_ids if design_id not in found_by_id]
    if missing_ids:
        raise ValueError(f"Design with id={missing_ids[0]} not found.")

    existing_ids = {design.id for design in project.designs}
    added_any = False
    for design_id in unique_ids:
        if design_id not in existing_ids:
            project.designs.append(found_by_id[design_id])
            added_any = True

    if added_any:
        db.commit()
        db.refresh(project)
    return project


def remove_design(db: Session, project_id: int, design_id: int) -> Project:
    project = db.get(Project, project_id)
    if not project:
        raise ValueError(f"Project with id={project_id} not found.")
    design = db.get(Design, design_id)
    if design and design in project.designs:
        project.designs.remove(design)
        db.commit()
    return project
