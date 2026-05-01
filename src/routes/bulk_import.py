"""
Bulk import wizard routes.

Step 1  GET  /import/          → form to enter one or more folder paths
Step 2  POST /import/scan      → scan folders, show scanned file list grouped by folder
Step 3  POST /import/precheck  → pre-import tag decision (first vs later import)
Step 4  POST /import/confirm   → persist confirmed designs
"""

from __future__ import annotations

import logging
import os
import re
import uuid

from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import HTMLResponse, JSONResponse, RedirectResponse
from sqlalchemy.orm import Session

import src.services.scanning as scanning_mod
from src.database import get_db
from src.models import Design, Hoop
from src.services import bulk_import as svc
from src.services import designers as des_svc
from src.services import settings_service as settings_svc
from src.services import sources as src_svc
from src.services import validation as val
from src.services.folder_picker import (
    FolderPickerUnavailableError,
    display_path,
    pick_folders,
    resolve_picker_initial_dir,
)
from src.services.scanning import scan_folders
from src.templating import templates

log = logging.getLogger(__name__)


def _parse_optional_positive_int(raw_value: str | None) -> int | None:
    if raw_value is None:
        return None
    value = raw_value.strip()
    if not value:
        return None
    try:
        parsed = int(value)
    except ValueError:
        return None
    return parsed if parsed > 0 else None


# ---------------------------------------------------------------------------
# In-memory import-context store (keyed by a random token).
# Used to preserve the selected files and folder paths while the user reviews
# tags between steps 3 and 4 of the import wizard.
# NOTE: This is intentionally a module-level dict — the app runs as a single
# local process for one user, so no cross-worker persistence is needed.
# ---------------------------------------------------------------------------
_IMPORT_CONTEXT: dict[str, dict] = {}

_UUID_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")


def _is_valid_token(token: str) -> bool:
    """Return True if *token* is a well-formed UUID v4 string."""
    return bool(_UUID_RE.match(token or ""))


def _store_import_context(data: dict) -> str:
    """Store import context and return an opaque token."""
    token = str(uuid.uuid4())
    _IMPORT_CONTEXT[token] = data
    return token


def _pop_import_context(token: str) -> dict | None:
    """Retrieve and remove import context by token, or return None."""
    if not _is_valid_token(token):
        return None
    return _IMPORT_CONTEXT.pop(token, None)


def _get_import_context(token: str) -> dict | None:
    """Retrieve import context by token without removing it."""
    if not _is_valid_token(token):
        return None
    return _IMPORT_CONTEXT.get(token)


router = APIRouter(prefix="/import", tags=["bulk_import"])


@router.get("/browse-folder", response_class=JSONResponse)
def browse_folder(start_dir: str = "", db: Session = Depends(get_db)):
    """Open a native folder picker and return one or more selected folders."""
    saved_dir = settings_svc.get_setting(db, settings_svc.SETTING_LAST_IMPORT_BROWSE_FOLDER)
    requested_dir = (start_dir or "").strip()
    if requested_dir:
        initial = resolve_picker_initial_dir(requested_dir)
    else:
        initial = resolve_picker_initial_dir(saved_dir, prefer_parent=True)
    try:
        raw_paths = pick_folders(
            start_dir=initial,
            title="Select embroidery design folder(s)",
        )
        paths = [display_path(path) for path in raw_paths]
        if raw_paths:
            settings_svc.set_setting(
                db,
                settings_svc.SETTING_LAST_IMPORT_BROWSE_FOLDER,
                raw_paths[0],
            )
        return JSONResponse(
            {
                "paths": paths,
                "path": paths[0] if paths else "",
            }
        )
    except FolderPickerUnavailableError as exc:
        return JSONResponse(
            {
                "error": f"Folder picker is not available on this system. Please enter the path manually. ({exc})"
            },
            status_code=200,
        )


@router.get("/", response_class=HTMLResponse)
def import_form(request: Request):
    return templates.TemplateResponse(
        request,
        "import/step1_folder.html",
        {},
    )


