"""Targeted coverage tests for folder picker and tagging helpers."""

import sys
from types import ModuleType, SimpleNamespace

import pytest


class DummyDb:
    def __init__(self):
        self.commit_count = 0

    def commit(self):
        self.commit_count += 1


class TestFolderPickerHelpers:
    def test_display_path_normalizes_and_handles_blank(self):
        from src.services import folder_picker as fp

        assert fp.display_path("") == ""
        assert fp.display_path("C:/Embroidery/Designs") == r"C:\Embroidery\Designs"

    def test_resolve_picker_initial_dir_prefers_existing_parent(self, tmp_path):
        from src.services import folder_picker as fp

        child = tmp_path / "chosen"
        child.mkdir()

        assert fp.resolve_picker_initial_dir(str(child), prefer_parent=True) == str(tmp_path)

    def test_resolve_picker_initial_dir_uses_existing_parent_for_missing_path(self, tmp_path):
        from src.services import folder_picker as fp

        existing_parent = tmp_path / "existing"
        existing_parent.mkdir()
        missing_child = existing_parent / "missing"

        assert fp.resolve_picker_initial_dir(str(missing_child)) == str(existing_parent)

    def test_resolve_picker_initial_dir_falls_back_to_c_drive_when_home_missing(self, monkeypatch):
        from src.services import folder_picker as fp

        monkeypatch.setattr(fp.os.path, "expanduser", lambda _value: "/no-home")
        monkeypatch.setattr(fp.os.path, "isdir", lambda _value: False)

        assert fp.resolve_picker_initial_dir("relative/path") == "C:\\"

    def test_pick_folder_returns_first_selected_path(self, monkeypatch):
        from src.services import folder_picker as fp

        captured = {}

        def fake_pick_folders(*, start_dir="", title="Select folder", allow_multiple=True):
            captured["start_dir"] = start_dir
            captured["title"] = title
            captured["allow_multiple"] = allow_multiple
            return [r"C:\one", r"C:\two"]

        monkeypatch.setattr(fp, "pick_folders", fake_pick_folders)

        assert fp.pick_folder(start_dir="C:/start", title="Pick one") == r"C:\one"
        assert captured == {
            "start_dir": "C:/start",
            "title": "Pick one",
            "allow_multiple": False,
        }

    def test_pick_folder_returns_blank_when_nothing_selected(self, monkeypatch):
        from src.services import folder_picker as fp

        monkeypatch.setattr(fp, "pick_folders", lambda **_kwargs: [])

        assert fp.pick_folder() == ""

    def test_pick_folders_raises_when_native_dialogs_disabled(self, monkeypatch):
        from src.services import folder_picker as fp

        monkeypatch.setattr(fp, "native_dialogs_disabled", lambda: True)

        with pytest.raises(fp.FolderPickerUnavailableError, match="disabled"):
            fp.pick_folders()

    def test_pick_folders_uses_native_windows_picker_when_available(self, monkeypatch, tmp_path):
        from src.services import folder_picker as fp

        captured = {}

        def fake_windows_picker(initial_dir, title, allow_multiple):
            captured["initial_dir"] = initial_dir
            captured["title"] = title
            captured["allow_multiple"] = allow_multiple
            return [r"C:\picked"]

        monkeypatch.setattr(fp, "native_dialogs_disabled", lambda: False)
        monkeypatch.setattr(fp.os, "name", "nt", raising=False)
        monkeypatch.setattr(fp, "_pick_folders_windows", fake_windows_picker)

        result = fp.pick_folders(
            start_dir=str(tmp_path), title="Choose folders", allow_multiple=False
        )

        assert result == [r"C:\picked"]
        assert captured == {
            "initial_dir": str(tmp_path),
            "title": "Choose folders",
            "allow_multiple": False,
        }

    def test_pick_folders_falls_back_to_tkinter_when_windows_picker_fails(
        self, monkeypatch, tmp_path
    ):
        from src.services import folder_picker as fp

        events = []

        class FakeRoot:
            def withdraw(self):
                events.append("withdraw")

            def wm_attributes(self, *args):
                events.append(("wm_attributes", args))

            def destroy(self):
                events.append("destroy")

        tk_module = ModuleType("tkinter")
        tk_module.Tk = FakeRoot
        tk_module.filedialog = SimpleNamespace(askdirectory=lambda **kwargs: "C:/chosen/path")

        monkeypatch.setattr(fp, "native_dialogs_disabled", lambda: False)
        monkeypatch.setattr(fp.os, "name", "nt", raising=False)
        monkeypatch.setattr(
            fp,
            "_pick_folders_windows",
            lambda *_args, **_kwargs: (_ for _ in ()).throw(RuntimeError("boom")),
        )
        monkeypatch.setitem(sys.modules, "tkinter", tk_module)

        result = fp.pick_folders(start_dir=str(tmp_path), title="Choose folders")

        assert result == [r"C:\chosen\path"]
        assert "withdraw" in events
        assert "destroy" in events

    def test_pick_folders_raises_when_tkinter_is_unavailable(self, monkeypatch):
        from src.services import folder_picker as fp

        tk_module = ModuleType("tkinter")

        def boom():
            raise RuntimeError("no display")

        tk_module.Tk = boom
        tk_module.filedialog = SimpleNamespace(askdirectory=lambda **kwargs: "")

        monkeypatch.setattr(fp, "native_dialogs_disabled", lambda: False)
        monkeypatch.setattr(fp.os, "name", "posix", raising=False)
        monkeypatch.setitem(sys.modules, "tkinter", tk_module)

        with pytest.raises(fp.FolderPickerUnavailableError, match="no display"):
            fp.pick_folders()


