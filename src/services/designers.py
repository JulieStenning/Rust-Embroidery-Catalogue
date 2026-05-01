"""CRUD service for Designers."""

from sqlalchemy.orm import Session

from src.models import Designer
from src.services.validation import validate_non_empty_string


def get_all(db: Session) -> list[Designer]:
    return db.query(Designer).order_by(Designer.name).all()


def get_by_id(db: Session, designer_id: int) -> Designer | None:
    return db.get(Designer, designer_id)


def create(db: Session, name: str) -> Designer:
    name = validate_non_empty_string(name, "Designer name")
    existing = db.query(Designer).filter(Designer.name == name).first()
    if existing:
        raise ValueError(f"Designer '{name}' already exists.")
    designer = Designer(name=name)
    db.add(designer)
    db.commit()
    db.refresh(designer)
    return designer


def update(db: Session, designer_id: int, name: str) -> Designer:
    name = validate_non_empty_string(name, "Designer name")
    designer = db.get(Designer, designer_id)
    if not designer:
        raise ValueError(f"Designer with id={designer_id} not found.")
    designer.name = name
    db.commit()
    db.refresh(designer)
    return designer


def delete(db: Session, designer_id: int) -> None:
    designer = db.get(Designer, designer_id)
    if not designer:
        raise ValueError(f"Designer with id={designer_id} not found.")
    db.delete(designer)
    db.commit()
