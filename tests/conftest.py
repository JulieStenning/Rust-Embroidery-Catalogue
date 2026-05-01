"""
Test configuration and shared fixtures.

Uses a separate SQLite database for tests (data/test_catalogue.db).
Set DATABASE_URL_TEST environment variable to override.
"""

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine, event
from sqlalchemy.orm import sessionmaker

from src.config import DATABASE_URL_TEST
from src.database import Base, get_db
from src.main import app
from src.services import settings_service as settings_svc

TEST_DATABASE_URL = DATABASE_URL_TEST

_is_sqlite = TEST_DATABASE_URL.startswith("sqlite")

if _is_sqlite:
    test_engine = create_engine(
        TEST_DATABASE_URL,
        echo=False,
        connect_args={"check_same_thread": False},
    )

    @event.listens_for(test_engine, "connect")
    def _set_sqlite_pragmas(dbapi_connection, connection_record):
        cursor = dbapi_connection.cursor()
        cursor.execute("PRAGMA journal_mode=WAL")
        cursor.execute("PRAGMA foreign_keys=ON")
        cursor.close()

        # Register the same custom functions used by the production engine
        def _parent_folder(path):
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

        def _folder_path(path):
            if not path:
                return ""
            path = path.replace("/", "\\")
            path = path.rstrip("\\")
            idx = path.rfind("\\")
            if idx == -1:
                return ""
            return path[:idx]

        dbapi_connection.create_function("folder_path", 1, _folder_path)

        def _file_extension(path):
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
    test_engine = create_engine(TEST_DATABASE_URL, echo=False)

TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=test_engine)


@pytest.fixture(scope="session", autouse=True)
def create_test_tables():
    """Create all tables once per test session, drop them after."""
    Base.metadata.create_all(bind=test_engine)
    yield
    Base.metadata.drop_all(bind=test_engine)


@pytest.fixture()
def db():
    """Yield a fresh database session that is rolled back after each test."""
    connection = test_engine.connect()
    transaction = connection.begin()
    session = TestingSessionLocal(bind=connection)
    try:
        yield session
    finally:
        session.close()
        transaction.rollback()
        connection.close()


@pytest.fixture()
def client(db):
    """FastAPI test client with the DB dependency overridden."""
    settings_svc.set_setting(db, settings_svc.SETTING_DISCLAIMER_ACCEPTED, "true")

    def override_get_db():
        try:
            yield db
        finally:
            pass

    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()


@pytest.fixture()
def client_unaccepted(db):
    """Test client for routes that should exercise the first-use disclaimer gate."""

    def override_get_db():
        try:
            yield db
        finally:
            pass

    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()
