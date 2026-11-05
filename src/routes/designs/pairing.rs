// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::launch::{get_designs_base_path, resolve_design_full_path};
use super::types::ScanFolderStitchFilesResult;
use crate::services::design_metadata;
use crate::AppState;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tauri::State;

struct TaggingLookups {
    valid_descriptions: HashSet<String>,
    description_to_tag_id: HashMap<String, i64>,
    synonyms: HashMap<String, Vec<String>>,
    valid_stitching_descriptions: HashSet<String>,
    stitching_tag_lookup: HashMap<String, i64>,
}

impl TaggingLookups {
    async fn load(pool: &SqlitePool) -> Self {
        let tag_rows: Vec<(i64, String)> =
            sqlx::query_as("SELECT id, description FROM tags ORDER BY id ASC")
                .fetch_all(pool)
                .await
                .unwrap_or_default();

        let valid_descriptions: HashSet<String> =
            tag_rows.iter().map(|(_, desc)| desc.clone()).collect();
        let description_to_tag_id: HashMap<String, i64> =
            tag_rows.into_iter().map(|(id, desc)| (desc, id)).collect();

        let synonyms = crate::services::tag_synonyms::get_synonym_lookup_map(pool)
            .await
            .unwrap_or_default();

        let stitching_rows: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, description FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching'",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let valid_stitching_descriptions: HashSet<String> = stitching_rows
            .iter()
            .map(|(_, desc)| desc.clone())
            .collect();
        let stitching_tag_lookup: HashMap<String, i64> = stitching_rows
            .into_iter()
            .map(|(id, desc)| (desc, id))
            .collect();

        Self {
            valid_descriptions,
            description_to_tag_id,
            synonyms,
            valid_stitching_descriptions,
            stitching_tag_lookup,
        }
    }
}

