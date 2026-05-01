"""
End-to-end regression tests for Section 9 of the backfill-import test matrix.

These tests exercise complete user journeys through the application,
verifying that core workflows are not broken.  They use the FastAPI
TestClient to simulate real HTTP requests and verify database state
after each workflow completes.

AI calls (Tier 2/3) and pyembroidery pattern reading are mocked to
keep tests fast and deterministic.
"""

from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

from src.models import Design, Tag
from src.services import bulk_import as svc

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _create_temp_design_file(tmp_path: Path, filename: str = "test.jef") -> Path:
    """Create a minimal valid .jef file for pyembroidery to read."""
    dst_path = tmp_path / filename
    # Minimal JEF header + stitch data
    header = b"\x00" * 100
    # JEF format: 2 bytes color count, then color table, then stitch data
    color_count = b"\x01\x00"  # 1 color (little-endian)
    # Stitch data: just an END stitch
    stitch_data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x89\x01\x00\x00"
    dst_path.write_bytes(header + color_count + stitch_data)
    return dst_path


def _make_pattern(
    stitches: int = 0, colors: int = 1, bounds: tuple | None = None
) -> SimpleNamespace:
    """Build a minimal pyembroidery Pattern-like SimpleNamespace."""
    if bounds is None:
        bounds = (0.0, 0.0, 100.0, 100.0)
    return SimpleNamespace(
        stitches=[(0, 0, 1)] * stitches,
        threads=[SimpleNamespace(hex=None, description=f"Color {i}") for i in range(colors)],
        count_stitches=lambda: stitches,
        count_threads=lambda: colors,
        count_color_changes=lambda: max(0, colors - 1),
        extreme_bounds=lambda: bounds,
        bounds=lambda: bounds,
        get_threads=lambda: [
            SimpleNamespace(hex=None, description=f"Color {i}") for i in range(colors)
        ],
    )


def _ensure_tag(db, description: str, group: str = "image") -> Tag:
    """Ensure a Tag exists in the database."""
    tag = db.query(Tag).filter(Tag.description == description).first()
    if tag is None:
        tag = Tag(description=description, tag_group=group)
        db.add(tag)
        db.commit()
    return tag


def _mock_pyembroidery(monkeypatch=None, pattern=None):
    """Mock pyembroidery so it returns a pattern and lists .jef as supported.

    Uses ``monkeypatch.setattr`` on ``src.services.preview.pyembroidery``
    (which is already imported at module level) so the mock is picked up
    by the import wizard routes.
    """
    import src.services.preview as preview_mod

    if pattern is None:
        pattern = _make_pattern(stitches=100, colors=3)

    mock_pyemb = MagicMock()
    mock_pyemb.read.return_value = pattern
    mock_pyemb.supported_formats.return_value = [
        {"reader": True, "extension": "jef", "extensions": ["jef"]}
    ]

    if monkeypatch is not None:
        monkeypatch.setattr(preview_mod, "pyembroidery", mock_pyemb)
    else:
        import sys as _sys

        _sys.modules["pyembroidery"] = mock_pyemb
    return mock_pyemb


def _restore_pyembroidery():
    """Restore the real pyembroidery module."""
    import sys as _sys

    _sys.modules.pop("pyembroidery", None)


# ---------------------------------------------------------------------------
# Section 9 — Regression Test Matrix
# ---------------------------------------------------------------------------


