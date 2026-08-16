use crate::models::{EmbPattern, Stitch, StitchType};
use crate::readers::{
    DstReader, EmbroideryReader, ExpReader, HusReader, JefReader, PesReader, Vp3Reader,
};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::fs;
use std::path::Path;

const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.70;

#[derive(Debug, Clone)]
struct Vector {
    length: f64,
    angle: f64,
}

pub fn suggest_stitching_from_pattern_file(
    pattern_path: &str,
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
    confidence_threshold: Option<f64>,
) -> Vec<String> {
    let pattern = match read_pattern_from_file(pattern_path) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    suggest_stitching_from_pattern(
        &pattern,
        filename,
        filepath,
        valid_descriptions,
        confidence_threshold,
    )
}

pub fn suggest_stitching_from_pattern(
    pattern: &EmbPattern,
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
    confidence_threshold: Option<f64>,
) -> Vec<String> {
    let folder_name = Path::new(filepath)
        .parent()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let identifier = StitchIdentifier::new(
        pattern,
        filename,
        folder_name,
        confidence_threshold.unwrap_or(DEFAULT_CONFIDENCE_THRESHOLD),
    );

    let detected = identifier.identify_stitches();
    if detected.is_empty() {
        return Vec::new();
    }

    let mut matched = Vec::new();
    let mapping = stitch_type_to_tag_map();
    for stitch_type in detected {
        if let Some(description) = mapping.get(stitch_type.as_str()) {
            if valid_descriptions.contains(*description) {
                matched.push((*description).to_string());
            }
        }
    }

    matched.sort();
    matched.dedup();
    matched
}

fn stitch_type_to_tag_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("applique", "Applique"),
        ("cross_stitch", "Cross Stitch"),
        ("cutwork", "Cutwork"),
        ("filled", "Filled"),
        ("ith", "In The Hoop"),
        ("lace", "Lace"),
        ("outline", "Line Outline"),
        ("satin", "Satin Stitch"),
    ])
}

fn read_pattern_from_file(file_path: &str) -> Result<EmbPattern, String> {
    let data = fs::read(file_path)
        .map_err(|error| format!("Could not read embroidery file '{}': {error}", file_path))?;

    let extension = Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| format!("Missing file extension for '{}'.", file_path))?;

    let parsed = match extension.as_str() {
        "pes" => PesReader.read(&data),
        "dst" => DstReader.read(&data),
        "exp" => ExpReader.read(&data),
        "jef" => JefReader.read(&data),
        "hus" => HusReader.read(&data),
        "vp3" => Vp3Reader.read(&data),
        _ => return Err(format!("Unsupported extension '.{}'", extension)),
    };

    parsed.map_err(|error| format!("Could not parse '{}': {error}", file_path))
}

struct StitchIdentifier<'a> {
    pattern: &'a EmbPattern,
    confidence_threshold: f64,
    vectors: Vec<Vector>,
    name_text: String,
    filename: String,
    folder_name: String,
}

impl<'a> StitchIdentifier<'a> {
    fn new(
        pattern: &'a EmbPattern,
        filename: &str,
        folder_name: &str,
        confidence_threshold: f64,
    ) -> Self {
        let raw_filename = if filename.contains("__") {
            filename.splitn(2, "__").nth(1).unwrap_or(filename)
        } else {
            filename
        };
        let name_text = format!("{} {}", folder_name, raw_filename).to_ascii_lowercase();

        Self {
            pattern,
            confidence_threshold,
            vectors: build_vectors(pattern),
            name_text,
            filename: filename.to_string(),
            folder_name: folder_name.to_string(),
        }
    }

