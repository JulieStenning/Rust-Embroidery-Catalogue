import io
import queue
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock

import pytest

import portable_launcher


def test_prepare_portable_target_copies_distribution_docs():
    script = (Path(__file__).resolve().parents[1] / "prepare_portable_target.bat").read_text(
        encoding="utf-8"
    )

    for filename in [
        "DISCLAIMER.html",
        "PRIVACY.md",
        "SECURITY.md",
        "THIRD_PARTY_NOTICES.md",
        "LICENSE",
    ]:
        assert filename in script


def test_prepare_portable_target_uses_env_example_not_live_env():
    """The batch script must copy .env.example to the target, never the live .env."""
    script = (Path(__file__).resolve().parents[1] / "prepare_portable_target.bat").read_text(
        encoding="utf-8"
    )

    # Must copy from .env.example
    assert ".env.example" in script
    # Must NOT read GOOGLE_API_KEY out of the live .env and write it to the target
    assert "GOOGLE_API_KEY" not in script


def test_prepare_portable_target_warns_when_live_env_present():
    script = (Path(__file__).resolve().parents[1] / "prepare_portable_target.bat").read_text(
        encoding="utf-8"
    )

    assert "Source .env detected" in script
    assert "Live secrets are never copied to portable targets" in script


class DummyVar:
    def __init__(self, value):
        self.value = value

    def get(self):
        return self.value

    def set(self, value):
        self.value = value


class DummyLauncher:
    def __init__(self, root="F:\\", designs="J:\\Designs", skip=False):
        self._selected_root = DummyVar(root)
        self._selected_designs = DummyVar(designs)
        self._no_designs = DummyVar(skip)
        self._designs_entry = MagicMock()
        self._designs_browse_btn = MagicMock()
        self._ok_btn = MagicMock()
        self._cancel_btn = MagicMock()
        self._copy_btn = MagicMock()
        self._bat_path = Path("prepare_portable_target.bat")
        self._deploy_script_path = Path("scripts") / "portable_deploy.py"
        self._run_in_progress = False
        self._output_queue = None
        self._append_output = MagicMock()
        self._run_batch = MagicMock()
        self._run_batch_worker = MagicMock()
        self._drain_output_queue = MagicMock()
        self.after = MagicMock()
        self.clipboard_clear = MagicMock()
        self.clipboard_append = MagicMock()
        self._output = MagicMock()


@pytest.mark.parametrize(
    "input_path,expected",
    [
        ("F:\\", "F:"),
        ("F:/", "F:"),
        ("F:", "F:"),
        (r"\\\\server\\share", r"\\\\server\\share"),
        (r"\\\\server\\share\\", r"\\\\server\\share"),
        ("/Volumes/MyDrive/", "/Volumes/MyDrive"),
        ("/Volumes/MyDrive", "/Volumes/MyDrive"),
        ("/", "/"),
        ("/Users/user/EmbroideryPortable/", "/Users/user/EmbroideryPortable"),
    ],
)
def test_normalise_root(input_path, expected):
    assert portable_launcher._normalise_root(input_path) == expected


def test_is_valid_root_drive():
    assert portable_launcher._is_valid_root("F:")
    assert not portable_launcher._is_valid_root("F:/folder")


def test_is_valid_root_unc():
    assert portable_launcher._is_valid_root(r"\\\\server\\share")
    assert portable_launcher._is_valid_root(r"\\\\server\\share\\folder")


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX-only path validation")
def test_is_valid_root_posix():
    assert portable_launcher._is_valid_root("/Volumes/MyDrive")
    assert portable_launcher._is_valid_root("/Users/user/EmbroideryPortable")
    assert not portable_launcher._is_valid_root("relative/path")
    assert not portable_launcher._is_valid_root("")


def test_check_path_writable(tmp_path):
    writable, err = portable_launcher._check_path_writable(str(tmp_path))
    assert writable
    assert err == ""


def test_is_removable_drive_false(tmp_path):
    assert not portable_launcher._is_removable_drive(str(tmp_path))


