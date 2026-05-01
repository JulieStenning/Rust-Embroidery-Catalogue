import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock

import pytest

import auto_tag
import backfill_images
import backfill_stitching_tags

ROOT = Path(__file__).resolve().parents[1]


def test_start_bat_uses_database_subfolder_paths():
    script = (ROOT / "start.bat").read_text(encoding="utf-8")

    assert "data\\database\\catalogue_dev.db" in script
    assert "data\\database\\catalogue.db" in script


def test_start_bat_can_suppress_browser_auto_open():
    script = (ROOT / "start.bat").read_text(encoding="utf-8")

    assert "EMBROIDERY_DISABLE_EXTERNAL_OPEN" in script
    assert "PYTEST_CURRENT_TEST" in script


def test_start_bat_keeps_errors_visible_for_portable_launches():
    script = (ROOT / "start.bat").read_text(encoding="utf-8")

    assert "EMBROIDERY_DISABLE_ERROR_PAUSE" in script
    assert "Startup failed" in script
    assert "pause" in script.lower()
    assert "startup-error.log" in script
    assert ') > "%STARTUP_LOG%"' in script


def test_setup_bat_writes_startup_error_log():
    script = (ROOT / "setup.bat").read_text(encoding="utf-8")

    assert "startup-error.log" in script


def test_portable_requirements_do_not_require_pywebview():
    requirements = (ROOT / "requirements.txt").read_text(encoding="utf-8").lower()

    assert "pywebview" not in requirements


def test_prepare_portable_target_copies_starter_tags_csv():
    script = (ROOT / "prepare_portable_target.bat").read_text(encoding="utf-8")

    assert "tags.csv" in script


def test_prepare_portable_target_write_probe_uses_embroideryapp_folder():
    script = (ROOT / "prepare_portable_target.bat").read_text(encoding="utf-8")

    assert 'set "_WRITE_TEST=%DEST%\\.write_test.tmp"' in script
    assert "Continuing anyway; robocopy will report any real write failure below." in script


def test_build_desktop_bat_checks_required_runtime_modules():
    script = (ROOT / "build_desktop.bat").read_text(encoding="utf-8")

    assert "import webview" in script
    assert "import sqlalchemy" in script


def test_build_portable_deployment_bat_builds_windowed_gui_exe():
    script = (ROOT / "build_portable_deployment.bat").read_text(encoding="utf-8")
    spec = (ROOT / "EmbroideryPortableDeploy.spec").read_text(encoding="utf-8")

    assert "--windowed" in script or "console=False" in script or "console=False" in spec


def test_build_desktop_bat_explains_dist_output_is_not_installed():
    script = (ROOT / "build_desktop.bat").read_text(encoding="utf-8")

    assert "Add/Remove Programs" in script
    assert "portable build" in script.lower()


def test_build_desktop_bat_supports_optional_code_signing():
    script = (ROOT / "build_desktop.bat").read_text(encoding="utf-8")

    assert "signtool" in script.lower()
    assert "SIGN_CERT_FILE" in script
    assert "SIGN_CERT_PASSWORD" in script
    assert "SIGN_TIMESTAMP_URL" in script


def test_installer_script_uses_explicit_64bit_install_mode():
    script = (ROOT / "installer" / "EmbroideryCatalogue.iss").read_text(encoding="utf-8")

    assert "ArchitecturesAllowed=x64compatible" in script
    assert "ArchitecturesInstallIn64BitMode=x64compatible" in script
    assert "PrivilegesRequired=admin" in script


def test_desktop_spec_uses_app_icon_file():
    script = (ROOT / "EmbroideryCatalogue.spec").read_text(encoding="utf-8")

    assert "static' / 'icons' / 'app-icon.ico" in script


def test_base_template_includes_app_icon_favicon():
    template = (ROOT / "templates" / "base.html").read_text(encoding="utf-8")

    assert "/static/icons/app-icon.svg" in template
    assert "/static/icons/apple-touch-icon.png" in template
    assert "/favicon.ico" in template


def test_installer_script_prompts_for_catalogue_data_location():
    script = (ROOT / "installer" / "EmbroideryCatalogue.iss").read_text(encoding="utf-8")

    assert "CreateInputDirPage" in script
    assert "Catalogue data location" in script
    assert "EmbroideryCatalogueData" in script


