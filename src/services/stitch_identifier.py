import math

from pyembroidery import COMMAND_MASK, JUMP, STITCH, TRIM


class StitchIdentifier:
    STITCH_TYPES = [
        "applique",
        "cross_stitch",
        "cutwork",
        "filled",
        "ith",
        "lace",
        "outline",
        "satin",
    ]

    NAME_KEYWORDS = {
        "ith": ("in the hoop", "ith", "hoop"),
        "applique": ("applique", "appliquee", "appliqué", "appique"),
        "cross_stitch": ("cross stitch", "cross-stitch", "cross_stitch"),
        "lace": ("lace", "fsl", "freestanding lace", "free standing lace"),
    }

    def __init__(self, pattern, filename, folder_name, confidence_threshold=0.70):
        self.pattern = pattern
        self.filename = filename
        self.folder_name = folder_name
        self.confidence_threshold = float(confidence_threshold)
        self._color_blocks = list(pattern.get_as_colorblocks())
        self._stitch_blocks = list(pattern.get_as_stitchblock())
        self._vectors = self._build_vectors()
        # Strip type prefix (e.g. "blackwork__anime4.jef" -> "anime4.jef") so that
        # previously-prefixed copies don't falsely trigger name-based detection.
        raw_filename = filename or ""
        if "__" in raw_filename:
            raw_filename = raw_filename.split("__", 1)[1]
        self._name_text = ((folder_name or "") + " " + raw_filename).lower()

    def identify_stitches(self):
        scores = self.get_detailed_analysis()
        found = []
        for stitch_type in self.STITCH_TYPES:
            if scores.get(stitch_type, 0.0) >= self.confidence_threshold:
                found.append(stitch_type)
        # Satin takes precedence over running-style categories when satin score
        # is strong enough, even if it is slightly below the main threshold.
        # However, do not override a strongly confident outline classification
        # (>= 0.78): very parallel sparse lines score high on satin-like metrics
        # but are unmistakably running/outline stitch at that confidence level.
        satin_precedence_threshold = max(0.63, self.confidence_threshold - 0.07)
        if (
            scores.get("satin", 0.0) >= satin_precedence_threshold
            and "lace" not in found
            and scores.get("outline", 0.0) < 0.78
        ):
            if "satin" not in found:
                found.append("satin")
            for running_type in ("outline",):
                if running_type in found:
                    found.remove(running_type)
        # Lace designs frequently contain dense areas that can look "filled".
        # Treat lace as more specific and suppress generic filled classification.
        if "lace" in found and "filled" in found:
            found.remove("filled")
        # Cross stitch X-arms are geometrically similar to satin columns, and
        # repeated X patterns mimic applique path repetition, and many colors
        # inflate the filled color-coverage score. Suppress all three when
        # cross_stitch is confidently detected.
        if "cross_stitch" in found:
            for false_positive in ("applique", "filled", "satin"):
                if false_positive in found:
                    found.remove(false_positive)
        # Applique already contains a satin border by definition; suppress the
        # redundant satin classification when applique is detected.
        if "applique" in found:
            for redundant in ("satin", "outline"):
                if redundant in found:
                    found.remove(redundant)
        # If no type reaches threshold, allow satin as a fallback for borderline
        # satin-column designs that otherwise end up as unknown.
        if (
            not found
            and scores.get("satin", 0.0) >= max(0.58, self.confidence_threshold - 0.12)
            and scores.get("outline", 0.0) < 0.60
        ):
            found.append("satin")
        # Mixed designs can split confidence across multiple types (e.g. half
        # filled, half outline), leaving both just below threshold. If nothing
        # was found, include up to two close near-threshold leaders.
        if not found:
            ranked = sorted(
                ((name, score) for name, score in scores.items() if score > 0.0),
                key=lambda item: item[1],
                reverse=True,
            )
            if len(ranked) >= 2:
                top_name, top_score = ranked[0]
                second_name, second_score = ranked[1]
                near_threshold = self.confidence_threshold - 0.12
                close_pair = abs(top_score - second_score) <= 0.10
                # Only use mixed fallback for fill-mix cases; avoid forcing
                # ambiguous outline/satin patterns out of unknown.
                fill_mix = top_name == "filled" or second_name == "filled"
                if (
                    top_score >= near_threshold
                    and second_score >= near_threshold
                    and close_pair
                    and fill_mix
                ):
                    found = [top_name, second_name]
        # Sparse pictorial outlines score lower than dense fills. Allow outline
        # as a last-resort fallback when nothing else fired, satin is not
        # competing, and the threshold is at/near default (not user-raised).
        if (
            not found
            and scores.get("outline", 0.0) >= 0.48
            and scores.get("satin", 0.0) < 0.58
            and self.confidence_threshold <= 0.75
        ):
            found.append("outline")
        return sorted(found)

    def get_detailed_analysis(self):
        return {
            "cross_stitch": self._detect_cross_stitch(),
            "ith": self._detect_ith(),
            "applique": self._detect_applique(),
            "filled": self._detect_filled(),
            "cutwork": self._detect_cutwork(),
            "lace": self._detect_lace(),
            "outline": self._detect_outline(),
            "satin": self._detect_satin(),
        }

    def _detect_cross_stitch(self):
        name_conf = self._name_confidence("cross_stitch")
        if not self._vectors:
            return name_conf if name_conf else 0.0
        satin = self._detect_satin_like_score()
        # Cross stitch = X-shaped stitches: two short arms crossing at ~90°.
        # Use tight angle tolerance so near-horizontal/vertical fill traversal
        # stitches are not miscounted as diagonals.
        slash = 0
        backslash = 0
        diagonal = 0
        orthogonal = 0
        for v in self._vectors:
            if v["length"] < 0.1:
                continue
            angle = v["angle"]
            if self._angle_close(angle, 45, 20) or self._angle_close(angle, 225, 20):
                slash += 1
                diagonal += 1
            elif self._angle_close(angle, 135, 20) or self._angle_close(angle, 315, 20):
                backslash += 1
                diagonal += 1
            elif (
                self._angle_close(angle, 0, 20)
                or self._angle_close(angle, 90, 20)
                or self._angle_close(angle, 180, 20)
                or self._angle_close(angle, 270, 20)
            ):
                orthogonal += 1
        if diagonal == 0:
            return 0.0
        # Balance: both slash and backslash must be present to form an X.
        balance = min(slash, backslash) / float(max(slash, backslash) or 1)
        diagonal_ratio = diagonal / float(len(self._vectors) or 1)
        # X-purity: true cross stitch has mostly diagonal strokes.
        cross_purity = diagonal / float((diagonal + orthogonal) or 1)
        # Length uniformity: X arms are short and consistent in length.
        # Fill traversal stitches produce high variance (low uniformity).
        lengths = [v["length"] for v in self._vectors]
        mean_len = sum(lengths) / len(lengths)
        variance = sum((length - mean_len) ** 2 for length in lengths) / len(lengths)
        std_len = math.sqrt(variance)
        cv = std_len / max(mean_len, 0.1)  # coefficient of variation
        uniformity = max(0.0, 1.0 - min(1.0, cv))
        base = min(
            1.0, 0.35 * balance + 0.30 * diagonal_ratio + 0.20 * uniformity + 0.15 * cross_purity
        )
        satin_penalty = max(0.0, min(1.0, (satin - 0.58) / 0.2))
        if cross_purity >= 0.85 and balance >= 0.75:
            satin_penalty *= 0.25
        score = base * (1.0 - 0.6 * satin_penalty)
        # Multi-colour cross stitch can include a separate outline colour block.
        # If any block is strongly cross-like, promote cross_stitch score so
        # cross+outline designs are not misread as applique.
        if len(self._color_blocks) >= 2:
            best_block_cross = 0.0
            for block, _thread in self._color_blocks:
                block_vectors = self._vectors_from_block(block)
                block_base, block_diag_ratio, block_cross_purity, block_balance = (
                    self._cross_base_from_vectors(block_vectors)
                )
                if (
                    block_diag_ratio >= 0.90
                    and block_cross_purity >= 0.90
                    and block_balance >= 0.70
                ):
                    best_block_cross = max(best_block_cross, block_base)
            if best_block_cross > 0.0:
                score = max(score, min(1.0, best_block_cross * 0.95))
        return score

    def _detect_ith(self):
        name_conf = self._name_confidence("ith")
        if name_conf:
            return name_conf
        if not self._vectors:
            return 0.0
        overlap_score = self._color_block_overlap_score()
        running_score = self._running_like_score()
        satin_score = self._detect_satin()
        path_repeat = self._path_repeat_score()
        trims = self.pattern.count_stitch_commands(TRIM)
        jumps = self.pattern.count_stitch_commands(JUMP)
        assembly_activity = min(1.0, (trims + jumps) / float((len(self._vectors) / 8.0) + 1.0))
        # Pattern-based ITH should show repeated placement/tack-down paths or
        # noticeable assembly-step activity.
        # Requiring repeated-path evidence avoids classifying designs with many
        # jumps but no fabric placement/tack-down loops as ITH.
        if path_repeat < 0.16:
            return 0.0
        # Cross stitch designs score high on path_repeat (repeated X patterns)
        # but are not ITH; suppress when cross stitch is strong and assembly
        # activity is negligible (real ITH needs fabric placement steps).
        if assembly_activity <= 0.05 and self._detect_cross_stitch() >= 0.75:
            return 0.0
        return min(
            1.0,
            0.28 * overlap_score
            + 0.20 * running_score
            + 0.22 * satin_score
            + 0.15 * assembly_activity
            + 0.15 * path_repeat,
        )

    def _detect_applique(self):
        name_conf = self._name_confidence("applique")
        if name_conf:
            return name_conf
        if not self._vectors:
            return 0.0
        satin_score = self._detect_satin()
        path_repeat = self._path_repeat_score()
        # Path A: repeated-path applique — single/few colour blocks where stitches
        # retrace the same outline (placement run + tack-down in same thread).
        if path_repeat >= 0.2:
            score = min(1.0, 0.55 * path_repeat + 0.45 * satin_score)
            # When multiple colour blocks all cover the same region (e.g. placement run,
            # tack-down run, cover stitch in separate colours), the overlap is a stronger
            # signal than satin alone — boost using overlap + running score.
            if len(self._color_blocks) >= 2:
                overlap = self._color_block_overlap_score()
                if overlap >= 0.8:
                    running = self._running_like_score()
                    boosted = min(
                        1.0,
                        0.35 * path_repeat + 0.35 * overlap + 0.20 * satin_score + 0.10 * running,
                    )
                    score = max(score, boosted)
            return score
        # Path B: multi-colour-block applique — placement run, tack-down run, and
        # satin border each in a different colour, so path_repeat is diluted by the
        # satin block. The key signal is that all colour passes cover the same region.
        # For multi-object applique (e.g. 2 hearts), one colour block spans both objects
        # joined by an internal jump stitch, which slightly reduces overlap. Detect
        # this by checking for any colour block that has stitches on both sides of an
        # internal jump/trim (indicating multiple objects in one colour stop).
        # Applique requires multiple colour blocks that stitch in the same region.
        # Check both: (1) bounding box intersection to filter out spatially separate
        # colour blocks (e.g. Y2.jef with one design in different location), and
        # (2) overlap score to ensure all blocks cover the same stitched area.
        if len(self._color_blocks) >= 2:
            # Early exit if colour blocks stitch in different places
            if not self._color_blocks_bounding_boxes_intersect():
                return 0.0
            overlap = self._color_block_overlap_score()
            density = self._stitch_density_score()
            if overlap >= 0.55 and density < 0.65:
                running = self._running_like_score()
                # Boost score when a block contains internal jumps (multi-object signal).
                has_internal_jump = self._has_internal_jump_in_any_block()
                jump_boost = 0.06 if has_internal_jump else 0.0
                return min(
                    1.0,
                    0.40 * overlap
                    + 0.30 * running
                    + 0.20 * satin_score
                    + 0.10 * (1.0 - density)
                    + jump_boost,
                )
        return 0.0

    def _detect_filled(self, _no_cross: bool = False):
        if not self._vectors:
            return 0.0
        density = self._stitch_density_score()
        outline = self._detect_outline()
        base = self._detect_filled_like_score()
        density = self._stitch_density_score()
        outline = self._detect_outline()
        # Rule 1: Dense, low-outline single-color blocks
        if density >= 0.41 and outline <= 0.38:
            base = max(base, 0.72)

        # Rule 2: Medium+ density single-color (ivy/filled designs density >= 0.29)
        if len(self._color_blocks) == 1 and density >= 0.29 and outline < 0.58:
            # Avoid recursion: if _no_cross, do not call _detect_satin
            satin_score = (
                self._detect_satin(_no_cross=True)
                if not _no_cross
                else self._detect_satin_like_score()
            )
            if satin_score < 0.55:
                base = max(base, min(1.0, 0.62 + 0.30 * density))

        # Rule 3: Aligned overlapping-row fill (Leaves.jef pattern)
        if len(self._color_blocks) == 1 and 0.20 <= density <= 0.40:
            satin_score = self._detect_satin_like_score()  # pre-penalty!
            axis_ratio = self._geometric_angle_score()
            turns = self._direction_change_score()
            if 0.62 <= satin_score <= 0.75 and axis_ratio >= 0.93 and turns <= 0.40:
                base = max(base, 0.72)
        return base

    def _detect_cutwork(self):
        if not self._vectors:
            return 0.0
        outline = self._detect_outline()
        satin = self._detect_satin()
        trims = self.pattern.count_stitch_commands(TRIM)
        trim_score = min(1.0, trims / float((len(self._vectors) / 12.0) + 1.0))
        return min(1.0, 0.35 * outline + 0.45 * satin + 0.2 * trim_score)

    def _detect_lace(self):
        name_conf = self._name_confidence("lace")
        if name_conf:
            return name_conf
        if not self._vectors:
            return 0.0
        satin = self._detect_satin_like_score()
        running = self._running_like_score()
        filled_like = self._detect_filled_like_score()
        density = self._stitch_density_score()
        overlap = self._color_block_overlap_score()
        # Lace tends to be one/few thread colors and stitched continuously.
        color_blocks = len(self._color_blocks)
        color_score = 1.0 if color_blocks <= 2 else max(0.0, 1.0 - ((color_blocks - 2.0) / 4.0))
        jumps = self.pattern.count_stitch_commands(JUMP)
        trims = self.pattern.count_stitch_commands(TRIM)
        continuity = 1.0 - min(1.0, (jumps + trims) / float((len(self._vectors) / 25.0) + 1.0))

        # Must look like satin columns connected by running stitches, with at
        # least some openwork. Very dense full coverage is not lace.
        if satin < 0.55 or running < 0.65 or filled_like < 0.6:
            return 0.0
        if density > 0.85:
            return 0.0
        # Multiple disconnected stitch blocks without overlap are unlikely to
        # be freestanding lace structures.
        if len(self._stitch_blocks) > 1 and overlap <= 0.0:
            return 0.0

        return min(
            1.0,
            0.30 * satin
            + 0.25 * running
            + 0.20 * filled_like
            + 0.15 * color_score
            + 0.10 * continuity,
        )

    def _detect_outline(self):
        if not self._vectors:
            return 0.0
        running = self._running_like_score()
        density = self._stitch_density_score()
        satin = self._detect_satin_like_score()
        fill = self._detect_filled_like_score()
        return max(0.0, min(1.0, 0.8 * running + 0.2 * (1.0 - density) - 0.25 * satin - 0.2 * fill))

    def _detect_satin(self, _no_cross: bool = False):
        if not self._vectors:
            return 0.0
        score = self._detect_satin_like_score()
        # Penalize satin for aligned overlapping-row fills that are area fill,
        # not satin columns (single-colour or multi-colour, medium+ density, very low turn rate).
        density = self._stitch_density_score()
        axis_ratio = self._geometric_angle_score()
        turns = self._direction_change_score()
        filled = (
            self._detect_filled_like_score() if _no_cross else self._detect_filled(_no_cross=True)
        )
        outline = self._detect_outline()
        # Single-color penalty (existing rule)
        if (
            len(self._color_blocks) == 1
            and 0.20 <= density <= 0.40
            and axis_ratio >= 0.93
            and turns <= 0.40
        ):
            score *= 0.78
        # Multi-color overlapping-row fill penalty
        if (
            len(self._color_blocks) > 1
            and 0.20 <= density <= 0.80
            and axis_ratio >= 0.80
            and turns <= 0.25
            and filled >= 0.65
            and outline < 0.70
        ):
            score *= 0.78
        return score

    def _detect_satin(self, _no_cross=False):
        return self._detect_satin_inner(_no_cross=_no_cross)

    def _detect_satin_inner(self, _no_cross=False):
        score = self._detect_satin_like_score()
        # Penalize satin for aligned overlapping-row fills that are area fill,
        # not satin columns (single-colour or multi-colour, medium+ density, very low turn rate).
        density = self._stitch_density_score()
        axis_ratio = self._geometric_angle_score()
        turns = self._direction_change_score()
        # Only use base filled/outline for cross-feature checks to avoid recursion
        filled = (
            self._detect_filled_like_score() if _no_cross else self._detect_filled(_no_cross=True)
        )
        outline = self._detect_outline()
        # Single-color penalty (existing rule)
        if (
            len(self._color_blocks) == 1
            and 0.20 <= density <= 0.40
            and axis_ratio >= 0.93
            and turns <= 0.40
        ):
            score *= 0.78
        # Multi-color overlapping-row fill penalty
        if (
            len(self._color_blocks) > 1
            and 0.20 <= density <= 0.80
            and axis_ratio >= 0.80
            and turns <= 0.25
            and filled >= 0.65
            and outline < 0.70
        ):
            score *= 0.78
        return score

    def _name_confidence(self, stitch_type):
        keywords = self.NAME_KEYWORDS.get(stitch_type)
        if not keywords:
            return 0.0
        for keyword in keywords:
            if keyword in self._name_text:
                return 0.95
        return 0.0

    def _build_vectors(self):
        vectors = []
        prev = None
        for stitch in self.pattern.stitches:
            command = stitch[2] & COMMAND_MASK
            if command != STITCH:
                prev = None
                continue
            if prev is not None:
                dx = stitch[0] - prev[0]
                dy = stitch[1] - prev[1]
                length = math.sqrt((dx * dx) + (dy * dy))
                if length > 0:
                    angle = math.degrees(math.atan2(dy, dx))
                    if angle < 0:
                        angle += 360.0
                    vectors.append({"dx": dx, "dy": dy, "length": length, "angle": angle})
            prev = stitch
        return vectors

    def _running_like_score(self):
        vectors = self._vectors
        if not vectors:
            return 0.0
        lengths = [v["length"] for v in vectors]
        avg_length = sum(lengths) / float(len(lengths))
        short_ratio = float(
            len([length for length in lengths if length <= avg_length * 1.35])
        ) / float(len(lengths))
        return min(1.0, short_ratio)

    def _stitch_density_score(self):
        stitch_count = len(self._vectors)
        if stitch_count <= 0:
            return 0.0
        min_x, min_y, max_x, max_y = self._stitch_bounds()
        width = max(1.0, max_x - min_x)
        height = max(1.0, max_y - min_y)
        area = width * height
        if area <= 0:
            return 0.0
        density = stitch_count / float(area)
        return min(1.0, density * 50.0)

    def _detect_satin_like_score(self):
        vectors = self._vectors
        if len(vectors) < 6:
            return 0.0
        long_lengths = [v["length"] for v in vectors]
        avg_len = sum(long_lengths) / float(len(long_lengths))
        long_ratio = float(len([length for length in long_lengths if length >= avg_len])) / float(
            len(long_lengths)
        )
        axis_ratio = self._geometric_angle_score()
        turns = self._direction_change_score()
        return min(1.0, 0.45 * long_ratio + 0.35 * axis_ratio + 0.2 * turns)

    def _detect_filled_like_score(self):
        return min(1.0, 0.6 * self._stitch_density_score() + 0.4 * self._direction_change_score())

    def _direction_change_score(self):
        vectors = self._vectors
        if len(vectors) < 3:
            return 0.0
        changes = 0
        total = 0
        last_angle = vectors[0]["angle"]
        for vector in vectors[1:]:
            total += 1
            diff = self._angle_diff(last_angle, vector["angle"])
            if diff > 45:
                changes += 1
            last_angle = vector["angle"]
        if total <= 0:
            return 0.0
        return changes / float(total)

    def _geometric_angle_score(self):
        vectors = self._vectors
        if not vectors:
            return 0.0
        anchors = (0, 45, 90, 135, 180, 225, 270, 315)
        matches = 0
        for vector in vectors:
            angle = vector["angle"]
            for anchor in anchors:
                if self._angle_close(angle, anchor, 16):
                    matches += 1
                    break
        return matches / float(len(vectors))

    def _has_internal_jump_in_any_block(self):
        """Return True if any colour block contains stitches on both sides of a
        jump/trim, indicating multiple objects sewn within one colour stop."""
        for block, _thread in self._color_blocks:
            seen_stitch = False
            in_jump = False
            for cmd_tuple in block:
                cmd = cmd_tuple[2] & COMMAND_MASK
                if cmd == STITCH:
                    if in_jump and seen_stitch:
                        return True
                    seen_stitch = True
                    in_jump = False
                elif cmd in (JUMP, TRIM):
                    in_jump = True
        return False

    def _path_repeat_score(self):
        points = []
        for stitch in self.pattern.stitches:
            command = stitch[2] & COMMAND_MASK
            if command == STITCH:
                points.append((round(stitch[0], 1), round(stitch[1], 1)))
        if len(points) < 6:
            return 0.0
        freq = {}
        for pt in points:
            freq[pt] = freq.get(pt, 0) + 1
        repeated = 0
        for count in freq.values():
            if count > 1:
                repeated += 1
        return min(1.0, repeated / float(len(freq) or 1))

    def _vectors_from_block(self, block):
        points = []
        for stitch in block:
            command = stitch[2] & COMMAND_MASK
            if command == STITCH:
                points.append((stitch[0], stitch[1]))
        vectors = []
        if len(points) < 2:
            return vectors
        for i in range(1, len(points)):
            dx = points[i][0] - points[i - 1][0]
            dy = points[i][1] - points[i - 1][1]
            length = math.hypot(dx, dy)
            if length < 0.1:
                continue
            angle = (math.degrees(math.atan2(dy, dx)) + 360.0) % 360.0
            vectors.append({"length": length, "angle": angle})
        return vectors

    def _cross_base_from_vectors(self, vectors):
        if not vectors:
            return 0.0, 0.0, 0.0, 0.0
        slash = 0
        backslash = 0
        diagonal = 0
        orthogonal = 0
        for v in vectors:
            if v["length"] < 0.1:
                continue
            angle = v["angle"]
            if self._angle_close(angle, 45, 20) or self._angle_close(angle, 225, 20):
                slash += 1
                diagonal += 1
            elif self._angle_close(angle, 135, 20) or self._angle_close(angle, 315, 20):
                backslash += 1
                diagonal += 1
            elif (
                self._angle_close(angle, 0, 20)
                or self._angle_close(angle, 90, 20)
                or self._angle_close(angle, 180, 20)
                or self._angle_close(angle, 270, 20)
            ):
                orthogonal += 1
        if diagonal == 0:
            return 0.0, 0.0, 0.0, 0.0
        balance = min(slash, backslash) / float(max(slash, backslash) or 1)
        diagonal_ratio = diagonal / float(len(vectors) or 1)
        cross_purity = diagonal / float((diagonal + orthogonal) or 1)
        lengths = [v["length"] for v in vectors]
        mean_len = sum(lengths) / len(lengths)
        variance = sum((length - mean_len) ** 2 for length in lengths) / len(lengths)
        std_len = math.sqrt(variance)
        cv = std_len / max(mean_len, 0.1)
        uniformity = max(0.0, 1.0 - min(1.0, cv))
        base = min(
            1.0, 0.35 * balance + 0.30 * diagonal_ratio + 0.20 * uniformity + 0.15 * cross_purity
        )
        return base, diagonal_ratio, cross_purity, balance

    def _color_block_overlap_score(self):
        if len(self._color_blocks) < 2:
            return 0.0
        boxes = []
        for block, _thread in self._color_blocks:
            boxes.append(self._block_bounds(block))
        overlaps = 0
        total = 0
        for i in range(0, len(boxes)):
            a = boxes[i]
            for j in range(i + 1, len(boxes)):
                b = boxes[j]
                total += 1
                if self._boxes_overlap(a, b):
                    overlaps += 1
        if total == 0:
            return 0.0
        return overlaps / float(total)

    def _stitch_bounds(self):
        stitches = self.pattern.stitches
        if not stitches:
            return 0.0, 0.0, 0.0, 0.0
        min_x = float("inf")
        min_y = float("inf")
        max_x = -float("inf")
        max_y = -float("inf")
        for stitch in stitches:
            command = stitch[2] & COMMAND_MASK
            if command != STITCH:
                continue
            x = stitch[0]
            y = stitch[1]
            if x < min_x:
                min_x = x
            if y < min_y:
                min_y = y
            if x > max_x:
                max_x = x
            if y > max_y:
                max_y = y
        if min_x == float("inf"):
            return 0.0, 0.0, 0.0, 0.0
        return min_x, min_y, max_x, max_y

    @staticmethod
    def _block_bounds(block):
        min_x = float("inf")
        min_y = float("inf")
        max_x = -float("inf")
        max_y = -float("inf")
        for stitch in block:
            x = stitch[0]
            y = stitch[1]
            if x < min_x:
                min_x = x
            if y < min_y:
                min_y = y
            if x > max_x:
                max_x = x
            if y > max_y:
                max_y = y
        if min_x == float("inf"):
            return 0.0, 0.0, 0.0, 0.0
        return min_x, min_y, max_x, max_y

    @staticmethod
    def _boxes_overlap(a, b):
        if a[2] < b[0] or b[2] < a[0]:
            return False
        if a[3] < b[1] or b[3] < a[1]:
            return False
        return True

    def _color_blocks_bounding_boxes_intersect(self):
        """Check if colour block bounding boxes suggest applique (similar sizes) vs
        separate designs (one tiny + one massive).
        Returns True if max/min area ratio <= 5.0; False for drastically different sizes.
        This filters out Y2.jef (7.0 ratio) while allowing multi-object applique variations.
        """
        if len(self._color_blocks) < 2:
            return True

        boxes = [self._block_bounds(block) for block, _thread in self._color_blocks]
        areas = [(b[2] - b[0]) * (b[3] - b[1]) for b in boxes]
        if any(a <= 0 for a in areas):
            return False

        # Reject if one block is > 5x another's size (indicates separate designs)
        max_area = max(areas)
        min_area = min(areas)
        return max_area / min_area <= 5.0

    @staticmethod
    def _angle_diff(a, b):
        d = abs(a - b) % 360.0
        if d > 180.0:
            d = 360.0 - d
        return d

    def _angle_close(self, a, b, tolerance):
        return self._angle_diff(a, b) <= tolerance