class TestFolderPickerWindowsHelpers:
    def test_check_hresult_handles_success_cancel_and_failure(self):
        from src.services import folder_picker as fp

        assert fp._check_hresult(0) is None

        with pytest.raises(fp._DialogCancelled):
            fp._check_hresult(fp.ERROR_CANCELLED, cancel_allowed=True)

        with pytest.raises(OSError, match="HRESULT"):
            fp._check_hresult(-1)

    def test_release_calls_release_and_swallows_exceptions(self):
        from src.services import folder_picker as fp

        released = []
        good_ptr = SimpleNamespace(
            contents=SimpleNamespace(
                lpVtbl=SimpleNamespace(
                    contents=SimpleNamespace(Release=lambda ptr: released.append(ptr) or 1)
                )
            )
        )
        bad_ptr = SimpleNamespace(
            contents=SimpleNamespace(
                lpVtbl=SimpleNamespace(
                    contents=SimpleNamespace(
                        Release=lambda _ptr: (_ for _ in ()).throw(RuntimeError("fail"))
                    )
                )
            )
        )

        fp._release(None)
        fp._release(good_ptr)
        fp._release(bad_ptr)

        assert released == [good_ptr]

    def test_shell_item_path_returns_display_path_and_frees_memory(self, monkeypatch):
        from src.services import folder_picker as fp

        freed = []

        def fake_get_display_name(_item, _sigdn, output):
            output._obj.value = "C:/Embroidery/Designs"
            return 0

        item = SimpleNamespace(
            contents=SimpleNamespace(
                lpVtbl=SimpleNamespace(
                    contents=SimpleNamespace(GetDisplayName=fake_get_display_name)
                )
            )
        )

        monkeypatch.setattr(
            fp, "ole32", SimpleNamespace(CoTaskMemFree=lambda ptr: freed.append(ptr))
        )
        monkeypatch.setattr(fp, "cast", lambda value, _target: ("cast", value))

        result = fp._shell_item_path(item)

        assert result == r"C:\Embroidery\Designs"
        assert freed and freed[0][0] == "cast"


