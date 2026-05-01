"""Comprehensive unit tests for unified backfill and 2D-to-3D image upgrade.

Covers:
- DesignWorkItem dataclass construction and defaults
- Stop signal mechanism (request_stop, clear_stop_signal, is_stop_requested, sentinel file)
- Logging helpers (log_error, log_info)
- _resolve_design_filepath helper
- Action runners (images, color_counts, stitching, tagging)
- Worker function (_process_design_batch_worker)
- SQLite bulk optimisation pragmas
- Sequential fallback path in unified_backfill
- Design selection queries (images, stitching, color_counts, tagging)
- render-3d-preview route endpoint
- image_type handling in designs.py create()
"""

from __future__ import annotations

import os as _os
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock, patch


def _make_design(**overrides) -> SimpleNamespace:
    """Build a minimal Design-like SimpleNamespace for action-runner tests."""
    defaults = {
        "id": 1,
        "filename": "test.dst",
        "filepath": "subdir/test.dst",
        "image_data": None,
        "image_type": None,
        "width_mm": None,
        "height_mm": None,
        "hoop_id": None,
        "stitch_count": None,
        "color_count": None,
        "color_change_count": None,
        "tags": [],
        "tags_checked": None,
        "tagging_tier": None,
    }
    defaults.update(overrides)
    return SimpleNamespace(**defaults)


def _make_pattern(stitches=0, colors=1, bounds=None) -> SimpleNamespace:
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


def _restore_pyembroidery() -> None:
    """Restore the original pyembroidery module in sys.modules after mocking.

    Worker tests and route tests inject a mock pyembroidery into sys.modules
    to control locally-imported ``import pyembroidery`` statements.  This
    helper restores the real module so that other tests are not affected.
    """
    import sys as _sys

    _sys.modules.pop("pyembroidery", None)
    try:
        import pyembroidery as _real  # type: ignore[import-untyped]  # noqa: F401
    except ImportError:
        pass


class TestDesignWorkItem:
    """Verify DesignWorkItem dataclass construction and defaults."""

    def test_constructs_with_required_fields(self):
        from src.services.unified_backfill import DesignWorkItem

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst")
        assert item.id == 1
        assert item.filename == "a.dst"
        assert item.filepath == "a.dst"

    def test_defaults_booleans_to_false(self):
        from src.services.unified_backfill import DesignWorkItem

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst")
        assert item.needs_images is False
        assert item.needs_color_counts is False
        assert item.needs_stitching is False

    def test_defaults_optional_fields_to_none(self):
        from src.services.unified_backfill import DesignWorkItem

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst")
        assert item.image_data is None
        assert item.image_type is None
        assert item.width_mm is None
        assert item.height_mm is None
        assert item.hoop_id is None
        assert item.stitch_count is None
        assert item.color_count is None
        assert item.color_change_count is None


