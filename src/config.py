"""
Application settings — read from environment variables or a .env file.

Copy .env.example → .env and set values for your machine.

Runtime modes
-------------
APP_MODE controls where writable data is stored and how the app launches:

  development  — default for the repository build; data lives under
                 <repo_root>/data/.  Developer port 8003.

  portable     — SD card / USB workflow; data lives self-relative under
                 <drive_root>/data/.  Portable port 8002.

  desktop      — packaged Windows installer build; small writable state
                 stays under %LOCALAPPDATA%\\EmbroideryCatalogue\\, while
                 the database and managed design library can live under a
                 separate user-chosen data root.  Port is selected
                 dynamically at runtime from free localhost ports.
"""

import json
import os
import shutil
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Application root
# ---------------------------------------------------------------------------

# When packaged with PyInstaller the executable unpacks to a temporary folder.
# sys._MEIPASS is set by PyInstaller to the folder containing bundled data.
# For a one-folder build the root is the folder containing the executable.
if getattr(sys, "frozen", False):
    # Running as a PyInstaller bundle
    _APP_ROOT = Path(sys.executable).parent
else:
    # Running from source — one level above src/
    _APP_ROOT = Path(__file__).parents[1]

# ---------------------------------------------------------------------------
# Runtime mode
# ---------------------------------------------------------------------------

# APP_MODE is set by the desktop launcher before importing this module, or by
# start.bat for development/portable use.  Default is "development" so the
# repository build continues to work without any extra configuration.
APP_MODE: str = os.environ.get("APP_MODE", "development").strip().lower()

# ---------------------------------------------------------------------------
# .env loading
# ---------------------------------------------------------------------------

# Load .env if present (no hard dependency on python-dotenv).
# For the desktop build, also look for a .env next to the executable so
# users can override settings without rebuilding the package.
_env_candidates = [_APP_ROOT / ".env"]
if APP_MODE == "desktop":
    # The exe and the .env are both in the install folder
    _env_candidates.insert(
        0,
        (
            Path(sys.executable).parent / ".env"
            if getattr(sys, "frozen", False)
            else _APP_ROOT / ".env"
        ),
    )

for _env_file in _env_candidates:
    if _env_file.exists():
        for _line in _env_file.read_text(encoding="utf-8").splitlines():
            _line = _line.strip()
            if _line and not _line.startswith("#") and "=" in _line:
                _k, _, _v = _line.partition("=")
                os.environ.setdefault(_k.strip(), _v.strip())
        break

# ---------------------------------------------------------------------------
# Per-mode state/data directories
# ---------------------------------------------------------------------------


def _load_storage_pointer(storage_file: Path) -> Path | None:
    """Return the configured desktop data root from ``storage.json`` if present."""
    if not storage_file.exists():
        return None
    try:
        payload = json.loads(storage_file.read_text(encoding="utf-8"))
    except (OSError, ValueError, TypeError, json.JSONDecodeError):
        return None

    data_root = payload.get("data_root") if isinstance(payload, dict) else None
    if isinstance(data_root, str) and data_root.strip():
        return Path(data_root.strip()).expanduser()
    return None


def _default_desktop_data_root(state_root: Path) -> Path:
    """Choose a safe default data location for desktop installs."""
    legacy_db = state_root / "database" / "catalogue.db"
    legacy_designs = state_root / "MachineEmbroideryDesigns"
    if legacy_db.exists() or legacy_designs.exists():
        return state_root

    # Use the drive root on Windows; fall back to home directory on macOS/Linux.
    if sys.platform == "win32" and _APP_ROOT.drive:
        return Path(_APP_ROOT.drive + "\\") / "EmbroideryCatalogueData"

    return Path.home() / "EmbroideryCatalogueData"


def _copy_missing_files(source: Path, target: Path) -> None:
    """Copy managed catalogue content without overwriting existing target files."""
    if not source.exists():
        return

    if source.is_file():
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        return

    if not target.exists():
        shutil.copytree(source, target)
        return

    for item in source.rglob("*"):
        relative = item.relative_to(source)
        destination = target / relative
        if item.is_dir():
            destination.mkdir(parents=True, exist_ok=True)
        elif not destination.exists():
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(item, destination)


def copy_managed_data_if_needed(source_root: Path, target_root: Path) -> None:
    """Copy database/design-library content into a new data root if needed.

    This is intentionally conservative: it only fills in missing files and never
    overwrites anything already present in the destination.
    """
    try:
        if source_root.resolve() == target_root.resolve():
            return
    except OSError:
        if str(source_root) == str(target_root):
            return

    for name in ("database", "MachineEmbroideryDesigns"):
        _copy_missing_files(source_root / name, target_root / name)


