"""
Persistence and file copy logic for design import.
"""

# Functions will be moved here from bulk_import.py

import logging
import os
import shutil

from src.services.settings_service import get_designs_base_path


def copy_design_to_managed_folder(db, sd, base_path=None):
    """Copy the source file into the managed folder if a source path is known.
    Returns (success: bool, error: str|None)
    """
    if not base_path:
        base_path = get_designs_base_path(db)

    rel_path = sd.filepath.lstrip("/\\").replace("\\", os.sep).replace("/", os.sep)
    dest_path = os.path.normpath(os.path.join(base_path, rel_path))
    base_norm = os.path.normpath(base_path)

    # Guard against path traversal: destination must stay within base_path.
    try:
        if os.path.commonpath(
            [os.path.abspath(dest_path), os.path.abspath(base_norm)]
        ) != os.path.abspath(base_norm):
            logging.warning(
                "Skipping file with path outside managed folder: %r → %r",
                sd.source_full_path,
                dest_path,
            )
            return False, "File path is outside the managed folder — import skipped."
    except ValueError:
        logging.warning(
            "Skipping file with path outside managed folder: %r → %r",
            sd.source_full_path,
            dest_path,
        )
        return False, "File path is outside the managed folder — import skipped."
    try:
        os.makedirs(os.path.dirname(dest_path), exist_ok=True)
        if not os.path.exists(dest_path):
            shutil.copy2(sd.source_full_path, dest_path)
    except OSError as exc:
        logging.warning(
            "Could not copy %r to managed folder %r: %s",
            sd.source_full_path,
            dest_path,
            exc,
        )
        return False, f"Could not copy file into catalogue: {exc}"
    return True, None
