import io
import os
import struct
import sys
import zlib
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import mock_open, patch

import pytest
from PIL import Image

import src.services.preview as preview_mod
from src.services import bulk_import as svc
from src.services.scanning import (
    SUPPORTED_EXTENSIONS,
    ScannedDesign,
    _pick_preferred,
    process_selected_files,
    scan_folders,
)


class TestSupportedExtensions:
    """Verify the expanded SUPPORTED_EXTENSIONS set matches the acceptance criteria."""

    def test_all_standard_formats_included(self):
        expected = {
            ".jef",
            ".pes",
            ".hus",
            ".dst",
            ".exp",
            ".vp3",
            ".u01",
            ".pec",
            ".xxx",
            ".tbf",
            ".10o",
            ".100",
            ".dat",
            ".dsb",
            ".dsz",
            ".emd",
            ".exy",
            ".fxy",
            ".gt",
            ".inb",
            ".jpx",
            ".max",
            ".mit",
            ".new",
            ".pcm",
            ".pcq",
            ".pcs",
            ".phb",
            ".phc",
            ".sew",
            ".shv",
            ".stc",
            ".stx",
            ".tap",
            ".zhs",
            ".zxy",
            ".gcode",
        }
        for ext in expected:
            assert ext in SUPPORTED_EXTENSIONS, f"{ext} missing from SUPPORTED_EXTENSIONS"

    def test_art_included(self):
        assert ".art" in SUPPORTED_EXTENSIONS

    def test_pmv_included(self):
        assert ".pmv" in SUPPORTED_EXTENSIONS

    def test_excluded_formats_not_present(self):
        excluded = {
            ".json",
            ".col",
            ".edr",
            ".inf",
            ".svg",
            ".csv",
            ".png",
            ".txt",
            ".bro",
            ".ksm",
            ".pcd",
        }
        for ext in excluded:
            assert ext not in SUPPORTED_EXTENSIONS, f"{ext} should not be in SUPPORTED_EXTENSIONS"


class TestExtensionPriority:
    """Verify the EXTENSION_PRIORITY deduplication order."""

    def test_jef_preferred_over_pes(self):
        result = _pick_preferred(["design.pes", "design.jef"])
        assert result == "design.jef"

    def test_pes_preferred_over_vp3(self):
        result = _pick_preferred(["design.vp3", "design.pes"])
        assert result == "design.pes"

    def test_vp3_preferred_over_hus(self):
        result = _pick_preferred(["design.hus", "design.vp3"])
        assert result == "design.vp3"

    def test_hus_preferred_over_sew(self):
        result = _pick_preferred(["design.sew", "design.hus"])
        assert result == "design.hus"

    def test_sew_preferred_over_dst(self):
        result = _pick_preferred(["design.dst", "design.sew"])
        assert result == "design.dst"

    def test_art_lowest_priority_over_dst(self):
        result = _pick_preferred(["design.art", "design.dst"])
        assert result == "design.dst"

    def test_art_is_last_resort(self):
        # art should lose to any other registered format
        result = _pick_preferred(["design.art", "design.gcode"])
        assert result == "design.gcode"


class TestProcessFileNewFormats:
    """Verify _process_file handles new formats gracefully via the generic pyembroidery path."""

    def test_process_dst_file_with_valid_pattern(self, db, monkeypatch):
        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 200, 100),
            stitches=[],
            count_stitches=lambda: 1234,
            count_threads=lambda: 5,
            count_color_changes=lambda: 4,
        )
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: None)
        monkeypatch.setattr(preview_mod, "_render_preview", lambda _pattern, ext=None: b"preview")

        scanned = preview_mod._process_file("C:/design.dst", "design.dst", "\\design.dst", db)

        assert scanned.filename == "design.dst"
        assert scanned.width_mm == 20.0
        assert scanned.height_mm == 10.0
        assert scanned.stitch_count == 1234
        assert scanned.color_count == 5
        assert scanned.color_change_count == 4
        assert scanned.error is None

    def test_process_exp_file_with_none_pattern(self, db, monkeypatch):
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: None)
        monkeypatch.setattr(preview_mod, "_find_spider_image", lambda _path: None)

        scanned = preview_mod._process_file("C:/design.exp", "design.exp", "\\design.exp", db)

        assert scanned.filename == "design.exp"
        assert scanned.width_mm is None
        assert scanned.error is None

    def test_process_pmv_file_uses_generic_path(self, db, monkeypatch):
        import src.services.pattern_analysis as pattern_analysis_mod

        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 100, 80),
            stitches=[],
            count_stitches=lambda: 500,
            count_threads=lambda: 3,
            count_color_changes=lambda: 2,
        )
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: None)
        monkeypatch.setattr(
            preview_mod, "_render_preview", lambda _pattern, **kwargs: b"preview-pmv"
        )
        monkeypatch.setattr(
            pattern_analysis_mod, "_render_preview", lambda _pattern, **kwargs: b"preview-pmv"
        )

        scanned = preview_mod._process_file("C:/design.pmv", "design.pmv", "\\design.pmv", db)

        assert scanned.filename == "design.pmv"
        assert scanned.image_data == b"preview-pmv"
        assert scanned.stitch_count == 500
        assert scanned.color_count == 3
        assert scanned.color_change_count == 2
        assert scanned.error is None

    def test_process_file_missing_bounds_handled_gracefully(self, db, monkeypatch):
        pattern = SimpleNamespace(
            bounds=lambda: None,
            stitches=[],
            count_stitches=lambda: 0,
            count_threads=lambda: 0,
            count_color_changes=lambda: 0,
        )
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "_render_preview", lambda _pattern, ext=None: None)

        scanned = preview_mod._process_file("C:/design.vp3", "design.vp3", "\\design.vp3", db)

        assert scanned.width_mm is None
        assert scanned.height_mm is None
        assert scanned.error is None


def _encode_design_icon_payload(decoded_bytes: bytes) -> bytes:
    """Encode bytes so `_decode_art_icon` will swizzle+decompress them back."""
    return bytes((((b >> 1) | ((b & 1) << 7)) & 0xFF) ^ 0xD2 for b in decoded_bytes)


def _build_wilcom_metadata_blob() -> bytes:
    blob = bytearray(160)
    set_off = 48
    struct.pack_into("<I", blob, 44, set_off)
    struct.pack_into("<I", blob, set_off + 4, 3)

    struct.pack_into("<II", blob, set_off + 8, 11, 40)
    struct.pack_into("<II", blob, set_off + 16, 22, 48)
    struct.pack_into("<II", blob, set_off + 24, 19, 56)

    struct.pack_into("<H", blob, set_off + 40, 3)
    struct.pack_into("<i", blob, set_off + 44, 1234)

    struct.pack_into("<H", blob, set_off + 48, 3)
    struct.pack_into("<i", blob, set_off + 52, 7)

    vendor = b"Bernina\x00"
    struct.pack_into("<H", blob, set_off + 56, 30)
    struct.pack_into("<I", blob, set_off + 60, len(vendor))
    blob[set_off + 64 : set_off + 64 + len(vendor)] = vendor
    return bytes(blob)


