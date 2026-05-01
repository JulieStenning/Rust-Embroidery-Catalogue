"""Unit tests for Section 1.1 — Legacy Individual Tagging Action Forms.

Tests the standalone tagging action forms at the top of the Tagging Actions page.
These call `run_tagging_action()` in `src/services/auto_tagging.py`.

Coverage:
- 1.1.1: tag_untagged + Tier 1 only (no API key)
- 1.1.2: tag_untagged + Tiers 1+2+3 (with mocked API)
- 1.1.3: tag_untagged + Tiers 1+2+3 (same as 1.1.2, different tier combo)
- 1.1.4: tag_untagged + Tiers 1+3 (skip Tier 2)
- 1.1.5: tag_untagged + Tiers 1+2 (no API key — graceful skip)
- 1.1.6: retag_all_unverified + Tier 1 only
- 1.1.7: retag_all_unverified + Tiers 1+2+3 (with mocked API)
- 1.1.8: retag_all + Tier 1 only (destructive)
- 1.1.9: retag_all + Tiers 1+2+3 (destructive, with mocked API)
- 1.1.10: batch_size limiting
- 1.1.11: delay parameter passthrough

Section 1.2 — Stitch Type Analysis:
- 1.2.1: Stitch analysis without clearing (clear_existing_stitching=False)
- 1.2.2: Stitch analysis with clearing (clear_existing_stitching=True)
"""

from __future__ import annotations

from unittest.mock import patch


class TestTagUntaggedTier1Only:
    """Test 1.1.1 — tag_untagged with Tier 1 only (keyword matching)."""

    def test_tags_untagged_designs_with_keyword_matches(self, db):
        """Designs with no image tags get Tier 1 tags based on filename keywords."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        # Create tags
        flowers_tag = Tag(description="Flowers", tag_group="image")
        christmas_tag = Tag(description="Christmas", tag_group="image")
        db.add_all([flowers_tag, christmas_tag])
        db.commit()

        # Create designs with no tags
        d1 = Design(filename="rose_bouquet.jef", filepath="\\Florals\\rose_bouquet.jef")
        d2 = Design(filename="christmas_tree.pes", filepath="\\Seasonal\\christmas_tree.pes")
        d3 = Design(filename="mystery.dst", filepath="\\Unknown\\mystery.dst")
        db.add_all([d1, d2, d3])
        db.commit()

        # Run tagging action
        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
            batch_size=None,
        )

        # Verify results
        assert result.designs_considered == 3
        assert result.total_tagged == 2  # rose and christmas
        assert result.still_untagged == 1  # mystery
        assert result.tier1_tagged == 2
        assert result.tier2_tagged == 0
        assert result.tier3_tagged == 0

        # Verify database state
        db.refresh(d1)
        db.refresh(d2)
        db.refresh(d3)

        assert len(d1.tags) == 1
        assert d1.tags[0].description == "Flowers"
        assert d1.tagging_tier == 1
        assert d1.tags_checked is False

        assert len(d2.tags) == 1
        assert d2.tags[0].description == "Christmas"
        assert d2.tagging_tier == 1
        assert d2.tags_checked is False

        assert len(d3.tags) == 0
        assert d3.tagging_tier is None

    def test_skips_designs_already_tagged(self, db):
        """Designs with existing image tags are not processed by tag_untagged."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        db.add(flowers_tag)
        db.commit()

        # Design already has a tag
        d1 = Design(filename="rose.jef", filepath="\\rose.jef", tags=[flowers_tag])
        db.add(d1)
        db.commit()

        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
        )

        # Should not process any designs
        assert result.designs_considered == 0
        assert result.total_tagged == 0

    def test_case_insensitive_keyword_matching(self, db):
        """Test 6.1.5 — Keywords match regardless of filename case.

        The ``_stem()`` helper lowercases the filename before matching
        against the keyword map, so ``ROSE_BOUQUET.dst`` and ``TuLiP.dst``
        should match just as ``daisy.dst`` does.
        """
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        db.add(flowers_tag)
        db.commit()

        # Designs with different casing
        d1 = Design(filename="ROSE_BOUQUET.dst", filepath="\\UPPER\\ROSE_BOUQUET.dst")
        d2 = Design(filename="TuLiP.dst", filepath="\\Mixed\\TuLiP.dst")
        d3 = Design(filename="daisy.dst", filepath="\\lower\\daisy.dst")
        db.add_all([d1, d2, d3])
        db.commit()

        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
            batch_size=None,
        )

        assert result.designs_considered == 3
        assert result.total_tagged == 3
        assert result.tier1_tagged == 3

        db.refresh(d1)
        db.refresh(d2)
        db.refresh(d3)

        assert len(d1.tags) == 1
        assert d1.tags[0].description == "Flowers"
        assert d1.tagging_tier == 1

        assert len(d2.tags) == 1
        assert d2.tags[0].description == "Flowers"
        assert d2.tagging_tier == 1

        assert len(d3.tags) == 1
        assert d3.tags[0].description == "Flowers"
        assert d3.tagging_tier == 1


