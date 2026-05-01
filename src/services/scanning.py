"""
Folder scanning and deduplication helpers for bulk import.
"""

import os
from collections import defaultdict
from dataclasses import dataclass
from pathlib import PureWindowsPath

from sqlalchemy.orm import Session

from src.models import Design as _Design

# Supported embroidery file extensions and their priority for import
SUPPORTED_EXTENSIONS = {
    ".jef",
    ".pes",
    ".hus",
    ".dst",
    ".exp",
    ".vp3",
    ".u01",
    ".pec",
    ".xxx",
    ".tbf",
    ".10o",
    ".100",
    ".dat",
    ".dsb",
    ".dsz",
    ".emd",
    ".exy",
    ".fxy",
    ".gt",
    ".inb",
    ".jpx",
    ".max",
    ".mit",
    ".new",
    ".pcm",
    ".pcq",
    ".pcs",
    ".phb",
    ".phc",
    ".sew",
    ".shv",
    ".stc",
    ".stx",
    ".tap",
    ".zhs",
    ".zxy",
    ".gcode",
    ".art",
    ".pmv",
}
EXTENSION_PRIORITY = [
    ".jef",
    ".pes",
    ".vp3",
    ".hus",
    ".dst",
    ".exp",
    ".u01",
    ".pec",
    ".xxx",
    ".tbf",
    ".10o",
    ".100",
    ".dat",
    ".dsb",
    ".dsz",
    ".emd",
    ".exy",
    ".fxy",
    ".gt",
    ".inb",
    ".jpx",
    ".max",
    ".mit",
    ".new",
    ".pcm",
    ".pcq",
    ".pcs",
    ".phb",
    ".phc",
    ".sew",
    ".shv",
    ".stc",
    ".stx",
    ".tap",
    ".zhs",
    ".zxy",
    ".gcode",
    ".art",
    ".pmv",
]


@dataclass
class ScannedDesign:
    filename: str
    filepath: str
    width_mm: float | None = None
    height_mm: float | None = None
    hoop_id: int | None = None
    hoop_name: str | None = None
    image_data: bytes | None = None
    error: str | None = None
    source_full_path: str | None = None
    designer_override_set: bool = False
    override_designer_id: int | None = None
    source_override_set: bool = False
    override_source_id: int | None = None
    folder_key: str | None = None
    folder_label: str | None = None
    folder_root: str | None = None
    stitch_count: int | None = None
    color_count: int | None = None
    color_change_count: int | None = None
    source_folder: str | None = None


def _pick_preferred(paths: list[str]) -> str:
    for ext in EXTENSION_PRIORITY:
        for p in paths:
            if os.path.splitext(p)[1].lower() == ext:
                return p
    return paths[0]


def _selected_source_root_name(folder_path: str) -> str:
    normalized = str(PureWindowsPath(folder_path.replace("/", "\\"))).rstrip("\\")
    return PureWindowsPath(normalized).name


def _build_managed_rel_filepath(
    source_folder: str,
    full_path: str,
    managed_root: str | None = None,
) -> str:
    try:
        rel = os.path.relpath(full_path, source_folder)
    except ValueError:
        rel = os.path.basename(full_path)
    source_root = managed_root or _selected_source_root_name(source_folder)
    rel_parts = [part for part in str(PureWindowsPath(rel)).split("\\") if part]
    parts = ([source_root] if source_root else []) + rel_parts
    return "\\" + "\\".join(parts)


def _resolve_source_full_path(
    source_folder: str,
    rel_filepath: str,
    managed_root: str | None = None,
) -> str | None:
    clean_rel = rel_filepath.lstrip("/\\")
    parts = [p for p in clean_rel.replace("/", "\\").split("\\") if p]
    source_root = _selected_source_root_name(source_folder)
    accepted_roots = {r.lower() for r in (managed_root, source_root) if r}
    if accepted_roots and parts and parts[0].lower() in accepted_roots:
        parts = parts[1:]
    source_rel = os.path.join(*parts) if parts else ""
    candidate = os.path.normpath(os.path.join(source_folder, source_rel))
    try:
        source_base = os.path.normcase(os.path.abspath(source_folder))
        candidate_abs = os.path.normcase(os.path.abspath(candidate))
        if os.path.commonpath([source_base, candidate_abs]) != source_base:
            return None
    except ValueError:
        return None
    return candidate