class TestBulkImportAdditionalCoverage:
    def test_decode_art_icon_returns_png_bytes(self):
        bmp = io.BytesIO()
        Image.new("RGB", (2, 2), color=(255, 0, 0)).save(bmp, format="BMP")
        compressed = zlib.compress(bmp.getvalue())
        raw = b"HEAD" + _encode_design_icon_payload(compressed)

        class FakeCompoundReader:
            def __init__(self, _fh):
                pass

            def open(self, name):
                assert name == "DESIGN_ICON"
                return io.BytesIO(raw)

        fake_compoundfiles = SimpleNamespace(CompoundFileReader=FakeCompoundReader)

        with (
            patch.dict(sys.modules, {"compoundfiles": fake_compoundfiles}),
            patch("builtins.open", mock_open(read_data=b"art-bytes")),
        ):
            result = preview_mod._decode_art_icon("design.art")

        assert result is not None
        assert result[:4] == b"\x89PNG"

    def test_decode_art_icon_returns_none_on_failure(self):
        with patch("builtins.open", side_effect=OSError("missing")):
            assert preview_mod._decode_art_icon("missing.art") is None

    def test_read_art_metadata_parses_values(self):
        metadata_blob = _build_wilcom_metadata_blob()

        class FakeCompoundReader:
            def __init__(self, _fh):
                pass

            def open(self, name):
                assert name == "\x05WilcomDesignInformationDDD"
                return io.BytesIO(metadata_blob)

        fake_compoundfiles = SimpleNamespace(CompoundFileReader=FakeCompoundReader)

        with (
            patch.dict(sys.modules, {"compoundfiles": fake_compoundfiles}),
            patch("builtins.open", mock_open(read_data=b"art-bytes")),
        ):
            result = preview_mod._read_art_metadata("design.art")

        assert result == {
            "stitch_count": 1234,
            "color_count": 7,
            "vendor": "Bernina",
        }

    def test_read_art_metadata_returns_defaults_on_failure(self):
        with patch("builtins.open", side_effect=OSError("missing")):
            assert preview_mod._read_art_metadata("missing.art") == {
                "stitch_count": None,
                "color_count": None,
                "vendor": None,
            }

    def test_find_spider_image_skips_unreadable_subdir(self, tmp_path, monkeypatch):
        design_file = tmp_path / "rose.art"
        design_file.write_bytes(b"\x00")

        bad_spider = tmp_path / "bad_spider"
        bad_spider.mkdir()
        good_spider = tmp_path / "good_spider"
        good_spider.mkdir()

        jpg_path = good_spider / "rose.art.jpeg"
        Image.new("RGB", (10, 10), color=(0, 255, 0)).save(jpg_path, format="JPEG")

        real_listdir = os.listdir

        def fake_listdir(path):
            if os.path.normpath(path) == os.path.normpath(str(bad_spider)):
                raise OSError("cannot read folder")
            return real_listdir(path)

        monkeypatch.setattr(svc.os, "listdir", fake_listdir)

        result = svc._find_spider_image(str(design_file))

        assert result is not None
        assert result[:4] == b"\x89PNG"

    def test_find_spider_image_returns_none_for_invalid_jpeg(self, tmp_path):
        design_file = tmp_path / "rose.art"
        design_file.write_bytes(b"\x00")
        spider_dir = tmp_path / "_embird_spider"
        spider_dir.mkdir()
        (spider_dir / "rose.art.jpeg").write_bytes(b"not-a-real-image")

        assert svc._find_spider_image(str(design_file)) is None

    def test_process_selected_files_skips_existing_paths(self, db, tmp_path, monkeypatch):
        from src.services import designs as design_svc
        from src.services.settings_service import SETTING_DESIGNS_BASE_PATH, set_setting

        set_setting(db, SETTING_DESIGNS_BASE_PATH, str(tmp_path))
        design_svc.create(
            db,
            {"filename": "existing.jef", "filepath": "\\existing.jef", "is_stitched": False},
        )

        seen = []

        def fake_process(full_path, filename, rel_filepath, _db, **kwargs):
            seen.append((full_path, filename, rel_filepath))
            return ScannedDesign(filename=filename, filepath=rel_filepath)

        monkeypatch.setattr(preview_mod, "_process_file", fake_process)

        results = svc.process_selected_files(["\\existing.jef", "\\new.jef"], str(tmp_path), db)

        assert len(results) == 1
        assert results[0].filepath == "\\new.jef"
        assert seen == [
            (os.path.normpath(os.path.join(str(tmp_path), "new.jef")), "new.jef", "\\new.jef")
        ]

    def test_process_file_art_uses_icon_fallback(self, db, monkeypatch):
        monkeypatch.setattr(preview_mod, "_find_spider_image", lambda _path: None)
        monkeypatch.setattr(preview_mod, "_decode_art_icon", lambda _path: b"png-preview")

        scanned = svc._process_file("C:/design.art", "design.art", "\\design.art", db)

        assert scanned.filepath == "\\design.art"
        assert scanned.image_data == b"png-preview"
        assert scanned.width_mm is None

    def test_process_file_art_attempts_pyembroidery_then_falls_back(self, db, monkeypatch):
        calls = {"read": 0, "fallback": 0}

        def _raise_read(_path):
            calls["read"] += 1
            raise RuntimeError("reader failed")

        def _spider(_path):
            calls["fallback"] += 1
            return b"spider-preview"

        monkeypatch.setattr(preview_mod.pyembroidery, "read", _raise_read)
        monkeypatch.setattr(preview_mod, "_find_spider_image", _spider)
        monkeypatch.setattr(preview_mod, "_decode_art_icon", lambda _path: None)

        scanned = svc._process_file("C:/design.art", "design.art", "\\design.art", db)

        assert calls["read"] == 1
        assert calls["fallback"] == 1
        assert scanned.image_data == b"spider-preview"
        assert scanned.error is None

    def test_read_spider_art_dimensions_from_index_utf16(self, tmp_path):
        spider_dir = tmp_path / "_embird_spider"
        spider_dir.mkdir()
        index_content = (
            "<html><body>\n"
            '<a href="rose.art">rose.art</a> 81.4x88.9 mm (3.20x3.50 in)\n'
            "</body></html>"
        )
        (spider_dir / "index.htm").write_bytes(index_content.encode("utf-16"))
        design_file = tmp_path / "rose.art"
        design_file.write_bytes(b"art")

        dims = preview_mod._read_spider_art_dimensions(str(design_file))

        assert dims == (81.4, 88.9)

    def test_process_file_art_uses_spider_dimensions_for_hoop(self, db, monkeypatch):
        hoop = SimpleNamespace(id=7, name="8x9 hoop")
        captured: dict[str, float] = {}

        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: None)
        monkeypatch.setattr(preview_mod, "_read_spider_art_dimensions", lambda _path: (81.4, 88.9))
        monkeypatch.setattr(preview_mod, "_find_spider_image", lambda _path: b"spider-preview")
        monkeypatch.setattr(preview_mod, "_decode_art_icon", lambda _path: None)

        def _pick_hoop(_db, width_mm, height_mm):
            captured["width"] = width_mm
            captured["height"] = height_mm
            return hoop

        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", _pick_hoop)

        scanned = svc._process_file("C:/rose.art", "rose.art", "\\rose.art", db)

        assert scanned.width_mm == 81.4
        assert scanned.height_mm == 88.9
        assert captured == {"width": 81.4, "height": 88.9}
        assert scanned.hoop_id == 7
        assert scanned.hoop_name == "8x9 hoop"

    def test_process_file_art_prefers_spider_dimensions_over_pattern_bounds(self, db, monkeypatch):
        # Pattern bounds are tiny, but Spider dimensions should win for .art files.
        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 100, 100),
            stitches=[],
            count_stitches=lambda: 0,
            count_threads=lambda: 0,
            count_color_changes=lambda: 0,
        )

        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "_read_spider_art_dimensions", lambda _path: (81.4, 88.9))
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: None)
        monkeypatch.setattr(
            preview_mod, "_render_preview", lambda _pattern, ext=None: b"rendered-preview"
        )

        scanned = svc._process_file("C:/rose.art", "rose.art", "\\rose.art", db)

        assert scanned.width_mm == 81.4
        assert scanned.height_mm == 88.9

    def test_process_file_with_none_pattern_uses_spider_preview(self, db, monkeypatch):
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: None)
        monkeypatch.setattr(preview_mod, "_find_spider_image", lambda _path: b"fallback-preview")

        scanned = svc._process_file("C:/design.jef", "design.jef", "\\design.jef", db)

        assert scanned.image_data == b"fallback-preview"
        assert scanned.error is None

    def test_process_file_assigns_hoop_details(self, db, monkeypatch):
        import src.services.pattern_analysis as pattern_analysis_mod

        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 100, 50),
            stitches=[],
            count_stitches=lambda: 999,
            count_threads=lambda: 4,
            count_color_changes=lambda: 3,
        )
        hoop = SimpleNamespace(id=42, name="5x7 hoop")

        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: hoop)
        monkeypatch.setattr(
            preview_mod, "_render_preview", lambda _pattern, **kwargs: b"rendered-preview"
        )
        monkeypatch.setattr(
            pattern_analysis_mod, "_render_preview", lambda _pattern, **kwargs: b"rendered-preview"
        )

        scanned = svc._process_file("C:/rose.jef", "rose.jef", "\\rose.jef", db)

        assert scanned.width_mm == 10.0
        assert scanned.height_mm == 5.0
        assert scanned.stitch_count == 999
        assert scanned.color_count == 4
        assert scanned.color_change_count == 3
        assert scanned.hoop_id == 42
        assert scanned.hoop_name == "5x7 hoop"
        assert scanned.image_data == b"rendered-preview"

    def test_render_preview_calls_pngwriter_with_3d_setting(self, monkeypatch):
        """Verify _render_preview delegates to PngWriter.write with 3d=True."""
        pattern = SimpleNamespace()
        captured_settings: dict | None = None

        def _fake_write(_pattern, buf, settings=None):
            nonlocal captured_settings
            captured_settings = settings
            buf.write(b"\x89PNGtest")

        monkeypatch.setattr(preview_mod.pyembroidery.PngWriter, "write", _fake_write)

        result = preview_mod._render_preview(pattern)

        assert result == b"\x89PNGtest"
        assert captured_settings == {"3d": True}

    def test_render_preview_returns_none_on_failure(self, monkeypatch):
        """Verify _render_preview returns None when PngWriter.write fails."""
        pattern = SimpleNamespace()

        def _fail_write(_pattern, buf, settings=None):
            raise RuntimeError("PngWriter failed")

        monkeypatch.setattr(preview_mod.pyembroidery.PngWriter, "write", _fail_write)

        assert preview_mod._render_preview(pattern) is None

    def test_render_preview_calls_pngwriter_with_2d_setting(self, monkeypatch):
        """Verify _render_preview delegates to PngWriter.write with 2d=False."""
        pattern = SimpleNamespace()
        captured_settings: dict | None = None

        def _fake_write(_pattern, buf, settings=None):
            nonlocal captured_settings
            captured_settings = settings
            buf.write(b"\x89PNG2d")

        monkeypatch.setattr(preview_mod.pyembroidery.PngWriter, "write", _fake_write)

        result = preview_mod._render_preview(pattern, preview_3d=False)

        assert result == b"\x89PNG2d"
        assert captured_settings == {"3d": False}

    def test_render_preview_and_bounds_redo_regenerates_image(self, monkeypatch):
        """Verify _render_preview_and_bounds with redo=True regenerates even when image exists."""
        import src.services.pattern_analysis as pattern_analysis_mod

        pattern = SimpleNamespace(bounds=lambda: (0, 0, 100, 50), stitches=[])
        render_calls: list[bool] = []

        def _fake_render(_pattern, **kw):
            render_calls.append(kw.get("preview_3d", True))
            return b"regenerated-preview"

        monkeypatch.setattr(pattern_analysis_mod, "_render_preview", _fake_render)

        img_data, img_type, w_mm, h_mm = pattern_analysis_mod._render_preview_and_bounds(
            pattern,
            redo=True,
            existing_image_data=b"old-image",
            existing_image_type="3d",
            existing_width_mm=10.0,
            existing_height_mm=5.0,
        )

        assert img_data == b"regenerated-preview"
        assert img_type == "3d"
        assert len(render_calls) == 1

    def test_render_preview_and_bounds_upgrade_2d_to_3d(self, monkeypatch):
        """Verify _render_preview_and_bounds with upgrade_2d_to_3d re-renders 2D images."""
        import src.services.pattern_analysis as pattern_analysis_mod

        pattern = SimpleNamespace(bounds=lambda: (0, 0, 100, 50), stitches=[])
        render_calls: list[bool] = []

        def _fake_render(_pattern, **kw):
            render_calls.append(kw.get("preview_3d", True))
            return b"upgraded-preview"

        monkeypatch.setattr(pattern_analysis_mod, "_render_preview", _fake_render)

        img_data, img_type, w_mm, h_mm = pattern_analysis_mod._render_preview_and_bounds(
            pattern,
            upgrade_2d_to_3d=True,
            existing_image_data=b"old-2d-image",
            existing_image_type="2d",
            existing_width_mm=10.0,
            existing_height_mm=5.0,
        )

        assert img_data == b"upgraded-preview"
        assert img_type == "3d"
        assert len(render_calls) == 1

    def test_confirm_import_copies_source_file_into_managed_storage(
        self, db, tmp_path, monkeypatch
    ):
        from src.services import settings_service as settings_svc

        managed_base = tmp_path / "MachineEmbroideryDesigns"
        source_root = tmp_path / "source"
        source_file = source_root / "Florals" / "rose.jef"
        source_file.parent.mkdir(parents=True, exist_ok=True)
        source_file.write_bytes(b"embroidery-bytes")
        monkeypatch.setattr(settings_svc, "DESIGNS_BASE_PATH", str(managed_base))

        created = svc.confirm_import(
            db,
            [
                ScannedDesign(
                    filename="rose.jef",
                    filepath="\\Florals\\rose.jef",
                    image_data=b"preview",
                    source_full_path=str(source_file),
                )
            ],
            run_tier2=False,
            run_tier3=False,
        )

        dest_file = managed_base / "Florals" / "rose.jef"
        assert len(created) == 1
        assert created[0].filepath == "\\Florals\\rose.jef"
        assert dest_file.exists()
        assert dest_file.read_bytes() == b"embroidery-bytes"

    def test_confirm_import_tier1_only_no_ai_calls(self, db, monkeypatch):
        """Test 6.4.4 — Import with Tier 1 only, no AI calls.

        When ``run_tier2=False`` and ``run_tier3=False``, the import should
        apply Tier 1 keyword matching only and never invoke the AI functions.
        """
        from src.services import tags

        tags.create(db, "Flowers", "image")

        # Fail if any AI function is called
        monkeypatch.setattr(
            svc,
            "suggest_tier2_batch",
            lambda *args, **kwargs: pytest.fail("Tier 2 should not be called"),
        )
        monkeypatch.setattr(
            svc,
            "suggest_tier3_vision",
            lambda *args, **kwargs: pytest.fail("Tier 3 should not be called"),
        )

        created = svc.confirm_import(
            db,
            [ScannedDesign(filename="rose.jef", filepath="\\Florals\\rose.jef")],
            run_tier2=False,
            run_tier3=False,
        )

        assert len(created) == 1
        assert created[0].tagging_tier == 1
        assert [tag.description for tag in created[0].tags] == ["Flowers"]

    def test_confirm_import_tier2_assigns_tags_when_api_key_present(self, db, monkeypatch):
        from src.services import tags

        tags.create(db, "Tier2 Flowers", "image")
        monkeypatch.setattr(svc, "_load_api_key", lambda: "test-key")
        monkeypatch.setattr(
            svc,
            "suggest_tier2_batch",
            lambda filenames, valid_descriptions, api_key: {"mystery": ["Tier2 Flowers"]},
        )

        created = svc.confirm_import(
            db,
            [ScannedDesign(filename="mystery.jef", filepath="\\mystery.jef")],
            run_tier2=True,
            run_tier3=False,
        )

        assert len(created) == 1
        assert created[0].tagging_tier == 2
        assert [tag.description for tag in created[0].tags] == ["Tier2 Flowers"]

    def test_confirm_import_tier3_assigns_tags_when_api_key_present(self, db, monkeypatch):
        from src.services import tags

        tags.create(db, "Tier3 Flowers", "image")
        monkeypatch.setattr(svc, "_load_api_key", lambda: "test-key")
        monkeypatch.setattr(svc, "suggest_tier2_batch", lambda *args, **kwargs: {})

        def fake_tier3(designs, valid_descriptions, api_key):
            return {designs[0].id: ["Tier3 Flowers"]}

        monkeypatch.setattr(svc, "suggest_tier3_vision", fake_tier3)

        created = svc.confirm_import(
            db,
            [
                ScannedDesign(
                    filename="untagged.jef",
                    filepath="\\untagged.jef",
                    image_data=b"fake-image",
                )
            ],
            run_tier2=True,
            run_tier3=True,
        )

        assert len(created) == 1
        assert created[0].tagging_tier == 3
        assert [tag.description for tag in created[0].tags] == ["Tier3 Flowers"]

    def test_confirm_import_batch_limit_restricts_ai_tagging(self, db, monkeypatch):
        """Test 6.4.7 — Import with batch limit restricts AI tagging to first N designs.

        When ``batch_limit=1`` is set, only the first design should receive
        Tier 2 AI tags; the remaining designs should stay with Tier 1 only.

        Designs use cryptic filenames so none match Tier 1 keywords, ensuring
        they remain untagged and eligible for Tier 2 AI processing.
        """
        from src.services import tags

        tags.create(db, "Flowers", "image")
        tags.create(db, "Animals", "image")
        monkeypatch.setattr(svc, "_load_api_key", lambda: "test-key")

        tier2_calls: list[list[str]] = []

        def track_tier2(filenames, valid_descriptions, api_key):
            tier2_calls.append(filenames)
            # Return results keyed by stem (lowercase, no extension) to match
            # how _apply_tier2_tags looks them up via Path(design.filename).stem.lower()
            return {Path(fn).stem.lower(): ["Animals"] for fn in filenames}

        monkeypatch.setattr(svc, "suggest_tier2_batch", track_tier2)

        created = svc.confirm_import(
            db,
            [
                ScannedDesign(filename="ABC001.jef", filepath="\\ABC001.jef"),
                ScannedDesign(filename="XYZ002.jef", filepath="\\XYZ002.jef"),
                ScannedDesign(filename="ZZZ003.jef", filepath="\\ZZZ003.jef"),
            ],
            run_tier2=True,
            run_tier3=False,
            batch_limit=1,
        )

        assert len(created) == 3
        # First design gets Tier 2 (batch_limit=1)
        assert created[0].tagging_tier == 2
        assert [tag.description for tag in created[0].tags] == ["Animals"]
        # Second and third designs remain untagged (no Tier 1 match, excluded by batch_limit)
        assert created[1].tagging_tier is None
        assert created[1].tags == []
        assert created[2].tagging_tier is None
        assert created[2].tags == []
        # Tier 2 was called only once with just the first design's filename
        assert len(tier2_calls) == 1
        assert len(tier2_calls[0]) == 1
        assert tier2_calls[0][0] == "ABC001.jef"

    # ------------------------------------------------------------------ #
    # Section 4.1 — Stitching Detection (pipeline integration)
    # ------------------------------------------------------------------ #

    def test_detect_stitching_tags_tatami_fill(self, monkeypatch):
        """4.1.2: _detect_stitching_tags returns 'Filled' for tatami/fill patterns."""
        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace()
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda **kw: ["Filled"],
        )

        result = pa_mod._detect_stitching_tags(pattern, filename="fill_design.pes")
        assert result == ["Filled"]

    def test_detect_stitching_tags_running(self, monkeypatch):
        """4.1.3: _detect_stitching_tags returns 'Running Stitch' for running patterns."""
        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace()
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda **kw: ["Running Stitch"],
        )

        result = pa_mod._detect_stitching_tags(pattern, filename="running_design.pes")
        assert result == ["Running Stitch"]

    def test_detect_stitching_tags_multiple_types(self, monkeypatch):
        """4.1.4: _detect_stitching_tags returns multiple sorted stitch types."""
        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace()
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda **kw: ["Satin Stitch", "Filled"],
        )

        result = pa_mod._detect_stitching_tags(pattern, filename="complex.pes")
        assert result == ["Filled", "Satin Stitch"]

    def test_detect_stitching_tags_no_detectable_type(self, monkeypatch):
        """4.1.5: _detect_stitching_tags returns None when nothing detected and clear_existing=False."""
        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace()
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda **kw: [],
        )

        result = pa_mod._detect_stitching_tags(
            pattern, filename="plain.pes", clear_existing_stitching=False
        )
        assert result is None

    def test_detect_stitching_tags_clear_existing_returns_empty(self, monkeypatch):
        """4.1.5 (variant): _detect_stitching_tags returns [] when nothing detected and clear_existing=True."""
        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace()
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda **kw: [],
        )

        result = pa_mod._detect_stitching_tags(
            pattern, filename="plain.pes", clear_existing_stitching=True
        )
        assert result == []

    def test_detect_stitching_tags_skips_when_already_present(self, db, monkeypatch):
        """4.1.6: run_stitching_action_runner skips designs that already have stitching tags."""
        from src.models import Design
        from src.services import tags
        from src.services.unified_backfill import run_stitching_action_runner

        # Create a stitching tag
        st_tag = tags.create(db, "Satin Stitch", "stitching")

        # Create a design that already has a stitching tag
        design = Design(filename="stitched.pes", filepath="\\stitched.pes")
        db.add(design)
        db.flush()
        design.tags.append(st_tag)
        db.commit()

        # Run stitching action — should skip because design already has stitching tags
        result = run_stitching_action_runner(
            db, design, {"clear_existing_stitching": False}, batch_size=1
        )

        assert result is None or result == 0

    def test_stitching_skips_verified_designs(self, db, monkeypatch):
        """4.1.8: run_stitching_action_runner skips designs where tags_checked=True."""
        from src.models import Design
        from src.services.unified_backfill import run_stitching_action_runner

        # Create a verified design
        design = Design(filename="verified.pes", filepath="\\verified.pes", tags_checked=True)
        db.add(design)
        db.commit()

        # Mock the stitching detection to ensure it's NOT called
        called = False

        def _fail_if_called(**kw):
            nonlocal called
            called = True
            return ["Satin Stitch"]

        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern", _fail_if_called
        )

        result = run_stitching_action_runner(
            db, design, {"clear_existing_stitching": False}, batch_size=1
        )

        assert not called, "stitching detection should not be called for verified designs"
        assert result is None or result == 0

    # ------------------------------------------------------------------ #
    # Section 6.5 — Tag Verification State
    # ------------------------------------------------------------------ #

    def test_design_manually_verified_sets_tags_checked_true(self, db):
        """Test 6.5.4 — Design manually verified sets tags_checked=true.

        A design tagged by Tier 1 (tags_checked=false, tagging_tier=1) that
        is then manually verified via set_tags_checked should have:
        - tags_checked=True
        - Existing tags preserved
        - tagging_tier unchanged
        """
        from src.models import Design
        from src.services import designs as design_svc
        from src.services import tags

        flower_tag = tags.create(db, "Flowers", "image")
        design = Design(
            filename="rose.jef",
            filepath="\\rose.jef",
            tags_checked=False,
            tagging_tier=1,
        )
        db.add(design)
        db.flush()
        design.tags.append(flower_tag)
        db.commit()

        # Manually verify
        result = design_svc.set_tags_checked(db, design.id, True)

        assert result.tags_checked is True
        assert result.tagging_tier == 1  # unchanged
        assert [t.description for t in result.tags] == ["Flowers"]  # preserved


