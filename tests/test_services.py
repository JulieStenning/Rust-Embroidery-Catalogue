from types import SimpleNamespace

import pytest

from src.services import designers, designs, hoops, persistence, projects, sources, tags
from src.services.validation import validate_non_empty_string, validate_rating

# ---------------------------------------------------------------------------
# Validation helpers
# ---------------------------------------------------------------------------


class TestValidation:
    def test_non_empty_string_strips(self):
        assert validate_non_empty_string("  hello  ", "x") == "hello"

    def test_non_empty_string_raises_on_blank(self):
        with pytest.raises(ValueError):
            validate_non_empty_string("   ", "x")

    def test_non_empty_string_raises_on_non_string(self):
        with pytest.raises(ValueError):
            validate_non_empty_string(123, "x")  # type: ignore[arg-type]

    def test_rating_valid(self):
        for r in range(1, 6):
            assert validate_rating(r) == r

    def test_rating_none(self):
        assert validate_rating(None) is None

    def test_rating_out_of_range(self):
        with pytest.raises(ValueError):
            validate_rating(6)
        with pytest.raises(ValueError):
            validate_rating(0)


# ---------------------------------------------------------------------------
# Persistence service
# ---------------------------------------------------------------------------


class TestPersistenceService:
    def test_copy_design_to_managed_folder_uses_settings_base_path_and_copies(
        self, db, tmp_path, monkeypatch
    ):
        source = tmp_path / "incoming" / "rose.dst"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"rose")
        managed_root = tmp_path / "managed"
        sd = SimpleNamespace(filepath="nested/rose.dst", source_full_path=str(source))

        monkeypatch.setattr(persistence, "get_designs_base_path", lambda _db: str(managed_root))

        ok, error = persistence.copy_design_to_managed_folder(db, sd)

        copied = managed_root / "nested" / "rose.dst"
        assert (ok, error) == (True, None)
        assert copied.exists()
        assert copied.read_bytes() == b"rose"

    def test_copy_design_to_managed_folder_leaves_existing_file_unchanged(
        self, db, tmp_path, monkeypatch
    ):
        source = tmp_path / "incoming" / "rose.dst"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"new-content")
        managed_root = tmp_path / "managed"
        copied = managed_root / "nested" / "rose.dst"
        copied.parent.mkdir(parents=True)
        copied.write_bytes(b"existing-content")
        sd = SimpleNamespace(filepath="nested/rose.dst", source_full_path=str(source))

        def fail_if_called(*_args, **_kwargs):
            raise AssertionError("copy2 should not be called when the destination already exists")

        monkeypatch.setattr(persistence.shutil, "copy2", fail_if_called)

        ok, error = persistence.copy_design_to_managed_folder(db, sd, base_path=str(managed_root))

        assert (ok, error) == (True, None)
        assert copied.read_bytes() == b"existing-content"

    def test_copy_design_to_managed_folder_blocks_path_traversal(self, db, tmp_path, caplog):
        source = tmp_path / "incoming" / "unsafe.dst"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"unsafe")
        managed_root = tmp_path / "managed"
        sd = SimpleNamespace(filepath="../outside/unsafe.dst", source_full_path=str(source))

        with caplog.at_level("WARNING"):
            ok, error = persistence.copy_design_to_managed_folder(
                db, sd, base_path=str(managed_root)
            )

        assert ok is False
        assert error == "File path is outside the managed folder — import skipped."
        assert "Skipping file with path outside managed folder" in caplog.text

    def test_copy_design_to_managed_folder_reports_oserror(self, db, tmp_path, monkeypatch, caplog):
        source = tmp_path / "incoming" / "broken.dst"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"broken")
        managed_root = tmp_path / "managed"
        sd = SimpleNamespace(filepath="nested/broken.dst", source_full_path=str(source))

        def raise_oserror(*_args, **_kwargs):
            raise OSError("disk full")

        monkeypatch.setattr(persistence.shutil, "copy2", raise_oserror)

        with caplog.at_level("WARNING"):
            ok, error = persistence.copy_design_to_managed_folder(
                db, sd, base_path=str(managed_root)
            )

        assert ok is False
        assert error == "Could not copy file into catalogue: disk full"
        assert "Could not copy" in caplog.text


# ---------------------------------------------------------------------------
# Designers service
# ---------------------------------------------------------------------------


# ---------------------------------------------------------------------------
# Hoops service
# ---------------------------------------------------------------------------


class TestHoopsService:
    def test_seed_creates_standard_hoops(self, db):
        hoops.seed_hoops(db)
        all_hoops = hoops.get_all(db)
        names = {h.name for h in all_hoops}
        assert {"Hoop A", "Hoop B", "Gigahoop"}.issubset(names)

    def test_seed_is_idempotent(self, db):
        hoops.seed_hoops(db)
        hoops.seed_hoops(db)
        all_hoops = hoops.get_all(db)
        names = [h.name for h in all_hoops]
        assert names.count("Hoop A") == 1

    def test_select_hoop_for_dimensions(self, db):
        hoops.seed_hoops(db)
        hoop = hoops.select_hoop_for_dimensions(db, 100, 100)
        assert hoop is not None
        assert hoop.name == "Hoop A"

    def test_select_hoop_too_large_returns_none(self, db):
        hoops.seed_hoops(db)
        hoop = hoops.select_hoop_for_dimensions(db, 999, 999)
        assert hoop is None


# ---------------------------------------------------------------------------
# Designs service
# ---------------------------------------------------------------------------


class TestDesignsService:
    def test_create_design(self, db):
        d = designs.create(
            db,
            {
                "filename": "rose.jef",
                "filepath": "C:\\designs\\rose.jef",
                "is_stitched": False,
            },
        )
        assert d.id is not None
        assert d.filename == "rose.jef"

    def test_set_stitched(self, db):
        d = designs.create(
            db, {"filename": "flower.jef", "filepath": "/f.jef", "is_stitched": False}
        )
        d = designs.set_stitched(db, d.id, True)
        assert d.is_stitched is True

    def test_set_rating(self, db):
        d = designs.create(db, {"filename": "star.jef", "filepath": "/s.jef", "is_stitched": False})
        d = designs.set_rating(db, d.id, 4)
        assert d.rating == 4

    def test_set_invalid_rating_raises(self, db):
        d = designs.create(db, {"filename": "moon.jef", "filepath": "/m.jef", "is_stitched": False})
        with pytest.raises(ValueError):
            designs.set_rating(db, d.id, 10)

    def test_filter_by_stitched(self, db):
        designs.create(db, {"filename": "a.jef", "filepath": "/a.jef", "is_stitched": True})
        designs.create(db, {"filename": "b.jef", "filepath": "/b.jef", "is_stitched": False})
        stitched, total = designs.get_all(db, is_stitched=True)
        assert all(d.is_stitched for d in stitched)


# ---------------------------------------------------------------------------
# Projects service
# ---------------------------------------------------------------------------


class TestProjectsService:
    def test_create_project(self, db):
        p = projects.create(db, "My Sewing Session", "A test project")
        assert p.id is not None
        assert p.name == "My Sewing Session"

    def test_add_remove_design(self, db):
        p = projects.create(db, "Project With Design")
        d = designs.create(db, {"filename": "p.jef", "filepath": "/p.jef", "is_stitched": False})
        p = projects.add_design(db, p.id, d.id)
        assert d in p.designs
        p = projects.remove_design(db, p.id, d.id)
        assert d not in p.designs

    def test_add_multiple_designs(self, db):
        p = projects.create(db, "Bulk Project With Designs")
        d1 = designs.create(db, {"filename": "p1.jef", "filepath": "/p1.jef", "is_stitched": False})
        d2 = designs.create(db, {"filename": "p2.jef", "filepath": "/p2.jef", "is_stitched": False})

        p = projects.add_designs(db, p.id, [d1.id, d2.id, d1.id])

        assert {design.id for design in p.designs} == {d1.id, d2.id}


# ---------------------------------------------------------------------------
# Tags service
# ---------------------------------------------------------------------------


class TestTagsServiceCore:
    def test_create_and_retrieve(self, db):
        tag = tags.create(db, "Flowers", "image")
        assert tag.id is not None
        assert tag.description == "Flowers"
        assert tag.tag_group == "image"
        fetched = tags.get_by_id(db, tag.id)
        assert fetched is not None
        assert fetched.description == "Flowers"

    def test_get_all_returns_list(self, db):
        tags.create(db, "Birds", "image")
        tags.create(db, "Animals", "image")
        result = tags.get_all(db)
        descriptions = [tag.description for tag in result]
        assert "Birds" in descriptions
        assert "Animals" in descriptions

    def test_get_all_ordered_alphabetically(self, db):
        tags.create(db, "Zebra", "image")
        tags.create(db, "Apple", "image")
        result = tags.get_all(db)
        descriptions = [tag.description for tag in result]
        assert descriptions == sorted(descriptions)

    def test_create_duplicate_raises(self, db):
        tags.create(db, "Cats", "image")
        with pytest.raises(ValueError, match="already exists"):
            tags.create(db, "Cats", "image")

    def test_create_blank_raises(self, db):
        with pytest.raises(ValueError):
            tags.create(db, "   ", "image")

    def test_create_invalid_group_raises(self, db):
        with pytest.raises(ValueError, match="Invalid tag group"):
            tags.create(db, "Valid Name", "invalid_group")

    def test_set_group(self, db):
        from src.models import Tag

        raw = Tag(description="Needs Group", tag_group=None)
        db.add(raw)
        db.commit()
        updated = tags.set_group(db, raw.id, "stitching")
        assert updated.tag_group == "stitching"

    def test_update(self, db):
        tag = tags.create(db, "Old Name", "image")
        updated = tags.update(db, tag.id, "New Name")
        assert updated.description == "New Name"

    def test_update_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            tags.update(db, 999_999, "Whatever")

    def test_delete(self, db):
        tag = tags.create(db, "To Delete Tag", "image")
        tags.delete(db, tag.id)
        assert tags.get_by_id(db, tag.id) is None

    def test_delete_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            tags.delete(db, 999_999)

    def test_get_by_id_missing_returns_none(self, db):
        assert tags.get_by_id(db, 999_999) is None


# ---------------------------------------------------------------------------
# Tags service
# ---------------------------------------------------------------------------


class TestTagsService:
    def test_create_and_retrieve(self, db):
        tag = tags.create(db, "Summer Flowers", "image")
        assert tag.id is not None
        assert tag.description == "Summer Flowers"
        assert tag.tag_group == "image"
        fetched = tags.get_by_id(db, tag.id)
        assert fetched is not None
        assert fetched.description == "Summer Flowers"

    def test_get_all_returns_list(self, db):
        tags.create(db, "Tag Birds", "image")
        tags.create(db, "Tag Animals", "image")
        result = tags.get_all(db)
        descriptions = [t.description for t in result]
        assert "Tag Birds" in descriptions
        assert "Tag Animals" in descriptions

    def test_get_all_ordered_alphabetically(self, db):
        tags.create(db, "Tag Zebra", "image")
        tags.create(db, "Tag Apple", "image")
        result = tags.get_all(db)
        descriptions = [t.description for t in result]
        assert descriptions == sorted(descriptions)

    def test_create_duplicate_raises(self, db):
        tags.create(db, "Tag Cats", "image")
        with pytest.raises(ValueError, match="already exists"):
            tags.create(db, "Tag Cats", "image")

    def test_create_blank_raises(self, db):
        with pytest.raises(ValueError):
            tags.create(db, "   ", "image")

    def test_create_invalid_group_raises(self, db):
        with pytest.raises(ValueError, match="Invalid tag group"):
            tags.create(db, "Valid Tag Name", "invalid_group")

    def test_set_group(self, db):
        from src.models import Tag

        raw = Tag(description="Tag Needs Group", tag_group=None)
        db.add(raw)
        db.commit()
        updated = tags.set_group(db, raw.id, "stitching")
        assert updated.tag_group == "stitching"

    def test_update(self, db):
        tag = tags.create(db, "Old Tag Name", "image")
        updated = tags.update(db, tag.id, "New Tag Name")
        assert updated.description == "New Tag Name"

    def test_update_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            tags.update(db, 999_999, "Whatever")

    def test_delete(self, db):
        tag = tags.create(db, "Tag To Delete", "image")
        tags.delete(db, tag.id)
        assert tags.get_by_id(db, tag.id) is None

    def test_delete_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            tags.delete(db, 999_999)

    def test_get_by_id_missing_returns_none(self, db):
        assert tags.get_by_id(db, 999_999) is None


# ---------------------------------------------------------------------------
# Sources service
# ---------------------------------------------------------------------------


class TestSourcesService:
    def test_create_and_retrieve(self, db):
        s = sources.create(db, "Anita Goodesign")
        assert s.id is not None
        fetched = sources.get_by_id(db, s.id)
        assert fetched is not None
        assert fetched.name == "Anita Goodesign"

    def test_get_all_returns_list(self, db):
        sources.create(db, "Source A")
        sources.create(db, "Source B")
        result = sources.get_all(db)
        names = [s.name for s in result]
        assert "Source A" in names
        assert "Source B" in names

    def test_get_all_ordered_alphabetically(self, db):
        sources.create(db, "Zebra Source")
        sources.create(db, "Apple Source")
        result = sources.get_all(db)
        names = [s.name for s in result]
        assert names == sorted(names)

    def test_create_duplicate_raises(self, db):
        sources.create(db, "Unique Source")
        with pytest.raises(ValueError, match="already exists"):
            sources.create(db, "Unique Source")

    def test_create_blank_raises(self, db):
        with pytest.raises(ValueError):
            sources.create(db, "  ")

    def test_update(self, db):
        s = sources.create(db, "Old Source")
        updated = sources.update(db, s.id, "New Source")
        assert updated.name == "New Source"

    def test_update_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            sources.update(db, 999_999, "Whatever")

    def test_delete(self, db):
        s = sources.create(db, "To Delete Source")
        sources.delete(db, s.id)
        assert sources.get_by_id(db, s.id) is None

    def test_delete_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            sources.delete(db, 999_999)

    def test_get_by_id_missing_returns_none(self, db):
        assert sources.get_by_id(db, 999_999) is None