def test_installer_script_writes_storage_pointer_for_app_startup():
    script = (ROOT / "installer" / "EmbroideryCatalogue.iss").read_text(encoding="utf-8")

    assert "storage.json" in script
    assert '"data_root"' in script or "data_root" in script
    assert "MachineEmbroideryDesigns" in script
    assert "database" in script


def test_installer_script_reads_selected_data_root_during_uninstall():
    script = (ROOT / "installer" / "EmbroideryCatalogue.iss").read_text(encoding="utf-8")

    assert "function ReadConfiguredDataRoot()" in script
    assert "LoadStringFromFile" in script
    assert '"data_root": "' in script


def test_installer_script_uses_home_folder_wording_and_ready_memo():
    script = (ROOT / "installer" / "EmbroideryCatalogue.iss").read_text(encoding="utf-8")

    assert "Select a home folder" in script
    assert "Program location:" in script
    assert "Home folder:" in script


class QueryStub:
    def __init__(self, all_result=None, scalar_result=None):
        self._all_result = [] if all_result is None else all_result
        self._scalar_result = scalar_result

    def order_by(self, *args, **kwargs):
        return self

    def filter(self, *args, **kwargs):
        return self

    def all(self):
        return self._all_result

    def scalar(self):
        return self._scalar_result


def _auto_tag_args(**overrides):
    args = {
        "redo": False,
        "skip_verified": False,
        "tier1_only": False,
        "tier3": False,
        "tier3_only": False,
        "dry_run": False,
        "batch_size": 20,
        "delay": 0.0,
        "vision_delay": 0.0,
        "limit": None,
    }
    args.update(overrides)
    return SimpleNamespace(**args)


def test_auto_tag_load_env_falls_back_to_environment(monkeypatch):
    monkeypatch.setenv("GOOGLE_API_KEY", "env-key")
    monkeypatch.setattr(auto_tag.Path, "exists", lambda self: False)

    assert auto_tag._load_env() == "env-key"


def test_auto_tag_load_env_reads_from_file(monkeypatch):
    monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
    monkeypatch.setattr(auto_tag.Path, "exists", lambda self: True)
    monkeypatch.setattr(
        auto_tag.Path, "read_text", lambda self: "OTHER_KEY=1\nGOOGLE_API_KEY=file-key\n"
    )

    assert auto_tag._load_env() == "file-key"


def test_auto_tag_main_nothing_to_do(monkeypatch, capsys):
    tag = SimpleNamespace(id=1, description="Flowers")
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub([tag]) if model is auto_tag.Tag else QueryStub([])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser, "parse_args", lambda self: _auto_tag_args()
    )

    auto_tag.main()

    assert "Nothing to do." in capsys.readouterr().out
    db.close.assert_called_once()


def test_auto_tag_main_applies_tier1_tags(monkeypatch):
    tag = SimpleNamespace(id=1, description="Flowers")
    design = SimpleNamespace(
        id=1, filename="rose.jef", tags=[], tags_checked=True, tagging_tier=None
    )
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub([tag]) if model is auto_tag.Tag else QueryStub([design])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser,
        "parse_args",
        lambda self: SimpleNamespace(
            redo=False,
            skip_verified=False,
            tier1_only=False,
            tier3=False,
            tier3_only=False,
            dry_run=False,
            batch_size=20,
            delay=0.0,
            vision_delay=0.0,
            limit=None,
        ),
    )
    monkeypatch.setattr(auto_tag, "suggest_tier1", lambda filename, valid: ["Flowers"])

    auto_tag.main()

    assert design.tagging_tier == 1
    assert design.tags_checked is False
    assert design.tags == [tag]
    db.commit.assert_called_once()
    db.close.assert_called_once()


def test_auto_tag_main_tier3_only_warns_without_key(monkeypatch, capsys):
    tag = SimpleNamespace(id=1, description="Flowers")
    design = SimpleNamespace(id=1, filename="rose.jef", image_data=b"png", tags=[])
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub([tag]) if model is auto_tag.Tag else QueryStub([design])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser, "parse_args", lambda self: _auto_tag_args(tier3_only=True)
    )
    monkeypatch.setattr(auto_tag, "_load_env", lambda: None)

    auto_tag.main()

    assert "No GOOGLE_API_KEY found" in capsys.readouterr().out
    db.close.assert_called_once()


