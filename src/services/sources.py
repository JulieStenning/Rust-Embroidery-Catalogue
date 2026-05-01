"""CRUD service for Sources."""

from sqlalchemy.orm import Session

from src.models import Source
from src.services.validation import validate_non_empty_string


def get_all(db: Session) -> list[Source]:
    return db.query(Source).order_by(Source.name).all()


def get_by_id(db: Session, source_id: int) -> Source | None:
    return db.get(Source, source_id)


def create(db: Session, name: str) -> Source:
    name = validate_non_empty_string(name, "Source name")
    existing = db.query(Source).filter(Source.name == name).first()
    if existing:
        raise ValueError(f"Source '{name}' already exists.")
    source = Source(name=name)
    db.add(source)
    db.commit()
    db.refresh(source)
    return source


def update(db: Session, source_id: int, name: str) -> Source:
    name = validate_non_empty_string(name, "Source name")
    source = db.get(Source, source_id)
    if not source:
        raise ValueError(f"Source with id={source_id} not found.")
    source.name = name
    db.commit()
    db.refresh(source)
    return source


def delete(db: Session, source_id: int) -> None:
    source = db.get(Source, source_id)
    if not source:
        raise ValueError(f"Source with id={source_id} not found.")
    db.delete(source)
    db.commit()
