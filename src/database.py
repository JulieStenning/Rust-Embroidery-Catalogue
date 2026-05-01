"""
Database engine and session factory.
Configure DATABASE_URL via .env or environment variable.
"""

import logging
import sys
from pathlib import Path

from sqlalchemy import create_engine, event, inspect, text
from sqlalchemy.exc import OperationalError
from sqlalchemy.orm import DeclarativeBase, sessionmaker

from src.config import APP_MODE, DATABASE_URL

logger = logging.getLogger(__name__)

_is_sqlite = DATABASE_URL.startswith("sqlite")

if _is_sqlite:
    engine = create_engine(
        DATABASE_URL,
        echo=False,
        connect_args={"check_same_thread": False},
    )

    def _cleanup_sqlite_orphaned_rows(dbapi_connection) -> None:
        """Remove stale many-to-many rows when explicitly invoked.

        This helper is intentionally *not* run automatically during ordinary app
        startup or per-request connection setup. User-triggered orphan cleanup in
        the maintenance UI should remain explicit, and routine connections should
        avoid surprise write activity.
        """
        cursor = dbapi_connection.cursor()
        try:
            existing_tables = {row[0] for row in cursor.execute("""
                    SELECT name
                    FROM sqlite_master
                    WHERE type = 'table'
                      AND name IN ('designs', 'tags', 'projects', 'design_tags', 'project_designs')
                    """).fetchall()}

            cleaned_design_tags = 0
            cleaned_project_designs = 0

            if {"designs", "tags", "design_tags"}.issubset(existing_tables):
                cursor.execute("""
                    DELETE FROM design_tags
                    WHERE NOT EXISTS (
                        SELECT 1 FROM designs WHERE designs.id = design_tags.design_id
                    )
                    OR NOT EXISTS (
                        SELECT 1 FROM tags WHERE tags.id = design_tags.tag_id
                    )
                    """)
                cleaned_design_tags = max(cursor.rowcount, 0)

            if {"designs", "projects", "project_designs"}.issubset(existing_tables):
                cursor.execute("""
                    DELETE FROM project_designs
                    WHERE NOT EXISTS (
                        SELECT 1 FROM projects WHERE projects.id = project_designs.project_id
                    )
                    OR NOT EXISTS (
                        SELECT 1 FROM designs WHERE designs.id = project_designs.design_id
                    )
                    """)
                cleaned_project_designs = max(cursor.rowcount, 0)

            if cleaned_design_tags or cleaned_project_designs:
                logger.warning(
                    "Cleaned %d orphan design_tags row(s) and %d orphan project_designs row(s).",
                    cleaned_design_tags,
                    cleaned_project_designs,
                )
                dbapi_connection.commit()
        finally:
            cursor.close()

    @event.listens_for(engine, "connect")
    def _set_sqlite_pragmas(dbapi_connection, connection_record):
        # WAL mode for better concurrent read performance + FK enforcement.
        cursor = dbapi_connection.cursor()
        cursor.execute("PRAGMA journal_mode=WAL")
        cursor.execute("PRAGMA foreign_keys=ON")
        cursor.close()

        # Custom function: extract the immediate parent folder name from a path
        # e.g. "\DVDs\Florals\pes\rose.pes" -> "pes"
        def _parent_folder(path: str | None) -> str | None:
            if not path:
                return None
            path = path.replace("/", "\\")
            path = path.rstrip("\\")
            idx = path.rfind("\\")
            if idx == -1:
                return path
            parent = path[:idx]
            idx2 = parent.rfind("\\")
            return parent[idx2 + 1 :]

        dbapi_connection.create_function("parent_folder", 1, _parent_folder)

        # Custom function: extract the full folder path without the filename
        # e.g. "\DVDs\Florals\pes\rose.pes" -> "\DVDs\Florals\pes"
        def _folder_path(path: str | None) -> str:
            if not path:
                return ""
            path = path.replace("/", "\\")
            path = path.rstrip("\\")
            idx = path.rfind("\\")
            if idx == -1:
                return ""
            return path[:idx]

        dbapi_connection.create_function("folder_path", 1, _folder_path)

        # Custom function: extract the file extension (including dot) from a path
        # e.g. "\DVDs\Florals\pes\rose.pes" -> ".pes"
        def _file_extension(path: str | None) -> str:
            if not path:
                return ""
            path = path.replace("\\", "/")
            basename = path.rsplit("/", 1)[-1]
            dot_idx = basename.rfind(".")
            if dot_idx == -1:
                return ""
            return basename[dot_idx:]

        dbapi_connection.create_function("file_extension", 1, _file_extension)