@router.post("/scan", response_class=HTMLResponse)
async def scan(request: Request, db: Session = Depends(get_db)):
    form = await request.form()

    # Accept both 'folder_paths' (multi-folder) and 'folder_path' (legacy single-folder)
    folder_paths: list[str] = list(form.getlist("folder_paths")) or list(
        form.getlist("folder_paths[]")
    )
    if not folder_paths:
        single = form.get("folder_path", "")
        if single and single.strip():
            folder_paths = [single.strip()]

    # Deduplicate, normalize, and validate folder paths
    seen: set[str] = set()
    clean_paths: list[str] = []
    errors: list[str] = []
    for p in folder_paths:
        p = p.strip()
        if not p:
            continue
        norm = val.normalize_path(p)
        if norm not in seen:
            seen.add(norm)
            try:
                val.validate_path_exists(norm, "Import folder")
                val.validate_is_directory(norm, "Import folder")
                clean_paths.append(norm)
            except Exception as exc:
                errors.append(val.user_friendly_path_error(exc, context=p))

    if not clean_paths:
        detail = ", ".join(errors) if errors else "At least one valid folder path is required."
        raise HTTPException(status_code=400, detail=detail)

    if errors:
        log.warning("Some import folders were invalid: %r", errors)

    log.info("SCAN requested: folder_paths=%r", clean_paths)
    scanned = scan_folders(clean_paths, db)
    ok = sum(1 for s in scanned if not s.error)
    err = sum(1 for s in scanned if s.error)
    log.info("SCAN complete: %d files found (%d OK, %d errors)", len(scanned), ok, err)
    for s in scanned:
        log.debug("  scanned: filepath=%r error=%r", s.filepath, s.error)

    # Group scanned designs by folder_key for the review template
    folder_groups: list[dict] = []
    key_to_group: dict[str, dict] = {}
    for sd in scanned:
        key = sd.folder_key or "0"
        if key not in key_to_group:
            group: dict = {
                "folder_key": key,
                "folder_path": sd.source_folder or clean_paths[0],
                "folder_label": sd.folder_label
                or os.path.basename(sd.source_folder or clean_paths[0]),
                "folder_root": sd.folder_root
                or os.path.basename(os.path.normpath(sd.source_folder or clean_paths[0])),
                "designs": [],
            }
            key_to_group[key] = group
            folder_groups.append(group)
        key_to_group[key]["designs"].append(sd)

    # If scan_folders returned nothing (all duplicates), ensure a group per path still appears
    for i, path in enumerate(clean_paths):
        key = str(i)
        if key not in key_to_group:
            group = {
                "folder_key": key,
                "folder_path": path,
                "folder_label": os.path.basename(os.path.normpath(path)),
                "folder_root": os.path.basename(os.path.normpath(path)),
                "designs": [],
            }
            key_to_group[key] = group
            folder_groups.append(group)

    all_designers = des_svc.get_all(db)
    all_sources = src_svc.get_all(db)

    # Warn if .art files are present but their parent folders have no Embird
    # Spider subfolder (named with 'spider' or 'embird').  We check each
    # unique parent directory at most once.
    warn_art_no_spider = False
    checked_spider_dirs: set[str] = set()
    for _sd in scanned:
        if os.path.splitext(_sd.filename)[1].lower() != ".art":
            continue
        _parent = os.path.dirname(_sd.source_full_path or "") or ""
        if not _parent or _parent in checked_spider_dirs:
            continue
        checked_spider_dirs.add(_parent)
        try:
            _has_spider = any(
                ("spider" in _d.lower() or "embird" in _d.lower())
                for _d in os.listdir(_parent)
                if os.path.isdir(os.path.join(_parent, _d))
            )
        except OSError:
            _has_spider = False
        if not _has_spider:
            warn_art_no_spider = True
            break

    return templates.TemplateResponse(
        request,
        "import/step2_review.html",
        {
            "scanned": scanned,
            "folder_paths": clean_paths,
            "folder_groups": folder_groups,
            "all_designers": all_designers,
            "all_sources": all_sources,
            "warn_art_no_spider": warn_art_no_spider,
        },
    )