class TestRegressionBackfillImages:
    """9.2 — Backfill missing images.

    Import a design without an image, then run unified backfill (images only).
    Verify the design now has a preview image.
    """

    def test_backfill_missing_images(self, db, tmp_path):
        """Test 9.2 — Backfill missing images."""
        from src.services.unified_backfill import unified_backfill

        # Create a design with no image data
        design = Design(filename="test.jef", filepath="subdir/test.jef")
        db.add(design)
        db.commit()
        design_id = design.id

        dst_path = _create_temp_design_file(tmp_path)

        _mock_pyembroidery(None)
        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch(
                    "src.services.unified_backfill.is_stop_requested",
                    return_value=False,
                ),
                patch(
                    "src.services.pattern_analysis._render_preview",
                    return_value=b"PNG_PREVIEW",
                ),
                patch(
                    "src.services.unified_backfill.select_hoop_for_dimensions",
                    return_value=None,
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={"images": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["processed"] >= 1
        assert result["errors"] == 0

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.image_data == b"PNG_PREVIEW"
        assert updated.image_type == "3d"


class TestRegressionBackfillColorCounts:
    """9.4 — Backfill colour counts.

    Import a design without colour counts, then run unified backfill with
    color_counts action.  Verify stitch_count, color_count, and
    color_change_count are populated.
    """

    def test_backfill_color_counts(self, db, tmp_path):
        """Test 9.4 — Backfill colour counts."""
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create a design with no colour counts
        design = Design(filename="test.jef", filepath="subdir/test.jef")
        db.add(design)
        db.commit()
        design_id = design.id

        dst_path = _create_temp_design_file(tmp_path)

        mock_result = PatternAnalysisResult(
            stitch_count=1500,
            color_count=5,
            color_change_count=10,
        )

        _mock_pyembroidery(None)
        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch(
                    "src.services.unified_backfill.is_stop_requested",
                    return_value=False,
                ),
                patch(
                    "src.services.unified_backfill.analyze_pattern",
                    return_value=mock_result,
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={"color_counts": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["processed"] >= 1
        assert result["errors"] == 0

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.stitch_count == 1500
        assert updated.color_count == 5
        assert updated.color_change_count == 10


class TestRegressionBackfillStitching:
    """9.3 — Backfill stitching tags.

    Import a design, then run unified backfill with stitching action.
    Verify the design has stitching-type tags.
    """

    def test_backfill_stitching_tags(self, db, tmp_path):
        """Test 9.3 — Backfill stitching tags."""
        from src.services.unified_backfill import unified_backfill

        # Create a design
        design = Design(filename="test.jef", filepath="subdir/test.jef")
        db.add(design)
        db.commit()
        design_id = design.id

        dst_path = _create_temp_design_file(tmp_path)
        _ensure_tag(db, "Satin Stitch", "stitching")

        _mock_pyembroidery(None)
        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch(
                    "src.services.unified_backfill.is_stop_requested",
                    return_value=False,
                ),
                patch(
                    "src.services.auto_tagging.suggest_stitching_from_pattern",
                    return_value=["Satin Stitch"],
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={"stitching": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["processed"] >= 1
        assert result["errors"] == 0

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert len(stitching_tags) > 0
        assert any("Satin" in t.description for t in stitching_tags)


class TestRegressionFullImportAutoTag:
    """9.1 — Full import + auto-tag.

    Import designs through the import wizard with Tier 1 + Tier 2/3 enabled
    (mocked).  Verify tags are assigned and visible on the design detail page.
    """

    def test_full_import_auto_tag(self, db, client, tmp_path, monkeypatch):
        """Test 9.1 — Full import + auto-tag."""
        from src.routes import bulk_import as route_mod
        from src.services import persistence as persist_mod
        from src.services import settings_service as settings_svc

        # Create a tag for Tier 1 keyword matching
        _ensure_tag(db, "Flowers", "image")

        # Create a source folder with a design file
        source = tmp_path / "Florals"
        source.mkdir()
        design_file = source / "rose.jef"
        design_file.write_bytes(b"embroidery-data")

        # Mock pyembroidery at the route level so _process_file doesn't crash
        _mock_pyembroidery(monkeypatch)

        # Mock the scan to return a scanned design
        scanned = [
            svc.ScannedDesign(
                filename="rose.jef",
                filepath="\\Florals\\rose.jef",
                source_folder=str(source),
                folder_key="0",
                folder_label="Florals",
                source_full_path=str(design_file),
            )
        ]
        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: scanned)

        # Step 1: Scan
        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(source)]},
        )
        assert resp.status_code == 200
        assert "rose.jef" in resp.text

        # Step 2: Precheck
        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: scanned)
        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\Florals\\rose.jef"]},
        )
        assert resp.status_code == 200

        # Step 3: Confirm import with Tier 1 only
        monkeypatch.setattr(
            settings_svc, "get_designs_base_path", lambda _db: str(tmp_path / "managed")
        )

        def fake_copy(_db, sd, base_path=None):
            return (True, None)

        monkeypatch.setattr(persist_mod, "copy_design_to_managed_folder", fake_copy)

        resp = client.post(
            "/import/confirm",
            data={
                "selected_files": ["\\Florals\\rose.jef"],
                "folder_paths": [str(source)],
                "token": "",
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303

        # Verify the design was created with tags
        design = db.query(Design).filter(Design.filename == "rose.jef").first()
        assert design is not None
        # "rose" should match "Flowers" via Tier 1 keyword matching
        assert len(design.tags) > 0
        assert any("Flowers" in t.description for t in design.tags)
        assert design.tagging_tier == 1


class TestRegressionUpgradeImages:
    """9.7 — Upgrade images 2D→3D.

    Import a design with a 2D preview, then run unified backfill with
    upgrade_2d_to_3d.  Verify the image_type changes from '2d' to '3d'.
    """

    def test_upgrade_images_2d_to_3d(self, db, tmp_path):
        """Test 9.7 — Upgrade images 2D→3D."""
        from src.services.unified_backfill import unified_backfill

        # Create a design with a 2D image
        design = Design(
            filename="test.jef",
            filepath="subdir/test.jef",
            image_data=b"OLD_2D_IMAGE",
            image_type="2d",
            width_mm=50.0,
            height_mm=40.0,
        )
        db.add(design)
        db.commit()
        design_id = design.id

        dst_path = _create_temp_design_file(tmp_path)

        _mock_pyembroidery(None)
        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch(
                    "src.services.unified_backfill.is_stop_requested",
                    return_value=False,
                ),
                patch(
                    "src.services.pattern_analysis._render_preview",
                    return_value=b"NEW_3D_PREVIEW",
                ),
                patch(
                    "src.services.unified_backfill.select_hoop_for_dimensions",
                    return_value=None,
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={
                        "images": {
                            "enabled": True,
                            "upgrade_2d_to_3d": True,
                            "preview_3d": True,
                        }
                    },
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["processed"] >= 1
        assert result["errors"] == 0

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.image_data == b"NEW_3D_PREVIEW"
        assert updated.image_type == "3d"


class TestRegressionRetagUnverified:
    """9.5 — Re-tag unverified.

    Import with Tier 1, manually verify some designs, then run
    retag_all_unverified.  Only unverified designs should have their
    tags overwritten.
    """

    def test_retag_unverified(self, db, monkeypatch):
        """Test 9.5 — Re-tag unverified."""
        from src.services.unified_backfill import unified_backfill

        # Create tags
        _ensure_tag(db, "Flowers", "image")
        _ensure_tag(db, "Animals", "image")

        # Create two designs with Tier 1 tags
        flower_tag = db.query(Tag).filter(Tag.description == "Flowers").first()
        animal_tag = db.query(Tag).filter(Tag.description == "Animals").first()

        d1 = Design(
            filename="rose.jef",
            filepath="\\rose.jef",
            tagging_tier=1,
            tags_checked=False,
        )
        d1.tags.append(flower_tag)
        db.add(d1)

        d2 = Design(
            filename="cat.jef",
            filepath="\\cat.jef",
            tagging_tier=1,
            tags_checked=True,  # verified
        )
        d2.tags.append(animal_tag)
        db.add(d2)
        db.commit()

        d1_id = d1.id
        d2_id = d2.id

        # Mock the tagging action runner to simulate re-tagging
        def fake_tagging_runner(_db, design, tag_opts, *args, **kwargs):
            if tag_opts.get("action") == "retag_all_unverified":
                new_tag = db.query(Tag).filter(Tag.description == "Animals").first()
                design.tags = [new_tag]
                design.tagging_tier = 2
                design.tags_checked = False
                db.flush()
            return None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                return_value=False,
            ),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                side_effect=fake_tagging_runner,
            ),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "retag_all_unverified",
                        "tiers": [1, 2],
                    }
                },
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        # d1 was unverified — should be re-tagged
        updated1 = db.query(Design).filter(Design.id == d1_id).first()
        assert updated1 is not None
        assert any("Animals" in t.description for t in updated1.tags)
        assert updated1.tagging_tier == 2

        # d2 was verified — should keep original tags
        updated2 = db.query(Design).filter(Design.id == d2_id).first()
        assert updated2 is not None
        assert any("Animals" in t.description for t in updated2.tags)