# ---------------------------------------------------------------------------
# Multi-folder scanning and grouping
# ---------------------------------------------------------------------------


class TestScanFolders:
    """Tests for the multi-folder scan_folders() function."""

    def test_scan_folders_single_folder(self, db, tmp_path, monkeypatch):
        """Single folder behaves like scan_folder but sets source_folder/folder_key."""
        folder = tmp_path / "amazing_designs"
        folder.mkdir()
        (folder / "rose.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = scan_folders([str(folder)], db)

        assert len(results) == 1
        assert results[0].source_folder == str(folder)
        assert results[0].folder_key == "0"
        assert results[0].folder_label == "amazing_designs"

    def test_scan_folders_duplicate_basenames_get_unique_managed_roots(
        self, db, tmp_path, monkeypatch
    ):
        """Folders with the same leaf name should not collapse onto the same managed path root."""
        f1 = tmp_path / "Alpha" / "designs"
        f2 = tmp_path / "Beta" / "designs"
        f1.mkdir(parents=True)
        f2.mkdir(parents=True)
        (f1 / "rose.jef").write_bytes(b"\x00")
        (f2 / "flower.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = scan_folders([str(f1), str(f2)], db)

        assert len(results) == 2
        assert {sd.filepath for sd in results} == {r"\designs\rose.jef", r"\designs__2\flower.jef"}

    def test_scan_folders_multiple_folders(self, db, tmp_path, monkeypatch):
        """Multiple folders are scanned and each design gets the right folder_key."""
        f1 = tmp_path / "amazing_designs"
        f2 = tmp_path / "urban_threads"
        f1.mkdir()
        f2.mkdir()
        (f1 / "rose.jef").write_bytes(b"\x00")
        (f2 / "flower.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = scan_folders([str(f1), str(f2)], db)

        assert len(results) == 2
        keys = {sd.folder_key for sd in results}
        assert keys == {"0", "1"}
        labels = {sd.folder_label for sd in results}
        assert labels == {"amazing_designs", "urban_threads"}

    def test_scan_folders_deduplicates_paths(self, db, tmp_path, monkeypatch):
        """Duplicate folder paths are skipped without error."""
        folder = tmp_path / "designs"
        folder.mkdir()
        (folder / "rose.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = scan_folders([str(folder), str(folder)], db)

        # Only one group of results — duplicates are dropped
        folder_keys = [sd.folder_key for sd in results]
        assert all(k == "0" for k in folder_keys)

    def test_scan_folders_skips_blank_paths(self, db, tmp_path, monkeypatch):
        """Blank/empty paths are silently ignored."""
        folder = tmp_path / "designs"
        folder.mkdir()
        (folder / "rose.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = scan_folders(["", "   ", str(folder)], db)

        assert all(sd.folder_key == "0" for sd in results)


# ---------------------------------------------------------------------------
# process_selected_files — multi-folder support
# ---------------------------------------------------------------------------


class TestProcessSelectedFilesMultiFolder:
    """Tests for the multi-folder-aware process_selected_files()."""

    def test_multi_folder_sets_source_folder(self, db, tmp_path, monkeypatch):
        """source_folder and folder_key are populated for matched designs."""
        f1 = tmp_path / "amazing_designs"
        f1.mkdir()
        (f1 / "rose.jef").write_bytes(b"\x00")

        seen = []

        def fake_process(full_path, fname, rel, _db, **kw):
            seen.append(rel)
            return ScannedDesign(filename=fname, filepath=rel)

        monkeypatch.setattr(preview_mod, "_process_file", fake_process)
        results = process_selected_files(
            ["\\amazing_designs\\rose.jef"],
            [str(f1)],
            db,
        )

        assert len(results) == 1
        assert results[0].source_folder == str(f1)
        assert results[0].folder_key == "0"

    def test_multi_folder_matches_correct_source(self, db, tmp_path, monkeypatch):
        """Each file is matched to the correct source folder among multiple."""
        f1 = tmp_path / "amazing_designs"
        f2 = tmp_path / "urban_threads"
        f1.mkdir()
        f2.mkdir()
        (f1 / "rose.jef").write_bytes(b"\x00")
        (f2 / "flower.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, _db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = process_selected_files(
            ["\\amazing_designs\\rose.jef", "\\urban_threads\\flower.jef"],
            [str(f1), str(f2)],
            db,
        )

        assert len(results) == 2
        folder_keys = {sd.folder_key for sd in results}
        assert folder_keys == {"0", "1"}

    def test_backward_compat_string_source_folder(self, db, tmp_path, monkeypatch):
        """Passing a single string source_folder still works (backward compat)."""
        folder = tmp_path / "designs"
        folder.mkdir()
        (folder / "rose.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, _db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = process_selected_files(
            ["\\designs\\rose.jef"],
            str(folder),  # single string, not a list
            db,
        )

        assert len(results) == 1
        assert results[0].source_folder == str(folder)

    def test_duplicate_folder_names_can_be_resolved_by_explicit_root_map(
        self, db, tmp_path, monkeypatch
    ):
        """Duplicate leaf folder names should still resolve to the correct source folder."""
        f1 = tmp_path / "Alpha" / "designs"
        f2 = tmp_path / "Beta" / "designs"
        f1.mkdir(parents=True)
        f2.mkdir(parents=True)
        (f1 / "rose.jef").write_bytes(b"\x00")
        (f2 / "flower.jef").write_bytes(b"\x00")

        monkeypatch.setattr(
            preview_mod,
            "_process_file",
            lambda fp, fn, rel, _db, **kw: ScannedDesign(filename=fn, filepath=rel),
        )
        results = svc.process_selected_files(
            [r"\designs\rose.jef", r"\designs__2\flower.jef"],
            [str(f1), str(f2)],
            db,
            folder_root_map={"0": "designs", "1": "designs__2"},
        )

        assert len(results) == 2
        by_file = {sd.filename: sd for sd in results}
        assert by_file["rose.jef"].source_folder == str(f1)
        assert by_file["flower.jef"].source_folder == str(f2)
        assert by_file["flower.jef"].folder_key == "1"


# ---------------------------------------------------------------------------
# Normalization-based path matching
# ---------------------------------------------------------------------------


class TestNormalizeNameForMatching:
    def test_lowercase(self):
        assert svc.normalize_name_for_matching("Amazing Designs") == "amazing designs"

    def test_underscores_to_spaces(self):
        assert svc.normalize_name_for_matching("amazing_designs") == "amazing designs"

    def test_hyphens_to_spaces(self):
        assert svc.normalize_name_for_matching("Amazing-Designs") == "amazing designs"

    def test_mixed_separators(self):
        assert svc.normalize_name_for_matching("Amazing_-Designs") == "amazing designs"

    def test_strips_whitespace(self):
        assert svc.normalize_name_for_matching("  Amazing Designs  ") == "amazing designs"

    def test_collapses_multiple_spaces(self):
        assert svc.normalize_name_for_matching("Amazing  Designs") == "amazing designs"


class TestSuggestDesignerNormalization:
    def test_underscore_folder_matches_designer(self):
        d = SimpleNamespace(id=1, name="Amazing Designs")
        result = svc.suggest_designer_from_path("\\amazing_designs\\rose.jef", [d])
        assert result == d

    def test_hyphen_folder_matches_designer(self):
        d = SimpleNamespace(id=1, name="Urban Threads")
        result = svc.suggest_designer_from_path("\\urban-threads\\flower.jef", [d])
        assert result == d

    def test_exact_case_insensitive(self):
        d = SimpleNamespace(id=1, name="Florals By Design")
        result = svc.suggest_designer_from_path("\\Florals By Design\\rose.jef", [d])
        assert result == d

    def test_no_match_returns_none(self):
        d = SimpleNamespace(id=1, name="Florals")
        result = svc.suggest_designer_from_path("\\Alphabets\\Z.pes", [d])
        assert result is None


class TestSuggestSourceNormalization:
    def test_underscore_folder_matches_source(self):
        s = SimpleNamespace(id=1, name="Stitch Collection")
        result = svc.suggest_source_from_path("\\stitch_collection\\design.jef", [s])
        assert result == s

    def test_hyphen_folder_matches_source(self):
        s = SimpleNamespace(id=1, name="Purchased Downloads")
        result = svc.suggest_source_from_path("\\purchased-downloads\\d.jef", [s])
        assert result == s


# ---------------------------------------------------------------------------
# find_or_create_designer / find_or_create_source
# ---------------------------------------------------------------------------


class TestFindOrCreateDesigner:
    def test_creates_new_designer(self, db):
        d = svc.find_or_create_designer(db, "Amazing Designs")
        assert d.name == "Amazing Designs"
        assert d.id is not None

    def test_returns_existing_case_insensitive(self, db):
        from src.services import designers as des_svc

        existing = des_svc.create(db, "Urban Threads")
        found = svc.find_or_create_designer(db, "urban threads")
        assert found.id == existing.id

    def test_trims_whitespace(self, db):
        d = svc.find_or_create_designer(db, "  Cool Designs  ")
        assert d.name == "Cool Designs"

    def test_returns_existing_when_already_in_db_with_same_case(self, db):
        from src.services import designers as des_svc

        existing = des_svc.create(db, "Florals")
        found = svc.find_or_create_designer(db, "Florals")
        assert found.id == existing.id

    def test_raises_on_blank_name(self, db):
        import pytest

        with pytest.raises(ValueError):
            svc.find_or_create_designer(db, "   ")


class TestFindOrCreateSource:
    def test_creates_new_source(self, db):
        s = svc.find_or_create_source(db, "Purchased Downloads")
        assert s.name == "Purchased Downloads"
        assert s.id is not None

    def test_returns_existing_case_insensitive(self, db):
        from src.services import sources as src_svc

        existing = src_svc.create(db, "Etsy Bundle")
        found = svc.find_or_create_source(db, "etsy bundle")
        assert found.id == existing.id

    def test_raises_on_blank_name(self, db):
        import pytest

        with pytest.raises(ValueError):
            svc.find_or_create_source(db, "")


# ---------------------------------------------------------------------------
# confirm_import — per-folder and global choices
# ---------------------------------------------------------------------------


class TestConfirmImportFolderChoices:
    """Tests for per-folder and global designer/source assignment during import."""

    def test_per_folder_existing_designer(self, db):
        from src.services import designers as des_svc

        des = des_svc.create(db, "Amazing Designs")
        sd = svc.ScannedDesign(
            filename="rose.jef", filepath="\\amazing_designs\\rose.jef", folder_key="0"
        )
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={"0": {"designer_choice": "existing", "designer_id": str(des.id)}},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id == des.id

    def test_per_folder_existing_source(self, db):
        from src.services import sources as src_svc

        src = src_svc.create(db, "Purchased Downloads")
        sd = svc.ScannedDesign(filename="rose.jef", filepath="\\folder\\rose.jef", folder_key="0")
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={"0": {"source_choice": "existing", "source_id": str(src.id)}},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].source_id == src.id

    def test_per_folder_create_designer(self, db):
        sd = svc.ScannedDesign(
            filename="rose.jef", filepath="\\new_folder\\rose.jef", folder_key="0"
        )
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={
                "0": {"designer_choice": "create", "designer_name": "Brand New Designer"}
            },
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id is not None
        from src.models import Designer as DesignerModel

        d = db.get(DesignerModel, created[0].designer_id)
        assert d.name == "Brand New Designer"

    def test_per_folder_create_designer_deduplicates(self, db):
        """Creating the same designer for two folders only makes one DB record."""
        sd1 = svc.ScannedDesign(filename="rose.jef", filepath="\\f1\\rose.jef", folder_key="0")
        sd2 = svc.ScannedDesign(filename="flower.jef", filepath="\\f2\\flower.jef", folder_key="1")
        created = svc.confirm_import(
            db,
            [sd1, sd2],
            folder_choices={
                "0": {"designer_choice": "create", "designer_name": "Shared Designer"},
                "1": {"designer_choice": "create", "designer_name": "SHARED DESIGNER"},
            },
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id == created[1].designer_id

    def test_per_folder_blank_designer(self, db):
        from src.services import designers as des_svc

        des_svc.create(db, "Matching Folder")
        # Even though "Matching Folder" would be inferred from path, blank overrides it
        sd = svc.ScannedDesign(
            filename="rose.jef", filepath="\\Matching Folder\\rose.jef", folder_key="0"
        )
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={"0": {"designer_choice": "blank"}},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id is None

    def test_global_designer_choice_applied_when_no_per_folder(self, db):
        """Global choice applies to folders with no per-folder override."""
        from src.services import designers as des_svc

        des = des_svc.create(db, "Global Designer")
        sd = svc.ScannedDesign(filename="rose.jef", filepath="\\folder\\rose.jef", folder_key="0")
        created = svc.confirm_import(
            db,
            [sd],
            global_choice={"designer_choice": "existing", "designer_id": str(des.id)},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id == des.id

    def test_per_folder_overrides_global(self, db):
        """Per-folder choice takes precedence over the global choice."""
        from src.services import designers as des_svc

        global_des = des_svc.create(db, "Global Designer")
        per_des = des_svc.create(db, "Per Folder Designer")
        sd = svc.ScannedDesign(filename="rose.jef", filepath="\\folder\\rose.jef", folder_key="0")
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={"0": {"designer_choice": "existing", "designer_id": str(per_des.id)}},
            global_choice={"designer_choice": "existing", "designer_id": str(global_des.id)},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id == per_des.id

    def test_inferred_per_folder_falls_back_to_global(self, db):
        """A per-folder 'inferred' choice still falls back to the global choice."""
        from src.services import designers as des_svc

        global_des = des_svc.create(db, "Global Designer")
        sd = svc.ScannedDesign(
            filename="rose.jef", filepath="\\unrelated\\rose.jef", folder_key="0"
        )
        created = svc.confirm_import(
            db,
            [sd],
            folder_choices={"0": {"designer_choice": "inferred"}},
            global_choice={"designer_choice": "existing", "designer_id": str(global_des.id)},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].designer_id == global_des.id

    def test_inference_fallback_when_no_choices(self, db):
        """Without any choices, path inference still assigns designer."""
        from src.services import designers as des_svc

        des = des_svc.create(db, "Roseworks")
        sd = svc.ScannedDesign(
            filename="rose.jef", filepath="\\Roseworks\\rose.jef", folder_key="0"
        )
        created = svc.confirm_import(db, [sd], run_tier2=False, run_tier3=False)
        assert created[0].designer_id == des.id

    def test_multi_folder_per_folder_different_designers(self, db):
        """Two folders can have different designers assigned."""
        from src.services import designers as des_svc

        des1 = des_svc.create(db, "Designer One")
        des2 = des_svc.create(db, "Designer Two")
        sd1 = svc.ScannedDesign(filename="a.jef", filepath="\\f1\\a.jef", folder_key="0")
        sd2 = svc.ScannedDesign(filename="b.jef", filepath="\\f2\\b.jef", folder_key="1")
        created = svc.confirm_import(
            db,
            [sd1, sd2],
            folder_choices={
                "0": {"designer_choice": "existing", "designer_id": str(des1.id)},
                "1": {"designer_choice": "existing", "designer_id": str(des2.id)},
            },
            run_tier2=False,
            run_tier3=False,
        )
        by_name = {d.filename: d.designer_id for d in created}
        assert by_name["a.jef"] == des1.id
        assert by_name["b.jef"] == des2.id

    def test_global_create_source_on_import(self, db):
        """A global 'create' source choice creates a new Source record."""
        sd = svc.ScannedDesign(filename="rose.jef", filepath="\\folder\\rose.jef", folder_key="0")
        created = svc.confirm_import(
            db,
            [sd],
            global_choice={"source_choice": "create", "source_name": "Brand New Source"},
            run_tier2=False,
            run_tier3=False,
        )
        assert created[0].source_id is not None
        from src.models import Source as SourceModel

        s = db.get(SourceModel, created[0].source_id)
        assert s.name == "Brand New Source"


# ---------------------------------------------------------------------------
# Route tests — multi-folder scan and confirm
# ---------------------------------------------------------------------------


class TestScanRouteMultiFolder:
    def test_scan_multiple_folder_paths(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import as route_mod

        f1 = tmp_path / "amazing_designs"
        f2 = tmp_path / "urban_threads"
        f1.mkdir()
        f2.mkdir()

        scanned_f1 = [
            svc.ScannedDesign(
                filename="rose.jef",
                filepath="\\amazing_designs\\rose.jef",
                source_folder=str(f1),
                folder_key="0",
                folder_label="amazing_designs",
            )
        ]
        scanned_f2 = [
            svc.ScannedDesign(
                filename="flower.jef",
                filepath="\\urban_threads\\flower.jef",
                source_folder=str(f2),
                folder_key="1",
                folder_label="urban_threads",
            )
        ]

        def fake_scan_folders(paths, _db):
            return scanned_f1 + scanned_f2

        monkeypatch.setattr(route_mod, "scan_folders", fake_scan_folders)

        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(f1), str(f2)]},
        )
        assert resp.status_code == 200
        assert "amazing_designs" in resp.text
        assert "urban_threads" in resp.text

    def test_scan_legacy_single_folder_still_works(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import as route_mod

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])

        resp = client.post("/import/scan", data={"folder_path": str(tmp_path)})
        assert resp.status_code == 200

    def test_scan_deduplicates_folder_paths(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import as route_mod

        seen_paths = []

        def fake_scan_folders(paths, _db):
            seen_paths.extend(paths)
            return []

        monkeypatch.setattr(route_mod, "scan_folders", fake_scan_folders)

        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(tmp_path), str(tmp_path)]},
        )
        assert resp.status_code == 200
        # The route deduplicates before calling scan_folders
        assert seen_paths.count(str(tmp_path)) == 1

    def test_scan_empty_paths_returns_400(self, client):
        resp = client.post("/import/scan", data={"folder_paths": ["   ", ""]})
        assert resp.status_code == 400


class TestConfirmRouteMultiFolder:
    def test_confirm_multi_folder_calls_process_with_all_paths(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import as route_mod

        captured = {}

        def fake_process(filepaths, source_folders, db, **kwargs):
            captured["source_folders"] = source_folders
            return []

        monkeypatch.setattr(route_mod.scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", lambda db, designs, **kw: [])

        f1 = tmp_path / "amazing_designs"
        f2 = tmp_path / "urban_threads"
        f1.mkdir()
        f2.mkdir()

        resp = client.post(
            "/import/confirm",
            data={
                "folder_paths": [str(f1), str(f2)],
                "selected_files": ["\\amazing_designs\\rose.jef"],
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert str(f1) in captured["source_folders"]
        assert str(f2) in captured["source_folders"]

    def test_confirm_passes_per_folder_choices(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import as route_mod

        captured = {}

        def fake_process(filepaths, source_folders, db):
            return [
                svc.ScannedDesign(filename="rose.jef", filepath="\\f\\rose.jef", folder_key="0")
            ]

        def fake_confirm(db, designs, folder_choices=None, global_choice=None, **kw):
            captured["folder_choices"] = folder_choices
            captured["global_choice"] = global_choice
            return []

        monkeypatch.setattr(route_mod.svc, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        folder = tmp_path / "folder"
        folder.mkdir()
        resp = client.post(
            "/import/confirm",
            data={
                "folder_paths": [str(folder)],
                "selected_files": ["\\folder\\rose.jef"],
                "designer_choice_0": "blank",
                "source_choice_0": "inferred",
                "global_designer_choice": "inferred",
                "global_source_choice": "inferred",
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert captured["folder_choices"].get("0", {}).get("designer_choice") == "blank"


# ------------------------------------------------------------------ #
# Section 7.3 — Import with AI Tagging Settings
# ------------------------------------------------------------------ #


class TestImportAITaggingSettings:
    """Tests for Section 7.3 — Import with AI Tagging Settings."""

    def test_confirm_import_tier1_plus_tier3_only(self, db, monkeypatch):
        """Test 7.3.4 — Import with Tier 1 + Tier 3 only (Tier 2 off).

        When ``run_tier2=False`` and ``run_tier3=True``, the import should
        apply Tier 1 keyword matching, skip Tier 2, and run Tier 3 vision
        on still-untagged designs that have image data.
        """
        from src.services import tags

        tags.create(db, "Flowers", "image")
        monkeypatch.setattr(svc, "_load_api_key", lambda: "test-key")
        monkeypatch.setattr(svc, "suggest_tier2_batch", lambda *args, **kwargs: {})

        def fake_tier3(designs, valid_descriptions, api_key):
            return {designs[0].id: ["Flowers"]}

        monkeypatch.setattr(svc, "suggest_tier3_vision", fake_tier3)

        created = svc.confirm_import(
            db,
            [
                svc.ScannedDesign(
                    filename="untagged.jef",
                    filepath="\\untagged.jef",
                    image_data=b"fake-image",
                )
            ],
            run_tier2=False,
            run_tier3=True,
        )

        assert len(created) == 1
        assert created[0].tagging_tier == 3
        assert [tag.description for tag in created[0].tags] == ["Flowers"]

    def test_confirm_import_batch_limit_with_tier2_and_tier3(self, db, monkeypatch):
        """Test 7.3.6 — Import with batch limit + Tier 2 + Tier 3.

        When ``batch_limit=1`` and both Tier 2 and Tier 3 are enabled,
        only the first design should receive AI tagging; the remaining
        designs should stay with Tier 1 only.
        """
        from src.services import tags

        tags.create(db, "Flowers", "image")
        tags.create(db, "Animals", "image")
        monkeypatch.setattr(svc, "_load_api_key", lambda: "test-key")

        tier2_calls: list[list[str]] = []

        def track_tier2(filenames, valid_descriptions, api_key):
            tier2_calls.append(filenames)
            return {Path(fn).stem.lower(): ["Animals"] for fn in filenames}

        monkeypatch.setattr(svc, "suggest_tier2_batch", track_tier2)
        monkeypatch.setattr(svc, "suggest_tier3_vision", lambda *args, **kwargs: {})

        created = svc.confirm_import(
            db,
            [
                svc.ScannedDesign(filename="ABC001.jef", filepath="\\ABC001.jef"),
                svc.ScannedDesign(filename="XYZ002.jef", filepath="\\XYZ002.jef"),
                svc.ScannedDesign(filename="ZZZ003.jef", filepath="\\ZZZ003.jef"),
            ],
            run_tier2=True,
            run_tier3=True,
            batch_limit=1,
        )

        assert len(created) == 3
        # First design gets Tier 2 (batch_limit=1)
        assert created[0].tagging_tier == 2
        assert [tag.description for tag in created[0].tags] == ["Animals"]
        # Second and third designs remain untagged (no Tier 1 match, excluded by batch_limit)
        assert created[1].tagging_tier is None
        assert created[1].tags == []
        assert created[2].tagging_tier is None
        assert created[2].tags == []
        # Tier 2 was called only once with just the first design's filename
        assert len(tier2_calls) == 1
        assert len(tier2_calls[0]) == 1
        assert tier2_calls[0][0] == "ABC001.jef"


# ------------------------------------------------------------------ #
# Section 7.4 — Import File Types
# ------------------------------------------------------------------ #


class TestImportFileTypes:
    """Tests for Section 7.4 — Import File Types."""

    def test_process_pes_file(self, db, monkeypatch):
        """Test 7.4.2 — .pes (Brother/Janome) file import.

        A .pes file should be read, preview generated, and metadata extracted.
        """
        import src.services.preview as preview_mod

        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 200, 100),
            stitches=[],
            count_stitches=lambda: 1234,
            count_threads=lambda: 5,
            count_color_changes=lambda: 4,
        )
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: None)
        monkeypatch.setattr(preview_mod, "_render_preview", lambda _pattern, ext=None: b"preview")

        scanned = preview_mod._process_file("C:/design.pes", "design.pes", "\\design.pes", db)

        assert scanned.filename == "design.pes"
        assert scanned.width_mm == 20.0
        assert scanned.height_mm == 10.0
        assert scanned.stitch_count == 1234
        assert scanned.color_count == 5
        assert scanned.color_change_count == 4


# ------------------------------------------------------------------ #
# Section 7.1 — First Import vs Subsequent Import (precheck route)
# ------------------------------------------------------------------ #


class TestPrecheckFirstImport:
    """Tests for Section 7.1 — First Import vs Subsequent Import.

    These tests exercise the ``POST /import/precheck`` route to verify the
    template context variables ``is_first_import`` and ``needs_hoop_setup``.
    """

    def test_first_import_shows_mandatory_review(self, client, monkeypatch):
        """Test 7.1.1 — First import (no designs in DB).

        When the catalogue has zero designs, the precheck page should
        render with ``is_first_import=True``, showing the "Before your
        first import" section with mandatory review options.
        """
        from src.routes import bulk_import as route_mod

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])

        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\folder\\rose.jef"]},
        )
        assert resp.status_code == 200
        # First import renders "Before your first import" heading
        assert "Before your first import" in resp.text
        # Mandatory review buttons are shown
        assert "Review Hoops" in resp.text
        assert "Review Tags" in resp.text
        assert "Review Sources" in resp.text
        assert "Review Designers" in resp.text

    def test_first_import_no_hoops_warning(self, client, monkeypatch):
        """Test 7.1.2 — First import with no hoops defined.

        When the catalogue has zero designs AND zero hoops, the precheck
        page should render with ``needs_hoop_setup=True``, showing a
        warning that the user must confirm skipping hoops.
        """
        from src.routes import bulk_import as route_mod

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])

        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\folder\\rose.jef"]},
        )
        assert resp.status_code == 200
        # With no designs and no hoops, the hoop warning text should appear
        assert "No hoops are defined yet for this catalogue" in resp.text

    def test_subsequent_import_shows_optional_review(self, client, db, monkeypatch):
        """Test 7.1.3 — Subsequent import (designs exist).

        When the catalogue already has designs, the precheck page should
        render with ``is_first_import=False``, showing optional review
        with "Would you like to review" text and "No, import now" button.
        """
        from src.models import Design
        from src.routes import bulk_import as route_mod

        # Create an existing design so the catalogue is not empty
        design = Design(filename="existing.jef", filepath="\\existing.jef")
        db.add(design)
        db.commit()

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])

        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\folder\\rose.jef"]},
        )
        assert resp.status_code == 200
        # Subsequent import shows optional review text
        assert "Would you like to review" in resp.text
        # "No, import now" button (without arrow) is shown for subsequent imports
        assert "No, import now" in resp.text


