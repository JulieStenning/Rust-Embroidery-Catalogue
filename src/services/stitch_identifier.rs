use crate::models::{EmbPattern, StitchType};
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
		}
	}

	fn identify_stitches(&self) -> Vec<String> {
		let scores = self.get_detailed_analysis();
		let stitch_types = [
			"applique",
			"cross_stitch",
			"cutwork",
			"filled",
			"ith",
			"lace",
			"outline",
			"satin",
		];

		let mut found = Vec::new();
		for stitch_type in stitch_types {
			if scores.get(stitch_type).copied().unwrap_or(0.0) >= self.confidence_threshold {
				found.push(stitch_type.to_string());
			}
		}

		let satin_precedence_threshold = (self.confidence_threshold - 0.07).max(0.63);
		let satin_score = scores.get("satin").copied().unwrap_or(0.0);
		let outline_score = scores.get("outline").copied().unwrap_or(0.0);
		if satin_score >= satin_precedence_threshold
			&& !found.contains(&"lace".to_string())
			&& outline_score < 0.78
		{
			if !found.contains(&"satin".to_string()) {
				found.push("satin".to_string());
			}
			found.retain(|name| name != "outline");
		}

		if found.contains(&"lace".to_string()) {
			found.retain(|name| name != "filled");
		}

		if found.contains(&"cross_stitch".to_string()) {
			found.retain(|name| !matches!(name.as_str(), "applique" | "filled" | "satin"));
		}

		if found.contains(&"applique".to_string()) {
			found.retain(|name| !matches!(name.as_str(), "satin" | "outline"));
		}

		if found.is_empty()
			&& satin_score >= (self.confidence_threshold - 0.12).max(0.58)
			&& outline_score < 0.60
		{
			found.push("satin".to_string());
		}

		if found.is_empty() && outline_score >= 0.48 && satin_score < 0.58 && self.confidence_threshold <= 0.75 {
			found.push("outline".to_string());
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

		let base = (0.35 * balance + 0.30 * diagonal_ratio + 0.20 * uniformity + 0.15 * cross_purity)
			.min(1.0);
		base.max(name_conf)
	}

	fn detect_ith(&self) -> f64 {
		let name_conf = self.name_confidence("ith");
		if name_conf > 0.0 {
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
		let assembly_activity = ((trims + jumps) / ((self.vectors.len() as f64 / 8.0) + 1.0)).min(1.0);

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
		if name_conf > 0.0 {
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
		if name_conf > 0.0 {
			return name_conf;
		}
		if self.vectors.is_empty() {
			return 0.0;
		}

		let satin = self.detect_satin_like_score();
		let running = self.running_like_score();
		let filled_like = self.detect_filled_like_score();
		let density = self.stitch_density_score();
		if satin < 0.55 || running < 0.65 || filled_like < 0.6 || density > 0.85 {
			return 0.0;
		}

		let color_blocks = self.color_blocks_count() as f64;
		let color_score = if color_blocks <= 2.0 {
			1.0
		} else {
			(1.0 - ((color_blocks - 2.0) / 4.0)).max(0.0)
		};
		let jumps = self.pattern.count_stitch_commands(StitchType::Jump) as f64;
		let trims = self.pattern.count_stitch_commands(StitchType::Trim) as f64;
		let continuity = 1.0 - ((jumps + trims) / ((self.vectors.len() as f64 / 25.0) + 1.0)).min(1.0);

		(0.30 * satin + 0.25 * running + 0.20 * filled_like + 0.15 * color_score + 0.10 * continuity)
			.min(1.0)
	}

	fn detect_outline(&self) -> f64 {
		if self.vectors.is_empty() {
			return 0.0;
		}

		let running = self.running_like_score();
		let density = self.stitch_density_score();
		let satin = self.detect_satin_like_score();
		let fill = self.detect_filled_like_score();
		(0.8 * running + 0.2 * (1.0 - density) - 0.25 * satin - 0.2 * fill)
			.clamp(0.0, 1.0)
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
				return 0.95;
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
}
