"""Service for application settings (key/value store)."""

from __future__ import annotations

import json
import ntpath
import os
from datetime import datetime
from pathlib import Path

from sqlalchemy.orm import Session

import src.config as app_config
from src.config import (
    APP_MODE,
    DATA_ROOT,
    DATABASE_URL,
    DESIGNS_BASE_PATH,
    LOGS_DIR,
    STORAGE_CONFIG_FILE,
)
from src.models import Setting

# Shared disclaimer acceptance marker used by both `start.bat` and the web app.
DISCLAIMER_ACK_FILE = Path(__file__).resolve().parents[2] / "data" / "disclaimer_accepted.txt"
ENV_FILE = Path(__file__).resolve().parents[2] / ".env"

# Known setting keys and their defaults (from config/.env)
SETTING_DESIGNS_BASE_PATH = "designs_base_path"
SETTING_DISCLAIMER_ACCEPTED = "disclaimer_accepted"
SETTING_BACKUP_DB_DESTINATION = "backup.database_destination"
SETTING_BACKUP_DESIGNS_DESTINATION = "backup.designs_destination"
SETTING_LAST_IMPORT_BROWSE_FOLDER = "import.last_browse_folder"
SETTING_AI_TIER2_AUTO = "ai.tier2_auto"
SETTING_AI_TIER3_AUTO = "ai.tier3_auto"
SETTING_AI_BATCH_SIZE = "ai.batch_size"
SETTING_AI_DELAY = "ai.delay"
SETTING_IMPORT_COMMIT_BATCH_SIZE = "import.commit_batch_size"
SETTING_IMAGE_PREFERENCE = "image.preference"

_DEFAULTS: dict[str, str] = {
    SETTING_DESIGNS_BASE_PATH: DESIGNS_BASE_PATH,
    SETTING_DISCLAIMER_ACCEPTED: "false",
    SETTING_BACKUP_DB_DESTINATION: "",
    SETTING_BACKUP_DESIGNS_DESTINATION: "",
    SETTING_LAST_IMPORT_BROWSE_FOLDER: "",
    SETTING_AI_TIER2_AUTO: "false",
    SETTING_AI_TIER3_AUTO: "false",
    SETTING_AI_BATCH_SIZE: "",
    SETTING_AI_DELAY: "",
    SETTING_IMPORT_COMMIT_BATCH_SIZE: "",
    SETTING_IMAGE_PREFERENCE: "2d",
}

_DESCRIPTIONS: dict[str, str] = {
    SETTING_DESIGNS_BASE_PATH: (
        "Managed folder where imported embroidery files are copied and stored. "
        "All design file paths in the database are relative to this location. "
        "Portable mode keeps it self-contained with the application folder, while "
        "desktop mode keeps it under the current catalogue data root."
    ),
    SETTING_DISCLAIMER_ACCEPTED: (
        "Whether the application's disclaimer has been accepted for this installation."
    ),
    SETTING_BACKUP_DB_DESTINATION: ("Destination folder for database backup files."),
    SETTING_BACKUP_DESIGNS_DESTINATION: ("Destination folder for incremental designs backup."),
    SETTING_LAST_IMPORT_BROWSE_FOLDER: ("Most recently used folder for the bulk import picker."),
    SETTING_AI_TIER2_AUTO: (
        "Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present."
    ),
    SETTING_AI_TIER3_AUTO: (
        "Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present."
    ),
    SETTING_AI_BATCH_SIZE: (
        "Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs."
    ),
    SETTING_AI_DELAY: (
        "Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds). "
        "Increase this if you encounter 429 Too Many Requests errors."
    ),
    SETTING_IMPORT_COMMIT_BATCH_SIZE: (
        "Maximum number of designs to persist or update before each database commit during import. "
        "Leave blank to use the default batch size (1000)."
    ),
    SETTING_IMAGE_PREFERENCE: (
        "Preferred image rendering mode for new imports: '2d' for fast flat previews or "
        "'3d' for detailed stitch-simulated previews. Can be overridden per import session."
    ),
}


def get_setting(db: Session, key: str) -> str:
    """Return the current value for *key*, falling back to the configured default."""
    row = db.get(Setting, key)
    if row is not None:
        return row.value
    return _DEFAULTS.get(key, "")


def get_all(db: Session) -> list[Setting]:
    return db.query(Setting).order_by(Setting.key).all()


def set_setting(db: Session, key: str, value: str) -> Setting:
    row = db.get(Setting, key)
    if row is None:
        row = Setting(
            key=key,
            value=value,
            description=_DESCRIPTIONS.get(key),
        )
        db.add(row)
    else:
        row.value = value
    db.commit()
    db.refresh(row)
    return row


def _is_truthy(value: str | None) -> bool:
    return (value or "").strip().lower() in {"1", "true", "yes", "y", "accepted"}


def is_disclaimer_accepted(db: Session) -> bool:
    """Return True if the disclaimer has already been accepted for this installation."""
    if _is_truthy(get_setting(db, SETTING_DISCLAIMER_ACCEPTED)):
        return True

    if DISCLAIMER_ACK_FILE.exists():
        set_setting(db, SETTING_DISCLAIMER_ACCEPTED, "true")
        return True

    return False