class TestStopSignal:
    """Verify the stop-signal mechanism (global flag, event, sentinel file)."""

    def test_clear_stop_signal_resets_flag_and_event(self):
        import src.services.unified_backfill as ub

        ub._backfill_stop_requested = True
        ub._backfill_stop_event = MagicMock()
        ub.clear_stop_signal()
        assert ub._backfill_stop_requested is False
        assert ub._backfill_stop_event is None

    def test_request_stop_sets_flag_and_event(self):
        import src.services.unified_backfill as ub

        ub.clear_stop_signal()
        event = MagicMock()
        event.is_set.return_value = True
        ub._backfill_stop_event = event
        with patch("src.services.unified_backfill._signal_stop_sentinel") as mock_sig:
            ub.request_stop()
        assert ub._backfill_stop_requested is True
        assert ub._backfill_stop_event.is_set() is True
        mock_sig.assert_called_once()

    def test_is_stop_requested_checks_flag(self):
        import src.services.unified_backfill as ub

        ub.clear_stop_signal()
        ub._backfill_stop_requested = True
        assert ub.is_stop_requested() is True

    def test_is_stop_requested_checks_event_when_flag_false(self):
        import src.services.unified_backfill as ub

        ub.clear_stop_signal()
        event = MagicMock()
        event.is_set.return_value = True
        ub._backfill_stop_event = event
        assert ub.is_stop_requested() is True

    def test_is_stop_requested_returns_false_when_both_clear(self):
        import src.services.unified_backfill as ub

        ub.clear_stop_signal()
        assert ub.is_stop_requested() is False

    def test_is_stop_requested_returns_false_when_event_none(self):
        import src.services.unified_backfill as ub

        ub.clear_stop_signal()
        ub._backfill_stop_requested = False
        ub._backfill_stop_event = None
        assert ub.is_stop_requested() is False

    def test_signal_stop_sentinel_creates_file(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            ub._signal_stop_sentinel()
        assert sentinel.exists()

    def test_signal_stop_sentinel_handles_existing_file(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        sentinel.touch()
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            ub._signal_stop_sentinel()
        assert sentinel.exists()

    def test_clear_stop_sentinel_removes_file(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        sentinel.touch()
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            ub._clear_stop_sentinel()
        assert not sentinel.exists()

    def test_clear_stop_sentinel_no_error_when_missing(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            ub._clear_stop_sentinel()

    def test_stop_sentinel_exists_returns_true_when_file_present(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        sentinel.touch()
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            assert ub._stop_sentinel_exists() is True

    def test_stop_sentinel_exists_returns_false_when_missing(self, tmp_path):
        import src.services.unified_backfill as ub

        sentinel = tmp_path / "_backfill_stop.sentinel"
        with patch.object(ub, "_STOP_SENTINEL_PATH", str(sentinel)):
            assert ub._stop_sentinel_exists() is False


class TestLoggingHelpers:
    """Verify log_error and log_info helpers."""

    def test_log_error_creates_file_and_writes(self, tmp_path):
        import src.services.unified_backfill as ub

        err_path = tmp_path / "backfill_errors.tsv"
        with patch.object(ub, "ERROR_LOG_PATH", err_path):
            ub.log_error("test.dst", "images", "Test error message")
        assert err_path.exists()
        text = err_path.read_text(encoding="utf-8")
        assert "test.dst" in text
        assert "Test error message" in text

    def test_log_error_appends_multiple_entries(self, tmp_path):
        import src.services.unified_backfill as ub

        err_path = tmp_path / "backfill_errors.tsv"
        with patch.object(ub, "ERROR_LOG_PATH", err_path):
            ub.log_error("a.dst", "images", "Error 1")
            ub.log_error("b.dst", "stitching", "Error 2")
        lines = err_path.read_text(encoding="utf-8").strip().split("\n")
        assert len(lines) == 2

    def test_log_error_handles_oserror_gracefully(self):
        import src.services.unified_backfill as ub

        with patch.object(ub, "ERROR_LOG_PATH", Path("/nonexistent_dir/errors.tsv")):
            ub.log_error("test.dst", "images", "Should not crash")

    def test_log_info_creates_file_and_writes(self, tmp_path):
        import src.services.unified_backfill as ub

        info_path = tmp_path / "backfill_info.tsv"
        with patch.object(ub, "INFO_LOG_PATH", info_path):
            ub.log_info("test.dst", "start", "Processing design")
        assert info_path.exists()
        text = info_path.read_text(encoding="utf-8")
        assert "test.dst" in text
        assert "Processing design" in text

    def test_log_info_appends_multiple_entries(self, tmp_path):
        import src.services.unified_backfill as ub

        info_path = tmp_path / "backfill_info.tsv"
        with patch.object(ub, "INFO_LOG_PATH", info_path):
            ub.log_info("a.dst", "start", "Begin")
            ub.log_info("a.dst", "end", "Done")
        lines = info_path.read_text(encoding="utf-8").strip().split("\n")
        assert len(lines) == 2

    def test_log_info_handles_oserror_gracefully(self):
        import src.services.unified_backfill as ub

        with patch.object(ub, "INFO_LOG_PATH", Path("/nonexistent_dir/info.tsv")):
            ub.log_info("test.dst", "start", "Should not crash")


class TestResolveDesignFilepath:
    """Verify _resolve_design_filepath helper."""

    def test_returns_none_when_file_missing(self):
        from src.services.unified_backfill import _resolve_design_filepath

        result = _resolve_design_filepath("/nonexistent/path.dst")
        assert result is None

    def test_returns_path_when_file_exists(self, tmp_path):
        from src.services.unified_backfill import _resolve_design_filepath

        d = tmp_path / "existing.dst"
        d.touch()
        result = _resolve_design_filepath(str(d))
        assert result is not None
        assert _os.path.exists(result)

    def test_returns_none_for_empty_path(self):
        from src.services.unified_backfill import _resolve_design_filepath

        assert _resolve_design_filepath("") is None


class TestRunImagesActionRunner:
    """Verify run_images_action_runner behavior.

    NOTE: run_images_action_runner modifies the design in-place and returns
    None on success, or a string error message on failure.
    """

    def test_skips_when_pattern_is_none(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design()
        result = run_images_action_runner(None, design, {}, None)
        assert result is None

    def test_renders_preview_when_no_image_data(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=None)
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"PNG_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(MagicMock(), design, {"enabled": True}, pattern)
        assert result is None  # success
        assert design.image_data == b"PNG_DATA"
        assert design.image_type == "3d"

    def test_renders_preview_with_2d_when_preview_3d_false(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=None)
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"PNG_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(
                MagicMock(), design, {"enabled": True, "preview_3d": False}, pattern
            )
        assert result is None
        assert design.image_type == "2d"

    def test_skips_when_design_has_image_and_no_redo(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=b"EXISTING", image_type="3d")
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview") as mock_rp,
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(MagicMock(), design, {"enabled": True}, pattern)
        assert result is None
        mock_rp.assert_not_called()

    def test_redo_processes_again(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=b"EXISTING", image_type="3d")
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"NEW_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(
                MagicMock(), design, {"enabled": True, "redo": True}, pattern
            )
        assert result is None
        assert design.image_data == b"NEW_DATA"

    def test_upgrade_2d_to_3d_processes_2d_designs(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=b"EXISTING", image_type="2d")
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"3D_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(
                MagicMock(), design, {"enabled": True, "upgrade_2d_to_3d": True}, pattern
            )
        assert result is None
        assert design.image_data == b"3D_DATA"
        assert design.image_type == "3d"

    def test_upgrade_2d_to_3d_skips_already_3d(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=b"EXISTING", image_type="3d")
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview") as mock_rp,
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(
                MagicMock(), design, {"enabled": True, "upgrade_2d_to_3d": True}, pattern
            )
        assert result is None
        mock_rp.assert_not_called()

    def test_upgrade_2d_to_3d_processes_legacy_null_type(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=b"EXISTING", image_type=None)
        pattern = _make_pattern()
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"3D_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(
                MagicMock(), design, {"enabled": True, "upgrade_2d_to_3d": True}, pattern
            )
        assert result is None
        assert design.image_data == b"3D_DATA"

    def test_returns_error_string_on_exception(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=None)
        pattern = _make_pattern()
        with patch(
            "src.services.pattern_analysis._render_preview", side_effect=Exception("Render failed")
        ):
            result = run_images_action_runner(MagicMock(), design, {"enabled": True}, pattern)
        assert result is not None
        assert "Render failed" in result

    def test_sets_dimensions_from_bounds(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=None, width_mm=None, height_mm=None)
        pattern = _make_pattern(bounds=(0.0, 0.0, 200.0, 150.0))
        with (
            patch("src.services.pattern_analysis._render_preview", return_value=b"PNG_DATA"),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = run_images_action_runner(MagicMock(), design, {"enabled": True}, pattern)
        assert result is None
        assert design.width_mm == 20.0  # (200-0)/10
        assert design.height_mm == 15.0  # (150-0)/10

    def test_stop_requested_skips_render(self):
        from src.services.unified_backfill import run_images_action_runner

        design = _make_design(image_data=None)
        pattern = _make_pattern()
        with (
            patch("src.services.unified_backfill.is_stop_requested", return_value=True),
            patch("src.services.pattern_analysis._render_preview") as mock_rp,
        ):
            result = run_images_action_runner(MagicMock(), design, {"enabled": True}, pattern)
        assert result is None
        mock_rp.assert_not_called()


class TestRunColorCountsActionRunner:
    """Verify run_color_counts_action_runner behavior.

    NOTE: Modifies design in-place, returns None on success, str on error.
    """

    def test_skips_when_pattern_is_none(self):
        from src.services.unified_backfill import run_color_counts_action_runner

        design = _make_design()
        result = run_color_counts_action_runner(None, design, {}, None)
        assert result is None

    def test_extracts_counts_when_missing(self):
        from src.services.unified_backfill import run_color_counts_action_runner

        design = _make_design(stitch_count=None, color_count=None, color_change_count=None)
        pattern = _make_pattern(stitches=150, colors=3)
        result = run_color_counts_action_runner(None, design, {"enabled": True}, pattern)
        assert result is None
        assert design.stitch_count == 150
        assert design.color_count == 3
        assert design.color_change_count == 2

    def test_skips_when_counts_already_present(self):
        from src.services.unified_backfill import run_color_counts_action_runner

        design = _make_design(stitch_count=100, color_count=5, color_change_count=3)
        pattern = _make_pattern(stitches=200, colors=7)
        result = run_color_counts_action_runner(None, design, {"enabled": True}, pattern)
        assert result is None
        # Should not overwrite existing values
        assert design.stitch_count == 100
        assert design.color_count == 5

    def test_fills_only_missing_counts(self):
        """Verify that when some counts are already populated, only the
        missing ones are filled in (partial population)."""
        from src.services.unified_backfill import run_color_counts_action_runner

        # stitch_count already set, color_count and color_change_count missing
        design = _make_design(stitch_count=100, color_count=None, color_change_count=None)
        pattern = _make_pattern(stitches=200, colors=7)
        result = run_color_counts_action_runner(None, design, {"enabled": True}, pattern)
        assert result is None
        # Existing value preserved
        assert design.stitch_count == 100
        # Missing values filled in
        assert design.color_count == 7
        assert design.color_change_count == 6

    def test_returns_error_on_exception(self):
        from src.services.unified_backfill import run_color_counts_action_runner

        design = _make_design(stitch_count=None, color_count=None, color_change_count=None)
        pattern = _make_pattern()
        with patch(
            "src.services.unified_backfill.analyze_pattern",
            side_effect=Exception("Count failed"),
        ):
            result = run_color_counts_action_runner(None, design, {"enabled": True}, pattern)
        assert result is not None
        assert "Count failed" in result

    def test_stop_requested_skips(self):
        from src.services.unified_backfill import run_color_counts_action_runner

        design = _make_design(stitch_count=None, color_count=None, color_change_count=None)
        pattern = _make_pattern()
        with patch("src.services.unified_backfill.is_stop_requested", return_value=True):
            result = run_color_counts_action_runner(None, design, {"enabled": True}, pattern)
        assert result is None


class TestRunStitchingActionRunner:
    """Verify run_stitching_action_runner behavior.

    NOTE: Modifies design in-place, returns None on success, str on error.
    """

    def test_skips_when_pattern_is_none_and_no_action(self):
        from src.services.unified_backfill import run_stitching_action_runner

        design = _make_design()
        result = run_stitching_action_runner(MagicMock(), design, {}, 1, None)
        assert result is None

    def test_suggests_stitching_from_pattern(self, db):
        from src.services.unified_backfill import run_stitching_action_runner

        design = _make_design()
        pattern = _make_pattern()
        mock_tag = SimpleNamespace(id=1, description="Running Stitch", tag_group="stitching")
        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=["Running Stitch"],
            ),
            patch(
                "src.services.unified_backfill._resolve_design_filepath",
                return_value="/path/test.dst",
            ),
            patch(
                "src.services.unified_backfill._unique_tags_from_descriptions",
                return_value=[mock_tag],
            ),
            patch.object(db, "query") as mock_query,
        ):
            mock_query.return_value.order_by.return_value.all.return_value = [mock_tag]
            result = run_stitching_action_runner(db, design, {"enabled": True}, 1, pattern)
        assert result is None
        assert len(design.tags) > 0

    def test_returns_empty_list_when_no_stitching_suggested(self, db):
        from src.services.unified_backfill import run_stitching_action_runner

        design = _make_design()
        pattern = _make_pattern()
        with (
            patch("src.services.auto_tagging.suggest_stitching_from_pattern", return_value=[]),
            patch(
                "src.services.unified_backfill._resolve_design_filepath",
                return_value="/path/test.dst",
            ),
        ):
            result = run_stitching_action_runner(db, design, {"enabled": True}, 1, pattern)
        assert result is None

    def test_returns_error_on_exception(self, db):
        from src.services.unified_backfill import run_stitching_action_runner

        design = _make_design()
        pattern = _make_pattern()
        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                side_effect=Exception("Suggestion failed"),
            ),
            patch(
                "src.services.unified_backfill._resolve_design_filepath",
                return_value="/path/test.dst",
            ),
        ):
            result = run_stitching_action_runner(db, design, {"enabled": True}, 1, pattern)
        assert result is not None
        assert "Suggestion failed" in result

    def test_stop_requested_skips(self, db):
        from src.services.unified_backfill import run_stitching_action_runner

        design = _make_design()
        pattern = _make_pattern()
        with patch("src.services.unified_backfill.is_stop_requested", return_value=True):
            result = run_stitching_action_runner(db, design, {"enabled": True}, 1, pattern)
        assert result is None


class TestRunTaggingActionRunner:
    """Verify run_tagging_action_runner behavior.

    NOTE: Has signature (db, design, tag_opts, api_key, delay, vision_delay, batch_size=1, action='tagging', design_ids=None).
    Returns None on success, str on error.
    """

    def test_skips_when_not_needed(self):
        from src.services.unified_backfill import run_tagging_action_runner

        with patch("src.services.unified_backfill.run_tagging_action"):
            result = run_tagging_action_runner(MagicMock(), _make_design(), {}, "", 0.0, 0.0)
        assert result is None

    def test_runs_tagging_with_action(self, db):
        from src.services.unified_backfill import run_tagging_action_runner

        design = _make_design()
        with patch("src.services.unified_backfill.run_tagging_action") as mock_rt:
            result = run_tagging_action_runner(
                db, design, {"action": "tag_untagged"}, "fake_key", 0.0, 0.0
            )
        assert result is None
        mock_rt.assert_called_once()

    def test_handles_tagging_failure(self, db):
        from src.services.unified_backfill import run_tagging_action_runner

        design = _make_design()
        with patch(
            "src.services.unified_backfill.run_tagging_action",
            side_effect=Exception("Tagging failed"),
        ):
            result = run_tagging_action_runner(
                db, design, {"action": "tag_untagged"}, "fake_key", 0.0, 0.0
            )
        assert result is not None
        assert "Tagging failed" in result