    /// Classify the dominant stitch type for the design.
    ///
    /// A design is treated as having ONE primary stitch character. Stitch
    /// types are checked in a priority chain, most specific first, and the
    /// first confidently-matched type is returned alone. Lower-priority
    /// types are never considered once a higher-priority one has matched.
    ///
    /// Signal is collected per colour block first so that dense regions are
    /// not diluted by sharing the whole-pattern bounding box with sparse
    /// regions. The priority chain then picks the single dominant type.
    ///
    /// Priority order:
    ///   1. Lace      - keyword only (filename/folder)
    ///   2. ITH       - keyword or whole-pattern geometry
    ///   3. Applique  - keyword only, or two geometrically-matching outline
    ///                  blocks (placement + tack-down)
    ///   4. Cross Stitch - diagonal X signature
    ///   5. Filled    - dense back-and-forth pattern
    ///   6. Satin     - dense long parallel stitches
    ///   7. Cutwork   - outline + satin + many trims
    ///   8. Outline   - running stitch over a sparse area (fallback)
    fn identify_stitches(&self) -> Vec<String> {
        // Fast keyword checks on the whole design (filename/folder).
        if self.name_confidence("lace") >= 0.99 {
            return vec!["lace".to_string()];
        }
        if self.name_confidence("ith") >= 0.99 || self.detect_ith() >= self.confidence_threshold {
            return vec!["ith".to_string()];
        }
        if self.name_confidence("applique") >= 0.99 {
            return vec!["applique".to_string()];
        }
        if self.name_confidence("cross_stitch") >= 0.99 {
            return vec!["cross_stitch".to_string()];
        }

        // Whole-pattern priority: if one detector clearly dominates on the
        // whole design, it wins immediately. Multi-colour fill designs whose
        // individual colour blocks are too fragmented to pass the threshold
        // on their own (e.g. long-row teapot house fills) still read as one
        // filled design from the aggregate.
        //
        // The fill/satin/outline detectors are computed on a FLATTENED,
        // single-colour view of the design — stitch density does not depend
        // on how many colours a region uses, so a multi-colour filled design
        // should still be read as one filled region rather than being
        // fragmented into per-colour blocks. The per-block fallback below
        // still uses the original colour-split blocks and applique geometry
        // still needs the separate colour layers.
        let flattened = flatten_colors(self.pattern);
        let flattened_identifier = StitchIdentifier::new(
            &flattened,
            &self.filename,
            &self.folder_name,
            self.confidence_threshold,
        );
        let whole_filled = flattened_identifier.detect_filled(false);
        let whole_satin = flattened_identifier.detect_satin(false);
        let whole_outline = flattened_identifier.detect_outline();

        let filled_effective_threshold = (self.confidence_threshold - 0.05).max(0.60);
        let filled_confident = whole_filled >= filled_effective_threshold;

        // Applique is detected by two geometrically-matching outline blocks
        // (placement + tack-down). This must run BEFORE the whole-pattern
        // priority gate - a genuine applique's placement/tack-down outlines
        // are sparse and would otherwise be caught by the outline detector.
        // BUT only consider it when the design is NOT confidently filled:
        // in a dense fill, matching sparse blocks are just interior elements
        // (windows, details), not applique layers.
        if !filled_confident && self.detect_applique_from_block_geometry() {
            return vec!["applique".to_string()];
        }

        let best = [
            (whole_filled, "filled"),
            (whole_satin, "satin"),
            (whole_outline, "outline"),
        ]
        .into_iter()
        .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((score, stitch_type)) = best {
            // Whole-pattern filled systematically under-scores fragmented
            // multi-colour fills (sparse interior holes dilute the density),
            // so allow a small margin below the threshold when filled is the
            // clearly-dominant detector. Satin/outline keep the strict bar.
            let effective_threshold = if stitch_type == "filled" {
                filled_effective_threshold
            } else {
                self.confidence_threshold
            };
            if score >= effective_threshold {
                return vec![stitch_type.to_string()];
            }
        }

        // Collect which stitch types pass the threshold in ANY colour block.
        // Note: applique is NOT included here - its whole-pattern heuristic
        // false-positives on multi-colour dense designs (the path-repeat
        // overlap proxy scores any multi-block design highly).
        let mut collected: HashSet<String> = HashSet::new();

        for block in split_into_color_blocks(self.pattern) {
            let stitch_count = block
                .iter()
                .filter(|s| s.stitch_type == StitchType::Stitch)
                .count();
            if stitch_count < 6 {
                continue;
            }

            let mut block_pattern = EmbPattern::new();
            block_pattern.stitches = block.clone();

            let block_identifier = StitchIdentifier::new(
                &block_pattern,
                &self.filename,
                &self.folder_name,
                self.confidence_threshold,
            );

            let block_scores = block_identifier.get_detailed_analysis();
            for &stitch_type in &["cross_stitch", "cutwork", "filled", "outline", "satin"] {
                if block_scores.get(stitch_type).copied().unwrap_or(0.0)
                    >= self.confidence_threshold
                {
                    collected.insert(stitch_type.to_string());
                }
            }
        }

        // Priority chain over the collected types - first match wins alone.
        for &stitch_type in &["cross_stitch", "filled", "satin", "cutwork", "outline"] {
            if collected.contains(stitch_type) {
                return vec![stitch_type.to_string()];
            }
        }

        // Nothing conclusive - leave the design untagged.
        Vec::new()
    }

