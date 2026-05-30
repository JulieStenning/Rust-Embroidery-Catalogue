use crate::services::image_generation::{generate_preview, ImageGenerationRequest};
use crate::services::stitch_identifier;
use crate::services::tagging;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

const TAG_ACTION_UNTAGGED: &str = "tag_untagged";
const TAG_ACTION_RETAG_ALL: &str = "retag_all";
const TAG_ACTION_RETAG_ALL_UNVERIFIED: &str = "retag_all_unverified";
const DEFAULT_DELAY_SECONDS: f64 = 5.0;
const DEFAULT_VISION_DELAY_SECONDS: f64 = 2.0;
const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_COMMIT_EVERY: i64 = 100;
const DEFAULT_WORKERS: i64 = 4;
const LOG_DIR: &str = "logs";
const ERROR_LOG_FILE: &str = "backfill_errors.log";
const INFO_LOG_FILE: &str = "backfill_info.log";

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedBackfillRequest {
	pub actions: Option<UnifiedBackfillActions>,
	pub batch_size: Option<i64>,
	pub commit_every: Option<i64>,
	pub workers: Option<i64>,
	pub preview_3d: Option<bool>,
	pub delay_seconds: Option<f64>,
	pub vision_delay_seconds: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedBackfillActions {
	pub tagging: Option<TaggingActionOptions>,
	pub stitching: Option<StitchingActionOptions>,
	pub images: Option<ImageActionOptions>,
	pub color_counts: Option<ColorCountsActionOptions>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaggingActionOptions {
	pub action: Option<String>,
	pub tiers: Option<Vec<i64>>,
	pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StitchingActionOptions {
	pub clear_existing_stitching: Option<bool>,
	pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageActionOptions {
	pub redo: Option<bool>,
	pub upgrade_2d_to_3d: Option<bool>,
	pub preview_3d: Option<bool>,
	pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorCountsActionOptions {
	pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedBackfillSummary {
	pub processed: i64,
	pub errors: i64,
	pub stopped: bool,
	pub actions: Vec<String>,
	pub commit_every: i64,
	pub batch_size: i64,
	pub workers: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopBackfillResult {
	pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackfillLogEntry {
	pub level: String,
	pub message: String,
}

pub fn request_stop() -> StopBackfillResult {
	let already = STOP_REQUESTED.swap(true, Ordering::SeqCst);
	StopBackfillResult {
		status: if already {
			"already_stopping".to_string()
		} else {
			"stopping".to_string()
		},
	}
}

pub fn clear_stop_signal() {
	STOP_REQUESTED.store(false, Ordering::SeqCst);
}

pub async fn run_unified_backfill(
	pool: &SqlitePool,
	request: UnifiedBackfillRequest,
	has_api_key: bool,
) -> Result<UnifiedBackfillSummary, String> {
	clear_stop_signal();
	truncate_logs_for_new_run()?;

	let actions = request.actions.unwrap_or(UnifiedBackfillActions {
		tagging: Some(TaggingActionOptions {
			action: Some(TAG_ACTION_UNTAGGED.to_string()),
			tiers: Some(vec![1]),
			enabled: Some(true),
		}),
		stitching: None,
		images: None,
		color_counts: None,
	});

	let batch_size = resolve_i64_option(
		request.batch_size,
		get_i64_setting(pool, "ai.batch_size").await?,
		DEFAULT_BATCH_SIZE,
		1,
		100_000,
	);
	let commit_every = resolve_i64_option(
		request.commit_every,
		get_i64_setting(pool, "import.commit_batch_size").await?,
		DEFAULT_COMMIT_EVERY,
		1,
		100_000,
	);
	let workers = request.workers.unwrap_or(DEFAULT_WORKERS).clamp(1, 32);
	let tier2_delay_seconds = resolve_f64_option(
		request.delay_seconds,
		get_f64_setting(pool, "ai.delay").await?,
		DEFAULT_DELAY_SECONDS,
		0.0,
		120.0,
	);
	let tier3_delay_seconds = resolve_f64_option(
		request.vision_delay_seconds,
		None,
		DEFAULT_VISION_DELAY_SECONDS,
		0.0,
		120.0,
	);

	log_info(format!(
		"Run started batch_size={} commit_every={} workers={} tier2_delay={} tier3_delay={} api_key={}",
		batch_size, commit_every, workers, tier2_delay_seconds, tier3_delay_seconds, has_api_key
	));

	let mut processed: i64 = 0;
	let mut errors: i64 = 0;
	let mut actions_run: Vec<String> = Vec::new();
	let mut touched_design_ids = HashSet::<i64>::new();

	if let Some(tagging_action) = actions.tagging {
		if tagging_action.enabled.unwrap_or(true) {
			actions_run.push("tagging".to_string());
			let mode = normalize_tag_mode(tagging_action.action.as_deref());
			let tiers = normalize_tiers(tagging_action.tiers.as_deref(), has_api_key);
			let tier1_enabled = tiers.contains(&1);
			let tier2_enabled = tiers.contains(&2) && has_api_key;
			let tier3_enabled = tiers.contains(&3) && has_api_key;

			let image_tag_map = get_image_tag_lookup(pool).await?;
			let valid_descriptions = image_tag_map.keys().cloned().collect::<HashSet<String>>();
			let design_ids = select_tagging_design_ids(pool, mode, batch_size).await?;
			log_info(format!(
				"Tagging action={} candidates={} tiers={:?}",
				mode,
				design_ids.len(),
				tiers
			));

			for (index, design_id) in design_ids.iter().enumerate() {
				if STOP_REQUESTED.load(Ordering::SeqCst) {
					log_info("Stop signal observed during tagging loop".to_string());
					break;
				}

				touched_design_ids.insert(*design_id);
				processed += 1;
				let tag_result = apply_tagging_tiers(
					pool,
					*design_id,
					&image_tag_map,
					&valid_descriptions,
					tier1_enabled,
					tier2_enabled,
					tier3_enabled,
					tier2_delay_seconds,
					tier3_delay_seconds,
				)
				.await;

				if let Err(error) = tag_result {
					errors += 1;
					log_error(format!("Tagging failed design_id={} error={}", design_id, error));
				}

				if ((index as i64) + 1) % commit_every == 0 {
					// SQLx autocommit mode keeps each statement durable.
					// This branch exists to preserve parity with commit cadence semantics.
				}
			}
		}
	}

	if let Some(stitching_action) = actions.stitching {
		if stitching_action.enabled.unwrap_or(true) {
			actions_run.push("stitching".to_string());
			if stitching_action.clear_existing_stitching.unwrap_or(false) {
				let cleared = clear_unverified_stitching_tags(pool).await?;
				touched_design_ids.extend(cleared);
				log_info("Cleared existing stitching tags for unverified designs".to_string());
			}
			let stitching_candidates = select_stitching_candidates(pool, batch_size).await?;
			let stitching_tag_lookup = get_stitching_tag_lookup(pool).await?;
			let valid_stitching_descriptions = stitching_tag_lookup
				.keys()
				.cloned()
				.collect::<HashSet<String>>();
			let default_stitching_tag_id = get_default_stitching_tag_id(pool).await?;
			for candidate in stitching_candidates {
				if STOP_REQUESTED.load(Ordering::SeqCst) {
					break;
				}
				touched_design_ids.insert(candidate.id);
				processed += 1;

				let detected_descriptions = stitch_identifier::suggest_stitching_from_pattern_file(
					&candidate.filepath,
					&candidate.filename,
					&candidate.filepath,
					&valid_stitching_descriptions,
					Some(0.70),
				);

				let mut detected_tag_ids = Vec::new();
				for description in &detected_descriptions {
					if let Some(tag_id) = stitching_tag_lookup.get(description) {
						detected_tag_ids.push(*tag_id);
					}
				}

				if detected_tag_ids.is_empty() {
					if let Some(tag_id) = default_stitching_tag_id {
						detected_tag_ids.push(tag_id);
					}
				}

				if let Err(error) = apply_stitching_tags(pool, candidate.id, &detected_tag_ids).await {
					errors += 1;
					log_error(format!(
						"Stitching update failed design_id={} error={}",
						candidate.id, error
					));
				} else if !detected_descriptions.is_empty() {
					log_info(format!(
						"Stitching detected design_id={} tags={:?}",
						candidate.id, detected_descriptions
					));
				}
			}
		}
	}

	if let Some(images_action) = actions.images {
		if images_action.enabled.unwrap_or(true) {
			actions_run.push("images".to_string());
			let preview_3d = images_action.preview_3d.or(request.preview_3d).unwrap_or(true);
			let preview_3d_profile = get_string_setting(pool, "image.preview_3d_profile")
				.await?
				.unwrap_or_else(|| "balanced".to_string());
			let image_candidates = select_image_candidates(
				pool,
				images_action.redo.unwrap_or(false),
				images_action.upgrade_2d_to_3d.unwrap_or(false),
				preview_3d,
				batch_size,
			)
			.await?;
			for design_id in image_candidates {
				if STOP_REQUESTED.load(Ordering::SeqCst) {
					break;
				}
				touched_design_ids.insert(design_id);
				processed += 1;

				if images_action.redo.unwrap_or(false) {
					let _ = clear_image_fields(pool, design_id).await;
				}

				if let Err(error) = generate_and_store_preview(pool, design_id, preview_3d, &preview_3d_profile).await {
					errors += 1;
					log_error(format!("Image action failed design_id={} error={}", design_id, error));
				}
			}
		}
	}

	if let Some(color_counts_action) = actions.color_counts {
		if color_counts_action.enabled.unwrap_or(true) {
			actions_run.push("color_counts".to_string());
			let color_candidates = select_color_count_candidates(pool, batch_size).await?;
			for design_id in color_candidates {
				if STOP_REQUESTED.load(Ordering::SeqCst) {
					break;
				}
				touched_design_ids.insert(design_id);
				processed += 1;
				if let Err(error) = update_color_counts_only(pool, design_id).await {
					errors += 1;
					log_error(format!(
						"Colour-count action failed design_id={} error={}",
						design_id, error
					));
				}
			}
		}
	}

	if processed == 0 {
		processed = touched_design_ids.len() as i64;
	}

	let stopped = STOP_REQUESTED.load(Ordering::SeqCst);
	log_info(format!(
		"Run complete processed={} errors={} stopped={} actions={:?}",
		processed, errors, stopped, actions_run
	));

	Ok(UnifiedBackfillSummary {
		processed,
		errors,
		stopped,
		actions: actions_run,
		commit_every,
		batch_size,
		workers,
	})
}

pub async fn get_backfill_log_entries(_pool: &SqlitePool, limit: i64) -> Result<Vec<BackfillLogEntry>, String> {
	let bounded = limit.clamp(1, 200) as usize;
	let mut entries = Vec::new();

	entries.extend(read_log_tail(&info_log_path(), "info", bounded)?);
	entries.extend(read_log_tail(&error_log_path(), "error", bounded)?);

	if entries.len() > bounded {
		entries = entries[entries.len() - bounded..].to_vec();
	}

	Ok(entries)
}

fn normalize_tag_mode(raw: Option<&str>) -> &str {
	match raw.unwrap_or(TAG_ACTION_UNTAGGED).trim().to_ascii_lowercase().as_str() {
		TAG_ACTION_RETAG_ALL => TAG_ACTION_RETAG_ALL,
		TAG_ACTION_RETAG_ALL_UNVERIFIED => TAG_ACTION_RETAG_ALL_UNVERIFIED,
		_ => TAG_ACTION_UNTAGGED,
	}
}

fn normalize_tiers(raw: Option<&[i64]>, has_api_key: bool) -> HashSet<i64> {
	let mut tiers = HashSet::new();
	tiers.insert(1);

	if let Some(values) = raw {
		for tier in values {
			if *tier == 1 || (*tier >= 2 && has_api_key) {
				tiers.insert(*tier);
			}
		}
	}

	tiers
}

async fn get_image_tag_lookup(pool: &SqlitePool) -> Result<HashMap<String, i64>, String> {
	let rows = sqlx::query(
		"SELECT id, description FROM tags WHERE lower(COALESCE(tag_group, '')) = 'image' ORDER BY description COLLATE NOCASE",
	)
	.fetch_all(pool)
	.await
	.map_err(|e| e.to_string())?;

	let mut map = HashMap::new();
	for row in rows {
		let tag_id: i64 = row.try_get("id").map_err(|e| e.to_string())?;
		let description: String = row.try_get("description").map_err(|e| e.to_string())?;
		map.insert(description, tag_id);
	}

	Ok(map)
}

async fn select_tagging_design_ids(pool: &SqlitePool, mode: &str, limit: i64) -> Result<Vec<i64>, String> {
	let sql = match mode {
		TAG_ACTION_RETAG_ALL => {
			"SELECT id FROM designs ORDER BY id ASC LIMIT ?"
		}
		TAG_ACTION_RETAG_ALL_UNVERIFIED => {
			"SELECT id FROM designs WHERE COALESCE(tags_checked, 0) = 0 ORDER BY id ASC LIMIT ?"
		}
		_ => {
			"SELECT d.id
			 FROM designs d
			 WHERE NOT EXISTS (
			   SELECT 1
			   FROM design_tags dt
			   JOIN tags t ON t.id = dt.tag_id
			   WHERE dt.design_id = d.id AND lower(COALESCE(t.tag_group, '')) = 'image'
			 )
			 ORDER BY d.id ASC
			 LIMIT ?"
		}
	};

	let rows = sqlx::query(sql)
		.bind(limit)
		.fetch_all(pool)
		.await
		.map_err(|e| e.to_string())?;

	let mut ids = Vec::with_capacity(rows.len());
	for row in rows {
		ids.push(row.try_get::<i64, _>("id").map_err(|e| e.to_string())?);
	}
	Ok(ids)
}

async fn apply_tagging_tiers(
	pool: &SqlitePool,
	design_id: i64,
	image_tag_map: &HashMap<String, i64>,
	valid_descriptions: &HashSet<String>,
	tier1_enabled: bool,
	tier2_enabled: bool,
	tier3_enabled: bool,
	tier2_delay_seconds: f64,
	tier3_delay_seconds: f64,
) -> Result<(), String> {
	let row = sqlx::query("SELECT filename, filepath, image_data FROM designs WHERE id = ?")
		.bind(design_id)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?;

	let Some(row) = row else {
		return Ok(());
	};

	let filename: String = row.try_get("filename").map_err(|e| e.to_string())?;
	let filepath: String = row.try_get("filepath").map_err(|e| e.to_string())?;
	let image_data: Option<Vec<u8>> = row.try_get("image_data").map_err(|e| e.to_string())?;

	if tier1_enabled {
		let tier1 = tagging::suggest_tier1_descriptions(&filename, &filepath, valid_descriptions);
		if !tier1.is_empty() {
			return apply_image_tags_and_tier(pool, design_id, image_tag_map, tier1, 1).await;
		}
	}

	if tier2_enabled {
		if tier2_delay_seconds > 0.0 {
			sleep(Duration::from_secs_f64(tier2_delay_seconds)).await;
		}
		let tier2 = suggest_tier2_descriptions(&filename, &filepath, valid_descriptions);
		if !tier2.is_empty() {
			return apply_image_tags_and_tier(pool, design_id, image_tag_map, tier2, 2).await;
		}
	}

	if tier3_enabled && image_data.is_some() {
		if tier3_delay_seconds > 0.0 {
			sleep(Duration::from_secs_f64(tier3_delay_seconds)).await;
		}
		let tier3 = suggest_tier3_descriptions(&filename, &filepath, valid_descriptions);
		if !tier3.is_empty() {
			return apply_image_tags_and_tier(pool, design_id, image_tag_map, tier3, 3).await;
		}
	}

	Ok(())
}

fn suggest_tier2_descriptions(
	filename: &str,
	filepath: &str,
	valid_descriptions: &HashSet<String>,
) -> Vec<String> {
	let combined = format!("{} {}", filename.to_ascii_lowercase(), filepath.to_ascii_lowercase());
	let mut suggestions = Vec::new();

	for description in valid_descriptions {
		let tokenized = description
			.to_ascii_lowercase()
			.replace('&', " ")
			.replace('-', " ")
			.replace('"', " ");
		let desc_tokens: Vec<&str> = tokenized.split_whitespace().filter(|token| token.len() > 2).collect();
		if !desc_tokens.is_empty() && desc_tokens.iter().all(|token| combined.contains(token)) {
			suggestions.push(description.clone());
		}
	}

	if suggestions.is_empty() {
		for description in ["Don't Know", "Patterns", "Flowers", "Animals"] {
			if valid_descriptions.contains(description) {
				suggestions.push(description.to_string());
				break;
			}
		}
	}

	suggestions.sort();
	suggestions
}

fn suggest_tier3_descriptions(
	filename: &str,
	filepath: &str,
	valid_descriptions: &HashSet<String>,
) -> Vec<String> {
	let mut tier3 = suggest_tier2_descriptions(filename, filepath, valid_descriptions);
	if tier3.is_empty() {
		if valid_descriptions.contains("Don't Know") {
			tier3.push("Don't Know".to_string());
		}
	}
	tier3
}

async fn apply_image_tags_and_tier(
	pool: &SqlitePool,
	design_id: i64,
	image_tag_map: &HashMap<String, i64>,
	descriptions: Vec<String>,
	tier: i64,
) -> Result<(), String> {
	if descriptions.is_empty() {
		return Ok(());
	}

	sqlx::query(
		"DELETE FROM design_tags
		 WHERE design_id = ?
		   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'image')",
	)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	for description in descriptions {
		if let Some(tag_id) = image_tag_map.get(&description) {
			sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
				.bind(design_id)
				.bind(*tag_id)
				.execute(pool)
				.await
				.map_err(|e| e.to_string())?;
		}
	}

	sqlx::query("UPDATE designs SET tagging_tier = ?, tags_checked = 0 WHERE id = ?")
		.bind(tier)
		.bind(design_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	Ok(())
}

async fn clear_unverified_stitching_tags(pool: &SqlitePool) -> Result<Vec<i64>, String> {
	let rows = sqlx::query(
		"SELECT DISTINCT dt.design_id AS id
		 FROM design_tags dt
		 JOIN designs d ON d.id = dt.design_id
		 JOIN tags t ON t.id = dt.tag_id
		 WHERE lower(COALESCE(t.tag_group, '')) = 'stitching'
		   AND COALESCE(d.tags_checked, 0) = 0",
	)
	.fetch_all(pool)
	.await
	.map_err(|e| e.to_string())?;

	sqlx::query(
		"DELETE FROM design_tags
		 WHERE design_id IN (SELECT id FROM designs WHERE COALESCE(tags_checked, 0) = 0)
		   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching')",
	)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	let mut ids = Vec::new();
	for row in rows {
		ids.push(row.try_get::<i64, _>("id").map_err(|e| e.to_string())?);
	}
	Ok(ids)
}

async fn select_stitching_candidates(
	pool: &SqlitePool,
	limit: i64,
) -> Result<Vec<StitchingCandidate>, String> {
	let rows = sqlx::query(
		"SELECT d.id, d.filename, d.filepath
		 FROM designs d
		 WHERE NOT EXISTS (
		   SELECT 1
		   FROM design_tags dt
		   JOIN tags t ON t.id = dt.tag_id
		   WHERE dt.design_id = d.id AND lower(COALESCE(t.tag_group, '')) = 'stitching'
		 )
		 ORDER BY d.id ASC
		 LIMIT ?",
	)
	.bind(limit)
	.fetch_all(pool)
	.await
	.map_err(|e| e.to_string())?;

	let mut candidates = Vec::new();
	for row in rows {
		candidates.push(StitchingCandidate {
			id: row.try_get::<i64, _>("id").map_err(|e| e.to_string())?,
			filename: row
				.try_get::<String, _>("filename")
				.map_err(|e| e.to_string())?,
			filepath: row
				.try_get::<String, _>("filepath")
				.map_err(|e| e.to_string())?,
		});
	}
	Ok(candidates)
}

#[derive(Debug, Clone)]
struct StitchingCandidate {
	id: i64,
	filename: String,
	filepath: String,
}

async fn get_stitching_tag_lookup(pool: &SqlitePool) -> Result<HashMap<String, i64>, String> {
	let rows = sqlx::query(
		"SELECT id, description
		 FROM tags
		 WHERE lower(COALESCE(tag_group, '')) = 'stitching'",
	)
	.fetch_all(pool)
	.await
	.map_err(|e| e.to_string())?;

	let mut map = HashMap::new();
	for row in rows {
		let tag_id = row.try_get::<i64, _>("id").map_err(|e| e.to_string())?;
		let description = row
			.try_get::<String, _>("description")
			.map_err(|e| e.to_string())?;
		map.insert(description, tag_id);
	}
	Ok(map)
}

async fn get_default_stitching_tag_id(pool: &SqlitePool) -> Result<Option<i64>, String> {
	let row = sqlx::query(
		"SELECT id
		 FROM tags
		 WHERE lower(COALESCE(tag_group, '')) = 'stitching'
		 ORDER BY CASE WHEN lower(description) = 'line outline' THEN 0 ELSE 1 END, description COLLATE NOCASE
		 LIMIT 1",
	)
	.fetch_optional(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(row
		.and_then(|record| record.try_get::<i64, _>("id").ok()))
}

async fn apply_stitching_tags(pool: &SqlitePool, design_id: i64, tag_ids: &[i64]) -> Result<(), String> {
	sqlx::query(
		"DELETE FROM design_tags
		 WHERE design_id = ?
		   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching')",
	)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	for tag_id in tag_ids {
		sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
			.bind(design_id)
			.bind(*tag_id)
			.execute(pool)
			.await
			.map_err(|e| e.to_string())?;
	}

	Ok(())
}

async fn select_image_candidates(
	pool: &SqlitePool,
	redo: bool,
	upgrade_2d_to_3d: bool,
	preview_3d: bool,
	limit: i64,
) -> Result<Vec<i64>, String> {
	let sql = if redo {
		"SELECT id FROM designs ORDER BY id ASC LIMIT ?"
	} else if upgrade_2d_to_3d && preview_3d {
		"SELECT id FROM designs WHERE lower(COALESCE(image_type, '')) = '2d' OR image_type IS NULL ORDER BY id ASC LIMIT ?"
	} else {
		"SELECT id FROM designs WHERE image_data IS NULL ORDER BY id ASC LIMIT ?"
	};

	let rows = sqlx::query(sql)
		.bind(limit)
		.fetch_all(pool)
		.await
		.map_err(|e| e.to_string())?;

	let mut ids = Vec::new();
	for row in rows {
		ids.push(row.try_get::<i64, _>("id").map_err(|e| e.to_string())?);
	}
	Ok(ids)
}

async fn clear_image_fields(pool: &SqlitePool, design_id: i64) -> Result<(), String> {
	sqlx::query(
		"UPDATE designs
		 SET image_data = NULL,
		     image_type = NULL,
		     width_mm = NULL,
		     height_mm = NULL
		 WHERE id = ?",
	)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;
	Ok(())
}

async fn generate_and_store_preview(
	pool: &SqlitePool,
	design_id: i64,
	preview_3d: bool,
	preview_3d_profile: &str,
) -> Result<(), String> {
	let row = sqlx::query("SELECT filepath FROM designs WHERE id = ?")
		.bind(design_id)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?;

	let Some(row) = row else {
		return Ok(());
	};

	let filepath: String = row.try_get("filepath").map_err(|e| e.to_string())?;
	let result = generate_preview(&ImageGenerationRequest {
		file_path: filepath,
		preview_3d,
		preview_3d_profile: Some(preview_3d_profile.to_string()),
	});

	if let Some(error) = result.error {
		return Err(error);
	}

	sqlx::query(
		"UPDATE designs
		 SET image_data = ?,
		     image_type = ?,
		     width_mm = ?,
		     height_mm = ?,
		     stitch_count = COALESCE(?, stitch_count),
		     color_count = COALESCE(?, color_count),
		     color_change_count = COALESCE(?, color_change_count)
		 WHERE id = ?",
	)
	.bind(result.image_data)
	.bind(result.image_type)
	.bind(result.width_mm.map(|value| value.round() as i64))
	.bind(result.height_mm.map(|value| value.round() as i64))
	.bind(result.stitch_count)
	.bind(result.color_count)
	.bind(result.color_change_count)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(())
}

async fn select_color_count_candidates(pool: &SqlitePool, limit: i64) -> Result<Vec<i64>, String> {
	let rows = sqlx::query(
		"SELECT id
		 FROM designs
		 WHERE stitch_count IS NULL OR color_count IS NULL OR color_change_count IS NULL
		 ORDER BY id ASC
		 LIMIT ?",
	)
	.bind(limit)
	.fetch_all(pool)
	.await
	.map_err(|e| e.to_string())?;

	let mut ids = Vec::new();
	for row in rows {
		ids.push(row.try_get::<i64, _>("id").map_err(|e| e.to_string())?);
	}
	Ok(ids)
}

async fn update_color_counts_only(pool: &SqlitePool, design_id: i64) -> Result<(), String> {
	let row = sqlx::query("SELECT filepath FROM designs WHERE id = ?")
		.bind(design_id)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?;

	let Some(row) = row else {
		return Ok(());
	};

	let filepath: String = row.try_get("filepath").map_err(|e| e.to_string())?;
	let result = generate_preview(&ImageGenerationRequest {
		file_path: filepath,
		preview_3d: false,
		preview_3d_profile: None,
	});

	if let Some(error) = result.error {
		return Err(error);
	}

	sqlx::query(
		"UPDATE designs
		 SET stitch_count = COALESCE(?, stitch_count),
		     color_count = COALESCE(?, color_count),
		     color_change_count = COALESCE(?, color_change_count)
		 WHERE id = ?",
	)
	.bind(result.stitch_count)
	.bind(result.color_count)
	.bind(result.color_change_count)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(())
}

async fn get_i64_setting(pool: &SqlitePool, key: &str) -> Result<Option<i64>, String> {
	let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
		.bind(key)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.and_then(|row| row.try_get::<String, _>("value").ok());

	Ok(value.and_then(|raw| raw.trim().parse::<i64>().ok()))
}

async fn get_f64_setting(pool: &SqlitePool, key: &str) -> Result<Option<f64>, String> {
	let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
		.bind(key)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.and_then(|row| row.try_get::<String, _>("value").ok());

	Ok(value.and_then(|raw| raw.trim().parse::<f64>().ok()))
}

async fn get_string_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
	let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
		.bind(key)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.and_then(|row| row.try_get::<String, _>("value").ok())
		.map(|raw| raw.trim().to_ascii_lowercase());

	Ok(value)
}

fn resolve_i64_option(request: Option<i64>, setting: Option<i64>, default: i64, min: i64, max: i64) -> i64 {
	request.or(setting).unwrap_or(default).clamp(min, max)
}

fn resolve_f64_option(request: Option<f64>, setting: Option<f64>, default: f64, min: f64, max: f64) -> f64 {
	request.or(setting).unwrap_or(default).clamp(min, max)
}

fn log_dir_path() -> PathBuf {
	Path::new(LOG_DIR).to_path_buf()
}

fn info_log_path() -> PathBuf {
	log_dir_path().join(INFO_LOG_FILE)
}

fn error_log_path() -> PathBuf {
	log_dir_path().join(ERROR_LOG_FILE)
}

fn truncate_logs_for_new_run() -> Result<(), String> {
	let dir = log_dir_path();
	fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	fs::write(info_log_path(), "").map_err(|e| e.to_string())?;
	fs::write(error_log_path(), "").map_err(|e| e.to_string())?;
	Ok(())
}

fn now_epoch_seconds() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs()
}

fn append_log_line(path: &Path, line: &str) {
	let existing = fs::read_to_string(path).unwrap_or_default();
	let mut content = existing;
	content.push_str(line);
	content.push('\n');
	let _ = fs::write(path, content);
}

fn log_info(message: String) {
	append_log_line(&info_log_path(), &format!("{}\t{}", now_epoch_seconds(), message));
}

fn log_error(message: String) {
	append_log_line(&error_log_path(), &format!("{}\t{}", now_epoch_seconds(), message));
}

fn read_log_tail(path: &Path, level: &str, limit: usize) -> Result<Vec<BackfillLogEntry>, String> {
	if !path.exists() {
		return Ok(Vec::new());
	}
	let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
	let mut lines = text
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(|line| BackfillLogEntry {
			level: level.to_string(),
			message: line.to_string(),
		})
		.collect::<Vec<_>>();

	if lines.len() > limit {
		lines = lines[lines.len() - limit..].to_vec();
	}

	Ok(lines)
}

#[cfg(test)]
mod tests {
	use super::*;

	async fn make_test_pool() -> SqlitePool {
		let pool = SqlitePool::connect("sqlite::memory:").await.expect("memory db");
		for sql in [
			"CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, description TEXT)",
			"CREATE TABLE tags (id INTEGER PRIMARY KEY, description TEXT NOT NULL, tag_group TEXT)",
			"CREATE TABLE designs (id INTEGER PRIMARY KEY, filename TEXT NOT NULL, filepath TEXT NOT NULL, image_data BLOB, image_type TEXT, width_mm INTEGER, height_mm INTEGER, stitch_count INTEGER, color_count INTEGER, color_change_count INTEGER, tags_checked INTEGER NOT NULL DEFAULT 0, tagging_tier INTEGER)",
			"CREATE TABLE design_tags (design_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, PRIMARY KEY(design_id, tag_id))",
		] {
			sqlx::query(sql).execute(&pool).await.expect("schema");
		}
		pool
	}

	async fn seed_basic(pool: &SqlitePool) {
		sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (1, 'Cats', 'image')").execute(pool).await.expect("seed tag");
		sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (2, 'Line Outline', 'stitching')").execute(pool).await.expect("seed tag2");
		sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (?, ?, ?)")
			.bind(3_i64)
			.bind("Don't Know")
			.bind("image")
			.execute(pool)
			.await
			.expect("seed tag3");
		sqlx::query("INSERT INTO designs (id, filename, filepath, tags_checked) VALUES (1, 'cute_cat.pes', 'tests/testdata/cute_cat.pes', 0)").execute(pool).await.expect("seed design1");
		sqlx::query("INSERT INTO designs (id, filename, filepath, tags_checked) VALUES (2, 'dog_crest.pes', 'tests/testdata/dog_crest.pes', 1)").execute(pool).await.expect("seed design2");
		sqlx::query("INSERT INTO designs (id, filename, filepath, tags_checked) VALUES (3, 'flower.pes', 'tests/testdata/flower.pes', 0)").execute(pool).await.expect("seed design3");
		sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 1)").execute(pool).await.expect("seed design tag");
	}

	#[tokio::test]
	async fn run_unified_backfill_tag_untagged_skips_tagged_designs() {
		let pool = make_test_pool().await;
		seed_basic(&pool).await;

		let summary = run_unified_backfill(
			&pool,
			UnifiedBackfillRequest {
				actions: Some(UnifiedBackfillActions {
					tagging: Some(TaggingActionOptions {
						action: Some("tag_untagged".to_string()),
						tiers: Some(vec![1]),
						enabled: Some(true),
					}),
					stitching: None,
					images: None,
					color_counts: None,
				}),
				batch_size: Some(100),
				commit_every: Some(100),
				workers: Some(1),
				preview_3d: Some(false),
				delay_seconds: Some(0.0),
				vision_delay_seconds: Some(0.0),
			},
			false,
		)
		.await
		.expect("run succeeds");

		assert!(summary.processed >= 2);
		let still_tagged = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 1")
			.fetch_one(&pool)
			.await
			.expect("count");
		assert_eq!(still_tagged, 1);
	}

	#[tokio::test]
	async fn stop_state_transitions_are_stable() {
		clear_stop_signal();
		let first = request_stop();
		let second = request_stop();
		assert_eq!(first.status, "stopping");
		assert_eq!(second.status, "already_stopping");
		clear_stop_signal();
	}

	#[test]
	fn normalize_tag_mode_covers_all_cases() {
		assert_eq!(normalize_tag_mode(Some("retag_all")), TAG_ACTION_RETAG_ALL);
		assert_eq!(
			normalize_tag_mode(Some("retag_all_unverified")),
			TAG_ACTION_RETAG_ALL_UNVERIFIED
		);
		assert_eq!(normalize_tag_mode(Some("unknown")), TAG_ACTION_UNTAGGED);
	}
}