pub(crate) async fn scan_folder_for_stitch_files_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<ScanFolderStitchFilesResult, String> {
    let row = sqlx::query_as::<_, (String, Option<String>, bool)>(
        "SELECT filepath, master_filepath, is_master_only FROM designs WHERE id = ? LIMIT 1",
    )
    .bind(design_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let (target_filepath, target_master_filepath, target_is_master_only) = match row {
        Some(data) => data,
        None => return Err(format!("Design with id={} not found.", design_id)),
    };

    let target_master_rel = if target_is_master_only {
        target_master_filepath
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(target_filepath)
    } else {
        target_filepath
    };

    let full_path = resolve_design_full_path(&target_master_rel);
    let folder_dir = if full_path.is_dir() {
        full_path
    } else {
        full_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(get_designs_base_path)
    };

    if !folder_dir.is_dir() {
        return Err(format!(
            "Folder not found on disk: {}",
            folder_dir.display()
        ));
    }

    let folder_display = folder_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Designs")
        .to_string();

    let canonical_target = crate::paths::canonical_design_rel(&target_master_rel);
    let folder_rel = Path::new(&canonical_target)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    let mut disk_stitch_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&folder_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if crate::services::scanning::is_supported_extension(ext) {
                        disk_stitch_files.push(p);
                    }
                }
            }
        }
    }
    disk_stitch_files.sort();
    let scanned_count = disk_stitch_files.len();

    let master_rows = sqlx::query_as::<_, (i64, String, String, Option<String>)>(
        "SELECT id, filename, filepath, master_filepath FROM designs WHERE is_master_only = 1",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let folder_candidates: Vec<(i64, String, String, Option<String>)> = master_rows
        .into_iter()
        .filter(|(_, _, filepath, master_fp)| {
            let rel = master_fp.as_deref().unwrap_or(filepath);
            let canonical = crate::paths::canonical_design_rel(rel);
            let parent = Path::new(&canonical)
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            parent.eq_ignore_ascii_case(&folder_rel)
        })
        .collect();

    let tagging_lookups = TaggingLookups::load(pool).await;
    let mut paired_count = 0;
    let mut current_design_paired = false;

    for stitch_path in &disk_stitch_files {
        let stitch_stem = stitch_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if stitch_stem.is_empty() {
            continue;
        }

        // Find candidate master record sharing this file stem
        let matched_candidate = folder_candidates
            .iter()
            .find(|(_, _, filepath, master_fp)| {
                let stem_from_master = master_fp
                    .as_deref()
                    .and_then(|p| Path::new(p).file_stem())
                    .and_then(|s| s.to_str());
                let stem_from_file = Path::new(filepath).file_stem().and_then(|s| s.to_str());

                stem_from_master
                    .map(|s| s.eq_ignore_ascii_case(stitch_stem))
                    .unwrap_or(false)
                    || stem_from_file
                        .map(|s| s.eq_ignore_ascii_case(stitch_stem))
                        .unwrap_or(false)
            });

        let Some((candidate_id, _, candidate_filepath, candidate_master_fp)) = matched_candidate
        else {
            continue;
        };

        let target_master_file = candidate_master_fp.as_deref().unwrap_or(candidate_filepath);
        let canonical_master = crate::paths::canonical_design_rel(target_master_file);

        let stitch_filename = stitch_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| stitch_stem.to_string());

        let stitch_rel = if folder_rel.is_empty() {
            stitch_filename.clone()
        } else {
            format!("{}/{}", folder_rel, stitch_filename)
        };
        let canonical_stitch = crate::paths::canonical_design_rel(&stitch_rel);

        let outcome = design_metadata::parse_design_file_lenient(stitch_path);
        let parsed = outcome.parsed;

        let hoop_id = match design_metadata::recommend_hoop_for_design(
            pool,
            parsed.width_mm,
            parsed.height_mm,
        )
        .await
        {
            Ok(id) => id,
            Err(_) => None,
        };

        let file_size_bytes = crate::routes::bulk_import::compute_file_size(stitch_path).ok();
        let file_hash_blake3 =
            crate::routes::bulk_import::compute_file_hash_blake3(stitch_path).ok();

        let preview_error = outcome.error.as_ref().map(|err| {
            if err.contains("design_exceeds_hoop") {
                "design_exceeds_hoop"
            } else {
                "unreadable"
            }
        });

        // 1. Upgrade the master record in place
        sqlx::query(
            "UPDATE designs SET filename = ?, filepath = ?, master_filepath = ?, is_master_only = 0, \
             date_updated = datetime('now'), hoop_id = ?, image_data = ?, image_type = ?, \
             width_mm = ?, height_mm = ?, stitch_count = ?, color_count = ?, color_change_count = ?, \
             file_size_bytes = ?, file_hash_blake3 = ?, preview_error = ? \
             WHERE id = ?",
        )
        .bind(&stitch_filename)
        .bind(&canonical_stitch)
        .bind(&canonical_master)
        .bind(hoop_id)
        .bind(parsed.image_data)
        .bind(parsed.image_type)
        .bind(parsed.width_mm)
        .bind(parsed.height_mm)
        .bind(parsed.stitch_count)
        .bind(parsed.color_count)
        .bind(parsed.color_change_count)
        .bind(file_size_bytes)
        .bind(file_hash_blake3)
        .bind(preview_error)
        .bind(*candidate_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Could not upgrade master design id={}: {}", candidate_id, e))?;

        // 2. Clean up any pre-existing duplicate row that held this stitch path
        let duplicate_ids: Vec<i64> =
            sqlx::query_scalar("SELECT id FROM designs WHERE filepath = ? AND id != ?")
                .bind(&canonical_stitch)
                .bind(*candidate_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

        for dup_id in duplicate_ids {
            // Merge tags from duplicate into candidate
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO design_tags (design_id, tag_id) \
                 SELECT ?, tag_id FROM design_tags WHERE design_id = ?",
            )
            .bind(*candidate_id)
            .bind(dup_id)
            .execute(pool)
            .await;

            let _ = sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
                .bind(dup_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM designs WHERE id = ?")
                .bind(dup_id)
                .execute(pool)
                .await;
        }

        // 3. Path rule and stitching tag inheritance
        let matched_descriptions = crate::services::tagging::suggest_path_rule_descriptions(
            &stitch_filename,
            &canonical_stitch,
            &tagging_lookups.valid_descriptions,
            &tagging_lookups.synonyms,
        );
        for description in &matched_descriptions {
            if let Some(tag_id) = tagging_lookups.description_to_tag_id.get(description) {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)",
                )
                .bind(*candidate_id)
                .bind(*tag_id)
                .execute(pool)
                .await;
            }
        }

        if stitch_path.exists() {
            let detected_stitching =
                crate::services::stitch_identifier::suggest_stitching_from_pattern_file(
                    &stitch_path.to_string_lossy(),
                    &stitch_filename,
                    &canonical_stitch,
                    &tagging_lookups.valid_stitching_descriptions,
                    Some(0.70),
                );
            for desc in detected_stitching {
                if let Some(tag_id) = tagging_lookups.stitching_tag_lookup.get(&desc) {
                    let _ = sqlx::query(
                        "INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)",
                    )
                    .bind(*candidate_id)
                    .bind(*tag_id)
                    .execute(pool)
                    .await;
                }
            }
        }

        if *candidate_id == design_id {
            current_design_paired = true;
        }
        paired_count += 1;
    }

    let message = if paired_count == 0 {
        format!(
            "No matching stitch files found in '{}' for unlinked master designs.",
            folder_display
        )
    } else if paired_count == 1 {
        format!(
            "Linked 1 master design with its stitch file in '{}'.",
            folder_display
        )
    } else {
        format!(
            "Linked {} master designs with their stitch files in '{}'.",
            paired_count, folder_display
        )
    };

    Ok(ScanFolderStitchFilesResult {
        folder: folder_display,
        scanned_count,
        paired_count,
        current_design_paired,
        message,
    })
}

#[tauri::command]
pub async fn scan_folder_for_stitch_files(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<ScanFolderStitchFilesResult, String> {
    scan_folder_for_stitch_files_with_pool(&state.db_pool()?, design_id).await
}