def scan_folder(
    folder_path: str,
    db: Session,
    managed_root: str | None = None,
) -> list[ScannedDesign]:
    stem_groups: dict[tuple[str, str], list[str]] = defaultdict(list)
    for root, _dirs, files in os.walk(folder_path):
        for fname in sorted(files):
            ext = os.path.splitext(fname)[1].lower()
            if ext not in SUPPORTED_EXTENSIONS:
                continue
            full_path = os.path.join(root, fname)
            p = PureWindowsPath(full_path)
            key = (str(p.parent).lower(), p.stem.lower())
            stem_groups[key].append(full_path)
    candidates = [_pick_preferred(paths) for paths in stem_groups.values()]
    existing = {fp.lower() for (fp,) in db.query(_Design.filepath).all()}
    results: list[ScannedDesign] = []
    for full_path in sorted(candidates):
        rel_filepath = _build_managed_rel_filepath(
            folder_path, full_path, managed_root=managed_root
        )
        if rel_filepath.lower() in existing:
            continue
        fname = os.path.basename(full_path)
        # _process_file will be imported from preview.py in the next step
        from src.services.preview import _process_file

        sd = _process_file(full_path, fname, rel_filepath, db, generate_preview=False)
        sd.source_full_path = full_path
        results.append(sd)
    return results


def scan_folders(folder_paths: list[str], db: Session) -> list[ScannedDesign]:
    seen_paths: set[str] = set()
    all_results: list[ScannedDesign] = []
    clean_paths: list[str] = []
    for raw_path in folder_paths:
        folder_path = raw_path.strip()
        if not folder_path:
            continue
        norm = os.path.normcase(os.path.normpath(folder_path))
        if norm in seen_paths:
            continue
        seen_paths.add(norm)
        clean_paths.append(folder_path)
    base_counts: dict[str, int] = {}
    for folder_path in clean_paths:
        base = _selected_source_root_name(folder_path) or "folder"
        base_counts[base.lower()] = base_counts.get(base.lower(), 0) + 1
    base_seen: dict[str, int] = {}
    for folder_index, folder_path in enumerate(clean_paths):
        key = str(folder_index)
        base = _selected_source_root_name(folder_path) or f"folder_{folder_index + 1}"
        occurrence = base_seen.get(base.lower(), 0) + 1
        base_seen[base.lower()] = occurrence
        folder_root = base if occurrence == 1 else f"{base}__{occurrence}"
        folder_label = base
        if base_counts.get(base.lower(), 0) > 1:
            parent = os.path.basename(os.path.dirname(os.path.normpath(folder_path)))
            folder_label = f"{base} ({parent})" if parent else f"{base} ({occurrence})"
        try:
            results = scan_folder(folder_path, db, managed_root=folder_root)
        except TypeError as exc:
            if "managed_root" not in str(exc):
                raise
            results = scan_folder(folder_path, db)
        for sd in results:
            sd.source_folder = folder_path
            sd.folder_key = key
            sd.folder_label = folder_label
            sd.folder_root = folder_root
        all_results.extend(results)
    return all_results