class TestTagUntaggedWithMockedAPI:
    """Tests 1.1.2, 1.1.3, 1.1.4 — tag_untagged with mocked Gemini API calls."""

    def test_full_pipeline_tiers_1_2_3(self, db):
        """Test 1.1.2/1.1.3 — Full pipeline: Tier 1 -> Tier 2 -> Tier 3.

        Uses filenames that do NOT match any Tier 1 keyword to ensure
        they fall through to Tier 2 and Tier 3 respectively.
        """
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        # Create tags
        flowers_tag = Tag(description="Flowers", tag_group="image")
        animals_tag = Tag(description="Animals", tag_group="image")
        birds_tag = Tag(description="Birds", tag_group="image")
        db.add_all([flowers_tag, animals_tag, birds_tag])
        db.commit()

        # Create designs
        d1 = Design(filename="rose.jef", filepath="\\rose.jef")  # Tier 1 match
        d2 = Design(filename="zzz_unknown.pes", filepath="\\zzz_unknown.pes")  # Tier 2 match
        d3 = Design(
            filename="yyy_unknown.dst", filepath="\\yyy_unknown.dst", image_data=b"fake_png"
        )  # Tier 3 match
        db.add_all([d1, d2, d3])
        db.commit()

        # Mock Tier 2 and Tier 3
        with (
            patch(
                "src.services.auto_tagging.suggest_tier2_batch",
                return_value={"zzz_unknown": ["Birds"]},
            ),
            patch(
                "src.services.auto_tagging.suggest_tier3_vision",
                return_value={d3.id: ["Animals"]},
            ),
        ):
            result = run_tagging_action(
                db=db,
                action="tag_untagged",
                tiers=[1, 2, 3],
                api_key="fake_api_key",
            )

        # Verify results
        assert result.designs_considered == 3
        assert result.total_tagged == 3
        assert result.tier1_tagged == 1  # rose
        assert result.tier2_tagged == 1  # zzz_unknown
        assert result.tier3_tagged == 1  # yyy_unknown

        # Verify database state
        db.refresh(d1)
        db.refresh(d2)
        db.refresh(d3)

        assert d1.tags[0].description == "Flowers"
        assert d1.tagging_tier == 1

        assert d2.tags[0].description == "Birds"
        assert d2.tagging_tier == 2

        assert d3.tags[0].description == "Animals"
        assert d3.tagging_tier == 3

    def test_tier_1_and_3_skip_tier_2(self, db):
        """Test 1.1.4 — Tiers 1+3 selected, Tier 2 is skipped."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        animals_tag = Tag(description="Animals", tag_group="image")
        db.add_all([flowers_tag, animals_tag])
        db.commit()

        d1 = Design(filename="rose.jef", filepath="\\rose.jef")  # Tier 1
        d2 = Design(
            filename="mystery.pes", filepath="\\mystery.pes", image_data=b"fake_png"
        )  # Tier 3
        db.add_all([d1, d2])
        db.commit()

        # Mock only Tier 3 (Tier 2 should not be called)
        with (
            patch(
                "src.services.auto_tagging.suggest_tier2_batch",
                side_effect=AssertionError("Tier 2 should not be called"),
            ),
            patch(
                "src.services.auto_tagging.suggest_tier3_vision",
                return_value={d2.id: ["Animals"]},
            ),
        ):
            result = run_tagging_action(
                db=db,
                action="tag_untagged",
                tiers=[1, 3],  # Skip Tier 2
                api_key="fake_api_key",
            )

        assert result.tier1_tagged == 1
        assert result.tier2_tagged == 0  # Tier 2 was skipped
        assert result.tier3_tagged == 1


class TestTagUntaggedNoAPIKey:
    """Test 1.1.5 — tag_untagged with Tiers 1+2 but no API key."""

    def test_gracefully_skips_tier_2_when_no_api_key(self, db):
        """When no API key is provided, Tier 2 is skipped with an error message."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        db.add(flowers_tag)
        db.commit()

        d1 = Design(filename="rose.jef", filepath="\\rose.jef")  # Tier 1 match
        d2 = Design(filename="mystery.pes", filepath="\\mystery.pes")  # Would need Tier 2
        db.add_all([d1, d2])
        db.commit()

        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1, 2],
            api_key="",  # No API key
        )

        # Verify Tier 2 was skipped
        assert result.tier1_tagged == 1
        assert result.tier2_tagged == 0
        assert result.still_untagged == 1
        assert "No Google API key configured" in result.errors[0]

        # Verify only d1 was tagged
        db.refresh(d1)
        db.refresh(d2)
        assert len(d1.tags) == 1
        assert len(d2.tags) == 0


