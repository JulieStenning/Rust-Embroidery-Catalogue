import unittest
from unittest.mock import patch

from pyembroidery import EmbPattern

from src.services.stitch_identifier import StitchIdentifier


class TestStitchIdentifier(unittest.TestCase):
    def test_constructor_requires_filename_and_folder(self):
        pattern = EmbPattern()
        with self.assertRaises(TypeError):
            StitchIdentifier(pattern)

    def test_name_based_ith_precedence_and_confidence(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "my_design.pes", "in_the_hoop")
        details = analyzer.get_detailed_analysis()
        self.assertEqual(details["ith"], 0.95)
        self.assertIn("ith", analyzer.identify_stitches())

    def test_name_based_applique_precedence_and_confidence(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "flower_applique.pes", "designs")
        details = analyzer.get_detailed_analysis()
        self.assertEqual(details["applique"], 0.95)
        self.assertIn("applique", analyzer.identify_stitches())

    def test_name_based_cross_stitch(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "cross_stitch_pattern.pes", "samples")
        details = analyzer.get_detailed_analysis()
        self.assertEqual(details["cross_stitch"], 0.95)
        detected = analyzer.identify_stitches()
        self.assertIn("cross_stitch", detected)

    def test_name_based_lace_precedence_and_confidence(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "wedding_lace_design.pes", "samples")
        details = analyzer.get_detailed_analysis()
        self.assertEqual(details["lace"], 0.95)
        self.assertIn("lace", analyzer.identify_stitches())

    def test_outline_pattern_detection(self):
        pattern = EmbPattern()
        pattern.add_block([(0, 0), (0, 20), (20, 20), (20, 0), (0, 0)], "red")
        analyzer = StitchIdentifier(pattern, "shape.pes", "outlines")
        details = analyzer.get_detailed_analysis()
        self.assertGreaterEqual(details["outline"], 0.70)
        self.assertIn("outline", analyzer.identify_stitches())

    def test_cross_stitch_pattern_detection(self):
        pattern = EmbPattern()
        pattern.add_thread("red")
        points = [
            (0, 0),
            (5, 5),
            (10, 0),
            (15, 5),
            (20, 0),
            (15, -5),
            (10, 0),
            (5, -5),
            (0, 0),
        ]
        pattern.add_block(points)
        analyzer = StitchIdentifier(pattern, "sample.pes", "geometric")
        self.assertIn("cross_stitch", analyzer.identify_stitches())

    def test_configurable_threshold(self):
        pattern = EmbPattern()
        pattern.add_block([(0, 0), (0, 20), (20, 20), (20, 0), (0, 0)], "red")
        strict = StitchIdentifier(pattern, "shape.pes", "outlines", confidence_threshold=0.95)
        relaxed = StitchIdentifier(pattern, "shape.pes", "outlines", confidence_threshold=0.50)
        self.assertNotIn("outline", strict.identify_stitches())
        self.assertIn("outline", relaxed.identify_stitches())

    def test_satin_like_not_cross_stitch(self):
        pattern = EmbPattern()
        points = []
        for y in range(0, 120, 4):
            points.append((0, y))
            points.append((24, y + 2))
        pattern.add_block(points, "red")

        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        details = analyzer.get_detailed_analysis()

        self.assertGreaterEqual(details["satin"], 0.70)
        self.assertLess(details["cross_stitch"], 0.70)
        detected = analyzer.identify_stitches()
        self.assertNotIn("cross_stitch", detected)

    def test_multicolor_dense_areas_detect_filled(self):
        pattern = EmbPattern()
        for y_offset in (0, 30, 60):
            block = []
            for y in range(y_offset, y_offset + 20, 2):
                block.append((0, y))
                block.append((40, y))
            pattern.add_block(block, "red")

        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        details = analyzer.get_detailed_analysis()
        self.assertGreaterEqual(details["filled"], 0.70)
        self.assertIn("filled", analyzer.identify_stitches())

    def test_satin_fallback_when_other_types_below_threshold(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        mocked_scores = {
            "cross_stitch": 0.45,
            "ith": 0.50,
            "applique": 0.0,
            "filled": 0.53,
            "cutwork": 0.40,
            "lace": 0.0,
            "outline": 0.52,
            "satin": 0.62,
        }
        with patch.object(StitchIdentifier, "get_detailed_analysis", return_value=mocked_scores):
            self.assertEqual(analyzer.identify_stitches(), ["satin"])

    def test_satin_precedence_over_running_types(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        mocked_scores = {
            "cross_stitch": 0.45,
            "ith": 0.20,
            "applique": 0.0,
            "filled": 0.40,
            "cutwork": 0.20,
            "lace": 0.0,
            "outline": 0.71,
            "satin": 0.64,
        }
        with patch.object(StitchIdentifier, "get_detailed_analysis", return_value=mocked_scores):
            self.assertEqual(analyzer.identify_stitches(), ["satin"])

    def test_disconnected_blocks_not_lace(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        analyzer._stitch_blocks = [
            ([(0, 0, 0), (1, 0, 0)], None),
            ([(10, 0, 0), (11, 0, 0)], None),
            ([(20, 0, 0), (21, 0, 0)], None),
        ]
        with (
            patch.object(StitchIdentifier, "_detect_satin_like_score", return_value=0.70),
            patch.object(StitchIdentifier, "_running_like_score", return_value=0.80),
            patch.object(StitchIdentifier, "_detect_filled_like_score", return_value=0.75),
            patch.object(StitchIdentifier, "_stitch_density_score", return_value=0.50),
            patch.object(StitchIdentifier, "_color_block_overlap_score", return_value=0.0),
        ):
            self.assertEqual(analyzer._detect_lace(), 0.0)

    def test_dense_low_open_space_detect_filled(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        analyzer._vectors = [{"length": 1.0, "angle": 0.0}]
        analyzer._color_blocks = [([], None)]
        with (
            patch.object(StitchIdentifier, "_stitch_density_score", return_value=0.46),
            patch.object(StitchIdentifier, "_direction_change_score", return_value=0.43),
            patch.object(StitchIdentifier, "_detect_outline", return_value=0.35),
        ):
            self.assertGreaterEqual(analyzer._detect_filled(), 0.70)

    def test_mixed_near_threshold_returns_two_types(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        mocked_scores = {
            "cross_stitch": 0.20,
            "ith": 0.20,
            "applique": 0.0,
            "filled": 0.63,
            "cutwork": 0.18,
            "lace": 0.0,
            "outline": 0.62,
            "satin": 0.50,
        }
        with patch.object(StitchIdentifier, "get_detailed_analysis", return_value=mocked_scores):
            self.assertEqual(analyzer.identify_stitches(), ["filled", "outline"])

    def test_ith_pattern_requires_repeat_or_assembly(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        analyzer._vectors = [{"length": 1.0, "angle": 0.0} for _ in range(200)]

        with (
            patch.object(StitchIdentifier, "_name_confidence", return_value=0.0),
            patch.object(StitchIdentifier, "_color_block_overlap_score", return_value=1.0),
            patch.object(StitchIdentifier, "_running_like_score", return_value=0.8),
            patch.object(StitchIdentifier, "_detect_satin", return_value=0.6),
            patch.object(StitchIdentifier, "_path_repeat_score", return_value=0.05),
            patch.object(pattern, "count_stitch_commands", return_value=0),
        ):
            self.assertEqual(analyzer._detect_ith(), 0.0)

    def test_satin_fallback_blocked_for_high_outline_ambiguity(self):
        pattern = EmbPattern()
        analyzer = StitchIdentifier(pattern, "sample.pes", "tests")
        mocked_scores = {
            "cross_stitch": 0.40,
            "ith": 0.45,
            "applique": 0.0,
            "filled": 0.35,
            "cutwork": 0.30,
            "lace": 0.0,
            "outline": 0.64,
            "satin": 0.60,
        }
        with patch.object(StitchIdentifier, "get_detailed_analysis", return_value=mocked_scores):
            self.assertEqual(analyzer.identify_stitches(), [])


if __name__ == "__main__":
    unittest.main()
