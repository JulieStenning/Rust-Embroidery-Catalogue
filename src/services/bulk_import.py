__all__ = [
    "confirm_import",
    "find_or_create_source",
    "find_or_create_designer",
    "backfill_designer_from_path",
    "backfill_source_from_path",
    "suggest_source_from_path",
    "suggest_designer_from_path",
    "normalize_name_for_matching",
    # Preview helpers (re-exported for backward compatibility)
    "_decode_art_icon",
    "_find_spider_image",
    "_render_preview",
    "_thread_color",
    "_process_file",
    "_read_art_metadata",
    # Scanning helpers (re-exported for backward compatibility)
    "process_selected_files",
    "scan_folders",
    "scan_folder",
    "ScannedDesign",
    "SUPPORTED_EXTENSIONS",
    # Auto-tagging helpers
    "suggest_tier1",
    "suggest_tier2_batch",
    "suggest_tier3_vision",
    "suggest_stitching_from_pattern",
]
import logging
import os
from datetime import date
from pathlib import Path, PureWindowsPath

# Additional imports for re-exports
from sqlalchemy.orm import Session

from src.models import Design, Designer, Source, Tag

# Import auto-tagging helpers for backward compatibility
from src.services.auto_tagging import (
    suggest_stitching_from_pattern,
    suggest_tier1,
    suggest_tier2_batch,
    suggest_tier3_vision,
)
from src.services.matching import (
    backfill_designer_from_path as _backfill_designer_from_path,
)

# Import matching helpers for backward compatibility
from src.services.matching import (
    backfill_source_from_path as _backfill_source_from_path,
)
from src.services.matching import (
    find_or_create_designer,
    find_or_create_source,
    normalize_name_for_matching,
    suggest_designer_from_path,
    suggest_source_from_path,
)

# Import preview helpers for backward compatibility
from src.services.preview import (
    _decode_art_icon,
    _find_spider_image,
    _read_art_metadata,
    _render_preview,
    _thread_color,
)
from src.services.preview import (
    _process_file as preview_process_file,
)

# Import scanning helpers for backward compatibility
from src.services.scanning import (
    SUPPORTED_EXTENSIONS,
    ScannedDesign,
    process_selected_files,
    scan_folder,
    scan_folders,
)

# Import tagging helpers
from src.services.tagging import _unique_tags_from_descriptions

# Re-export for backward compatibility
backfill_source_from_path = _backfill_source_from_path
backfill_designer_from_path = _backfill_designer_from_path
_process_file = preview_process_file

logger = logging.getLogger(__name__)

DEFAULT_IMPORT_COMMIT_BATCH_SIZE = 1000


def _coerce_batch_size(value: int | None, default: int = DEFAULT_IMPORT_COMMIT_BATCH_SIZE) -> int:
    """Return a safe positive batch size."""
    if value is None:
        return default
    try:
        parsed = int(value)
    except (TypeError, ValueError):
        return default
    return parsed if parsed > 0 else default


def _load_api_key() -> str | None:
    """Load GOOGLE_API_KEY from .env file or environment."""
    env_path = Path(__file__).parent.parent.parent / ".env"
    if env_path.exists():
        try:
            for line in env_path.read_text().splitlines():
                if line.startswith("GOOGLE_API_KEY="):
                    return line.split("=", 1)[1].strip()
        except OSError:
            pass
    return os.environ.get("GOOGLE_API_KEY")


def _build_managed_rel_filepath(
    source_folder: str,
    full_path: str,
    managed_root: str | None = None,
) -> str:
    """Return the DB filepath to store for a scanned design.

    The selected source folder itself is preserved as the top-level folder inside
    the managed catalogue so importing `J:\\MachineEmbroideryDesigns\\Craftsy...`
    stores files under `data\\MachineEmbroideryDesigns\\Craftsy...`.

    ``managed_root`` defaults to the selected folder's basename, but callers may
    provide a unique alias (for example ``"designs__2"``) when multiple selected
    folders share the same leaf name.
    """
    try:
        rel = os.path.relpath(full_path, source_folder)
    except ValueError:
        rel = os.path.basename(full_path)
    source_root = managed_root or _selected_source_root_name(source_folder)
    rel_parts = [part for part in str(PureWindowsPath(rel)).split("\\") if part]
    parts = ([source_root] if source_root else []) + rel_parts
    return "\\" + "\\".join(parts)


def _selected_source_root_name(folder_path: str) -> str:
    """Extract the normalized root name from a folder path."""
    normalized = str(PureWindowsPath(folder_path.replace("/", "\\"))).rstrip("\\")
    return PureWindowsPath(normalized).name