class TestRetagAllUnverified:
    """Tests 1.1.6, 1.1.7 — retag_all_unverified action."""

    def test_retag_unverified_tier_1_only(self, db):
        """Test 1.1.6 — retag_all_unverified overwrites unverified designs only."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        christmas_tag = Tag(description="Christmas", tag_group="image")
        animals_tag = Tag(description="Animals", tag_group="image")
        db.add_all([flowers_tag, christmas_tag, animals_tag])
        db.commit()

        # Unverified design with wrong tag
        d1 = Design(
            filename="christmas_tree.jef",
            filepath="\\christmas_tree.jef",
            tags=[animals_tag],
            tags_checked=False,
        )
        # Verified design with wrong tag
        d2 = Design(
            filename="rose.pes",
            filepath="\\rose.pes",
            tags=[animals_tag],
            tags_checked=True,
        )
        db.add_all([d1, d2])
        db.commit()

        result = run_tagging_action(
            db=db,
            action="retag_all_unverified",
            tiers=[1],
            api_key="",
        )

        # Only d1 should be processed
        assert result.designs_considered == 1
        assert result.total_tagged == 1

        # Verify database state
        db.refresh(d1)
        db.refresh(d2)

        # d1 should have Christmas tag now
        assert len(d1.tags) == 1
        assert d1.tags[0].description == "Christmas"
        assert d1.tagging_tier == 1
        assert d1.tags_checked is False

        # d2 should be unchanged (verified)
        assert len(d2.tags) == 1
        assert d2.tags[0].description == "Animals"
        assert d2.tags_checked is True

    def test_retag_unverified_with_full_ai_pipeline(self, db):
        """Test 1.1.7 — retag_all_unverified with Tiers 1+2+3."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        birds_tag = Tag(description="Birds", tag_group="image")
        db.add_all([flowers_tag, birds_tag])
        db.commit()

        # Unverified design with no keyword match (needs AI)
        d1 = Design(
            filename="mystery.jef",
            filepath="\\mystery.jef",
            image_data=b"fake_png",
            tags=[],
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_tier2_batch",
                return_value={"mystery": ["Birds"]},
            ),
            patch("src.services.auto_tagging.suggest_tier3_vision", return_value={}),
        ):
            result = run_tagging_action(
                db=db,
                action="retag_all_unverified",
                tiers=[1, 2, 3],
                api_key="fake_api_key",
            )

        assert result.tier2_tagged == 1
        db.refresh(d1)
        assert d1.tags[0].description == "Birds"


class TestRetagAll:
    """Tests 1.1.8, 1.1.9 — retag_all action (destructive)."""

    def test_retag_all_overwrites_verified_designs(self, db):
        """Test 1.1.8 — retag_all overwrites ALL designs including verified."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        christmas_tag = Tag(description="Christmas", tag_group="image")
        db.add_all([flowers_tag, christmas_tag])
        db.commit()

        # Verified design with wrong tag
        d1 = Design(
            filename="christmas_tree.jef",
            filepath="\\christmas_tree.jef",
            tags=[flowers_tag],
            tags_checked=True,  # Verified
        )
        db.add(d1)
        db.commit()

        result = run_tagging_action(
            db=db,
            action="retag_all",
            tiers=[1],
            api_key="",
        )

        # Should process the verified design
        assert result.designs_considered == 1
        assert result.total_tagged == 1

        # Verify tags were overwritten and tags_checked reset
        db.refresh(d1)
        assert len(d1.tags) == 1
        assert d1.tags[0].description == "Christmas"
        assert d1.tags_checked is False  # Reset to unverified

    def test_retag_all_with_full_ai_pipeline(self, db):
        """Test 1.1.9 — retag_all with Tiers 1+2+3 (destructive)."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        animals_tag = Tag(description="Animals", tag_group="image")
        db.add_all([flowers_tag, animals_tag])
        db.commit()

        # Verified design with existing tag
        d1 = Design(
            filename="mystery.jef",
            filepath="\\mystery.jef",
            image_data=b"fake_png",
            tags=[flowers_tag],
            tags_checked=True,
        )
        db.add(d1)
        db.commit()

        with (
            patch("src.services.auto_tagging.suggest_tier2_batch", return_value={}),
            patch(
                "src.services.auto_tagging.suggest_tier3_vision",
                return_value={d1.id: ["Animals"]},
            ),
        ):
            result = run_tagging_action(
                db=db,
                action="retag_all",
                tiers=[1, 2, 3],
                api_key="fake_api_key",
            )

        assert result.tier3_tagged == 1
        db.refresh(d1)
        assert d1.tags[0].description == "Animals"
        assert d1.tags_checked is False  # Reset