    /// Detects an applique/ITH design where two (or more) colour blocks are
    /// near-identical outline runs (placement + tack-down) layered on top of
    /// each other. This is a separate signal from the whole-pattern
    /// `detect_applique()` heuristic, which relies on path repetition.
    fn detect_applique_from_block_geometry(&self) -> bool {
        let block_stitches = split_into_color_blocks(self.pattern);
        let mut blocks = Vec::new();

        for stitches in block_stitches {
            let stitch_count = stitches
                .iter()
                .filter(|s| s.stitch_type == StitchType::Stitch)
                .count();
            if stitch_count < 6 {
                continue;
            }

            let mut block_pattern = EmbPattern::new();
            block_pattern.stitches = stitches.clone();

            let block_identifier = StitchIdentifier::new(
                &block_pattern,
                &self.filename,
                &self.folder_name,
                self.confidence_threshold,
            );

            let block_scores = block_identifier.get_detailed_analysis();
            blocks.push((stitches, block_scores));
        }

        // Require at least TWO outline-like blocks (placement + tack-down)
        // that match geometrically.
        for idx_a in 0..blocks.len() {
            for idx_b in (idx_a + 1)..blocks.len() {
                let (stitches_a, scores_a) = &blocks[idx_a];
                let (stitches_b, scores_b) = &blocks[idx_b];

                if geometry_matches(stitches_a, stitches_b) {
                    let outline_a = scores_a.get("outline").copied().unwrap_or(0.0);
                    let outline_b = scores_b.get("outline").copied().unwrap_or(0.0);
                    if outline_a >= self.confidence_threshold
                        && outline_b >= self.confidence_threshold
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn get_detailed_analysis(&self) -> HashMap<&'static str, f64> {
        HashMap::from([
            ("cross_stitch", self.detect_cross_stitch()),
            ("ith", self.detect_ith()),
            ("applique", self.detect_applique()),
            ("filled", self.detect_filled(false)),
            ("cutwork", self.detect_cutwork()),
            ("lace", self.detect_lace()),
            ("outline", self.detect_outline()),
            ("satin", self.detect_satin(false)),
        ])
    }

    fn detect_cross_stitch(&self) -> f64 {
        let name_conf = self.name_confidence("cross_stitch");
        if name_conf >= 0.99 {
            return name_conf;
        }
        if self.vectors.is_empty() {
            return name_conf;
        }

        let mut slash = 0usize;
        let mut backslash = 0usize;
        let mut diagonal = 0usize;
        let mut orthogonal = 0usize;

        for v in &self.vectors {
            if v.length < 0.1 {
                continue;
            }
            let angle = v.angle;
            if angle_close(angle, 45.0, 20.0) || angle_close(angle, 225.0, 20.0) {
                slash += 1;
                diagonal += 1;
            } else if angle_close(angle, 135.0, 20.0) || angle_close(angle, 315.0, 20.0) {
                backslash += 1;
                diagonal += 1;
            } else if angle_close(angle, 0.0, 20.0)
                || angle_close(angle, 90.0, 20.0)
                || angle_close(angle, 180.0, 20.0)
                || angle_close(angle, 270.0, 20.0)
            {
                orthogonal += 1;
            }
        }

        if diagonal == 0 {
            return 0.0;
        }

        // Cross-stitch is made of short individual legs (typically 1-5
        // stitching units). A 45-degree serpentine FILL pattern also produces
        // balanced slash/backslash angles, but its row stitches are far
        // longer. Gate on average stitch length so long-diagonal fills are
        // not mistaken for cross-stitch.
        let lengths: Vec<f64> = self.vectors.iter().map(|v| v.length).collect();
        let mean_len = lengths.iter().sum::<f64>() / (lengths.len() as f64);
        if mean_len > 8.0 {
            return name_conf;
        }

        let balance = (slash.min(backslash) as f64) / ((slash.max(backslash)).max(1) as f64);
        let diagonal_ratio = (diagonal as f64) / (self.vectors.len() as f64);
        let cross_purity = (diagonal as f64) / ((diagonal + orthogonal).max(1) as f64);

        let variance = lengths
            .iter()
            .map(|length| (length - mean_len) * (length - mean_len))
            .sum::<f64>()
            / (lengths.len() as f64);
        let std_len = variance.sqrt();
        let cv = std_len / mean_len.max(0.1);
        let uniformity = (1.0 - cv.min(1.0)).max(0.0);

        let base =
            (0.35 * balance + 0.30 * diagonal_ratio + 0.20 * uniformity + 0.15 * cross_purity)
                .min(1.0);
        base.max(name_conf)
    }

    fn detect_ith(&self) -> f64 {
        let name_conf = self.name_confidence("ith");
        if name_conf >= 0.99 {
            return name_conf;
        }
        if self.vectors.is_empty() {
            return 0.0;
        }

        let path_repeat = self.path_repeat_score();
        if path_repeat < 0.16 {
            return 0.0;
        }

        let trims = self.pattern.count_stitch_commands(StitchType::Trim) as f64;
        let jumps = self.pattern.count_stitch_commands(StitchType::Jump) as f64;
        let assembly_activity =
            ((trims + jumps) / ((self.vectors.len() as f64 / 8.0) + 1.0)).min(1.0);

        let overlap_score = self.color_block_overlap_proxy();
        let running_score = self.running_like_score();
        let satin_score = self.detect_satin(false);

        (0.28 * overlap_score
            + 0.20 * running_score
            + 0.22 * satin_score
            + 0.15 * assembly_activity
            + 0.15 * path_repeat)
            .min(1.0)
    }

    fn detect_applique(&self) -> f64 {
        let name_conf = self.name_confidence("applique");
        if name_conf >= 0.99 {
            return name_conf;
        }
        if self.vectors.is_empty() {
            return 0.0;
        }

        let satin_score = self.detect_satin(false);
        let path_repeat = self.path_repeat_score();
        if path_repeat >= 0.2 {
            let overlap = self.color_block_overlap_proxy();
            let running = self.running_like_score();
            return (0.40 * path_repeat + 0.25 * overlap + 0.20 * satin_score + 0.15 * running)
                .min(1.0);
        }

        0.0
    }

    fn detect_filled(&self, _no_cross: bool) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let density = self.stitch_density_score();
        let outline = self.detect_outline();
        let mut base = self.detect_filled_like_score();

        if density >= 0.41 && outline <= 0.38 {
            base = base.max(0.72);
        }

        if self.pattern.count_color_changes() == 0 && density >= 0.29 && outline < 0.58 {
            let satin_score = self.detect_satin_like_score();
            if satin_score < 0.55 {
                base = base.max((0.62 + 0.30 * density).min(1.0));
            }
        }

        if self.pattern.count_color_changes() == 0 && (0.20..=0.40).contains(&density) {
            let satin_score = self.detect_satin_like_score();
            let axis_ratio = self.geometric_angle_score();
            let turns = self.direction_change_score();
            if (0.62..=0.75).contains(&satin_score) && axis_ratio >= 0.93 && turns <= 0.40 {
                base = base.max(0.72);
            }
        }

        base
    }

    fn detect_cutwork(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let outline = self.detect_outline();
        let satin = self.detect_satin(false);
        let trims = self.pattern.count_stitch_commands(StitchType::Trim) as f64;
        let trim_score = (trims / ((self.vectors.len() as f64 / 12.0) + 1.0)).min(1.0);

        (0.35 * outline + 0.45 * satin + 0.2 * trim_score).min(1.0)
    }

    fn detect_lace(&self) -> f64 {
        let name_conf = self.name_confidence("lace");
        if name_conf >= 0.99 {
            return name_conf;
        }
        0.0
    }

    fn detect_outline(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let running = self.running_like_score();
        let density = self.stitch_density_score();
        let satin = self.detect_satin_like_score();
        let fill = self.detect_filled_like_score();
        (0.8 * running + 0.2 * (1.0 - density) - 0.25 * satin - 0.2 * fill).clamp(0.0, 1.0)
    }

    fn detect_satin(&self, _no_cross: bool) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let mut score = self.detect_satin_like_score();
        let density = self.stitch_density_score();
        // Satin is a densely-worked stitch. Sparse running/perimeter designs
        // (e.g. a logical outline) share the long, axis-aligned stitch
        // signature but are NOT satin, so require a minimum density.
        if density < 0.20 {
            return 0.0;
        }
        // Satin's defining signature is the ZIGZAG: the leg direction flips
        // between the two column edges on essentially every stitch, so the
        // consecutive direction-change rate is very high. Long-row
        // serpentine fills keep a constant direction within each row and
        // only reverse at row ends, so they show a LOW change rate. Gate on
        // the zigzag signature, NOT stitch length - real satin legs scale
        // with the column width and routinely exceed the old 20-unit cap
        // (e.g. wide satin motifs in the "Patterns" collections).
        let turns = self.direction_change_score();
        if turns < 0.50 {
            return 0.0;
        }
        // Coarse sanity gate: legs far longer than any satin column are
        // running jumps / outlines, not satin.
        let lengths: Vec<f64> = self.vectors.iter().map(|v| v.length).collect();
        let mean_len = lengths.iter().sum::<f64>() / (lengths.len().max(1) as f64);
        if mean_len > 80.0 {
            return 0.0;
        }
        // A strong zigzag (turns >= 0.60) is the most reliable satin signal.
        // The generic satin-likeness score can under-rate diagonal columns
        // (their legs avoid the 8 anchor angles), so give a direct boost
        // that scales with how pure the zigzag is.
        if turns >= 0.60 {
            score = score.max(0.70 + 0.40 * (turns - 0.60));
        }

        score
    }

    fn name_confidence(&self, stitch_type: &str) -> f64 {
        let keywords: &[&str] = match stitch_type {
            "ith" => &["in the hoop", "ith", "hoop"],
            "applique" => &["applique", "appliquee", "appliquÃ©", "appique"],
            "cross_stitch" => &["cross stitch", "cross-stitch", "cross_stitch"],
            "lace" => &["lace", "fsl", "freestanding lace", "free standing lace"],
            _ => &[],
        };

        for keyword in keywords {
            if self.name_text.contains(keyword) {
                return 0.99;
            }
        }

        0.0
    }

    fn color_blocks_count(&self) -> usize {
        self.pattern
            .count_color_changes()
            .saturating_add(1)
            .max(self.pattern.count_threads().max(1))
    }

    fn color_block_overlap_proxy(&self) -> f64 {
        if self.color_blocks_count() < 2 {
            return 0.0;
        }
        // Approximation without explicit pyembroidery-style color blocks in Rust.
        // Designs with multiple color blocks and repeated paths are likely overlapping.
        (self.path_repeat_score() * 1.2).min(1.0)
    }

    fn running_like_score(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let lengths: Vec<f64> = self.vectors.iter().map(|v| v.length).collect();
        let avg_length = lengths.iter().sum::<f64>() / (lengths.len() as f64);
        let short_ratio = lengths
            .iter()
            .filter(|length| **length <= avg_length * 1.35)
            .count() as f64
            / (lengths.len() as f64);
        short_ratio.min(1.0)
    }

    fn stitch_density_score(&self) -> f64 {
        let stitch_count = self.vectors.len();
        if stitch_count == 0 {
            return 0.0;
        }

        let (min_x, min_y, max_x, max_y) = stitch_bounds(self.pattern);
        let width = (max_x - min_x).max(1.0);
        let height = (max_y - min_y).max(1.0);
        let area = width * height;
        if area <= 0.0 {
            return 0.0;
        }

        let density = stitch_count as f64 / area;
        (density * 50.0).min(1.0)
    }

    fn detect_satin_like_score(&self) -> f64 {
        if self.vectors.len() < 6 {
            return 0.0;
        }

        let lengths: Vec<f64> = self.vectors.iter().map(|v| v.length).collect();
        let avg_len = lengths.iter().sum::<f64>() / (lengths.len() as f64);
        let long_ratio = lengths.iter().filter(|length| **length >= avg_len).count() as f64
            / (lengths.len() as f64);
        let axis_ratio = self.geometric_angle_score();
        let turns = self.direction_change_score();
        (0.45 * long_ratio + 0.35 * axis_ratio + 0.20 * turns).min(1.0)
    }

    fn detect_filled_like_score(&self) -> f64 {
        (0.6 * self.stitch_density_score() + 0.4 * self.direction_change_score()).min(1.0)
    }

    fn direction_change_score(&self) -> f64 {
        if self.vectors.len() < 3 {
            return 0.0;
        }

        let mut changes = 0usize;
        let mut total = 0usize;
        let mut last_angle = self.vectors[0].angle;

        for vector in self.vectors.iter().skip(1) {
            total += 1;
            if angle_diff(last_angle, vector.angle) > 45.0 {
                changes += 1;
            }
            last_angle = vector.angle;
        }

        if total == 0 {
            return 0.0;
        }

        changes as f64 / total as f64
    }

    fn geometric_angle_score(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }

        let anchors = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];
        let mut matches = 0usize;