@router.post("/precheck", response_class=HTMLResponse)
async def precheck(request: Request, db: Session = Depends(get_db)):
    """Step 3 — decide whether to review tags before importing.

    For the first-ever import (no designs in the catalogue) this step is
    mandatory: the user is shown an explanation and must review tags before
    continuing.  For subsequent imports the user is given a choice.
    """
    form = await request.form()

    # Collect the full form context from the review step so we can pass it
    # on to the confirm step later.
    folder_paths: list[str] = list(form.getlist("folder_paths"))
    if not folder_paths:
        single = form.get("folder_path", "")
        if single and single.strip():
            folder_paths = [single.strip()]
    folder_paths = [p.strip() for p in folder_paths if p.strip()]

    selected_files: list[str] = list(form.getlist("selected_files"))

    if not selected_files:
        log.warning("PRECHECK: no files selected — redirecting back to import form")
        return RedirectResponse("/import/", status_code=303)

    # Collect the rest of the form fields (designer/source choices, folder
    # roots, etc.) so that they survive the tag-review detour.
    extra: dict[str, list[str]] = {}
    for key in form:
        if key in ("folder_paths", "selected_files"):
            continue
        extra[key] = list(form.getlist(key))

    # Detect first vs subsequent import.
    is_first_import = db.query(Design).count() == 0
    needs_hoop_setup = is_first_import and db.query(Hoop).count() == 0

    # Store context and generate a token for retrieval after any review step.
    import_context = {
        "folder_paths": folder_paths,
        "selected_files": selected_files,
        "extra": extra,
        "is_first_import": is_first_import,
        "needs_hoop_setup": needs_hoop_setup,
    }
    token = _store_import_context(import_context)

    # Load AI tagging settings for the banner and import flow
    has_api_key = bool(settings_svc.get_google_api_key())
    ai_tier2_auto = settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER2_AUTO)
    )
    ai_tier3_auto = settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER3_AUTO)
    )
    ai_batch_size_raw = settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE)
    ai_batch_size = _parse_optional_positive_int(ai_batch_size_raw)
    import_commit_batch_size_raw = settings_svc.get_setting(
        db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE
    )
    import_commit_batch_size = _parse_optional_positive_int(import_commit_batch_size_raw)

    # Read image preference setting (2d or 3d)
    image_preference = settings_svc.get_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE)

    log.info(
        "PRECHECK: first_import=%s, folder_paths=%r, selected_files=%d",
        is_first_import,
        folder_paths,
        len(selected_files),
    )

    return templates.TemplateResponse(
        request,
        "import/step3_precheck.html",
        {
            "import_token": token,
            "is_first_import": is_first_import,
            "needs_hoop_setup": needs_hoop_setup,
            "has_api_key": has_api_key,
            "ai_tier2_auto": ai_tier2_auto,
            "ai_tier3_auto": ai_tier3_auto,
            "ai_batch_size": ai_batch_size,
            "import_commit_batch_size": import_commit_batch_size,
            "selected_file_count": len(selected_files),
            "ai_tagging_guide_url": "/about/document/ai-tagging",
            "image_preference": image_preference,
        },
    )