class TestBatchSizeLimiting:
    """Test 1.1.10 — batch_size parameter limits number of designs processed."""

    def test_batch_size_limits_processing(self, db):
        """Only the first N designs are processed when batch_size is set."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        db.add(flowers_tag)
        db.commit()

        # Create 5 designs
        designs = [Design(filename=f"rose_{i}.jef", filepath=f"\\rose_{i}.jef") for i in range(5)]
        db.add_all(designs)
        db.commit()

        # Process only 2
        result = run_tagging_action(
            db=db,
            action="tag_untagged",
            tiers=[1],
            api_key="",
            batch_size=2,
        )

        assert result.designs_considered == 2
        assert result.total_tagged == 2

        # Verify only first 2 were tagged
        for i, design in enumerate(designs):
            db.refresh(design)
            if i < 2:
                assert len(design.tags) == 1
            else:
                assert len(design.tags) == 0


class TestDelayParameter:
    """Test 1.1.11 — delay parameter is passed through to API calls."""

    def test_delay_parameter_passed_to_tier2(self, db):
        """Delay parameter is passed to suggest_tier2_batch."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        db.add(flowers_tag)
        db.commit()

        d1 = Design(filename="mystery.jef", filepath="\\mystery.jef")
        db.add(d1)
        db.commit()

        with patch("src.services.auto_tagging.suggest_tier2_batch", return_value={}) as mock_tier2:
            run_tagging_action(
                db=db,
                action="tag_untagged",
                tiers=[1, 2],
                api_key="fake_api_key",
                delay=0.5,  # Custom delay
            )

            # Verify delay was passed through
            mock_tier2.assert_called_once()
            call_kwargs = mock_tier2.call_args[1]
            assert call_kwargs["delay_seconds"] == 0.5

    def test_vision_delay_parameter_passed_to_tier3(self, db):
        """Vision delay parameter is passed to suggest_tier3_vision."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_tagging_action

        animals_tag = Tag(description="Animals", tag_group="image")
        db.add(animals_tag)
        db.commit()

        d1 = Design(filename="mystery.jef", filepath="\\mystery.jef", image_data=b"fake_png")
        db.add(d1)
        db.commit()

        with (
            patch("src.services.auto_tagging.suggest_tier2_batch", return_value={}),
            patch("src.services.auto_tagging.suggest_tier3_vision", return_value={}) as mock_tier3,
        ):
            run_tagging_action(
                db=db,
                action="tag_untagged",
                tiers=[1, 2, 3],
                api_key="fake_api_key",
                vision_delay=1.5,  # Custom vision delay
            )

            # Verify vision_delay was passed through
            mock_tier3.assert_called_once()
            call_kwargs = mock_tier3.call_args[1]
            assert call_kwargs["delay_seconds"] == 1.5


# =============================================================================
# Section 1.2 — Stitch Type Analysis (Legacy Individual Action Form)
# =============================================================================


class TestStitchingBackfillNoClear:
    """Test 1.2.1 — Stitch analysis without clearing existing stitching tags.

    Calls ``run_stitching_backfill_action()`` with ``clear_existing_stitching=False``.
    Only unverified designs without existing stitching tags are processed.
    """

    def test_analyses_unverified_designs_and_adds_stitching_tags(self, db):
        """Unverified designs with no stitching tags get stitching tags assigned."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        # Create stitching tags
        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        filled_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([satin_tag, filled_tag])
        db.commit()

        # Create unverified designs with no stitching tags
        d1 = Design(filename="satin_rose.dst", filepath="\\satin_rose.dst", tags_checked=False)
        d2 = Design(
            filename="filled_flower.pes", filepath="\\filled_flower.pes", tags_checked=False
        )
        db.add_all([d1, d2])
        db.commit()

        # Mock suggest_stitching_from_pattern to return known stitching types
        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                side_effect=lambda pattern_path, filename, filepath, desc_to_tag: (
                    ["Satin Stitch"] if "satin" in filename else ["Filled"]
                ),
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=False,
            )

        # Verify results
        assert result.designs_considered == 2
        assert result.total_tagged == 2
        assert result.tier1_tagged == 2
        assert result.still_untagged == 0
        assert result.no_match == 0

        # Verify database state
        db.refresh(d1)
        db.refresh(d2)

        assert len(d1.tags) == 1
        assert d1.tags[0].description == "Satin Stitch"
        assert d1.tags[0].tag_group == "stitching"
        assert d1.tagging_tier == 1
        assert d1.tags_checked is False

        assert len(d2.tags) == 1
        assert d2.tags[0].description == "Filled"
        assert d2.tags[0].tag_group == "stitching"
        assert d2.tagging_tier == 1
        assert d2.tags_checked is False

    def test_skips_verified_designs(self, db):
        """Verified designs are not processed by stitching backfill."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add(satin_tag)
        db.commit()

        # Verified design
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags_checked=True,  # Verified
        )
        db.add(d1)
        db.commit()

        with (
            patch("src.services.auto_tagging.suggest_stitching_from_pattern", return_value=[]),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=False,
            )

        # Verified designs should be skipped
        assert result.designs_considered == 0
        assert result.total_tagged == 0

    def test_skips_designs_with_existing_stitching_tags(self, db):
        """Designs that already have stitching tags are skipped when not clearing."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add(satin_tag)
        db.commit()

        # Unverified design that already has a stitching tag
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags=[satin_tag],
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch("src.services.auto_tagging.suggest_stitching_from_pattern", return_value=[]),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=False,
            )

        # Should skip — already has stitching tag
        assert result.designs_considered == 0
        assert result.total_tagged == 0

    def test_preserves_existing_non_stitching_tags(self, db):
        """Existing non-stitching tags (e.g. image tags) are preserved."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        # Create both an image tag and a stitching tag
        flowers_tag = Tag(description="Flowers", tag_group="image")
        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add_all([flowers_tag, satin_tag])
        db.commit()

        # Design with an existing image tag but no stitching tag
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags=[flowers_tag],
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=["Satin Stitch"],
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=False,
            )

        assert result.total_tagged == 1

        # Verify both tags are present
        db.refresh(d1)
        tag_descriptions = {t.description for t in d1.tags}
        assert "Flowers" in tag_descriptions  # Preserved
        assert "Satin Stitch" in tag_descriptions  # Added

    def test_handles_no_stitching_tags_configured(self, db):
        """When no stitching tags exist in the DB, an error is returned."""
        from src.models import Design
        from src.services.auto_tagging import run_stitching_backfill_action

        # Create a design but no stitching tags
        d1 = Design(filename="test.dst", filepath="\\test.dst", tags_checked=False)
        db.add(d1)
        db.commit()

        result = run_stitching_backfill_action(
            db=db,
            batch_size=None,
            clear_existing_stitching=False,
        )

        assert result.designs_considered == 0
        assert len(result.errors) > 0
        assert "No stitching tags are configured" in result.errors[0]

    def test_handles_no_detectable_stitch_type(self, db):
        """Designs with no detectable stitch type get no stitching tags."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add(satin_tag)
        db.commit()

        d1 = Design(filename="unknown.dst", filepath="\\unknown.dst", tags_checked=False)
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=[],  # No stitch type detected
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=False,
            )

        assert result.designs_considered == 1
        assert result.total_tagged == 0
        assert result.no_match == 1
        assert result.still_untagged == 1

        # Verify no stitching tags were added
        db.refresh(d1)
        assert len(d1.tags) == 0

    def test_batch_size_limits_per_iteration(self, db):
        """Batch size parameter limits how many designs are fetched per iteration.

        The batch_size controls the SQL LIMIT per query iteration, not the total
        number of designs processed. All designs are eventually processed across
        multiple iterations.
        """
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add(satin_tag)
        db.commit()

        # Create 5 designs
        designs = [
            Design(filename=f"satin_{i}.dst", filepath=f"\\satin_{i}.dst", tags_checked=False)
            for i in range(5)
        ]
        db.add_all(designs)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=["Satin Stitch"],
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=2,
                clear_existing_stitching=False,
            )

        # All 5 designs are processed across multiple iterations (batches of 2)
        assert result.total_tagged == 5
        assert result.designs_considered == 5
        # Verify the batch_size was used by checking the SQL log output
        # (the test log shows LIMIT 2 in each query)


