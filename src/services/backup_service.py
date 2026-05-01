"""Backup service — database and designs folder backup logic."""

from __future__ import annotations

import logging
import os
import shutil
import sqlite3
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Result dataclasses
# ---------------------------------------------------------------------------


@dataclass
class DatabaseBackupResult:
    """Result of a database backup operation."""

    success: bool
    backup_path: str = ""
    size_bytes: int = 0
    completed_at: str = ""
    error: str = ""

    @property
    def size_mb(self) -> float:
        return round(self.size_bytes / (1024 * 1024), 2)


@dataclass
class DesignsBackupResult:
    """Result of an incremental designs backup operation."""

    success: bool
    copied: int = 0
    updated: int = 0
    unchanged: int = 0
    archived: int = 0
    total_bytes_copied: int = 0
    scanned: int = 0
    completed_at: str = ""
    error: str = ""
    archived_paths: list[str] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Database backup
# ---------------------------------------------------------------------------


def backup_database(db_path: str, destination_dir: str) -> DatabaseBackupResult:
    """Back up the SQLite database to *destination_dir* with a timestamped filename.

    For a live in-app backup, this uses SQLite's built-in backup API so the
    resulting copy is transactionally consistent even when the application is
    still running. If the source file cannot be opened as a SQLite database,
    a plain file copy is used as a narrow fallback.

    Parameters
    ----------
    db_path:
        Absolute path to the live ``catalogue.db`` file.
    destination_dir:
        Directory where the backup file will be written.  Created if it does
        not exist.

    Returns
    -------
    DatabaseBackupResult
        Summary of the operation, including success/failure details.
    """
    db_file = Path(db_path)
    dest_dir = Path(destination_dir)

    if not db_file.is_file():
        return DatabaseBackupResult(
            success=False,
            error=f"Database file not found: {db_path}",
        )

    try:
        dest_dir.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        return DatabaseBackupResult(
            success=False,
            error=f"Cannot create destination folder: {exc}",
        )

    if not os.access(dest_dir, os.W_OK):
        return DatabaseBackupResult(
            success=False,
            error=f"Destination folder is not writable: {destination_dir}",
        )

    timestamp = datetime.now().strftime("%Y-%m-%d_%H%M")
    backup_filename = f"catalogue_{timestamp}.db"
    backup_path = dest_dir / backup_filename

    try:
        if backup_path.exists():
            backup_path.unlink()
        _backup_sqlite_database(db_file, backup_path)
    except sqlite3.DatabaseError as exc:
        logger.warning(
            "SQLite backup API could not read %s; falling back to file copy: %s",
            db_file,
            exc,
        )
        try:
            if backup_path.exists():
                backup_path.unlink()
            shutil.copy2(str(db_file), str(backup_path))
        except OSError as copy_exc:
            return DatabaseBackupResult(
                success=False,
                error=f"Failed to back up database: {copy_exc}",
            )
    except (sqlite3.Error, OSError) as exc:
        if backup_path.exists():
            try:
                backup_path.unlink()
            except OSError:
                pass
        return DatabaseBackupResult(
            success=False,
            error=f"Failed to back up database: {exc}",
        )

    size_bytes = backup_path.stat().st_size
    completed_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    logger.info(
        "Database backup complete: %s (%.2f MB)",
        backup_path,
        size_bytes / (1024 * 1024),
    )

    return DatabaseBackupResult(
        success=True,
        backup_path=str(backup_path),
        size_bytes=size_bytes,
        completed_at=completed_at,
    )


# ---------------------------------------------------------------------------
# Designs incremental backup
# ---------------------------------------------------------------------------


