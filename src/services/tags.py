"""CRUD service for tags."""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from sqlalchemy.orm import Session

from src.models import Tag
from src.services.validation import validate_non_empty_string

# Allowed values for the tag_group field.
# "stitching" — describes the stitch technique (e.g. Applique, Blackwork, Cross Stitch).
# "image"     — describes what is depicted in the design (e.g. Animals, Flowers, Christmas).
VALID_TAG_GROUPS = frozenset({"stitching", "image"})

# Minimal built-in starter set used only when the delivered CSV is unavailable.
_FALLBACK_DEFAULT_TAG_ROWS: tuple[dict[str, int | str | None], ...] = (
    {"id": 1, "description": "Cross Stitch", "tag_group": "stitching"},
    {"id": 2, "description": "In The Hoop", "tag_group": "stitching"},
    {"id": 3, "description": "Filled", "tag_group": "stitching"},
    {"id": 4, "description": "Redwork", "tag_group": "stitching"},
    {"id": 5, "description": "Blackwork", "tag_group": "stitching"},
    {"id": 7, "description": "Cutwork", "tag_group": "stitching"},
    {"id": 9, "description": "Line Outline", "tag_group": "stitching"},
    {"id": 10, "description": "Satin Stitch", "tag_group": "stitching"},
    {"id": 11, "description": "Applique", "tag_group": "stitching"},
    {"id": 14, "description": "Lace", "tag_group": "stitching"},
    {"id": 18, "description": "Animals", "tag_group": "image"},
    {"id": 19, "description": "Flowers", "tag_group": "image"},
    {"id": 24, "description": "Music", "tag_group": "image"},
    {"id": 34, "description": "Christmas", "tag_group": "image"},
    {"id": 61, "description": "Quilting", "tag_group": "stitching"},
    {"id": 105, "description": "ITH Accessories", "tag_group": "stitching"},
    {"id": 111, "description": "Valentine's Day", "tag_group": "image"},
    {"id": 115, "description": "Sewing", "tag_group": "image"},
    {"id": 118, "description": "Dancing", "tag_group": "image"},
)


def _resolve_default_tags_csv() -> Path:
    """Return the delivered ``tags.csv`` path for source and packaged builds."""
    here = Path(__file__).resolve()
    candidates: list[Path] = []

    bundle_dir = getattr(sys, "_MEIPASS", None)
    if bundle_dir:
        candidates.append(Path(bundle_dir) / "data" / "tags.csv")

    if getattr(sys, "frozen", False):
        exe_dir = Path(sys.executable).resolve().parent
        candidates.append(exe_dir / "data" / "tags.csv")
        candidates.append(exe_dir / "_internal" / "data" / "tags.csv")

    for parent in here.parents:
        candidates.append(parent / "data" / "tags.csv")

    cwd = Path.cwd().resolve()
    candidates.append(cwd / "data" / "tags.csv")
    for parent in cwd.parents:
        candidates.append(parent / "data" / "tags.csv")

    seen: set[Path] = set()
    unique_candidates: list[Path] = []
    for candidate in candidates:
        if candidate not in seen:
            unique_candidates.append(candidate)
            seen.add(candidate)

    for candidate in unique_candidates:
        if candidate.exists():
            return candidate

    searched = "\n - ".join(str(path) for path in unique_candidates)
    raise FileNotFoundError(f"Could not locate delivered tags.csv. Tried:\n - {searched}")


def _validate_tag_group(tag_group: str | None, *, required: bool = False) -> str | None:
    if tag_group is None:
        if required:
            raise ValueError("A tag group is required. Choose 'stitching' or 'image'.")
        return None
    tag_group = tag_group.strip().lower()
    if tag_group not in VALID_TAG_GROUPS:
        raise ValueError(
            f"Invalid tag group '{tag_group}'. Must be one of: {', '.join(sorted(VALID_TAG_GROUPS))}."
        )
    return tag_group


def _load_default_tag_rows() -> list[dict[str, int | str | None]]:
    """Load the delivered starter tags from ``data/tags.csv``.

    If the CSV is unavailable (for example in a constrained CI or packaged
    environment), fall back to a built-in starter set so bootstrap_database()
    can still seed the essential tags required for first-run startup.
    """
    rows: list[dict[str, int | str | None]] = []
    try:
        tags_csv = _resolve_default_tags_csv()
    except FileNotFoundError:
        return [dict(row) for row in _FALLBACK_DEFAULT_TAG_ROWS]

    with tags_csv.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            rows.append(
                {
                    "id": int((row.get("id") or "0").strip()),
                    "description": validate_non_empty_string(
                        row.get("description", ""), "Tag description"
                    ),
                    "tag_group": _validate_tag_group(row.get("tag_group") or None),
                }
            )
    return rows


def seed_default_tags(db: Session) -> int:
    """Ensure the delivered starter tags exist with the current descriptions/groups."""
    changes = 0

    for row in _load_default_tag_rows():
        tag = db.get(Tag, row["id"])
        if tag is None:
            tag = db.query(Tag).filter(Tag.description == row["description"]).first()

        if tag is None:
            db.add(
                Tag(
                    id=int(row["id"]),
                    description=str(row["description"]),
                    tag_group=row["tag_group"],
                )
            )
            changes += 1
            continue

        changed = False
        if tag.description != row["description"]:
            tag.description = str(row["description"])
            changed = True
        if tag.tag_group != row["tag_group"]:
            tag.tag_group = row["tag_group"]
            changed = True
        if changed:
            changes += 1

    db.commit()
    return changes


def get_all(db: Session) -> list[Tag]:
    return db.query(Tag).order_by(Tag.description).all()


def get_by_id(db: Session, tag_id: int) -> Tag | None:
    return db.get(Tag, tag_id)


def create(db: Session, description: str, tag_group: str) -> Tag:
    """Create a new tag. `tag_group` is required for new tags."""
    description = validate_non_empty_string(description, "Tag description")
    validated_group = _validate_tag_group(tag_group, required=True)
    existing = db.query(Tag).filter(Tag.description == description).first()
    if existing:
        raise ValueError(f"Tag '{description}' already exists.")
    tag = Tag(description=description, tag_group=validated_group)
    db.add(tag)
    db.commit()
    db.refresh(tag)
    return tag


def update(db: Session, tag_id: int, description: str, tag_group: str | None = None) -> Tag:
    """Update a tag's description and optionally its group."""
    description = validate_non_empty_string(description, "Tag description")
    tag = db.get(Tag, tag_id)
    if not tag:
        raise ValueError(f"Tag with id={tag_id} not found.")
    tag.description = description
    if tag_group is not None:
        tag.tag_group = _validate_tag_group(tag_group)
    db.commit()
    db.refresh(tag)
    return tag


def set_group(db: Session, tag_id: int, tag_group: str) -> Tag:
    """Set (or change) the group for an existing tag."""
    validated_group = _validate_tag_group(tag_group, required=True)
    tag = db.get(Tag, tag_id)
    if not tag:
        raise ValueError(f"Tag with id={tag_id} not found.")
    tag.tag_group = validated_group
    db.commit()
    db.refresh(tag)
    return tag


def delete(db: Session, tag_id: int) -> None:
    tag = db.get(Tag, tag_id)
    if not tag:
        raise ValueError(f"Tag with id={tag_id} not found.")
    db.delete(tag)
    db.commit()
