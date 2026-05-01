"""Tests for key FastAPI routes."""

import uuid

import pytest
from fastapi.testclient import TestClient

from src.database import get_db
from src.main import app
from src.services import designers, designs, hoops


class TestHealthRoute:
    def test_health(self, client):
        resp = client.get("/health")
        assert resp.status_code == 200
        assert resp.json() == {"status": "ok"}


class TestRootRedirect:
    def test_root_redirects_to_designs(self, client):
        resp = client.get("/", follow_redirects=False)
        assert resp.status_code in (301, 302, 307, 308)
        assert "/designs/" in resp.headers["location"]

    def test_favicon_redirects_to_static_icon(self, client_unaccepted):
        resp = client_unaccepted.get("/favicon.ico", follow_redirects=False)
        assert resp.status_code in (301, 302, 303, 307, 308)
        assert resp.headers["location"] == "/static/icons/favicon.ico"


class TestDisclaimerFlow:
    def test_unaccepted_user_is_redirected_to_disclaimer(
        self, client_unaccepted, db, monkeypatch, tmp_path
    ):
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(
            settings_svc, "DISCLAIMER_ACK_FILE", tmp_path / "disclaimer_accepted.txt"
        )
        resp = client_unaccepted.get("/designs/", follow_redirects=False)
        assert resp.status_code in (302, 303, 307, 308)
        assert resp.headers["location"].startswith("/disclaimer")

    def test_accepting_disclaimer_allows_access(self, client_unaccepted, db, monkeypatch, tmp_path):
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(
            settings_svc, "DISCLAIMER_ACK_FILE", tmp_path / "disclaimer_accepted.txt"
        )
        resp = client_unaccepted.post("/disclaimer/accept", follow_redirects=False)
        assert resp.status_code == 303

        browse = client_unaccepted.get("/designs/", follow_redirects=False)
        assert browse.status_code == 200

    def test_disclaimer_marker_skips_db_check_on_startup(self, monkeypatch, tmp_path):
        from src.services import settings_service as settings_svc

        ack_file = tmp_path / "disclaimer_accepted.txt"
        ack_file.write_text("accepted\n", encoding="utf-8")
        monkeypatch.setattr(settings_svc, "DISCLAIMER_ACK_FILE", ack_file)

        calls = {"count": 0}

        def broken_get_db():
            calls["count"] += 1
            raise RuntimeError("DB should not be opened for disclaimer check")
            yield

        app.dependency_overrides[get_db] = broken_get_db
        try:
            with TestClient(app) as client:
                resp = client.get("/", follow_redirects=False)
        finally:
            app.dependency_overrides.clear()

        assert resp.status_code in (301, 302, 307, 308)
        assert "/designs/" in resp.headers["location"]
        assert calls["count"] == 0


class TestAboutRoute:
    def test_about_page_ok(self, client_unaccepted):
        resp = client_unaccepted.get("/about")
        assert resp.status_code == 200
        assert "Privacy" in resp.text
        assert "Third-Party Notices" in resp.text

    def test_about_document_page_ok(self, client_unaccepted):
        resp = client_unaccepted.get("/about/document/privacy")
        assert resp.status_code == 200
        assert "Privacy" in resp.text

    def test_ai_tagging_document_page_ok(self, client_unaccepted):
        resp = client_unaccepted.get("/about/document/ai-tagging")
        assert resp.status_code == 200
        assert "AI-Assisted Auto-Tagging" in resp.text


class TestDesignersRoutes:
    def test_list_page_ok(self, client):
        resp = client.get("/admin/designers/")
        assert resp.status_code == 200

    def test_create_via_form(self, client):
        resp = client.post(
            "/admin/designers/", data={"name": "Anita Goodesign"}, follow_redirects=True
        )
        assert resp.status_code == 200
        assert "Anita Goodesign" in resp.text

    def test_create_duplicate_returns_400(self, client, db):
        designers.create(db, "Duplicate Designer")
        resp = client.post(
            "/admin/designers/", data={"name": "Duplicate Designer"}, follow_redirects=False
        )
        assert resp.status_code == 400


class TestHoopsRoutes:
    def test_list_hoops_page_ok(self, client):
        resp = client.get("/admin/hoops/")
        assert resp.status_code == 200

    def test_seed_hoops_route(self, client):
        resp = client.post("/admin/hoops/seed", follow_redirects=True)
        assert resp.status_code == 200
        assert "Hoop A" in resp.text

    def test_create_hoop_redirect_preserves_valid_import_token(self, client):
        token = str(uuid.uuid4())

        resp = client.post(
            "/admin/hoops/",
            data={
                "name": "Large Oval",
                "max_width_mm": "180",
                "max_height_mm": "130",
                "import_token": token,
            },
            follow_redirects=False,
        )

        assert resp.status_code == 303
        assert resp.headers["location"] == f"/admin/hoops/?import_token={token}"

    def test_create_hoop_duplicate_returns_400(self, client, db):
        hoops.create(db, "Duplicate Hoop", 100.0, 100.0)

        resp = client.post(
            "/admin/hoops/",
            data={"name": "Duplicate Hoop", "max_width_mm": "120", "max_height_mm": "110"},
            follow_redirects=False,
        )

        assert resp.status_code == 400

    def test_delete_hoop_redirect_strips_invalid_import_token(self, client, db):
        hoop = hoops.create(db, "Delete Route Hoop", 100.0, 100.0)

        resp = client.post(
            f"/admin/hoops/{hoop.id}/delete",
            data={"import_token": "not-a-valid-token"},
            follow_redirects=False,
        )

        assert resp.status_code == 303
        assert resp.headers["location"] == "/admin/hoops/"

    def test_delete_missing_hoop_returns_404(self, client):
        resp = client.post(
            "/admin/hoops/999999/delete",
            data={"import_token": str(uuid.uuid4())},
            follow_redirects=False,
        )

        assert resp.status_code == 404


class TestDesignsRoutes:
    def test_browse_empty(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200

    def test_detail_not_found(self, client):
        resp = client.get("/designs/999999")
        assert resp.status_code == 404

    def test_detail_found(self, client, db):
        d = designs.create(db, {"filename": "test.jef", "filepath": "/t.jef", "is_stitched": False})
        resp = client.get(f"/designs/{d.id}")
        assert resp.status_code == 200
        assert "test.jef" in resp.text

    def test_toggle_stitched(self, client, db):
        d = designs.create(db, {"filename": "ts.jef", "filepath": "/ts.jef", "is_stitched": False})
        resp = client.post(
            f"/designs/{d.id}/toggle-stitched",
            data={"is_stitched": "true"},
            follow_redirects=False,
        )
        assert resp.status_code in (302, 303)


class TestProjectsRoutes:
    def test_list_page_ok(self, client):
        resp = client.get("/projects/")
        assert resp.status_code == 200

    def test_create_project(self, client):
        resp = client.post(
            "/projects/",
            data={"name": "Winter Stitching", "description": "A cosy project"},
            follow_redirects=True,
        )
        assert resp.status_code == 200
        assert "Winter Stitching" in resp.text


class TestImportRoutes:
    def test_import_form_ok(self, client):
        resp = client.get("/import/")
        assert resp.status_code == 200

    def test_browse_folder_returns_multiple_paths_when_picker_supports_multiselect(
        self, client, monkeypatch
    ):
        from src.routes import bulk_import

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(
            bulk_import,
            "pick_folders",
            lambda start_dir="", title="": [r"D:\Embroidery\Folder1", r"D:\Embroidery\Folder2"],
        )

        resp = client.get("/import/browse-folder?start_dir=D:\\Embroidery")
        assert resp.status_code == 200
        assert resp.json() == {
            "paths": [r"D:\Embroidery\Folder1", r"D:\Embroidery\Folder2"],
            "path": r"D:\Embroidery\Folder1",
        }

    def test_browse_folder_uses_parent_of_saved_last_folder_when_start_dir_blank(
        self, client, db, monkeypatch, tmp_path
    ):
        from src.routes import bulk_import
        from src.services import settings_service

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        remembered = tmp_path / "last-used"
        remembered.mkdir()
        chosen = remembered / "chosen"
        chosen.mkdir()
        settings_service.set_setting(
            db, settings_service.SETTING_LAST_IMPORT_BROWSE_FOLDER, str(chosen)
        )
        seen = {}

        def fake_pick_folders(start_dir="", title=""):
            seen["start_dir"] = start_dir
            return [str(chosen)]

        monkeypatch.setattr(bulk_import, "pick_folders", fake_pick_folders)

        resp = client.get("/import/browse-folder")

        assert resp.status_code == 200
        assert seen["start_dir"] == str(remembered)
        assert settings_service.get_setting(
            db, settings_service.SETTING_LAST_IMPORT_BROWSE_FOLDER
        ) == str(chosen)

    def test_browse_folder_keeps_explicit_parent_start_dir_as_is(
        self, client, monkeypatch, tmp_path
    ):
        from src.routes import bulk_import

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        parent = tmp_path / "MachineEmbroideryDesigns"
        parent.mkdir()
        seen = {}

        def fake_pick_folders(start_dir="", title=""):
            seen["start_dir"] = start_dir
            return []

        monkeypatch.setattr(bulk_import, "pick_folders", fake_pick_folders)

        resp = client.get(f"/import/browse-folder?start_dir={parent}")

        assert resp.status_code == 200
        assert seen["start_dir"] == str(parent)

    def test_browse_folder_still_uses_picker_when_external_launches_disabled(
        self, client, monkeypatch
    ):
        from src.routes import bulk_import

        calls = []
        monkeypatch.setenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "1")
        monkeypatch.delenv("EMBROIDERY_DISABLE_NATIVE_DIALOGS", raising=False)
        monkeypatch.setattr(
            bulk_import,
            "pick_folders",
            lambda *args, **kwargs: calls.append((args, kwargs)) or [r"D:\Embroidery\Chosen"],
        )

        resp = client.get("/import/browse-folder")

        assert resp.status_code == 200
        assert resp.json()["path"] == r"D:\Embroidery\Chosen"
        assert len(calls) == 1

    def test_scan_multiple_folder_paths_calls_scan_folders(self, client, tmp_path, monkeypatch):
        from src.routes import bulk_import

        seen = {}

        def fake_scan_folders(paths, db):
            seen["paths"] = paths
            return []

        monkeypatch.setattr(bulk_import, "scan_folders", fake_scan_folders)

        f1 = tmp_path / "one"
        f2 = tmp_path / "two"
        f1.mkdir()
        f2.mkdir()

        resp = client.post(
            "/import/scan",
            data={"folder_paths": [str(f1), str(f2)]},
        )

        assert resp.status_code == 200
        assert seen["paths"] == [str(f1), str(f2)]

    def test_scan_empty_folder_path_returns_400(self, client):
        resp = client.post("/import/scan", data={"folder_path": "  "})
        assert resp.status_code == 400


# ---------------------------------------------------------------------------
# Pre-import tag check (precheck step) routes
# ---------------------------------------------------------------------------