@router.post("/precheck-action", response_class=HTMLResponse)
async def precheck_action(request: Request, db: Session = Depends(get_db)):
    """Handle the user's decision from the precheck page.

    ``action`` can be one of:
    - ``review_tags``  → open the tag-management page in import mode
    - ``import_now``   → skip tag review and proceed directly to import
    - ``cancel``       → abort and return to the import form
    """
    form = await request.form()
    action = form.get("action", "cancel")
    token = str(form.get("import_token", ""))

    if action == "cancel" or not token:
        return RedirectResponse("/import/", status_code=303)

    context = _get_import_context(token)
    if context is None:
        log.warning("PRECHECK-ACTION: unknown or expired token — redirecting to /import/")
        return RedirectResponse("/import/", status_code=303)

    if action == "import_now":
        if context.get("needs_hoop_setup") and form.get("confirm_skip_hoops") != "yes":
            return templates.TemplateResponse(
                request,
                "import/step3_confirm_skip_hoops.html",
                {"import_token": token},
            )

        # Capture the image_preference override from the precheck form
        image_preference = str(form.get("image_preference", "")).strip()
        if image_preference in ("2d", "3d"):
            context["image_preference"] = image_preference

        # Run the import directly without an intermediate redirect so the
        # token is never exposed in a URL.
        context = _pop_import_context(token)
        return await _run_confirm(db, context)

    if action == "review_hoops":
        from src.services import hoops as hoops_svc  # local import to avoid circular deps

        hoop_items = hoops_svc.get_all(db)
        return templates.TemplateResponse(
            request,
            "admin/hoops.html",
            {
                "hoops": hoop_items,
                "import_token": token,
            },
        )

    if action == "review_sources":
        items = src_svc.get_all(db)
        return templates.TemplateResponse(
            request,
            "admin/sources.html",
            {
                "sources": items,
                "import_token": token,
            },
        )

    if action == "review_designers":
        items = des_svc.get_all(db)
        return templates.TemplateResponse(
            request,
            "admin/designers.html",
            {
                "designers": items,
                "import_token": token,
            },
        )

    # action == "review_tags"
    from src.services import tags as tags_svc  # local import to avoid circular deps

    tag_items = tags_svc.get_all(db)
    return templates.TemplateResponse(
        request,
        "admin/tags.html",
        {
            "tags": tag_items,
            "import_token": token,
        },
    )


@router.post("/do-confirm", response_class=RedirectResponse)
async def do_confirm_from_token(
    request: Request,
    import_token: str = Form(default=""),
    db: Session = Depends(get_db),
):
    """POST endpoint that retrieves import context by token and runs the import.

    The token is submitted in the request body (not in the URL) so it is not
    exposed in browser history or server access logs.
    """
    context = _pop_import_context(import_token)
    if not context:
        log.warning("DO-CONFIRM: unknown or invalid token — redirecting to /import/")
        return RedirectResponse("/import/", status_code=303)
    return await _run_confirm(db, context)