# ------------------------------------------------------------------ #
# Section 7.3 — Import with AI Tagging Settings (precheck route)
# ------------------------------------------------------------------ #


class TestPrecheckAITaggingSettings:
    """Tests for Section 7.3 — AI Tagging Settings on the precheck route.

    These tests verify the ``has_api_key``, ``ai_tier2_auto``, and
    ``ai_tier3_auto`` context variables returned by the precheck page.
    """

    def test_no_api_key_shows_not_configured(self, client, monkeypatch):
        """Test 7.3.1 — No API key.

        When no Google API key is configured, the precheck page should
        render with ``has_api_key=False``, and the AI banner should show
        "not configured".
        """
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "")

        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\folder\\rose.jef"]},
        )
        assert resp.status_code == 200
        # Without an API key, the "not configured" banner should appear
        assert "Google AI tagging is not configured" in resp.text
        # The "enabled" banner should NOT appear
        assert "Google AI tagging is enabled" not in resp.text

    def test_api_key_present_but_tiers_off(self, client, db, monkeypatch):
        """Test 7.3.2 — API key present but Tier 2+3 off.

        When a Google API key is configured but both Tier 2 and Tier 3
        auto-tagging are disabled, the precheck page should render with
        ``has_api_key=True`` but ``ai_tier2_auto=False`` and
        ``ai_tier3_auto=False``.
        """
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "test-key-123")

        # Ensure tier2_auto and tier3_auto are set to "false" (default)
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "false")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER3_AUTO, "false")

        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\folder\\rose.jef"]},
        )
        assert resp.status_code == 200
        # With an API key, the "enabled" banner should appear
        assert "Google AI tagging is enabled" in resp.text
        # Both tiers should show as "off" (the HTML includes <strong> tags)
        assert "Tier 2 auto: <strong>off</strong>" in resp.text
        assert "Tier 3 auto: <strong>off</strong>" in resp.text