def test_auto_tag_main_tier3_only_applies_tags(monkeypatch):
    tag = SimpleNamespace(id=1, description="Flowers")
    first = SimpleNamespace(
        id=1, filename="rose.jef", image_data=b"png", tags=[], tags_checked=True
    )
    second = SimpleNamespace(
        id=2, filename="skip.jef", image_data=b"png", tags=[], tags_checked=True
    )
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub([tag]) if model is auto_tag.Tag else QueryStub([first, second])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser,
        "parse_args",
        lambda self: _auto_tag_args(tier3_only=True, limit=1),
    )
    monkeypatch.setattr(auto_tag, "_load_env", lambda: "test-key")
    monkeypatch.setattr(
        auto_tag,
        "suggest_tier3_vision",
        lambda candidates, valid, api_key, delay_seconds=0.0: {1: ["Flowers"]},
    )

    auto_tag.main()

    assert first.tags_checked is False
    assert first.tags
    assert second.tags == []
    db.commit.assert_called_once()
    db.close.assert_called_once()


def test_auto_tag_main_full_tier_flow_assigns_tiers(monkeypatch):
    tags = [
        SimpleNamespace(id=1, description="Flowers"),
        SimpleNamespace(id=2, description="Animals"),
        SimpleNamespace(id=3, description="Birds"),
    ]
    tier1_design = SimpleNamespace(
        id=1, filename="rose.jef", tags=[], tags_checked=True, tagging_tier=None
    )
    tier2_design = SimpleNamespace(
        id=2, filename="mystery.jef", tags=[], tags_checked=True, tagging_tier=None
    )
    tier3_design = SimpleNamespace(
        id=3, filename="owl.jef", tags=[], image_data=b"png", tags_checked=True, tagging_tier=None
    )
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub(tags)
        if model is auto_tag.Tag
        else QueryStub([tier1_design, tier2_design, tier3_design])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser, "parse_args", lambda self: _auto_tag_args(tier3=True)
    )
    monkeypatch.setattr(
        auto_tag,
        "suggest_tier1",
        lambda filename, valid: ["Flowers"] if filename == "rose.jef" else [],
    )
    monkeypatch.setattr(auto_tag, "_load_env", lambda: "test-key")
    monkeypatch.setattr(
        auto_tag,
        "suggest_tier2_batch",
        lambda filenames, valid, api_key, batch_size=20, delay_seconds=0.0: {
            "mystery": ["Animals"]
        },
    )
    monkeypatch.setattr(
        auto_tag,
        "suggest_tier3_vision",
        lambda designs, valid, api_key, delay_seconds=0.0: {3: ["Birds"]},
    )

    auto_tag.main()

    assert tier1_design.tagging_tier == 1
    assert tier2_design.tagging_tier == 2
    assert tier3_design.tagging_tier == 3
    assert tier1_design.tags_checked is False
    assert tier2_design.tags_checked is False
    assert tier3_design.tags_checked is False
    db.commit.assert_called_once()
    db.close.assert_called_once()


def test_auto_tag_main_redo_tier1_only_dry_run(monkeypatch, capsys):
    tag = SimpleNamespace(id=1, description="Flowers")
    design = SimpleNamespace(
        id=1, filename="untagged.jef", tags=[], tags_checked=True, tagging_tier=None
    )
    db = MagicMock()
    db.query.side_effect = lambda model: (
        QueryStub([tag]) if model is auto_tag.Tag else QueryStub([design])
    )

    monkeypatch.setattr(auto_tag, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        auto_tag.argparse.ArgumentParser,
        "parse_args",
        lambda self: _auto_tag_args(redo=True, tier1_only=True, tier3=True, dry_run=True),
    )
    monkeypatch.setattr(auto_tag, "suggest_tier1", lambda filename, valid: [])

    auto_tag.main()

    out = capsys.readouterr().out
    assert "--redo: processing all 1 designs." in out
    assert "Skipping Tier 2 (--tier1-only)." in out
    assert "Skipping Tier 3 (--tier1-only)." in out
    assert "DRY RUN complete." in out
    db.commit.assert_not_called()
    db.close.assert_called_once()