async def _run_confirm(db: Session, context: dict) -> RedirectResponse:
    """Run the import using context saved from the precheck step."""
    folder_paths: list[str] = context.get("folder_paths", [])
    selected_files: list[str] = context.get("selected_files", [])
    extra: dict[str, list[str]] = context.get("extra", {})

    log.info(
        "CONFIRM (from token): folder_paths=%r, selected_files count=%d",
        folder_paths,
        len(selected_files),
    )

    if not selected_files:
        log.warning("CONFIRM: no files in context — redirecting back to import form")
        return RedirectResponse("/import/", status_code=303)

    # Reconstruct folder choices and root map from the saved extra fields.
    folder_choices: dict[str, dict] = {}
    folder_root_map: dict[str, str] = {}
    for key, values in extra.items():
        raw_value = values[0] if values else ""
        for prefix, field_suffix in (
            ("designer_choice_", "designer_choice"),
            ("designer_id_", "designer_id"),
            ("designer_name_", "designer_name"),
            ("source_choice_", "source_choice"),
            ("source_id_", "source_id"),
            ("source_name_", "source_name"),
            ("folder_root_", "folder_root"),
        ):
            if key.startswith(prefix):
                folder_key = key[len(prefix) :]
                if field_suffix == "folder_root":
                    folder_root_map[folder_key] = raw_value
                else:
                    folder_choices.setdefault(folder_key, {})[field_suffix] = raw_value
                break

    global_choice: dict = {
        "designer_choice": (extra.get("global_designer_choice") or ["inferred"])[0],
        "designer_id": (extra.get("global_designer_id") or [""])[0],
        "designer_name": (extra.get("global_designer_name") or [""])[0],
        "source_choice": (extra.get("global_source_choice") or ["inferred"])[0],
        "source_id": (extra.get("global_source_id") or [""])[0],
        "source_name": (extra.get("global_source_name") or [""])[0],
    }

    # Read AI tagging settings
    has_api_key = bool(settings_svc.get_google_api_key())
    run_tier2 = has_api_key and settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER2_AUTO)
    )
    run_tier3 = has_api_key and settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER3_AUTO)
    )
    ai_batch_size_raw = settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE)
    batch_limit = _parse_optional_positive_int(ai_batch_size_raw)
    import_commit_batch_size_raw = settings_svc.get_setting(
        db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE
    )
    import_commit_batch_size = _parse_optional_positive_int(import_commit_batch_size_raw)

    batch_size = import_commit_batch_size or svc.DEFAULT_IMPORT_COMMIT_BATCH_SIZE

    # Pre-load reference data so process_selected_files can persist designs
    # during the scanning loop (interleaving preview rendering with DB commits).
    from src.models import Designer, Source, Tag

    all_tags: list[Tag] = db.query(Tag).all()
    all_designers: list[Designer] = db.query(Designer).all()
    all_sources: list[Source] = db.query(Source).all()
    desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in all_tags}
    from src.services.settings_service import get_designs_base_path

    base_path = get_designs_base_path(db)

    # Read image_preference from context (set during precheck-action)
    image_preference = context.get("image_preference", "")
    preview_3d = image_preference != "2d"  # Default to True (3D) unless explicitly "2d"

    if folder_root_map:
        to_import = scanning_mod.process_selected_files(
            selected_files,
            folder_paths,
            db,
            folder_root_map=folder_root_map,
            commit_batch_size=batch_size,
            desc_to_tag=desc_to_tag,
            all_designers=all_designers,
            all_sources=all_sources,
            base_path=base_path,
            preview_3d=preview_3d,
        )
    else:
        to_import = scanning_mod.process_selected_files(
            selected_files,
            folder_paths,
            db,
            commit_batch_size=batch_size,
            desc_to_tag=desc_to_tag,
            all_designers=all_designers,
            all_sources=all_sources,
            base_path=base_path,
            preview_3d=preview_3d,
        )

    log.info(
        "CONFIRM: %d files to process (%d already in DB)",
        len(to_import),
        len(selected_files) - len(to_import),
    )

    created = svc.confirm_import(
        db,
        to_import,
        folder_choices=folder_choices,
        global_choice=global_choice,
        run_tier2=run_tier2,
        run_tier3=run_tier3,
        batch_limit=batch_limit,
        commit_batch_size=batch_size,
        # Pass pre-loaded data so _build_design_records can skip already-persisted designs
        desc_to_tag=desc_to_tag,
        all_designers=all_designers,
        all_sources=all_sources,
        base_path=base_path,
        preview_3d=preview_3d,
    )

    log.info("CONFIRM: %d designs created in DB", len(created))
    return RedirectResponse("/designs/", status_code=303)