def test_is_removable_drive_true_with_mocked_ctypes(monkeypatch):
    fake_kernel32 = SimpleNamespace(GetDriveTypeW=lambda path: 2)
    fake_ctypes = SimpleNamespace(windll=SimpleNamespace(kernel32=fake_kernel32))
    monkeypatch.setitem(__import__("sys").modules, "ctypes", fake_ctypes)
    monkeypatch.setattr(portable_launcher.sys, "platform", "win32")

    assert portable_launcher._is_removable_drive("F:") is True


def test_is_removable_drive_macos_volume(monkeypatch):
    """On macOS, /Volumes/* paths (non-boot volumes) are treated as removable."""
    monkeypatch.setattr(portable_launcher.sys, "platform", "darwin")

    assert portable_launcher._is_removable_drive("/Volumes/MyUSBDrive") is True
    assert portable_launcher._is_removable_drive("/Volumes/MySDCard") is True
    assert portable_launcher._is_removable_drive("/Volumes/Macintosh HD") is False
    assert portable_launcher._is_removable_drive("/Users/user/portable") is False
    assert portable_launcher._is_removable_drive("/tmp") is False


def test_normalise_designs_source(tmp_path):
    folder = tmp_path / "designs"
    folder.mkdir()
    assert portable_launcher._normalise_designs_source(str(folder)) == str(folder)


def test_validate_designs_source(tmp_path):
    folder = tmp_path / "designs"
    folder.mkdir()
    assert portable_launcher._validate_designs_source(str(folder))
    assert not portable_launcher._validate_designs_source(str(folder / "missing"))


def test_read_registry_value_returns_fallback_without_winreg(monkeypatch, tmp_path):
    monkeypatch.setattr(portable_launcher, "_winreg_module", None)
    # No settings file → should return the fallback.
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: tmp_path / "settings.json")
    assert portable_launcher._read_registry_value("LastDeploymentRoot", "F:\\") == "F:\\"


def test_read_registry_value_uses_registry_when_available(monkeypatch):
    class FakeKey:
        def __enter__(self):
            return self

        def __exit__(self, exc_type, exc, tb):
            return False

    fake_winreg = SimpleNamespace(
        HKEY_CURRENT_USER=object(),
        OpenKey=lambda hive, key: FakeKey(),
        QueryValueEx=lambda key, value_name: ("Z:\\", 1),
    )
    monkeypatch.setattr(portable_launcher, "_winreg_module", fake_winreg)

    assert portable_launcher._read_registry_value("LastDeploymentRoot", "F:\\") == "Z:\\"


def test_read_registry_value_falls_back_to_legacy_last_sd_root(monkeypatch):
    """Existing users with LastSdRoot registry entry should get their path prefilled."""

    class FakeKey:
        def __enter__(self):
            return self

        def __exit__(self, exc_type, exc, tb):
            return False

    def fake_query(key, value_name):
        if value_name == portable_launcher.REGISTRY_VALUE_ROOT_LEGACY:
            return ("E:\\", 1)
        raise OSError("value not found")

    fake_winreg = SimpleNamespace(
        HKEY_CURRENT_USER=object(),
        OpenKey=lambda hive, key: FakeKey(),
        QueryValueEx=fake_query,
    )
    monkeypatch.setattr(portable_launcher, "_winreg_module", fake_winreg)

    # Helper function should migrate transparently: LastDeploymentRoot not set, legacy provides value
    result = portable_launcher._get_last_root_with_legacy_fallback()
    assert result == "E:\\"


def test_write_registry_value_handles_oserror(monkeypatch, capsys):
    def raise_oserror(*args, **kwargs):
        raise OSError("registry unavailable")

    fake_winreg = SimpleNamespace(
        HKEY_CURRENT_USER=object(),
        REG_SZ=1,
        CreateKey=raise_oserror,
        SetValueEx=lambda *args, **kwargs: None,
    )
    monkeypatch.setattr(portable_launcher, "_winreg_module", fake_winreg)

    portable_launcher._write_registry_value("LastDeploymentRoot", "F:\\")

    assert "Could not write to Registry" in capsys.readouterr().err


# ---------------------------------------------------------------------------
# File-based settings persistence (non-Windows / macOS)
# ---------------------------------------------------------------------------

