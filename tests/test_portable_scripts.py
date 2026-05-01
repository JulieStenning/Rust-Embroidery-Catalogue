"""
Tests for cross-platform portable scripts:
  scripts/portable_deploy.py
  scripts/portable_setup.py
  scripts/portable_stop.py
  scripts/portable_start.py
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

SCRIPTS_DIR = Path(__file__).resolve().parents[1] / "scripts"


def _import_deploy():
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "portable_deploy", SCRIPTS_DIR / "portable_deploy.py"
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ---------------------------------------------------------------------------
# portable_deploy — path helpers
# ---------------------------------------------------------------------------


class TestPortableDeployPathHelpers:
    """Tests for _normalise_root and _is_valid_root in portable_deploy.py."""

    @pytest.fixture(autouse=True)
    def load_module(self):
        self.deploy = _import_deploy()

    def test_normalise_root_windows_drive(self):
        # Windows drive root: trailing separator is stripped.
        assert self.deploy._normalise_root("F:") == "F:"
        assert self.deploy._normalise_root("F:\\").rstrip("\\") == "F:"

    def test_normalise_root_posix_path(self):
        assert self.deploy._normalise_root("/Volumes/MyDrive/") == "/Volumes/MyDrive"
        assert self.deploy._normalise_root("/Volumes/MyDrive") == "/Volumes/MyDrive"
        assert self.deploy._normalise_root("/") == "/"

    def test_is_valid_root_windows_drive(self):
        assert self.deploy._is_valid_root("F:\\")
        assert self.deploy._is_valid_root("F:")

    @pytest.mark.skipif(sys.platform == "win32", reason="POSIX-only")
    def test_is_valid_root_posix_absolute(self):
        assert self.deploy._is_valid_root("/Volumes/MyDrive")
        assert self.deploy._is_valid_root("/home/user/portable")
        assert not self.deploy._is_valid_root("relative/path")
        assert not self.deploy._is_valid_root("")

    def test_is_valid_root_unc(self):
        assert self.deploy._is_valid_root("\\\\server\\share")


# ---------------------------------------------------------------------------
# portable_deploy — deploy() function
# ---------------------------------------------------------------------------


class TestPortableDeployFunction:
    """Tests for the deploy() function in portable_deploy.py."""

    @pytest.fixture(autouse=True)
    def load_module(self):
        self.deploy = _import_deploy()

    def test_deploy_copies_src_and_templates(self, tmp_path):
        """deploy() should mirror src/, templates/, static/, alembic/ to the target."""
        source = tmp_path / "source"
        target = tmp_path / "target"

        # Create minimal source structure.
        for d in ("src", "templates", "static", "alembic"):
            (source / d).mkdir(parents=True)
        (source / "data").mkdir()
        (source / "data" / "tags.csv").write_text("tag1\n", encoding="utf-8")
        (source / ".env.example").write_text("# example\n", encoding="utf-8")
        (source / "requirements.txt").write_text("", encoding="utf-8")
        target.mkdir()

        self.deploy.deploy(
            str(target),
            str(source / "designs"),
            skip_designs=True,
            source_root=source,
        )

        dest = target / "EmbroideryApp" / "app"
        assert (dest / "src").is_dir()
        assert (dest / "templates").is_dir()
        assert (dest / "static").is_dir()
        assert (dest / "alembic").is_dir()
        assert (dest / ".env").exists()
        assert (dest / "data" / "tags.csv").exists()

    def test_deploy_creates_env_from_example(self, tmp_path):
        """deploy() should create .env from .env.example."""
        source = tmp_path / "source"
        target = tmp_path / "target"
        for d in ("src", "templates", "static", "alembic"):
            (source / d).mkdir(parents=True)
        (source / "data").mkdir()
        (source / "data" / "tags.csv").write_text("", encoding="utf-8")
        (source / ".env.example").write_text("# my env\n", encoding="utf-8")
        target.mkdir()

        self.deploy.deploy(
            str(target),
            str(source / "designs"),
            skip_designs=True,
            source_root=source,
        )

        env_content = (target / "EmbroideryApp" / "app" / ".env").read_text(encoding="utf-8")
        assert "# my env" in env_content

    def test_deploy_creates_minimal_env_when_example_missing(self, tmp_path):
        """deploy() should create a minimal .env when .env.example is absent."""
        source = tmp_path / "source"
        target = tmp_path / "target"
        for d in ("src", "templates", "static", "alembic"):
            (source / d).mkdir(parents=True)
        (source / "data").mkdir()
        (source / "data" / "tags.csv").write_text("", encoding="utf-8")
        target.mkdir()

        self.deploy.deploy(
            str(target),
            str(source / "designs"),
            skip_designs=True,
            source_root=source,
        )

        env_file = target / "EmbroideryApp" / "app" / ".env"
        assert env_file.exists()

    def test_deploy_exits_when_src_missing(self, tmp_path):
        """deploy() should exit with code 1 when the src/ folder is missing."""
        source = tmp_path / "source"
        target = tmp_path / "target"
        source.mkdir()
        target.mkdir()

        with pytest.raises(SystemExit) as exc:
            self.deploy.deploy(
                str(target),
                str(source / "designs"),
                skip_designs=True,
                source_root=source,
            )

        assert exc.value.code != 0

    def test_deploy_exits_when_designs_source_missing(self, tmp_path):
        """deploy() should exit when designs source is missing and skip_designs is False."""
        source = tmp_path / "source"
        target = tmp_path / "target"
        for d in ("src", "templates", "static", "alembic"):
            (source / d).mkdir(parents=True)
        (source / "data").mkdir()
        (source / "data" / "tags.csv").write_text("", encoding="utf-8")
        target.mkdir()

        with pytest.raises(SystemExit) as exc:
            self.deploy.deploy(
                str(target),
                str(tmp_path / "nonexistent_designs"),
                skip_designs=False,
                source_root=source,
            )

        assert exc.value.code != 0


# ---------------------------------------------------------------------------
# portable_deploy — _parse_args
# ---------------------------------------------------------------------------


class TestPortableDeployParseArgs:
    @pytest.fixture(autouse=True)
    def load_module(self):
        self.deploy = _import_deploy()

    def test_parse_args_defaults(self):
        root, designs, skip = self.deploy._parse_args([])
        assert root == self.deploy._DEFAULT_ROOT
        assert designs == self.deploy._DEFAULT_DESIGNS
        assert skip is False

    def test_parse_args_target_only(self):
        root, designs, skip = self.deploy._parse_args(["/Volumes/MyDrive"])
        assert root == "/Volumes/MyDrive"
        assert skip is False

    def test_parse_args_no_designs_flag(self):
        root, designs, skip = self.deploy._parse_args(["/Volumes/MyDrive", "--no-designs"])
        assert root == "/Volumes/MyDrive"
        assert skip is True

    def test_parse_args_target_and_designs(self):
        root, designs, skip = self.deploy._parse_args(["/Volumes/MyDrive", "/path/to/designs"])
        assert root == "/Volumes/MyDrive"
        assert designs == "/path/to/designs"
        assert skip is False


# ---------------------------------------------------------------------------
# portable_scripts — verify scripts exist and are importable
# ---------------------------------------------------------------------------


class TestPortableScriptsExist:
    """Verify that all four cross-platform scripts are present."""

    @pytest.mark.parametrize(
        "script_name",
        [
            "portable_setup.py",
            "portable_start.py",
            "portable_stop.py",
            "portable_deploy.py",
        ],
    )
    def test_script_file_exists(self, script_name):
        assert (SCRIPTS_DIR / script_name).exists(), f"Expected scripts/{script_name} to exist"

    @pytest.mark.parametrize(
        "script_name",
        [
            "portable_setup.py",
            "portable_start.py",
            "portable_stop.py",
            "portable_deploy.py",
        ],
    )
    def test_script_has_main_entrypoint(self, script_name):
        source = (SCRIPTS_DIR / script_name).read_text(encoding="utf-8")
        assert "def main()" in source, f"Expected scripts/{script_name} to define a main() function"
        assert (
            '__name__ == "__main__"' in source
        ), f"Expected scripts/{script_name} to have if __name__ == '__main__'"