def mark_disclaimer_accepted(db: Session) -> Setting:
    """Persist disclaimer acceptance in both the DB and the shared marker file."""
    row = set_setting(db, SETTING_DISCLAIMER_ACCEPTED, "true")
    try:
        DISCLAIMER_ACK_FILE.parent.mkdir(parents=True, exist_ok=True)
        DISCLAIMER_ACK_FILE.write_text(
            f"Accepted on {datetime.now().isoformat(timespec='seconds')}\n",
            encoding="utf-8",
        )
    except OSError:
        # The DB flag is the primary source of truth; the file is a convenience marker.
        pass
    return row


def get_google_api_key() -> str:
    """Return the configured Google API key from `.env` or the current environment."""
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if stripped.startswith("GOOGLE_API_KEY="):
                return stripped.split("=", 1)[1].strip()
    return os.environ.get("GOOGLE_API_KEY", "")


def save_google_api_key(value: str) -> None:
    """Persist the Google API key in the project `.env` file.

    If ``value`` is blank, any existing ``GOOGLE_API_KEY`` entry is removed.
    Other settings and comments are preserved.
    """
    key = value.strip()
    lines = ENV_FILE.read_text(encoding="utf-8").splitlines() if ENV_FILE.exists() else []

    updated_lines: list[str] = []
    replaced = False
    for line in lines:
        if line.strip().startswith("GOOGLE_API_KEY="):
            if key:
                updated_lines.append(f"GOOGLE_API_KEY={key}")
            replaced = True
            continue
        updated_lines.append(line)

    if key and not replaced:
        if updated_lines and updated_lines[-1].strip():
            updated_lines.append("")
        updated_lines.append(f"GOOGLE_API_KEY={key}")

    ENV_FILE.parent.mkdir(parents=True, exist_ok=True)
    content = "\n".join(updated_lines)
    if content:
        content += "\n"
    ENV_FILE.write_text(content, encoding="utf-8")

    if key:
        os.environ["GOOGLE_API_KEY"] = key
    else:
        os.environ.pop("GOOGLE_API_KEY", None)


def get_designs_base_path(db: Session) -> str:
    """Return the managed embroidery-file storage folder.

    The catalogue now uses managed-only storage. The old ``designs_base_path``
    setting is retained only as legacy metadata and no longer overrides the
    actual on-disk storage location.
    """
    return os.path.normpath(DESIGNS_BASE_PATH)


def get_data_root(db: Session | None = None) -> str:
    """Return the current root folder that holds the database and design library."""
    return os.path.normpath(str(DATA_ROOT))


def get_database_file_path(db: Session | None = None) -> str:
    """Return the filesystem path for the active database file when using SQLite."""
    if DATABASE_URL.startswith("sqlite:///"):
        raw_path = DATABASE_URL.split("sqlite:///", 1)[1]
        if len(raw_path) >= 2 and raw_path[1] == ":":
            return ntpath.normpath(raw_path.replace("/", "\\"))
        return os.path.normpath(raw_path)
    return DATABASE_URL


def get_logs_dir(db: Session | None = None) -> str:
    """Return the folder used for desktop log files."""
    return os.path.normpath(str(LOGS_DIR))


def save_data_root(value: str) -> str:
    """Persist the desktop data root pointer in ``storage.json``.

    This only affects desktop installs. The new location is used on the next
    app start so the database and managed design library can live on a larger
    drive while lightweight state remains in LocalAppData.
    """
    global DATA_ROOT, DATABASE_URL, DESIGNS_BASE_PATH

    raw_value = (value or "").strip()
    if not raw_value:
        raise ValueError("A data root path is required")

    normalized = os.path.normpath(os.path.abspath(os.path.expanduser(raw_value)))

    target_root = Path(normalized)
    target_root.mkdir(parents=True, exist_ok=True)
    (target_root / "database").mkdir(parents=True, exist_ok=True)
    (target_root / "MachineEmbroideryDesigns").mkdir(parents=True, exist_ok=True)

    try:
        app_config.copy_managed_data_if_needed(Path(DATA_ROOT), target_root)
    except OSError:
        pass

    if APP_MODE == "desktop":
        STORAGE_CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
        STORAGE_CONFIG_FILE.write_text(
            json.dumps({"data_root": normalized}, indent=2) + "\n",
            encoding="utf-8",
        )
        os.environ["EMBROIDERY_DATA_ROOT"] = normalized

        DATA_ROOT = Path(normalized)
        DESIGNS_BASE_PATH = os.path.normpath(str(DATA_ROOT / "MachineEmbroideryDesigns"))
        DATABASE_URL = "sqlite:///" + str(DATA_ROOT / "database" / "catalogue.db")

        app_config.DATA_ROOT = DATA_ROOT
        app_config._DATA_ROOT = DATA_ROOT
        app_config.DESIGNS_BASE_PATH = DESIGNS_BASE_PATH
        app_config.DATABASE_URL = DATABASE_URL

    return normalized