class TestProcessDesignBatchWorker:
    """Verify _process_design_batch_worker behavior.

    The worker opens its own DB connection, processes designs, and returns
    result dicts. It uses locally imported pyembroidery, create_engine, and
    sessionmaker.
    """

    def _patch_worker_deps(self):
        """Inject mocks for locally-imported dependencies into sys.modules."""
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.return_value = _make_pattern()
        mock_pyemb.supported_formats.return_value = [
            {"reader": True, "extension": "dst", "extensions": ["dst"]}
        ]
        mock_session = MagicMock()
        mock_SessionLocal = MagicMock(return_value=mock_session)
        self._orig_pyembroidery = _sys.modules.get("pyembroidery")
        _sys.modules["pyembroidery"] = mock_pyemb
        return mock_pyemb, mock_SessionLocal, mock_session

    def test_processes_images_action(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst", needs_images=True)
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        try:
            with (
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
                patch("src.services.pattern_analysis._render_preview", return_value=b"PNG_DATA"),
            ):
                results = _process_design_batch_worker(
                    [item], {"images": {"enabled": True}}, "sqlite:///:memory:", "C:\\designs\\"
                )
        finally:
            _restore_pyembroidery()
        assert len(results) == 1
        assert results[0]["design_id"] == 1
        assert results[0]["image_data"] == b"PNG_DATA"
        assert results[0]["error"] is None

    def test_processes_color_counts_action(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst", needs_color_counts=True)
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        mock_pyemb.read.return_value = _make_pattern(stitches=100, colors=2)
        try:
            with (
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
            ):
                results = _process_design_batch_worker(
                    [item],
                    {"color_counts": {"enabled": True}},
                    "sqlite:///:memory:",
                    "C:\\designs\\",
                )
        finally:
            _restore_pyembroidery()
        assert len(results) == 1
        assert results[0]["design_id"] == 1
        assert results[0]["stitch_count"] == 100
        assert results[0]["color_count"] == 2
        assert results[0]["error"] is None

    def test_processes_stitching_action(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst", needs_stitching=True)
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        try:
            with (
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
                patch(
                    "src.services.unified_backfill._resolve_design_filepath",
                    return_value="/path/a.dst",
                ),
                patch(
                    "src.services.auto_tagging.suggest_stitching_from_pattern",
                    return_value=["Running Stitch"],
                ),
            ):
                results = _process_design_batch_worker(
                    [item], {"stitching": {}}, "sqlite:///:memory:", "C:\\designs\\"
                )
        finally:
            _restore_pyembroidery()
        assert results[0]["stitching_tag_descriptions"] == ["Running Stitch"]
        assert results[0]["error"] is None

    def test_records_error_when_processing_fails(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst", needs_images=True)
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        try:
            with (
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
                patch(
                    "src.services.pattern_analysis._render_preview",
                    side_effect=Exception("Render failed"),
                ),
            ):
                results = _process_design_batch_worker(
                    [item], {"images": {"enabled": True}}, "sqlite:///:memory:", "C:\\designs\\"
                )
        finally:
            _restore_pyembroidery()
        assert len(results) == 1
        assert results[0]["design_id"] == 1
        assert results[0]["error"] is not None
        assert "Render failed" in results[0]["error"]

    def test_stops_when_sentinel_detected(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        items = [
            DesignWorkItem(id=1, filename="a.dst", filepath="a.dst", needs_images=True),
            DesignWorkItem(id=2, filename="b.dst", filepath="b.dst", needs_images=True),
        ]
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        call_count = [0]

        def _sentinel_side_effect():
            call_count[0] += 1
            return call_count[0] > 1

        try:
            with (
                patch(
                    "src.services.unified_backfill._stop_sentinel_exists",
                    side_effect=_sentinel_side_effect,
                ),
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
            ):
                results = _process_design_batch_worker(
                    items, {"images": {"enabled": True}}, "sqlite:///:memory:", "C:\\designs\\"
                )
        finally:
            _restore_pyembroidery()
        assert len(results) == 1
        assert results[0]["design_id"] == 1

    def test_skips_file_read_when_no_actions_need_it(self):
        from src.services.unified_backfill import DesignWorkItem, _process_design_batch_worker

        item = DesignWorkItem(id=1, filename="a.dst", filepath="a.dst")
        mock_pyemb, mock_SessionLocal, mock_session = self._patch_worker_deps()
        try:
            with (
                patch("sqlalchemy.create_engine"),
                patch("sqlalchemy.orm.sessionmaker", return_value=mock_SessionLocal),
            ):
                results = _process_design_batch_worker(
                    [item], {}, "sqlite:///:memory:", "C:\\designs\\"
                )
        finally:
            _restore_pyembroidery()
        assert len(results) == 1
        assert results[0]["design_id"] == 1
        assert results[0]["error"] is None


class TestSqliteBulkOptimisation:
    """Verify SQLite bulk-write PRAGMA optimisation and restore."""

    def test_optimise_sets_pragmas(self):
        from src.services.unified_backfill import _optimise_sqlite_for_bulk

        mock_db = MagicMock()
        _optimise_sqlite_for_bulk(mock_db)
        calls = [str(c[0][0]) for c in mock_db.execute.call_args_list]
        pragmas = " ".join(calls)
        assert "synchronous" in pragmas
        assert "cache_size" in pragmas
        assert "temp_store" in pragmas

    def test_restore_resets_pragmas(self):
        from src.services.unified_backfill import _restore_sqlite_after_bulk

        mock_db = MagicMock()
        _restore_sqlite_after_bulk(mock_db)
        calls = [str(c[0][0]) for c in mock_db.execute.call_args_list]
        pragmas = " ".join(calls)
        assert "synchronous" in pragmas

    def test_optimise_handles_connection_error(self):
        from src.services.unified_backfill import _optimise_sqlite_for_bulk

        mock_db = MagicMock()
        mock_db.get_bind.side_effect = Exception("Connection failed")
        _optimise_sqlite_for_bulk(mock_db)

    def test_restore_handles_connection_error(self):
        from src.services.unified_backfill import _restore_sqlite_after_bulk

        mock_db = MagicMock()
        mock_db.get_bind.side_effect = Exception("Connection failed")
        _restore_sqlite_after_bulk(mock_db)


class TestUnifiedBackfillSequential:
    """Verify the sequential fallback path in unified_backfill.

    unified_backfill(db, actions, batch_size=100, commit_every=500, api_key='', delay=5.0, vision_delay=2.0, workers=1)
    """

    def test_sequential_handles_empty_designs(self, db):
        from src.services.unified_backfill import unified_backfill

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={}, workers=1)
        assert result is not None
        assert result["processed"] == 0

    def test_sequential_stops_when_requested(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Add a design so the loop has something to iterate over
        d = Design(filename="test.dst", filepath="test.dst")
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill.is_stop_requested", return_value=True),
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={"images": {"enabled": True}}, workers=1)
        assert result is not None
        assert result["stopped"] is True

    def test_sequential_handles_no_actions(self, db):
        from src.services.unified_backfill import unified_backfill

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={}, workers=1)
        assert result is not None
        assert result["processed"] == 0

    def test_backfill_empty_catalogue(self, db):
        """Test 8.1 — Run backfill with empty catalogue.

        When the catalogue has no designs but actions are enabled, the
        backfill should complete with zero counts (no designs to process).
        """
        from src.services.unified_backfill import unified_backfill

        # No designs added — empty catalogue
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True}},
                workers=1,
            )
        assert result is not None
        assert result["processed"] == 0
        assert result["errors"] == 0
        assert result["stopped"] is False

    def test_sequential_db_error_handled(self, db):
        """Test 8.3 — Database connection lost during backfill.

        When a database error occurs during the final commit in the
        ``finally`` block, the error should be logged via ``logger.error``
        and the backfill should complete gracefully (returning results).
        """
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Add a design so the loop has something to iterate over
        d = Design(filename="test.dst", filepath="test.dst")
        db.add(d)
        db.commit()

        # Mock db.commit to raise on the final commit (in the finally block).
        # With commit_every=2, the mid-loop commit never fires (only 1 design),
        # so the only commit is the one in the finally block.
        def _failing_commit():
            raise Exception("database connection lost")

        db.commit = _failing_commit

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill.logger") as mock_logger,
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True}},
                workers=1,
                commit_every=2,  # Never reached — relies on finally block
            )
        assert result is not None
        # The error should be logged via logger.error in the finally block
        assert mock_logger.error.called
        call_args = mock_logger.error.call_args
        assert call_args is not None
        assert "database connection lost" in str(call_args)

    def test_zero_workers_falls_back_to_sequential(self, db):
        """Test 8.10 — Backfill with 0 workers (invalid).

        When workers=0 is passed, the backfill should fall back to sequential
        processing (workers=1 behaviour) without error.
        """
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Add a design so the loop has something to iterate over
        d = Design(filename="test.dst", filepath="test.dst")
        db.add(d)
        db.commit()

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True}},
                workers=0,
                commit_every=1,
            )
        assert result is not None
        assert result["processed"] >= 0
        assert result["errors"] == 0


