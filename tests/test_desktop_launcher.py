"""
Tests for the desktop launcher and supporting utilities.

These tests run on all platforms (including Linux CI) and cover:
  - Dynamic free-port selection  (src/utils/ports.py)
  - Server readiness polling      (src/utils/ports.py)
  - APP_MODE / config paths       (src/config.py)
  - desktop_launcher helpers      (desktop_launcher.py)
"""

from __future__ import annotations

import os
import socket
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import pytest

# ---------------------------------------------------------------------------
# src/utils/ports -- find_free_port
# ---------------------------------------------------------------------------


class TestFindFreePort:
    def test_returns_an_integer_port(self):
        from src.utils.ports import find_free_port

        port = find_free_port()
        assert isinstance(port, int)
        assert 1 <= port <= 65535

    def test_port_is_bindable(self):
        """The returned port should actually be free to bind."""
        from src.utils.ports import find_free_port

        port = find_free_port()
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(("127.0.0.1", port))  # should not raise

    def test_raises_when_range_exhausted(self):
        """When every port in the range is occupied, OSError is raised."""
        from src.utils.ports import find_free_port

        # Simulate every port being busy by making bind always fail.
        with patch("socket.socket") as mock_sock_cls:
            mock_sock = MagicMock()
            mock_sock.__enter__ = lambda s: s
            mock_sock.__exit__ = MagicMock(return_value=False)
            mock_sock.bind.side_effect = OSError("address in use")
            mock_sock_cls.return_value = mock_sock

            with pytest.raises(OSError, match="No free port"):
                find_free_port(start=8100, end=8102)

    def test_skips_occupied_ports(self):
        """find_free_port should skip a port that is already bound."""
        from src.utils.ports import find_free_port

        # Bind a socket on an arbitrary port to occupy it.
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as occupied:
            occupied.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            occupied.bind(("127.0.0.1", 0))
            occupied_port = occupied.getsockname()[1]

            # Ask find_free_port to start from that occupied port.
            port = find_free_port(start=occupied_port, end=occupied_port + 10)
            assert port != occupied_port


# ---------------------------------------------------------------------------
# src/utils/ports -- wait_for_server
# ---------------------------------------------------------------------------


class _OkHandler(BaseHTTPRequestHandler):
    """Minimal HTTP handler that returns 200 OK for every request."""

    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, *args):  # suppress test output
        pass


def _start_test_http_server() -> tuple[HTTPServer, int]:
    server = HTTPServer(("127.0.0.1", 0), _OkHandler)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, port


class TestWaitForServer:
    def test_returns_true_when_server_ready(self):
        from src.utils.ports import wait_for_server

        srv, port = _start_test_http_server()
        try:
            result = wait_for_server(f"http://127.0.0.1:{port}/", timeout=5.0)
            assert result is True
        finally:
            srv.shutdown()

    def test_returns_false_on_timeout(self):
        from src.utils.ports import wait_for_server

        # Use a port that is guaranteed not to be serving.
        result = wait_for_server("http://127.0.0.1:1/", timeout=0.5, interval=0.1)
        assert result is False

    def test_polls_until_ready(self):
        """Server becomes available after a short delay; wait_for_server finds it."""
        from src.utils.ports import find_free_port, wait_for_server

        port = find_free_port()
        srv_container: list[HTTPServer] = []

        def delayed_start():
            time.sleep(0.3)
            srv = HTTPServer(("127.0.0.1", port), _OkHandler)
            srv_container.append(srv)
            srv.serve_forever()

        t = threading.Thread(target=delayed_start, daemon=True)
        t.start()

        result = wait_for_server(f"http://127.0.0.1:{port}/", timeout=5.0, interval=0.1)
        if srv_container:
            srv_container[0].shutdown()
        assert result is True


# ---------------------------------------------------------------------------
# src/config.py -- APP_MODE and data path selection
# ---------------------------------------------------------------------------