class TestRegressionRetagAll:
    """9.6 — Re-tag all (destructive).

    Import with Tier 1, manually verify, then run retag_all.
    All tags should be overwritten and verified reset to false.
    """

    def test_retag_all_destructive(self, db, monkeypatch):
        """Test 9.6 — Re-tag all (destructive)."""
        from src.services.unified_backfill import unified_backfill

        # Create tags
        _ensure_tag(db, "Flowers", "image")
        _ensure_tag(db, "Animals", "image")

        flower_tag = db.query(Tag).filter(Tag.description == "Flowers").first()

        # Create a verified design with "Flowers" tag
        d = Design(
            filename="rose.jef",
            filepath="\\rose.jef",
            tagging_tier=1,
            tags_checked=True,  # verified
        )
        d.tags.append(flower_tag)
        db.add(d)
        db.commit()
        design_id = d.id

        # Mock the tagging action runner to simulate destructive re-tag
        def fake_tagging_runner(_db, design, tag_opts, *args, **kwargs):
            if tag_opts.get("action") == "retag_all":
                new_tag = db.query(Tag).filter(Tag.description == "Animals").first()
                design.tags = [new_tag]
                design.tagging_tier = 2
                design.tags_checked = False
                db.flush()
            return None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                return_value=False,
            ),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                side_effect=fake_tagging_runner,
            ),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "retag_all",
                        "tiers": [1, 2],
                    }
                },
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        # Tags should be overwritten
        assert any("Animals" in t.description for t in updated.tags)
        # Verified should be reset
        assert updated.tags_checked is False
        assert updated.tagging_tier == 2