if APP_MODE == "desktop":
    # Determine the platform default for the state directory.
    # LOCALAPPDATA is respected on all platforms when explicitly set (used in tests and
    # Windows installs).  On macOS the default is ~/Library/Application Support;
    # on Linux it is XDG_DATA_HOME (or ~/.local/share).
    if sys.platform == "win32":
        _default_appdata: Path = Path.home() / "AppData" / "Local"
    elif sys.platform == "darwin":
        _default_appdata = Path.home() / "Library" / "Application Support"
    else:
        _default_appdata = Path(
            os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local" / "share"))
        )
    _local_app_data = Path(os.environ.get("LOCALAPPDATA", str(_default_appdata)))
    STATE_ROOT: Path = _local_app_data / "EmbroideryCatalogue"
    STORAGE_CONFIG_FILE: Path = STATE_ROOT / "storage.json"

    _configured_data_root = os.environ.get("EMBROIDERY_DATA_ROOT", "").strip()
    if _configured_data_root:
        DATA_ROOT: Path = Path(_configured_data_root).expanduser()
    else:
        DATA_ROOT = _load_storage_pointer(STORAGE_CONFIG_FILE) or _default_desktop_data_root(
            STATE_ROOT
        )
else:
    # Development and portable modes stay fully self-contained and movable.
    STATE_ROOT = _APP_ROOT / "data"
    STORAGE_CONFIG_FILE = STATE_ROOT / "storage.json"
    DATA_ROOT = _APP_ROOT / "data"

if APP_MODE == "desktop":
    try:
        copy_managed_data_if_needed(STATE_ROOT, DATA_ROOT)
    except OSError:
        pass

# Backwards-compatible alias for older imports/tests.
_DATA_ROOT = DATA_ROOT

# ---------------------------------------------------------------------------
# Database URL
# ---------------------------------------------------------------------------

if APP_MODE == "development":
    _default_db = "sqlite:///" + str(DATA_ROOT / "database" / "catalogue_dev.db")
else:
    _default_db = "sqlite:///" + str(DATA_ROOT / "database" / "catalogue.db")

DATABASE_URL: str = os.environ.get("DATABASE_URL", _default_db)

DATABASE_URL_TEST: str = os.environ.get(
    "DATABASE_URL_TEST",
    "sqlite:///" + str(_APP_ROOT / "data" / "test_catalogue.db"),
)

# ---------------------------------------------------------------------------
# Managed designs folder
# ---------------------------------------------------------------------------

# All design filepaths in the database are relative to this folder.
# Keep this fixed to the managed per-mode data root so legacy external path
# overrides cannot break portability or desktop installs.
DESIGNS_BASE_PATH: str = str(DATA_ROOT / "MachineEmbroideryDesigns")

# ---------------------------------------------------------------------------
# Log directory (desktop mode only — other modes log to the console)
# ---------------------------------------------------------------------------

LOGS_DIR: Path = STATE_ROOT / "logs" if APP_MODE == "desktop" else _APP_ROOT / "logs"

# ---------------------------------------------------------------------------
# Ensure required data directories exist
# ---------------------------------------------------------------------------

for _d in [
    STATE_ROOT,
    DATA_ROOT,
    LOGS_DIR,
    (
        Path(DATABASE_URL.replace("sqlite:///", "")).parent
        if DATABASE_URL.startswith("sqlite:///")
        else DATA_ROOT / "database"
    ),
    Path(DESIGNS_BASE_PATH),
]:
    try:
        _d.mkdir(parents=True, exist_ok=True)
    except OSError:
        pass  # best-effort; errors will surface when the app tries to use the path


def _env_flag_enabled(name: str) -> bool:
    """Return True when the named environment variable is set to a truthy value."""
    value = os.environ.get(name, "").strip().lower()
    return value in {"1", "true", "yes", "on"}


def external_launches_disabled() -> bool:
    """Return True when browser/editor/file-manager launches should be suppressed."""
    return bool(os.environ.get("PYTEST_CURRENT_TEST")) or _env_flag_enabled(
        "EMBROIDERY_DISABLE_EXTERNAL_OPEN"
    )


def native_dialogs_disabled() -> bool:
    """Return True when native file/folder dialogs should be suppressed."""
    return bool(os.environ.get("PYTEST_CURRENT_TEST")) or _env_flag_enabled(
        "EMBROIDERY_DISABLE_NATIVE_DIALOGS"
    )