else:
    engine = create_engine(DATABASE_URL, echo=False)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


class Base(DeclarativeBase):
    pass


def _user_table_names() -> set[str]:
    """Return non-internal table names for the configured database."""
    with engine.connect() as connection:
        return {
            name
            for name in inspect(connection).get_table_names()
            if name != "alembic_version" and not name.startswith("sqlite_")
        }


def _build_alembic_config():
    """Build an Alembic config bound to the current ``DATABASE_URL``."""
    from alembic.config import Config

    app_root = Path(__file__).resolve().parents[1]
    config = Config(str(app_root / "alembic.ini"))
    config.set_main_option("script_location", str(app_root / "alembic"))
    config.set_main_option("sqlalchemy.url", DATABASE_URL)
    # When bootstrapping programmatically from the desktop launcher, keep the
    # application's existing logging configuration instead of letting Alembic
    # reset handlers via ``fileConfig(...)``.
    config.attributes["skip_logging_file_config"] = True
    return config


def _stamp_database_revision(revision: str) -> None:
    """Mark the database as being at the given Alembic *revision*."""
    from alembic import command

    command.stamp(_build_alembic_config(), revision)


def _stamp_database_head() -> None:
    """Mark a freshly created database as being at the current Alembic head."""
    _stamp_database_revision("head")


def _upgrade_database_head() -> None:
    """Apply Alembic upgrades for an existing database."""
    from alembic import command

    command.upgrade(_build_alembic_config(), "head")


def _current_alembic_revision() -> str | None:
    """Return the recorded Alembic revision, or ``None`` for unstamped DBs."""
    with engine.connect() as connection:
        inspector = inspect(connection)
        if "alembic_version" not in inspector.get_table_names():
            return None
        revision = connection.execute(
            text("SELECT version_num FROM alembic_version LIMIT 1")
        ).scalar()
        return str(revision) if revision else None


def _infer_revision_for_unstamped_database(existing_tables: set[str]) -> str | None:
    """Infer the closest Alembic baseline for a legacy database with no stamp."""
    required_initial_tables = {
        "designers",
        "sources",
        "hoops",
        "tags",
        "projects",
        "designs",
        "design_tags",
        "project_designs",
    }
    if not required_initial_tables.issubset(existing_tables):
        return None

    with engine.connect() as connection:
        inspector = inspect(connection)
        design_columns = {column["name"] for column in inspector.get_columns("designs")}
        tag_columns = {column["name"] for column in inspector.get_columns("tags")}

    if "settings" not in existing_tables:
        return "0001_initial"
    if "tags_checked" not in design_columns:
        return "0002_settings"
    if "tagging_tier" not in design_columns:
        return "0003_tags_types"
    if "tag_group" not in tag_columns:
        return "0004_tagging_tier"
    return "head"


def _seed_delivered_tags() -> int:
    """Ensure the delivered starter tags exist with current descriptions/groups."""
    from src.services import tags as tags_svc

    db = SessionLocal()
    try:
        return tags_svc.seed_default_tags(db)
    finally:
        db.close()


def _seed_delivered_tags_for_startup(context: str) -> int:
    """Best-effort wrapper so a missing starter CSV does not block app startup."""
    try:
        return _seed_delivered_tags()
    except FileNotFoundError as exc:
        logger.warning(
            "Delivered starter tags are unavailable during %s; continuing without seeding. %s",
            context,
            exc,
        )
        return 0