def test_write_settings_value_creates_json_file(monkeypatch, tmp_path):
    settings_path = tmp_path / "settings.json"
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: settings_path)
    portable_launcher._write_settings_value("TestKey", "TestValue")
    assert settings_path.exists()


def test_read_settings_value_returns_written_value(monkeypatch, tmp_path):
    settings_path = tmp_path / "settings.json"
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: settings_path)
    portable_launcher._write_settings_value("TestKey", "TestValue")
    assert portable_launcher._read_settings_value("TestKey", "default") == "TestValue"


def test_read_settings_value_returns_fallback_when_file_missing(monkeypatch, tmp_path):
    settings_path = tmp_path / "settings.json"
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: settings_path)
    assert portable_launcher._read_settings_value("MissingKey", "fallback") == "fallback"


def test_write_registry_value_uses_settings_file_on_non_windows(monkeypatch, tmp_path):
    """On non-Windows (winreg unavailable), write should go to the JSON settings file."""
    settings_path = tmp_path / "settings.json"
    monkeypatch.setattr(portable_launcher, "_winreg_module", None)
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: settings_path)
    portable_launcher._write_registry_value("LastDeploymentRoot", "/Volumes/MyDrive")
    assert portable_launcher._read_settings_value("LastDeploymentRoot", "default") == "/Volumes/MyDrive"


def test_read_registry_value_reads_settings_file_on_non_windows(monkeypatch, tmp_path):
    """On non-Windows (winreg unavailable), read should fall back to the JSON settings file."""
    settings_path = tmp_path / "settings.json"
    monkeypatch.setattr(portable_launcher, "_winreg_module", None)
    monkeypatch.setattr(portable_launcher, "_settings_file_path", lambda: settings_path)
    portable_launcher._write_settings_value("LastDeploymentRoot", "/Volumes/MyDrive")
    assert portable_launcher._read_registry_value("LastDeploymentRoot", "default") == "/Volumes/MyDrive"


def test_get_batch_path_uses_executable_when_frozen(monkeypatch):
    monkeypatch.setattr(portable_launcher.sys, "frozen", True, raising=False)
    monkeypatch.setattr(
        portable_launcher.sys, "executable", r"D:\Portable\EmbroiderySdLauncher.exe"
    )

    assert portable_launcher.get_batch_path() == r"D:\Portable\prepare_portable_target.bat"


def test_get_batch_path_uses_script_path_when_not_frozen(monkeypatch):
    monkeypatch.setattr(portable_launcher.sys, "frozen", False, raising=False)
    monkeypatch.setattr(portable_launcher, "__file__", r"D:\\Workspace\\portable_launcher.py")

    assert portable_launcher.get_batch_path() == r"D:\Workspace\prepare_portable_target.bat"


def test_on_skip_designs_toggle_disables_fields():
    launcher = DummyLauncher(skip=True)

    portable_launcher.SdLauncherApp._on_skip_designs_toggle(launcher)

    launcher._designs_entry.configure.assert_called_once_with(state="disabled")
    launcher._designs_browse_btn.configure.assert_called_once_with(state="disabled")


def test_browse_root_updates_selected_path(monkeypatch):
    launcher = DummyLauncher(root="missing")
    monkeypatch.setattr(portable_launcher.os.path, "exists", lambda path: False)
    monkeypatch.setattr(
        portable_launcher.filedialog, "askdirectory", lambda **kwargs: r"D:\\SD Card\\"
    )

    portable_launcher.SdLauncherApp._browse_root(launcher)

    assert launcher._selected_root.get() == r"D:\SD Card"


def test_browse_designs_updates_selected_path(monkeypatch):
    launcher = DummyLauncher(designs="missing")
    monkeypatch.setattr(portable_launcher.os.path, "exists", lambda path: False)
    monkeypatch.setattr(
        portable_launcher.filedialog, "askdirectory", lambda **kwargs: r"C:\\Designs\\"
    )

    portable_launcher.SdLauncherApp._browse_designs(launcher)

    assert launcher._selected_designs.get() == r"C:\Designs"


def test_on_ok_shows_error_for_empty_root(monkeypatch):
    launcher = DummyLauncher(root="   ")
    showerror = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", showerror)

    portable_launcher.SdLauncherApp._on_ok(launcher)

    showerror.assert_called_once()
    launcher._run_batch.assert_not_called()