@router.post("/confirm", response_class=RedirectResponse)
async def confirm(request: Request, db: Session = Depends(get_db)):
    form = await request.form()

    # Collect folder paths (support both multi and legacy single)
    folder_paths: list[str] = list(form.getlist("folder_paths")) or list(
        form.getlist("folder_paths[]")
    )
    if not folder_paths:
        single = form.get("folder_path", "")
        if single and single.strip():
            folder_paths = [single.strip()]
    folder_paths = [p.strip() for p in folder_paths if p.strip()]

    selected_files: list[str] = list(form.getlist("selected_files"))

    log.info(
        "CONFIRM requested: folder_paths=%r, selected_files count=%d",
        folder_paths,
        len(selected_files),
    )
    for f in selected_files:
        log.debug("  selected: %r", f)

    if not selected_files:
        log.warning("CONFIRM: no files selected — redirecting back to import form")
        return RedirectResponse("/import/", status_code=303)

    # Parse per-folder designer/source choices from form fields
    # Field naming: designer_choice_{folder_key}, designer_id_{folder_key}, designer_name_{folder_key}
    #               source_choice_{folder_key},   source_id_{folder_key},   source_name_{folder_key}
    folder_choices: dict[str, dict] = {}
    folder_root_map: dict[str, str] = {}
    for key in form:
        for prefix, field_suffix in (
            ("designer_choice_", "designer_choice"),
            ("designer_id_", "designer_id"),
            ("designer_name_", "designer_name"),
            ("source_choice_", "source_choice"),
            ("source_id_", "source_id"),
            ("source_name_", "source_name"),
            ("folder_root_", "folder_root"),
        ):
            if key.startswith(prefix):
                folder_key = key[len(prefix) :]
                if field_suffix == "folder_root":
                    folder_root_map[folder_key] = str(form[key])
                else:
                    folder_choices.setdefault(folder_key, {})[field_suffix] = form[key]
                break

    # Global designer/source choice (applies to all folders not explicitly overridden)
    global_choice: dict = {
        "designer_choice": form.get("global_designer_choice", "inferred"),
        "designer_id": form.get("global_designer_id", ""),
        "designer_name": form.get("global_designer_name", ""),
        "source_choice": form.get("global_source_choice", "inferred"),
        "source_id": form.get("global_source_id", ""),
        "source_name": form.get("global_source_name", ""),
    }

    # Read AI tagging settings
    has_api_key = bool(settings_svc.get_google_api_key())
    run_tier2 = has_api_key and settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER2_AUTO)
    )
    run_tier3 = has_api_key and settings_svc._is_truthy(
        settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER3_AUTO)
    )
    ai_batch_size_raw = settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE)
    batch_limit = _parse_optional_positive_int(ai_batch_size_raw)
    import_commit_batch_size_raw = settings_svc.get_setting(
        db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE
    )
    import_commit_batch_size = _parse_optional_positive_int(import_commit_batch_size_raw)

    batch_size = import_commit_batch_size or svc.DEFAULT_IMPORT_COMMIT_BATCH_SIZE

    # Pre-load reference data so process_selected_files can persist designs
    # during the scanning loop (interleaving preview rendering with DB commits).
    from src.models import Designer, Source, Tag

    all_tags: list[Tag] = db.query(Tag).all()
    all_designers: list[Designer] = db.query(Designer).all()
    all_sources: list[Source] = db.query(Source).all()
    desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in all_tags}
    from src.services.settings_service import get_designs_base_path

    base_path = get_designs_base_path(db)

    # Read image_preference from settings (legacy confirm route — no precheck override)
    image_preference = settings_svc.get_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE)
    preview_3d = image_preference != "2d"  # Default to True (3D) unless explicitly "2d"

    # Process exactly the selected files — no re-scan needed
    if folder_root_map:
        to_import = scanning_mod.process_selected_files(
            selected_files,
            folder_paths,
            db,
            folder_root_map=folder_root_map,
            commit_batch_size=batch_size,
            desc_to_tag=desc_to_tag,
            all_designers=all_designers,
            all_sources=all_sources,
            base_path=base_path,
            preview_3d=preview_3d,
        )
    else:
        to_import = scanning_mod.process_selected_files(
            selected_files,
            folder_paths,
            db,
            commit_batch_size=batch_size,
            desc_to_tag=desc_to_tag,
            all_designers=all_designers,
            all_sources=all_sources,
            base_path=base_path,
            preview_3d=preview_3d,
        )
    log.info(
        "CONFIRM: %d files to process (%d already in DB)",
        len(to_import),
        len(selected_files) - len(to_import),
    )

    created = svc.confirm_import(
        db,
        to_import,
        folder_choices=folder_choices,
        global_choice=global_choice,
        run_tier2=run_tier2,
        run_tier3=run_tier3,
        batch_limit=batch_limit,
        commit_batch_size=batch_size,
        desc_to_tag=desc_to_tag,
        all_designers=all_designers,
        all_sources=all_sources,
        base_path=base_path,
        preview_3d=preview_3d,
    )

    log.info("CONFIRM: %d designs created in DB", len(created))
    return RedirectResponse("/designs/", status_code=303)