def test_backfill_images_parse_args(monkeypatch):
    monkeypatch.setattr(
        sys,
        "argv",
        ["backfill_images.py", "--base", "D:/Designs", "--batch", "5", "--dry-run", "--redo"],
    )

    args = backfill_images.parse_args()

    assert args.base == "D:/Designs"
    assert args.batch == 5
    assert args.dry_run is True
    assert args.redo is True


def test_backfill_images_main_updates_design(monkeypatch):
    design = SimpleNamespace(
        id=1,
        filename="rose.jef",
        filepath="\\rose.jef",
        image_data=None,
        width_mm=None,
        height_mm=None,
        hoop_id=None,
    )
    query = QueryStub([design])
    db = MagicMock()
    db.query.return_value = query

    pattern = SimpleNamespace(bounds=lambda: (0, 0, 120, 80))
    hoop = SimpleNamespace(id=7)

    monkeypatch.setattr(
        backfill_images,
        "parse_args",
        lambda: SimpleNamespace(
            base="D:/Designs",
            batch=1,
            dry_run=False,
            redo=False,
            workers=1,
            preview_2d=False,
        ),
    )
    monkeypatch.setattr(backfill_images, "SessionLocal", lambda: db)
    monkeypatch.setattr(backfill_images, "get_designs_base_path", lambda _db: "")
    monkeypatch.setattr(backfill_images.os.path, "isfile", lambda path: True)
    monkeypatch.setattr(backfill_images.pyembroidery, "read", lambda path: pattern)
    monkeypatch.setattr(
        backfill_images, "_render_preview", lambda pattern, **kwargs: b"png-preview"
    )
    monkeypatch.setattr(backfill_images, "select_hoop_for_dimensions", lambda *_args: hoop)

    backfill_images.main()

    assert design.width_mm == 12.0
    assert design.height_mm == 8.0
    assert design.hoop_id == 7
    assert design.image_data == b"png-preview"
    assert db.commit.call_count >= 1
    db.close.assert_called_once()


def test_backfill_images_main_exits_on_query_error(monkeypatch):
    db = MagicMock()
    db.query.side_effect = RuntimeError("db unavailable")

    monkeypatch.setattr(
        backfill_images,
        "parse_args",
        lambda: SimpleNamespace(
            base="D:/Designs",
            batch=1,
            dry_run=False,
            redo=False,
            workers=1,
            preview_2d=False,
        ),
    )
    monkeypatch.setattr(backfill_images, "SessionLocal", lambda: db)
    monkeypatch.setattr(backfill_images, "get_designs_base_path", lambda _db: "")

    with pytest.raises(SystemExit) as exc:
        backfill_images.main()

    assert exc.value.code == 1
    db.close.assert_called_once()


def test_backfill_stitching_tags_parse_args(monkeypatch):
    monkeypatch.setattr(
        sys,
        "argv",
        ["backfill_stitching_tags.py", "--batch", "5", "--dry-run", "--limit", "10"],
    )

    args = backfill_stitching_tags.parse_args()

    assert args.batch == 5
    assert args.dry_run is True
    assert args.limit == 10


def test_backfill_stitching_tags_main_delegates_to_run_stitching_backfill_action(monkeypatch):
    from src.services.auto_tagging import TaggingActionResult

    db = MagicMock()
    monkeypatch.setattr(backfill_stitching_tags, "SessionLocal", lambda: db)
    monkeypatch.setattr(
        backfill_stitching_tags,
        "parse_args",
        lambda: SimpleNamespace(
            batch=1,
            dry_run=False,
            limit=None,
            clear_existing_stitching=False,
            workers=1,
        ),
    )

    result = TaggingActionResult(action="backfill_stitching", tiers_run=[1])
    result.designs_considered = 2
    result.tier1_tagged = 1
    result.total_tagged = 1
    result.tag_breakdown = {"Filled": 1}

    monkeypatch.setattr(
        backfill_stitching_tags,
        "run_stitching_backfill_action",
        lambda **kwargs: result,
    )

    backfill_stitching_tags.main()

    assert db.close.call_count >= 1