        for vector in &self.vectors {
            if anchors
                .iter()
                .any(|anchor| angle_close(vector.angle, *anchor, 16.0))
            {
                matches += 1;
            }
        }

        matches as f64 / self.vectors.len() as f64
    }

    fn path_repeat_score(&self) -> f64 {
        let mut points = Vec::new();
        for stitch in &self.pattern.stitches {
            if stitch.stitch_type == StitchType::Stitch {
                points.push((round_tenth(stitch.x as f64), round_tenth(stitch.y as f64)));
            }
        }

        if points.len() < 6 {
            return 0.0;
        }

        let mut freq: HashMap<(i64, i64), usize> = HashMap::new();
        for point in points {
            *freq.entry(point).or_insert(0) += 1;
        }

        let repeated = freq.values().filter(|count| **count > 1).count();
        (repeated as f64 / (freq.len().max(1) as f64)).min(1.0)
    }
}

fn round_tenth(value: f64) -> i64 {
    (value * 10.0).round() as i64
}

fn build_vectors(pattern: &EmbPattern) -> Vec<Vector> {
    let mut vectors = Vec::new();
    let mut prev: Option<(f64, f64)> = None;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            prev = None;
            continue;
        }

        let current = (stitch.x as f64, stitch.y as f64);
        if let Some((prev_x, prev_y)) = prev {
            let dx = current.0 - prev_x;
            let dy = current.1 - prev_y;
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                let mut angle = dy.atan2(dx) * (180.0 / PI);
                if angle < 0.0 {
                    angle += 360.0;
                }
                vectors.push(Vector { length, angle });
            }
        }

        prev = Some(current);
    }

    vectors
}