def test_on_ok_rejects_invalid_path(monkeypatch):
    launcher = DummyLauncher(root="not-a-root")
    monkeypatch.setattr(portable_launcher, "_normalise_root", lambda path: "not-a-root")
    monkeypatch.setattr(portable_launcher, "_is_valid_root", lambda path: False)
    showerror = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", showerror)

    portable_launcher.SdLauncherApp._on_ok(launcher)

    assert showerror.call_args[0][0] == "Invalid path"
    launcher._run_batch.assert_not_called()


def test_on_ok_stops_when_designs_source_missing(monkeypatch):
    launcher = DummyLauncher(root="F:\\", designs="C:\\missing", skip=False)
    monkeypatch.setattr(portable_launcher, "_normalise_root", lambda path: "F:")
    monkeypatch.setattr(portable_launcher, "_is_valid_root", lambda path: True)
    monkeypatch.setattr(portable_launcher.os.path, "exists", lambda path: True)
    monkeypatch.setattr(portable_launcher, "_check_path_writable", lambda root: (True, ""))
    monkeypatch.setattr(portable_launcher, "_is_removable_drive", lambda root: True)
    monkeypatch.setattr(portable_launcher.os.path, "isdir", lambda path: False)
    showerror = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", showerror)

    portable_launcher.SdLauncherApp._on_ok(launcher)

    assert showerror.call_args[0][0] == "Designs source not found"
    launcher._run_batch.assert_not_called()


def test_on_ok_runs_batch_when_inputs_are_valid(monkeypatch):
    launcher = DummyLauncher(root="F:\\", designs="C:\\Designs", skip=False)
    writes = []

    monkeypatch.setattr(portable_launcher, "_normalise_root", lambda path: "F:")
    monkeypatch.setattr(portable_launcher, "_is_valid_root", lambda path: True)
    monkeypatch.setattr(portable_launcher.os.path, "exists", lambda path: True)
    monkeypatch.setattr(portable_launcher, "_check_path_writable", lambda root: (True, ""))
    monkeypatch.setattr(portable_launcher, "_is_removable_drive", lambda root: True)
    monkeypatch.setattr(portable_launcher.os.path, "isdir", lambda path: True)
    monkeypatch.setattr(
        portable_launcher, "_write_registry_value", lambda name, value: writes.append((name, value))
    )

    portable_launcher.SdLauncherApp._on_ok(launcher)

    assert (portable_launcher.REGISTRY_VALUE_ROOT, "F:") in writes
    assert (portable_launcher.REGISTRY_VALUE_DESIGNS, "C:\\Designs") in writes
    launcher._run_batch.assert_called_once_with("F:", "C:\\Designs", False)


def test_run_batch_sets_up_worker_and_output_queue(monkeypatch):
    launcher = DummyLauncher()
    thread = MagicMock()
    monkeypatch.setattr(portable_launcher.threading, "Thread", lambda **kwargs: thread)

    portable_launcher.SdLauncherApp._run_batch(launcher, "F:", "C:\\Designs", False)

    assert isinstance(launcher._output_queue, queue.Queue)
    launcher._ok_btn.configure.assert_called_once_with(state="disabled")
    thread.start.assert_called_once()
    launcher.after.assert_called_once_with(50, launcher._drain_output_queue)


def test_run_batch_worker_queues_output_and_done(monkeypatch):
    launcher = DummyLauncher()
    launcher._output_queue = queue.Queue()
    proc = SimpleNamespace(
        stdout=io.StringIO("line one\nline two\n"),
        wait=lambda: 0,
        returncode=0,
    )
    monkeypatch.setattr(portable_launcher.subprocess, "Popen", lambda *args, **kwargs: proc)

    portable_launcher.SdLauncherApp._run_batch_worker(
        launcher, ["prepare_portable_target.bat", "F:"], "F:"
    )

    items = []
    while not launcher._output_queue.empty():
        items.append(launcher._output_queue.get())

    assert ("line", "line one\n") in items
    assert ("line", "line two\n") in items
    assert ("done", 0, "F:") in items