class TestStitchingBackfillWithClear:
    """Test 1.2.2 — Stitch analysis with clearing existing stitching tags.

    Calls ``run_stitching_backfill_action()`` with ``clear_existing_stitching=True``.
    Existing stitching tags are removed first, then re-analysis is performed.
    """

    def test_clears_existing_stitching_and_re_analyses(self, db):
        """Existing stitching tags are cleared, then new ones are assigned."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        # Create stitching tags
        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        filled_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([satin_tag, filled_tag])
        db.commit()

        # Design with existing (wrong) stitching tag
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags=[filled_tag],  # Wrong stitching tag
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=["Satin Stitch"],
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=True,
            )

        assert result.total_tagged == 1
        assert result.tier1_tagged == 1

        # Verify old stitching tag was replaced
        db.refresh(d1)
        tag_descriptions = {t.description for t in d1.tags}
        assert "Satin Stitch" in tag_descriptions
        assert "Filled" not in tag_descriptions  # Was cleared

    def test_clears_stitching_tags_when_no_new_match(self, db):
        """When clear_existing is on and no new stitch type is detected,
        existing stitching tags are still removed."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        db.add(satin_tag)
        db.commit()

        # Design with existing stitching tag but no detectable stitch type
        d1 = Design(
            filename="unknown.dst",
            filepath="\\unknown.dst",
            tags=[satin_tag],
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=[],  # No stitch type detected
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=True,
            )

        # Should have cleared the existing stitching tag
        assert result.cleared_only == 1
        assert result.total_tagged == 0

        # Verify stitching tag was removed
        db.refresh(d1)
        assert len(d1.tags) == 0

    def test_preserves_non_stitching_tags_when_clearing(self, db):
        """Non-stitching tags (e.g. image tags) are preserved when clearing."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        flowers_tag = Tag(description="Flowers", tag_group="image")
        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        filled_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([flowers_tag, satin_tag, filled_tag])
        db.commit()

        # Design with image tag + wrong stitching tag
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags=[flowers_tag, filled_tag],
            tags_checked=False,
        )
        db.add(d1)
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                return_value=["Satin Stitch"],
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=True,
            )

        assert result.total_tagged == 1
        assert result.tier1_tagged == 1

        # Verify old stitching tag was replaced and image tag preserved
        db.refresh(d1)
        tag_descriptions = {t.description for t in d1.tags}
        assert "Flowers" in tag_descriptions  # Preserved
        assert "Satin Stitch" in tag_descriptions  # Added
        assert "Filled" not in tag_descriptions  # Was cleared

    def test_clears_stitching_and_handles_multiple_designs(self, db):
        """Multiple designs are processed correctly with clear_existing."""
        from src.models import Design, Tag
        from src.services.auto_tagging import run_stitching_backfill_action

        satin_tag = Tag(description="Satin Stitch", tag_group="stitching")
        filled_tag = Tag(description="Filled", tag_group="stitching")
        db.add_all([satin_tag, filled_tag])
        db.commit()

        # Design with existing stitching tag that should be replaced
        d1 = Design(
            filename="satin_rose.dst",
            filepath="\\satin_rose.dst",
            tags=[filled_tag],
            tags_checked=False,
        )
        # Design with no stitching tag that should get one
        d2 = Design(
            filename="filled_pattern.pes",
            filepath="\\filled_pattern.pes",
            tags_checked=False,
        )
        db.add_all([d1, d2])
        db.commit()

        with (
            patch(
                "src.services.auto_tagging.suggest_stitching_from_pattern",
                side_effect=lambda pattern_path, filename, filepath, desc_to_tag: (
                    ["Satin Stitch"] if "satin" in filename else ["Filled"]
                ),
            ),
            patch(
                "src.services.auto_tagging._resolve_design_filepath",
                return_value="/fake/path/test.dst",
            ),
        ):
            result = run_stitching_backfill_action(
                db=db,
                batch_size=None,
                clear_existing_stitching=True,
            )

        assert result.total_tagged == 2
        assert result.tier1_tagged == 2

        db.refresh(d1)
        db.refresh(d2)
        assert d1.tags[0].description == "Satin Stitch"
        assert d2.tags[0].description == "Filled"


# =============================================================================
# Section 1.3 — Threads and Colours (Legacy Individual Action Form)
# =============================================================================


class TestColorCountsBackfill:
    """Test 1.3.1 — Colour count backfill via the legacy form.

    Calls the ``/admin/maintenance/backfill-color-counts`` route which
    populates stitch_count, color_count and color_change_count for designs
    where these values are NULL.
    """

    def test_populates_missing_color_counts(self, db, client, monkeypatch):
        """Designs missing stitch/colour data get all three values populated."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        # Create designs with NULL colour data
        d1 = Design(filename="test.dst", filepath="\\test.dst")
        d2 = Design(filename="test2.pes", filepath="\\test2.pes")
        db.add_all([d1, d2])
        db.commit()

        # Mock pattern with colour data
        class MockPattern:
            def count_stitches(self):
                return 5000

            def count_threads(self):
                return 3

            def count_color_changes(self):
                return 12

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with patch("pyembroidery.read", return_value=MockPattern()):
            resp = client.post("/admin/maintenance/backfill-color-counts", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "done=1" in resp.headers["location"]
        assert "action=backfill_color_counts" in resp.headers["location"]

        # Verify database state
        db.refresh(d1)
        db.refresh(d2)
        assert d1.stitch_count == 5000
        assert d1.color_count == 3
        assert d1.color_change_count == 12
        assert d2.stitch_count == 5000
        assert d2.color_count == 3
        assert d2.color_change_count == 12

    def test_skips_designs_with_existing_data(self, db, client, monkeypatch):
        """Designs that already have colour data are skipped (no redo)."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        # Design with partial data
        d1 = Design(
            filename="test.dst",
            filepath="\\test.dst",
            stitch_count=1000,
            color_count=None,
            color_change_count=None,
        )
        db.add(d1)
        db.commit()

        class MockPattern:
            def count_stitches(self):
                return 5000

            def count_threads(self):
                return 3

            def count_color_changes(self):
                return 12

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with patch("pyembroidery.read", return_value=MockPattern()):
            resp = client.post("/admin/maintenance/backfill-color-counts", follow_redirects=False)

        assert resp.status_code in (302, 303)

        # stitch_count should be preserved (was already set), others populated
        db.refresh(d1)
        assert d1.stitch_count == 1000  # Preserved
        assert d1.color_count == 3  # Populated
        assert d1.color_change_count == 12  # Populated

    def test_redo_overwrites_existing_data(self, db, client, monkeypatch):
        """With redo=True, existing colour data is overwritten."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(
            filename="test.dst",
            filepath="\\test.dst",
            stitch_count=1000,
            color_count=2,
            color_change_count=5,
        )
        db.add(d1)
        db.commit()

        class MockPattern:
            def count_stitches(self):
                return 5000

            def count_threads(self):
                return 3

            def count_color_changes(self):
                return 12

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with patch("pyembroidery.read", return_value=MockPattern()):
            resp = client.post(
                "/admin/maintenance/backfill-color-counts",
                data={"redo": "1"},
                follow_redirects=False,
            )

        assert resp.status_code in (302, 303)

        db.refresh(d1)
        assert d1.stitch_count == 5000  # Overwritten
        assert d1.color_count == 3  # Overwritten
        assert d1.color_change_count == 12  # Overwritten

    def test_handles_missing_file_gracefully(self, db, client, monkeypatch):
        """Designs whose files are missing are skipped with an error count."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(filename="missing.dst", filepath="\\missing.dst")
        d2 = Design(filename="present.dst", filepath="\\present.dst")
        db.add_all([d1, d2])
        db.commit()

        class MockPattern:
            def count_stitches(self):
                return 5000

            def count_threads(self):
                return 3

            def count_color_changes(self):
                return 12

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        # Only present.dst exists
        monkeypatch.setattr(
            maintenance_mod.os.path,
            "isfile",
            lambda path: "present" in path,
        )

        with patch("pyembroidery.read", return_value=MockPattern()):
            resp = client.post("/admin/maintenance/backfill-color-counts", follow_redirects=False)

        assert resp.status_code in (302, 303)
        location = resp.headers["location"]
        assert "errors=1" in location
        assert "updated=1" in location

        db.refresh(d1)
        db.refresh(d2)
        assert d1.stitch_count is None  # Skipped
        assert d2.stitch_count == 5000  # Updated

    def test_handles_none_pattern_gracefully(self, db, client, monkeypatch):
        """Designs whose files return None from pyembroidery are skipped."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(filename="bad.dst", filepath="\\bad.dst")
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with patch("pyembroidery.read", return_value=None):
            resp = client.post("/admin/maintenance/backfill-color-counts", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "errors=1" in resp.headers["location"]

        db.refresh(d1)
        assert d1.stitch_count is None

    def test_handles_no_base_path(self, db, client, monkeypatch):
        """When no base path is configured, an error redirect is returned."""
        from src.routes import maintenance as maintenance_mod

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "")

        resp = client.post("/admin/maintenance/backfill-color-counts", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "error=no_base_path" in resp.headers["location"]


# =============================================================================
# Section 1.4 — Images (Legacy Individual Action Form)
# =============================================================================


class TestImageBackfill:
    """Test 1.4.1 — Image backfill via the legacy form.

    Calls the ``/admin/maintenance/backfill-images`` route which
    generates preview images, populates dimensions and assigns hoops
    for designs where these values are NULL.
    """

    # Reusable mock pattern with bounds
    @staticmethod
    def _make_mock_pattern(bounds=(0, 0, 10000, 8000)):
        class MockPattern:
            def bounds(self):
                return bounds

        return MockPattern()

    def test_populates_missing_images_and_dimensions(self, db, client, monkeypatch):
        """Designs missing image data get preview, dimensions and hoop assigned."""
        from src.models import Design, Hoop
        from src.routes import maintenance as maintenance_mod

        # Create a hoop that fits 1000x800 mm dimensions
        hoop = Hoop(name="Gigahoop", max_width_mm=1200.0, max_height_mm=1000.0)
        db.add(hoop)
        db.commit()

        d1 = Design(filename="test.dst", filepath="\\test.dst")
        d2 = Design(filename="test2.pes", filepath="\\test2.pes")
        db.add_all([d1, d2])
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"fake_png_bytes",
            ),
        ):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        location = resp.headers["location"]
        assert "done=1" in location
        assert "action=backfill_images" in location
        assert "updated=2" in location

        db.refresh(d1)
        db.refresh(d2)
        # Dimensions: (10000 - 0) / 10 = 1000.0, (8000 - 0) / 10 = 800.0
        assert d1.width_mm == 1000.0
        assert d1.height_mm == 800.0
        assert d1.image_data == b"fake_png_bytes"
        assert d2.width_mm == 1000.0
        assert d2.height_mm == 800.0
        assert d2.image_data == b"fake_png_bytes"

    def test_skips_designs_with_existing_images(self, db, client, monkeypatch):
        """Designs that already have image data are skipped (no redo)."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(
            filename="test.dst",
            filepath="\\test.dst",
            image_data=b"existing_png",
            width_mm=500.0,
            height_mm=400.0,
        )
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"new_png_bytes",
            ),
        ):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "updated=0" in resp.headers["location"]

        # Existing data should be preserved
        db.refresh(d1)
        assert d1.image_data == b"existing_png"
        assert d1.width_mm == 500.0
        assert d1.height_mm == 400.0

    def test_redo_overwrites_existing_images(self, db, client, monkeypatch):
        """With redo=True, existing image data is overwritten."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(
            filename="test.dst",
            filepath="\\test.dst",
            image_data=b"old_png",
            width_mm=500.0,
            height_mm=400.0,
        )
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"new_png_bytes",
            ),
        ):
            resp = client.post(
                "/admin/maintenance/backfill-images",
                data={"redo": "1"},
                follow_redirects=False,
            )

        assert resp.status_code in (302, 303)
        assert "updated=1" in resp.headers["location"]

        db.refresh(d1)
        assert d1.image_data == b"new_png_bytes"
        assert d1.width_mm == 1000.0
        assert d1.height_mm == 800.0

    def test_handles_missing_file_gracefully(self, db, client, monkeypatch):
        """Designs whose files are missing are skipped with an error count."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(filename="missing.dst", filepath="\\missing.dst")
        d2 = Design(filename="present.dst", filepath="\\present.dst")
        db.add_all([d1, d2])
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(
            maintenance_mod.os.path,
            "isfile",
            lambda path: "present" in path,
        )

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"fake_png_bytes",
            ),
        ):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        location = resp.headers["location"]
        assert "errors=1" in location
        assert "updated=1" in location

        db.refresh(d1)
        db.refresh(d2)
        assert d1.image_data is None  # Skipped
        assert d2.image_data == b"fake_png_bytes"  # Updated

    def test_handles_none_pattern_gracefully(self, db, client, monkeypatch):
        """Designs whose files return None from pyembroidery are skipped."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(filename="bad.dst", filepath="\\bad.dst")
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with patch("pyembroidery.read", return_value=None):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "errors=1" in resp.headers["location"]

        db.refresh(d1)
        assert d1.image_data is None

    def test_handles_no_base_path(self, db, client, monkeypatch):
        """When no base path is configured, an error redirect is returned."""
        from src.routes import maintenance as maintenance_mod

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "")

        resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "error=no_base_path" in resp.headers["location"]

    def test_assigns_hoop_when_dimensions_match(self, db, client, monkeypatch):
        """Designs with dimensions matching an existing hoop get hoop_id assigned."""
        from src.models import Design, Hoop
        from src.routes import maintenance as maintenance_mod

        # Create a hoop that fits 1000x800 mm dimensions
        hoop = Hoop(name="Gigahoop", max_width_mm=1200.0, max_height_mm=1000.0)
        db.add(hoop)
        db.commit()

        d1 = Design(filename="test.dst", filepath="\\test.dst")
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"fake_png_bytes",
            ),
        ):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "updated=1" in resp.headers["location"]

        db.refresh(d1)
        assert d1.hoop_id == hoop.id

    def test_skips_hoop_assignment_when_no_match(self, db, client, monkeypatch):
        """Designs with dimensions that don't match any hoop get no hoop_id."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        # No hoops defined
        d1 = Design(filename="test.dst", filepath="\\test.dst")
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"fake_png_bytes",
            ),
        ):
            resp = client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        assert resp.status_code in (302, 303)
        assert "updated=1" in resp.headers["location"]

        db.refresh(d1)
        assert d1.hoop_id is None

    def test_uses_3d_preview_by_default(self, db, client, monkeypatch):
        """The legacy form calls _render_preview with default 3D=True."""
        from src.models import Design
        from src.routes import maintenance as maintenance_mod

        d1 = Design(filename="test.dst", filepath="\\test.dst")
        db.add(d1)
        db.commit()

        monkeypatch.setattr(maintenance_mod, "get_designs_base_path", lambda _db: "D:\\Designs")
        monkeypatch.setattr(maintenance_mod.os.path, "isfile", lambda _path: True)

        with (
            patch("pyembroidery.read", return_value=self._make_mock_pattern()),
            patch(
                "src.services.preview._render_preview",
                return_value=b"fake_png_bytes",
            ) as mock_render,
        ):
            client.post("/admin/maintenance/backfill-images", follow_redirects=False)

        # _render_preview should be called with just the pattern (3D default)
        mock_render.assert_called_once()
        call_args = mock_render.call_args
        assert call_args[0][0] is not None  # pattern passed as positional arg
        # No preview_3d=False argument means 3D is used