def process_selected_files(
    rel_filepaths: list[str],
    source_folders: list[str] | str,
    db: Session,
    folder_root_map: dict[str, str] | None = None,
    commit_batch_size: int = 0,
    desc_to_tag: dict | None = None,
    all_designers: list | None = None,
    all_sources: list | None = None,
    base_path: str | None = None,
    preview_3d: bool = True,
) -> list[ScannedDesign]:

    if isinstance(source_folders, str):
        source_folders = [source_folders]
    existing = {fp.lower() for (fp,) in db.query(_Design.filepath).all()}
    results: list[ScannedDesign] = []
    folder_entries: list[tuple[str, str, str, str]] = []
    for i, folder in enumerate(source_folders):
        folder_key = str(i)
        folder_label = _selected_source_root_name(folder)
        folder_root = (folder_root_map or {}).get(folder_key) or folder_label
        folder_entries.append((folder_key, folder, folder_label, folder_root))
    scanned_count = 0
    for rel_filepath in rel_filepaths:
        if rel_filepath.lower() in existing:
            continue
        full_path = None
        matched_folder = None
        matched_key = None
        matched_label = None
        matched_root = None
        clean_rel = rel_filepath.lstrip("/\\")
        parts = [p for p in clean_rel.replace("/", "\\").split("\\") if p]
        first_component = parts[0].lower() if parts else ""
        candidates = [
            entry
            for entry in folder_entries
            if first_component and first_component in {entry[2].lower(), entry[3].lower()}
        ]
        if not candidates:
            candidates = folder_entries
        for folder_key, folder, folder_label, folder_root in candidates:
            candidate = _resolve_source_full_path(folder, rel_filepath, managed_root=folder_root)
            if candidate:
                full_path = candidate
                matched_folder = folder
                matched_key = folder_key
                matched_label = folder_label
                matched_root = folder_root
                break
        if not full_path or matched_folder is None or matched_key is None:
            import logging

            logger = logging.getLogger(__name__)
            logger.warning("Skipping selected file outside all source folders: %r", rel_filepath)
            continue
        fname = os.path.basename(full_path)
        from src.services.preview import _process_file

        sd = _process_file(full_path, fname, rel_filepath, db, preview_3d=preview_3d)
        sd.source_full_path = full_path

        sd.source_folder = matched_folder
        sd.folder_key = matched_key
        sd.folder_label = matched_label or _selected_source_root_name(matched_folder)
        sd.folder_root = matched_root or _selected_source_root_name(matched_folder)
        results.append(sd)
        scanned_count += 1

        # If we have the lookup tables, create and persist the Design record now
        # so DB commits happen during scanning, not just at the end.
        if commit_batch_size > 0 and desc_to_tag is not None and not sd.error:
            _persist_scanned_design(db, sd, desc_to_tag, all_designers, all_sources, base_path)
            if scanned_count % commit_batch_size == 0:
                db.commit()
                import logging

                logging.getLogger(__name__).info(
                    "process_selected_files: committed batch of %d designs (%d scanned so far)",
                    commit_batch_size,
                    scanned_count,
                )

    return results


def _persist_scanned_design(
    db: Session,
    sd: ScannedDesign,
    desc_to_tag: dict,
    all_designers: list | None = None,
    all_sources: list | None = None,
    base_path: str | None = None,
) -> None:
    """Create a Design ORM record from a ScannedDesign and add it to the session.

    Copies the source file to managed storage when ``source_full_path`` is set.
    Applies Tier 1 keyword tags and infers designer/source from path.
    Does **not** commit — caller is responsible for batching commits.
    """
    from datetime import date

    from src.services.auto_tagging import suggest_tier1
    from src.services.matching import suggest_designer_from_path, suggest_source_from_path
    from src.services.persistence import copy_design_to_managed_folder
    from src.services.tagging import _unique_tags_from_descriptions

    # Copy the source file into the managed folder if a source path is known.
    if sd.source_full_path and base_path:
        success, error = copy_design_to_managed_folder(db, sd, base_path)
        if not success:
            sd.error = error
            return

    design = _Design(
        filename=sd.filename,
        filepath=sd.filepath,
        width_mm=sd.width_mm,
        height_mm=sd.height_mm,
        stitch_count=sd.stitch_count,
        color_count=sd.color_count,
        color_change_count=sd.color_change_count,
        hoop_id=sd.hoop_id,
        image_data=sd.image_data,
        is_stitched=False,
        date_added=date.today(),
    )
    # Apply explicit per-folder overrides first; fall back to path inference
    if sd.designer_override_set:
        design.designer_id = sd.override_designer_id
    elif all_designers:
        suggested = suggest_designer_from_path(sd.filepath, all_designers)
        if suggested:
            design.designer_id = suggested.id
    if sd.source_override_set:
        design.source_id = sd.override_source_id
    elif all_sources:
        suggested_src = suggest_source_from_path(sd.filepath, all_sources)
        if suggested_src:
            design.source_id = suggested_src.id

    # Tier 1: keyword matching against filename + folder path
    valid_descriptions_set = set(desc_to_tag.keys())
    matched_descriptions = suggest_tier1(sd.filename, valid_descriptions_set, sd.filepath)

    # StitchIdentifier pattern analysis
    if sd.source_full_path:
        from src.services.auto_tagging import suggest_stitching_from_pattern

        pattern_descriptions = suggest_stitching_from_pattern(
            sd.source_full_path, sd.filename, sd.filepath, desc_to_tag
        )
        if pattern_descriptions:
            matched_descriptions = sorted(set(matched_descriptions) | set(pattern_descriptions))

    if matched_descriptions:
        design.tags = _unique_tags_from_descriptions(matched_descriptions, desc_to_tag)
        design.tagging_tier = 1
    db.add(design)