class TestAppModeConfig:
    def test_development_mode_uses_repo_relative_data(self, tmp_path, monkeypatch):
        """In development mode the DATABASE_URL should point inside the repo."""
        monkeypatch.setenv("APP_MODE", "development")
        monkeypatch.delenv("DATABASE_URL", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert "catalogue_dev.db" in cfg.DATABASE_URL

    def test_portable_mode_uses_repo_relative_data(self, tmp_path, monkeypatch):
        """In portable mode the DATABASE_URL should use catalogue.db (not _dev)."""
        monkeypatch.setenv("APP_MODE", "portable")
        monkeypatch.delenv("DATABASE_URL", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert "catalogue.db" in cfg.DATABASE_URL
        assert "catalogue_dev.db" not in cfg.DATABASE_URL

    def test_desktop_mode_uses_configurable_data_root(self, tmp_path, monkeypatch):
        """In desktop mode the database should live under the configured data root."""
        data_root = tmp_path / "CatalogueData"
        monkeypatch.setenv("APP_MODE", "desktop")
        monkeypatch.setenv("LOCALAPPDATA", str(tmp_path / "LocalAppData"))
        monkeypatch.setenv("EMBROIDERY_DATA_ROOT", str(data_root))
        monkeypatch.delenv("DATABASE_URL", raising=False)
        monkeypatch.delenv("DESIGNS_BASE_PATH", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert str(data_root) in cfg.DATABASE_URL
        assert str(data_root) in cfg.DESIGNS_BASE_PATH

    def test_desktop_mode_uses_storage_pointer_when_present(self, tmp_path, monkeypatch):
        """In desktop mode a stored pointer file should control where big data lives."""
        local_state = tmp_path / "LocalAppData"
        pointed_root = tmp_path / "BigDriveData"
        pointer_file = local_state / "EmbroideryCatalogue" / "storage.json"
        pointer_file.parent.mkdir(parents=True, exist_ok=True)
        escaped = str(pointed_root).replace("\\", "\\\\")
        pointer_file.write_text(
            f'{{"data_root": "{escaped}"}}',
        )

        monkeypatch.setenv("APP_MODE", "desktop")
        monkeypatch.setenv("LOCALAPPDATA", str(local_state))
        monkeypatch.delenv("EMBROIDERY_DATA_ROOT", raising=False)
        monkeypatch.delenv("DATABASE_URL", raising=False)
        monkeypatch.delenv("DESIGNS_BASE_PATH", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert str(pointed_root) in cfg.DATABASE_URL
        assert str(pointed_root) in cfg.DESIGNS_BASE_PATH

    def test_desktop_mode_logs_stay_in_localappdata(self, tmp_path, monkeypatch):
        """In desktop mode logs should remain under %LOCALAPPDATA% even when data root moves."""
        local_state = tmp_path / "LocalAppData"
        data_root = tmp_path / "CatalogueData"
        monkeypatch.setenv("APP_MODE", "desktop")
        monkeypatch.setenv("LOCALAPPDATA", str(local_state))
        monkeypatch.setenv("EMBROIDERY_DATA_ROOT", str(data_root))
        monkeypatch.delenv("DATABASE_URL", raising=False)
        monkeypatch.delenv("DESIGNS_BASE_PATH", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert str(local_state) in str(cfg.LOGS_DIR)
        assert str(data_root) not in str(cfg.LOGS_DIR)

    def test_desktop_mode_copies_legacy_data_to_new_root_when_empty(self, tmp_path, monkeypatch):
        """Existing LocalAppData catalogue content should be copied into a new desktop data root on first start."""
        local_state = tmp_path / "LocalAppData"
        state_root = local_state / "EmbroideryCatalogue"
        legacy_db = state_root / "database" / "catalogue.db"
        legacy_design = state_root / "MachineEmbroideryDesigns" / "Floral" / "rose.pes"
        target_root = tmp_path / "BigDriveData"

        legacy_db.parent.mkdir(parents=True, exist_ok=True)
        legacy_db.write_bytes(b"legacy-db")
        legacy_design.parent.mkdir(parents=True, exist_ok=True)
        legacy_design.write_text("rose design", encoding="utf-8")

        monkeypatch.setenv("APP_MODE", "desktop")
        monkeypatch.setenv("LOCALAPPDATA", str(local_state))
        monkeypatch.setenv("EMBROIDERY_DATA_ROOT", str(target_root))
        monkeypatch.delenv("DATABASE_URL", raising=False)
        monkeypatch.delenv("DESIGNS_BASE_PATH", raising=False)

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert (target_root / "database" / "catalogue.db").read_bytes() == b"legacy-db"
        assert (target_root / "MachineEmbroideryDesigns" / "Floral" / "rose.pes").read_text(
            encoding="utf-8"
        ) == "rose design"

    def test_external_launches_disabled_in_desktop_mode(self, monkeypatch):
        """desktop_launcher sets EMBROIDERY_DISABLE_EXTERNAL_OPEN=1 by default."""
        monkeypatch.setenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "1")

        import importlib

        import src.config as cfg

        importlib.reload(cfg)

        assert cfg.external_launches_disabled() is True


class TestConfigHelperFunctions:
    def test_frozen_mode_uses_executable_parent_as_app_root(self, tmp_path, monkeypatch):
        import importlib

        import src.config as cfg

        fake_exe = tmp_path / "Portable" / "EmbroideryCatalogue.exe"
        fake_exe.parent.mkdir(parents=True)
        fake_exe.write_text("", encoding="utf-8")

        with monkeypatch.context() as m:
            m.setattr(sys, "frozen", True, raising=False)
            m.setattr(sys, "executable", str(fake_exe))
            m.setenv("APP_MODE", "portable")
            m.delenv("DATABASE_URL", raising=False)
            importlib.reload(cfg)
            assert cfg._APP_ROOT == fake_exe.parent

        importlib.reload(cfg)

    def test_load_storage_pointer_returns_none_for_missing_invalid_and_blank_payloads(
        self, tmp_path
    ):
        import src.config as cfg

        missing = tmp_path / "missing.json"
        invalid = tmp_path / "invalid.json"
        invalid.write_text("{not-json", encoding="utf-8")
        blank = tmp_path / "blank.json"
        blank.write_text('{"data_root": "   "}', encoding="utf-8")
        wrong_shape = tmp_path / "wrong-shape.json"
        wrong_shape.write_text('["not", "a", "dict"]', encoding="utf-8")

        assert cfg._load_storage_pointer(missing) is None
        assert cfg._load_storage_pointer(invalid) is None
        assert cfg._load_storage_pointer(blank) is None
        assert cfg._load_storage_pointer(wrong_shape) is None

    def test_default_desktop_data_root_prefers_legacy_state_and_falls_back_safely(
        self, tmp_path, monkeypatch
    ):
        import src.config as cfg

        legacy_state = tmp_path / "LegacyState"
        (legacy_state / "database").mkdir(parents=True)
        (legacy_state / "database" / "catalogue.db").write_text("db", encoding="utf-8")
        assert cfg._default_desktop_data_root(legacy_state) == legacy_state

        fresh_state = tmp_path / "FreshState"
        if cfg._APP_ROOT.drive:
            expected = Path(cfg._APP_ROOT.drive + "\\") / "EmbroideryCatalogueData"
            assert cfg._default_desktop_data_root(fresh_state) == expected

        monkeypatch.setattr(cfg, "_APP_ROOT", Path("relative-root"))
        monkeypatch.setattr(Path, "home", lambda: tmp_path / "Home")
        assert (
            cfg._default_desktop_data_root(fresh_state)
            == tmp_path / "Home" / "EmbroideryCatalogueData"
        )

    def test_copy_missing_files_copies_single_file_and_merges_without_overwrite(self, tmp_path):
        import src.config as cfg

        single_source = tmp_path / "single.txt"
        single_source.write_text("fresh", encoding="utf-8")
        single_target = tmp_path / "copied" / "single.txt"

        cfg._copy_missing_files(single_source, single_target)

        assert single_target.read_text(encoding="utf-8") == "fresh"

        source_dir = tmp_path / "source-dir"
        (source_dir / "nested").mkdir(parents=True)
        (source_dir / "nested" / "existing.txt").write_text("new-value", encoding="utf-8")
        (source_dir / "nested" / "added.txt").write_text("added", encoding="utf-8")

        target_dir = tmp_path / "target-dir"
        (target_dir / "nested").mkdir(parents=True)
        (target_dir / "nested" / "existing.txt").write_text("keep-me", encoding="utf-8")

        cfg._copy_missing_files(source_dir, target_dir)

        assert (target_dir / "nested" / "existing.txt").read_text(encoding="utf-8") == "keep-me"
        assert (target_dir / "nested" / "added.txt").read_text(encoding="utf-8") == "added"

    def test_copy_managed_data_if_needed_returns_immediately_for_same_resolved_path(
        self, monkeypatch
    ):
        import src.config as cfg

        copied = []
        monkeypatch.setattr(cfg, "_copy_missing_files", lambda *args, **kwargs: copied.append(args))

        cfg.copy_managed_data_if_needed(Path("same-path"), Path("same-path"))

        assert copied == []

    def test_copy_managed_data_if_needed_returns_when_same_path_cannot_resolve(self, monkeypatch):
        import src.config as cfg

        copied = []

        def broken_resolve(self, *args, **kwargs):
            raise OSError("resolve failed")

        monkeypatch.setattr(Path, "resolve", broken_resolve)
        monkeypatch.setattr(cfg, "_copy_missing_files", lambda *args, **kwargs: copied.append(args))

        cfg.copy_managed_data_if_needed(Path("same-path"), Path("same-path"))

        assert copied == []

    def test_desktop_reload_tolerates_copy_and_mkdir_oserrors(self, tmp_path, monkeypatch):
        import importlib
        import shutil

        import src.config as cfg

        local_state = tmp_path / "LocalAppData"
        state_root = local_state / "EmbroideryCatalogue"
        legacy_db = state_root / "database" / "catalogue.db"
        legacy_db.parent.mkdir(parents=True, exist_ok=True)
        legacy_db.write_bytes(b"legacy-db")

        with monkeypatch.context() as m:
            m.setenv("APP_MODE", "desktop")
            m.setenv("LOCALAPPDATA", str(local_state))
            m.setenv("EMBROIDERY_DATA_ROOT", str(tmp_path / "NewDataRoot"))
            m.delenv("DATABASE_URL", raising=False)
            m.setattr(
                shutil,
                "copytree",
                lambda *_args, **_kwargs: (_ for _ in ()).throw(OSError("copy blocked")),
            )
            m.setattr(
                Path,
                "mkdir",
                lambda self, *args, **kwargs: (_ for _ in ()).throw(OSError("mkdir blocked")),
            )
            importlib.reload(cfg)
            assert cfg.APP_MODE == "desktop"
            assert str(tmp_path / "NewDataRoot") in cfg.DATABASE_URL

        importlib.reload(cfg)

    def test_env_flag_helpers_cover_false_and_native_dialogs(self, monkeypatch):
        import src.config as cfg

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.setenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "no")
        monkeypatch.setenv("EMBROIDERY_DISABLE_NATIVE_DIALOGS", "on")

        assert cfg._env_flag_enabled("EMBROIDERY_DISABLE_EXTERNAL_OPEN") is False
        assert cfg._env_flag_enabled("EMBROIDERY_DISABLE_NATIVE_DIALOGS") is True
        assert cfg.external_launches_disabled() is False
        assert cfg.native_dialogs_disabled() is True


# ---------------------------------------------------------------------------
# desktop_launcher.py -- helper functions (no server actually started)
# ---------------------------------------------------------------------------


class TestDesktopLauncherHelpers:
    def test_show_error_dialog_falls_back_to_stderr_when_tkinter_missing(self, capsys):
        """_show_error_dialog should not raise even when tkinter is unavailable."""
        import desktop_launcher

        with patch.dict(sys.modules, {"tkinter": None, "tkinter.messagebox": None}):
            desktop_launcher._show_error_dialog("Test Title", "Test Message")

        captured = capsys.readouterr()
        assert "Test Title" in captured.err or "Test Message" in captured.err

    def test_request_server_shutdown_is_safe_without_server(self):
        """_request_server_shutdown should not raise when no server is attached."""
        import desktop_launcher

        stop_event = threading.Event()
        # No _server attribute set -- should be a no-op
        desktop_launcher._request_server_shutdown(stop_event)

    def test_request_server_shutdown_sets_should_exit(self):
        """_request_server_shutdown sets should_exit on the stashed server."""
        import desktop_launcher

        mock_server = MagicMock()
        mock_server.should_exit = False

        stop_event = threading.Event()
        stop_event._server = mock_server

        desktop_launcher._request_server_shutdown(stop_event)

        assert mock_server.should_exit is True

    def test_start_server_records_startup_error(self):
        """_start_server should log and stash failures instead of failing silently."""
        import desktop_launcher

        class BrokenConfig:
            def __init__(self, *args, **kwargs):
                raise RuntimeError("boom")

        fake_uvicorn = SimpleNamespace(Config=BrokenConfig, Server=object)
        stop_event = threading.Event()

        with patch.dict(sys.modules, {"uvicorn": fake_uvicorn}):
            desktop_launcher._start_server("127.0.0.1", 9999, stop_event)

        assert isinstance(getattr(stop_event, "_startup_error", None), RuntimeError)
        assert str(stop_event._startup_error) == "boom"

    def test_start_server_uses_window_safe_uvicorn_config(self):
        """The packaged desktop build should not let Uvicorn reconfigure console logging."""
        import desktop_launcher

        captured = {}

        class FakeConfig:
            def __init__(self, *args, **kwargs):
                captured.update(kwargs)

        class FakeServer:
            def __init__(self, config):
                self.config = config
                self.should_exit = False
                self.install_signal_handlers = None

            def run(self):
                return None

        fake_uvicorn = SimpleNamespace(Config=FakeConfig, Server=FakeServer)
        stop_event = threading.Event()

        with patch.dict(sys.modules, {"uvicorn": fake_uvicorn}):
            desktop_launcher._start_server("127.0.0.1", 9999, stop_event)

        assert captured["log_config"] is None
        assert captured["loop"] == "asyncio"
        assert captured["http"] == "h11"

    def test_open_window_falls_back_to_browser_when_webview_missing(self, monkeypatch):
        """When pywebview is not installed, _open_window opens an external browser."""
        import desktop_launcher

        opened = []

        def fake_open(url):
            opened.append(url)

        with patch.dict(sys.modules, {"webview": None}):
            with patch("webbrowser.open", side_effect=fake_open):
                with patch.object(
                    desktop_launcher, "_run_browser_fallback", return_value=None
                ) as fallback:
                    stop_event = threading.Event()
                    desktop_launcher._open_window("http://127.0.0.1:9999", stop_event)

        assert opened == ["http://127.0.0.1:9999"]
        fallback.assert_called_once()

    def test_browser_fallback_handles_lost_stdin(self):
        """Browser fallback should not crash in a windowed build with no stdin."""
        import desktop_launcher

        stop_event = threading.Event()

        with patch.dict(sys.modules, {"tkinter": None, "tkinter.ttk": None}):
            with patch("builtins.input", side_effect=RuntimeError("lost sys.stdin")):
                with patch.object(stop_event, "wait", return_value=True) as wait_mock:
                    desktop_launcher._run_browser_fallback("http://127.0.0.1:9999", stop_event)

        wait_mock.assert_called_once()

    def test_resolve_app_icon_prefers_frozen_executable_icon(self, tmp_path, monkeypatch):
        import desktop_launcher

        exe_dir = tmp_path / "dist"
        exe_path = exe_dir / "EmbroideryCatalogue.exe"
        exe_path.parent.mkdir(parents=True, exist_ok=True)
        exe_path.write_text("", encoding="utf-8")
        icon_path = exe_dir / "static" / "icons" / "app-icon.ico"
        icon_path.parent.mkdir(parents=True, exist_ok=True)
        icon_path.write_text("icon", encoding="utf-8")

        monkeypatch.setattr(desktop_launcher.sys, "frozen", True, raising=False)
        monkeypatch.setattr(desktop_launcher.sys, "executable", str(exe_path))

        assert desktop_launcher._resolve_app_icon() == icon_path

    def test_prepare_runtime_environment_sets_desktop_defaults_and_bundle_path(
        self, tmp_path, monkeypatch
    ):
        import desktop_launcher

        bundle_dir = tmp_path / "bundle"
        bundle_dir.mkdir()
        exe_path = bundle_dir / "EmbroideryCatalogue.exe"
        exe_path.write_text("", encoding="utf-8")
        bundle_dir_str = str(bundle_dir)
        original_sys_path = list(sys.path)
        seen = {}

        monkeypatch.delenv("APP_MODE", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(desktop_launcher.sys, "frozen", True, raising=False)
        monkeypatch.setattr(desktop_launcher.sys, "executable", str(exe_path))
        monkeypatch.setattr(desktop_launcher.sys, "_MEIPASS", bundle_dir_str, raising=False)
        monkeypatch.setattr(
            desktop_launcher.os, "chdir", lambda path: seen.setdefault("chdir", path)
        )

        try:
            if bundle_dir_str in sys.path:
                sys.path.remove(bundle_dir_str)

            desktop_launcher._prepare_runtime_environment()

            assert os.environ["APP_MODE"] == "desktop"
            assert os.environ["EMBROIDERY_DISABLE_EXTERNAL_OPEN"] == "1"
            assert sys.path[0] == bundle_dir_str
            assert seen["chdir"] == bundle_dir
        finally:
            sys.path[:] = original_sys_path

    def test_configure_logging_creates_log_file_and_tracks_path(self, tmp_path, monkeypatch):
        import desktop_launcher
        import src.config as cfg

        root_logger = desktop_launcher.logging.getLogger()
        existing_handlers = list(root_logger.handlers)

        monkeypatch.setattr(cfg, "APP_MODE", "desktop", raising=False)
        monkeypatch.setattr(cfg, "LOGS_DIR", tmp_path / "logs", raising=False)

        app_mode, log_file = desktop_launcher._configure_logging()

        try:
            assert app_mode == "desktop"
            assert log_file == tmp_path / "logs" / "app.log"
            assert desktop_launcher._log_file == log_file
            assert log_file.parent.exists()
        finally:
            for handler in list(root_logger.handlers):
                if handler not in existing_handlers:
                    root_logger.removeHandler(handler)
                    handler.close()

    def test_open_window_uses_webview_when_available(self, monkeypatch):
        import desktop_launcher

        created = {}
        started = {}
        fake_icon = Path(r"C:\icons\app-icon.ico")
        fake_webview = SimpleNamespace(
            create_window=lambda **kwargs: created.update(kwargs),
            start=lambda **kwargs: started.update(kwargs),
        )

        monkeypatch.setattr(desktop_launcher, "_resolve_app_icon", lambda: fake_icon)

        with patch.dict(sys.modules, {"webview": fake_webview}):
            desktop_launcher._open_window("http://127.0.0.1:9999", threading.Event())

        assert created["title"] == "Embroidery Catalogue"
        assert created["url"] == "http://127.0.0.1:9999"
        assert started == {"debug": False, "icon": str(fake_icon)}

    def test_show_error_dialog_uses_tkinter_messagebox_when_available(self):
        import desktop_launcher

        seen = {}

        class FakeRoot:
            def withdraw(self):
                seen["withdrew"] = True

            def destroy(self):
                seen["destroyed"] = True

        fake_messagebox = SimpleNamespace(
            showerror=lambda title, message: seen.update(title=title, message=message)
        )
        fake_tk = SimpleNamespace(Tk=lambda: FakeRoot(), messagebox=fake_messagebox)

        with patch.dict(sys.modules, {"tkinter": fake_tk, "tkinter.messagebox": fake_messagebox}):
            desktop_launcher._show_error_dialog("Dialog Title", "Dialog Message")

        assert seen["title"] == "Dialog Title"
        assert seen["message"] == "Dialog Message"
        assert seen["withdrew"] is True
        assert seen["destroyed"] is True


class TestDesktopLauncherMain:
    def test_main_happy_path_bootstraps_opens_window_and_shuts_down(self, monkeypatch, tmp_path):
        import desktop_launcher
        import src.database as dbmod
        from src.utils import ports

        seen = {}
        log_file = tmp_path / "app.log"

        class FakeThread:
            def __init__(self, target=None, args=(), daemon=None, name=None):
                self.target = target
                self.args = args
                self.daemon = daemon
                self.name = name
                self.started = False
                self.join_timeout = None
                seen["thread"] = self

            def start(self):
                self.started = True

            def join(self, timeout=None):
                self.join_timeout = timeout

            def is_alive(self):
                return True

        monkeypatch.setattr(
            desktop_launcher,
            "_prepare_runtime_environment",
            lambda: seen.setdefault("prepared", True),
        )
        monkeypatch.setattr(desktop_launcher, "_configure_logging", lambda: ("desktop", log_file))
        monkeypatch.setattr(
            dbmod, "bootstrap_database", lambda: seen.setdefault("bootstrap", "created")
        )
        monkeypatch.setattr(
            ports,
            "find_free_port",
            lambda host: seen.update(host=host) or 8765,
        )
        monkeypatch.setattr(
            ports,
            "wait_for_server",
            lambda url, timeout=30.0: seen.update(health_url=url) or True,
        )
        monkeypatch.setattr(desktop_launcher.threading, "Thread", FakeThread)
        monkeypatch.setattr(
            desktop_launcher,
            "_open_window",
            lambda url, stop_event: seen.setdefault("open_url", url),
        )
        monkeypatch.setattr(
            desktop_launcher,
            "_request_server_shutdown",
            lambda stop_event: seen.setdefault("shutdown", True),
        )

        desktop_launcher.main()

        assert seen["prepared"] is True
        assert seen["bootstrap"] == "created"
        assert seen["host"] == "127.0.0.1"
        assert seen["health_url"] == "http://127.0.0.1:8765/health"
        assert seen["open_url"] == "http://127.0.0.1:8765"
        assert seen["shutdown"] is True
        assert seen["thread"].started is True
        assert seen["thread"].join_timeout == 10

    def test_main_shows_error_dialog_when_database_bootstrap_fails(self, monkeypatch, tmp_path):
        import desktop_launcher
        import src.database as dbmod

        shown = {}
        log_file = tmp_path / "app.log"

        monkeypatch.setattr(desktop_launcher, "_prepare_runtime_environment", lambda: None)
        monkeypatch.setattr(desktop_launcher, "_configure_logging", lambda: ("desktop", log_file))
        monkeypatch.setattr(
            dbmod, "bootstrap_database", MagicMock(side_effect=RuntimeError("db boom"))
        )
        monkeypatch.setattr(
            desktop_launcher,
            "_show_error_dialog",
            lambda title, message: shown.update(title=title, message=message),
        )

        with pytest.raises(SystemExit) as exc:
            desktop_launcher.main()

        assert exc.value.code == 1
        assert shown["title"] == "Embroidery Catalogue -- Startup Error"
        assert "database could not be initialised" in shown["message"].lower()
        assert str(log_file) in shown["message"]

    def test_main_shows_error_dialog_when_no_free_port_is_available(self, monkeypatch):
        import desktop_launcher
        import src.database as dbmod
        from src.utils import ports

        shown = {}

        monkeypatch.setattr(desktop_launcher, "_prepare_runtime_environment", lambda: None)
        monkeypatch.setattr(
            desktop_launcher, "_configure_logging", lambda: ("desktop", Path("app.log"))
        )
        monkeypatch.setattr(dbmod, "bootstrap_database", lambda: "created")
        monkeypatch.setattr(ports, "find_free_port", MagicMock(side_effect=OSError("busy")))
        monkeypatch.setattr(
            desktop_launcher,
            "_show_error_dialog",
            lambda title, message: shown.update(title=title, message=message),
        )

        with pytest.raises(SystemExit) as exc:
            desktop_launcher.main()

        assert exc.value.code == 1
        assert "could not find a free port" in shown["message"].lower()

    def test_main_requests_shutdown_and_exits_when_server_never_becomes_ready(self, monkeypatch):
        import desktop_launcher
        import src.database as dbmod
        from src.utils import ports

        shown = {}
        seen = {}

        class FakeThread:
            def __init__(self, target=None, args=(), daemon=None, name=None):
                self.target = target
                self.args = args
                self.daemon = daemon
                self.name = name
                self.started = False
                seen["thread"] = self

            def start(self):
                self.started = True

            def join(self, timeout=None):
                seen["join_timeout"] = timeout

            def is_alive(self):
                return False

        monkeypatch.setattr(desktop_launcher, "_prepare_runtime_environment", lambda: None)
        monkeypatch.setattr(
            desktop_launcher, "_configure_logging", lambda: ("desktop", Path("app.log"))
        )
        monkeypatch.setattr(dbmod, "bootstrap_database", lambda: "created")
        monkeypatch.setattr(ports, "find_free_port", lambda host: 8765)
        monkeypatch.setattr(ports, "wait_for_server", lambda url, timeout=30.0: False)
        monkeypatch.setattr(desktop_launcher.threading, "Thread", FakeThread)
        monkeypatch.setattr(
            desktop_launcher,
            "_show_error_dialog",
            lambda title, message: shown.update(title=title, message=message),
        )
        monkeypatch.setattr(
            desktop_launcher,
            "_request_server_shutdown",
            lambda stop_event: seen.setdefault("shutdown", True),
        )

        with pytest.raises(SystemExit) as exc:
            desktop_launcher.main()

        assert exc.value.code == 1
        assert seen["thread"].started is True
        assert seen["shutdown"] is True
        assert "did not start in time" in shown["message"].lower()


# ---------------------------------------------------------------------------
# Packaging spec and build script are present in the repo
# ---------------------------------------------------------------------------


ROOT = Path(__file__).resolve().parents[1]


def test_pyinstaller_spec_exists():
    assert (ROOT / "EmbroideryCatalogue.spec").exists(), "EmbroideryCatalogue.spec is missing"


def test_pyinstaller_spec_bundles_tags_csv():
    spec_text = (ROOT / "EmbroideryCatalogue.spec").read_text(encoding="utf-8")
    assert "tags.csv" in spec_text, "EmbroideryCatalogue.spec must bundle data/tags.csv"


def test_pyinstaller_spec_bundles_user_documents():
    spec_text = (ROOT / "EmbroideryCatalogue.spec").read_text(encoding="utf-8")
    assert "DISCLAIMER.html" in spec_text, "EmbroideryCatalogue.spec must bundle DISCLAIMER.html"
    assert "PRIVACY.md" in spec_text, "EmbroideryCatalogue.spec must bundle PRIVACY.md"
    assert "SECURITY.md" in spec_text, "EmbroideryCatalogue.spec must bundle SECURITY.md"
    assert (
        "docs" in spec_text and "AI_TAGGING.md" in spec_text
    ), "EmbroideryCatalogue.spec must bundle docs/AI_TAGGING.md"


def test_build_script_exists():
    assert (ROOT / "build_desktop.bat").exists(), "build_desktop.bat is missing"


def test_installer_script_exists():
    assert (
        ROOT / "installer" / "EmbroideryCatalogue.iss"
    ).exists(), "installer/EmbroideryCatalogue.iss is missing"


def test_desktop_launcher_exists():
    assert (ROOT / "desktop_launcher.py").exists(), "desktop_launcher.py is missing"