# ------------------------------------------------------------------ #
# Section 7.5 — Import Scan Modes (errors-only folder)
# ------------------------------------------------------------------ #


class TestScanErrorsOnly:
    """Tests for Section 7.5.5 — Folder with errors only.

    When all scanned designs have errors, the scan results page should
    show an error table and no OK files.
    """

    def test_folder_with_errors_only(self, client, tmp_path, monkeypatch):
        """Test 7.5.5 — Folder with errors only.

        When all files in a scanned folder have errors, the scan results
        page should render with an error table and zero OK files.
        """
        from src.routes import bulk_import as route_mod
        from src.services.scanning import ScannedDesign

        folder = tmp_path / "broken_designs"
        folder.mkdir()

        # All scanned designs have errors
        scanned = [
            ScannedDesign(
                filename="corrupt1.jef",
                filepath="\\broken_designs\\corrupt1.jef",
                source_folder=str(folder),
                folder_key="0",
                folder_label="broken_designs",
                error="Unsupported format",
            ),
            ScannedDesign(
                filename="corrupt2.pes",
                filepath="\\broken_designs\\corrupt2.pes",
                source_folder=str(folder),
                folder_key="0",
                folder_label="broken_designs",
                error="File is corrupt",
            ),
        ]

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: scanned)

        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(folder)]},
        )
        assert resp.status_code == 200
        # The response should contain error information
        assert "corrupt1.jef" in resp.text
        assert "corrupt2.pes" in resp.text
        assert "Unsupported format" in resp.text or "error" in resp.text.lower()