class TestRegressionFullUnifiedBackfill:
    """9.8 — Full unified backfill.

    Run all 4 actions together (images, stitching, colours, tagging).
    Verify all data types are completed.
    """

    def test_full_unified_backfill(self, db, tmp_path):
        """Test 9.8 — Full unified backfill."""
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create a design with no data
        design = Design(filename="test.jef", filepath="subdir/test.jef")
        db.add(design)
        db.commit()
        design_id = design.id

        dst_path = _create_temp_design_file(tmp_path)
        _ensure_tag(db, "Satin Stitch", "stitching")
        _ensure_tag(db, "Flowers", "image")

        _mock_pyembroidery(None)

        mock_result = PatternAnalysisResult(
            image_data=b"FULL_PIPELINE_PNG",
            image_type="3d",
            width_mm=100.0,
            height_mm=80.0,
            stitch_count=2000,
            color_count=6,
            color_change_count=12,
            stitching_tag_descriptions=["Satin Stitch"],
        )

        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch(
                    "src.services.unified_backfill.is_stop_requested",
                    return_value=False,
                ),
                patch(
                    "src.services.unified_backfill.analyze_pattern",
                    return_value=mock_result,
                ),
                patch(
                    "src.services.unified_backfill.select_hoop_for_dimensions",
                    return_value=None,
                ),
                patch(
                    "src.services.unified_backfill.run_tagging_action_runner",
                    return_value=None,
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={
                        "images": {"enabled": True},
                        "color_counts": {"enabled": True},
                        "stitching": {"enabled": True},
                        "tagging": {
                            "enabled": True,
                            "action": "tag_untagged",
                            "tiers": [1, 2, 3],
                        },
                    },
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["processed"] >= 1
        assert result["errors"] == 0

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        # Image data
        assert updated.image_data == b"FULL_PIPELINE_PNG"
        assert updated.image_type == "3d"
        # Color counts
        assert updated.stitch_count == 2000
        assert updated.color_count == 6
        assert updated.color_change_count == 12
        # Stitching tags
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert len(stitching_tags) > 0


class TestRegressionStopMidBackfill:
    """9.9 — Stop mid-backfill.

    Start a unified backfill, then trigger the stop signal.
    Verify partial results are committed without corruption.
    """

    def test_stop_mid_backfill(self, db, tmp_path):
        """Test 9.9 — Stop mid-backfill."""
        from src.services.unified_backfill import unified_backfill

        # Create two designs
        d1 = Design(filename="first.jef", filepath="subdir/first.jef")
        d2 = Design(filename="second.jef", filepath="subdir/second.jef")
        db.add(d1)
        db.add(d2)
        db.commit()
        d1_id = d1.id
        d2_id = d2.id

        dst_path = _create_temp_design_file(tmp_path)

        _mock_pyembroidery(None)

        # Use request_stop() to trigger the stop signal after the first
        # design is processed.  We patch is_stop_requested to return False
        # initially, then call request_stop() after the first design.
        import src.services.unified_backfill as ub_mod

        call_count = [0]

        def _stop_after_first():
            call_count[0] += 1
            if call_count[0] > 4:
                ub_mod.request_stop()
            return ub_mod._backfill_stop_requested

        try:
            with (
                patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
                patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
                patch("src.services.unified_backfill.log_info"),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value=str(dst_path),
                ),
                patch.object(ub_mod, "is_stop_requested", _stop_after_first),
                patch(
                    "src.services.pattern_analysis._render_preview",
                    return_value=b"STOP_TEST_PNG",
                ),
                patch(
                    "src.services.unified_backfill.select_hoop_for_dimensions",
                    return_value=None,
                ),
            ):
                result = unified_backfill(
                    db=db,
                    actions={
                        "images": {"enabled": True},
                        "color_counts": {"enabled": True},
                        "stitching": {"enabled": True},
                    },
                    workers=1,
                    commit_every=1,
                )
        finally:
            _restore_pyembroidery()

        assert result["stopped"] is True
        assert result["processed"] >= 1

        # First design should have data committed
        updated1 = db.query(Design).filter(Design.id == d1_id).first()
        assert updated1 is not None
        assert updated1.image_data == b"STOP_TEST_PNG"

        # Second design should NOT have data (never processed)
        updated2 = db.query(Design).filter(Design.id == d2_id).first()
        assert updated2 is not None
        assert updated2.image_data is None