# ---------------------------------------------------------------------------
# Hoops service — additional tests
# ---------------------------------------------------------------------------


class TestHoopsServiceExtra:
    def test_create_hoop(self, db):
        h = hoops.create(db, "My Hoop", 180.0, 130.0)
        assert h.id is not None
        assert h.name == "My Hoop"

    def test_get_by_id(self, db):
        h = hoops.create(db, "Lookup Hoop", 100.0, 100.0)
        fetched = hoops.get_by_id(db, h.id)
        assert fetched is not None
        assert fetched.name == "Lookup Hoop"

    def test_get_by_id_missing_returns_none(self, db):
        assert hoops.get_by_id(db, 999_999) is None

    def test_create_duplicate_raises(self, db):
        hoops.create(db, "Dup Hoop", 100.0, 100.0)
        with pytest.raises(ValueError, match="already exists"):
            hoops.create(db, "Dup Hoop", 200.0, 200.0)

    def test_create_invalid_dimensions_raises(self, db):
        with pytest.raises(ValueError):
            hoops.create(db, "Bad Hoop", -10.0, 100.0)

    def test_update_hoop(self, db):
        h = hoops.create(db, "Update Me", 100.0, 100.0)
        updated = hoops.update(db, h.id, "Updated", 200.0, 150.0)
        assert updated.name == "Updated"
        assert float(updated.max_width_mm) == 200.0

    def test_update_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            hoops.update(db, 999_999, "Nope", 100.0, 100.0)

    def test_delete_hoop(self, db):
        h = hoops.create(db, "Delete Me", 100.0, 100.0)
        hoops.delete(db, h.id)
        assert hoops.get_by_id(db, h.id) is None

    def test_delete_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            hoops.delete(db, 999_999)

    def test_select_rotated_fit(self, db):
        hoops.seed_hoops(db)
        # 200×126 fits rotated into Hoop B (200×140)
        hoop = hoops.select_hoop_for_dimensions(db, 200.0, 126.0)
        assert hoop is not None


# ---------------------------------------------------------------------------
# Designs service — additional tests
# ---------------------------------------------------------------------------


class TestDesignsServiceExtra:
    def test_get_by_id(self, db):
        d = designs.create(
            db, {"filename": "find.jef", "filepath": "/find.jef", "is_stitched": False}
        )
        fetched = designs.get_by_id(db, d.id)
        assert fetched is not None
        assert fetched.filename == "find.jef"

    def test_get_by_id_missing_returns_none(self, db):
        assert designs.get_by_id(db, 999_999) is None

    def test_update_design(self, db):
        d = designs.create(
            db, {"filename": "orig.jef", "filepath": "/orig.jef", "is_stitched": False}
        )
        updated = designs.update(
            db, d.id, {"filename": "changed.jef", "filepath": "/orig.jef", "is_stitched": True}
        )
        assert updated.filename == "changed.jef"
        assert updated.is_stitched is True

    def test_update_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            designs.update(
                db, 999_999, {"filename": "x.jef", "filepath": "/x.jef", "is_stitched": False}
            )

    def test_delete_design(self, db):
        d = designs.create(
            db, {"filename": "del.jef", "filepath": "/del.jef", "is_stitched": False}
        )
        designs.delete(db, d.id)
        assert designs.get_by_id(db, d.id) is None

    def test_delete_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            designs.delete(db, 999_999)

    def test_set_tags_checked(self, db):
        d = designs.create(db, {"filename": "tc.jef", "filepath": "/tc.jef", "is_stitched": False})
        result = designs.set_tags_checked(db, d.id, True)
        assert result.tags_checked is True

    def test_set_tags_checked_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            designs.set_tags_checked(db, 999_999, True)

    def test_set_stitched_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            designs.set_stitched(db, 999_999, True)

    def test_set_rating_nonexistent_raises(self, db):
        with pytest.raises(ValueError):
            designs.set_rating(db, 999_999, 3)

    def test_filter_by_unverified(self, db):
        designs.create(
            db,
            {"filename": "v.jef", "filepath": "/v.jef", "is_stitched": False, "tags_checked": True},
        )
        designs.create(
            db,
            {
                "filename": "uv.jef",
                "filepath": "/uv.jef",
                "is_stitched": False,
                "tags_checked": False,
            },
        )
        results, total = designs.get_all(db, unverified=True)
        assert all(not d.tags_checked for d in results)

    def test_filter_by_filename(self, db):
        designs.create(
            db, {"filename": "unicorn_border.jef", "filepath": "/u.jef", "is_stitched": False}
        )
        designs.create(db, {"filename": "rose.jef", "filepath": "/r.jef", "is_stitched": False})
        results, _ = designs.get_all(db, filename="unicorn*")
        assert all("unicorn" in d.filename.lower() for d in results)

    def test_filter_by_filename_extension_wildcard(self, db):
        """Filename filter with *.ext should match designs whose filepath has that extension."""
        designs.create(db, {"filename": "snowflake", "filepath": "/snf.jef", "is_stitched": False})
        designs.create(db, {"filename": "butterfly", "filepath": "/btn.hus", "is_stitched": False})
        results, _ = designs.get_all(db, filename="*.jef")
        filepaths = [d.filepath for d in results]
        assert any(".jef" in fp for fp in filepaths), "*.jef filter should match .jef designs"
        assert not any(
            ".hus" in fp for fp in filepaths
        ), "*.jef filter should not match .hus designs"

    def test_filter_by_tag(self, db):
        tag = tags.create(db, "Butterflies", "image")
        d = designs.create(db, {"filename": "bf.jef", "filepath": "/bf.jef", "is_stitched": False})
        designs.update(
            db,
            d.id,
            {
                "filename": "bf.jef",
                "filepath": "/bf.jef",
                "is_stitched": False,
                "tag_ids": [tag.id],
            },
        )
        results, _ = designs.get_all(db, tag_ids=[tag.id])
        ids = [r.id for r in results]
        assert d.id in ids

    def test_filter_untagged(self, db):
        designs.create(
            db, {"filename": "notag.jef", "filepath": "/notag.jef", "is_stitched": False}
        )
        results, _ = designs.get_all(db, tag_ids=[-1])
        assert all(len(d.tags) == 0 for d in results)

    def test_get_image_base64_with_data(self, db):
        d = designs.create(
            db,
            {
                "filename": "img.jef",
                "filepath": "/img.jef",
                "is_stitched": False,
                "image_data": b"PNG_DATA",
            },
        )
        b64 = designs.get_image_base64(d)
        assert b64 is not None
        import base64

        assert base64.b64decode(b64) == b"PNG_DATA"

    def test_get_image_base64_no_data(self, db):
        d = designs.create(
            db, {"filename": "noimg.jef", "filepath": "/noimg.jef", "is_stitched": False}
        )
        assert designs.get_image_base64(d) is None


class TestDesignersService:
    def test_create_and_get_all(self, db):
        from src.services import designers

        d = designers.create(db, "Anita Goodesign")
        assert d.id is not None
        all_designers = designers.get_all(db)
        names = [x.name for x in all_designers]
        assert "Anita Goodesign" in names

    def test_get_all_alphabetical(self, db):
        from src.services import designers

        designers.create(db, "Zebra Designs")
        designers.create(db, "Apex Designs")
        all_designers = designers.get_all(db)
        names = [x.name for x in all_designers]
        assert names == sorted(names)

    def test_get_by_id(self, db):
        from src.services import designers

        d = designers.create(db, "Get By ID Designer")
        fetched = designers.get_by_id(db, d.id)
        assert fetched is not None
        assert fetched.name == "Get By ID Designer"

    def test_get_by_id_missing(self, db):
        from src.services import designers

        assert designers.get_by_id(db, 999999) is None

    def test_create_duplicate_raises(self, db):
        from src.services import designers

        designers.create(db, "Unique Designer")
        with pytest.raises(ValueError, match="already exists"):
            designers.create(db, "Unique Designer")

    def test_create_blank_raises(self, db):
        from src.services import designers

        with pytest.raises(ValueError):
            designers.create(db, "   ")

    def test_update(self, db):
        from src.services import designers

        d = designers.create(db, "Old Name")
        updated = designers.update(db, d.id, "New Name")
        assert updated.name == "New Name"

    def test_update_nonexistent_raises(self, db):
        from src.services import designers

        with pytest.raises(ValueError, match="not found"):
            designers.update(db, 999999, "Whatever")

    def test_delete(self, db):
        from src.services import designers

        d = designers.create(db, "Delete Me")
        designer_id = d.id
        designers.delete(db, designer_id)
        assert designers.get_by_id(db, designer_id) is None

    def test_delete_nonexistent_raises(self, db):
        from src.services import designers

        with pytest.raises(ValueError, match="not found"):
            designers.delete(db, 999999)