fn stitch_bounds(pattern: &EmbPattern) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            continue;
        }
        let x = stitch.x as f64;
        let y = stitch.y as f64;
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
    }

    if !min_x.is_finite() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    (min_x, min_y, max_x, max_y)
}

/// Returns a copy of the pattern with every `ColorChange` stitch converted
/// to a `Jump`.
///
/// Fill/satin/outline density does not depend on the number of colours — a
/// multi-colour fill is still one filled region. Analysing it on this
/// colour-flattened view lets the density-based detectors read the design as
/// a whole instead of fragmenting it into per-colour blocks that
/// individually can't prove themselves.
///
/// `ColorChange` is REPLACED (not removed) so that `build_vectors` still
/// treats the boundary as a needle move and does NOT synthesize a long
/// stitch between disjoint colour regions (which would corrupt the geometry
/// detectors and the running/density scoring).
fn flatten_colors(pattern: &EmbPattern) -> EmbPattern {
    let mut flattened = pattern.clone();
    for stitch in &mut flattened.stitches {
        if stitch.stitch_type == StitchType::ColorChange {
            stitch.stitch_type = StitchType::Jump;
        }
    }
    flattened
}

fn split_into_color_blocks(pattern: &EmbPattern) -> Vec<Vec<Stitch>> {
    let mut blocks = Vec::new();
    let mut current_block = Vec::new();

    for stitch in &pattern.stitches {
        current_block.push(*stitch);
        if stitch.stitch_type == StitchType::ColorChange {
            if !current_block.is_empty() {
                blocks.push(current_block);
                current_block = Vec::new();
            }
        }
    }
    if !current_block.is_empty() {
        blocks.push(current_block);
    }
    blocks
}