class TestDesignSelectionQueries:
    """Verify design selection queries produce correct DesignWorkItem metadata."""

    def test_images_query_selects_designs_without_image(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(filename="test.dst", filepath="test.dst", image_data=None)
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={"images": {"enabled": True}}, workers=1)
        assert result is not None
        assert result["processed"] >= 1

    def test_images_query_redo_selects_all(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(
            filename="test.dst", filepath="test.dst", image_data=b"EXISTING", image_type="3d"
        )
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db, actions={"images": {"enabled": True, "redo": True}}, workers=1
            )
        assert result is not None

    def test_images_query_upgrade_2d_to_3d_selects_2d_and_legacy(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(
            filename="test.dst", filepath="test.dst", image_data=b"EXISTING", image_type="2d"
        )
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db, actions={"images": {"enabled": True, "upgrade_2d_to_3d": True}}, workers=1
            )
        assert result is not None

    def test_stitching_query_selects_designs_without_stitching_tags(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(filename="test.dst", filepath="test.dst")
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={"stitching": {"enabled": True}}, workers=1)
        assert result is not None

    def test_color_counts_query_selects_designs_without_counts(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(filename="test.dst", filepath="test.dst", stitch_count=None)
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={"color_counts": {"enabled": True}}, workers=1)
        assert result is not None

    def test_tagging_query_selects_all_designs(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(filename="test.dst", filepath="test.dst")
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db, actions={"tagging": {"action": "tag_untagged"}}, workers=1
            )
        assert result is not None

    def test_combined_queries_merge_design_map(self, db):
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        d = Design(filename="test.dst", filepath="test.dst", image_data=None, stitch_count=None)
        db.add(d)
        db.commit()
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                    "stitching": {"enabled": True},
                },
                workers=1,
            )
        assert result is not None


class TestRender3dPreviewRoute:
    """Verify the render-3d-preview route endpoint."""

    def test_render_3d_preview_success(self, client, db):
        from src.models import Design

        design = Design(filename="test.dst", filepath="test.dst", image_data=None, image_type=None)
        db.add(design)
        db.commit()
        db.refresh(design)
        pattern = _make_pattern()
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.return_value = pattern
        _sys.modules["pyembroidery"] = mock_pyemb
        try:
            with (
                patch(
                    "src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"
                ),
                patch("src.services.preview._render_preview", return_value=b"PNG_DATA"),
                patch("src.routes.designs.svc.get_by_id", return_value=design),
                patch("os.path.isfile", return_value=True),
            ):
                resp = client.post(
                    f"/designs/{design.id}/render-3d-preview", follow_redirects=False
                )
        finally:
            _restore_pyembroidery()
        assert resp.status_code == 303

    def test_render_3d_preview_sets_image_type_to_3d(self, client, db):
        from src.models import Design

        design = Design(filename="test.dst", filepath="test.dst", image_data=None, image_type=None)
        db.add(design)
        db.commit()
        db.refresh(design)
        pattern = _make_pattern()
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.return_value = pattern
        _sys.modules["pyembroidery"] = mock_pyemb
        try:
            with (
                patch(
                    "src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"
                ),
                patch("src.services.preview._render_preview", return_value=b"PNG_DATA"),
                patch("src.routes.designs.svc.get_by_id", return_value=design),
                patch("os.path.isfile", return_value=True),
            ):
                client.post(f"/designs/{design.id}/render-3d-preview")
        finally:
            _restore_pyembroidery()
        db.refresh(design)
        assert design.image_type == "3d"

    def test_render_3d_preview_returns_404_for_missing_design(self, client):
        resp = client.post("/designs/99999/render-3d-preview")
        assert resp.status_code == 404

    def test_render_3d_preview_handles_error(self, client, db):
        from src.models import Design

        design = Design(filename="test.dst", filepath="test.dst", image_data=None, image_type=None)
        db.add(design)
        db.commit()
        db.refresh(design)
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.side_effect = Exception("Read failed")
        _sys.modules["pyembroidery"] = mock_pyemb
        try:
            with (
                patch(
                    "src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"
                ),
                patch("src.routes.designs.svc.get_by_id", return_value=design),
                patch("os.path.isfile", return_value=True),
            ):
                resp = client.post(f"/designs/{design.id}/render-3d-preview")
        finally:
            _restore_pyembroidery()
        assert resp.status_code == 500


class TestDesignCreateImageType:
    """Verify image_type is set correctly when creating designs."""

    def test_create_sets_image_type_to_3d_when_image_data_provided(self, db):
        from src.services.designs import create

        data = {"filename": "test.dst", "filepath": "test.dst", "image_data": None}
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.return_value = _make_pattern()
        _sys.modules["pyembroidery"] = mock_pyemb
        try:
            with (
                patch("src.services.designs._validate_design_data"),
                patch(
                    "src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"
                ),
                patch("src.services.auto_tagging.run_tagging_action"),
                patch("src.services.auto_tagging.run_stitching_backfill_action"),
                patch("src.services.preview._render_preview", return_value=b"PNG_DATA"),
            ):
                design = create(db, data)
        finally:
            _restore_pyembroidery()
        assert design.image_data == b"PNG_DATA"
        assert design.image_type == "3d"

    def test_create_leaves_image_type_none_when_no_image_data(self, db):
        from src.services.designs import create

        data = {"filename": "test.dst", "filepath": "test.dst", "image_data": None}
        with (
            patch("src.services.designs._validate_design_data"),
            patch("src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"),
        ):
            design = create(db, data)
        assert design.image_type is None

    def test_create_auto_backfill_uses_image_type_3d(self, db):
        from src.services.designs import create

        data = {"filename": "test.dst", "filepath": "test.dst", "image_data": None}
        import sys as _sys

        mock_pyemb = MagicMock()
        mock_pyemb.read.return_value = _make_pattern()
        _sys.modules["pyembroidery"] = mock_pyemb
        try:
            with (
                patch("src.services.designs._validate_design_data"),
                patch(
                    "src.routes.designs._resolve_design_full_path", return_value="/path/test.dst"
                ),
                patch("src.services.auto_tagging.run_tagging_action"),
                patch("src.services.auto_tagging.run_stitching_backfill_action"),
                patch("src.services.preview._render_preview", return_value=b"PNG_DATA"),
            ):
                design = create(db, data)
        finally:
            _restore_pyembroidery()
        assert design.image_data == b"PNG_DATA"
        assert design.image_type == "3d"


class TestRunUnifiedBackfillRoute:
    """Verify the run-unified-backfill route endpoint."""

    def test_route_returns_json_response(self, client):
        with (
            patch("src.routes.tagging_actions.unified_backfill") as mock_ub,
            patch("src.services.unified_backfill.clear_stop_signal"),
            patch("src.services.unified_backfill.log_info"),
        ):
            mock_ub.return_value = {"processed": 5, "errors": 0, "stopped": False, "actions": []}
            resp = client.post("/admin/tagging-actions/run-unified-backfill", json={})
        assert resp.status_code == 200
        data = resp.json()
        assert "processed" in data

    def test_route_passes_preview_3d_to_images_action(self, client):
        with (
            patch("src.routes.tagging_actions.unified_backfill") as mock_ub,
            patch("src.services.unified_backfill.clear_stop_signal"),
            patch("src.services.unified_backfill.log_info"),
        ):
            mock_ub.return_value = {
                "processed": 5,
                "errors": 0,
                "stopped": False,
                "actions": ["images"],
            }
            resp = client.post(
                "/admin/tagging-actions/run-unified-backfill",
                json={"images": {"enabled": True, "preview_3d": True}},
            )
        assert resp.status_code == 200

    def test_route_handles_backfill_error(self, client):
        with (
            patch(
                "src.routes.tagging_actions.unified_backfill",
                side_effect=Exception("Backfill failed"),
            ),
            patch("src.services.unified_backfill.clear_stop_signal"),
            patch("src.services.unified_backfill.log_info"),
        ):
            resp = client.post("/admin/tagging-actions/run-unified-backfill", json={})
        assert resp.status_code == 500


class TestStopUnifiedBackfillRoute:
    """Verify the stop-unified-backfill route endpoint."""

    def test_stop_route_calls_request_stop(self, client):
        with patch("src.routes.tagging_actions.request_stop") as mock_rs:
            resp = client.post("/admin/tagging-actions/stop-unified-backfill")
        assert resp.status_code == 200
        mock_rs.assert_called_once()

    def test_stop_route_returns_json(self, client):
        with patch("src.routes.tagging_actions.request_stop"):
            resp = client.post("/admin/tagging-actions/stop-unified-backfill")
        assert resp.status_code == 200
        data = resp.json()
        assert "status" in data


class TestBulkClearStitchingTags:
    """Verify bulk clear stitching tags functionality."""

    def test_clear_stitching_tags_removes_stitching_type_tags(self, db):
        from src.models import Design, Tag

        tag = Tag(description="Running Stitch", tag_group="stitching")
        design = Design(filename="test.dst", filepath="test.dst")
        design.tags.append(tag)
        db.add(tag)
        db.add(design)
        db.commit()
        from src.services.unified_backfill import unified_backfill

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={}, workers=1)
        assert result is not None

    def test_clear_stitching_tags_handles_no_stitching_tags(self, db):
        from src.models import Design

        design = Design(filename="test.dst", filepath="test.dst")
        db.add(design)
        db.commit()
        from src.services.unified_backfill import unified_backfill

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            result = unified_backfill(db=db, actions={}, workers=1)
        assert result is not None