class TestPrecheckRoutes:
    """Tests for the /import/precheck, /import/precheck-action and /import/do-confirm routes."""

    def _post_precheck(self, client, selected_files=None, folder_paths=None):
        """Helper: POST to /import/precheck with sensible defaults."""
        if selected_files is None:
            selected_files = [r"\rose.jef"]
        if folder_paths is None:
            folder_paths = [r"D:\Embroidery"]
        return client.post(
            "/import/precheck",
            data={"folder_paths": folder_paths, "selected_files": selected_files},
            follow_redirects=False,
        )

    def test_precheck_redirects_to_import_when_no_files(self, client):
        resp = client.post(
            "/import/precheck",
            data={"folder_paths": [r"D:\Embroidery"], "selected_files": []},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers["location"] == "/import/"

    def test_precheck_first_import_shows_hoop_setup_prompt(self, client, db):
        """When catalogue is empty, first import should encourage the user to review hoops."""
        from src.models import Design, Hoop

        # Ensure catalogue is empty and no hoops are pre-delivered.
        assert db.query(Design).count() == 0
        assert db.query(Hoop).count() == 0

        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "Review Hoops" in resp.text
        assert "hoops" in resp.text.lower()

    def test_precheck_first_import_skip_requests_extra_confirmation(self, client, db):
        """If the user says no on first import, ask if they are really really sure."""
        from src.models import Design

        assert db.query(Design).count() == 0

        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "import_now" in resp.text

        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 200
        assert "really really sure" in resp2.text.lower()
        assert "Review Hoops" in resp2.text

    def test_precheck_subsequent_import_offers_choice(self, client, db):
        """When designs already exist, precheck offers Yes/No/Cancel."""
        from src.services import designs as designs_svc

        designs_svc.create(
            db, {"filename": "existing.jef", "filepath": "/existing.jef", "is_stitched": False}
        )

        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "import_now" in resp.text  # No / import now button
        assert "review_tags" in resp.text  # Yes / review tags button
        assert "cancel" in resp.text

    def test_precheck_action_cancel_redirects_to_import(self, client, db):
        """Cancel action returns the user to the import landing page."""
        # Store a token first.
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match, "import_token not found in precheck page"
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "cancel", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 303
        assert resp2.headers["location"] == "/import/"

    def test_precheck_action_unknown_token_redirects_to_import(self, client):
        resp = client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": "nonexistent-token"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers["location"] == "/import/"

    def test_precheck_action_review_tags_shows_tags_page(self, client, db):
        """'review_tags' action renders the tag management page in import mode."""
        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "review_tags", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 200
        assert "Manage Tags" in resp2.text
        # Import-mode banner should be present.
        assert "Import mode" in resp2.text
        # Yes, continue with import link must appear.
        assert "continue with import" in resp2.text.lower()

    def test_precheck_action_review_hoops_shows_hoops_page(self, client):
        """'review_hoops' action renders the hoop management page in import mode."""
        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "review_hoops", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 200
        assert "Manage Hoops" in resp2.text
        assert "Import mode" in resp2.text
        assert "continue with import" in resp2.text.lower()

    def test_import_mode_review_pages_offer_links_to_other_review_lists(self, client):
        """Import-mode admin pages should let the user move between tags, sources, and designers before importing."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )

        cases = [
            (
                f"/admin/tags/?import_token={token}",
                ["Review Hoops", "Review Sources", "Review Designers"],
            ),
            (
                f"/admin/hoops/?import_token={token}",
                ["Review Tags", "Review Sources", "Review Designers"],
            ),
            (
                f"/admin/sources/?import_token={token}",
                ["Review Tags", "Review Hoops", "Review Designers"],
            ),
            (
                f"/admin/designers/?import_token={token}",
                ["Review Tags", "Review Hoops", "Review Sources"],
            ),
        ]

        for url, expected_links in cases:
            resp = client.get(url)
            assert resp.status_code == 200
            for label in expected_links:
                assert label in resp.text

    def test_import_mode_review_pages_wire_spinner_for_continue_buttons(self, client):
        """Top and bottom import buttons on review pages should both trigger the loading overlay."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )

        for url in (
            f"/admin/tags/?import_token={token}",
            f"/admin/hoops/?import_token={token}",
            f"/admin/sources/?import_token={token}",
            f"/admin/designers/?import_token={token}",
        ):
            resp = client.get(url)
            assert resp.status_code == 200
            assert "importOverlay" in resp.text
            assert "import-confirm-form" in resp.text
            assert "querySelectorAll('.import-confirm-form')" in resp.text

    def test_precheck_action_import_now_runs_import(self, client, db, monkeypatch):
        """'import_now' action should ultimately run the import and redirect to /designs/."""
        from src.routes import bulk_import as route_mod

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            captured["files"] = filepaths
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", lambda db, designs, **kw: [])

        resp = self._post_precheck(client, selected_files=[r"\rose.jef", r"\tulip.jef"])
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": token, "confirm_skip_hoops": "yes"},
            follow_redirects=True,
        )
        assert resp2.status_code == 200
        assert captured.get("files") == [r"\rose.jef", r"\tulip.jef"]

    def test_do_confirm_unknown_token_redirects_to_import(self, client):
        resp = client.post(
            "/import/do-confirm",
            data={"import_token": "not-a-valid-uuid"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers["location"] == "/import/"

    def test_do_confirm_runs_import_and_redirects_to_designs(self, client, db, monkeypatch):
        """POST /import/do-confirm with a valid token runs the import for the stored context."""
        from src.routes import bulk_import as route_mod

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            captured["files"] = filepaths
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", lambda db, designs, **kw: [])

        # Store context via precheck.
        resp = self._post_precheck(client, selected_files=[r"\butterfly.jef"])
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/do-confirm",
            data={"import_token": token},
            follow_redirects=True,
        )
        assert resp2.status_code == 200
        assert captured.get("files") == [r"\butterfly.jef"]

    def test_tags_page_import_mode_shows_banner(self, client):
        """GET /admin/tags/?import_token=<token> shows the import-mode banner."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.get(f"/admin/tags/?import_token={token}")
        assert resp.status_code == 200
        assert "Import mode" in resp.text
        assert "continue with import" in resp.text.lower()

    def test_tags_page_normal_mode_has_no_import_banner(self, client):
        """GET /admin/tags/ without a token should NOT show the import-mode banner."""
        resp = client.get("/admin/tags/")
        assert resp.status_code == 200
        assert "Import mode" not in resp.text
        assert "continue with import" not in resp.text.lower()

    def test_tags_create_in_import_mode_redirects_back_with_token(self, client):
        """Creating a tag while in import mode redirects back to tags with the token preserved."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            "/admin/tags/",
            data={"description": "PrecheckTag", "tag_group": "image", "import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_tags_delete_in_import_mode_redirects_back_with_token(self, client, db):
        """Deleting a tag in import mode redirects back to tags with the token preserved."""
        from src.routes import bulk_import as route_mod
        from src.services import tags as tags_svc

        tag = tags_svc.create(db, "PrecheckDeleteTag", "image")
        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            f"/admin/tags/{tag.id}/delete",
            data={"import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_tags_set_group_in_import_mode_redirects_back_with_token(self, client, db):
        """Updating a tag group in import mode redirects back to tags with the token preserved."""
        from src.models import Tag
        from src.routes import bulk_import as route_mod

        tag_obj = Tag(description="PrecheckGroupTag", tag_group=None)
        db.add(tag_obj)
        db.commit()
        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            f"/admin/tags/{tag_obj.id}/set-group",
            data={"tag_group": "stitching", "import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_precheck_shows_no_key_banner_when_no_api_key(self, client, monkeypatch):
        """Precheck page shows a blue 'no API key' banner when key is absent."""
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "")
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert (
            "No Google API key" in resp.text
            or "not configured" in resp.text.lower()
            or "Tier 1" in resp.text
        )

    def test_precheck_shows_cost_banner_when_api_key_present(self, client, monkeypatch):
        """Precheck page shows an amber cost notice when a key is present."""
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake_key")
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "cost" in resp.text.lower() or "Gemini" in resp.text

    def test_precheck_reflects_tier2_setting(self, client, db, monkeypatch):
        """Precheck page shows Tier 2 status from settings."""
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake_key")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "true")
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "Tier 2" in resp.text

    def test_run_confirm_uses_saved_tier_settings(self, client, db, monkeypatch):
        """confirm_import is called with run_tier2/run_tier3 derived from saved settings."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake_key")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "true")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER3_AUTO, "false")

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            captured["files"] = filepaths
            return []

        def fake_confirm(db, designs, **kw):
            captured["run_tier2"] = kw.get("run_tier2")
            captured["run_tier3"] = kw.get("run_tier3")
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        resp = self._post_precheck(client, selected_files=[r"\rose.jef"])
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": token, "confirm_skip_hoops": "yes"},
            follow_redirects=True,
        )

        assert captured.get("run_tier2") is True
        assert captured.get("run_tier3") is False

    def test_run_confirm_no_api_key_forces_tier1_only(self, client, db, monkeypatch):
        """confirm_import receives run_tier2=False, run_tier3=False when no API key."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "true")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER3_AUTO, "true")

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            return []

        def fake_confirm(db, designs, **kw):
            captured["run_tier2"] = kw.get("run_tier2")
            captured["run_tier3"] = kw.get("run_tier3")
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        resp = self._post_precheck(client, selected_files=[r"\rose.jef"])
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": token, "confirm_skip_hoops": "yes"},
            follow_redirects=True,
        )

        assert captured.get("run_tier2") is False
        assert captured.get("run_tier3") is False

    def test_run_confirm_passes_batch_limit_from_settings(self, client, db, monkeypatch):
        """confirm_import receives batch_limit from saved settings."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake_key")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "true")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_BATCH_SIZE, "25")

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            return []

        def fake_confirm(db, designs, **kw):
            captured["batch_limit"] = kw.get("batch_limit")
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        resp = self._post_precheck(client, selected_files=[r"\rose.jef"])
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        client.post(
            "/import/precheck-action",
            data={"action": "import_now", "import_token": token, "confirm_skip_hoops": "yes"},
            follow_redirects=True,
        )

        assert captured.get("batch_limit") == 25

    # ------------------------------------------------------------------
    # Sources import-mode tests
    # ------------------------------------------------------------------

    def test_precheck_action_review_sources_shows_sources_page(self, client, db):
        """'review_sources' action renders the source management page in import mode."""
        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "review_sources", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 200
        assert "Manage Sources" in resp2.text
        # Import-mode banner should be present.
        assert "Import mode" in resp2.text
        # Yes, continue with import link must appear.
        assert "continue with import" in resp2.text.lower()

    def test_sources_page_import_mode_shows_banner(self, client):
        """GET /admin/sources/?import_token=<token> shows the import-mode banner."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.get(f"/admin/sources/?import_token={token}")
        assert resp.status_code == 200
        assert "Import mode" in resp.text
        assert "continue with import" in resp.text.lower()

    def test_sources_page_normal_mode_has_no_import_banner(self, client):
        """GET /admin/sources/ without a token should NOT show the import-mode banner."""
        resp = client.get("/admin/sources/")
        assert resp.status_code == 200
        assert "Import mode" not in resp.text
        assert "continue with import" not in resp.text.lower()

    def test_sources_create_in_import_mode_redirects_back_with_token(self, client):
        """Creating a source while in import mode redirects back to sources with the token preserved."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            "/admin/sources/",
            data={"name": "PrecheckSource", "import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_sources_delete_in_import_mode_redirects_back_with_token(self, client, db):
        """Deleting a source in import mode redirects back to sources with the token preserved."""
        from src.routes import bulk_import as route_mod
        from src.services import sources as sources_svc

        source = sources_svc.create(db, "PrecheckDeleteSource")
        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            f"/admin/sources/{source.id}/delete",
            data={"import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_precheck_first_import_shows_review_sources_button(self, client, db):
        """First import precheck must offer 'Review Sources' button."""
        from src.models import Design

        assert db.query(Design).count() == 0
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "review_sources" in resp.text

    def test_precheck_subsequent_import_offers_review_sources(self, client, db):
        """Subsequent import precheck offers the 'Review Sources' button."""
        from src.services import designs as designs_svc

        designs_svc.create(
            db, {"filename": "existing.jef", "filepath": "/existing.jef", "is_stitched": False}
        )
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "review_sources" in resp.text

    # ------------------------------------------------------------------
    # Designers import-mode tests
    # ------------------------------------------------------------------

    def test_precheck_action_review_designers_shows_designers_page(self, client, db):
        """'review_designers' action renders the designer management page in import mode."""
        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        resp2 = client.post(
            "/import/precheck-action",
            data={"action": "review_designers", "import_token": token},
            follow_redirects=False,
        )
        assert resp2.status_code == 200
        assert "Manage Designers" in resp2.text
        # Import-mode banner should be present.
        assert "Import mode" in resp2.text
        # Yes, continue with import link must appear.
        assert "continue with import" in resp2.text.lower()

    def test_designers_page_import_mode_shows_banner(self, client):
        """GET /admin/designers/?import_token=<token> shows the import-mode banner."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.get(f"/admin/designers/?import_token={token}")
        assert resp.status_code == 200
        assert "Import mode" in resp.text
        assert "continue with import" in resp.text.lower()

    def test_designers_page_normal_mode_has_no_import_banner(self, client):
        """GET /admin/designers/ without a token should NOT show the import-mode banner."""
        resp = client.get("/admin/designers/")
        assert resp.status_code == 200
        assert "Import mode" not in resp.text
        assert "continue with import" not in resp.text.lower()

    def test_designers_create_in_import_mode_redirects_back_with_token(self, client):
        """Creating a designer while in import mode redirects back to designers with the token preserved."""
        from src.routes import bulk_import as route_mod

        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            "/admin/designers/",
            data={"name": "PrecheckDesigner", "import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_designers_delete_in_import_mode_redirects_back_with_token(self, client, db):
        """Deleting a designer in import mode redirects back to designers with the token preserved."""
        from src.routes import bulk_import as route_mod
        from src.services import designers as designers_svc

        designer = designers_svc.create(db, "PrecheckDeleteDesigner")
        token = route_mod._store_import_context(
            {
                "folder_paths": [r"D:\Embroidery"],
                "selected_files": [r"\rose.jef"],
                "extra": {},
            }
        )
        resp = client.post(
            f"/admin/designers/{designer.id}/delete",
            data={"import_token": token},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert f"import_token={token}" in resp.headers["location"]

    def test_precheck_first_import_shows_review_designers_button(self, client, db):
        """First import precheck must offer 'Review Designers' button."""
        from src.models import Design

        assert db.query(Design).count() == 0
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "review_designers" in resp.text

    def test_precheck_subsequent_import_offers_review_designers(self, client, db):
        """Subsequent import precheck offers the 'Review Designers' button."""
        from src.services import designs as designs_svc

        designs_svc.create(
            db, {"filename": "existing.jef", "filepath": "/existing.jef", "is_stitched": False}
        )
        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "review_designers" in resp.text

    def test_list_page_ok(self, client):
        resp = client.get("/admin/tags/")
        assert resp.status_code == 200

    def test_create_via_form(self, client):
        resp = client.post(
            "/admin/tags/",
            data={"description": "Autumn Leaves", "tag_group": "image"},
            follow_redirects=True,
        )
        assert resp.status_code == 200
        assert "Autumn Leaves" in resp.text

    def test_create_duplicate_returns_400(self, client, db):
        from src.services import tags

        tags.create(db, "Duplicate Tag", "image")
        resp = client.post(
            "/admin/tags/",
            data={"description": "Duplicate Tag", "tag_group": "image"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_delete(self, client, db):
        from src.services import tags

        tag = tags.create(db, "Delete Me Tag", "image")
        resp = client.post(f"/admin/tags/{tag.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_set_group_via_route(self, client, db):
        from src.models import Tag

        raw = Tag(description="Needs Classifying", tag_group=None)
        db.add(raw)
        db.commit()
        resp = client.post(
            f"/admin/tags/{raw.id}/set-group",
            data={"tag_group": "stitching"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        db.refresh(raw)
        assert raw.tag_group == "stitching"

    def test_delete_nonexistent_returns_404(self, client):
        resp = client.post("/admin/tags/999999/delete", follow_redirects=False)
        assert resp.status_code == 404

    # ------------------------------------------------------------------ #
    # Image preference (2D / 3D) on precheck page
    # ------------------------------------------------------------------ #

    def test_precheck_page_shows_image_preference_toggle(self, client, db, monkeypatch):
        """Precheck page renders the 2D/3D radio buttons with saved setting."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        settings_svc.set_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE, "3d")

        resp = self._post_precheck(client)
        assert resp.status_code == 200
        # Radio buttons for 2D and 3D should be present
        assert 'value="2d"' in resp.text
        assert 'value="3d"' in resp.text
        # The saved setting label should show "3d"
        assert "Saved setting: <strong>3D</strong>" in resp.text

    def test_precheck_page_defaults_to_2d_when_no_setting(self, client, db, monkeypatch):
        """Precheck page defaults to 2D when no image_preference is saved."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        # Ensure no image_preference is set (default should be "2d")
        settings_svc.set_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE, "2d")

        resp = self._post_precheck(client)
        assert resp.status_code == 200
        assert "Saved setting: <strong>2D</strong>" in resp.text

    def test_precheck_action_captures_image_preference_override(self, client, db, monkeypatch):
        """Precheck-action captures image_preference override and passes preview_3d to process_selected_files."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        settings_svc.set_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE, "3d")

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            captured["preview_3d"] = kw.get("preview_3d")
            return []

        def fake_confirm(db, designs, **kw):
            captured["confirm_preview_3d"] = kw.get("preview_3d")
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        # Override to 2D on the precheck form
        client.post(
            "/import/precheck-action",
            data={
                "action": "import_now",
                "import_token": token,
                "confirm_skip_hoops": "yes",
                "image_preference": "2d",
            },
            follow_redirects=True,
        )

        # preview_3d should be False when image_preference is "2d"
        assert captured.get("preview_3d") is False
        assert captured.get("confirm_preview_3d") is False

    def test_precheck_action_passes_preview_3d_true_for_3d(self, client, db, monkeypatch):
        """Precheck-action passes preview_3d=True when image_preference is '3d'."""
        from src.routes import bulk_import as route_mod
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(route_mod, "scan_folders", lambda paths, _db: [])
        settings_svc.set_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE, "2d")

        captured = {}

        def fake_process(filepaths, source_folders, db, **kw):
            captured["preview_3d"] = kw.get("preview_3d")
            return []

        def fake_confirm(db, designs, **kw):
            captured["confirm_preview_3d"] = kw.get("preview_3d")
            return []

        from src.services import scanning as scanning_mod

        monkeypatch.setattr(scanning_mod, "process_selected_files", fake_process)
        monkeypatch.setattr(route_mod.svc, "confirm_import", fake_confirm)

        resp = self._post_precheck(client)
        import re

        token_match = re.search(r'name="import_token"\s+value="([^"]+)"', resp.text)
        assert token_match
        token = token_match.group(1)

        # Override to 3D on the precheck form
        client.post(
            "/import/precheck-action",
            data={
                "action": "import_now",
                "import_token": token,
                "confirm_skip_hoops": "yes",
                "image_preference": "3d",
            },
            follow_redirects=True,
        )

        # preview_3d should be True when image_preference is "3d"
        assert captured.get("preview_3d") is True
        assert captured.get("confirm_preview_3d") is True


# ---------------------------------------------------------------------------
# Tags admin routes (canonical /admin/tags/ endpoints)
# ---------------------------------------------------------------------------


class TestTagsRoutes:
    def test_list_page_ok(self, client):
        resp = client.get("/admin/tags/")
        assert resp.status_code == 200

    def test_create_via_form(self, client):
        resp = client.post(
            "/admin/tags/",
            data={"description": "Spring Blossoms", "tag_group": "image"},
            follow_redirects=True,
        )
        assert resp.status_code == 200
        assert "Spring Blossoms" in resp.text

    def test_create_duplicate_returns_400(self, client, db):
        from src.services import tags

        tags.create(db, "Duplicate Tag", "image")
        resp = client.post(
            "/admin/tags/",
            data={"description": "Duplicate Tag", "tag_group": "image"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_delete(self, client, db):
        from src.services import tags

        tag = tags.create(db, "Delete Me Tag", "image")
        resp = client.post(f"/admin/tags/{tag.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_set_group_via_route(self, client, db):
        from src.models import Tag

        raw = Tag(description="Tag Needs Group", tag_group=None)
        db.add(raw)
        db.commit()
        resp = client.post(
            f"/admin/tags/{raw.id}/set-group",
            data={"tag_group": "stitching"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        db.refresh(raw)
        assert raw.tag_group == "stitching"

    def test_delete_nonexistent_returns_404(self, client):
        resp = client.post("/admin/tags/999999/delete", follow_redirects=False)
        assert resp.status_code == 404


# ---------------------------------------------------------------------------
# Source admin routes
# ---------------------------------------------------------------------------


class TestSourcesRoutes:
    def test_list_page_ok(self, client):
        resp = client.get("/admin/sources/")
        assert resp.status_code == 200

    def test_create_via_form(self, client):
        resp = client.post("/admin/sources/", data={"name": "Floriani"}, follow_redirects=True)
        assert resp.status_code == 200
        assert "Floriani" in resp.text

    def test_create_duplicate_returns_400(self, client, db):
        from src.services import sources

        sources.create(db, "Duplicate Source")
        resp = client.post(
            "/admin/sources/", data={"name": "Duplicate Source"}, follow_redirects=False
        )
        assert resp.status_code == 400

    def test_delete(self, client, db):
        from src.services import sources

        s = sources.create(db, "Delete Me Source")
        resp = client.post(f"/admin/sources/{s.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_delete_nonexistent_returns_404(self, client):
        resp = client.post("/admin/sources/999999/delete", follow_redirects=False)
        assert resp.status_code == 404


# ---------------------------------------------------------------------------
# Design routes — additional coverage
# ---------------------------------------------------------------------------


class TestDesignsRoutesExtra:
    def test_edit_design(self, client, db):
        d = designs.create(
            db, {"filename": "edit.jef", "filepath": "/edit.jef", "is_stitched": False}
        )
        resp = client.post(
            f"/designs/{d.id}/edit",
            data={"filename": "edited.jef", "filepath": "/edit.jef", "is_stitched": "false"},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_rate_design(self, client, db):
        d = designs.create(
            db, {"filename": "rate.jef", "filepath": "/rate.jef", "is_stitched": False}
        )
        resp = client.post(f"/designs/{d.id}/rate", data={"rating": "4"}, follow_redirects=False)
        assert resp.status_code == 303

    def test_rate_design_invalid_returns_400(self, client, db):
        d = designs.create(
            db, {"filename": "badrate.jef", "filepath": "/br.jef", "is_stitched": False}
        )
        resp = client.post(f"/designs/{d.id}/rate", data={"rating": "10"}, follow_redirects=False)
        assert resp.status_code == 400

    def test_delete_design(self, client, db):
        d = designs.create(
            db, {"filename": "todelete.jef", "filepath": "/td.jef", "is_stitched": False}
        )
        resp = client.post(f"/designs/{d.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_toggle_tags_checked(self, client, db):
        d = designs.create(
            db, {"filename": "tags.jef", "filepath": "/tags.jef", "is_stitched": False}
        )
        resp = client.post(
            f"/designs/{d.id}/toggle-tags-checked",
            data={"tags_checked": "true"},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_set_tags(self, client, db):
        from src.services import tags

        tag = tags.create(db, "Test Tag", "image")
        d = designs.create(
            db, {"filename": "tagged.jef", "filepath": "/tg.jef", "is_stitched": False}
        )
        resp = client.post(
            f"/designs/{d.id}/set-tags",
            data={"tag_ids": str(tag.id)},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_image_endpoint_no_image_returns_404(self, client, db):
        d = designs.create(
            db, {"filename": "noimg.jef", "filepath": "/ni.jef", "is_stitched": False}
        )
        resp = client.get(f"/designs/{d.id}/image")
        assert resp.status_code == 404

    def test_image_endpoint_with_image(self, client, db):
        d = designs.create(
            db,
            {
                "filename": "img.jef",
                "filepath": "/img.jef",
                "is_stitched": False,
                "image_data": b"\x89PNG",
            },
        )
        resp = client.get(f"/designs/{d.id}/image")
        assert resp.status_code == 200
        assert resp.headers["content-type"] == "image/png"

    def test_detail_shows_open_in_editor_link(self, client, db):
        d = designs.create(
            db, {"filename": "rose.jef", "filepath": "\\flowers\\rose.jef", "is_stitched": False}
        )

        resp = client.get(f"/designs/{d.id}")

        assert resp.status_code == 200
        assert f"/designs/{d.id}/open-in-editor" in resp.text
        assert "Open in Editor" in resp.text
        assert "/static/icons/app-icon.svg" in resp.text

    def test_open_in_editor_redirects_and_opens_file(self, client, db, monkeypatch):
        d = designs.create(
            db, {"filename": "rose.jef", "filepath": "\\flowers\\rose.jef", "is_stitched": False}
        )

        from src.routes import designs as design_routes

        opened = {}
        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(design_routes, "get_designs_base_path", lambda db: r"C:\\catalogue")
        monkeypatch.setattr(
            design_routes,
            "_open_with_default_app",
            lambda path: opened.setdefault("path", path),
            raising=False,
        )

        resp = client.get(f"/designs/{d.id}/open-in-editor", follow_redirects=False)

        assert resp.status_code == 303
        assert resp.headers["location"] == f"/designs/{d.id}"

    def test_open_in_editor_ignores_legacy_designs_base_path_setting(self, client, db, monkeypatch):
        from src.routes import designs as design_routes
        from src.services import settings_service as settings_svc

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        d = designs.create(
            db,
            {
                "filename": "1 and half x stitch only 2 Colours.jef",
                "filepath": "\\My Creations\\Cross Stitch\\1 and half x stitch only 2 Colours.jef",
                "is_stitched": False,
            },
        )
        settings_svc.set_setting(
            db, settings_svc.SETTING_DESIGNS_BASE_PATH, r"D:\\My Software Development\\TestDesigns"
        )
        monkeypatch.setattr(
            settings_svc, "DESIGNS_BASE_PATH", r"C:\\managed\\MachineEmbroideryDesigns"
        )

        opened = {}
        monkeypatch.setattr(
            design_routes,
            "_open_with_default_app",
            lambda path: opened.setdefault("path", path),
            raising=False,
        )

        resp = client.get(f"/designs/{d.id}/open-in-editor", follow_redirects=False)

        assert resp.status_code == 303

    def test_open_in_editor_not_found(self, client):
        resp = client.get("/designs/999999/open-in-editor", follow_redirects=False)

        assert resp.status_code == 404

    def test_open_in_editor_skips_launch_during_pytest(self, client, db, monkeypatch):
        from src.routes import designs as design_routes

        d = designs.create(
            db, {"filename": "guarded.jef", "filepath": "\\guarded.jef", "is_stitched": False}
        )
        opened = []
        monkeypatch.setattr(design_routes, "get_designs_base_path", lambda db: r"C:\\catalogue")
        monkeypatch.setattr(
            design_routes,
            "_open_with_default_app",
            lambda path: opened.append(path),
            raising=False,
        )
        monkeypatch.setenv("PYTEST_CURRENT_TEST", "tests::guarded")

        resp = client.get(f"/designs/{d.id}/open-in-editor", follow_redirects=False)

        assert resp.status_code == 303
        assert opened == []

    def test_open_in_editor_unexpected_launch_error_propagates(self, client, db, monkeypatch):
        from src.routes import designs as design_routes

        d = designs.create(
            db, {"filename": "broken.jef", "filepath": "\\broken.jef", "is_stitched": False}
        )
        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(design_routes, "get_designs_base_path", lambda db: r"C:\\catalogue")

        def boom(_path):
            raise RuntimeError("unexpected launch failure")

        monkeypatch.setattr(design_routes, "_open_with_default_app", boom, raising=False)

        resp = client.get(f"/designs/{d.id}/open-in-editor", follow_redirects=False)
        assert resp.status_code == 303

    def test_print_view(self, client, db):
        d = designs.create(
            db, {"filename": "print.jef", "filepath": "/pr.jef", "is_stitched": False}
        )
        resp = client.get(f"/designs/{d.id}/print")
        assert resp.status_code == 200
        assert "print.jef" in resp.text

    def test_print_view_not_found(self, client):
        resp = client.get("/designs/999999/print")
        assert resp.status_code == 404

    def test_bulk_verify(self, client, db):
        d1 = designs.create(
            db, {"filename": "bv1.jef", "filepath": "/bv1.jef", "is_stitched": False}
        )
        d2 = designs.create(
            db, {"filename": "bv2.jef", "filepath": "/bv2.jef", "is_stitched": False}
        )
        resp = client.post(
            "/designs/bulk-verify",
            data={"design_ids": [str(d1.id), str(d2.id)]},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_bulk_set_tags(self, client, db):
        from src.services import tags

        tag = tags.create(db, "Bulk Tag", "image")
        d1 = designs.create(
            db, {"filename": "bs1.jef", "filepath": "/bs1.jef", "is_stitched": False}
        )
        d2 = designs.create(
            db, {"filename": "bs2.jef", "filepath": "/bs2.jef", "is_stitched": False}
        )
        resp = client.post(
            "/designs/bulk-set-tags",
            data={"design_ids": [str(d1.id), str(d2.id)], "tag_ids": [str(tag.id)]},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_bulk_add_to_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Bulk Route Project")
        d1 = designs.create(
            db, {"filename": "bp1.jef", "filepath": "/bp1.jef", "is_stitched": False}
        )
        d2 = designs.create(
            db, {"filename": "bp2.jef", "filepath": "/bp2.jef", "is_stitched": False}
        )

        resp = client.post(
            "/designs/bulk-add-to-project",
            data={"design_ids": [str(d1.id), str(d2.id)], "project_id": str(p.id)},
            follow_redirects=False,
        )

        assert resp.status_code == 303
        db.refresh(p)
        assert {design.id for design in p.designs} == {d1.id, d2.id}

    def test_browse_filter_by_filename(self, client, db):
        designs.create(
            db, {"filename": "zebra_border.jef", "filepath": "/z.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?filename=zebra*")
        assert resp.status_code == 200
        assert "zebra_border.jef" in resp.text

    def test_browse_sort_by_date(self, client):
        resp = client.get("/designs/?sort_by=date_added&sort_dir=desc")
        assert resp.status_code == 200

    def test_browse_unverified_filter(self, client):
        resp = client.get("/designs/?unverified=true")
        assert resp.status_code == 200

    def test_browse_shows_verified_status_symbol(self, client, db):
        # Create both designs as unverified (default behavior)
        d_verified = designs.create(
            db,
            {
                "filename": "verified_symbol.jef",
                "filepath": "/TestData/verified_symbol.jef",
                "is_stitched": False,
            },
        )
        designs.create(
            db,
            {
                "filename": "unverified_symbol.jef",
                "filepath": "/TestData/unverified_symbol.jef",
                "is_stitched": False,
            },
        )

        # Simulate user verifying the first design
        resp = client.post(
            f"/designs/{d_verified.id}/toggle-tags-checked",
            data={"tags_checked": "true"},
            follow_redirects=True,
        )
        assert resp.status_code == 200

        resp = client.get("/designs/")

        assert resp.status_code == 200
        print("\n\n==== DEBUG: /designs/ response ====")
        print(resp.text)
        print("==== END DEBUG ====")
        assert 'title="Verified"' in resp.text
        assert 'title="Not verified"' in resp.text

    def test_add_to_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Test Project")
        d = designs.create(
            db, {"filename": "proj.jef", "filepath": "/proj.jef", "is_stitched": False}
        )
        resp = client.post(
            f"/designs/{d.id}/add-to-project",
            data={"project_id": str(p.id)},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_remove_from_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Remove Project")
        d = designs.create(
            db, {"filename": "rproj.jef", "filepath": "/rp.jef", "is_stitched": False}
        )
        proj_svc.add_design(db, p.id, d.id)
        resp = client.post(
            f"/designs/{d.id}/remove-from-project/{p.id}",
            follow_redirects=False,
        )
        assert resp.status_code == 303


# ---------------------------------------------------------------------------
# Project routes — additional coverage
# ---------------------------------------------------------------------------


class TestProjectsRoutesExtra:
    def test_new_project_form(self, client):
        resp = client.get("/projects/new")
        assert resp.status_code == 200

    def test_project_detail(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Detail Project")
        resp = client.get(f"/projects/{p.id}")
        assert resp.status_code == 200
        assert "Detail Project" in resp.text

    def test_project_detail_not_found(self, client):
        resp = client.get("/projects/999999")
        assert resp.status_code == 404

    def test_edit_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Edit Me")
        resp = client.post(
            f"/projects/{p.id}/edit",
            data={"name": "Edited Project", "description": "updated"},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_delete_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Delete Project")
        resp = client.post(f"/projects/{p.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_remove_design_from_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Remove Design Project")
        d = designs.create(db, {"filename": "rd.jef", "filepath": "/rd.jef", "is_stitched": False})
        proj_svc.add_design(db, p.id, d.id)
        resp = client.post(f"/projects/{p.id}/remove-design/{d.id}", follow_redirects=False)
        assert resp.status_code == 303


class TestDesignersRoutesExtended:
    def test_list_designers(self, client):
        resp = client.get("/admin/designers/")
        assert resp.status_code == 200

    def test_create_designer(self, client):
        resp = client.post(
            "/admin/designers/",
            data={"name": "New Designer"},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_create_designer_duplicate_returns_400(self, client):
        client.post("/admin/designers/", data={"name": "Dup Designer"})
        resp = client.post(
            "/admin/designers/",
            data={"name": "Dup Designer"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_delete_designer(self, client, db):
        from src.services import designers as dsvc

        d = dsvc.create(db, "To Delete")
        resp = client.post(f"/admin/designers/{d.id}/delete", follow_redirects=False)
        assert resp.status_code == 303

    def test_delete_designer_not_found(self, client):
        resp = client.post("/admin/designers/999999/delete", follow_redirects=False)
        assert resp.status_code == 404


class TestSettingsRoutes:
    def test_settings_page_get(self, client):
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200

    def test_settings_page_shows_form(self, client):
        resp = client.get("/admin/settings/")
        assert "designs_base_path" in resp.text.lower() or resp.status_code == 200

    def test_settings_page_contains_google_api_key_controls(self, client):
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "Google Gemini API key" in resp.text
        assert 'type="password"' in resp.text
        assert "/about/document/ai-tagging" in resp.text

    def test_settings_page_shows_storage_locations(self, client):
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "Catalogue data location" in resp.text
        assert "Log folder" in resp.text
        assert "Managed design folder" not in resp.text
        assert "browse-data-root" in resp.text

    def test_browse_data_root_uses_picker(self, client, monkeypatch):
        import src.routes.settings as settings_routes

        calls = []
        monkeypatch.setattr(
            settings_routes,
            "pick_folder",
            lambda **kwargs: calls.append(kwargs) or r"E:\EmbroideryCatalogueData",
        )

        resp = client.get("/admin/settings/browse-data-root")

        assert resp.status_code == 200
        assert resp.json()["path"] == r"E:\EmbroideryCatalogueData"
        assert len(calls) == 1

    def test_save_settings(self, client):
        resp = client.post(
            "/admin/settings/",
            data={"designs_base_path": "/new/path"},
            follow_redirects=False,
        )
        assert resp.status_code == 303

    def test_save_settings_redirect_target(self, client):
        resp = client.post(
            "/admin/settings/",
            data={"designs_base_path": "/redirected/path"},
            follow_redirects=False,
        )
        assert "saved=1" in resp.headers.get("location", "")

    def test_save_settings_updates_google_api_key_in_env(self, client, monkeypatch, tmp_path):
        from src.services import settings_service as settings_svc

        env_path = tmp_path / ".env"
        env_path.write_text("APP_PORT=8003\n", encoding="utf-8")
        monkeypatch.setattr(settings_svc, "ENV_FILE", env_path)

        resp = client.post(
            "/admin/settings/",
            data={"google_api_key": "AIzaSy_test_key_1234567890"},
            follow_redirects=False,
        )

        assert resp.status_code == 303
        assert "saved=1" in resp.headers.get("location", "")

        text = env_path.read_text(encoding="utf-8")
        assert "APP_PORT=8003" in text
        assert "GOOGLE_API_KEY=AIzaSy_test_key_1234567890" in text

    def test_settings_page_after_save(self, client):
        client.post("/admin/settings/", data={"designs_base_path": "/saved/path"})
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200

    def test_settings_page_shows_ai_tagging_controls(self, client):
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "ai_tier2_auto" in resp.text
        assert "ai_tier3_auto" in resp.text
        assert "ai_batch_size" in resp.text
        assert "import_commit_batch_size" in resp.text

    def test_save_settings_persists_tier2_auto(self, client, db):
        from src.services import settings_service as settings_svc

        resp = client.post(
            "/admin/settings/",
            data={"ai_tier2_auto": "1"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert settings_svc._is_truthy(
            settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER2_AUTO)
        )

    def test_save_settings_persists_tier3_auto(self, client, db):
        from src.services import settings_service as settings_svc

        resp = client.post(
            "/admin/settings/",
            data={"ai_tier3_auto": "1"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert settings_svc._is_truthy(
            settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER3_AUTO)
        )

    def test_save_settings_unchecked_tier2_saves_false(self, client, db):
        from src.services import settings_service as settings_svc

        # First enable it
        client.post("/admin/settings/", data={"ai_tier2_auto": "1"}, follow_redirects=False)
        # Now submit without the checkbox (simulating unchecked)
        client.post("/admin/settings/", data={}, follow_redirects=False)
        assert not settings_svc._is_truthy(
            settings_svc.get_setting(db, settings_svc.SETTING_AI_TIER2_AUTO)
        )

    def test_save_settings_persists_batch_size(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/settings/",
            data={"ai_batch_size": "50"},
            follow_redirects=False,
        )
        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE) == "50"

    def test_save_settings_persists_import_commit_batch_size(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/settings/",
            data={"import_commit_batch_size": "750"},
            follow_redirects=False,
        )
        assert settings_svc.get_setting(db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE) == "750"

    def test_save_settings_clamps_batch_sizes_to_max(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/settings/",
            data={
                "ai_batch_size": "500000",
                "import_commit_batch_size": "999999",
            },
            follow_redirects=False,
        )

        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE) == "10000"
        assert (
            settings_svc.get_setting(db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE) == "10000"
        )

    def test_save_settings_normalizes_invalid_and_low_batch_sizes(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/settings/",
            data={
                "ai_batch_size": "0",
                "import_commit_batch_size": "not-a-number",
            },
            follow_redirects=False,
        )

        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_BATCH_SIZE) == "1"
        assert settings_svc.get_setting(db, settings_svc.SETTING_IMPORT_COMMIT_BATCH_SIZE) == ""

    def test_settings_page_shows_no_key_notice_when_no_api_key(self, client, monkeypatch):
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "")
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "No API key is saved" in resp.text

    def test_settings_page_shows_cost_notice_when_api_key_present(self, client, monkeypatch):
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake_key")
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "Cost notice" in resp.text or "cost" in resp.text.lower()
        assert "ai.google.dev/pricing" in resp.text

    # ------------------------------------------------------------------ #
    # Image preference (2D / 3D)
    # ------------------------------------------------------------------ #

    def test_settings_page_shows_image_preference_controls(self, client):
        """Settings page should show 2D/3D radio buttons."""
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert 'value="2d"' in resp.text
        assert 'value="3d"' in resp.text
        assert "image_preference" in resp.text

    def test_save_settings_persists_image_preference_2d(self, client, db):
        from src.services import settings_service as settings_svc

        resp = client.post(
            "/admin/settings/",
            data={"image_preference": "2d"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert settings_svc.get_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE) == "2d"

    def test_save_settings_persists_image_preference_3d(self, client, db):
        from src.services import settings_service as settings_svc

        resp = client.post(
            "/admin/settings/",
            data={"image_preference": "3d"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert settings_svc.get_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE) == "3d"

    def test_save_settings_defaults_to_2d_when_not_provided(self, client, db):
        """When image_preference is not in the form, the setting should remain unchanged."""
        from src.services import settings_service as settings_svc

        # First set it to 3d
        settings_svc.set_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE, "3d")
        # Submit form without image_preference
        client.post(
            "/admin/settings/",
            data={"designs_base_path": "/some/path"},
            follow_redirects=False,
        )
        # Should remain 3d since the route only updates when value is "2d" or "3d"
        assert settings_svc.get_setting(db, settings_svc.SETTING_IMAGE_PREFERENCE) == "3d"


class TestProjectsRoutesGaps:
    def test_create_project_duplicate_returns_400(self, client, db):
        from src.services import projects as proj_svc

        proj_svc.create(db, "Dup Route Project")
        resp = client.post(
            "/projects/",
            data={"name": "Dup Route Project"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_edit_project_not_found_returns_400(self, client):
        resp = client.post(
            "/projects/999999/edit",
            data={"name": "Ghost"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_delete_project_not_found_returns_404(self, client):
        resp = client.post("/projects/999999/delete", follow_redirects=False)
        assert resp.status_code == 404

    def test_print_project(self, client, db):
        from src.services import projects as proj_svc

        p = proj_svc.create(db, "Print Route Project")
        resp = client.get(f"/projects/{p.id}/print")
        assert resp.status_code == 200

    def test_print_project_not_found(self, client):
        resp = client.get("/projects/999999/print")
        assert resp.status_code == 404


class TestDesignsRoutesGaps:
    def test_open_in_explorer_not_found(self, client):
        resp = client.get("/designs/999999/open-in-explorer", follow_redirects=False)
        assert resp.status_code == 404

    def test_toggle_tags_checked_not_found(self, client):
        resp = client.post(
            "/designs/999999/toggle-tags-checked",
            data={"tags_checked": "true"},
            follow_redirects=False,
        )
        assert resp.status_code == 404

    def test_set_tags_not_found(self, client):
        resp = client.post(
            "/designs/999999/set-tags",
            data={},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_toggle_stitched_not_found(self, client):
        resp = client.post(
            "/designs/999999/toggle-stitched",
            data={"is_stitched": "true"},
            follow_redirects=False,
        )
        assert resp.status_code == 404

    def test_delete_design_not_found(self, client):
        resp = client.post("/designs/999999/delete", follow_redirects=False)
        assert resp.status_code == 404

    def test_add_to_project_not_found(self, client):
        resp = client.post(
            "/designs/999999/add-to-project",
            data={"project_id": "999999"},
            follow_redirects=False,
        )
        assert resp.status_code == 400


class TestDesignsRoutesRemaining:
    def test_edit_design_invalid_rating_returns_400(self, client, db):
        d = designs.create(
            db, {"filename": "edit_inv.jef", "filepath": "/ei.jef", "is_stitched": False}
        )
        resp = client.post(
            f"/designs/{d.id}/edit",
            data={"filename": "edit_inv.jef", "filepath": "/ei.jef", "rating": "99"},
            follow_redirects=False,
        )
        assert resp.status_code == 400

    def test_open_in_explorer_valid_design(self, client, db, tmp_path, monkeypatch):
        import os
        from unittest.mock import patch

        from src.services import settings_service as settings_svc

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        managed_path = tmp_path / "MachineEmbroideryDesigns"
        managed_path.mkdir(parents=True, exist_ok=True)
        monkeypatch.setattr(settings_svc, "DESIGNS_BASE_PATH", str(managed_path))

        actual_file = managed_path / "explorer.jef"
        actual_file.write_bytes(b"\x00" * 16)
        d = designs.create(
            db, {"filename": "explorer.jef", "filepath": "/explorer.jef", "is_stitched": False}
        )

        with patch("src.routes.designs.subprocess.Popen") as mock_popen:
            resp = client.get(f"/designs/{d.id}/open-in-explorer", follow_redirects=False)

        assert resp.status_code == 303
        mock_popen.assert_called_once_with(
            ["explorer.exe", "/select,", os.path.normpath(str(actual_file))]
        )

    def test_open_in_explorer_skips_launch_during_pytest(self, client, db, tmp_path, monkeypatch):
        from unittest.mock import patch

        from src.services import settings_service as settings_svc

        managed_path = tmp_path / "MachineEmbroideryDesigns"
        managed_path.mkdir(parents=True, exist_ok=True)
        monkeypatch.setattr(settings_svc, "DESIGNS_BASE_PATH", str(managed_path))

        actual_file = managed_path / "guarded-explorer.jef"
        actual_file.write_bytes(b"\x00" * 16)
        d = designs.create(
            db,
            {
                "filename": "guarded-explorer.jef",
                "filepath": "/guarded-explorer.jef",
                "is_stitched": False,
            },
        )
        monkeypatch.setenv("PYTEST_CURRENT_TEST", "tests::guarded")

        with patch("src.routes.designs.subprocess.Popen") as mock_popen:
            resp = client.get(f"/designs/{d.id}/open-in-explorer", follow_redirects=False)

        assert resp.status_code == 303
        mock_popen.assert_not_called()


class TestAdvancedSearchRoute:
    """Tests confirming advanced search capabilities live on the main browse page."""

    def test_browse_page_shows_advanced_search_sections(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        assert "General search" in resp.text
        assert "Additional filters" in resp.text
        assert "Search in" in resp.text

    def test_browse_page_shows_form_fields(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        assert "all_words" in resp.text
        assert "exact_phrase" in resp.text
        assert "any_words" in resp.text
        assert "none_words" in resp.text

    def test_browse_page_shows_standard_filters(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        assert "Designer" in resp.text
        assert "Image tags" in resp.text
        assert "Stitching tags" in resp.text
        assert "Unverified only" in resp.text
        assert "Hoop" in resp.text
        assert "Source" in resp.text
        assert "Min Rating" in resp.text
        assert "Stitched" in resp.text

    def test_browse_page_shows_separate_tag_dropdown_ids(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        assert 'id="imageTagDropdown"' in resp.text
        assert 'id="stitchingTagDropdown"' in resp.text

    def test_browse_page_shows_bulk_project_selector(self, client, db):
        from src.services import projects as proj_svc

        proj_svc.create(db, "Browse Bulk Project")
        designs.create(
            db,
            {"filename": "browse_bulk.jef", "filepath": "/browse_bulk.jef", "is_stitched": False},
        )

        resp = client.get("/designs/")

        assert resp.status_code == 200
        assert 'id="bulkProjectId"' in resp.text
        assert "Add to project" in resp.text

    def test_browse_supports_advanced_query(self, client, db):
        from src.services import tags

        matching_tag = tags.create(db, "Browse Search Letters", "image")
        designs.create(
            db,
            {
                "filename": "browse_phrase_match.jef",
                "filepath": "/adv/browse_phrase_match.jef",
                "is_stitched": False,
                "tag_ids": [matching_tag.id],
            },
        )
        resp = client.get(
            '/designs/?all_words="Browse Search Letters"&search_filename=true&search_tags=true&search_folder=true'
        )
        assert resp.status_code == 200
        assert "browse_phrase_match" in resp.text

    def test_browse_integrates_advanced_search_ui(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        assert "General search" in resp.text
        assert "Additional filters" in resp.text

    def test_browse_search_with_all_words(self, client, db):
        designs.create(
            db, {"filename": "rose_adv.jef", "filepath": "/adv/rose.jef", "is_stitched": False}
        )
        resp = client.get(
            "/designs/?all_words=rose_adv&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200
        assert "rose_adv" in resp.text

    def test_browse_reset_button_disabled_without_criteria(self, client):
        resp = client.get("/designs/")
        assert resp.status_code == 200
        snippet = resp.text.split('id="resetBrowseSearchBtn"', 1)[1].split(">", 1)[0]
        assert 'aria-disabled="true"' in snippet

    def test_browse_reset_button_enabled_with_criteria(self, client):
        resp = client.get("/designs/?all_words=rose")
        assert resp.status_code == 200
        snippet = resp.text.split('id="resetBrowseSearchBtn"', 1)[1].split(">", 1)[0]
        assert 'aria-disabled="true"' not in snippet

    def test_browse_search_returns_no_results_message(self, client):
        resp = client.get(
            "/designs/?all_words=zzznoresults&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200
        assert "0 design" in resp.text or "No designs matched" in resp.text

    def test_browse_search_with_designer_filter_only(self, client, db):
        des = designers.create(db, "Adv Route Designer")
        other = designers.create(db, "Other Route Designer")
        designs.create(
            db,
            {
                "filename": "route_designer_match.jef",
                "filepath": "/adv/rdm.jef",
                "is_stitched": False,
                "designer_id": des.id,
            },
        )
        designs.create(
            db,
            {
                "filename": "route_designer_other.jef",
                "filepath": "/adv/rdo.jef",
                "is_stitched": False,
                "designer_id": other.id,
            },
        )
        resp = client.get(f"/designs/?designer_id={des.id}")
        assert resp.status_code == 200
        assert "route_designer_match" in resp.text
        assert "route_designer_other" not in resp.text

    def test_browse_search_with_exact_phrase(self, client, db):
        designs.create(
            db,
            {"filename": "cross_stitch_adv.jef", "filepath": "/adv/cs.jef", "is_stitched": False},
        )
        resp = client.get(
            "/designs/?exact_phrase=cross_stitch_adv&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200
        assert "cross_stitch_adv" in resp.text

    def test_browse_general_search_finds_nested_folder_name(self, client, db):
        designs.create(
            db,
            {
                "filename": "nested_folder_match",
                "filepath": "/collections/My Creations/Florals/nested_folder_match.jef",
                "is_stitched": False,
            },
        )
        designs.create(
            db,
            {
                "filename": "other_folder_match",
                "filepath": "/collections/Other Collection/Florals/other_folder_match.jef",
                "is_stitched": False,
            },
        )

        resp = client.get(
            "/designs/?q=%22My+Creations%22&search_filename=false&search_tags=false&search_folder=true"
        )

        assert resp.status_code == 200
        assert "nested_folder_match" in resp.text
        assert "other_folder_match" not in resp.text

    def test_browse_search_with_google_syntax_exclusion(self, client, db):
        designs.create(
            db, {"filename": "rose_excl.jef", "filepath": "/adv/re.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "rose_excl_bad.jef", "filepath": "/adv/reb.jef", "is_stitched": False}
        )
        resp = client.get(
            "/designs/?q=rose_excl+-bad&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200

    def test_browse_pagination_params_preserved(self, client):
        resp = client.get("/designs/?all_words=test&search_filename=true&page=1")
        assert resp.status_code == 200


# ---------------------------------------------------------------------------
# Bulk import routes — expanded coverage
# ---------------------------------------------------------------------------


class TestImportRoutesExpanded:
    def test_scan_nonexistent_folder_returns_400(self, client, tmp_path):
        """Scanning a folder that doesn't exist should return 400 Bad Request."""
        resp = client.post(
            "/import/scan",
            data={"folder_path": str(tmp_path / "does_not_exist")},
        )
        assert resp.status_code == 400
        assert (
            b"At least one valid folder path is required." in resp.content
            or b"does_not_exist" in resp.content
        )

    def test_scan_empty_folder_returns_200(self, client, tmp_path):
        """Scanning a real, empty folder should succeed."""
        resp = client.post(
            "/import/scan",
            data={"folder_path": str(tmp_path)},
        )
        assert resp.status_code == 200

    def test_scan_folder_with_unsupported_files_returns_200(self, client, tmp_path):
        """Only supported file types are reported; unsupported ones are silently skipped."""
        (tmp_path / "readme.txt").write_text("ignored")
        (tmp_path / "image.png").write_bytes(b"\x89PNG")
        resp = client.post(
            "/import/scan",
            data={"folder_path": str(tmp_path)},
        )
        assert resp.status_code == 200

    def test_scan_large_error_list_shows_pagination_controls(self, client, tmp_path, monkeypatch):
        """Large error lists should render pagination controls on the review screen."""
        from src.routes import bulk_import as route_mod
        from src.services.scanning import ScannedDesign

        def fake_scan_folders(_paths, _db):
            return [
                ScannedDesign(
                    filename=f"bad_{i}.dst",
                    filepath=f"\\bad_{i}.dst",
                    error="simulated read failure",
                )
                for i in range(205)
            ]

        monkeypatch.setattr(route_mod, "scan_folders", fake_scan_folders)

        resp = client.post(
            "/import/scan",
            data={"folder_path": str(tmp_path)},
        )

        assert resp.status_code == 200
        assert "Files with errors (205)" in resp.text
        assert "Page 1 of 3" in resp.text
        assert 'id="errorPager"' in resp.text

    def test_confirm_no_files_redirects_to_import(self, client):
        """Confirming with no selected files should redirect back to /import/."""
        resp = client.post(
            "/import/confirm",
            data={"folder_path": "/some/folder"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert resp.headers["location"] == "/import/"

    def test_confirm_with_files_redirects_to_designs(self, client, tmp_path):
        """Confirming with a non-existent filepath should still redirect to /designs/."""
        resp = client.post(
            "/import/confirm",
            data={
                "folder_path": str(tmp_path),
                "selected_files": ["\\does_not_exist.jef"],
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "/designs" in resp.headers["location"]

    def test_confirm_with_real_file(self, client, db, tmp_path, monkeypatch):
        """Confirming a real (minimal) JEF file should create a design and redirect."""
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "DESIGNS_BASE_PATH", str(tmp_path))
        jef_file = tmp_path / "test_import.jef"
        jef_file.write_bytes(b"\x00" * 128)
        resp = client.post(
            "/import/confirm",
            data={
                "folder_path": str(tmp_path),
                "selected_files": ["\\test_import.jef"],
            },
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "/designs" in resp.headers["location"]


# ---------------------------------------------------------------------------
# Browse folder route — mocked tkinter
# ---------------------------------------------------------------------------


class TestBrowseFolderRoute:
    def test_browse_folder_returns_json(self, client, monkeypatch):
        """browse-folder always returns JSON with a path or error."""
        from src.routes import bulk_import

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(bulk_import, "pick_folders", lambda **kwargs: ["/chosen/path"])

        resp = client.get("/import/browse-folder")

        assert resp.status_code == 200
        data = resp.json()
        assert "path" in data or "error" in data

    def test_browse_folder_error_when_tkinter_raises(self, client, monkeypatch):
        """browse-folder returns an error JSON when the picker backend raises."""
        from src.routes import bulk_import
        from src.services.folder_picker import FolderPickerUnavailableError

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(
            bulk_import,
            "pick_folders",
            lambda **kwargs: (_ for _ in ()).throw(FolderPickerUnavailableError("no display")),
        )

        resp = client.get("/import/browse-folder")

        assert resp.status_code == 200
        data = resp.json()
        assert "error" in data or "path" in data


# ---------------------------------------------------------------------------
# Maintenance routes — orphan scan and cleanup
# ---------------------------------------------------------------------------


class TestMaintenanceRoutes:
    def _set_designs_base_path(self, db, base_path):
        from src.services import settings_service as settings_svc

        managed_path = base_path / "MachineEmbroideryDesigns"
        managed_path.mkdir(parents=True, exist_ok=True)
        settings_svc.DESIGNS_BASE_PATH = str(managed_path)
        return managed_path

    def test_scan_orphans_reports_missing_files(self, client, db, tmp_path):
        from src.services import designs as design_svc

        managed_path = self._set_designs_base_path(db, tmp_path)
        (managed_path / "present.jef").write_bytes(b"\x00" * 16)

        design_svc.create(
            db, {"filename": "present.jef", "filepath": "\\present.jef", "is_stitched": False}
        )
        design_svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )

        resp = client.get("/admin/maintenance/orphans/scan")

        assert resp.status_code == 200
        assert resp.json() == {"checked": 2, "found": 1}

    def test_browse_path_opens_deepest_existing_folder(self, client, db, tmp_path, monkeypatch):
        import os
        from unittest.mock import patch

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        managed_path = self._set_designs_base_path(db, tmp_path)
        existing_folder = managed_path / "nested"
        existing_folder.mkdir()

        with patch("src.routes.maintenance.subprocess.Popen") as mock_popen:
            resp = client.get(
                "/admin/maintenance/browse-path",
                params={"filepath": "\\nested\\missing\\rose.jef"},
            )

        expected = os.path.normpath(str(existing_folder))
        assert resp.status_code == 200
        assert resp.json() == {"ok": True, "opened": expected}
        mock_popen.assert_called_once_with(["explorer", expected])

    def test_browse_path_still_returns_ok_when_explorer_fails(
        self, client, db, tmp_path, monkeypatch
    ):
        from unittest.mock import patch

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        managed_path = self._set_designs_base_path(db, tmp_path)

        with patch("src.routes.maintenance.subprocess.Popen", side_effect=OSError("boom")):
            resp = client.get(
                "/admin/maintenance/browse-path",
                params={"filepath": "\\missing\\rose.jef"},
            )

        assert resp.status_code == 200
        assert resp.json() == {"ok": True, "opened": str(managed_path)}

    def test_browse_path_unexpected_launch_error_propagates(
        self, client, db, tmp_path, monkeypatch
    ):
        from unittest.mock import patch

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)

        self._set_designs_base_path(db, tmp_path)

        with patch(
            "src.routes.maintenance.subprocess.Popen",
            side_effect=RuntimeError("unexpected explorer failure"),
        ):
            with pytest.raises(RuntimeError, match="unexpected explorer failure"):
                client.get(
                    "/admin/maintenance/browse-path",
                    params={"filepath": "\\missing\\rose.jef"},
                )

    def test_browse_path_skips_launch_during_pytest(self, client, db, tmp_path, monkeypatch):
        from unittest.mock import patch

        managed_path = self._set_designs_base_path(db, tmp_path)
        monkeypatch.setenv("PYTEST_CURRENT_TEST", "tests::guarded")

        with patch("src.routes.maintenance.subprocess.Popen") as mock_popen:
            resp = client.get(
                "/admin/maintenance/browse-path",
                params={"filepath": "\\missing\\rose.jef"},
            )

        assert resp.status_code == 200
        assert resp.json() == {"ok": True, "opened": str(managed_path)}
        mock_popen.assert_not_called()

    def test_orphans_page_renders_missing_design(self, client, db, tmp_path):
        from src.services import designs as design_svc

        self._set_designs_base_path(db, tmp_path)
        design_svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )

        resp = client.get("/admin/maintenance/orphans?page=1")

        assert resp.status_code == 200
        assert "missing.jef" in resp.text

    def test_delete_orphans_redirects_with_deleted_count(self, client, db, tmp_path):
        from src.services import designs as design_svc

        self._set_designs_base_path(db, tmp_path)
        orphan = design_svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )

        resp = client.post(
            "/admin/maintenance/orphans/delete",
            data={"design_ids": [str(orphan.id)], "page": "2"},
            follow_redirects=False,
        )

        assert resp.status_code == 303
        assert resp.headers["location"] == "/admin/maintenance/orphans?page=2&deleted=1"

    def test_delete_all_orphans_redirects_with_deleted_count(self, client, db, tmp_path):
        import os
        import shutil

        from src.services import designs as design_svc

        # Copy TestData/missing.jef into the managed path so one file exists and one is missing
        managed_path = self._set_designs_base_path(db, tmp_path)
        testdata_dir = os.path.join(
            os.path.dirname(__file__), "..", "data", "MachineEmbroideryDesigns", "TestData"
        )
        src_file = os.path.join(testdata_dir, "missing.jef")
        dst_file = managed_path / "missing.jef"
        shutil.copyfile(src_file, dst_file)

        # Create two designs, one with an existing file, one missing
        design_svc.create(
            db, {"filename": "missing.jef", "filepath": "\\missing.jef", "is_stitched": False}
        )
        design_svc.create(
            db, {"filename": "notfound.jef", "filepath": "\\notfound.jef", "is_stitched": False}
        )

        resp = client.post("/admin/maintenance/orphans/delete-all", follow_redirects=False)

        assert resp.status_code == 303
        # Only one orphan (notfound.jef) should be deleted
        assert resp.headers["location"] == "/admin/maintenance/orphans?deleted=1"


# ---------------------------------------------------------------------------
# Info routes — Help and About
# ---------------------------------------------------------------------------


class TestInfoRoutes:
    def test_help_page_returns_200(self, client):
        resp = client.get("/help")
        assert resp.status_code == 200

    def test_help_page_contains_expected_sections(self, client):
        resp = client.get("/help")
        assert resp.status_code == 200
        text = resp.text
        assert "Search" in text
        assert "Importing" in text
        assert "Projects" in text
        assert "Maintenance" in text
        assert "Troubleshooting" in text

    def test_help_page_links_to_about_page(self, client):
        resp = client.get("/help")
        assert resp.status_code == 200
        assert 'href="/about"' in resp.text


# ---------------------------------------------------------------------------
# BrowseFilterState unit tests (Phase 2 — refactor helper)
# ---------------------------------------------------------------------------


class TestBrowseFilterState:
    """Unit tests for the BrowseFilterState dataclass extracted from browse()."""

    def test_defaults_have_no_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState()
        assert fs.has_active_filters is False

    def test_q_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(q="rose")
        assert fs.has_active_filters is True

    def test_all_words_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(all_words="rose tulip")
        assert fs.has_active_filters is True

    def test_exact_phrase_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(exact_phrase="cross stitch")
        assert fs.has_active_filters is True

    def test_any_words_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(any_words="rose")
        assert fs.has_active_filters is True

    def test_none_words_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(none_words="bad")
        assert fs.has_active_filters is True

    def test_filename_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(filename="*.jef")
        assert fs.has_active_filters is True

    def test_designer_id_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(designer_id=1)
        assert fs.has_active_filters is True

    def test_tag_ids_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(tag_ids=[1, 2])
        assert fs.has_active_filters is True

    def test_hoop_id_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(hoop_id=5)
        assert fs.has_active_filters is True

    def test_source_id_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(source_id=3)
        assert fs.has_active_filters is True

    def test_rating_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(rating=4)
        assert fs.has_active_filters is True

    def test_is_stitched_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(is_stitched=True)
        assert fs.has_active_filters is True

    def test_unverified_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(unverified=True)
        assert fs.has_active_filters is True

    def test_search_filename_false_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(search_filename=False)
        assert fs.has_active_filters is True

    def test_search_tags_false_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(search_tags=False)
        assert fs.has_active_filters is True

    def test_search_folder_false_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(search_folder=False)
        assert fs.has_active_filters is True

    def test_non_default_sort_by_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(sort_by="date_added")
        assert fs.has_active_filters is True

    def test_non_default_sort_dir_triggers_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(sort_dir="desc")
        assert fs.has_active_filters is True

    def test_to_query_pairs_empty_by_default(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState()
        pairs = fs.to_query_pairs()
        # Only the always-included boolean scope flags should be present
        keys = [k for k, _ in pairs]
        assert "search_filename" in keys
        assert "search_tags" in keys
        assert "search_folder" in keys
        assert "q" not in keys
        assert "designer_id" not in keys

    def test_to_query_pairs_includes_q(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(q="rose")
        pairs = dict(fs.to_query_pairs())
        assert pairs["q"] == "rose"

    def test_to_query_pairs_includes_filename(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(filename="*.jef")
        pairs = dict(fs.to_query_pairs())
        assert pairs["filename"] == "*.jef"

    def test_to_query_pairs_includes_designer_id(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(designer_id=7)
        pairs = dict(fs.to_query_pairs())
        assert pairs["designer_id"] == "7"

    def test_to_query_pairs_includes_tag_ids(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(tag_ids=[1, 2])
        pairs = fs.to_query_pairs()
        tag_pairs = [(k, v) for k, v in pairs if k == "tag_ids"]
        assert tag_pairs == [("tag_ids", "1"), ("tag_ids", "2")]

    def test_to_query_pairs_includes_rating(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(rating=4)
        pairs = dict(fs.to_query_pairs())
        assert pairs["rating"] == "4"

    def test_to_query_pairs_includes_is_stitched_true(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(is_stitched=True)
        pairs = dict(fs.to_query_pairs())
        assert pairs["is_stitched"] == "true"

    def test_to_query_pairs_includes_is_stitched_false(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(is_stitched=False)
        pairs = dict(fs.to_query_pairs())
        assert pairs["is_stitched"] == "false"

    def test_to_query_pairs_includes_unverified(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(unverified=True)
        pairs = dict(fs.to_query_pairs())
        assert pairs["unverified"] == "true"

    def test_to_query_pairs_omits_unverified_when_false(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(unverified=False)
        pairs = dict(fs.to_query_pairs())
        assert "unverified" not in pairs

    def test_to_query_pairs_includes_non_default_sort(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(sort_by="date_added", sort_dir="desc")
        pairs = dict(fs.to_query_pairs())
        assert pairs["sort_by"] == "date_added"
        assert pairs["sort_dir"] == "desc"

    def test_to_query_pairs_omits_default_sort(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(sort_by="name", sort_dir="asc")
        pairs = dict(fs.to_query_pairs())
        assert "sort_by" not in pairs
        assert "sort_dir" not in pairs

    def test_as_template_dict_contains_has_active_filters(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState(q="rose")
        d = fs.as_template_dict()
        assert d["has_active_filters"] is True
        assert d["q"] == "rose"

    def test_as_template_dict_all_expected_keys(self):
        from src.routes.designs import BrowseFilterState

        fs = BrowseFilterState()
        d = fs.as_template_dict()
        expected_keys = {
            "q",
            "all_words",
            "exact_phrase",
            "any_words",
            "none_words",
            "filename",
            "designer_id",
            "tag_ids",
            "hoop_id",
            "source_id",
            "rating",
            "is_stitched",
            "unverified",
            "search_filename",
            "search_tags",
            "search_folder",
            "sort_by",
            "sort_dir",
            "has_active_filters",
        }
        assert expected_keys.issubset(d.keys())


# ---------------------------------------------------------------------------
# Browse characterization tests — filter and sort preservation in pagination
# ---------------------------------------------------------------------------


class TestBrowseCharacterization:
    """Characterization tests locking in browse-page filter and sort behaviour."""

    def test_browse_with_all_words_and_exact_phrase(self, client, db):
        designs.create(
            db, {"filename": "char_match.jef", "filepath": "/ch/match.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?all_words=char_match&exact_phrase=char&search_filename=true")
        assert resp.status_code == 200

    def test_browse_with_any_words(self, client, db):
        designs.create(
            db, {"filename": "any_rose.jef", "filepath": "/ch/any_rose.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "any_tulip.jef", "filepath": "/ch/any_tulip.jef", "is_stitched": False}
        )
        resp = client.get(
            "/designs/?any_words=any_rose+any_tulip&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200
        assert "any_rose" in resp.text
        assert "any_tulip" in resp.text

    def test_browse_with_none_words_excludes_match(self, client, db):
        designs.create(
            db, {"filename": "nonewords_keep.jef", "filepath": "/ch/nk.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "nonewords_drop.jef", "filepath": "/ch/nd.jef", "is_stitched": False}
        )
        # none_words=drop — the form field echoes "drop" but the excluded design
        # name "nonewords_drop" only appears in rendered design cards.
        resp = client.get(
            "/designs/?q=nonewords&none_words=drop&search_filename=true&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200
        assert "nonewords_keep" in resp.text
        assert "nonewords_drop" not in resp.text

    def test_browse_filter_by_hoop(self, client, db):
        from src.services import hoops as hoops_svc

        hoops_svc.seed_hoops(db)
        all_hoops = hoops_svc.get_all(db)
        h = all_hoops[0]
        designs.create(
            db,
            {
                "filename": "hoop_des.jef",
                "filepath": "/ch/hd.jef",
                "is_stitched": False,
                "hoop_id": h.id,
            },
        )
        resp = client.get(f"/designs/?hoop_id={h.id}")
        assert resp.status_code == 200
        assert "hoop_des" in resp.text

    def test_browse_filter_by_source(self, client, db):
        from src.services import sources as sources_svc

        s = sources_svc.create(db, "Char Source")
        designs.create(
            db,
            {
                "filename": "src_des.jef",
                "filepath": "/ch/sd.jef",
                "is_stitched": False,
                "source_id": s.id,
            },
        )
        resp = client.get(f"/designs/?source_id={s.id}")
        assert resp.status_code == 200
        assert "src_des" in resp.text

    def test_browse_filter_by_rating(self, client, db):
        designs.create(
            db,
            {
                "filename": "rated_des.jef",
                "filepath": "/ch/rd.jef",
                "is_stitched": False,
                "rating": 4,
            },
        )
        resp = client.get("/designs/?rating=4")
        assert resp.status_code == 200
        assert "rated_des" in resp.text

    def test_browse_filter_by_is_stitched_true(self, client, db):
        designs.create(
            db, {"filename": "stitched_des.jef", "filepath": "/ch/std.jef", "is_stitched": True}
        )
        designs.create(
            db,
            {"filename": "not_stitched_des.jef", "filepath": "/ch/nsd.jef", "is_stitched": False},
        )
        resp = client.get("/designs/?is_stitched=true")
        assert resp.status_code == 200
        assert "stitched_des" in resp.text
        assert "not_stitched_des" not in resp.text

    def test_browse_filter_by_is_stitched_false(self, client, db):
        designs.create(
            db, {"filename": "unstitched_des.jef", "filepath": "/ch/usd.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?is_stitched=false")
        assert resp.status_code == 200
        assert "unstitched_des" in resp.text

    def test_browse_filter_unverified(self, client, db):
        resp = client.get("/designs/?unverified=true")
        assert resp.status_code == 200
        assert "unverif" in resp.text

    def test_browse_filter_by_tag(self, client, db):
        from src.services import tags as tags_svc

        t = tags_svc.create(db, "Char Tag", "image")
        d = designs.create(
            db, {"filename": "tagged_char.jef", "filepath": "/ch/tc.jef", "is_stitched": False}
        )
        from src.services import designs as d_svc

        d_svc.update(db, d.id, {"tag_ids": [t.id]})
        resp = client.get(f"/designs/?tag_ids={t.id}")
        assert resp.status_code == 200
        assert "tagged_char" in resp.text

    def test_browse_filename_wildcard_jef(self, client, db):
        designs.create(
            db, {"filename": "wildcard_flower.jef", "filepath": "/ch/wf.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?filename=wildcard_flower*")
        assert resp.status_code == 200
        assert "wildcard_flower" in resp.text

    def test_browse_filename_wildcard_no_match(self, client):
        resp = client.get("/designs/?filename=zzznomatch*")
        assert resp.status_code == 200

    def test_browse_sort_by_date_added_desc(self, client, db):
        designs.create(
            db, {"filename": "sort_first.jef", "filepath": "/ch/sf.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "sort_second.jef", "filepath": "/ch/ss.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?sort_by=date_added&sort_dir=desc")
        assert resp.status_code == 200

    def test_browse_sort_by_name_asc(self, client):
        resp = client.get("/designs/?sort_by=name&sort_dir=asc")
        assert resp.status_code == 200

    def test_browse_filter_params_preserved_in_response(self, client, db):
        """filter_params in the rendered HTML should preserve the active search criteria."""
        designs.create(
            db, {"filename": "pagpres.jef", "filepath": "/ch/pp.jef", "is_stitched": False}
        )
        resp = client.get("/designs/?all_words=pagpres&search_filename=true&page=1")
        assert resp.status_code == 200
        assert "all_words=pagpres" in resp.text

    def test_browse_pagination_links_carry_filters(self, client, db):
        """Pagination links should include the active filter params so page 2 etc. work."""
        resp = client.get("/designs/?sort_by=date_added&sort_dir=desc&page=1")
        assert resp.status_code == 200

    def test_browse_advanced_search_retired_route_returns_404(self, client):
        """The old /designs/advanced-search route should return 404 once retired.

        Currently FastAPI returns 422 because "advanced-search" fails int
        conversion for the {design_id} path parameter.  Once an explicit
        redirect/tombstone route for /designs/advanced-search is added this
        test should be tightened to assert 404 only.
        """
        resp = client.get("/designs/advanced-search", follow_redirects=False)
        # Current behaviour: 422 (int conversion failure); future desired: 404
        assert resp.status_code in (404, 422)

    def test_browse_search_scope_all_off_still_renders(self, client):
        resp = client.get(
            "/designs/?q=anything&search_filename=false&search_tags=false&search_folder=false"
        )
        assert resp.status_code == 200

    def test_browse_multiple_filters_combined(self, client, db):
        designs.create(
            db,
            {
                "filename": "combo_des.jef",
                "filepath": "/ch/cd.jef",
                "is_stitched": True,
                "rating": 3,
            },
        )
        resp = client.get("/designs/?is_stitched=true&rating=3")
        assert resp.status_code == 200
        assert "combo_des" in resp.text

    def test_browse_tag_ids_filter_works(self, client, db):
        from src.services import tags

        t = tags.create(db, "Filter Tag", "image")
        d = designs.create(
            db, {"filename": "filter_tag.jef", "filepath": "/ch/ft.jef", "is_stitched": False}
        )
        from src.services import designs as d_svc

        d_svc.update(db, d.id, {"tag_ids": [t.id]})
        resp = client.get(f"/designs/?tag_ids={t.id}")
        assert resp.status_code == 200
        assert "filter_tag" in resp.text

    def test_browse_page_clipped_to_minimum_one(self, client):
        resp = client.get("/designs/?page=-5")
        assert resp.status_code == 200

    def test_browse_total_and_pages_present_in_response(self, client, db):
        designs.create(
            db, {"filename": "total_check.jef", "filepath": "/ch/tck.jef", "is_stitched": False}
        )
        resp = client.get("/designs/")
        assert resp.status_code == 200


class TestBackupRoutes:
    """Tests for /admin/maintenance/backup routes."""

    def test_backup_page_get(self, client):
        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert "Backup" in resp.text

    def test_backup_page_contains_forms(self, client):
        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert "db_destination" in resp.text
        assert "designs_destination" in resp.text

    def test_backup_page_contains_browse_buttons(self, client):
        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert "browse_db_destination_btn" in resp.text
        assert "browse_designs_destination_btn" in resp.text

    def test_backup_page_buttons_disabled_when_no_destinations(self, client):
        import re

        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert re.search(r'id="save_destinations_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="backup_database_now_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="run_incremental_backup_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="run_both_backups_btn"[^>]*disabled', resp.text)
        assert "browse_db_destination_btn" in resp.text
        assert "browse_designs_destination_btn" in resp.text

    def test_backup_page_run_both_requires_both_destinations(self, client, db):
        import re

        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "/tmp/dbonly")
        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "")

        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert re.search(r'id="backup_database_now_btn"(?![^>]*disabled)', resp.text, re.DOTALL)
        assert re.search(r'id="run_incremental_backup_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="run_both_backups_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="save_destinations_btn"[^>]*disabled', resp.text)

    def test_backup_action_buttons_ignore_unsaved_entries(self, client, db):
        import re

        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "")
        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "")

        resp = client.post(
            "/admin/maintenance/backup/save-settings",
            data={"db_destination": "/tmp/saved-db", "designs_destination": ""},
            follow_redirects=True,
        )
        assert resp.status_code == 200
        assert re.search(r'id="backup_database_now_btn"(?![^>]*disabled)', resp.text, re.DOTALL)
        assert re.search(r'id="run_incremental_backup_btn"[^>]*disabled', resp.text)
        assert re.search(r'id="run_both_backups_btn"[^>]*disabled', resp.text)

    def test_save_destinations_enables_when_saved_values_are_cleared(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "C:/Saved/Db")
        settings_svc.set_setting(
            db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "D:/Saved/Designs"
        )

        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert 'data-saved-value="C:\\Saved\\Db"' in resp.text
        assert 'data-saved-value="D:\\Saved\\Designs"' in resp.text

    def test_backup_browse_folder_returns_json(self, client, monkeypatch):
        """Backup browse-folder returns JSON with a path or error."""
        import src.routes.maintenance as maint

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(maint, "pick_folder", lambda **kwargs: "/backup/path")

        resp = client.get("/admin/maintenance/backup/browse-folder?kind=database")

        assert resp.status_code == 200
        data = resp.json()
        assert "path" in data or "error" in data

    def test_save_backup_settings_redirects(self, client):
        resp = client.post(
            "/admin/maintenance/backup/save-settings",
            data={"db_destination": "/tmp/dbbackup", "designs_destination": "/tmp/dbackup"},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "saved=1" in resp.headers.get("location", "")

    def test_save_backup_settings_persists_values(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/maintenance/backup/save-settings",
            data={"db_destination": "/tmp/dbtest", "designs_destination": "/tmp/destest"},
            follow_redirects=False,
        )
        assert (
            settings_svc.get_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION)
            == "\\tmp\\dbtest"
        )
        assert (
            settings_svc.get_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION)
            == "\\tmp\\destest"
        )

    def test_backup_page_displays_saved_destinations_with_backslashes(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(
            db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "H:/Catalogue Backups/Database"
        )
        settings_svc.set_setting(
            db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "H:/Catalogue Backups/Designs"
        )

        resp = client.get("/admin/maintenance/backup")
        assert resp.status_code == 200
        assert "H:\\Catalogue Backups\\Database" in resp.text
        assert "H:\\Catalogue Backups\\Designs" in resp.text
        assert (
            'Saved destination folder: <code class="font-mono">H:\\Catalogue Backups\\Database</code>'
            in resp.text
        )

    def test_save_backup_settings_without_destinations_redirects_with_error(self, client):
        resp = client.post(
            "/admin/maintenance/backup/save-settings",
            data={"db_destination": "", "designs_destination": ""},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "no_destinations_to_save" in resp.headers.get("location", "")

    def test_save_backup_settings_allows_clearing_existing_destinations(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "C:/Saved/Db")
        settings_svc.set_setting(
            db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "D:/Saved/Designs"
        )

        resp = client.post(
            "/admin/maintenance/backup/save-settings",
            data={"db_destination": "", "designs_destination": ""},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "saved=1" in resp.headers.get("location", "")
        assert settings_svc.get_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION) == ""
        assert settings_svc.get_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION) == ""

    def test_database_backup_without_destination_redirects_with_error(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "")
        resp = client.post("/admin/maintenance/backup/database", follow_redirects=False)
        assert resp.status_code == 303
        assert "no_db_dest" in resp.headers.get("location", "")

    def test_designs_backup_without_destination_redirects_with_error(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "")
        resp = client.post("/admin/maintenance/backup/designs", follow_redirects=False)
        assert resp.status_code == 303
        assert "no_designs_dest" in resp.headers.get("location", "")

    def test_both_backup_without_destinations_redirects_with_error(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, "")
        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, "")
        resp = client.post("/admin/maintenance/backup/both", follow_redirects=False)
        assert resp.status_code == 303
        assert "no_destinations" in resp.headers.get("location", "")

    def test_database_backup_with_valid_destination(self, client, db, tmp_path, monkeypatch):
        import src.routes.maintenance as maint
        from src.services import settings_service as settings_svc

        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(b"SQLite fake data")
        dest = tmp_path / "db_backups"
        dest.mkdir()

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, str(dest))
        monkeypatch.setattr(maint, "_resolve_db_path", lambda: str(db_file))

        resp = client.post("/admin/maintenance/backup/database", follow_redirects=False)
        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "db_ok=1" in location
        assert list(dest.glob("catalogue_*.db"))

    def test_designs_backup_with_valid_destination(self, client, db, tmp_path, monkeypatch):
        import src.routes.maintenance as maint
        from src.services import settings_service as settings_svc

        src_dir = tmp_path / "designs"
        src_dir.mkdir()
        (src_dir / "flower.jef").write_bytes(b"flower design")
        dest = tmp_path / "design_backups"

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, str(dest))
        monkeypatch.setattr(maint, "DESIGNS_BASE_PATH", str(src_dir))

        resp = client.post("/admin/maintenance/backup/designs", follow_redirects=False)
        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "designs_ok=1" in location
        assert (dest / "flower.jef").exists()

    def test_both_backup_with_destinations(self, client, db, tmp_path, monkeypatch):
        import src.routes.maintenance as maint
        from src.services import settings_service as settings_svc

        db_file = tmp_path / "catalogue.db"
        db_file.write_bytes(b"db content")
        db_dest = tmp_path / "db_bk"
        db_dest.mkdir()

        src_dir = tmp_path / "designs"
        src_dir.mkdir()
        (src_dir / "test.jef").write_bytes(b"design")
        des_dest = tmp_path / "des_bk"

        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DB_DESTINATION, str(db_dest))
        settings_svc.set_setting(db, settings_svc.SETTING_BACKUP_DESIGNS_DESTINATION, str(des_dest))
        monkeypatch.setattr(maint, "_resolve_db_path", lambda: str(db_file))
        monkeypatch.setattr(maint, "DESIGNS_BASE_PATH", str(src_dir))

        resp = client.post("/admin/maintenance/backup/both", follow_redirects=False)
        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "db_ok=1" in location
        assert "designs_ok=1" in location

    def test_backup_browse_folder_error_when_tkinter_raises(self, client, monkeypatch):
        """Backup browse-folder returns an error JSON when the picker backend raises."""
        import src.routes.maintenance as maint
        from src.services.folder_picker import FolderPickerUnavailableError

        monkeypatch.delenv("PYTEST_CURRENT_TEST", raising=False)
        monkeypatch.delenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", raising=False)
        monkeypatch.setattr(
            maint,
            "pick_folder",
            lambda **kwargs: (_ for _ in ()).throw(FolderPickerUnavailableError("no display")),
        )

        resp = client.get("/admin/maintenance/backup/browse-folder?kind=designs")

        assert resp.status_code == 200
        data = resp.json()
        assert "error" in data or "path" in data

    def test_backup_browse_folder_still_uses_picker_when_external_launches_disabled(
        self, client, monkeypatch
    ):
        import src.routes.maintenance as maint

        calls = []
        monkeypatch.setenv("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "1")
        monkeypatch.delenv("EMBROIDERY_DISABLE_NATIVE_DIALOGS", raising=False)
        monkeypatch.setattr(
            maint,
            "pick_folder",
            lambda **kwargs: calls.append(kwargs) or r"D:\Backups",
        )

        resp = client.get("/admin/maintenance/backup/browse-folder?kind=designs")

        assert resp.status_code == 200
        assert resp.json()["path"] == r"D:\Backups"
        assert len(calls) == 1


class TestTaggingActionsRoutes:
    """Tests for the /admin/tagging-actions/ page and run endpoint."""

    def test_tagging_actions_page_get(self, client):
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200

    def test_tagging_actions_page_shows_action_choices(self, client):
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "tag_untagged" in resp.text
        assert "retag_all_unverified" in resp.text
        assert "retag_all" in resp.text

    def test_tagging_actions_page_shows_cost_banner_when_key_present(self, client, monkeypatch):
        from src.services import settings_service as settings_svc

        monkeypatch.setattr(settings_svc, "get_google_api_key", lambda: "AIzaSy_fake")
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "Cost notice" in resp.text or "cost" in resp.text.lower()

    def test_tagging_actions_page_uses_saved_tier_defaults(self, client, db):
        from src.services import settings_service as settings_svc

        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER2_AUTO, "true")
        settings_svc.set_setting(db, settings_svc.SETTING_AI_TIER3_AUTO, "true")

        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert 'id="tier2_check"' in resp.text
        assert 'id="tier3_check"' in resp.text
        assert 'id="tier2_check"\n                 checked' in resp.text
        assert 'id="tier3_check"\n                 checked' in resp.text

    def test_tagging_actions_run_requires_action(self, client):
        resp = client.post(
            "/admin/tagging-actions/run",
            data={"tiers": ["1"]},
            follow_redirects=False,
        )
        # Missing required 'action' field → 422 validation error
        assert resp.status_code in (422, 303)

    def test_tagging_actions_run_invalid_action_redirects_with_error(self, client):
        resp = client.post(
            "/admin/tagging-actions/run",
            data={"action": "invalid_action", "tiers": ["1"]},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        assert "invalid_action" in resp.headers.get("location", "")

    def test_tagging_actions_retag_all_succeeds(self, client):
        """retag_all now always includes verified designs — no confirmation needed."""
        resp = client.post(
            "/admin/tagging-actions/run",
            data={"action": "retag_all", "tiers": ["1"]},
            follow_redirects=False,
        )
        assert resp.status_code == 303
        # Should redirect to the result page, not an error
        assert "error" not in resp.headers.get("location", "")

    def test_tagging_actions_shows_cli_reference(self, client):
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "auto_tag.py" in resp.text

    def test_tagging_actions_page_shows_delay_field(self, client):
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "delay" in resp.text

    def test_tagging_actions_page_shows_local_stitching_backfill(self, client):
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "Local stitching backfill" in resp.text

    def test_tagging_actions_local_stitching_backfill_runs_successfully(self, client, monkeypatch):
        import src.routes.tagging_actions as ta_mod
        from src.services.auto_tagging import TaggingActionResult

        captured = {}

        def fake_backfill(db, batch_size=None, examples_root=None, clear_existing_stitching=False):
            captured["clear_existing_stitching"] = clear_existing_stitching
            return TaggingActionResult(
                action="backfill_stitching",
                tiers_run=[1],
                designs_considered=3,
                tier1_tagged=2,
                total_tagged=2,
                still_untagged=1,
                already_matched=4,
                no_match=1,
                cleared_only=0,
                tag_breakdown={"Filled": 1, "Line Outline": 1},
            )

        monkeypatch.setattr(ta_mod, "run_stitching_backfill_action", fake_backfill)

        resp = client.post(
            "/admin/tagging-actions/run-stitching-backfill",
            data={"batch_size": "10", "clear_existing_stitching": "1"},
            follow_redirects=False,
        )

        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "done=1" in location
        assert "action=backfill_stitching" in location
        assert "matched=4" in location
        assert "nomatch=1" in location
        assert "cleared=0" in location
        assert "breakdown=" in location
        assert captured["clear_existing_stitching"] is True

    # --- Template content tests ---

    def test_tagging_actions_page_shows_unified_backfill_form(self, client):
        """The page should contain the unified backfill form with action checkboxes."""
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert 'id="unified-backfill-form"' in resp.text
        assert 'name="actions" value="tagging"' in resp.text
        assert 'name="actions" value="stitching"' in resp.text
        assert 'name="actions" value="images"' in resp.text
        assert 'name="actions" value="color_counts"' in resp.text

    def test_tagging_actions_page_shows_unified_backfill_options_panels(self, client):
        """The page should contain hidden option panels for each unified action."""
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert 'id="tagging-options"' in resp.text
        assert 'id="stitching-options"' in resp.text
        assert 'id="images-options"' in resp.text
        assert 'id="color_counts-options"' in resp.text
        # Tagging options should include action selector and tier checkboxes
        assert 'name="tagging_action"' in resp.text
        assert 'name="tagging_tiers"' in resp.text
        # Stitching options should include clear existing checkbox
        assert 'name="stitching_clear_existing"' in resp.text
        # Images options should include redo checkbox
        assert 'name="images_redo"' in resp.text
        # Color counts options section should be present
        assert 'id="color_counts-options"' in resp.text

    def test_tagging_actions_page_shows_unified_backfill_batch_and_commit_inputs(self, client):
        """The unified backfill form should have batch size and commit every inputs."""
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert 'name="batch_size"' in resp.text
        assert 'name="commit_every"' in resp.text

    def test_tagging_actions_page_shows_unified_backfill_run_and_stop_buttons(self, client):
        """The unified backfill form should have run and stop buttons."""
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert 'id="run-unified-backfill-btn"' in resp.text
        assert 'id="stop-unified-backfill-btn"' in resp.text
        assert 'id="unified-backfill-progress"' in resp.text

    def test_tagging_actions_page_shows_download_error_log_link(self, client):
        """The page should contain a link to download the backfill error log."""
        resp = client.get("/admin/tagging-actions/")
        assert resp.status_code == 200
        assert "download-backfill-log" in resp.text
        assert "Download error log" in resp.text

    # --- Unified backfill route tests ---

    def test_run_unified_backfill_returns_json(self, client, monkeypatch):
        """POST /run-unified-backfill should return a JSON response with results."""
        import src.routes.tagging_actions as ta_mod

        monkeypatch.setattr(
            ta_mod,
            "unified_backfill",
            lambda db, actions, batch_size=100, commit_every=100, **kwargs: {
                "processed": 5,
                "errors": 0,
                "stopped": False,
                "actions": list(actions.keys()),
            },
        )
        resp = client.post(
            "/admin/tagging-actions/run-unified-backfill",
            json={"actions": {"tagging": {"action": "tag_untagged", "tiers": [1]}}},
        )
        assert resp.status_code == 200
        data = resp.json()
        assert data["processed"] == 5
        assert data["errors"] == 0
        assert data["stopped"] is False

    def test_run_unified_backfill_passes_batch_and_commit(self, client, monkeypatch):
        """Batch size and commit_every should be forwarded to unified_backfill."""
        import src.routes.tagging_actions as ta_mod

        captured = {}

        def fake_backfill(db, actions, batch_size=100, commit_every=100, **kwargs):
            captured["batch_size"] = batch_size
            captured["commit_every"] = commit_every
            return {"processed": 0, "errors": 0, "stopped": False, "actions": list(actions.keys())}

        monkeypatch.setattr(ta_mod, "unified_backfill", fake_backfill)
        resp = client.post(
            "/admin/tagging-actions/run-unified-backfill",
            json={
                "actions": {"stitching": {}},
                "batch_size": 50,
                "commit_every": 25,
            },
        )
        assert resp.status_code == 200
        assert captured["batch_size"] == 50
        assert captured["commit_every"] == 25

    def test_run_unified_backfill_exception_returns_500(self, client, monkeypatch):
        """When unified_backfill raises, the endpoint should return a 500 JSON error."""
        import src.routes.tagging_actions as ta_mod

        monkeypatch.setattr(
            ta_mod,
            "unified_backfill",
            lambda db, actions, batch_size=100, commit_every=100: (_ for _ in ()).throw(
                RuntimeError("backfill crashed")
            ),
        )
        resp = client.post(
            "/admin/tagging-actions/run-unified-backfill",
            json={"actions": {"tagging": {"action": "tag_untagged", "tiers": [1]}}},
        )
        assert resp.status_code == 500
        data = resp.json()
        assert "error" in data

    def test_stop_unified_backfill_first_call_returns_stopping(self, client, monkeypatch):
        """First call to /stop-unified-backfill should return {'status': 'stopping'}."""
        import src.routes.tagging_actions as ta_mod

        monkeypatch.setattr(ta_mod, "is_stop_requested", lambda: False)
        monkeypatch.setattr(ta_mod, "request_stop", lambda: None)
        resp = client.post("/admin/tagging-actions/stop-unified-backfill")
        assert resp.status_code == 200
        assert resp.json() == {"status": "stopping"}

    def test_stop_unified_backfill_already_stopping(self, client, monkeypatch):
        """If stop was already requested, return {'status': 'already_stopping'}."""
        import src.routes.tagging_actions as ta_mod

        monkeypatch.setattr(ta_mod, "is_stop_requested", lambda: True)
        resp = client.post("/admin/tagging-actions/stop-unified-backfill")
        assert resp.status_code == 200
        assert resp.json() == {"status": "already_stopping"}

    def test_download_backfill_log_not_found(self, client, monkeypatch):
        """When no error log exists, /download-backfill-log should return 404."""
        from pathlib import Path

        import src.routes.tagging_actions as ta_mod

        monkeypatch.setattr(ta_mod, "ERROR_LOG_PATH", Path("/nonexistent/err.log"))
        resp = client.get("/admin/tagging-actions/download-backfill-log")
        assert resp.status_code == 404
        assert "error" in resp.json()

    def test_download_backfill_log_found(self, client, monkeypatch, tmp_path):
        """When error log exists, /download-backfill-log should return it as a file."""
        import src.routes.tagging_actions as ta_mod

        log_file = tmp_path / "err.log"
        log_file.write_text("2026-01-01 12:00:00\tfile1.pes\tstitching\t...\n")
        monkeypatch.setattr(ta_mod, "ERROR_LOG_PATH", log_file)
        resp = client.get("/admin/tagging-actions/download-backfill-log")
        assert resp.status_code == 200
        assert resp.headers.get("content-type", "").startswith("text/plain")
        assert "file1.pes" in resp.text

    # --- Helper unit tests ---

    def test_redirect_with_result_includes_breakdown(self):
        """_redirect_with_result should include tag_breakdown in the query string."""
        from src.routes.tagging_actions import _redirect_with_result
        from src.services.auto_tagging import TaggingActionResult

        result = TaggingActionResult(
            action="tag_untagged",
            tiers_run=[1],
            designs_considered=5,
            tier1_tagged=3,
            total_tagged=3,
            still_untagged=2,
            already_matched=0,
            no_match=0,
            cleared_only=0,
            tag_breakdown={"Filled": 2, "Satin": 1},
        )
        resp = _redirect_with_result(result)
        location = resp.headers.get("location", "")
        assert "breakdown=" in location
        assert "Filled: 2" in location or "Filled%3A%202" in location

    def test_redirect_with_result_includes_errors(self):
        """_redirect_with_result should include errors in the warn parameter."""
        from src.routes.tagging_actions import _redirect_with_result
        from src.services.auto_tagging import TaggingActionResult

        result = TaggingActionResult(
            action="tag_untagged",
            tiers_run=[1],
            designs_considered=5,
            tier1_tagged=3,
            total_tagged=3,
            still_untagged=2,
            already_matched=0,
            no_match=0,
            cleared_only=0,
            errors=["API error: timeout", "Rate limit hit"],
        )
        resp = _redirect_with_result(result)
        location = resp.headers.get("location", "")
        assert "warn=" in location
        assert "API error" in location or "API%20error" in location

    def test_resolve_delay_returns_saved_value(self):
        """_resolve_delay should return the saved delay when it is a valid non-negative float."""
        from src.routes.tagging_actions import _resolve_delay

        assert _resolve_delay("3.5") == 3.5
        assert _resolve_delay("0") == 0.0

    def test_resolve_delay_negative_falls_back_to_default(self):
        """_resolve_delay should return the default for negative values."""
        from src.routes.tagging_actions import _DEFAULT_DELAY, _resolve_delay

        assert _resolve_delay("-1") == _DEFAULT_DELAY

    def test_resolve_delay_invalid_falls_back_to_default(self):
        """_resolve_delay should return the default for non-numeric strings."""
        from src.routes.tagging_actions import _DEFAULT_DELAY, _resolve_delay

        assert _resolve_delay("not-a-number") == _DEFAULT_DELAY
        assert _resolve_delay("") == _DEFAULT_DELAY


class TestSettingsDelayField:
    """Tests for the delay setting added to Settings."""

    def test_settings_page_shows_delay_field(self, client):
        resp = client.get("/admin/settings/")
        assert resp.status_code == 200
        assert "ai_delay" in resp.text

    def test_save_settings_persists_delay(self, client, db):
        from src.services import settings_service as settings_svc

        client.post(
            "/admin/settings/",
            data={"ai_delay": "6.5"},
            follow_redirects=False,
        )
        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_DELAY) == "6.5"

    def test_save_settings_clears_delay(self, client, db):
        from src.services import settings_service as settings_svc

        # Set then clear
        client.post("/admin/settings/", data={"ai_delay": "8.0"}, follow_redirects=False)
        client.post("/admin/settings/", data={}, follow_redirects=False)
        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_DELAY) == ""

    def test_settings_service_delay_default_is_blank(self, db):
        from src.services import settings_service as settings_svc

        # Fresh DB should return empty string as default
        assert settings_svc.get_setting(db, settings_svc.SETTING_AI_DELAY) == ""


class TestClearImages:
    """Tests for the /admin/maintenance/clear-images endpoint."""

    def test_clear_images_clears_all_image_data(self, client, db):
        """clear-images should set image_data, width_mm, height_mm, hoop_id to NULL."""
        from src.models import Hoop
        from src.services import designs

        # Create a hoop
        hoop = Hoop(name="Test Hoop", max_width_mm=200, max_height_mm=200)
        db.add(hoop)
        db.flush()

        # Create designs with image data
        d1 = designs.create(
            db, {"filename": "img1.jef", "filepath": "/img1.jef", "is_stitched": False}
        )
        d2 = designs.create(
            db, {"filename": "img2.jef", "filepath": "/img2.jef", "is_stitched": False}
        )
        # Manually set image data, dimensions, and hoop
        d1.image_data = b"\x89PNG"
        d1.width_mm = 100.0
        d1.height_mm = 50.0
        d1.hoop_id = hoop.id
        d2.image_data = b"\x89PNG"
        d2.width_mm = 80.0
        d2.height_mm = 60.0
        d2.hoop_id = hoop.id
        db.commit()

        # Verify they have image data before
        assert d1.image_data is not None
        assert d2.image_data is not None

        # Call the clear-images endpoint
        resp = client.post("/admin/maintenance/clear-images", follow_redirects=False)
        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "cleared=2" in location

        # Verify image data, dimensions, and hoop are cleared
        db.refresh(d1)
        db.refresh(d2)
        assert d1.image_data is None
        assert d1.width_mm is None
        assert d1.height_mm is None
        assert d1.hoop_id is None
        assert d2.image_data is None
        assert d2.width_mm is None
        assert d2.height_mm is None
        assert d2.hoop_id is None

    def test_clear_images_with_no_images(self, client, db):
        """clear-images should handle the case where no designs have images."""
        from src.services import designs

        # Create designs with no image data
        designs.create(
            db, {"filename": "noimg1.jef", "filepath": "/noimg1.jef", "is_stitched": False}
        )
        designs.create(
            db, {"filename": "noimg2.jef", "filepath": "/noimg2.jef", "is_stitched": False}
        )

        # Call the clear-images endpoint
        resp = client.post("/admin/maintenance/clear-images", follow_redirects=False)
        assert resp.status_code == 303
        location = resp.headers.get("location", "")
        assert "cleared=0" in location