fn block_bounds(stitches: &[Stitch]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stitch in stitches {
        if stitch.stitch_type != StitchType::Stitch {
            continue;
        }
        let x = stitch.x as f64;
        let y = stitch.y as f64;
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
    }

    if !min_x.is_finite() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    (min_x, min_y, max_x, max_y)
}

fn geometry_matches(block_a: &[Stitch], block_b: &[Stitch]) -> bool {
    let (min_xa, min_ya, max_xa, max_ya) = block_bounds(block_a);
    let (min_xb, min_yb, max_xb, max_yb) = block_bounds(block_b);

    let wa = max_xa - min_xa;
    let ha = max_ya - min_ya;
    let wb = max_xb - min_xb;
    let hb = max_yb - min_yb;

    if wa <= 1.0 || ha <= 1.0 || wb <= 1.0 || hb <= 1.0 {
        return false;
    }

    let center_xa = (min_xa + max_xa) / 2.0;
    let center_ya = (min_ya + max_ya) / 2.0;
    let center_xb = (min_xb + max_xb) / 2.0;
    let center_yb = (min_yb + max_yb) / 2.0;

    let w_diff = (wa - wb).abs();
    let h_diff = (ha - hb).abs();
    let cx_diff = (center_xa - center_xb).abs();
    let cy_diff = (center_ya - center_yb).abs();

    let max_w = wa.max(wb);
    let max_h = ha.max(hb);
    let max_dim = max_w.max(max_h);

    let size_tol = 2.5;
    let center_tol = 2.0;

    let w_match = w_diff <= size_tol || (w_diff / max_w) <= 0.10;
    let h_match = h_diff <= size_tol || (h_diff / max_h) <= 0.10;
    let cx_match = cx_diff <= center_tol || (cx_diff / max_dim) <= 0.08;
    let cy_match = cy_diff <= center_tol || (cy_diff / max_dim) <= 0.08;

    w_match && h_match && cx_match && cy_match
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = (a - b).abs() % 360.0;
    if d > 180.0 {
        d = 360.0 - d;
    }
    d
}

fn angle_close(a: f64, b: f64, tolerance: f64) -> bool {
    angle_diff(a, b) <= tolerance
}
#[cfg(test)]
#[path = "stitch_identifier_tests.rs"]
mod tests;