# ---------------------------------------------------------------------------
# Persistence and tagging orchestration
# ---------------------------------------------------------------------------


def _build_design_records(
    db: Session,
    scanned_designs: list[ScannedDesign],
    desc_to_tag: dict[str, Tag],
    all_designers: list[Designer],
    all_sources: list[Source],
    commit_batch_size: int = DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
) -> list[Design]:
    """Create Design ORM objects from valid scanned entries and apply Tier 1 tags.

    When a ``ScannedDesign`` has a ``source_full_path`` set, the file is copied
    into the managed base folder (preserving the relative subfolder structure)
    before the record is persisted.  This ensures that once imported, designs
    are accessible regardless of whether the original source folder still exists.

    Args:
        desc_to_tag: pre-built mapping of tag description -> Tag, shared with the
            AI tagging helpers to avoid repeated database queries.
        all_designers / all_sources: pre-loaded lists for path-based inference.

    Persists all created records in batched commits and returns them.

    Designs whose ``filepath`` already exists in the database are skipped — this
    allows the function to be safely called after designs have already been
    persisted during the scanning phase (interleaved persistence).
    """
    from src.services.persistence import copy_design_to_managed_folder
    from src.services.settings_service import get_designs_base_path

    base_path = get_designs_base_path(db)
    valid_descriptions_set = set(desc_to_tag.keys())

    # Build a set of filepaths already in the DB so we can skip designs that
    # were already persisted during the scanning phase.
    existing_filepaths: set[str] = {fp.lower() for (fp,) in db.query(Design.filepath).all()}

    batch_size = _coerce_batch_size(commit_batch_size)
    created: list[Design] = []
    pending_in_batch = 0
    for sd in scanned_designs:
        if sd.error:
            continue

        # Skip designs already persisted during the scanning phase.
        if sd.filepath.lower() in existing_filepaths:
            # Still need to return the Design record for AI tagging.
            existing_design = db.query(Design).filter(Design.filepath == sd.filepath).first()
            if existing_design:
                created.append(existing_design)
            continue

        # Copy the source file into the managed folder if a source path is known.
        if sd.source_full_path:
            success, error = copy_design_to_managed_folder(db, sd, base_path)
            if not success:
                sd.error = error
                continue

        design = Design(
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
        else:
            suggested = suggest_designer_from_path(sd.filepath, all_designers)
            if suggested:
                design.designer_id = suggested.id
        if sd.source_override_set:
            design.source_id = sd.override_source_id
        else:
            suggested_src = suggest_source_from_path(sd.filepath, all_sources)
            if suggested_src:
                design.source_id = suggested_src.id
        # Tier 1: keyword matching against filename + folder path
        matched_descriptions = suggest_tier1(sd.filename, valid_descriptions_set, sd.filepath)

        # StitchIdentifier pattern analysis examines the actual stitch geometry
        # (vectors, angles, densities) for accurate stitch-type detection.
        if sd.source_full_path:
            pattern_descriptions = suggest_stitching_from_pattern(
                sd.source_full_path, sd.filename, sd.filepath, desc_to_tag
            )
            if pattern_descriptions:
                matched_descriptions = sorted(set(matched_descriptions) | set(pattern_descriptions))

        if matched_descriptions:
            design.tags = _unique_tags_from_descriptions(matched_descriptions, desc_to_tag)
            design.tagging_tier = 1
        db.add(design)
        created.append(design)
        pending_in_batch += 1

        if pending_in_batch >= batch_size:
            db.commit()
            logger.info(
                "_build_design_records: committed batch of %d designs (%d total so far)",
                batch_size,
                len(created),
            )
            pending_in_batch = 0

    if pending_in_batch:
        db.commit()
        logger.info(
            "_build_design_records: committed final batch of %d designs (%d total)",
            pending_in_batch,
            len(created),
        )
    return created


def _apply_tier2_tags(
    db: Session,
    created: list[Design],
    desc_to_tag: dict[str, Tag],
    valid_descriptions_list: list[str],
    api_key: str,
    commit_batch_size: int = DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
) -> None:
    """Run Gemini filename AI (Tier 2) on designs still untagged after Tier 1.

    Mutates the Design objects in-place and commits when any tags are assigned.
    """
    needs_tier2 = [d for d in created if not d.tags]
    if not needs_tier2:
        return
    logger.info("Tier 2: Gemini filename AI on %d untagged designs", len(needs_tier2))
    batch_size = _coerce_batch_size(commit_batch_size)

    for start in range(0, len(needs_tier2), batch_size):
        batch = needs_tier2[start : start + batch_size]
        filenames = [d.filename for d in batch]
        tier2_results = suggest_tier2_batch(filenames, valid_descriptions_list, api_key)
        updated = 0
        for design in batch:
            stem = Path(design.filename).stem.lower()
            descriptions = tier2_results.get(stem, [])
            if descriptions:
                design.tags = _unique_tags_from_descriptions(descriptions, desc_to_tag)
                design.tagging_tier = 2
                updated += 1
        if updated:
            db.commit()


def _apply_tier3_tags(
    db: Session,
    created: list[Design],
    desc_to_tag: dict[str, Tag],
    valid_descriptions_list: list[str],
    api_key: str,
    commit_batch_size: int = DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
) -> None:
    """Run Gemini vision AI (Tier 3) on designs still untagged after Tiers 1+2.

    Only designs that have stored image data are eligible.
    Mutates the Design objects in-place and commits when any tags are assigned.
    """
    needs_tier3 = [d for d in created if not d.tags and d.image_data is not None]
    if not needs_tier3:
        return
    logger.info("Tier 3: vision AI on %d still-untagged designs", len(needs_tier3))
    batch_size = _coerce_batch_size(commit_batch_size)

    for start in range(0, len(needs_tier3), batch_size):
        batch = needs_tier3[start : start + batch_size]
        tier3_results = suggest_tier3_vision(batch, valid_descriptions_list, api_key)
        updated = 0
        for design in batch:
            descriptions = tier3_results.get(design.id, [])
            if descriptions:
                design.tags = _unique_tags_from_descriptions(descriptions, desc_to_tag)
                design.tagging_tier = 3
                updated += 1
        if updated:
            db.commit()


def _resolve_designer_choice(
    db: Session,
    choice: str | None,
    designer_id: str | int | None,
    designer_name: str | None,
    cache: dict[str, int | None],
) -> tuple[int | None, bool]:
    """Resolve a designer choice to ``(designer_id_or_none, override_set)``.

    Returns ``(id, True)`` when a concrete assignment should be made (including
    an explicit blank), or ``(None, False)`` when path inference should be used.
    *cache* maps normalized names to already-resolved IDs to avoid duplicate
    ``find_or_create_designer`` calls within a single import.
    """
    if choice == "existing" and designer_id:
        try:
            return int(designer_id), True
        except (ValueError, TypeError):
            return None, False
    if choice == "create" and designer_name and designer_name.strip():
        norm = normalize_name_for_matching(designer_name)
        if norm not in cache:
            try:
                cache[norm] = find_or_create_designer(db, designer_name).id
            except Exception:  # noqa: BLE001
                cache[norm] = None
        return cache[norm], True
    if choice == "blank":
        return None, True
    # "inferred" or absent → let path inference handle it
    return None, False


def _resolve_source_choice(
    db: Session,
    choice: str | None,
    source_id: str | int | None,
    source_name: str | None,
    cache: dict[str, int | None],
) -> tuple[int | None, bool]:
    """Resolve a source choice to ``(source_id_or_none, override_set)``.

    Mirrors :func:`_resolve_designer_choice`.
    """
    if choice == "existing" and source_id:
        try:
            return int(source_id), True
        except (ValueError, TypeError):
            return None, False
    if choice == "create" and source_name and source_name.strip():
        norm = normalize_name_for_matching(source_name)
        if norm not in cache:
            try:
                cache[norm] = find_or_create_source(db, source_name).id
            except Exception:  # noqa: BLE001
                cache[norm] = None
        return cache[norm], True
    if choice == "blank":
        return None, True
    return None, False


def confirm_import(
    db: Session,
    scanned_designs: list[ScannedDesign],
    folder_choices: dict[str, dict] | None = None,
    global_choice: dict | None = None,
    run_tier2: bool = True,
    run_tier3: bool = True,
    batch_limit: int | None = None,
    commit_batch_size: int = DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
    desc_to_tag: dict[str, Tag] | None = None,
    all_designers: list[Designer] | None = None,
    all_sources: list[Source] | None = None,
    base_path: str | None = None,
    preview_3d: bool = True,
) -> list[Design]:
    """
    Persist scanned designs and orchestrate all auto-tagging tiers.

    Workflow:
        1. Apply per-folder and global designer/source overrides.
        2. Create Design records; apply Tier 1 (keyword) tags; infer designer/source.
        3. Optionally run Tier 2 (Gemini filename AI) on still-untagged designs.
        4. Optionally run Tier 3 (Gemini vision AI) on designs still untagged with stored image data.

    *folder_choices* maps ``folder_key`` (string index from the review form) to a dict with keys ``designer_choice``, ``designer_id``, ``designer_name``, ``source_choice``, ``source_id``, ``source_name``. Values for each key are ``"inferred"``, ``"existing"``, ``"create"``, or ``"blank"``.

    *global_choice* has the same structure and is applied to any folder that does not have an explicit per-folder choice (or whose per-folder choice is ``"inferred"``).

    Assignment precedence:
        1. Explicit per-folder choice
        2. Global choice
        3. Existing path inference
        4. Blank / null

    Returns the list of created Design records (excludes entries with errors).

    When ``desc_to_tag``, ``all_designers``, ``all_sources``, and ``base_path`` are
    provided (pre-loaded by the caller), they are reused instead of querying the
    database again.  This allows designs that were already persisted during the
    scanning phase to be detected and skipped.
    """
    # Load reference data once (or reuse pre-loaded data from caller)
    if desc_to_tag is None:
        all_tags: list[Tag] = db.query(Tag).all()
        desc_to_tag = {tag.description: tag for tag in all_tags}
    if all_designers is None:
        all_designers = db.query(Designer).all()
    if all_sources is None:
        all_sources = db.query(Source).all()

    valid_descriptions_list = sorted(desc_to_tag.keys())

    # Caches shared across all create-on-import calls
    designer_cache: dict[str, int | None] = {}
    source_cache: dict[str, int | None] = {}

    # Pre-resolve global choices (if any)
    global_fc = global_choice or {}
    global_designer_id, global_designer_set = _resolve_designer_choice(
        db,
        global_fc.get("designer_choice"),
        global_fc.get("designer_id"),
        global_fc.get("designer_name"),
        designer_cache,
    )
    global_source_id, global_source_set = _resolve_source_choice(
        db,
        global_fc.get("source_choice"),
        global_fc.get("source_id"),
        global_fc.get("source_name"),
        source_cache,
    )

    # Apply overrides to each ScannedDesign before persistence
    for sd in scanned_designs:
        if sd.designer_override_set or sd.source_override_set:
            # Already resolved externally — respect as-is
            pass
        else:
            folder_key = sd.folder_key or ""
            per_folder = (folder_choices or {}).get(folder_key, {})

            # Designer
            d_choice = per_folder.get("designer_choice")
            if d_choice and d_choice != "inferred":
                did, dset = _resolve_designer_choice(
                    db,
                    d_choice,
                    per_folder.get("designer_id"),
                    per_folder.get("designer_name"),
                    designer_cache,
                )
            elif global_designer_set:
                did, dset = global_designer_id, True
            else:
                did, dset = None, False
            sd.override_designer_id = did
            sd.designer_override_set = dset

            # Source
            s_choice = per_folder.get("source_choice")
            if s_choice and s_choice != "inferred":
                sid, sset = _resolve_source_choice(
                    db,
                    s_choice,
                    per_folder.get("source_id"),
                    per_folder.get("source_name"),
                    source_cache,
                )
            elif global_source_set:
                sid, sset = global_source_id, True
            else:
                sid, sset = None, False
            sd.override_source_id = sid
            sd.source_override_set = sset

    # Step 1: persist records + Tier 1 tagging
    created = _build_design_records(
        db,
        scanned_designs,
        desc_to_tag,
        all_designers,
        all_sources,
        commit_batch_size=commit_batch_size,
    )

    # Steps 2 & 3: AI-assisted tagging (skipped when no API key is configured)
    if (run_tier2 or run_tier3) and created:
        api_key = _load_api_key()
        if api_key:
            # Apply batch limit if specified
            ai_candidates = (
                created[:batch_limit] if batch_limit is not None and batch_limit > 0 else created
            )
            if run_tier2:
                _apply_tier2_tags(
                    db,
                    ai_candidates,
                    desc_to_tag,
                    valid_descriptions_list,
                    api_key,
                    commit_batch_size=commit_batch_size,
                )
            if run_tier3:
                _apply_tier3_tags(
                    db,
                    ai_candidates,
                    desc_to_tag,
                    valid_descriptions_list,
                    api_key,
                    commit_batch_size=commit_batch_size,
                )
        else:
            logger.info("AI tagging skipped: no GOOGLE_API_KEY found.")

    return created
