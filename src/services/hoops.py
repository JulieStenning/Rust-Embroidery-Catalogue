"""CRUD service for Hoops."""

from sqlalchemy.orm import Session

from src.models import Hoop
from src.services.validation import validate_non_empty_string, validate_positive_number

# Seed data as defined in the project plan
SEED_HOOPS = [
    {"name": "Hoop A", "max_width_mm": 126.0, "max_height_mm": 126.0},
    {"name": "Hoop B", "max_width_mm": 200.0, "max_height_mm": 140.0},
    {"name": "Gigahoop", "max_width_mm": 230.0, "max_height_mm": 200.0},
]


def get_all(db: Session) -> list[Hoop]:
    return db.query(Hoop).order_by(Hoop.name).all()


def get_by_id(db: Session, hoop_id: int) -> Hoop | None:
    return db.get(Hoop, hoop_id)


def create(db: Session, name: str, max_width_mm: float, max_height_mm: float) -> Hoop:
    name = validate_non_empty_string(name, "Hoop name")
    validate_positive_number(max_width_mm, "max_width_mm")
    validate_positive_number(max_height_mm, "max_height_mm")
    existing = db.query(Hoop).filter(Hoop.name == name).first()
    if existing:
        raise ValueError(f"Hoop '{name}' already exists.")
    hoop = Hoop(name=name, max_width_mm=max_width_mm, max_height_mm=max_height_mm)
    db.add(hoop)
    db.commit()
    db.refresh(hoop)
    return hoop


def update(db: Session, hoop_id: int, name: str, max_width_mm: float, max_height_mm: float) -> Hoop:
    name = validate_non_empty_string(name, "Hoop name")
    validate_positive_number(max_width_mm, "max_width_mm")
    validate_positive_number(max_height_mm, "max_height_mm")
    hoop = db.get(Hoop, hoop_id)
    if not hoop:
        raise ValueError(f"Hoop with id={hoop_id} not found.")
    hoop.name = name
    hoop.max_width_mm = max_width_mm
    hoop.max_height_mm = max_height_mm
    db.commit()
    db.refresh(hoop)
    return hoop


def delete(db: Session, hoop_id: int) -> None:
    hoop = db.get(Hoop, hoop_id)
    if not hoop:
        raise ValueError(f"Hoop with id={hoop_id} not found.")
    db.delete(hoop)
    db.commit()


def select_hoop_for_dimensions(db: Session, width_mm: float, height_mm: float) -> Hoop | None:
    """
    Return the smallest hoop that fits the given dimensions.
    Checks both orientations (width×height and height×width).
    """
    hoops = db.query(Hoop).order_by(Hoop.max_width_mm, Hoop.max_height_mm).all()
    for hoop in hoops:
        fits_normal = width_mm <= float(hoop.max_width_mm) and height_mm <= float(
            hoop.max_height_mm
        )
        fits_rotated = height_mm <= float(hoop.max_width_mm) and width_mm <= float(
            hoop.max_height_mm
        )
        if fits_normal or fits_rotated:
            return hoop
    return None


def seed_hoops(db: Session) -> None:
    """Insert the standard hoops if they don't already exist."""
    for hoop_data in SEED_HOOPS:
        existing = db.query(Hoop).filter(Hoop.name == hoop_data["name"]).first()
        if not existing:
            hoop = Hoop(**hoop_data)
            db.add(hoop)
    db.commit()