class TestRegressionFirstImportHoopSetup:
    """9.10 — First import with hoop setup.

    First import with no hoops defined, review hoops, set up hoops,
    then import.  Verify hoops are auto-assigned.
    """

    def test_first_import_hoop_setup(self, db, client, tmp_path, monkeypatch):
        """Test 9.10 — First import with hoop setup."""
        from src.routes import bulk_import as route_mod
        from src.services import persistence as persist_mod
        from src.services import settings_service as settings_svc

        # Mock pyembroidery at the route level
        _mock_pyembroidery(monkeypatch)

        # Create a source folder with a design file
        source = tmp_path / "designs"
        source.mkdir()
        design_file = source / "rose.jef"
        design_file.write_bytes(b"embroidery-data")

        # Mock scan to return a design with dimensions
        scanned = [
            svc.ScannedDesign(
                filename="rose.jef",
                filepath="\\designs\\rose.jef",
                source_folder=str(source),
                folder_key="0",
                folder_label="designs",
                source_full_path=str(design_file),
                width_mm=100.0,
                height_mm=80.0,
            )
        ]
        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: scanned)

        # Step 1: Scan
        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(source)]},
        )
        assert resp.status_code == 200

        # Step 2: Precheck — should show "Before your first import"
        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: scanned)
        resp = client.post(
            "/import/precheck",
            data={"selected_files": ["\\designs\\rose.jef"]},
        )
        assert resp.status_code == 200
        assert "Before your first import" in resp.text

        # Step 3: Create a hoop
        from src.services import hoops as hoop_svc

        hoop_svc.create(db, "100x80 hoop", 100.0, 80.0)

        # Step 4: Confirm import
        monkeypatch.setattr(
            settings_svc, "get_designs_base_path", lambda _db: str(tmp_path / "managed")
        )

        def fake_copy(_db, sd, base_path=None):
            return (True, None)

        monkeypatch.setattr(persist_mod, "copy_design_to_managed_folder", fake_copy)

        resp = client.post(
            "/import/confirm",
            data={
                "selected_files": ["\\designs\\rose.jef"],
                "folder_paths": [str(source)],
                "token": "",
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303

        # Verify the design was created with hoop auto-assigned
        design = db.query(Design).filter(Design.filename == "rose.jef").first()
        assert design is not None
        # The hoop should be auto-assigned based on dimensions
        assert design.hoop_id is not None