def test_run_batch_worker_reports_launch_error(monkeypatch):
    launcher = DummyLauncher()
    launcher._output_queue = queue.Queue()
    monkeypatch.setattr(
        portable_launcher.subprocess, "Popen", MagicMock(side_effect=OSError("boom"))
    )

    portable_launcher.SdLauncherApp._run_batch_worker(
        launcher, ["prepare_portable_target.bat", "F:"], "F:"
    )

    assert launcher._output_queue.get() == ("launch_error", "boom")


def test_drain_output_queue_handles_success(monkeypatch):
    launcher = DummyLauncher()
    launcher._run_in_progress = True
    launcher._output_queue = queue.Queue()
    launcher._output_queue.put(("line", "hello\n"))
    launcher._output_queue.put(("done", 0, "F:"))
    showinfo = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showinfo", showinfo)
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", MagicMock())
    writes = []
    monkeypatch.setattr(
        portable_launcher, "_write_registry_value", lambda name, value: writes.append((name, value))
    )

    portable_launcher.SdLauncherApp._drain_output_queue(launcher)

    launcher._append_output.assert_any_call("hello\n")
    assert (portable_launcher.REGISTRY_VALUE_ROOT, "F:") in writes
    showinfo.assert_called_once()
    launcher._ok_btn.configure.assert_called_once_with(state="normal")
    launcher._cancel_btn.configure.assert_called_once_with(text="Close")
    launcher._copy_btn.configure.assert_called_once_with(state="normal")
    assert launcher._output_queue is None


def test_drain_output_queue_handles_launch_error(monkeypatch):
    launcher = DummyLauncher()
    launcher._run_in_progress = True
    launcher._output_queue = queue.Queue()
    launcher._output_queue.put(("launch_error", "bad launch"))
    showerror = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", showerror)

    portable_launcher.SdLauncherApp._drain_output_queue(launcher)

    showerror.assert_called_once_with("Launch error", "bad launch")
    launcher._cancel_btn.configure.assert_called_once_with(text="Close")
    launcher._copy_btn.configure.assert_called_once_with(state="normal")
    assert launcher._output_queue is None


def test_append_output_writes_to_text_widget():
    launcher = DummyLauncher()

    portable_launcher.SdLauncherApp._append_output(launcher, "hello")

    launcher._output.configure.assert_any_call(state="normal")
    launcher._output.insert.assert_called_once_with("end", "hello")
    launcher._output.see.assert_called_once_with("end")


def test_copy_output_copies_text_to_clipboard():
    launcher = DummyLauncher()
    launcher._output.get.return_value = "captured output"

    portable_launcher.SdLauncherApp._copy_output(launcher)

    launcher.clipboard_clear.assert_called_once()
    launcher.clipboard_append.assert_called_once_with("captured output")
    launcher._append_output.assert_called_once_with("[INFO] Output copied to clipboard.\n")


def test_main_exits_when_tkinter_unavailable(monkeypatch, capsys):
    monkeypatch.setattr(portable_launcher, "_tkinter_available", False)

    with pytest.raises(SystemExit) as exc:
        portable_launcher.main()

    assert exc.value.code == 1
    assert "tkinter is not available" in capsys.readouterr().err


def test_main_runs_app_when_tkinter_available(monkeypatch):
    app = MagicMock()
    monkeypatch.setattr(portable_launcher, "_tkinter_available", True)
    monkeypatch.setattr(portable_launcher, "SdLauncherApp", lambda: app)

    portable_launcher.main()

    app.mainloop.assert_called_once()


def test_main_reports_unhandled_exception(monkeypatch):
    monkeypatch.setattr(portable_launcher, "_tkinter_available", True)
    monkeypatch.setattr(
        portable_launcher, "SdLauncherApp", MagicMock(side_effect=RuntimeError("boom"))
    )
    showerror = MagicMock()
    monkeypatch.setattr(portable_launcher.messagebox, "showerror", showerror)

    with pytest.raises(SystemExit) as exc:
        portable_launcher.main()

    assert exc.value.code == 1
    showerror.assert_called_once()
    assert "boom" in showerror.call_args[0][1]
