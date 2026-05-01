import importlib
import os
import sqlite3
import subprocess
import sys
from pathlib import Path

import pytest
from sqlalchemy import create_engine, inspect
from sqlalchemy.exc import OperationalError
from sqlalchemy.orm import sessionmaker

from src import database as dbmod


class TestDatabaseHelpers:
    def test_parent_folder_sqlite_function_handles_common_paths(self):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            dbmod._set_sqlite_pragmas(conn, None)
            row = conn.execute(
                "SELECT parent_folder(?), parent_folder(?), parent_folder(?)",
                (
                    "\\DVDs\\Florals\\pes\\rose.pes",
                    "single-file.jef",
                    None,
                ),
            ).fetchone()
        finally:
            conn.close()

        assert row == ("pes", "single-file.jef", None)

    def test_sqlite_pragmas_enable_foreign_keys(self):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            dbmod._set_sqlite_pragmas(conn, None)
            enabled = conn.execute("PRAGMA foreign_keys").fetchone()[0]
        finally:
            conn.close()

        assert enabled == 1

    def test_sqlite_pragmas_do_not_cleanup_orphaned_link_rows_automatically(self):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            conn.executescript(
                """
                CREATE TABLE designs (id INTEGER PRIMARY KEY);
                CREATE TABLE tags (id INTEGER PRIMARY KEY);
                CREATE TABLE projects (id INTEGER PRIMARY KEY);
                CREATE TABLE design_tags (
                    design_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    PRIMARY KEY (design_id, tag_id)
                );
                CREATE TABLE project_designs (
                    project_id INTEGER NOT NULL,
                    design_id INTEGER NOT NULL,
                    PRIMARY KEY (project_id, design_id)
                );
                INSERT INTO design_tags (design_id, tag_id) VALUES (999, 1);
                INSERT INTO project_designs (project_id, design_id) VALUES (123, 999);
                """
            )

            dbmod._set_sqlite_pragmas(conn, None)

            design_tag_count = conn.execute("SELECT COUNT(*) FROM design_tags").fetchone()[0]
            project_design_count = conn.execute("SELECT COUNT(*) FROM project_designs").fetchone()[
                0
            ]
        finally:
            conn.close()

        assert design_tag_count == 1
        assert project_design_count == 1

    def test_sqlite_orphan_cleanup_helper_removes_orphaned_link_rows(self):
        if not hasattr(dbmod, "_cleanup_sqlite_orphaned_rows"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            conn.executescript(
                """
                CREATE TABLE designs (id INTEGER PRIMARY KEY);
                CREATE TABLE tags (id INTEGER PRIMARY KEY);
                CREATE TABLE projects (id INTEGER PRIMARY KEY);
                CREATE TABLE design_tags (
                    design_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    PRIMARY KEY (design_id, tag_id)
                );
                CREATE TABLE project_designs (
                    project_id INTEGER NOT NULL,
                    design_id INTEGER NOT NULL,
                    PRIMARY KEY (project_id, design_id)
                );
                INSERT INTO design_tags (design_id, tag_id) VALUES (999, 1);
                INSERT INTO project_designs (project_id, design_id) VALUES (123, 999);
                """
            )

            dbmod._set_sqlite_pragmas(conn, None)
            dbmod._cleanup_sqlite_orphaned_rows(conn)

            design_tag_count = conn.execute("SELECT COUNT(*) FROM design_tags").fetchone()[0]
            project_design_count = conn.execute("SELECT COUNT(*) FROM project_designs").fetchone()[
                0
            ]
        finally:
            conn.close()

        assert design_tag_count == 0
        assert project_design_count == 0

    def test_parent_folder_sqlite_function_normalises_forward_slashes(self):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            dbmod._set_sqlite_pragmas(conn, None)
            value = conn.execute(
                "SELECT parent_folder(?)",
                ("folder/subfolder/designs/rose.pes",),
            ).fetchone()[0]
        finally:
            conn.close()

        assert value == "designs"

    def test_folder_path_sqlite_function_returns_nested_directory_path(self):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            dbmod._set_sqlite_pragmas(conn, None)
            row = conn.execute(
                "SELECT folder_path(?), folder_path(?)",
                (
                    "folder/subfolder/My Creations/rose.pes",
                    "single-file.jef",
                ),
            ).fetchone()
        finally:
            conn.close()

        assert row == ("folder\\subfolder\\My Creations", "")

    def test_searchable_design_columns_have_indexes(self, db):
        inspector = inspect(db.bind)
        design_indexes = {
            tuple(index["column_names"]) for index in inspector.get_indexes("designs")
        }
        tag_link_indexes = {
            tuple(index["column_names"]) for index in inspector.get_indexes("design_tags")
        }
        project_link_indexes = {
            tuple(index["column_names"]) for index in inspector.get_indexes("project_designs")
        }

        assert ("filename",) in design_indexes
        assert ("filepath",) in design_indexes
        assert ("designer_id",) in design_indexes
        assert ("source_id",) in design_indexes
        assert ("date_added",) in design_indexes
        assert ("designer_id", "filename") in design_indexes
        assert ("source_id", "filename") in design_indexes
        assert ("tag_id",) in tag_link_indexes
        assert ("tag_id", "design_id") in tag_link_indexes
        assert ("design_id",) in project_link_indexes
        assert ("design_id", "project_id") in project_link_indexes

    def test_lookup_names_remain_uniquely_constrained(self, db):
        inspector = inspect(db.bind)
        table_names = set(inspector.get_table_names())

        assert "tags" in table_names
        assert "design_tags" in table_names

        expected_unique_columns = {
            "designers": ("name",),
            "sources": ("name",),
            "projects": ("name",),
            "tags": ("description",),
        }

        for table_name, unique_columns in expected_unique_columns.items():
            unique_sets = {
                tuple(constraint["column_names"])
                for constraint in inspector.get_unique_constraints(table_name)
            }
            assert unique_columns in unique_sets

    def test_no_legacy_design_type_names_in_alembic_history(self):
        repo_root = Path(__file__).resolve().parents[1]
        versions_dir = repo_root / "alembic" / "versions"

        for path in versions_dir.glob("*.py"):
            text = path.read_text(encoding="utf-8")
            assert "design_types" not in text
            assert "design_design_types" not in text

    def test_bootstrap_database_works_from_clean_python_process(self, tmp_path):
        repo_root = Path(__file__).resolve().parents[1]
        db_path = tmp_path / "bootstrap_subprocess.db"

        env = os.environ.copy()
        env["DATABASE_URL"] = f"sqlite:///{db_path.as_posix()}"

        result = subprocess.run(
            [
                sys.executable,
                "-c",
                "from src.database import bootstrap_database; print(bootstrap_database())",
            ],
            cwd=repo_root,
            env=env,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, result.stderr or result.stdout

        conn = sqlite3.connect(db_path)
        try:
            tables = {
                row[0]
                for row in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type='table'"
                ).fetchall()
            }
        finally:
            conn.close()

        assert {"tags", "designs", "settings"}.issubset(tables)

    def test_bootstrap_database_creates_current_schema_and_seeds_tags_but_not_hoops(
        self, tmp_path, monkeypatch
    ):
        db_path = tmp_path / "bootstrapped_catalogue.db"
        db_url = f"sqlite:///{db_path.as_posix()}"
        engine = create_engine(db_url, connect_args={"check_same_thread": False})
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        stamped = {"called": False}

        monkeypatch.setattr(dbmod, "DATABASE_URL", db_url)
        monkeypatch.setattr(dbmod, "engine", engine)
        monkeypatch.setattr(dbmod, "SessionLocal", session_local)
        monkeypatch.setattr(
            dbmod, "_stamp_database_head", lambda: stamped.__setitem__("called", True)
        )

        status = dbmod.bootstrap_database()

        assert status == "created"
        assert stamped["called"] is True

        conn = sqlite3.connect(db_path)
        try:
            tables = {
                row[0]
                for row in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type='table'"
                ).fetchall()
            }
            tag_columns = {row[1] for row in conn.execute("PRAGMA table_info(tags)").fetchall()}
            design_columns = {
                row[1] for row in conn.execute("PRAGMA table_info(designs)").fetchall()
            }
            seeded_tags = dict(conn.execute("SELECT description, tag_group FROM tags").fetchall())
            hoop_count = conn.execute("SELECT COUNT(*) FROM hoops").fetchone()[0]
        finally:
            conn.close()
            engine.dispose()

        assert {"tags", "design_tags", "settings", "hoops"}.issubset(tables)
        assert {"tag_group"}.issubset(tag_columns)
        assert {"tags_checked", "tagging_tier"}.issubset(design_columns)
        assert seeded_tags["Cross Stitch"] == "stitching"
        assert seeded_tags["Dancing"] == "image"
        assert hoop_count == 0

    def test_bootstrap_database_stamps_existing_current_schema_without_revision(
        self, tmp_path, monkeypatch
    ):
        db_path = tmp_path / "existing_catalogue.db"
        db_url = f"sqlite:///{db_path.as_posix()}"
        engine = create_engine(db_url, connect_args={"check_same_thread": False})
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)

        monkeypatch.setattr(dbmod, "DATABASE_URL", db_url)
        monkeypatch.setattr(dbmod, "engine", engine)
        monkeypatch.setattr(dbmod, "SessionLocal", session_local)

        import src.models  # noqa: F401

        dbmod.Base.metadata.create_all(bind=engine)

        status = dbmod.bootstrap_database()

        conn = sqlite3.connect(db_path)
        try:
            version_row = conn.execute("SELECT version_num FROM alembic_version").fetchone()
            cross_stitch_group = conn.execute(
                "SELECT tag_group FROM tags WHERE description = 'Cross Stitch'"
            ).fetchone()[0]
        finally:
            conn.close()
            engine.dispose()

        assert status == "stamped"
        assert version_row == ("0011_add_image_type",)
        assert cross_stitch_group == "stitching"

    def test_bootstrap_database_uses_alembic_on_fresh_desktop_db(self, tmp_path, monkeypatch):
        db_path = tmp_path / "desktop_catalogue.db"
        db_url = f"sqlite:///{db_path.as_posix()}"
        engine = create_engine(db_url, connect_args={"check_same_thread": False})
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        upgraded = {"called": False}
        seeded = {"called": False}

        monkeypatch.setattr(dbmod, "APP_MODE", "desktop")
        monkeypatch.setattr(dbmod, "DATABASE_URL", db_url)
        monkeypatch.setattr(dbmod, "engine", engine)
        monkeypatch.setattr(dbmod, "SessionLocal", session_local)
        monkeypatch.setattr(
            dbmod.Base.metadata,
            "create_all",
            lambda bind=None: (_ for _ in ()).throw(
                AssertionError("create_all should not be used in desktop mode")
            ),
        )
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: upgraded.__setitem__("called", True)
        )
        monkeypatch.setattr(
            dbmod, "_seed_delivered_tags", lambda: seeded.__setitem__("called", True) or 98
        )

        status = dbmod.bootstrap_database()

        engine.dispose()
        assert status == "created"
        assert upgraded["called"] is True
        assert seeded["called"] is True

    def test_bootstrap_database_continues_when_delivered_tags_csv_is_missing(
        self, tmp_path, monkeypatch, caplog
    ):
        db_path = tmp_path / "missing_tags_catalogue.db"
        db_url = f"sqlite:///{db_path.as_posix()}"
        engine = create_engine(db_url, connect_args={"check_same_thread": False})
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        upgraded = {"called": False}

        monkeypatch.setattr(dbmod, "APP_MODE", "desktop")
        monkeypatch.setattr(dbmod, "DATABASE_URL", db_url)
        monkeypatch.setattr(dbmod, "engine", engine)
        monkeypatch.setattr(dbmod, "SessionLocal", session_local)
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: upgraded.__setitem__("called", True)
        )
        monkeypatch.setattr(
            dbmod,
            "_seed_delivered_tags",
            lambda: (_ for _ in ()).throw(FileNotFoundError("tags.csv missing")),
        )

        with caplog.at_level("WARNING"):
            status = dbmod.bootstrap_database()

        engine.dispose()
        assert status == "created"
        assert upgraded["called"] is True
        assert "tags.csv missing" in caplog.text

    def test_bootstrap_database_falls_back_to_alembic_when_create_all_fails(
        self, tmp_path, monkeypatch
    ):
        db_path = tmp_path / "fallback_catalogue.db"
        db_url = f"sqlite:///{db_path.as_posix()}"
        engine = create_engine(db_url, connect_args={"check_same_thread": False})
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        upgraded = {"called": False}
        seeded = {"called": False}

        monkeypatch.setattr(dbmod, "APP_MODE", "development")
        monkeypatch.setattr(dbmod, "DATABASE_URL", db_url)
        monkeypatch.setattr(dbmod, "engine", engine)
        monkeypatch.setattr(dbmod, "SessionLocal", session_local)
        monkeypatch.setattr(
            dbmod.Base.metadata,
            "create_all",
            lambda bind=None: (_ for _ in ()).throw(
                OperationalError(
                    "CREATE INDEX ix_designers_id ON designers (id)",
                    {},
                    Exception("no such table: main.designers"),
                )
            ),
        )
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: upgraded.__setitem__("called", True)
        )
        monkeypatch.setattr(
            dbmod, "_seed_delivered_tags", lambda: seeded.__setitem__("called", True) or 98
        )

        status = dbmod.bootstrap_database()

        engine.dispose()
        assert status == "created"
        assert upgraded["called"] is True
        assert seeded["called"] is True

    def test_alembic_upgrade_head_seeds_current_grouped_tags(self, tmp_path):
        repo_root = Path(__file__).resolve().parents[1]
        db_path = tmp_path / "fresh_catalogue.db"

        env = os.environ.copy()
        env["DATABASE_URL"] = f"sqlite:///{db_path.as_posix()}"

        result = subprocess.run(
            [sys.executable, "-m", "alembic", "upgrade", "head"],
            cwd=repo_root,
            env=env,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, result.stderr or result.stdout

        conn = sqlite3.connect(db_path)
        try:
            columns = {row[1] for row in conn.execute("PRAGMA table_info(tags)").fetchall()}
            seeded_tags = dict(conn.execute("SELECT description, tag_group FROM tags").fetchall())
            hoop_count = conn.execute("SELECT COUNT(*) FROM hoops").fetchone()[0]
        finally:
            conn.close()

        assert "tag_group" in columns
        assert seeded_tags["Cross Stitch"] == "stitching"
        assert seeded_tags["Dancing"] == "image"
        assert len(seeded_tags) >= 90
        assert hoop_count == 0

    def test_get_db_yields_session_and_closes_it(self, monkeypatch):
        class DummySession:
            def __init__(self):
                self.closed = False

            def close(self):
                self.closed = True

        session = DummySession()
        monkeypatch.setattr(dbmod, "SessionLocal", lambda: session)

        dependency = dbmod.get_db()
        yielded = next(dependency)

        assert yielded is session

        with pytest.raises(StopIteration):
            next(dependency)

        assert session.closed is True

    def test_folder_path_and_file_extension_sqlite_functions_handle_empty_and_extensionless_values(
        self,
    ):
        if not hasattr(dbmod, "_set_sqlite_pragmas"):
            pytest.skip("SQLite-only test")

        conn = sqlite3.connect(":memory:")
        try:
            dbmod._set_sqlite_pragmas(conn, None)
            row = conn.execute(
                "SELECT folder_path(?), folder_path(?), file_extension(?), file_extension(?), file_extension(?)",
                (
                    None,
                    "single-file.jef",
                    None,
                    "folder/design",
                    "folder/archive.tar.gz",
                ),
            ).fetchone()
        finally:
            conn.close()

        assert row == ("", "", "", "", ".gz")

    @pytest.mark.parametrize(
        ("existing_tables", "design_columns", "tag_columns", "expected"),
        [
            ({"designers"}, set(), set(), None),
            (
                {
                    "designers",
                    "sources",
                    "hoops",
                    "tags",
                    "projects",
                    "designs",
                    "design_tags",
                    "project_designs",
                },
                {"tags_checked", "tagging_tier"},
                {"tag_group"},
                "0001_initial",
            ),
            (
                {
                    "designers",
                    "sources",
                    "hoops",
                    "tags",
                    "projects",
                    "designs",
                    "design_tags",
                    "project_designs",
                    "settings",
                },
                {"tagging_tier"},
                {"tag_group"},
                "0002_settings",
            ),
            (
                {
                    "designers",
                    "sources",
                    "hoops",
                    "tags",
                    "projects",
                    "designs",
                    "design_tags",
                    "project_designs",
                    "settings",
                },
                {"tags_checked"},
                {"tag_group"},
                "0003_tags_types",
            ),
            (
                {
                    "designers",
                    "sources",
                    "hoops",
                    "tags",
                    "projects",
                    "designs",
                    "design_tags",
                    "project_designs",
                    "settings",
                },
                {"tags_checked", "tagging_tier"},
                set(),
                "0004_tagging_tier",
            ),
        ],
    )
    def test_infer_revision_for_unstamped_database_variants(
        self, monkeypatch, existing_tables, design_columns, tag_columns, expected
    ):
        class DummyConnection:
            def __enter__(self):
                return object()

            def __exit__(self, exc_type, exc, tb):
                return False

        class DummyInspector:
            def get_columns(self, name):
                columns = design_columns if name == "designs" else tag_columns
                return [{"name": column} for column in columns]

        class DummyEngine:
            def connect(self):
                return DummyConnection()

        monkeypatch.setattr(dbmod, "engine", DummyEngine())
        monkeypatch.setattr(dbmod, "inspect", lambda _connection: DummyInspector())

        assert dbmod._infer_revision_for_unstamped_database(existing_tables) == expected

    def test_current_alembic_revision_returns_recorded_revision(self, tmp_path, monkeypatch):
        db_path = tmp_path / "revision_catalogue.db"
        engine = create_engine(
            f"sqlite:///{db_path.as_posix()}", connect_args={"check_same_thread": False}
        )

        with engine.begin() as conn:
            conn.exec_driver_sql("CREATE TABLE alembic_version (version_num VARCHAR(32) NOT NULL)")
            conn.exec_driver_sql(
                "INSERT INTO alembic_version (version_num) VALUES ('0009_remove_legacy_designs_base_path_setting')"
            )

        monkeypatch.setattr(dbmod, "engine", engine)

        try:
            assert (
                dbmod._current_alembic_revision() == "0009_remove_legacy_designs_base_path_setting"
            )
        finally:
            engine.dispose()

    def test_upgrade_database_head_invokes_alembic_upgrade(self, monkeypatch):
        from alembic import command

        seen = {}

        monkeypatch.setattr(dbmod, "_build_alembic_config", lambda: "cfg")
        monkeypatch.setattr(
            command, "upgrade", lambda cfg, revision: seen.update(cfg=cfg, revision=revision)
        )

        dbmod._upgrade_database_head()

        assert seen == {"cfg": "cfg", "revision": "head"}

    def test_bootstrap_database_upgrades_existing_stamped_database(self, monkeypatch):
        seen = {}

        def fake_seed():
            seen["seeded"] = 12
            return 12

        monkeypatch.setattr(dbmod, "_user_table_names", lambda: {"designs", "tags"})
        monkeypatch.setattr(dbmod, "_current_alembic_revision", lambda: "0002_settings")
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: seen.__setitem__("upgraded", True)
        )
        monkeypatch.setattr(dbmod, "_seed_delivered_tags", fake_seed)

        status = dbmod.bootstrap_database()

        assert status == "upgraded"
        assert seen == {"upgraded": True, "seeded": 12}

    def test_bootstrap_database_upgrades_existing_legacy_unstamped_database(self, monkeypatch):
        seen = {}

        def fake_seed():
            seen["seeded"] = 7
            return 7

        monkeypatch.setattr(dbmod, "_user_table_names", lambda: {"designs", "tags", "settings"})
        monkeypatch.setattr(dbmod, "_current_alembic_revision", lambda: None)
        monkeypatch.setattr(
            dbmod, "_infer_revision_for_unstamped_database", lambda _tables: "0002_settings"
        )
        monkeypatch.setattr(
            dbmod,
            "_stamp_database_revision",
            lambda revision: seen.__setitem__("stamped", revision),
        )
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: seen.__setitem__("upgraded", True)
        )
        monkeypatch.setattr(dbmod, "_seed_delivered_tags", fake_seed)

        status = dbmod.bootstrap_database()

        assert status == "upgraded"
        assert seen == {"stamped": "0002_settings", "upgraded": True, "seeded": 7}

    def test_bootstrap_database_upgrades_when_no_recognised_baseline_exists(self, monkeypatch):
        seen = {}

        def fake_seed():
            seen["seeded"] = 3
            return 3

        monkeypatch.setattr(dbmod, "_user_table_names", lambda: {"mystery_table"})
        monkeypatch.setattr(dbmod, "_current_alembic_revision", lambda: None)
        monkeypatch.setattr(dbmod, "_infer_revision_for_unstamped_database", lambda _tables: None)
        monkeypatch.setattr(
            dbmod, "_upgrade_database_head", lambda: seen.__setitem__("upgraded", True)
        )
        monkeypatch.setattr(dbmod, "_seed_delivered_tags", fake_seed)

        status = dbmod.bootstrap_database()

        assert status == "upgraded"
        assert seen == {"upgraded": True, "seeded": 3}

    def test_database_module_reloads_non_sqlite_urls_without_sqlite_connect_args(self, monkeypatch):
        import sqlalchemy

        import src.config as cfg

        captured = {}
        original_url = cfg.DATABASE_URL

        def fake_create_engine(url, echo=False, **kwargs):
            captured["url"] = url
            captured["echo"] = echo
            captured["kwargs"] = kwargs

            class DummyEngine:
                pass

            return DummyEngine()

        with monkeypatch.context() as m:
            m.setattr(cfg, "DATABASE_URL", "postgresql://example.invalid/catalogue", raising=False)
            m.setattr(sqlalchemy, "create_engine", fake_create_engine)
            importlib.reload(dbmod)
            assert captured == {
                "url": "postgresql://example.invalid/catalogue",
                "echo": False,
                "kwargs": {},
            }

        monkeypatch.setattr(cfg, "DATABASE_URL", original_url, raising=False)
        importlib.reload(dbmod)
