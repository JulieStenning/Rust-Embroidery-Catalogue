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

    fn identify_stitches(&self) -> Vec<String> {
        let mut found = Vec::new();

        // 1. Quick win: Prioritize metadata keywords.
        for &stitch_type in &["ith", "applique", "cross_stitch", "lace"] {
            if self.name_confidence(stitch_type) >= 0.99 {
                found.push(stitch_type.to_string());
            }
        }

        // 2. Split pattern into color blocks
        let block_stitches = split_into_color_blocks(self.pattern);
        let mut block_analyses = Vec::new();

        for (i, stitches) in block_stitches.iter().enumerate() {
            let stitch_count = stitches
                .iter()
                .filter(|s| s.stitch_type == StitchType::Stitch)
                .count();
            if stitch_count < 6 {
                continue;
            }

            // Create a temporary EmbPattern for this block
            let mut block_pattern = EmbPattern::new();
            block_pattern.stitches = stitches.clone();

            let block_identifier = StitchIdentifier::new(
                &block_pattern,
                &self.filename,
                &self.folder_name,
                self.confidence_threshold,
            );

            // Run block-level analysis
            let block_scores = block_identifier.get_detailed_analysis();
            block_analyses.push((i, stitches.clone(), block_scores.clone(), stitch_count));

            // Now split the block into islands based on jumps/trims/stops
            let islands = split_block_into_islands(stitches);
            let mut block_tags = Vec::new();

            if islands.len() <= 1 {
                let mut tags = Vec::new();
                for &stitch_type in &[
                    "cross_stitch",
                    "cutwork",
                    "filled",
                    "lace",
                    "outline",
                    "satin",
                ] {
                    if block_scores.get(stitch_type).copied().unwrap_or(0.0)
                        >= self.confidence_threshold
                    {
                        tags.push(stitch_type.to_string());
                    }
                }
                apply_precedence_rules(&mut tags, &block_scores, self.confidence_threshold);
                block_tags = tags;
            } else {
                for island_stitches in islands {
                    let island_stitch_count = island_stitches
                        .iter()
                        .filter(|s| s.stitch_type == StitchType::Stitch)
                        .count();
                    if island_stitch_count < 6 {
                        continue;
                    }

                    let mut island_pattern = EmbPattern::new();
                    island_pattern.stitches = island_stitches;

                    let island_identifier = StitchIdentifier::new(
                        &island_pattern,
                        &self.filename,
                        &self.folder_name,
                        self.confidence_threshold,
                    );

                    let island_scores = island_identifier.get_detailed_analysis();
                    let mut island_tags = Vec::new();
                    for &stitch_type in &[
                        "cross_stitch",
                        "cutwork",
                        "filled",
                        "lace",
                        "outline",
                        "satin",
                    ] {
                        if island_scores.get(stitch_type).copied().unwrap_or(0.0)
                            >= self.confidence_threshold
                        {
                            island_tags.push(stitch_type.to_string());
                        }
                    }
                    apply_precedence_rules(
                        &mut island_tags,
                        &island_scores,
                        self.confidence_threshold,
                    );
                    block_tags.extend(island_tags);
                }
            }

            found.extend(block_tags);
        }

        // 3. Run Applique/ITH geometry-matching detection between all pairs of blocks
        // We require at least TWO outline-like blocks (placement + tack-down) that match geometrically.
        let mut has_applique = false;
        for idx_a in 0..block_analyses.len() {
            for idx_b in (idx_a + 1)..block_analyses.len() {
                let (_, stitches_a, scores_a, _) = &block_analyses[idx_a];
                let (_, stitches_b, scores_b, _) = &block_analyses[idx_b];

                if geometry_matches(stitches_a, stitches_b) {
                    let outline_a = scores_a.get("outline").copied().unwrap_or(0.0);
                    let outline_b = scores_b.get("outline").copied().unwrap_or(0.0);
                    if outline_a >= self.confidence_threshold
                        && outline_b >= self.confidence_threshold
                    {
                        has_applique = true;
                        break;
                    }
                }
            }
            if has_applique {
                break;
            }
        }

        if has_applique {
            if !found.contains(&"applique".to_string()) {
                found.push("applique".to_string());
            }
            if !found.contains(&"ith".to_string()) {
                found.push("ith".to_string());
            }
        }
        // --- New Step: Demote Outlines that accent a Fill/Satin block ---
        let mut has_substantial_fill_or_satin = false;
        let mut outline_block_indices = Vec::new();
        let mut filled_satin_block_indices = Vec::new();

        // Identify which blocks contributed what scores
        for (idx, _, scores, _) in &block_analyses {
            let satin = scores.get("satin").copied().unwrap_or(0.0);
            let filled = scores.get("filled").copied().unwrap_or(0.0);
            let outline = scores.get("outline").copied().unwrap_or(0.0);

            if (satin >= self.confidence_threshold || filled >= self.confidence_threshold)
                && (satin > outline && filled > outline)
            {
                filled_satin_block_indices.push(idx);
                has_substantial_fill_or_satin = true;
            }
            if outline >= self.confidence_threshold {
                outline_block_indices.push(idx);
            }
        }

        // If a design contains BOTH a heavy fill/satin and an outline, check their geometry
        if has_substantial_fill_or_satin && !outline_block_indices.is_empty() {
            let mut remove_outline_tag = false;

            for &out_idx in &outline_block_indices {
                for &fill_idx in &filled_satin_block_indices {
                    let (_, stitches_out, _, _) = &block_analyses[*out_idx];
                    let (_, stitches_fill, _, _) = &block_analyses[*fill_idx];

                    // If the outline block's geometry tightly matches or frames the fill block,
                    // it's an accent line, not a standalone "Line Outline" design.
                    if geometry_matches(stitches_out, stitches_fill) {
                        remove_outline_tag = true;
                        break;
                    }
                }
            }

            if remove_outline_tag {
                // Only strip the "outline" tag if it isn't an applique/ITH file
                if !found.contains(&"applique".to_string()) {
                    found.retain(|name| name != "outline");
                }
            }
        }
        // Apply global cleanup filters
        if found.contains(&"lace".to_string()) {
            found.retain(|name| name != "filled");
        }

        if found.contains(&"cross_stitch".to_string()) {
            found.retain(|name| !matches!(name.as_str(), "applique" | "filled" | "satin"));
        }

        if found.contains(&"applique".to_string()) {
            found.retain(|name| !matches!(name.as_str(), "satin" | "outline"));
        }

        found.sort();
        found.dedup();
        found
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

        let balance = (slash.min(backslash) as f64) / ((slash.max(backslash)).max(1) as f64);
        let diagonal_ratio = (diagonal as f64) / (self.vectors.len() as f64);
        let cross_purity = (diagonal as f64) / ((diagonal + orthogonal).max(1) as f64);

        let lengths: Vec<f64> = self.vectors.iter().map(|v| v.length).collect();
        let mean_len = lengths.iter().sum::<f64>() / (lengths.len() as f64);
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

        if self.color_blocks_count() == 1 && density >= 0.29 && outline < 0.58 {
            let satin_score = self.detect_satin_like_score();
            if satin_score < 0.55 {
                base = base.max((0.62 + 0.30 * density).min(1.0));
            }
        }

        if self.color_blocks_count() == 1 && (0.20..=0.40).contains(&density) {
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
        let axis_ratio = self.geometric_angle_score();
        let turns = self.direction_change_score();
        let filled = self.detect_filled_like_score();
        let outline = self.detect_outline();

        if self.color_blocks_count() == 1
            && (0.20..=0.40).contains(&density)
            && axis_ratio >= 0.93
            && turns <= 0.40
        {
            score *= 0.78;
        }

        if self.color_blocks_count() > 1
            && (0.20..=0.80).contains(&density)
            && axis_ratio >= 0.80
            && turns <= 0.25
            && filled >= 0.65
            && outline < 0.70
        {
            score *= 0.78;
        }

        score
    }

    fn name_confidence(&self, stitch_type: &str) -> f64 {
        let keywords: &[&str] = match stitch_type {
            "ith" => &["in the hoop", "ith", "hoop"],
            "applique" => &["applique", "appliquee", "appliqué", "appique"],
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

fn split_block_into_islands(stitches: &[Stitch]) -> Vec<Vec<Stitch>> {
    let mut islands = Vec::new();
    let mut current_island = Vec::new();

    for stitch in stitches {
        if stitch.stitch_type == StitchType::Jump
            || stitch.stitch_type == StitchType::Trim
            || stitch.stitch_type == StitchType::Stop
        {
            if !current_island.is_empty() {
                islands.push(current_island);
                current_island = Vec::new();
            }
        } else if stitch.stitch_type == StitchType::Stitch {
            current_island.push(*stitch);
        }
    }
    if !current_island.is_empty() {
        islands.push(current_island);
    }
    islands
}

fn apply_precedence_rules(
    tags: &mut Vec<String>,
    scores: &HashMap<&'static str, f64>,
    confidence_threshold: f64,
) {
    let satin_precedence_threshold = (confidence_threshold - 0.07).max(0.63);
    let satin_score = scores.get("satin").copied().unwrap_or(0.0);
    let outline_score = scores.get("outline").copied().unwrap_or(0.0);
    if satin_score >= satin_precedence_threshold
        && !tags.contains(&"lace".to_string())
        && outline_score < 0.78
    {
        if !tags.contains(&"satin".to_string()) {
            tags.push("satin".to_string());
        }
        tags.retain(|name| name != "outline");
    }

    if tags.is_empty()
        && satin_score >= (confidence_threshold - 0.12).max(0.58)
        && outline_score < 0.60
    {
        tags.push("satin".to_string());
    }

    if tags.is_empty()
        && outline_score >= 0.48
        && satin_score < 0.58
        && confidence_threshold <= 0.75
    {
        tags.push("outline".to_string());
    }
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
mod tests {
    use super::*;
    use crate::models::{EmbPattern, Stitch};

    fn filled_pattern() -> EmbPattern {
        let mut pattern = EmbPattern::new();

        // Dense meandering fill-like block.
        let mut x = 0.0_f32;
        let mut y = 0.0_f32;
        for row in 0..20 {
            for step in 0..40 {
                x = step as f32;
                pattern.stitches.push(Stitch {
                    x,
                    y,
                    stitch_type: StitchType::Stitch,
                });
            }
            y += 0.7;
            pattern.stitches.push(Stitch {
                x,
                y,
                stitch_type: StitchType::Stitch,
            });
            if row % 2 == 0 {
                for step in (0..40).rev() {
                    x = step as f32;
                    pattern.stitches.push(Stitch {
                        x,
                        y,
                        stitch_type: StitchType::Stitch,
                    });
                }
            }
        }

        pattern
    }

    fn outline_pattern() -> EmbPattern {
        let mut pattern = EmbPattern::new();
        // Sparse perimeter over a large area should score as outline/running.
        for index in 0..80 {
            pattern.stitches.push(Stitch {
                x: index as f32 * 12.0,
                y: 0.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for index in 0..80 {
            pattern.stitches.push(Stitch {
                x: 79.0 * 12.0,
                y: index as f32 * 12.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for index in (0..80).rev() {
            pattern.stitches.push(Stitch {
                x: index as f32 * 12.0,
                y: 79.0 * 12.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for index in (0..80).rev() {
            pattern.stitches.push(Stitch {
                x: 0.0,
                y: index as f32 * 12.0,
                stitch_type: StitchType::Stitch,
            });
        }
        pattern
    }

    #[test]
    fn identifies_filled_for_dense_pattern() {
        let pattern = filled_pattern();
        let valid = HashSet::from([
            "Filled".to_string(),
            "Line Outline".to_string(),
            "Satin Stitch".to_string(),
        ]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "dense-fill.pes",
            "C:/imports/filled/dense-fill.pes",
            &valid,
            Some(0.70),
        );

        assert!(tags.contains(&"Filled".to_string()));
    }

    #[test]
    fn identifies_outline_for_sparse_lines() {
        let pattern = outline_pattern();
        let valid = HashSet::from([
            "Filled".to_string(),
            "Line Outline".to_string(),
            "Satin Stitch".to_string(),
        ]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "outline.pes",
            "C:/imports/outline/outline.pes",
            &valid,
            Some(0.70),
        );

        assert!(tags.contains(&"Line Outline".to_string()));
    }

    #[test]
    fn identifies_metadata_priority_keyword() {
        // Even if the pattern is empty, name metadata "some_fsl_design.pes" should identify it as Lace.
        let pattern = EmbPattern::new();
        let valid = HashSet::from(["Lace".to_string(), "In The Hoop".to_string()]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "some_fsl_design.pes",
            "C:/imports/lace/some_fsl_design.pes",
            &valid,
            Some(0.70),
        );

        assert!(tags.contains(&"Lace".to_string()));
    }

    #[test]
    fn identifies_multi_block_mixed_types() {
        let mut pattern = outline_pattern();
        // Color change
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::ColorChange,
        });
        // Block 2: dense fill
        let mut y = 2000.0_f32;
        for _row in 0..15 {
            for step in 0..20 {
                let x = 2000.0 + step as f32 * 2.0;
                pattern.stitches.push(Stitch {
                    x,
                    y,
                    stitch_type: StitchType::Stitch,
                });
            }
            y += 1.0;
        }

        let valid = HashSet::from(["Filled".to_string(), "Line Outline".to_string()]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "mixed.pes",
            "C:/imports/mixed/mixed.pes",
            &valid,
            Some(0.70),
        );

        assert!(tags.contains(&"Line Outline".to_string()));
        assert!(tags.contains(&"Filled".to_string()));
    }

    #[test]
    fn identifies_applique_geometric_matching() {
        let mut pattern = EmbPattern::new();
        // Block 1: outline square (placement)
        for i in 0..=5 {
            pattern.stitches.push(Stitch {
                x: i as f32 * 20.0,
                y: 0.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 100.0,
                y: i as f32 * 20.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 100.0 - i as f32 * 20.0,
                y: 100.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..5 {
            pattern.stitches.push(Stitch {
                x: 0.0,
                y: 100.0 - i as f32 * 20.0,
                stitch_type: StitchType::Stitch,
            });
        }
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::ColorChange,
        });
        // Block 2: identical square (tackdown) but slightly offset
        for i in 0..=5 {
            pattern.stitches.push(Stitch {
                x: i as f32 * 20.0 + 1.0,
                y: 1.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 101.0,
                y: i as f32 * 20.0 + 1.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 101.0 - i as f32 * 20.0,
                y: 101.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..5 {
            pattern.stitches.push(Stitch {
                x: 1.0,
                y: 101.0 - i as f32 * 20.0,
                stitch_type: StitchType::Stitch,
            });
        }

        let valid = HashSet::from(["Applique".to_string(), "In The Hoop".to_string()]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "test_app.pes",
            "C:/imports/test_app.pes",
            &valid,
            Some(0.70),
        );

        assert!(tags.contains(&"Applique".to_string()));
        assert!(tags.contains(&"In The Hoop".to_string()));
    }

    #[test]
    fn does_not_identify_applique_for_single_outline() {
        let mut pattern = EmbPattern::new();
        // Block 1: dense fill block
        let mut y = 0.0_f32;
        for _row in 0..15 {
            for step in 0..20 {
                let x = step as f32 * 2.0;
                pattern.stitches.push(Stitch {
                    x,
                    y,
                    stitch_type: StitchType::Stitch,
                });
            }
            y += 1.0;
        }
        // Color change
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::ColorChange,
        });
        // Block 2: outline matching geometrically
        for i in 0..=5 {
            pattern.stitches.push(Stitch {
                x: i as f32 * 7.6,
                y: 0.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 38.0,
                y: i as f32 * 2.8,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..=5 {
            pattern.stitches.push(Stitch {
                x: 38.0 - i as f32 * 7.6,
                y: 14.0,
                stitch_type: StitchType::Stitch,
            });
        }
        for i in 1..5 {
            pattern.stitches.push(Stitch {
                x: 0.0,
                y: 14.0 - i as f32 * 2.8,
                stitch_type: StitchType::Stitch,
            });
        }

        let valid = HashSet::from(["Applique".to_string(), "In The Hoop".to_string()]);

        let tags = suggest_stitching_from_pattern(
            &pattern,
            "regular.pes",
            "C:/imports/regular/regular.pes",
            &valid,
            Some(0.70),
        );

        assert!(!tags.contains(&"Applique".to_string()));
        assert!(!tags.contains(&"In The Hoop".to_string()));
    }
}
