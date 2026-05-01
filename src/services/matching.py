"""
Designer/source path matching and normalization helpers.
"""

import re

from src.models import Design, Designer, Source


def normalize_name_for_matching(name: str) -> str:
    """Normalize a name for case-insensitive, separator-agnostic comparison."""
    name = name.lower()
    name = re.sub(r"[_\-/\\]+", " ", name)
    name = re.sub(r"\s+", " ", name)
    return name.strip()


def suggest_source_from_path(filepath: str, sources: list[Source]) -> Source | None:
    path_norm = normalize_name_for_matching(filepath)
    for s in sorted(sources, key=lambda x: len(x.name), reverse=True):
        if hasattr(s, "name") and s.name.lower() in {"don't know", "me"}:
            continue
        norm = normalize_name_for_matching(s.name)
        if norm and norm in path_norm:
            return s
    return None


def suggest_designer_from_path(filepath: str, designers: list[Designer]) -> Designer | None:
    path_norm = normalize_name_for_matching(filepath)
    for d in sorted(designers, key=lambda x: len(x.name), reverse=True):
        if hasattr(d, "name") and d.name.lower() in {"don't know", "me"}:
            continue
        norm = normalize_name_for_matching(d.name)
        if norm and norm in path_norm:
            return d
    return None


def find_or_create_designer(db, name: str) -> Designer:
    name = name.strip()
    if not name:
        raise ValueError("Designer name must not be blank.")
    name_lower = name.lower()
    for d in db.query(Designer).all():
        if d.name.lower() == name_lower:
            return d
    designer = Designer(name=name)
    db.add(designer)
    db.commit()
    db.refresh(designer)
    return designer


def find_or_create_source(db, name: str) -> Source:
    name = name.strip()
    if not name:
        raise ValueError("Source name must not be blank.")
    name_lower = name.lower()
    for s in db.query(Source).all():
        if s.name.lower() == name_lower:
            return s
    source = Source(name=name)
    db.add(source)
    db.commit()
    db.refresh(source)
    return source


def backfill_source_from_path(db) -> int:
    sources = db.query(Source).all()
    unassigned = db.query(Design).filter(Design.source_id.is_(None)).all()
    updated = 0
    for design in unassigned:
        match = suggest_source_from_path(design.filepath, sources)
        if match:
            design.source_id = match.id
            updated += 1
    if updated:
        db.commit()
    return updated


def backfill_designer_from_path(db) -> int:
    designers = db.query(Designer).all()
    unassigned = db.query(Design).filter(Design.designer_id.is_(None)).all()
    updated = 0
    for design in unassigned:
        match = suggest_designer_from_path(design.filepath, designers)
        if match:
            design.designer_id = match.id
            updated += 1
    if updated:
        db.commit()
    return updated