def backup_designs(source_dir: str, destination_dir: str) -> DesignsBackupResult:
    """Incrementally back up the designs folder.

    Comparison is done by relative path, file size and modification time.
    Files present in the backup but absent from the source are moved into a
    dated ``_deleted/YYYY-MM-DD/`` archive folder rather than being deleted.

    Parameters
    ----------
    source_dir:
        The live ``data/MachineEmbroideryDesigns/`` folder.
    destination_dir:
        The user-chosen designs backup folder.

    Returns
    -------
    DesignsBackupResult
        Summary counters for the operation.
    """
    src = Path(source_dir)
    dest = Path(destination_dir)

    if not src.is_dir():
        return DesignsBackupResult(
            success=False,
            error=f"Source designs folder not found: {source_dir}",
        )

    try:
        dest.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        return DesignsBackupResult(
            success=False,
            error=f"Cannot create destination folder: {exc}",
        )

    if not os.access(dest, os.W_OK):
        return DesignsBackupResult(
            success=False,
            error=f"Destination folder is not writable: {destination_dir}",
        )

    result = DesignsBackupResult(success=True)
    today = datetime.now().strftime("%Y-%m-%d")
    deleted_archive_dir = dest / "_deleted" / today

    # Build a set of all source files (relative paths)
    source_files: set[Path] = set()
    for abs_path in src.rglob("*"):
        if abs_path.is_file():
            rel = abs_path.relative_to(src)
            source_files.add(rel)

    result.scanned = len(source_files)

    # Forward pass — copy new/changed files from source to backup
    for rel in source_files:
        src_file = src / rel
        dest_file = dest / rel

        try:
            if not dest_file.exists():
                dest_file.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(str(src_file), str(dest_file))
                result.copied += 1
                result.total_bytes_copied += src_file.stat().st_size
            elif _file_changed(src_file, dest_file):
                shutil.copy2(str(src_file), str(dest_file))
                result.updated += 1
                result.total_bytes_copied += src_file.stat().st_size
            else:
                result.unchanged += 1
        except OSError as exc:
            logger.warning("Skipping %s — copy error: %s", rel, exc)

    # Reverse pass — archive backup files that no longer exist in source
    for abs_dest_file in list(dest.rglob("*")):
        if not abs_dest_file.is_file():
            continue
        # Skip files inside the _deleted archive tree
        try:
            abs_dest_file.relative_to(dest / "_deleted")
            continue
        except ValueError:
            pass

        rel = abs_dest_file.relative_to(dest)
        if rel not in source_files:
            archive_target = deleted_archive_dir / rel
            try:
                archive_target.parent.mkdir(parents=True, exist_ok=True)
                shutil.move(str(abs_dest_file), str(archive_target))
                result.archived += 1
                result.archived_paths.append(str(rel))
                logger.info("Archived deleted file: %s → %s", rel, archive_target)
            except OSError as exc:
                logger.warning("Failed to archive %s: %s", rel, exc)

    _remove_empty_backup_dirs(dest)

    result.completed_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    logger.info(
        "Designs backup complete — scanned %d, copied %d, updated %d, " "unchanged %d, archived %d",
        result.scanned,
        result.copied,
        result.updated,
        result.unchanged,
        result.archived,
    )

    return result


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _backup_sqlite_database(db_file: Path, backup_path: Path) -> None:
    """Create a consistent SQLite backup using the sqlite3 backup API."""
    source_uri = db_file.resolve().as_uri() + "?mode=ro"
    source_conn = None
    dest_conn = None

    try:
        source_conn = sqlite3.connect(source_uri, uri=True, timeout=30.0)
        source_conn.execute("PRAGMA busy_timeout = 30000")
        dest_conn = sqlite3.connect(str(backup_path), timeout=30.0)
        source_conn.backup(dest_conn)
        dest_conn.commit()
    finally:
        if dest_conn is not None:
            dest_conn.close()
        if source_conn is not None:
            source_conn.close()


def _remove_empty_backup_dirs(dest: Path) -> None:
    """Remove empty directories from the live backup tree, excluding `_deleted`."""
    deleted_root = dest / "_deleted"
    dirs = [p for p in dest.rglob("*") if p.is_dir()]
    dirs.sort(key=lambda path: len(path.parts), reverse=True)

    for folder in dirs:
        if folder == dest:
            continue
        try:
            folder.relative_to(deleted_root)
            continue
        except ValueError:
            pass

        try:
            next(folder.iterdir())
        except StopIteration:
            try:
                folder.rmdir()
            except OSError:
                pass
        except OSError:
            pass


def _file_changed(src: Path, dest: Path) -> bool:
    """Return True if *src* differs from *dest* by size or modification time."""
    src_stat = src.stat()
    dest_stat = dest.stat()
    if src_stat.st_size != dest_stat.st_size:
        return True
    # Allow 2-second tolerance for FAT filesystem timestamp granularity
    return abs(src_stat.st_mtime - dest_stat.st_mtime) > 2