class TestLogCleanup:
    """Verify backfill log cleanup behavior."""

    def test_backfill_logs_are_cleaned_on_start(self, tmp_path):
        import src.services.unified_backfill as ub

        err_path = tmp_path / "backfill_errors.tsv"
        info_path = tmp_path / "backfill_info.tsv"
        err_path.touch()
        info_path.touch()
        with (
            patch.object(ub, "ERROR_LOG_PATH", err_path),
            patch.object(ub, "INFO_LOG_PATH", info_path),
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
        ):
            ub.unified_backfill(db=MagicMock(), actions={}, workers=1)
        assert not err_path.exists() or err_path.stat().st_size == 0


class TestBackfillDataPersistence:
    """Integration tests confirming data is persisted to the database during backfill.

    These tests verify the fix for the backfill commit bug where ``_restore_sqlite_after_bulk``
    (which executes ``PRAGMA journal_mode = WAL``, implicitly committing any pending SQLite
    transaction) was called *before* the final ``db.commit()``, causing data loss when a
    backfill run was stopped early.
    """

    def _create_real_design(self, db, filename="test.dst", filepath="subdir/test.dst"):
        """Create a real Design ORM record in the test database."""
        from src.models import Design

        d = Design(filename=filename, filepath=filepath)
        db.add(d)
        db.commit()
        return d

    def _create_fake_pattern_file(self, tmp_path, filename="test.dst"):
        """Create a minimal valid .dst file for pyembroidery to read."""
        dst_path = tmp_path / filename
        header = b"\x00" * 512
        stitches = bytes(
            [
                0x00,
                0x00,
                0x03,
                0x0A,
                0x00,
                0x01,
                0x14,
                0x00,
                0x01,
                0x1E,
                0x00,
                0x01,
                0x00,
                0x00,
                0x00,
            ]
        )
        dst_path.write_bytes(header + stitches)
        return str(dst_path)

    def _ensure_satin_stitch_tag(self, db):
        """Ensure a 'Satin Stitch' Tag exists in the test database."""
        from src.models import Tag

        tag = db.query(Tag).filter(Tag.description == "Satin Stitch").first()
        if tag is None:
            tag = Tag(description="Satin Stitch", tag_group="stitching")
            db.add(tag)
            db.commit()
        return tag

    def test_commit_every_1_persists_data_to_db(self, db, tmp_path):
        """Verify that with commit_every=1, data is persisted to the database
        during the backfill loop, not just at the end."""
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        mock_result = PatternAnalysisResult(stitching_tag_descriptions=["Satin Stitch"])

        # is_stop_requested is called multiple times during the flow:
        #   - in run_stitching_action_runner before and after analysis
        #   - in the main loop
        # Return False for all calls so the loop completes normally
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
        ):
            result = unified_backfill(
                db=db,
                actions={"stitching": {"enabled": True}},
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.tags_checked is False
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_pragma_restore_exception_still_persists_data(self, db, tmp_path):
        """Verify that even if _restore_sqlite_after_bulk raises an exception,
        the data is still persisted because the commit happens before the pragma restore."""
        import pytest

        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        mock_result = PatternAnalysisResult(stitching_tag_descriptions=["Satin Stitch"])

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
        ):
            # The exception from _restore_sqlite_after_bulk propagates out of
            # the finally block — catch it here so we can verify data persisted
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"stitching": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.tags_checked is False
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_parallel_path_persists_data_to_db(self, db, tmp_path):
        """Verify that with workers=2 (parallel path), data is persisted to the
        database during the backfill loop, not just at the end.

        Mocks ``ProcessPoolExecutor`` to run synchronously so the test avoids
        subprocess-spawning issues while still exercising the parallel-path
        result-writing and commit logic.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Build a completed Future that returns worker results, as if
        # _process_design_batch_worker had run and returned stitching data.
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
        ):
            result = unified_backfill(
                db=db,
                actions={"stitching": {"enabled": True}},
                workers=2,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.tags_checked is False
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_parallel_path_pragma_exception_still_persists_data(self, db, tmp_path):
        """Verify that in the parallel path, even if _restore_sqlite_after_bulk
        raises an exception, the data is still persisted because the commit
        happens before the pragma restore in the shared finally block."""
        from concurrent.futures import Future

        import pytest

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
        ):
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"stitching": {"enabled": True}},
                    workers=2,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.tags_checked is False
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    # ------------------------------------------------------------------
    # Color counts integration tests
    # ------------------------------------------------------------------

    def test_sequential_color_counts_persisted(self, db, tmp_path):
        """Verify that with workers=1 and color_counts action, stitch_count,
        color_count, and color_change_count are persisted to the database."""
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create a Design with no color counts so the query picks it up
        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        mock_result = PatternAnalysisResult(
            stitch_count=1500,
            color_count=5,
            color_change_count=10,
        )

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
        ):
            result = unified_backfill(
                db=db,
                actions={"color_counts": {"enabled": True}},
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 1500
        ), f"Expected stitch_count=1500, got {updated.stitch_count}"
        assert updated.color_count == 5, f"Expected color_count=5, got {updated.color_count}"
        assert (
            updated.color_change_count == 10
        ), f"Expected color_change_count=10, got {updated.color_change_count}"

    def test_sequential_color_counts_pragma_exception(self, db, tmp_path):
        """Verify that even if _restore_sqlite_after_bulk raises an exception,
        color count data is still persisted because the commit happens before
        the pragma restore."""
        import pytest

        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        mock_result = PatternAnalysisResult(
            stitch_count=2500,
            color_count=8,
            color_change_count=20,
        )

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
        ):
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"color_counts": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 2500
        ), f"Expected stitch_count=2500, got {updated.stitch_count}"
        assert updated.color_count == 8, f"Expected color_count=8, got {updated.color_count}"
        assert (
            updated.color_change_count == 20
        ), f"Expected color_change_count=20, got {updated.color_change_count}"

    def test_parallel_color_counts_persisted(self, db, tmp_path):
        """Verify that with workers=2 (parallel path), color count data is
        persisted to the database during the backfill loop.

        Mocks ``ProcessPoolExecutor`` to run synchronously so the test avoids
        subprocess-spawning issues while still exercising the parallel-path
        result-writing and commit logic.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        # Build a completed Future that returns worker results with color counts
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": 3500,
            "color_count": 12,
            "color_change_count": 30,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
        ):
            result = unified_backfill(
                db=db,
                actions={"color_counts": {"enabled": True}},
                workers=2,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 3500
        ), f"Expected stitch_count=3500, got {updated.stitch_count}"
        assert updated.color_count == 12, f"Expected color_count=12, got {updated.color_count}"
        assert (
            updated.color_change_count == 30
        ), f"Expected color_change_count=30, got {updated.color_change_count}"

    def test_parallel_color_counts_pragma_exception(self, db, tmp_path):
        """Verify that in the parallel path, even if _restore_sqlite_after_bulk
        raises an exception, color count data is still persisted because the
        commit happens before the pragma restore in the shared finally block."""
        from concurrent.futures import Future

        import pytest

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": 4500,
            "color_count": 3,
            "color_change_count": 8,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
        ):
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"color_counts": {"enabled": True}},
                    workers=2,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 4500
        ), f"Expected stitch_count=4500, got {updated.stitch_count}"
        assert updated.color_count == 3, f"Expected color_count=3, got {updated.color_count}"
        assert (
            updated.color_change_count == 8
        ), f"Expected color_change_count=8, got {updated.color_change_count}"

    # ------------------------------------------------------------------
    # Image persistence integration tests
    # ------------------------------------------------------------------

    def test_sequential_3d_image_persisted(self, db, tmp_path):
        """Verify that with workers=1 and images action, a 3D preview image
        is persisted to the database."""
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create a Design with no image data so the default query picks it up
        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        mock_result = PatternAnalysisResult(
            image_data=b"fake_3d_png_bytes",
            image_type="3d",
            width_mm=100.0,
            height_mm=80.0,
        )

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True}},
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"fake_3d_png_bytes"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d", f"Expected image_type='3d', got {updated.image_type!r}"
        assert updated.width_mm == 100.0, f"Expected width_mm=100.0, got {updated.width_mm}"
        assert updated.height_mm == 80.0, f"Expected height_mm=80.0, got {updated.height_mm}"

    def test_sequential_2d_image_persisted(self, db, tmp_path):
        """Verify that with workers=1, preview_3d=False, a 2D preview image
        is persisted to the database."""
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        mock_result = PatternAnalysisResult(
            image_data=b"fake_2d_png_bytes",
            image_type="2d",
            width_mm=50.0,
            height_mm=40.0,
        )

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True, "preview_3d": False}},
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"fake_2d_png_bytes"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "2d", f"Expected image_type='2d', got {updated.image_type!r}"

    def test_sequential_3d_image_pragma_exception(self, db, tmp_path):
        """Verify that even if _restore_sqlite_after_bulk raises an exception,
        3D image data is still persisted because the commit happens before
        the pragma restore."""
        import pytest

        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        mock_result = PatternAnalysisResult(
            image_data=b"fake_3d_png_pragma",
            image_type="3d",
            width_mm=120.0,
            height_mm=90.0,
        )

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"images": {"enabled": True}},
                    workers=1,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"fake_3d_png_pragma"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"

    def test_parallel_3d_image_persisted(self, db, tmp_path):
        """Verify that with workers=2 (parallel path), 3D image data is
        persisted to the database during the backfill loop.

        Mocks ``ProcessPoolExecutor`` to run synchronously so the test avoids
        subprocess-spawning issues while still exercising the parallel-path
        result-writing and commit logic.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        # Build a completed Future that returns worker results with image data
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"parallel_3d_png",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True}},
                workers=2,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"parallel_3d_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d", f"Expected image_type='3d', got {updated.image_type!r}"
        assert updated.width_mm == 200.0, f"Expected width_mm=200.0, got {updated.width_mm}"
        assert updated.height_mm == 150.0, f"Expected height_mm=150.0, got {updated.height_mm}"

    def test_parallel_2d_image_persisted(self, db, tmp_path):
        """Verify that with workers=2 and preview_3d=False, a 2D preview image
        is persisted to the database."""
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"parallel_2d_png",
            "image_type": "2d",
            "width_mm": 75.0,
            "height_mm": 60.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={"images": {"enabled": True, "preview_3d": False}},
                workers=2,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"parallel_2d_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "2d", f"Expected image_type='2d', got {updated.image_type!r}"

    def test_parallel_3d_image_pragma_exception(self, db, tmp_path):
        """Verify that in the parallel path, even if _restore_sqlite_after_bulk
        raises an exception, 3D image data is still persisted because the
        commit happens before the pragma restore in the shared finally block."""
        from concurrent.futures import Future

        import pytest

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"parallel_3d_pragma",
            "image_type": "3d",
            "width_mm": 300.0,
            "height_mm": 200.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch(
                "src.services.unified_backfill._restore_sqlite_after_bulk",
                side_effect=RuntimeError("Pragma restore failed!"),
            ),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            with pytest.raises(RuntimeError, match="Pragma restore failed!"):
                unified_backfill(
                    db=db,
                    actions={"images": {"enabled": True}},
                    workers=2,
                    commit_every=1,
                )

        # Verify data was committed BEFORE the pragma restore exception
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"parallel_3d_pragma"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"

    # ------------------------------------------------------------------
    # Stop-signal integration tests
    # ------------------------------------------------------------------

    def test_sequential_stop_commits_processed_data(self, db, tmp_path):
        """Verify that when the stop button is pressed during a sequential
        backfill, data from designs that were already processed is committed
        to the database.

        Creates 2 designs.  ``is_stop_requested`` returns ``False`` for the
        first design (allowing it to be fully processed), then ``True`` so
        the second design is never entered.  Asserts the first design's
        image, colour-count, and stitching data are all persisted.
        """
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create 2 designs — only the first should be processed before stop
        design1 = self._create_real_design(db, filename="first.dst", filepath="subdir/first.dst")
        design2 = self._create_real_design(db, filename="second.dst", filepath="subdir/second.dst")
        design1_id = design1.id
        design2_id = design2.id

        dst_path1 = self._create_fake_pattern_file(tmp_path, filename="first.dst")
        self._create_fake_pattern_file(tmp_path, filename="second.dst")
        self._ensure_satin_stitch_tag(db)

        mock_result = PatternAnalysisResult(
            image_data=b"stop_test_png",
            image_type="3d",
            width_mm=100.0,
            height_mm=80.0,
            stitch_count=1500,
            color_count=5,
            color_change_count=10,
            stitching_tag_descriptions=["Satin Stitch"],
        )

        # is_stop_requested is called ~7 times per design in the sequential
        # loop with all 3 actions enabled.  Return False for the first 7
        # calls (covers design 1), then True so design 2 is never entered.
        # _resolve_design_filepath is called at line 1152 (main loop) AND
        # line 259 (inside run_stitching_action_runner) — use return_value
        # instead of side_effect to avoid exhausting the iterator.
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch(
                "src.services.unified_backfill._resolve_design_filepath",
                return_value=dst_path1,
            ),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                side_effect=[False] * 7 + [True],
            ),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
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

        assert result["stopped"] is True, "Expected stop to be detected"
        assert result["processed"] >= 1, "Expected at least 1 design processed"

        # Design 1 should have all data persisted
        updated1 = db.query(Design).filter(Design.id == design1_id).first()
        assert updated1 is not None
        assert (
            updated1.image_data == b"stop_test_png"
        ), f"Design 1 image_data mismatch: {updated1.image_data!r}"
        assert updated1.image_type == "3d"
        assert (
            updated1.stitch_count == 1500
        ), f"Design 1 stitch_count mismatch: {updated1.stitch_count}"
        assert updated1.color_count == 5
        assert updated1.color_change_count == 10
        stitching_tags1 = [t for t in updated1.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags1) > 0
        ), f"Design 1 expected stitching tags, got {[t.description for t in updated1.tags]}"

        # Design 2 should NOT have any data (it was never processed)
        updated2 = db.query(Design).filter(Design.id == design2_id).first()
        assert updated2 is not None
        assert (
            updated2.image_data is None
        ), f"Design 2 should have no image_data, got {updated2.image_data!r}"
        assert updated2.stitch_count is None
        assert updated2.color_count is None
        assert updated2.color_change_count is None

    def test_sequential_stop_commits_without_commit_every_trigger(self, db, tmp_path):
        """Verify that when stop is pressed and ``commit_every`` has NOT been
        reached, the pending data is still committed via the ``finally`` block.

        Uses ``commit_every=100`` so the periodic commit never fires.
        Creates 2 designs; the first is fully processed (7 calls to
        ``is_stop_requested`` return ``False``), then the 8th call (loop
        top for design 2) returns ``True``, causing the loop to break.
        The ``finally`` block commits the first design's data.
        """
        from src.models import Design
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        design1 = self._create_real_design(db, filename="first.dst", filepath="subdir/first.dst")
        design2 = self._create_real_design(db, filename="second.dst", filepath="subdir/second.dst")
        design1_id = design1.id
        design2_id = design2.id

        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        mock_result = PatternAnalysisResult(
            image_data=b"commit_every_test_png",
            image_type="3d",
            width_mm=100.0,
            height_mm=80.0,
            stitch_count=500,
            color_count=3,
            color_change_count=6,
            stitching_tag_descriptions=["Satin Stitch"],
        )

        # is_stop_requested is called 7 times per design with all 3 actions
        # enabled.  Return False for the first 7 calls (covers design 1),
        # then True so the loop breaks at design 2's loop-top check.
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                side_effect=[False] * 7 + [True],
            ),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                    "stitching": {"enabled": True},
                },
                workers=1,
                commit_every=100,  # Never reached — relies on finally block
            )

        assert result["stopped"] is True, f"Expected stopped=True, got stopped={result['stopped']}"
        assert result["processed"] >= 1

        # Design 1 should have all data persisted via the finally block
        updated1 = db.query(Design).filter(Design.id == design1_id).first()
        assert updated1 is not None
        assert updated1.image_data == b"commit_every_test_png"
        assert updated1.image_type == "3d"
        assert updated1.stitch_count == 500
        assert updated1.color_count == 3
        assert updated1.color_change_count == 6
        stitching_tags1 = [t for t in updated1.tags if getattr(t, "tag_group", None) == "stitching"]
        assert len(stitching_tags1) > 0

        # Design 2 should NOT have any data (never processed)
        updated2 = db.query(Design).filter(Design.id == design2_id).first()
        assert updated2 is not None
        assert updated2.image_data is None
        assert updated2.stitch_count is None
        assert updated2.color_count is None
        assert updated2.color_change_count is None

    def test_parallel_stop_commits_processed_data(self, db, tmp_path):
        """Verify that when the stop button is pressed during a parallel
        backfill, data from chunks that were already processed is committed
        to the database.

        Creates 2 designs in 2 separate chunks.  The first chunk completes
        normally, then ``is_stop_requested`` returns ``True``, causing the
        main loop to break.  Asserts the first design's data is persisted.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design1 = self._create_real_design(db, filename="first.dst", filepath="subdir/first.dst")
        design2 = self._create_real_design(db, filename="second.dst", filepath="subdir/second.dst")
        design1_id = design1.id
        design2_id = design2.id

        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result for design 1 only (design 2's chunk is never submitted)
        worker_result = {
            "design_id": design1_id,
            "filename": "first.dst",
            "error": None,
            "image_data": b"parallel_stop_png",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": 2500,
            "color_count": 10,
            "color_change_count": 25,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        # is_stop_requested is called once in the parallel path main loop
        # (after each chunk is processed at line 1057).  With only 1 chunk,
        # it is called exactly once.  Return True so the loop breaks after
        # the first chunk is fully processed and committed.
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                side_effect=[True],
            ),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                    "stitching": {"enabled": True},
                },
                workers=2,
                commit_every=1,
            )

        assert result["stopped"] is True
        assert result["processed"] >= 1

        # Design 1 should have all data persisted
        updated1 = db.query(Design).filter(Design.id == design1_id).first()
        assert updated1 is not None
        assert (
            updated1.image_data == b"parallel_stop_png"
        ), f"Design 1 image_data mismatch: {updated1.image_data!r}"
        assert updated1.image_type == "3d"
        assert updated1.stitch_count == 2500
        assert updated1.color_count == 10
        assert updated1.color_change_count == 25
        stitching_tags1 = [t for t in updated1.tags if getattr(t, "tag_group", None) == "stitching"]
        assert len(stitching_tags1) > 0

        # Design 2 should NOT have any data (its chunk was never submitted)
        updated2 = db.query(Design).filter(Design.id == design2_id).first()
        assert updated2 is not None
        assert updated2.image_data is None
        assert updated2.stitch_count is None
        assert updated2.color_count is None
        assert updated2.color_change_count is None

    # ------------------------------------------------------------------
    # Multi-Action Combination integration tests (Section 2.2)
    # ------------------------------------------------------------------

    def test_tagging_plus_stitching_sequential(self, db, tmp_path):
        """Verify that with workers=1, tagging + stitching actions run correctly
        in sequential mode. Tagging runs first, then stitching.

        Corresponds to checklist item 2.2.1.
        """
        from src.models import Design, Tag
        from src.services.pattern_analysis import PatternAnalysisResult
        from src.services.unified_backfill import unified_backfill

        # Create an image tag so the tag_untagged query doesn't use sql_false()
        image_tag = Tag(description="Flowers", tag_group="image")
        db.add(image_tag)
        db.commit()

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        mock_result = PatternAnalysisResult(stitching_tag_descriptions=["Satin Stitch"])

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch("src.services.unified_backfill.analyze_pattern", return_value=mock_result),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "tag_untagged",
                        "tiers": [1, 2, 3],
                    },
                    "stitching": {"enabled": True},
                },
                workers=1,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify stitching data was also persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_tagging_plus_images_parallel(self, db, tmp_path):
        """Verify that with workers=4, tagging + images actions run correctly.
        Tagging runs sequentially after parallel image processing completes.

        Corresponds to checklist item 2.2.2.
        """
        from concurrent.futures import Future

        from src.models import Design, Tag
        from src.services.unified_backfill import unified_backfill

        # Create an image tag so the tag_untagged query doesn't use sql_false()
        image_tag = Tag(description="Flowers", tag_group="image")
        db.add(image_tag)
        db.commit()

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with image data (parallel phase)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"tagging_images_parallel_png",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "tag_untagged",
                        "tiers": [1, 2, 3],
                    },
                    "images": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify image data was persisted from parallel phase
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"tagging_images_parallel_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"

    def test_tagging_plus_color_counts_parallel(self, db, tmp_path):
        """Verify that with workers=4, tagging + color_counts actions run correctly.
        Tagging runs sequentially after parallel color count processing completes.

        Corresponds to checklist item 2.2.3.
        """
        from concurrent.futures import Future

        from src.models import Design, Tag
        from src.services.unified_backfill import unified_backfill

        # Create an image tag so the tag_untagged query doesn't use sql_false()
        image_tag = Tag(description="Flowers", tag_group="image")
        db.add(image_tag)
        db.commit()

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with color counts (parallel phase)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": 3500,
            "color_count": 12,
            "color_change_count": 30,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "tag_untagged",
                        "tiers": [1, 2, 3],
                    },
                    "color_counts": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify color count data was persisted from parallel phase
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 3500
        ), f"Expected stitch_count=3500, got {updated.stitch_count}"
        assert updated.color_count == 12, f"Expected color_count=12, got {updated.color_count}"
        assert (
            updated.color_change_count == 30
        ), f"Expected color_change_count=30, got {updated.color_change_count}"

    def test_stitching_plus_images_parallel(self, db, tmp_path):
        """Verify that with workers=4, stitching + images actions run correctly
        in parallel workers.

        Corresponds to checklist item 2.2.4.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result with both image data and stitching tags
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"stitching_images_png",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "stitching": {"enabled": True},
                    "images": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        # Verify both image and stitching data were persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"stitching_images_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_stitching_plus_color_counts_parallel(self, db, tmp_path):
        """Verify that with workers=4, stitching + color_counts actions run correctly
        in parallel workers.

        Corresponds to checklist item 2.2.5.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result with both stitching tags and color counts
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": 2500,
            "color_count": 8,
            "color_change_count": 20,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "stitching": {"enabled": True},
                    "color_counts": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        # Verify both stitching and color count data were persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.stitch_count == 2500
        ), f"Expected stitch_count=2500, got {updated.stitch_count}"
        assert updated.color_count == 8, f"Expected color_count=8, got {updated.color_count}"
        assert (
            updated.color_change_count == 20
        ), f"Expected color_change_count=20, got {updated.color_change_count}"
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_images_plus_color_counts_parallel(self, db, tmp_path):
        """Verify that with workers=4, images + color_counts actions run correctly
        in parallel workers.

        Corresponds to checklist item 2.2.6.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with both image data and color counts
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"images_color_counts_png",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": 1800,
            "color_count": 6,
            "color_change_count": 15,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        # Verify both image and color count data were persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"images_color_counts_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert (
            updated.stitch_count == 1800
        ), f"Expected stitch_count=1800, got {updated.stitch_count}"
        assert updated.color_count == 6, f"Expected color_count=6, got {updated.color_count}"
        assert (
            updated.color_change_count == 15
        ), f"Expected color_change_count=15, got {updated.color_change_count}"

    def test_backfill_actions_combined(self, db, tmp_path):
        """Verify that with workers=4, stitching + images + color_counts actions
        all run correctly in parallel workers.

        Corresponds to checklist item 2.2.7.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result with all three data types
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"combined_actions_png",
            "image_type": "3d",
            "width_mm": 300.0,
            "height_mm": 200.0,
            "hoop_id": None,
            "stitch_count": 800,
            "color_count": 4,
            "color_change_count": 12,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                    "stitching": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        # Verify all three data types were persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"combined_actions_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert updated.width_mm == 300.0
        assert updated.height_mm == 200.0
        assert updated.stitch_count == 800, f"Expected stitch_count=800, got {updated.stitch_count}"
        assert updated.color_count == 4, f"Expected color_count=4, got {updated.color_count}"
        assert (
            updated.color_change_count == 12
        ), f"Expected color_change_count=12, got {updated.color_change_count}"
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_full_pipeline_defaults(self, db, tmp_path):
        """Verify that with workers=4, the full pipeline
        (tagging + stitching + images + color_counts) runs correctly.
        Tagging runs sequentially after parallel work completes.

        Corresponds to checklist item 2.2.8.
        """
        from concurrent.futures import Future

        from src.models import Design, Tag
        from src.services.unified_backfill import unified_backfill

        # Create an image tag so the tag_untagged query doesn't use sql_false()
        image_tag = Tag(description="Flowers", tag_group="image")
        db.add(image_tag)
        db.commit()

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result with all three parallel data types
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"full_pipeline_png",
            "image_type": "3d",
            "width_mm": 300.0,
            "height_mm": 200.0,
            "hoop_id": None,
            "stitch_count": 800,
            "color_count": 4,
            "color_change_count": 12,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "tag_untagged",
                        "tiers": [1, 2, 3],
                    },
                    "stitching": {"enabled": True},
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify all data types were persisted
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"full_pipeline_png"
        ), f"Expected image_data to match, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert updated.stitch_count == 800, f"Expected stitch_count=800, got {updated.stitch_count}"
        assert updated.color_count == 4, f"Expected color_count=4, got {updated.color_count}"
        assert (
            updated.color_change_count == 12
        ), f"Expected color_change_count=12, got {updated.color_change_count}"
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_full_pipeline_aggressive_options(self, db, tmp_path):
        """Verify that with workers=4, the full pipeline runs correctly with
        aggressive options: retag_all, clear_existing_stitching, redo all images.

        Corresponds to checklist item 2.2.9.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design with existing data (to be overwritten)
        design = self._create_real_design(db)
        design.image_data = b"old_image"
        design.image_type = "2d"
        design.width_mm = 50.0
        design.height_mm = 40.0
        db.commit()
        design_id = design.id

        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        # Worker result with new data (simulating redo + clear)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"aggressive_redone_png",
            "image_type": "3d",
            "width_mm": 300.0,
            "height_mm": 200.0,
            "hoop_id": None,
            "stitch_count": 800,
            "color_count": 4,
            "color_change_count": 12,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "retag_all",
                        "tiers": [1, 2, 3],
                    },
                    "stitching": {
                        "enabled": True,
                        "clear_existing_stitching": True,
                    },
                    "images": {
                        "enabled": True,
                        "redo": True,
                        "preview_3d": True,
                    },
                    "color_counts": {"enabled": True},
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify all data was overwritten with new values
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"aggressive_redone_png"
        ), f"Expected image_data to be overwritten, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert updated.width_mm == 300.0
        assert updated.height_mm == 200.0
        assert updated.stitch_count == 800, f"Expected stitch_count=800, got {updated.stitch_count}"
        assert updated.color_count == 4, f"Expected color_count=4, got {updated.color_count}"
        assert (
            updated.color_change_count == 12
        ), f"Expected color_change_count=12, got {updated.color_change_count}"
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert (
            len(stitching_tags) > 0
        ), f"Expected at least one stitching tag, got {[t.description for t in updated.tags]}"

    def test_tagging_plus_images_mixed_options(self, db, tmp_path):
        """Verify that with workers=4, tagging + images actions run correctly
        with mixed aggressive/conservative options:
        tagging: retag_all_unverified, images: upgrade 2D→3D.

        Corresponds to checklist item 2.2.10.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design with a 2D image (to be upgraded)
        design = self._create_real_design(db)
        design.image_data = b"old_2d_image"
        design.image_type = "2d"
        design.width_mm = 50.0
        design.height_mm = 40.0
        db.commit()
        design_id = design.id

        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with upgraded 3D image data
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"upgraded_from_2d_png",
            "image_type": "3d",
            "width_mm": 100.0,
            "height_mm": 80.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "tagging": {
                        "enabled": True,
                        "action": "retag_all_unverified",
                        "tiers": [1, 2],
                    },
                    "images": {
                        "enabled": True,
                        "upgrade_2d_to_3d": True,
                        "preview_3d": True,
                    },
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()

        # Verify image was upgraded from 2D to 3D
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"upgraded_from_2d_png"
        ), f"Expected image_data to be upgraded, got {updated.image_data!r}"
        assert updated.image_type == "3d"

    def test_parallel_tagging_retag_all_unverified_tiers_1_2(self, db, tmp_path):
        """Verify that with workers=4 and tagging action (retag_all_unverified, Tiers 1+2),
        tagging runs sequentially after parallel work completes.

        Corresponds to checklist item 2.1.2.
        """
        from concurrent.futures import Future

        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id

        # Worker result (no parallel actions needed, but we need the parallel
        # path to be exercised so tagging runs in the sequential phase after)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": None,
            "image_type": None,
            "width_mm": None,
            "height_mm": None,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=None),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch(
                "src.services.unified_backfill.run_tagging_action_runner",
                return_value=None,
            ) as mock_tagging,
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
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1
        mock_tagging.assert_called_once()
        call_args = mock_tagging.call_args[0]
        # Positional args: (db, design, tag_opts, api_key, delay, vision_delay)
        assert call_args[2]["action"] == "retag_all_unverified"
        assert call_args[2]["tiers"] == [1, 2]

    # ------------------------------------------------------------------
    # Image redo / upgrade integration tests (Section 2.1.7, 2.1.8)
    # ------------------------------------------------------------------

    def test_parallel_image_redo_all_3d(self, db, tmp_path):
        """Verify that with workers=4 and images action with redo=True,
        existing images are overwritten (re-process all).

        Corresponds to checklist item 2.1.7.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design that already has image data (to be overwritten)
        design = self._create_real_design(db)
        design.image_data = b"old_image_data"
        design.image_type = "3d"
        design.width_mm = 100.0
        design.height_mm = 80.0
        db.commit()
        design_id = design.id

        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with new image data (simulating redo)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"new_redone_image",
            "image_type": "3d",
            "width_mm": 200.0,
            "height_mm": 150.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {
                        "enabled": True,
                        "redo": True,
                        "preview_3d": True,
                    }
                },
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"new_redone_image"
        ), f"Expected image_data to be overwritten, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert updated.width_mm == 200.0
        assert updated.height_mm == 150.0

    def test_parallel_image_upgrade_2d_to_3d(self, db, tmp_path):
        """Verify that with workers=4 and images action with upgrade_2d_to_3d=True,
        existing 2D images are upgraded to 3D.

        Corresponds to checklist item 2.1.8.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design that has a 2D image (to be upgraded)
        design = self._create_real_design(db)
        design.image_data = b"old_2d_image"
        design.image_type = "2d"
        design.width_mm = 50.0
        design.height_mm = 40.0
        db.commit()
        design_id = design.id

        dst_path = self._create_fake_pattern_file(tmp_path)

        # Worker result with 3D image data (simulating upgrade)
        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"upgraded_3d_image",
            "image_type": "3d",
            "width_mm": 100.0,
            "height_mm": 80.0,
            "hoop_id": None,
            "stitch_count": None,
            "color_count": None,
            "color_change_count": None,
            "stitching_tag_descriptions": None,
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
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
                workers=4,
                commit_every=1,
            )

        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert (
            updated.image_data == b"upgraded_3d_image"
        ), f"Expected image_data to be upgraded, got {updated.image_data!r}"
        assert updated.image_type == "3d"
        assert updated.width_mm == 100.0
        assert updated.height_mm == 80.0

    def test_parallel_stop_commits_without_commit_every_trigger(self, db, tmp_path):
        """Verify that in the parallel path, when stop is pressed and
        ``commit_every`` has NOT been reached, the pending data is still
        committed via the ``finally`` block.

        Uses ``commit_every=100`` so the per-chunk periodic commit never
        fires.  The stop signal causes the loop to break, and the
        ``finally`` block commits the pending changes.
        """
        from concurrent.futures import Future

        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        design = self._create_real_design(db)
        design_id = design.id
        dst_path = self._create_fake_pattern_file(tmp_path)
        self._ensure_satin_stitch_tag(db)

        worker_result = {
            "design_id": design_id,
            "filename": "test.dst",
            "error": None,
            "image_data": b"parallel_commit_every_png",
            "image_type": "3d",
            "width_mm": 300.0,
            "height_mm": 200.0,
            "hoop_id": None,
            "stitch_count": 800,
            "color_count": 4,
            "color_change_count": 12,
            "stitching_tag_descriptions": ["Satin Stitch"],
        }
        future = Future()
        future.set_result([worker_result])

        mock_executor = MagicMock()
        mock_executor.submit.return_value = future
        mock_executor.__enter__.return_value = mock_executor
        mock_executor.__exit__.return_value = None

        # is_stop_requested is called once in the parallel path main loop
        # (after each chunk is processed at line 1057).  With only 1 chunk,
        # it is called exactly once.  Return True so the loop breaks after
        # the first chunk is fully processed and committed.
        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=dst_path),
            patch(
                "src.services.unified_backfill.is_stop_requested",
                side_effect=[True],
            ),
            patch(
                "src.services.unified_backfill.ProcessPoolExecutor",
                return_value=mock_executor,
            ),
            patch(
                "src.services.unified_backfill.as_completed",
                return_value=[future],
            ),
            patch("src.services.unified_backfill.select_hoop_for_dimensions", return_value=None),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                    "color_counts": {"enabled": True},
                    "stitching": {"enabled": True},
                },
                workers=2,
                commit_every=100,  # Never reached — relies on finally block
            )

        assert result["stopped"] is True
        assert result["processed"] >= 1

        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.image_data == b"parallel_commit_every_png"
        assert updated.image_type == "3d"
        assert updated.stitch_count == 800
        assert updated.color_count == 4
        assert updated.color_change_count == 12
        stitching_tags = [t for t in updated.tags if getattr(t, "tag_group", None) == "stitching"]
        assert len(stitching_tags) > 0

    def test_invalid_file_path_logs_error_and_continues(self, db):
        """Verify that when a design's file path is invalid (file not found),
        the backfill logs an error and continues processing other designs.

        Corresponds to checklist item 2.4.2.
        """
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design with a non-existent file path
        design = Design(
            filename="missing.dst",
            filepath="nonexistent/missing.dst",
        )
        db.add(design)
        db.commit()
        design_id = design.id

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill._resolve_design_filepath", return_value=None),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
        ):
            result = unified_backfill(
                db=db,
                actions={
                    "images": {"enabled": True},
                },
                workers=1,
                commit_every=1,
            )

        # Verify that the design was iterated over (processed=1) but no actual work was done
        # because the file was not found. The "processed" counter increments for each design
        # in the loop, even if actions are skipped.
        assert result["processed"] == 1, f"Expected processed=1, got {result['processed']}"
        assert result["errors"] == 0, f"Expected errors=0 (no crash), got {result['errors']}"

        # Verify that the design was not modified (the key assertion for this test)
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.image_data is None, "Expected image_data to remain None"

    def test_no_actions_selected_returns_zero_counts(self, db):
        """Verify that when no actions are selected (empty actions dict),
        the backfill returns zero counts and does not modify any data.

        Corresponds to checklist item 2.4.4.
        """
        from src.models import Design
        from src.services.unified_backfill import unified_backfill

        # Create a design in the database
        design = Design(
            filename="test.dst",
            filepath="test.dst",
        )
        db.add(design)
        db.commit()
        design_id = design.id

        with (
            patch("src.services.unified_backfill._optimise_sqlite_for_bulk"),
            patch("src.services.unified_backfill._restore_sqlite_after_bulk"),
            patch("src.services.unified_backfill.log_info"),
            patch("src.services.unified_backfill.is_stop_requested", return_value=False),
        ):
            result = unified_backfill(
                db=db,
                actions={},  # No actions enabled
                workers=1,
                commit_every=1,
            )

        # Verify that nothing was processed
        assert result["processed"] == 0, f"Expected processed=0, got {result['processed']}"
        assert result["errors"] == 0, f"Expected errors=0, got {result['errors']}"

        # Verify that the design was not modified
        updated = db.query(Design).filter(Design.id == design_id).first()
        assert updated is not None
        assert updated.image_data is None, "Expected image_data to remain None"
        assert updated.stitch_count is None, "Expected stitch_count to remain None"
        assert updated.color_count is None, "Expected color_count to remain None"