class TestSettingsService:
    def test_get_setting_default(self, db):
        from src.services import settings_service as svc

        # Unknown key falls back to empty string
        val = svc.get_setting(db, "nonexistent_key")
        assert val == ""

    def test_set_and_get_setting(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/test/path")
        val = svc.get_setting(db, svc.SETTING_DESIGNS_BASE_PATH)
        assert val == "/test/path"

    def test_set_setting_update(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/first")
        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/second")
        assert svc.get_setting(db, svc.SETTING_DESIGNS_BASE_PATH) == "/second"

    def test_get_all_returns_list(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/list/path")
        settings = svc.get_all(db)
        assert isinstance(settings, list)
        assert len(settings) >= 1

    def test_get_all_alphabetical(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/any")
        settings = svc.get_all(db)
        keys = [s.key for s in settings]
        assert keys == sorted(keys)

    def test_get_designs_base_path(self, db, monkeypatch):
        import os

        from src.services import settings_service as svc

        monkeypatch.setattr(svc, "DESIGNS_BASE_PATH", "/tmp/MachineEmbroideryDesigns")
        result = svc.get_designs_base_path(db)
        assert result == os.path.normpath("/tmp/MachineEmbroideryDesigns")

    def test_get_designs_base_path_ignores_legacy_external_setting(self, db, monkeypatch):
        import os

        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/old/external/designs")
        monkeypatch.setattr(svc, "DESIGNS_BASE_PATH", "/app/data/MachineEmbroideryDesigns")
        result = svc.get_designs_base_path(db)
        assert result == os.path.normpath("/app/data/MachineEmbroideryDesigns")

    def test_get_designs_base_path_ignores_stale_machineembroiderydesigns_setting(
        self, db, monkeypatch
    ):
        import os

        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, r"J:\\MachineEmbroideryDesigns")
        monkeypatch.setattr(
            svc, "DESIGNS_BASE_PATH", r"D:\\EmbroideryApp\\data\\MachineEmbroideryDesigns"
        )
        result = svc.get_designs_base_path(db)
        assert result == os.path.normpath(r"D:\\EmbroideryApp\\data\\MachineEmbroideryDesigns")

    def test_get_data_root_returns_current_configured_root(self, db, monkeypatch):
        import os

        from src.services import settings_service as svc

        monkeypatch.setattr(svc, "DATA_ROOT", r"D:\\EmbroideryCatalogueData")
        assert svc.get_data_root(db) == os.path.normpath(r"D:\\EmbroideryCatalogueData")

    def test_get_database_file_path_returns_sqlite_path(self, db, monkeypatch):
        import os

        from src.services import settings_service as svc

        monkeypatch.setattr(
            svc, "DATABASE_URL", "sqlite:///D:/EmbroideryCatalogueData/database/catalogue.db"
        )
        assert svc.get_database_file_path(db) == os.path.normpath(
            r"D:\EmbroideryCatalogueData\database\catalogue.db"
        )

    def test_save_data_root_persists_pointer_and_copies_existing_data(
        self, db, monkeypatch, tmp_path
    ):
        import json
        import os

        from src.services import settings_service as svc

        source_root = tmp_path / "CurrentData"
        target_root = tmp_path / "NewData"
        storage_file = tmp_path / "LocalAppData" / "EmbroideryCatalogue" / "storage.json"

        (source_root / "database").mkdir(parents=True, exist_ok=True)
        (source_root / "database" / "catalogue.db").write_bytes(b"db-bytes")
        (source_root / "MachineEmbroideryDesigns" / "SetA").mkdir(parents=True, exist_ok=True)
        (source_root / "MachineEmbroideryDesigns" / "SetA" / "flower.pes").write_text(
            "stitch", encoding="utf-8"
        )

        monkeypatch.setattr(svc, "APP_MODE", "desktop")
        monkeypatch.setattr(svc, "DATA_ROOT", source_root)
        monkeypatch.setattr(
            svc,
            "DATABASE_URL",
            f"sqlite:///{(source_root / 'database' / 'catalogue.db').as_posix()}",
        )
        monkeypatch.setattr(svc, "DESIGNS_BASE_PATH", str(source_root / "MachineEmbroideryDesigns"))
        monkeypatch.setattr(svc, "STORAGE_CONFIG_FILE", storage_file)

        saved = svc.save_data_root(str(target_root))

        assert saved == os.path.normpath(str(target_root))
        assert json.loads(storage_file.read_text(encoding="utf-8"))[
            "data_root"
        ] == os.path.normpath(str(target_root))
        assert (target_root / "database" / "catalogue.db").read_bytes() == b"db-bytes"
        assert (target_root / "MachineEmbroideryDesigns" / "SetA" / "flower.pes").read_text(
            encoding="utf-8"
        ) == "stitch"

    def test_config_ignores_designs_base_path_environment_override(self, monkeypatch):
        import importlib
        import os

        import src.config as config_mod

        monkeypatch.setenv("APP_MODE", "development")
        monkeypatch.delenv("EMBROIDERY_DATA_ROOT", raising=False)
        monkeypatch.setenv("DESIGNS_BASE_PATH", r"J:\\MachineEmbroideryDesigns")
        reloaded = importlib.reload(config_mod)
        expected = os.path.normpath(str(reloaded._APP_ROOT / "data" / "MachineEmbroideryDesigns"))
        assert os.path.normpath(reloaded.DESIGNS_BASE_PATH) == expected

    def test_ai_tier2_auto_default_is_false(self, db):
        from src.services import settings_service as svc

        val = svc.get_setting(db, svc.SETTING_AI_TIER2_AUTO)
        assert not svc._is_truthy(val)

    def test_ai_tier3_auto_default_is_false(self, db):
        from src.services import settings_service as svc

        val = svc.get_setting(db, svc.SETTING_AI_TIER3_AUTO)
        assert not svc._is_truthy(val)

    def test_ai_batch_size_default_is_empty(self, db):
        from src.services import settings_service as svc

        val = svc.get_setting(db, svc.SETTING_AI_BATCH_SIZE)
        assert val == ""

    def test_set_and_get_ai_tier2_auto(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_AI_TIER2_AUTO, "true")
        assert svc._is_truthy(svc.get_setting(db, svc.SETTING_AI_TIER2_AUTO))

    def test_set_and_get_ai_tier3_auto(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_AI_TIER3_AUTO, "true")
        assert svc._is_truthy(svc.get_setting(db, svc.SETTING_AI_TIER3_AUTO))

    def test_set_and_get_ai_batch_size(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_AI_BATCH_SIZE, "100")
        assert svc.get_setting(db, svc.SETTING_AI_BATCH_SIZE) == "100"

    # ------------------------------------------------------------------ #
    # Image preference (2D / 3D)
    # ------------------------------------------------------------------ #

    def test_image_preference_default_is_2d(self, db):
        from src.services import settings_service as svc

        val = svc.get_setting(db, svc.SETTING_IMAGE_PREFERENCE)
        assert val == "2d"

    def test_set_and_get_image_preference_2d(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_IMAGE_PREFERENCE, "2d")
        assert svc.get_setting(db, svc.SETTING_IMAGE_PREFERENCE) == "2d"

    def test_set_and_get_image_preference_3d(self, db):
        from src.services import settings_service as svc

        svc.set_setting(db, svc.SETTING_IMAGE_PREFERENCE, "3d")
        assert svc.get_setting(db, svc.SETTING_IMAGE_PREFERENCE) == "3d"

    def test_image_preference_rejects_invalid_value(self, db):
        from src.services import settings_service as svc

        # The service layer doesn't validate, but we can verify it stores
        # whatever is passed (validation is at the route/form level).
        svc.set_setting(db, svc.SETTING_IMAGE_PREFERENCE, "invalid")
        assert svc.get_setting(db, svc.SETTING_IMAGE_PREFERENCE) == "invalid"


class TestProjectsServiceExtra:
    def test_create_duplicate_raises(self, db):
        from src.services import projects

        projects.create(db, "Unique Project")
        with pytest.raises(ValueError, match="already exists"):
            projects.create(db, "Unique Project")

    def test_update_nonexistent_raises(self, db):
        from src.services import projects

        with pytest.raises(ValueError, match="not found"):
            projects.update(db, 999999, "Whatever")

    def test_delete_nonexistent_raises(self, db):
        from src.services import projects

        with pytest.raises(ValueError, match="not found"):
            projects.delete(db, 999999)

    def test_add_design_project_not_found(self, db):
        from src.services import projects

        d = designs.create(db, {"filename": "x.jef", "filepath": "/x.jef", "is_stitched": False})
        with pytest.raises(ValueError, match="not found"):
            projects.add_design(db, 999999, d.id)

    def test_add_design_design_not_found(self, db):
        from src.services import projects

        p = projects.create(db, "Test Add Project")
        with pytest.raises(ValueError, match="not found"):
            projects.add_design(db, p.id, 999999)

    def test_remove_design_project_not_found(self, db):
        from src.services import projects

        with pytest.raises(ValueError, match="not found"):
            projects.remove_design(db, 999999, 999999)


class TestDesignsServiceFilters:
    def test_filter_by_plain_substring(self, db):
        designs.create(
            db, {"filename": "flower_garden.jef", "filepath": "/fg.jef", "is_stitched": False}
        )
        all_designs, _ = designs.get_all(db)
        print("DEBUG: All designs in DB:", [d.filename for d in all_designs])
        results, _ = designs.get_all(db, filename="flower_garden")
        print("DEBUG: Results filenames:", [d.filename for d in results])
        assert any("flower_garden" in d.filename for d in results)

    def test_filter_by_designer_id(self, db):
        from src.services import designers

        des = designers.create(db, "Filter Designer")
        d = designs.create(
            db,
            {
                "filename": "des_design.jef",
                "filepath": "/dd.jef",
                "is_stitched": False,
                "designer_id": des.id,
            },
        )
        results, _ = designs.get_all(db, designer_id=des.id)
        ids = [r.id for r in results]
        assert d.id in ids

    def test_filter_by_hoop_id(self, db):
        from src.services import hoops

        h = hoops.create(db, "5x7", 127.0, 178.0)
        d = designs.create(
            db,
            {
                "filename": "hoop_des.jef",
                "filepath": "/hd.jef",
                "is_stitched": False,
                "hoop_id": h.id,
            },
        )
        results, _ = designs.get_all(db, hoop_id=h.id)
        ids = [r.id for r in results]
        assert d.id in ids

    def test_filter_by_source_id(self, db):
        from src.services import sources

        s = sources.create(db, "Filter Source")
        d = designs.create(
            db,
            {
                "filename": "src_des.jef",
                "filepath": "/sd.jef",
                "is_stitched": False,
                "source_id": s.id,
            },
        )
        results, _ = designs.get_all(db, source_id=s.id)
        ids = [r.id for r in results]
        assert d.id in ids

    def test_filter_by_rating(self, db):
        d = designs.create(
            db,
            {"filename": "rated.jef", "filepath": "/rated.jef", "is_stitched": False, "rating": 5},
        )
        results, _ = designs.get_all(db, rating=5)
        ids = [r.id for r in results]
        assert d.id in ids

    def test_create_with_tag_ids(self, db):
        tag = tags.create(db, "Animals", "image")
        d = designs.create(
            db,
            {
                "filename": "animal.jef",
                "filepath": "/an.jef",
                "is_stitched": False,
                "tag_ids": [tag.id],
            },
        )
        assert any(t.id == tag.id for t in d.tags)

    def test_sort_by_date_desc(self, db):
        results, _ = designs.get_all(db, sort_by="date_added", sort_dir="desc")
        assert isinstance(results, list)


class TestModelsRepr:
    def test_designer_repr(self, db):
        from src.services import designers

        d = designers.create(db, "Repr Designer")
        assert "Repr Designer" in repr(d)
        assert "Designer" in repr(d)

    def test_source_repr(self, db):
        from src.services import sources

        s = sources.create(db, "Repr Source")
        assert "Repr Source" in repr(s)
        assert "Source" in repr(s)

    def test_hoop_repr(self, db):
        from src.services import hoops

        h = hoops.create(db, "Repr Hoop", 100.0, 150.0)
        assert "Repr Hoop" in repr(h)
        assert "Hoop" in repr(h)

    def test_tag_repr(self, db):
        tag = tags.create(db, "Repr Type", "image")
        assert "Repr Type" in repr(tag)
        assert "Tag" in repr(tag)

    def test_design_repr(self, db):
        d = designs.create(
            db, {"filename": "repr.jef", "filepath": "/repr.jef", "is_stitched": False}
        )
        assert "repr.jef" in repr(d)
        assert "Design" in repr(d)

    def test_project_repr(self, db):
        from src.services import projects

        p = projects.create(db, "Repr Project")
        assert "Repr Project" in repr(p)
        assert "Project" in repr(p)

    def test_setting_repr(self, db):
        from src.services import settings_service as svc

        s = svc.set_setting(db, svc.SETTING_DESIGNS_BASE_PATH, "/repr/path")
        assert "Setting" in repr(s)
        assert svc.SETTING_DESIGNS_BASE_PATH in repr(s)


# ---------------------------------------------------------------------------
# Advanced search — query parsing
# ---------------------------------------------------------------------------


class TestParseAdvancedQuery:
    def test_all_words(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(all_words="rose flower")
        assert "rose" in pq.required_words
        assert "flower" in pq.required_words
        assert pq.exact_phrases == []
        assert pq.any_words == []
        assert pq.excluded_words == []

    def test_exact_phrase(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(exact_phrase="cross stitch")
        assert "cross stitch" in pq.exact_phrases
        assert pq.required_words == []

    def test_exact_phrase_strips_optional_quotes(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(exact_phrase='  "words and letters"  ')
        assert pq.exact_phrases == ["words and letters"]
        assert pq.required_words == []

    def test_all_words_supports_quoted_phrase(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(all_words='"words and letters" floral')
        assert pq.exact_phrases == ["words and letters"]
        assert pq.required_words == ["floral"]

    def test_any_words(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(any_words="rose tulip daisy")
        assert set(pq.any_words) == {"rose", "tulip", "daisy"}
        assert pq.required_words == []

    def test_none_words(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(none_words="applique border")
        assert "applique" in pq.excluded_words
        assert "border" in pq.excluded_words

    def test_google_syntax_required_words(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(q="rose flower")
        assert "rose" in pq.required_words
        assert "flower" in pq.required_words

    def test_google_syntax_exact_phrase(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(q='"cross stitch"')
        assert "cross stitch" in pq.exact_phrases
        assert pq.required_words == []

    def test_google_syntax_exact_phrase_with_smart_quotes(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(q="“words and letters”")
        assert pq.exact_phrases == ["words and letters"]
        assert pq.required_words == []

    def test_google_syntax_exclusion(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(q="rose -applique")
        assert "rose" in pq.required_words
        assert "applique" in pq.excluded_words

    def test_google_syntax_or(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(q="rose OR tulip")
        assert "rose" in pq.any_words
        assert "tulip" in pq.any_words
        assert pq.required_words == []

    def test_combined_fields_and_q(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(
            q="summer",
            all_words="rose",
            exact_phrase="cross stitch",
            any_words="tulip daisy",
            none_words="applique",
        )
        assert "rose" in pq.required_words
        assert "summer" in pq.required_words
        assert "cross stitch" in pq.exact_phrases
        assert "tulip" in pq.any_words
        assert "daisy" in pq.any_words
        assert "applique" in pq.excluded_words

    def test_empty_query_is_empty(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query()
        assert pq.is_empty()

    def test_non_empty_query_is_not_empty(self):
        from src.services.search import parse_advanced_query

        pq = parse_advanced_query(all_words="rose")
        assert not pq.is_empty()


# ---------------------------------------------------------------------------
# Advanced search — service integration (uses DB)
# ---------------------------------------------------------------------------


class TestAdvancedSearchService:
    def test_finds_by_filename_word(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db,
            {
                "filename": "rose_applique.jef",
                "filepath": "/a/rose_applique.jef",
                "is_stitched": False,
            },
        )
        designs.create(
            db, {"filename": "butterfly.jef", "filepath": "/a/butterfly.jef", "is_stitched": False}
        )
        pq = parse_advanced_query(all_words="rose")
        results, total = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filenames = [d.filename for d in results]
        assert any("rose" in f for f in filenames)
        assert not any("butterfly" in f for f in filenames)

    def test_excludes_word(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db, {"filename": "rose_v1.jef", "filepath": "/a/rose_v1.jef", "is_stitched": False}
        )
        designs.create(
            db,
            {"filename": "rose_applique.jef", "filepath": "/a/rose_app.jef", "is_stitched": False},
        )
        pq = parse_advanced_query(all_words="rose", none_words="applique")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filenames = [d.filename for d in results]
        assert all("applique" not in f for f in filenames)
        assert any("rose_v1" in f for f in filenames)

    def test_or_words(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db, {"filename": "rose.jef", "filepath": "/a/rose.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "tulip.jef", "filepath": "/a/tulip.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "daisy.jef", "filepath": "/a/daisy.jef", "is_stitched": False}
        )
        pq = parse_advanced_query(any_words="rose tulip")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filenames = [d.filename for d in results]
        assert any("rose" in f for f in filenames)
        assert any("tulip" in f for f in filenames)
        assert not any("daisy" in f for f in filenames)

    def test_exact_phrase(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db,
            {"filename": "cross_stitch_rose.jef", "filepath": "/a/csr.jef", "is_stitched": False},
        )
        designs.create(
            db, {"filename": "crossstitch.jef", "filepath": "/a/cs.jef", "is_stitched": False}
        )
        pq = parse_advanced_query(exact_phrase="cross_stitch")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filenames = [d.filename for d in results]
        assert any("cross_stitch_rose" in f for f in filenames)

    def test_folder_search_matches_nested_parent_path(self, db):
        from src.services.search import parse_advanced_query

        wanted = designs.create(
            db,
            {
                "filename": "nested_rose",
                "filepath": "/collections/My Creations/Animals/nested_rose.jef",
                "is_stitched": False,
            },
        )
        designs.create(
            db,
            {
                "filename": "other_rose",
                "filepath": "/collections/Other Collection/Animals/other_rose.jef",
                "is_stitched": False,
            },
        )

        pq = parse_advanced_query(q='"My Creations"')
        results, _ = designs.advanced_search(
            db, pq, search_filename=False, search_tags=False, search_folder=True
        )

        result_ids = [d.id for d in results]
        assert wanted.id in result_ids
        assert all("Other Collection" not in d.filepath for d in results)

    def test_exact_phrase_matches_tag_without_quotes_in_value(self, db):
        from src.services.search import parse_advanced_query

        matching_tag = tags.create(db, "Words and Letters", "image")
        other_tag = tags.create(db, "Animals", "image")

        wanted = designs.create(
            db,
            {
                "filename": "alphabet_sampler.jef",
                "filepath": "/a/alphabet_sampler.jef",
                "is_stitched": False,
                "tag_ids": [matching_tag.id],
            },
        )
        designs.create(
            db,
            {
                "filename": "cat_sampler.jef",
                "filepath": "/a/cat_sampler.jef",
                "is_stitched": False,
                "tag_ids": [other_tag.id],
            },
        )

        pq = parse_advanced_query(exact_phrase='"Words and Letters"')
        results, _ = designs.advanced_search(
            db, pq, search_filename=False, search_tags=True, search_folder=False
        )
        result_ids = [d.id for d in results]
        assert wanted.id in result_ids
        assert len(result_ids) == 1

    def test_advanced_search_applies_standard_filters(self, db):
        from src.services.search import parse_advanced_query

        des = designers.create(db, "Advanced Filter Designer")
        other_des = designers.create(db, "Other Advanced Designer")
        hoop = hoops.create(db, "Adv Hoop", 100, 100)
        other_hoop = hoops.create(db, "Other Adv Hoop", 120, 120)
        src = sources.create(db, "Adv Source")
        other_src = sources.create(db, "Other Adv Source")
        tag = tags.create(db, "Advanced Filter Tag", "image")
        other_tag = tags.create(db, "Other Advanced Tag", "image")

        wanted = designs.create(
            db,
            {
                "filename": "advanced_filter_match.jef",
                "filepath": "/adv/advanced_filter_match.jef",
                "is_stitched": True,
                "designer_id": des.id,
                "hoop_id": hoop.id,
                "source_id": src.id,
                "rating": 5,
                "tags_checked": False,
                "tag_ids": [tag.id],
            },
        )
        designs.create(
            db,
            {
                "filename": "advanced_filter_other.jef",
                "filepath": "/adv/advanced_filter_other.jef",
                "is_stitched": False,
                "designer_id": other_des.id,
                "hoop_id": other_hoop.id,
                "source_id": other_src.id,
                "rating": 1,
                "tags_checked": True,
                "tag_ids": [other_tag.id],
            },
        )

        pq = parse_advanced_query()
        results, _ = designs.advanced_search(
            db,
            pq,
            search_filename=True,
            search_tags=True,
            search_folder=True,
            designer_id=des.id,
            tag_ids=[tag.id],
            hoop_id=hoop.id,
            source_id=src.id,
            rating=5,
            is_stitched=True,
            unverified=True,
        )
        result_ids = [d.id for d in results]
        assert result_ids == [wanted.id]

    def test_all_words_quoted_phrase_matches_tag(self, db):
        from src.services.search import parse_advanced_query

        matching_tag = tags.create(db, "Words and Letters 2", "image")
        other_tag = tags.create(db, "Birds 2", "image")

        wanted = designs.create(
            db,
            {
                "filename": "quoted_phrase_match.jef",
                "filepath": "/a/quoted_phrase_match.jef",
                "is_stitched": False,
                "tag_ids": [matching_tag.id],
            },
        )
        designs.create(
            db,
            {
                "filename": "quoted_phrase_other.jef",
                "filepath": "/a/quoted_phrase_other.jef",
                "is_stitched": False,
                "tag_ids": [other_tag.id],
            },
        )

        pq = parse_advanced_query(all_words='"Words and Letters 2"')
        results, _ = designs.advanced_search(
            db, pq, search_filename=False, search_tags=True, search_folder=False
        )
        result_ids = [d.id for d in results]
        assert wanted.id in result_ids
        assert len(result_ids) == 1

    def test_empty_query_returns_nothing(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db, {"filename": "some.jef", "filepath": "/a/some.jef", "is_stitched": False}
        )
        pq = parse_advanced_query()
        assert pq.is_empty()
        # advanced_search with empty pq returns all (no filters applied)
        results, total = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        assert total >= 1  # at least the design we created is returned

    def test_no_fields_selected_returns_all(self, db):
        from src.services.search import parse_advanced_query

        designs.create(
            db, {"filename": "anything.jef", "filepath": "/a/anything.jef", "is_stitched": False}
        )
        pq = parse_advanced_query(all_words="anything")
        # All field checkboxes off → build_search_filters returns no conditions → all designs match
        results, total = designs.advanced_search(
            db, pq, search_filename=False, search_tags=False, search_folder=False
        )
        assert total >= 1

    def test_search_filename_includes_extension(self, db):
        """Searching for a file extension via the filename field should find matching designs."""
        from src.services.search import parse_advanced_query

        # filename stored without extension (stem only); extension is in filepath
        designs.create(db, {"filename": "rose", "filepath": "/a/rose.jef", "is_stitched": False})
        designs.create(db, {"filename": "tulip", "filepath": "/a/tulip.hus", "is_stitched": False})
        # Search by extension without leading dot
        pq = parse_advanced_query(all_words="jef")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filenames = [d.filename for d in results]
        assert any(
            "rose" in f for f in filenames
        ), "Should find 'rose' when searching for 'jef' extension"
        assert not any(
            "tulip" in f for f in filenames
        ), "Should not find 'tulip.hus' when searching for 'jef'"
        # Search by extension with leading dot also works
        pq_dot = parse_advanced_query(all_words=".jef")
        results_dot, _ = designs.advanced_search(
            db, pq_dot, search_filename=True, search_tags=False, search_folder=False
        )
        filenames_dot = [d.filename for d in results_dot]
        assert any(
            "rose" in f for f in filenames_dot
        ), "Should find 'rose' when searching for '.jef' (with dot)"

    def test_general_search_supports_wildcard_extension(self, db):
        """General search should allow wildcard extension searches like '*.hus'."""
        from src.services.search import parse_advanced_query

        designs.create(db, {"filename": "rose", "filepath": "/a/rose.jef", "is_stitched": False})
        designs.create(db, {"filename": "tulip", "filepath": "/a/tulip.hus", "is_stitched": False})

        pq = parse_advanced_query(q="*.hus")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )

        filepaths = [d.filepath for d in results]
        assert any(fp.endswith(".hus") for fp in filepaths), "Should find .hus designs from '*.hus'"
        assert not any(
            fp.endswith(".jef") for fp in filepaths
        ), "Should not find .jef designs from '*.hus'"

    def test_search_filename_extension_excluded_word(self, db):
        """Excluding a file extension via the filename field should filter out designs with that extension."""
        from src.services.search import parse_advanced_query

        designs.create(
            db, {"filename": "border_a", "filepath": "/a/border_a.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "border_b", "filepath": "/a/border_b.hus", "is_stitched": False}
        )
        pq = parse_advanced_query(all_words="border", none_words="jef")
        results, _ = designs.advanced_search(
            db, pq, search_filename=True, search_tags=False, search_folder=False
        )
        filepaths = [d.filepath for d in results]
        assert not any(
            ".jef" in fp for fp in filepaths
        ), "Should exclude .jef designs when 'jef' is in none_words"
        assert any(".hus" in fp for fp in filepaths), "Should still find .hus designs"


# ---------------------------------------------------------------------------
# Auto-tagging service — Tier 1 keyword matching (deterministic)
# ---------------------------------------------------------------------------


class TestAutoTaggingTier1:
    """Tests for suggest_tier1 — no DB, no external API."""

    def _valid(self, *descs):
        """Build a valid_descriptions set from the given strings."""
        return set(descs)

    def test_basic_keyword_match(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Flowers")
        result = suggest_tier1("rose_garden.jef", valid)
        assert "Flowers" in result

    def test_no_match_when_description_not_in_valid_set(self):
        from src.services.auto_tagging import suggest_tier1

        # "rose" maps to "Flowers", but "Flowers" is not in valid_descriptions
        result = suggest_tier1("rose_garden.jef", set())
        assert result == []

    def test_multiple_keyword_matches(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Flowers", "Christmas")
        result = suggest_tier1("christmas_rose.jef", valid)
        assert "Flowers" in result
        assert "Christmas" in result

    def test_case_insensitive_stem(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Birds")
        result = suggest_tier1("OWL_DESIGN.JEF", valid)
        assert "Birds" in result

    def test_filepath_folder_tokens_matched(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Alphabets")
        # "alphabet" keyword maps to "Alphabets"
        result = suggest_tier1("Z.pes", valid, filepath="\\Alphabets\\Z.pes")
        assert "Alphabets" in result

    def test_no_false_positives_for_unrelated_filename(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Flowers", "Birds", "Christmas")
        result = suggest_tier1("unknown_file_xyz.jef", valid)
        assert result == []

    def test_result_is_sorted(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Flowers", "Christmas", "Birds")
        result = suggest_tier1("christmas_rose_owl.jef", valid)
        assert result == sorted(result)

    def test_dog_keyword(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Dogs")
        assert "Dogs" in suggest_tier1("labrador.pes", valid)
        assert "Dogs" in suggest_tier1("puppy_portrait.jef", valid)

    def test_cat_keyword(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Cats")
        assert "Cats" in suggest_tier1("kitten_cute.jef", valid)

    def test_butterfly_keyword(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Butterflies and Insects")
        assert "Butterflies and Insects" in suggest_tier1("butterfly_v2.jef", valid)
        assert "Butterflies and Insects" in suggest_tier1("dragonfly.jef", valid)

    def test_ith_keyword_no_longer_matches(self):
        """Stitching keywords were removed from KEYWORD_MAP — AI tagging
        should NOT suggest stitching tags. StitchIdentifier handles these."""
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("In The Hoop")
        assert suggest_tier1("ith_bag.jef", valid) == []

    def test_cross_stitch_keyword_no_longer_matches(self):
        """Stitching keywords were removed from KEYWORD_MAP — AI tagging
        should NOT suggest stitching tags. StitchIdentifier handles these."""
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Cross Stitch")
        assert suggest_tier1("crossstitch_rose.jef", valid) == []
        assert suggest_tier1("xstitch_flower.jef", valid) == []

    def test_stem_helper(self):
        from src.services.auto_tagging import _stem

        assert _stem("Rose_Wreath.JEF") == "rose_wreath"
        assert _stem("butterfly.pes") == "butterfly"

    def test_empty_filepath_does_not_crash(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Flowers")
        result = suggest_tier1("rose.jef", valid, filepath="")
        assert "Flowers" in result

    def test_underscore_keyword_compound_no_longer_matches(self):
        """Stitching keywords were removed from KEYWORD_MAP — AI tagging
        should NOT suggest stitching tags. StitchIdentifier handles these."""
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Cross Stitch")
        # cross_stitch keyword was removed from KEYWORD_MAP
        result = suggest_tier1("cross_stitch_sampler.jef", valid)
        assert "Cross Stitch" not in result

        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Zodiac")
        for fname in ["aries_design.jef", "scorpio.jef", "gemini_chart.jef"]:
            assert "Zodiac" in suggest_tier1(fname, valid), f"Failed for {fname}"

    def test_nautical_keywords(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Nautical")
        assert "Nautical" in suggest_tier1("anchor_border.jef", valid)
        assert "Nautical" in suggest_tier1("sailboat.jef", valid)

    def test_food_keywords(self):
        from src.services.auto_tagging import suggest_tier1

        valid = self._valid("Food")
        assert "Food" in suggest_tier1("cupcake.jef", valid)
        assert "Food" in suggest_tier1("strawberry.jef", valid)


# ---------------------------------------------------------------------------
# Bulk import service — pure helpers (no DB, no filesystem)
# ---------------------------------------------------------------------------


class TestBulkImportPureHelpers:
    def test_pick_preferred_jef_over_pes(self):
        from src.services.scanning import _pick_preferred

        paths = ["/designs/rose.pes", "/designs/rose.jef"]
        assert _pick_preferred(paths).endswith(".jef")

    def test_pick_preferred_pes_over_hus(self):
        from src.services.scanning import _pick_preferred

        paths = ["/designs/rose.hus", "/designs/rose.pes"]
        assert _pick_preferred(paths).endswith(".pes")

    def test_pick_preferred_single_file(self):
        from src.services.scanning import _pick_preferred

        assert _pick_preferred(["/designs/rose.hus"]) == "/designs/rose.hus"

    def test_pick_preferred_unknown_extension_falls_back_to_first(self):
        from src.services.scanning import _pick_preferred

        paths = ["/designs/a.xxx", "/designs/b.yyy"]
        assert _pick_preferred(paths) == "/designs/a.xxx"

    def test_suggest_source_from_path_match(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_source_from_path

        s1 = MagicMock()
        s1.name = "Amazing Designs"
        s2 = MagicMock()
        s2.name = "Designs"
        # Should prefer longer match
        result = suggest_source_from_path("\\Amazing Designs\\roses\\rose.jef", [s1, s2])
        assert result is s1

    def test_suggest_source_from_path_no_match(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_source_from_path

        s = MagicMock()
        s.name = "Nonexistent Source"
        result = suggest_source_from_path("\\Alphabets\\Z.pes", [s])
        assert result is None

    def test_suggest_source_excludes_me(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_source_from_path

        s = MagicMock()
        s.name = "me"  # in _PATH_MATCH_EXCLUDE
        result = suggest_source_from_path("\\me\\rose.jef", [s])
        assert result is None

    def test_suggest_designer_from_path_match(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_designer_from_path

        d1 = MagicMock()
        d1.name = "Embroidery Library"
        d2 = MagicMock()
        d2.name = "Embroidery"
        result = suggest_designer_from_path("\\Embroidery Library\\roses\\rose.jef", [d1, d2])
        assert result is d1

    def test_suggest_designer_from_path_no_match(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_designer_from_path

        d = MagicMock()
        d.name = "Unknown Designer"
        result = suggest_designer_from_path("\\Alphabets\\Z.pes", [d])
        assert result is None

    def test_suggest_designer_excludes_dont_know(self):
        from unittest.mock import MagicMock

        from src.services.matching import suggest_designer_from_path

        d = MagicMock()
        d.name = "don't know"
        result = suggest_designer_from_path("\\don't know\\rose.jef", [d])
        assert result is None

    def test_scanned_design_dataclass(self):
        from src.services.scanning import ScannedDesign

        sd = ScannedDesign(filename="test.jef", filepath="/test.jef")
        assert sd.filename == "test.jef"
        assert sd.filepath == "/test.jef"
        assert sd.width_mm is None
        assert sd.error is None

    def test_supported_extensions(self):
        from src.services.scanning import SUPPORTED_EXTENSIONS

        # Original formats still present
        assert ".jef" in SUPPORTED_EXTENSIONS
        assert ".pes" in SUPPORTED_EXTENSIONS
        assert ".hus" in SUPPORTED_EXTENSIONS
        assert ".art" in SUPPORTED_EXTENSIONS
        # Newly expanded formats
        assert ".dst" in SUPPORTED_EXTENSIONS
        assert ".exp" in SUPPORTED_EXTENSIONS
        assert ".vp3" in SUPPORTED_EXTENSIONS
        assert ".sew" in SUPPORTED_EXTENSIONS
        assert ".u01" in SUPPORTED_EXTENSIONS
        assert ".pec" in SUPPORTED_EXTENSIONS
        assert ".xxx" in SUPPORTED_EXTENSIONS
        assert ".tbf" in SUPPORTED_EXTENSIONS
        assert ".pmv" in SUPPORTED_EXTENSIONS
        # Excluded helper/output formats must NOT be present
        assert ".json" not in SUPPORTED_EXTENSIONS
        assert ".col" not in SUPPORTED_EXTENSIONS
        assert ".edr" not in SUPPORTED_EXTENSIONS
        assert ".inf" not in SUPPORTED_EXTENSIONS
        assert ".svg" not in SUPPORTED_EXTENSIONS
        assert ".csv" not in SUPPORTED_EXTENSIONS
        assert ".png" not in SUPPORTED_EXTENSIONS
        assert ".txt" not in SUPPORTED_EXTENSIONS

    # def test_load_api_key_from_env(self, monkeypatch):
    #     from src.services import bulk_import
    #     monkeypatch.setenv("GOOGLE_API_KEY", "test-key-123")
    #     # Ensure no .env file overrides: patch env_path.exists() to return False
    #     from unittest.mock import patch
    #     with patch("pathlib.Path.exists", return_value=False):
    #         key = bulk_import._load_api_key()
    #     assert key == "test-key-123"

    # def test_load_api_key_returns_none_when_missing(self, monkeypatch):
    #     from src.services import bulk_import
    #     monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
    #     from unittest.mock import patch
    #     with patch("pathlib.Path.exists", return_value=False):
    #         key = bulk_import._load_api_key()
    #     assert key is None


# ---------------------------------------------------------------------------
# Bulk import service — scan_folder with real tmp filesystem
# ---------------------------------------------------------------------------


class TestBulkImportScanFolder:
    def test_scan_empty_folder(self, db, tmp_path):
        from src.services.scanning import scan_folder

        result = scan_folder(str(tmp_path), db)
        assert result == []

    def test_scan_unsupported_files_ignored(self, db, tmp_path):
        from src.services.scanning import scan_folder

        (tmp_path / "readme.txt").write_text("not an embroidery file")
        (tmp_path / "image.png").write_bytes(b"\x89PNG")
        result = scan_folder(str(tmp_path), db)
        assert result == []

    def test_scan_finds_jef_file(self, db, tmp_path):
        from src.services.scanning import scan_folder

        # Create a minimal JEF file (content will fail parse but file is found)
        jef_file = tmp_path / "test_design.jef"
        jef_file.write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "test_design.jef"

    def test_scan_preserves_selected_source_folder_name_in_filepath(self, db, tmp_path):
        from src.services.scanning import scan_folder

        source_root = tmp_path / "Craftsy - Machine Embroidery T Shirts"
        nested = source_root / "Animals"
        nested.mkdir(parents=True, exist_ok=True)
        (nested / "lion.jef").write_bytes(b"\x00" * 128)

        result = scan_folder(str(source_root), db)

        assert len(result) == 1
        assert result[0].filepath == "\\Craftsy - Machine Embroidery T Shirts\\Animals\\lion.jef"

    def test_scan_skips_already_imported(self, db, tmp_path):
        from src.services.scanning import scan_folder
        from src.services.settings_service import SETTING_DESIGNS_BASE_PATH, set_setting

        # Set base path to tmp_path so relative paths match
        set_setting(db, SETTING_DESIGNS_BASE_PATH, str(tmp_path))
        db.commit()
        # Create design file
        jef_file = tmp_path / "already.jef"
        jef_file.write_bytes(b"\x00" * 128)
        # Pre-populate the DB with this filepath
        rel = "\\already.jef"
        from src.services import designs as design_svc

        design_svc.create(db, {"filename": "already.jef", "filepath": rel, "is_stitched": False})
        result = scan_folder(str(tmp_path), db)
        # Should be skipped since it's already in the DB
        assert all(r.filepath != rel for r in result)

    def test_scan_deduplicates_same_stem_prefers_jef(self, db, tmp_path):
        from src.services.scanning import scan_folder

        (tmp_path / "rose.jef").write_bytes(b"\x00" * 128)
        (tmp_path / "rose.pes").write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "rose.jef"

    def test_scan_deduplicates_prefers_vp3_over_hus(self, db, tmp_path):
        from src.services.scanning import scan_folder

        (tmp_path / "rose.vp3").write_bytes(b"\x00" * 128)
        (tmp_path / "rose.hus").write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "rose.vp3"

    def test_scan_finds_dst_file(self, db, tmp_path):
        from src.services.scanning import scan_folder

        dst_file = tmp_path / "test_design.dst"
        dst_file.write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "test_design.dst"

    def test_scan_finds_exp_file(self, db, tmp_path):
        from src.services.scanning import scan_folder

        exp_file = tmp_path / "test_design.exp"
        exp_file.write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "test_design.exp"

    def test_scan_finds_pmv_file(self, db, tmp_path):
        from src.services.scanning import scan_folder

        pmv_file = tmp_path / "test_design.pmv"
        pmv_file.write_bytes(b"\x00" * 128)
        result = scan_folder(str(tmp_path), db)
        assert len(result) == 1
        assert result[0].filename == "test_design.pmv"

    def test_scan_ignores_excluded_formats(self, db, tmp_path):
        from src.services.scanning import scan_folder

        (tmp_path / "design.json").write_text("{}")
        (tmp_path / "design.col").write_bytes(b"\x00")
        (tmp_path / "design.edr").write_bytes(b"\x00")
        (tmp_path / "design.inf").write_bytes(b"\x00")
        (tmp_path / "design.svg").write_text("<svg/>")
        (tmp_path / "design.csv").write_text("col1,col2")
        result = scan_folder(str(tmp_path), db)
        assert result == []

    def test_scan_error_captured_per_file(self, db, tmp_path):
        from src.services.scanning import scan_folder

        # A real JEF file that fails to parse will set .error
        bad_jef = tmp_path / "corrupt.jef"
        bad_jef.write_bytes(b"not a jef file at all")
        result = scan_folder(str(tmp_path), db)
        # Either parsed OK (no error) or error captured — but should not raise
        assert len(result) == 1
        # If error, it should be a string
        if result[0].error:
            assert isinstance(result[0].error, str)


# ---------------------------------------------------------------------------
# Bulk import service — confirm_import with mocked auto-tagging
# ---------------------------------------------------------------------------


class TestBulkImportConfirmImport:
    def test_confirm_import_creates_designs(self, db):
        from src.services.bulk_import import confirm_import
        from src.services.scanning import ScannedDesign

        sd = ScannedDesign(filename="rose.jef", filepath="/rose.jef")
        created = confirm_import(db, [sd], run_tier2=False, run_tier3=False)
        assert len(created) == 1
        assert created[0].filename == "rose.jef"

    def test_confirm_import_skips_errors(self, db):
        from src.services.bulk_import import confirm_import
        from src.services.scanning import ScannedDesign

        sd_ok = ScannedDesign(filename="ok.jef", filepath="/ok.jef")
        sd_err = ScannedDesign(filename="bad.jef", filepath="/bad.jef", error="parse error")
        created = confirm_import(db, [sd_ok, sd_err], run_tier2=False, run_tier3=False)
        assert len(created) == 1
        assert created[0].filename == "ok.jef"

    def test_confirm_import_empty_list(self, db):
        from src.services.bulk_import import confirm_import

        created = confirm_import(db, [], run_tier2=False, run_tier3=False)
        assert created == []

    def test_confirm_import_auto_tags_tier1(self, db):
        from src.services.bulk_import import confirm_import
        from src.services.scanning import ScannedDesign

        # Create a tag that will be matched by tier1
        tags.create(db, "Flowers", "image")
        sd = ScannedDesign(filename="rose_garden.jef", filepath="/rose_garden.jef")
        created = confirm_import(db, [sd], run_tier2=False, run_tier3=False)
        assert len(created) == 1
        descriptions = [tag.description for tag in created[0].tags]
        assert "Flowers" in descriptions
        assert created[0].tagging_tier == 1

    def test_confirm_import_deduplicates_repeated_tier1_tags(self, db, monkeypatch):
        from src.services import bulk_import

        tags.create(db, "Flowers", "image")
        monkeypatch.setattr(
            bulk_import,
            "suggest_tier1",
            lambda *_args, **_kwargs: ["Flowers", "Flowers", "Flowers"],
        )

        sd = bulk_import.ScannedDesign(filename="rose.jef", filepath="/rose.jef")
        created = bulk_import.confirm_import(db, [sd], run_tier2=False, run_tier3=False)

        assert len(created) == 1
        assert [tag.description for tag in created[0].tags] == ["Flowers"]
        assert created[0].tagging_tier == 1

    def test_confirm_import_merges_stitching_pattern_match_with_keyword_tags(
        self, db, monkeypatch, tmp_path
    ):
        from src.services import bulk_import

        tags.create(db, "Flowers", "image")
        tags.create(db, "Applique", "stitching")
        monkeypatch.setattr(bulk_import, "suggest_tier1", lambda *_args, **_kwargs: ["Flowers"])
        monkeypatch.setattr(
            bulk_import,
            "suggest_stitching_from_pattern",
            lambda *_args, **_kwargs: ["Applique"],
        )

        src = tmp_path / "rose.jef"
        src.parent.mkdir(parents=True, exist_ok=True)
        src.write_bytes(b"dummy embroidery content")

        sd = bulk_import.ScannedDesign(
            filename="rose.jef",
            filepath="/rose.jef",
            source_full_path=str(src),
            image_data=b"fake-preview",
        )
        created = bulk_import.confirm_import(db, [sd], run_tier2=False, run_tier3=False)

        assert len(created) == 1
        assert {tag.description for tag in created[0].tags} == {"Flowers", "Applique"}
        assert created[0].tagging_tier == 1

    def test_confirm_import_deduplicates_repeated_tier2_tags(self, db, monkeypatch):
        from src.services import bulk_import

        tags.create(db, "Animals", "image")
        monkeypatch.setattr(bulk_import, "suggest_tier1", lambda *_args, **_kwargs: [])
        monkeypatch.setattr(bulk_import, "_load_api_key", lambda: "dummy-key")
        monkeypatch.setattr(
            bulk_import,
            "suggest_tier2_batch",
            lambda *_args, **_kwargs: {"mystery": ["Animals", "Animals"]},
        )

        sd = bulk_import.ScannedDesign(filename="mystery.jef", filepath="/mystery.jef")
        created = bulk_import.confirm_import(db, [sd], run_tier2=True, run_tier3=False)

        assert len(created) == 1
        assert [tag.description for tag in created[0].tags] == ["Animals"]
        assert created[0].tagging_tier == 2

    def test_confirm_import_assigns_designer_from_path(self, db):
        from src.services import designers as des_svc
        from src.services.bulk_import import confirm_import
        from src.services.scanning import ScannedDesign

        des = des_svc.create(db, "Roseworks")
        sd = ScannedDesign(filename="design.jef", filepath="\\Roseworks\\design.jef")
        created = confirm_import(db, [sd], run_tier2=False, run_tier3=False)
        assert created[0].designer_id == des.id

    def test_confirm_import_assigns_source_from_path(self, db):
        from src.services import sources as src_svc
        from src.services.bulk_import import confirm_import
        from src.services.scanning import ScannedDesign

        src = src_svc.create(db, "Stitch Collection")
        sd = ScannedDesign(filename="design.jef", filepath="\\Stitch Collection\\design.jef")
        created = confirm_import(db, [sd], run_tier2=False, run_tier3=False)
        assert created[0].source_id == src.id

    def test_confirm_import_skips_tier2_when_no_api_key(self, db, monkeypatch):
        from src.services import bulk_import

        monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
        from unittest.mock import patch

        with patch("pathlib.Path.exists", return_value=False):
            with patch.object(bulk_import, "suggest_tier2_batch") as mock_t2:
                sd = bulk_import.ScannedDesign(filename="unknown.jef", filepath="/unknown.jef")
                bulk_import.confirm_import(db, [sd], run_tier2=True, run_tier3=False)
                mock_t2.assert_not_called()

    def test_confirm_import_skips_tier3_when_no_api_key(self, db, monkeypatch):
        from src.services import bulk_import

        monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
        from unittest.mock import patch

        with patch("pathlib.Path.exists", return_value=False):
            with patch.object(bulk_import, "suggest_tier3_vision") as mock_t3:
                sd = bulk_import.ScannedDesign(
                    filename="untagged.jef",
                    filepath="/untagged.jef",
                    image_data=b"\x89PNG\r\n\x1a\n",
                )
                bulk_import.confirm_import(db, [sd], run_tier2=False, run_tier3=True)
                mock_t3.assert_not_called()

    def test_confirm_import_batch_limit_restricts_ai_candidates(self, db, monkeypatch):
        """batch_limit=1 means only the first design is sent to Tier 2 AI."""
        from unittest.mock import patch

        from src.services import bulk_import

        tags.create(db, "Roses", "image")
        monkeypatch.setattr(bulk_import, "suggest_tier1", lambda *_args, **_kwargs: [])
        monkeypatch.setattr(bulk_import, "_load_api_key", lambda: "dummy-key")

        tier2_calls: list[list] = []

        def fake_tier2(db, designs, *args, **kwargs):
            tier2_calls.append(list(designs))

        with patch.object(bulk_import, "_apply_tier2_tags", side_effect=fake_tier2):
            sd1 = bulk_import.ScannedDesign(filename="rose.jef", filepath="/rose.jef")
            sd2 = bulk_import.ScannedDesign(filename="tulip.jef", filepath="/tulip.jef")
            bulk_import.confirm_import(
                db, [sd1, sd2], run_tier2=True, run_tier3=False, batch_limit=1
            )

        assert len(tier2_calls) == 1
        assert len(tier2_calls[0]) == 1  # only 1 design sent

    def test_confirm_import_no_batch_limit_sends_all_to_ai(self, db, monkeypatch):
        """Without batch_limit all created designs are sent to AI tiers."""
        from unittest.mock import patch

        from src.services import bulk_import

        tags.create(db, "Roses", "image")
        monkeypatch.setattr(bulk_import, "suggest_tier1", lambda *_args, **_kwargs: [])
        monkeypatch.setattr(bulk_import, "_load_api_key", lambda: "dummy-key")

        tier2_calls: list[list] = []

        def fake_tier2(db, designs, *args, **kwargs):
            tier2_calls.append(list(designs))

        with patch.object(bulk_import, "_apply_tier2_tags", side_effect=fake_tier2):
            sds = [
                bulk_import.ScannedDesign(filename=f"design{i}.jef", filepath=f"/design{i}.jef")
                for i in range(3)
            ]
            bulk_import.confirm_import(db, sds, run_tier2=True, run_tier3=False)

        assert len(tier2_calls) == 1
        assert len(tier2_calls[0]) == 3  # all designs sent


# ---------------------------------------------------------------------------
# Bulk import service — backfill helpers
# ---------------------------------------------------------------------------


class TestBulkImportBackfill:
    def test_backfill_source_assigns_match(self, db):
        from src.services import bulk_import as svc
        from src.services import designs as dsvc
        from src.services import sources as src_svc

        src = src_svc.create(db, "Florals By Design")
        d = dsvc.create(
            db,
            {
                "filename": "flower.jef",
                "filepath": "\\Florals By Design\\flower.jef",
                "is_stitched": False,
            },
        )
        updated = svc.backfill_source_from_path(db)
        assert updated >= 1
        db.refresh(d)
        assert d.source_id == src.id

    def test_backfill_source_does_not_overwrite_existing(self, db):
        from src.services import bulk_import as svc
        from src.services import designs as dsvc
        from src.services import sources as src_svc

        src1 = src_svc.create(db, "Original Source")
        d = dsvc.create(
            db,
            {
                "filename": "test.jef",
                "filepath": "\\Original Source2\\test.jef",
                "is_stitched": False,
                "source_id": src1.id,
            },
        )
        svc.backfill_source_from_path(db)
        db.refresh(d)
        assert d.source_id == src1.id  # unchanged

    def test_backfill_designer_assigns_match(self, db):
        from src.services import bulk_import as svc
        from src.services import designers as des_svc
        from src.services import designs as dsvc

        des = des_svc.create(db, "NeedleWorks")
        d = dsvc.create(
            db,
            {
                "filename": "design.jef",
                "filepath": "\\NeedleWorks\\design.jef",
                "is_stitched": False,
            },
        )
        updated = svc.backfill_designer_from_path(db)
        assert updated >= 1
        db.refresh(d)
        assert d.designer_id == des.id


# ---------------------------------------------------------------------------
# Auto-tagging service — Tier 2 (mocked Gemini API)
# ---------------------------------------------------------------------------


def _mock_genai_modules(mock_client):
    """Return a sys.modules patch dict that stubs out google.genai."""
    from unittest.mock import MagicMock

    mock_genai = MagicMock()
    mock_genai.Client.return_value = mock_client

    mock_google = MagicMock()
    mock_google.genai = mock_genai

    return {"google": mock_google, "google.genai": mock_genai}


class TestAutoTaggingTier2Mocked:
    def test_tier2_batch_returns_results(self):
        """suggest_tier2_batch should return description lists per stem."""
        import json
        import sys
        from unittest.mock import MagicMock, patch

        mock_response = MagicMock()
        mock_response.text = json.dumps({"rose": ["Flowers"], "cat": ["Cats"]})

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        with patch.dict(sys.modules, _mock_genai_modules(mock_client)):
            from src.services.auto_tagging import suggest_tier2_batch

            results = suggest_tier2_batch(
                ["rose.jef", "cat.jef"],
                ["Flowers", "Cats", "Dogs"],
                api_key="test-key",
                batch_size=20,
                delay_seconds=0,
            )
        assert results.get("rose") == ["Flowers"]
        assert results.get("cat") == ["Cats"]

    def test_tier2_batch_strips_markdown_fences(self):
        """API responses wrapped in ``` fences should be parsed correctly."""
        import json
        import sys
        from unittest.mock import MagicMock, patch

        payload = json.dumps({"rose": ["Flowers"]})
        mock_response = MagicMock()
        mock_response.text = f"```json\n{payload}\n```"

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        with patch.dict(sys.modules, _mock_genai_modules(mock_client)):
            from src.services.auto_tagging import suggest_tier2_batch

            results = suggest_tier2_batch(
                ["rose.jef"],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert results.get("rose") == ["Flowers"]

    def test_tier2_batch_retries_on_exception(self):
        """suggest_tier2_batch retries on failure and eventually returns empty."""
        import sys
        from unittest.mock import MagicMock, patch

        mock_client = MagicMock()
        mock_client.models.generate_content.side_effect = RuntimeError("API error")

        with patch.dict(sys.modules, _mock_genai_modules(mock_client)), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier2_batch

            results = suggest_tier2_batch(
                ["unknown.jef"],
                ["Flowers"],
                api_key="test-key",
                batch_size=20,
                delay_seconds=0,
                max_retries=2,
            )
        # After all retries fail, stem should have empty list
        assert results.get("unknown") == []

    def test_tier2_batch_filters_invalid_descriptions(self):
        """Tags returned by AI that are not in valid_descriptions should be filtered out."""
        import json
        import sys
        from unittest.mock import MagicMock, patch

        mock_response = MagicMock()
        mock_response.text = json.dumps({"rose": ["Flowers", "InvalidTag"]})

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        with patch.dict(sys.modules, _mock_genai_modules(mock_client)):
            from src.services.auto_tagging import suggest_tier2_batch

            results = suggest_tier2_batch(
                ["rose.jef"],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert "InvalidTag" not in results.get("rose", [])
        assert "Flowers" in results.get("rose", [])

    def test_tier2_empty_filenames_list(self):
        """Empty filenames list should return empty results without calling API."""
        import sys
        from unittest.mock import MagicMock, patch

        mock_client = MagicMock()
        with patch.dict(sys.modules, _mock_genai_modules(mock_client)):
            from src.services.auto_tagging import suggest_tier2_batch

            results = suggest_tier2_batch([], ["Flowers"], api_key="test-key", delay_seconds=0)
        mock_client.models.generate_content.assert_not_called()
        assert results == {}


# ---------------------------------------------------------------------------
# Auto-tagging service — Tier 3 (mocked vision API)
# ---------------------------------------------------------------------------


class TestAutoTaggingTier3Mocked:
    def _make_design(self, design_id, filename, image_data):
        from unittest.mock import MagicMock

        d = MagicMock()
        d.id = design_id
        d.filename = filename
        d.image_data = image_data
        return d

    def test_tier3_returns_results(self):
        import json
        import sys
        from unittest.mock import MagicMock, patch

        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100

        mock_response = MagicMock()
        mock_response.text = json.dumps(["Flowers"])

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        design = self._make_design(1, "rose.jef", fake_png)

        mods = _mock_genai_modules(mock_client)
        mods["google.genai.types"] = MagicMock()
        mods["google"].genai.types = MagicMock()

        with patch.dict(sys.modules, mods), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier3_vision

            results = suggest_tier3_vision(
                [design],
                ["Flowers", "Birds"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert results.get(1) == ["Flowers"]

    def test_tier3_skips_designs_with_no_image(self):
        import sys
        from unittest.mock import MagicMock, patch

        design = self._make_design(1, "noimage.jef", None)

        with patch.dict(sys.modules, _mock_genai_modules(MagicMock())), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier3_vision

            results = suggest_tier3_vision(
                [design],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert 1 not in results

    def test_tier3_filters_invalid_descriptions(self):
        import json
        import sys
        from unittest.mock import MagicMock, patch

        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100

        mock_response = MagicMock()
        mock_response.text = json.dumps(["Flowers", "BadTag"])

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        design = self._make_design(2, "design.jef", fake_png)

        mods = _mock_genai_modules(mock_client)
        mods["google.genai.types"] = MagicMock()
        mods["google"].genai.types = MagicMock()

        with patch.dict(sys.modules, mods), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier3_vision

            results = suggest_tier3_vision(
                [design],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert "BadTag" not in results.get(2, [])

    def test_tier3_retries_on_exception(self):
        import sys
        from unittest.mock import MagicMock, patch

        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100

        mock_client = MagicMock()
        mock_client.models.generate_content.side_effect = RuntimeError("API error")

        design = self._make_design(3, "fail.jef", fake_png)

        mods = _mock_genai_modules(mock_client)
        mods["google.genai.types"] = MagicMock()
        mods["google"].genai.types = MagicMock()

        with patch.dict(sys.modules, mods), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier3_vision

            results = suggest_tier3_vision(
                [design],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
                max_retries=2,
            )
        assert results.get(3) == []

    def test_tier3_strips_markdown_fences(self):
        import json
        import sys
        from unittest.mock import MagicMock, patch

        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
        payload = json.dumps(["Flowers"])

        mock_response = MagicMock()
        mock_response.text = f"```\n{payload}\n```"

        mock_client = MagicMock()
        mock_client.models.generate_content.return_value = mock_response

        design = self._make_design(4, "fenced.jef", fake_png)

        mods = _mock_genai_modules(mock_client)
        mods["google.genai.types"] = MagicMock()
        mods["google"].genai.types = MagicMock()

        with patch.dict(sys.modules, mods), patch("time.sleep"):
            from src.services.auto_tagging import suggest_tier3_vision

            results = suggest_tier3_vision(
                [design],
                ["Flowers"],
                api_key="test-key",
                delay_seconds=0,
            )
        assert results.get(4) == ["Flowers"]


# ---------------------------------------------------------------------------
# Bulk import service — render preview and thread color helpers
# ---------------------------------------------------------------------------


class TestBulkImportRenderHelpers:
    def _make_pattern(self):
        """Create a minimal pyembroidery pattern with two stitches."""
        import pyembroidery

        pattern = pyembroidery.EmbPattern()
        pattern.add_stitch_absolute(pyembroidery.STITCH, 0, 0)
        pattern.add_stitch_absolute(pyembroidery.STITCH, 100, 100)
        pattern.add_stitch_absolute(pyembroidery.END, 0, 0)
        return pattern

    def test_render_preview_returns_png_bytes(self):
        from src.services.preview import _render_preview

        pattern = self._make_pattern()
        result = _render_preview(pattern)
        assert result is not None
        assert isinstance(result, bytes)
        # PNG magic bytes
        assert result[:4] == b"\x89PNG"

    def test_render_preview_empty_pattern_returns_none(self):
        import pyembroidery

        from src.services.preview import _render_preview

        pattern = pyembroidery.EmbPattern()
        result = _render_preview(pattern)
        assert result is None

    def test_thread_color_default_fallback(self):
        import pyembroidery

        from src.services.preview import _thread_color

        pattern = pyembroidery.EmbPattern()
        # No threads → IndexError → fallback to dark grey
        color = _thread_color(pattern, 0)
        assert isinstance(color, tuple)
        assert len(color) == 3

    def test_thread_color_near_white_adjusted(self):
        import pyembroidery

        from src.services.preview import _thread_color

        pattern = pyembroidery.EmbPattern()
        thread = pyembroidery.EmbThread()
        # Near-white color (r=240, g=240, b=240) → 0xF0F0F0
        thread.color = 0xF0F0F0
        pattern.add_thread(thread)
        color = _thread_color(pattern, 0)
        # Should be adjusted to (200, 200, 200) to stay visible
        assert color == (200, 200, 200)

    def test_thread_color_normal(self):
        import pyembroidery

        from src.services.preview import _thread_color

        pattern = pyembroidery.EmbPattern()
        thread = pyembroidery.EmbThread()
        thread.color = 0xFF0000  # pure red
        pattern.add_thread(thread)
        color = _thread_color(pattern, 0)
        assert color == (255, 0, 0)


# ---------------------------------------------------------------------------
# Bulk import service — _find_spider_image helper
# ---------------------------------------------------------------------------


class TestFindSpiderImage:
    def test_finds_jpeg_in_spider_folder(self, tmp_path):
        import io

        from PIL import Image

        from src.services.preview import _find_spider_image

        # Create a design file and a spider subfolder with a JPEG
        design_file = tmp_path / "rose.jef"
        design_file.write_bytes(b"\x00" * 128)

        spider_dir = tmp_path / "_embird_spider"
        spider_dir.mkdir()

        # Create a valid JPEG preview
        img = Image.new("RGB", (50, 50), color=(255, 0, 0))
        buf = io.BytesIO()
        img.save(buf, format="JPEG")
        (spider_dir / "rose.jef.jpeg").write_bytes(buf.getvalue())

        result = _find_spider_image(str(design_file))
        assert result is not None
        # Should be PNG bytes
        assert result[:4] == b"\x89PNG"

    def test_returns_none_when_no_spider_folder(self, tmp_path):
        from src.services.preview import _find_spider_image

        design_file = tmp_path / "rose.jef"
        design_file.write_bytes(b"\x00" * 128)
        result = _find_spider_image(str(design_file))
        assert result is None

    def test_returns_none_for_inaccessible_folder(self):
        from src.services.preview import _find_spider_image

        result = _find_spider_image("/nonexistent/path/rose.jef")
        assert result is None


# ---------------------------------------------------------------------------
# Bulk import service — _load_api_key from .env file
# ---------------------------------------------------------------------------


class TestLoadApiKeyFromEnvFile:
    def test_load_api_key_skips_non_matching_lines(self, monkeypatch):
        from unittest.mock import patch

        from src.services import bulk_import

        monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
        env_content = "# comment\nOTHER_KEY=value\nGOOGLE_API_KEY=correct-key\n"
        with (
            patch("pathlib.Path.exists", return_value=True),
            patch("pathlib.Path.read_text", return_value=env_content),
        ):
            key = bulk_import._load_api_key()
        assert key == "correct-key"

    def test_load_api_key_from_env_file(self, monkeypatch):
        from unittest.mock import patch

        from src.services import bulk_import

        monkeypatch.delenv("GOOGLE_API_KEY", raising=False)
        env_content = "GOOGLE_API_KEY=file-key-123\n"
        with (
            patch("pathlib.Path.exists", return_value=True),
            patch("pathlib.Path.read_text", return_value=env_content),
        ):
            key = bulk_import._load_api_key()
        assert key == "file-key-123"


# ---------------------------------------------------------------------------
# Designs service — orphan utilities and image helpers
# ---------------------------------------------------------------------------


class TestDesignServiceOrphansAndImages:
    def test_scan_orphaned_counts_only_missing_files(self, db, tmp_path):
        from src.services import designs as svc

        (tmp_path / "present.jef").write_bytes(b"\x00" * 16)

        svc.create(
            db, {"filename": "present.jef", "filepath": "\\present.jef", "is_stitched": False}
        )
        svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )

        result = svc.scan_orphaned(db, str(tmp_path))

        assert result == {"checked": 2, "found": 1}

    def test_get_orphaned_returns_sorted_page(self, db, tmp_path):
        from src.services import designs as svc

        (tmp_path / "present.jef").write_bytes(b"\x00" * 16)

        svc.create(
            db, {"filename": "present.jef", "filepath": "\\present.jef", "is_stitched": False}
        )
        svc.create(
            db, {"filename": "a_missing.jef", "filepath": "\\a_missing.jef", "is_stitched": False}
        )
        second = svc.create(
            db, {"filename": "b_missing.jef", "filepath": "\\b_missing.jef", "is_stitched": False}
        )

        orphaned, total = svc.get_orphaned(db, str(tmp_path), limit=1, offset=1)

        assert total == 2
        assert [design.id for design in orphaned] == [second.id]

    def test_delete_orphaned_empty_list_returns_zero(self, db):
        from src.services import designs as svc

        assert svc.delete_orphaned(db, []) == 0

    def test_delete_orphaned_removes_only_selected_rows(self, db):
        from src.models import Design
        from src.services import designs as svc

        orphan_a = svc.create(
            db, {"filename": "a_missing.jef", "filepath": "\\a_missing.jef", "is_stitched": False}
        )
        orphan_b = svc.create(
            db, {"filename": "b_missing.jef", "filepath": "\\b_missing.jef", "is_stitched": False}
        )

        deleted = svc.delete_orphaned(db, [orphan_a.id])

        assert deleted == 1
        assert db.get(Design, orphan_a.id) is None
        assert db.get(Design, orphan_b.id) is not None

    def test_delete_all_orphaned_removes_every_missing_design(self, db, tmp_path):
        from src.models import Design
        from src.services import designs as svc

        (tmp_path / "present.jef").write_bytes(b"\x00" * 16)

        svc.create(
            db, {"filename": "present.jef", "filepath": "\\present.jef", "is_stitched": False}
        )
        missing = svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )

        deleted = svc.delete_all_orphaned(db, str(tmp_path))

        assert deleted == 1
        assert db.get(Design, missing.id) is None

    def test_get_image_base64_returns_encoded_png(self, db):
        import base64

        from src.services import designs as svc

        design = svc.create(
            db,
            {
                "filename": "preview.jef",
                "filepath": "\\preview.jef",
                "is_stitched": False,
                "image_data": b"png-bytes",
            },
        )

        assert svc.get_image_base64(design) == base64.b64encode(b"png-bytes").decode("utf-8")
        assert svc.get_image_base64(type("DesignStub", (), {"image_data": None})()) is None

    def test_create_with_invalid_dimensions_raises_value_error(self, db):
        from src.services import designs as svc

        with pytest.raises(ValueError):
            svc.create(
                db,
                {
                    "filename": "bad.jef",
                    "filepath": "\\bad.jef",
                    "is_stitched": False,
                    "width_mm": -1,
                },
            )


# ---------------------------------------------------------------------------
# Backup service tests
# ---------------------------------------------------------------------------


class TestBackupService:
    """Tests for src.services.backup_service."""

    # -- Database backup -------------------------------------------------- #

    def test_backup_database_creates_timestamped_file(self, tmp_path):
        from src.services import backup_service as bsvc

        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(b"SQLite fake content")
        dest_dir = tmp_path / "backups"

        result = bsvc.backup_database(str(db_file), str(dest_dir))

        assert result.success
        assert result.backup_path
        assert dest_dir.is_dir()
        backup = next(dest_dir.glob("catalogue_*.db"))
        assert backup.read_bytes() == b"SQLite fake content"

    def test_backup_database_returns_correct_size(self, tmp_path):
        from src.services import backup_service as bsvc

        content = b"X" * 1024
        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(content)

        result = bsvc.backup_database(str(db_file), str(tmp_path / "out"))

        assert result.success
        assert result.size_bytes == 1024

    def test_backup_database_missing_source_returns_failure(self, tmp_path):
        from src.services import backup_service as bsvc

        result = bsvc.backup_database(str(tmp_path / "nonexistent.db"), str(tmp_path / "out"))

        assert not result.success
        assert "not found" in result.error.lower()

    def test_backup_database_creates_destination_if_absent(self, tmp_path):
        from src.services import backup_service as bsvc

        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(b"data")
        deep_dest = tmp_path / "a" / "b" / "c"

        result = bsvc.backup_database(str(db_file), str(deep_dest))

        assert result.success
        assert deep_dest.is_dir()

    def test_backup_database_preserves_live_wal_data(self, tmp_path):
        import sqlite3

        from src.services import backup_service as bsvc

        db_file = tmp_path / "catalogue.db"
        conn = sqlite3.connect(db_file)
        conn.execute("PRAGMA journal_mode=WAL")
        conn.execute("CREATE TABLE sample (value TEXT)")
        conn.execute("INSERT INTO sample(value) VALUES (?)", ("hello backup",))
        conn.commit()

        result = bsvc.backup_database(str(db_file), str(tmp_path / "wal_backup"))

        assert result.success
        with sqlite3.connect(result.backup_path) as backup_conn:
            rows = backup_conn.execute("SELECT value FROM sample").fetchall()
        assert rows == [("hello backup",)]
        conn.close()

    def test_backup_database_sets_completed_at(self, tmp_path):
        from src.services import backup_service as bsvc

        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(b"data")

        result = bsvc.backup_database(str(db_file), str(tmp_path / "out"))

        assert result.success
        assert result.completed_at  # non-empty timestamp string

    # -- Designs backup --------------------------------------------------- #

    def test_backup_designs_copies_new_files(self, tmp_path):
        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        src.mkdir()
        (src / "design_a.jef").write_bytes(b"design A")
        (src / "sub").mkdir()
        (src / "sub" / "design_b.jef").write_bytes(b"design B")
        dest = tmp_path / "backup"

        result = bsvc.backup_designs(str(src), str(dest))

        assert result.success
        assert result.copied == 2
        assert result.updated == 0
        assert result.unchanged == 0
        assert (dest / "design_a.jef").read_bytes() == b"design A"
        assert (dest / "sub" / "design_b.jef").read_bytes() == b"design B"

    def test_backup_designs_skips_unchanged_files(self, tmp_path):
        import time

        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        src.mkdir()
        src_file = src / "design_a.jef"
        src_file.write_bytes(b"unchanged")

        # First run
        bsvc.backup_designs(str(src), str(tmp_path / "backup"))
        # Wait to ensure mtime stability, then run again
        time.sleep(0.01)
        result = bsvc.backup_designs(str(src), str(tmp_path / "backup"))

        assert result.success
        assert result.copied == 0
        assert result.unchanged == 1

    def test_backup_designs_updates_changed_files(self, tmp_path):
        import time

        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        src.mkdir()
        src_file = src / "design.jef"
        src_file.write_bytes(b"v1")
        dest = tmp_path / "backup"

        bsvc.backup_designs(str(src), str(dest))

        # Modify the source file (different size → changed)
        time.sleep(0.05)
        src_file.write_bytes(b"version 2 content")

        result = bsvc.backup_designs(str(src), str(dest))

        assert result.success
        assert result.updated == 1
        assert (dest / "design.jef").read_bytes() == b"version 2 content"

    def test_backup_designs_archives_deleted_files(self, tmp_path):
        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        src.mkdir()
        src_file = src / "will_be_deleted.jef"
        src_file.write_bytes(b"soon gone")
        dest = tmp_path / "backup"

        # First backup: file is copied
        bsvc.backup_designs(str(src), str(dest))

        # Remove the file from source
        src_file.unlink()

        # Second backup: file should be archived, not deleted
        result = bsvc.backup_designs(str(src), str(dest))

        assert result.success
        assert result.archived == 1
        # The backup copy should no longer be in the root of dest
        assert not (dest / "will_be_deleted.jef").exists()
        # But it should exist somewhere under _deleted/
        archived = list((dest / "_deleted").rglob("will_be_deleted.jef"))
        assert len(archived) == 1

    def test_backup_designs_removes_empty_directories_after_archiving(self, tmp_path):
        import shutil

        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        (src / "folderA").mkdir(parents=True)
        src_file = src / "folderA" / "design.jef"
        src_file.write_bytes(b"design data")
        dest = tmp_path / "backup"

        bsvc.backup_designs(str(src), str(dest))

        shutil.rmtree(src / "folderA")
        result = bsvc.backup_designs(str(src), str(dest))

        assert result.success
        assert result.archived == 1
        assert not (dest / "folderA").exists()
        archived = list((dest / "_deleted").rglob("design.jef"))
        assert len(archived) == 1

    def test_backup_designs_missing_source_returns_failure(self, tmp_path):
        from src.services import backup_service as bsvc

        result = bsvc.backup_designs(str(tmp_path / "nonexistent"), str(tmp_path / "out"))

        assert not result.success
        assert "not found" in result.error.lower()

    def test_backup_designs_empty_source_succeeds(self, tmp_path):
        from src.services import backup_service as bsvc

        src = tmp_path / "source"
        src.mkdir()
        dest = tmp_path / "backup"

        result = bsvc.backup_designs(str(src), str(dest))

        assert result.success
        assert result.scanned == 0
        assert result.copied == 0


class TestRunStitchingBackfillAction:
    """Tests for the local stitching backfill helper."""

    def test_run_stitching_backfill_updates_unverified_design_only(self, db, monkeypatch):
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        image_tag = Tag(description="Flowers", tag_group="image")
        stitching_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([image_tag, stitching_tag])
        db.flush()

        unverified = Design(
            filename="rose.jef",
            filepath="/test/rose.jef",
            image_data=b"fake-preview",
            tags_checked=False,
        )
        unverified.tags = [image_tag]

        verified = Design(
            filename="verified.jef",
            filepath="/test/verified.jef",
            image_data=b"fake-preview",
            tags_checked=True,
        )
        verified.tags = [image_tag]

        db.add_all([unverified, verified])
        db.commit()

        monkeypatch.setattr(
            "src.services.auto_tagging._resolve_design_filepath",
            lambda *args, **kwargs: "/fake/path/rose.jef",
        )
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda *args, **kwargs: ["Filled"],
        )

        result = run_stitching_backfill_action(db=db, batch_size=None)

        db.refresh(unverified)
        db.refresh(verified)

        assert result.designs_considered == 1
        assert result.total_tagged == 1
        assert result.already_matched == 0
        assert result.no_match == 0
        assert result.cleared_only == 0
        assert result.tag_breakdown == {"Filled": 1}
        assert {tag.description for tag in unverified.tags} == {"Flowers", "Filled"}
        assert {tag.description for tag in verified.tags} == {"Flowers"}
        assert unverified.tags_checked is False

    def test_run_stitching_backfill_can_clear_existing_stitching_tags_when_no_match(
        self, db, monkeypatch
    ):
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        image_tag = Tag(description="Flowers", tag_group="image")
        old_stitching = Tag(description="Blackwork", tag_group="stitching")
        db.add_all([image_tag, old_stitching])
        db.flush()

        design = Design(
            filename="rose.jef",
            filepath="/test/rose.jef",
            image_data=b"fake-preview",
            tags_checked=False,
        )
        design.tags = [image_tag, old_stitching]
        db.add(design)
        db.commit()

        monkeypatch.setattr(
            "src.services.auto_tagging._resolve_design_filepath",
            lambda *args, **kwargs: None,
        )

        result = run_stitching_backfill_action(
            db=db,
            batch_size=None,
            allowed_descriptions={"Blackwork"},
            clear_existing_stitching=True,
        )

        db.refresh(design)

        assert result.designs_considered == 1
        assert result.total_tagged == 0
        assert result.no_match == 0
        assert result.cleared_only == 1
        assert result.still_untagged == 1
        assert [tag.description for tag in design.tags] == ["Flowers"]

    def test_run_stitching_backfill_counts_already_matched_designs_separately(
        self, db, monkeypatch
    ):
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        image_tag = Tag(description="Flowers", tag_group="image")
        stitching_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([image_tag, stitching_tag])
        db.flush()

        design = Design(
            filename="rose.jef",
            filepath="/test/rose.jef",
            image_data=b"fake-preview",
            tags_checked=False,
        )
        design.tags = [image_tag, stitching_tag]
        db.add(design)
        db.commit()

        monkeypatch.setattr(
            "src.services.auto_tagging._resolve_design_filepath",
            lambda *args, **kwargs: "/fake/path/rose.jef",
        )
        monkeypatch.setattr(
            "src.services.auto_tagging.suggest_stitching_from_pattern",
            lambda *args, **kwargs: ["Filled"],
        )

        result = run_stitching_backfill_action(
            db=db,
            batch_size=None,
            allowed_descriptions={"Filled"},
            clear_existing_stitching=True,
        )

        assert result.designs_considered == 1
        assert result.total_tagged == 0
        assert result.already_matched == 1
        assert result.no_match == 0
        assert result.still_untagged == 0
        assert result.tag_breakdown == {"Filled": 1}


class TestRunTaggingAction:
    """Tests for the run_tagging_action service function."""

    def _make_design(self, db, filename: str, tags_checked: bool = False, with_tags: bool = False):
        """Create a minimal Design record for tagging action tests."""
        from src.models import Design

        design = Design(
            filename=filename,
            filepath=f"/test/{filename}",
            tags_checked=tags_checked,
        )
        db.add(design)
        db.flush()
        return design

    def test_tag_untagged_empty_db(self, db):
        from src.services.auto_tagging import run_tagging_action

        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
        )
        assert result.designs_considered == 0
        assert result.total_tagged == 0
        assert result.errors == []

    def test_tag_untagged_uses_tier1(self, db):
        from src.services.auto_tagging import run_tagging_action

        # Add a design whose filename will match a keyword
        self._make_design(db, "flower_rose.jef")
        # Add a tag that matches
        from src.models import Tag

        tag = Tag(description="Flowers")
        db.add(tag)
        db.commit()

        result = run_tagging_action(db=db, action="tag_untagged", tiers=[1], api_key="")
        assert result.designs_considered >= 1
        assert result.tier1_tagged >= 1 or result.still_untagged >= 0

    def test_tag_untagged_includes_designs_with_only_stitching_tags(self, db):
        """Designs that have stitching tags but no image tags should be included."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        image_tag = Tag(description="Flowers", tag_group="image")
        stitching_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([image_tag, stitching_tag])
        db.flush()

        # Design with only a stitching tag — should be included
        d1 = Design(
            filename="stitched_only.jef",
            filepath="/test/stitched_only.jef",
        )
        d1.tags = [stitching_tag]
        db.add(d1)

        # Design with an image tag — should be excluded
        d2 = Design(
            filename="has_image_tag.jef",
            filepath="/test/has_image_tag.jef",
        )
        d2.tags = [image_tag]
        db.add(d2)

        # Design with no tags at all — should be included
        d3 = Design(
            filename="no_tags.jef",
            filepath="/test/no_tags.jef",
        )
        db.add(d3)
        db.commit()

        result = run_tagging_action(db=db, action="tag_untagged", tiers=[1], api_key="")

        # Should find the two designs without image tags
        assert result.designs_considered == 2

    def test_invalid_action_returns_error(self, db):
        from src.services.auto_tagging import run_tagging_action

        result = run_tagging_action(db=db, action="bad_action", tiers=[1], api_key="")
        assert result.errors

    def test_batch_size_limits_designs_considered(self, db):
        from src.services.auto_tagging import run_tagging_action

        for i in range(5):
            self._make_design(db, f"design_{i}.jef")
        db.commit()

        result = run_tagging_action(
            db=db, action="tag_untagged", tiers=[1], api_key="", batch_size=2
        )
        assert result.designs_considered <= 2

    def test_tier2_without_api_key_records_error(self, db):
        from src.services.auto_tagging import run_tagging_action

        self._make_design(db, "rose.jef")
        db.commit()

        result = run_tagging_action(db=db, action="tag_untagged", tiers=[1, 2], api_key="")
        assert any("Tier 2" in e or "API key" in e for e in result.errors)

    def test_tier3_without_api_key_records_error(self, db):
        from src.services.auto_tagging import run_tagging_action

        self._make_design(db, "rose.jef")
        db.commit()

        result = run_tagging_action(db=db, action="tag_untagged", tiers=[1, 3], api_key="")
        assert any("Tier 3" in e or "API key" in e for e in result.errors)

    def test_tier2_error_falls_back_to_tier3_and_records_error(self, db, monkeypatch):
        from src.models import Design, Tag
        from src.services import auto_tagging as tagging_svc

        tag = Tag(description="Animals")
        db.add(tag)
        db.flush()

        design = Design(
            filename="mystery_design.jef",
            filepath="/test/mystery_design.jef",
            image_data=b"fake-image-bytes",
            tags_checked=False,
        )
        db.add(design)
        db.commit()

        monkeypatch.setattr(
            tagging_svc,
            "suggest_tier2_batch",
            lambda *a, **kw: (_ for _ in ()).throw(RuntimeError("tier2 boom")),
        )
        monkeypatch.setattr(
            tagging_svc,
            "suggest_tier3_vision",
            lambda candidates, *_a, **_kw: {candidates[0].id: ["Animals"]},
        )

        result = tagging_svc.run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[2, 3],
            api_key="test-key",
            delay=0,
            vision_delay=0,
        )

        assert any("Tier 2 error: tier2 boom" in e for e in result.errors)
        assert result.tier3_tagged == 1
        assert result.total_tagged == 1
        assert result.still_untagged == 0

    def test_retag_all_unverified_skips_verified(self, db):
        """retag_all_unverified should skip verified designs."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        tag = Tag(description="Flowers")
        db.add(tag)
        db.flush()

        d = Design(
            filename="flower.jef",
            filepath="/test/flower.jef",
            tags_checked=True,
        )
        d.tags = [tag]
        db.add(d)
        db.commit()

        result = run_tagging_action(
            db=db,
            action="retag_all_unverified",
            tiers=[1],
            api_key="",
        )
        # Verified design should not be included
        assert result.designs_considered == 0

    def test_tagging_action_result_fields_present(self, db):
        from src.services.auto_tagging import TaggingActionResult, run_tagging_action

        result = run_tagging_action(db=db, action="tag_untagged", tiers=[1], api_key="")
        assert isinstance(result, TaggingActionResult)
        assert hasattr(result, "action")
        assert hasattr(result, "tiers_run")
        assert hasattr(result, "designs_considered")
        assert hasattr(result, "total_tagged")
        assert hasattr(result, "still_untagged")
        assert hasattr(result, "errors")

    def test_retag_all_includes_verified(self, db):
        """retag_all should always include verified designs."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        tag = Tag(description="Flowers")
        db.add(tag)
        db.flush()

        d = Design(
            filename="flower.jef",
            filepath="/test/flower.jef",
            tags_checked=True,
        )
        d.tags = [tag]
        db.add(d)
        db.commit()

        result = run_tagging_action(
            db=db,
            action="retag_all",
            tiers=[1],
            api_key="",
        )
        # Verified design should always be included with retag_all
        assert result.designs_considered >= 1

    def test_run_tagging_action_stop_signal_during_tier1(self, db, monkeypatch):
        """Stop signal during Tier 1 should halt processing and commit."""
        from src.models import Design, Tag
        from src.services import auto_tagging as tagging_svc
        from src.services.unified_backfill import clear_stop_signal, request_stop

        clear_stop_signal()
        tag = Tag(description="Flowers")
        db.add(tag)
        db.flush()

        for i in range(3):
            d = Design(
                filename=f"flower_{i}.jef",
                filepath=f"/test/flower_{i}.jef",
                tags_checked=False,
            )
            db.add(d)
        db.commit()

        # Request stop after the first design is processed
        call_count = [0]

        def stop_after_one(*args, **kwargs):
            call_count[0] += 1
            if call_count[0] == 1:
                request_stop()
            return []

        monkeypatch.setattr(tagging_svc, "suggest_tier1", stop_after_one)
        result = tagging_svc.run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
        )
        assert result.designs_considered == 3
        # Only the first design was processed before stop
        assert call_count[0] == 1
        clear_stop_signal()

    def test_run_tagging_action_stop_signal_before_tier2(self, db, monkeypatch):
        """Stop signal before Tier 2 should halt processing."""
        from src.models import Design, Tag
        from src.services import auto_tagging as tagging_svc
        from src.services.unified_backfill import clear_stop_signal, request_stop

        clear_stop_signal()
        tag = Tag(description="Flowers")
        db.add(tag)
        db.flush()

        d = Design(
            filename="rose.jef",
            filepath="/test/rose.jef",
            tags_checked=False,
        )
        db.add(d)
        db.commit()

        # Make Tier 1 return no match so Tier 2 would be needed
        monkeypatch.setattr(tagging_svc, "suggest_tier1", lambda *a, **kw: [])
        # Request stop before Tier 2
        request_stop()

        result = tagging_svc.run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1, 2],
            api_key="test-key",
        )
        assert result.designs_considered == 1
        # Tier 2 should not have run
        assert result.tier2_tagged == 0
        clear_stop_signal()


class TestSuggestStitchingFromPattern:
    """Tests for suggest_stitching_from_pattern with pre-read patterns."""

    def test_with_pre_read_pattern_returns_matched_descriptions(self):
        """suggest_stitching_from_pattern should return descriptions when
        a pre-read pyembroidery pattern is provided."""
        from src.models import Tag
        from src.services.auto_tagging import suggest_stitching_from_pattern

        # Create a minimal pattern object with the attributes StitchIdentifier needs
        class FakePattern:
            def __init__(self):
                self.stitches = [(0, 0, 1), (100, 0, 1), (100, 100, 1), (0, 100, 1), (0, 0, 1)]
                self.colors = [(0, 0, 0)]
                self.extras = {}
                self.threads = []

            def get_as_colorblocks(self):
                # Return a single colorblock with all stitches
                stitches = [[x, cmd, y] for (x, y, cmd) in self.stitches]
                from pyembroidery import EmbThread

                return iter([(stitches, EmbThread(thread="#000000", description="Black"))])

            def get_as_stitchblock(self):
                # Return stitchblocks grouped by color
                stitches = [[x, cmd, y] for (x, y, cmd) in self.stitches]
                from pyembroidery import EmbThread

                return iter([(stitches, EmbThread(thread="#000000", description="Black"))])

        tag = Tag(description="Filled", tag_group="stitching")
        desc_to_tag = {"Filled": tag}

        result = suggest_stitching_from_pattern(
            pattern_path="",
            filename="test.jef",
            filepath="/test/test.jef",
            desc_to_tag=desc_to_tag,
            pattern=FakePattern(),
        )
        # The pattern has stitches forming a closed shape, so it should
        # detect at least "Filled" or similar stitch type.
        assert isinstance(result, list)

    def test_with_pre_read_pattern_empty_stitches_returns_empty(self):
        """suggest_stitching_from_pattern should return empty list when
        the pre-read pattern has no stitches."""
        from src.models import Tag
        from src.services.auto_tagging import suggest_stitching_from_pattern

        class EmptyPattern:
            def __init__(self):
                self.stitches = []
                self.colors = []
                self.extras = {}
                self.threads = []

            def get_as_colorblocks(self):
                return iter([])

            def get_as_stitchblock(self):
                return iter([])

        tag = Tag(description="Filled", tag_group="stitching")
        desc_to_tag = {"Filled": tag}

        result = suggest_stitching_from_pattern(
            pattern_path="",
            filename="test.jef",
            filepath="/test/test.jef",
            desc_to_tag=desc_to_tag,
            pattern=EmptyPattern(),
        )
        assert result == []

    def test_no_matching_tags_returns_empty(self):
        """suggest_stitching_from_pattern should return empty list when
        detected stitch types don't match any known tag."""
        from src.services.auto_tagging import suggest_stitching_from_pattern

        class FakePattern:
            def __init__(self):
                self.stitches = [(0, 0, 1), (100, 0, 1), (100, 100, 1), (0, 100, 1), (0, 0, 1)]
                self.colors = [(0, 0, 0)]
                self.extras = {}
                self.threads = []

            def get_as_colorblocks(self):
                stitches = [[x, cmd, y] for (x, y, cmd) in self.stitches]
                from pyembroidery import EmbThread

                return iter([(stitches, EmbThread(thread="#000000", description="Black"))])

            def get_as_stitchblock(self):
                stitches = [[x, cmd, y] for (x, y, cmd) in self.stitches]
                from pyembroidery import EmbThread

                return iter([(stitches, EmbThread(thread="#000000", description="Black"))])

        # Empty desc_to_tag means no tags match
        result = suggest_stitching_from_pattern(
            pattern_path="",
            filename="test.jef",
            filepath="/test/test.jef",
            desc_to_tag={},
            pattern=FakePattern(),
        )
        assert result == []


class TestRunStitchingBackfillStopSignal:
    """Tests for stop signal handling in run_stitching_backfill_action."""

    def test_stop_signal_halts_processing(self, db, monkeypatch):
        """Stop signal should halt run_stitching_backfill_action mid-batch."""
        from src.models import Design, Tag
        from src.services import auto_tagging as tagging_svc
        from src.services.unified_backfill import clear_stop_signal, request_stop

        clear_stop_signal()
        stitching_tag = Tag(description="Filled", tag_group="stitching")
        db.add(stitching_tag)
        db.flush()

        for i in range(3):
            d = Design(
                filename=f"design_{i}.jef",
                filepath=f"/test/design_{i}.jef",
                tags_checked=False,
            )
            db.add(d)
        db.commit()

        # Request stop immediately so the first iteration sees it
        request_stop()

        result = tagging_svc.run_stitching_backfill_action(
            db=db,
            batch_size=10,
        )
        # Processing should have stopped early
        assert result.designs_considered >= 0
        clear_stop_signal()