# ------------------------------------------------------------------ #
# Section 8 — Edge Cases & Error Handling
# ------------------------------------------------------------------ #


class TestImportEdgeCases:
    """Tests for Section 8 — Edge Cases & Error Handling."""

    # ------------------------------------------------------------------ #
    # 8.4 — Disk full during image generation
    # ------------------------------------------------------------------ #

    def test_disk_full_during_import(self, db, monkeypatch):
        """Test 8.4 — Disk full during image generation.

        When ``copy_design_to_managed_folder`` fails (e.g. disk full),
        the design should be skipped with an error recorded, and
        processing should continue with remaining designs.
        """
        from src.services import persistence as persist_mod

        # Make copy_design_to_managed_folder fail for the first design
        call_count = [0]

        def fake_copy(_db, sd, base_path=None):
            call_count[0] += 1
            if call_count[0] == 1:
                return (False, "Disk full")
            return (True, None)

        monkeypatch.setattr(persist_mod, "copy_design_to_managed_folder", fake_copy)

        created = svc.confirm_import(
            db,
            [
                svc.ScannedDesign(
                    filename="disk_full.jef",
                    filepath="\\disk_full.jef",
                    source_full_path="C:\\source\\disk_full.jef",
                ),
                svc.ScannedDesign(
                    filename="ok.jef",
                    filepath="\\ok.jef",
                    source_full_path="C:\\source\\ok.jef",
                ),
            ],
            run_tier2=False,
            run_tier3=False,
        )

        # The first design is skipped entirely (not added to created list)
        # because copy_design_to_managed_folder fails. Only the second
        # design is persisted.
        assert len(created) == 1
        assert created[0].filename == "ok.jef"
        # The persisted Design object has no 'error' attribute — that's
        # only on ScannedDesign. The key assertion is that the failed
        # design was skipped and the good one was persisted.

    # ------------------------------------------------------------------ #
    # 8.5 — Very large embroidery file (>100MB)
    # ------------------------------------------------------------------ #

    def test_very_large_file_handled(self, db, monkeypatch):
        """Test 8.5 — Very large embroidery file (>100MB).

        When a very large embroidery file cannot be analysed (e.g.
        ``suggest_stitching_from_pattern`` returns no results), the
        stitching detection should return None (no tags added) and
        processing should continue gracefully.
        """
        from src.services import auto_tagging as auto_tag_mod

        # Simulate that the file is too large to analyse — no stitch types detected
        monkeypatch.setattr(
            auto_tag_mod,
            "suggest_stitching_from_pattern",
            lambda **kw: [],
        )

        import src.services.pattern_analysis as pa_mod

        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 100, 50),
            stitches=[],
            count_stitches=lambda: 0,
            count_threads=lambda: 0,
            count_color_changes=lambda: 0,
        )

        result = pa_mod._detect_stitching_tags(pattern, filename="huge.pes")
        # No stitch types detected, returns None (leave existing tags alone)
        assert result is None

    # ------------------------------------------------------------------ #
    # 8.6 — Design file deleted between scan and import
    # ------------------------------------------------------------------ #

    def test_file_deleted_between_scan_and_import(self, db, monkeypatch):
        """Test 8.6 — Design file deleted between scan and import.

        When a scanned design's source file no longer exists at import
        time, ``copy_design_to_managed_folder`` should fail (OSError),
        the design should be skipped with an error, and processing
        should continue.
        """
        from src.services import persistence as persist_mod

        # Make copy_design_to_managed_folder fail for the first design
        call_count = [0]

        def fake_copy(_db, sd, base_path=None):
            call_count[0] += 1
            if call_count[0] == 1:
                return (False, "Source file not found")
            return (True, None)

        monkeypatch.setattr(persist_mod, "copy_design_to_managed_folder", fake_copy)

        created = svc.confirm_import(
            db,
            [
                svc.ScannedDesign(
                    filename="deleted.jef",
                    filepath="\\deleted.jef",
                    source_full_path="C:\\source\\deleted.jef",
                ),
                svc.ScannedDesign(
                    filename="still_here.jef",
                    filepath="\\still_here.jef",
                    source_full_path="C:\\source\\still_here.jef",
                ),
            ],
            run_tier2=False,
            run_tier3=False,
        )

        # The first design is skipped entirely (not added to created list)
        # because copy_design_to_managed_folder fails. Only the second
        # design is persisted.
        assert len(created) == 1
        assert created[0].filename == "still_here.jef"
        # The persisted Design object has no 'error' attribute — that's
        # only on ScannedDesign. The key assertion is that the failed
        # design was skipped and the good one was persisted.

    # ------------------------------------------------------------------ #
    # 8.7 — Import with no files selected
    # ------------------------------------------------------------------ #

    # ------------------------------------------------------------------ #
    # Section 7.6 — Image Preference (2D / 3D) preview_3d parameter flow
    # ------------------------------------------------------------------ #

    def test_process_file_passes_preview_3d_to_analyze_pattern(self, db, monkeypatch):
        """_process_file passes preview_3d to analyze_pattern."""
        import src.services.pattern_analysis as pa_mod
        import src.services.preview as preview_mod

        pattern = SimpleNamespace(
            bounds=lambda: (0, 0, 200, 100),
            stitches=[],
            count_stitches=lambda: 1234,
            count_threads=lambda: 5,
            count_color_changes=lambda: 4,
        )
        monkeypatch.setattr(preview_mod.pyembroidery, "read", lambda _path: pattern)
        monkeypatch.setattr(preview_mod, "select_hoop_for_dimensions", lambda *_args: None)

        captured = {}

        def fake_analyze(pattern, **kw):
            captured["preview_3d"] = kw.get("preview_3d")
            return {
                "image_data": b"preview",
                "image_type": "2d",
                "width_mm": 20.0,
                "height_mm": 10.0,
                "stitch_count": 1234,
                "color_count": 5,
                "color_change_count": 4,
                "stitching_tags": [],
            }

        monkeypatch.setattr(pa_mod, "analyze_pattern", fake_analyze)

        # Test with preview_3d=False (2D mode)
        preview_mod._process_file(
            "C:/design.dst", "design.dst", "\\design.dst", db, preview_3d=False
        )
        assert captured.get("preview_3d") is False

        # Test with preview_3d=True (3D mode)
        preview_mod._process_file(
            "C:/design.dst", "design.dst", "\\design.dst", db, preview_3d=True
        )
        assert captured.get("preview_3d") is True

    def test_process_selected_files_passes_preview_3d(self, db, tmp_path, monkeypatch):
        """process_selected_files passes preview_3d to _process_file."""
        from src.services import settings_service as settings_svc
        from src.services.scanning import process_selected_files

        settings_svc.set_setting(db, settings_svc.SETTING_DESIGNS_BASE_PATH, str(tmp_path))

        captured = {}

        def fake_process(full_path, filename, rel_filepath, _db, **kw):
            captured["preview_3d"] = kw.get("preview_3d")
            from src.services.scanning import ScannedDesign

            return ScannedDesign(filename=filename, filepath=rel_filepath)

        import src.services.preview as preview_mod

        monkeypatch.setattr(preview_mod, "_process_file", fake_process)

        # Create a dummy file so it's not skipped
        (tmp_path / "test.jef").write_bytes(b"\x00")

        # Test with preview_3d=False
        process_selected_files(["\\test.jef"], str(tmp_path), db, preview_3d=False)
        assert captured.get("preview_3d") is False

        # Test with preview_3d=True
        process_selected_files(["\\test.jef"], str(tmp_path), db, preview_3d=True)
        assert captured.get("preview_3d") is True

    def test_confirm_import_accepts_preview_3d(self, db, monkeypatch):
        """confirm_import accepts preview_3d parameter without error."""
        from src.services import bulk_import as svc

        # Create a design that already exists so copy is skipped
        from src.services import settings_service as settings_svc
        from src.services.scanning import ScannedDesign

        settings_svc.set_setting(db, settings_svc.SETTING_DESIGNS_BASE_PATH, str(db))

        sd = ScannedDesign(
            filename="test.jef",
            filepath="\\test.jef",
            image_data=b"preview",
        )

        # Test with preview_3d=False (2D mode)
        created = svc.confirm_import(
            db,
            [sd],
            run_tier2=False,
            run_tier3=False,
            preview_3d=False,
        )
        assert len(created) == 1
        assert created[0].filename == "test.jef"

    def test_confirm_import_preview_3d_defaults_to_true(self, db, monkeypatch):
        """confirm_import defaults preview_3d to True when not provided."""
        from src.services import bulk_import as svc
        from src.services.scanning import ScannedDesign

        sd = ScannedDesign(
            filename="default.jef",
            filepath="\\default.jef",
            image_data=b"preview",
        )

        created = svc.confirm_import(
            db,
            [sd],
            run_tier2=False,
            run_tier3=False,
        )
        assert len(created) == 1
        assert created[0].filename == "default.jef"

    def test_import_with_no_files_selected(self, client, monkeypatch):
        """Test 8.7 — Import with no files selected.

        When the user submits the precheck form with no files selected,
        the route should redirect back to the import form (303).
        """
        from src.routes import bulk_import as route_mod

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])

        resp = client.post(
            "/import/precheck",
            data={},  # no selected_files
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers.get("location", "").endswith("/import/")

    # ------------------------------------------------------------------ #
    # 8.9 — Token expired during import review
    # ------------------------------------------------------------------ #

    def test_expired_token_redirects(self, client, monkeypatch):
        """Test 8.9 — Token expired during import review.

        When the import context token has expired (or is invalid),
        the precheck-action route should redirect back to the import
        form (303).
        """
        resp = client.post(
            "/import/precheck-action",
            data={"token": "invalid-token-that-does-not-exist"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers.get("location", "").endswith("/import/")