class TestTaggingHelpers:
    def test_unique_tags_from_descriptions_ignores_missing_and_duplicates(self):
        from src.services import tagging

        flower = SimpleNamespace(id=1, description="Flower")
        leaf = SimpleNamespace(id=2, description="Leaf")

        result = tagging._unique_tags_from_descriptions(
            ["Flower", "Flower", "Missing", "Leaf"],
            {"Flower": flower, "Leaf": leaf},
        )

        assert result == [flower, leaf]

    def test_apply_tier2_tags_returns_early_when_everything_is_already_tagged(self, monkeypatch):
        from src.services import auto_tagging, tagging

        db = DummyDb()
        already_tagged = SimpleNamespace(filename="rose.jef", tags=["existing"], tagging_tier=None)

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier2_batch",
            lambda *_args, **_kwargs: pytest.fail("Tier 2 AI should not be called"),
        )

        tagging._apply_tier2_tags(db, [already_tagged], {}, [], "api-key")

        assert db.commit_count == 0
        assert already_tagged.tagging_tier is None

    def test_apply_tier2_tags_assigns_unique_matches_and_commits(self, monkeypatch):
        from src.services import auto_tagging, tagging

        db = DummyDb()
        flower = SimpleNamespace(id=1, description="Flower")
        target = SimpleNamespace(filename="Mystery.JEF", tags=[], tagging_tier=None)
        pretagged = SimpleNamespace(filename="known.jef", tags=["manual"], tagging_tier=None)

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier2_batch",
            lambda filenames, valid_descriptions, api_key: {
                "mystery": ["Flower", "Flower", "Missing"]
            },
        )

        tagging._apply_tier2_tags(
            db,
            [target, pretagged],
            {"Flower": flower},
            ["Flower"],
            "api-key",
        )

        assert target.tags == [flower]
        assert target.tagging_tier == 2
        assert pretagged.tags == ["manual"]
        assert db.commit_count == 1

    def test_apply_tier2_tags_cryptic_filename_returns_no_tags(self, monkeypatch):
        """Test 6.2.2 — Cryptic filename results in no Tier 2 tags.

        When a design has a non-descriptive filename (e.g. ``ABC123``,
        ``zzz_001``), Gemini may return an empty list. The design should
        remain untagged with ``tagging_tier`` left as ``None``.
        """
        from src.services import auto_tagging, tagging

        db = DummyDb()
        flower = SimpleNamespace(id=1, description="Flower")
        cryptic = SimpleNamespace(filename="ABC123.JEF", tags=[], tagging_tier=None)

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier2_batch",
            lambda filenames, valid_descriptions, api_key: {
                "abc123": [],  # Gemini returned no matches
            },
        )

        tagging._apply_tier2_tags(
            db,
            [cryptic],
            {"Flower": flower},
            ["Flower"],
            "api-key",
        )

        # No tags should be assigned
        assert cryptic.tags == []
        assert cryptic.tagging_tier is None
        # No commit needed when nothing was updated
        assert db.commit_count == 0

    def test_apply_tier3_tags_returns_early_when_no_design_has_image_data(self, monkeypatch):
        from src.services import auto_tagging, tagging

        db = DummyDb()
        no_preview = SimpleNamespace(
            id=1, filename="owl.jef", image_data=None, tags=[], tagging_tier=None
        )

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier3_vision",
            lambda *_args, **_kwargs: pytest.fail("Tier 3 AI should not be called"),
        )

        tagging._apply_tier3_tags(db, [no_preview], {}, [], "api-key")

        assert db.commit_count == 0
        assert no_preview.tagging_tier is None

    def test_apply_tier3_tags_skips_when_all_designs_already_tagged(self, monkeypatch):
        """Test 6.3.3 — All designs already tagged by Tiers 1+2 → Tier 3 skipped.

        When every design already has tags (from Tier 1 or Tier 2 keyword
        matching), ``_apply_tier3_tags`` should return early without calling
        the Gemini vision API, even if designs have ``image_data``.
        """
        from src.services import auto_tagging, tagging

        db = DummyDb()
        already_tagged = SimpleNamespace(
            id=1, filename="rose.jef", image_data=b"png", tags=["existing"], tagging_tier=None
        )

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier3_vision",
            lambda *_args, **_kwargs: pytest.fail("Tier 3 AI should not be called"),
        )

        tagging._apply_tier3_tags(db, [already_tagged], {}, [], "api-key")

        assert db.commit_count == 0
        assert already_tagged.tagging_tier is None
        assert already_tagged.tags == ["existing"]

    def test_apply_tier3_tags_assigns_results_by_design_id_and_commits(self, monkeypatch):
        from src.services import auto_tagging, tagging

        db = DummyDb()
        animal = SimpleNamespace(id=11, description="Animal")
        with_image = SimpleNamespace(
            id=11, filename="owl.jef", image_data=b"png", tags=[], tagging_tier=None
        )
        without_match = SimpleNamespace(
            id=12, filename="untagged.jef", image_data=b"png", tags=[], tagging_tier=None
        )

        monkeypatch.setattr(
            auto_tagging,
            "suggest_tier3_vision",
            lambda designs, valid_descriptions, api_key: {11: ["Animal", "Animal"], 12: []},
        )

        tagging._apply_tier3_tags(
            db,
            [with_image, without_match],
            {"Animal": animal},
            ["Animal"],
            "api-key",
        )

        assert with_image.tags == [animal]
        assert with_image.tagging_tier == 3
        assert without_match.tags == []
        assert without_match.tagging_tier is None
        assert db.commit_count == 1