def bootstrap_database() -> str:
    """Prepare the database for application startup.

    - Fresh database: create the latest schema directly from SQLAlchemy metadata,
      seed the delivered tags, and stamp the Alembic head.
    - Existing stamped database: run the normal Alembic upgrade path.
    - Existing unstamped database: infer the closest baseline, stamp it, and
      then upgrade (or stamp straight to head if the schema is already current).
    """
    existing_tables = _user_table_names()
    if existing_tables:
        revision = _current_alembic_revision()
        if revision:
            logger.info(
                "Existing database detected (%s); applying Alembic upgrades from %s.",
                ", ".join(sorted(existing_tables)),
                revision,
            )
            _upgrade_database_head()
            seeded = _seed_delivered_tags_for_startup("database upgrade")
            logger.info("Database upgraded to head and %d delivered tag(s) were ensured.", seeded)
            return "upgraded"

        inferred_revision = _infer_revision_for_unstamped_database(existing_tables)
        if inferred_revision == "head":
            logger.warning(
                "Existing database detected (%s) but no Alembic revision is recorded; "
                "assuming the schema is already current and stamping head.",
                ", ".join(sorted(existing_tables)),
            )
            seeded = _seed_delivered_tags_for_startup("database stamping")
            _stamp_database_head()
            logger.info(
                "Unstamped current-schema database stamped to head and %d delivered tag(s) were ensured.",
                seeded,
            )
            return "stamped"

        if inferred_revision:
            logger.warning(
                "Existing database detected (%s) but no Alembic revision is recorded; "
                "stamping %s before upgrading to head.",
                ", ".join(sorted(existing_tables)),
                inferred_revision,
            )
            _stamp_database_revision(inferred_revision)
            _upgrade_database_head()
            seeded = _seed_delivered_tags_for_startup("legacy database upgrade")
            logger.info(
                "Legacy database upgraded to head and %d delivered tag(s) were ensured.", seeded
            )
            return "upgraded"

        logger.info(
            "Existing database detected (%s) without a recognised Alembic baseline; "
            "attempting a normal upgrade.",
            ", ".join(sorted(existing_tables)),
        )
        _upgrade_database_head()
        seeded = _seed_delivered_tags_for_startup("database upgrade")
        logger.info("Database upgraded to head and %d delivered tag(s) were ensured.", seeded)
        return "upgraded"

    logger.info("No application tables found; creating a new database from the latest schema.")

    # The frozen Windows desktop build has shown intermittent SQLite DDL issues
    # during direct ``create_all()`` on a completely fresh database, while the
    # Alembic migration path is reliable there. Prefer migrations in that mode.
    if APP_MODE == "desktop" or getattr(sys, "frozen", False):
        logger.info(
            "Fresh desktop/frozen database detected; creating schema via Alembic migrations."
        )
        _upgrade_database_head()
        seeded = _seed_delivered_tags_for_startup("fresh database creation")
        logger.info(
            "Database created via Alembic migrations and %d delivered tag(s) were ensured.", seeded
        )
        return "created"

    # Import models here so their tables are registered on ``Base.metadata``
    # even when bootstrap runs from a clean Python process before the web app
    # entrypoint has imported ``src.models``.
    import src.models  # noqa: F401

    try:
        Base.metadata.create_all(bind=engine)
    except OperationalError:
        logger.exception(
            "Direct metadata schema creation failed for a fresh database; "
            "falling back to Alembic upgrade head."
        )
        _upgrade_database_head()
        seeded = _seed_delivered_tags_for_startup("fresh database creation")
        logger.info(
            "Database created via Alembic fallback and %d delivered tag(s) were ensured.", seeded
        )
        return "created"

    seeded = _seed_delivered_tags_for_startup("fresh database creation")

    _stamp_database_head()
    logger.info(
        "Database created from current metadata and %d delivered tag(s) were ensured.", seeded
    )
    return "created"


def get_db():
    """FastAPI dependency: yields a database session and closes it after the request."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
