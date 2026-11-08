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

pub(crate) async fn insert_or_update_design_file(
    pool: &SqlitePool,
    design_id: i64,
    canonical_filepath: &str,
    filename: &str,
    extension: &str,
    file_type: &str,
    is_primary: bool,
    stitch_count: Option<i64>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    color_count: Option<i64>,
    color_change_count: Option<i64>,
    file_size_bytes: Option<i64>,
    file_hash_blake3: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO design_files (
            design_id, filepath, filename, extension, file_type, is_primary,
            stitch_count, width_mm, height_mm, color_count, color_change_count,
            file_size_bytes, file_hash_blake3, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
        ON CONFLICT(filepath) DO UPDATE SET
            design_id = excluded.design_id,
            filename = excluded.filename,
            extension = excluded.extension,
            file_type = excluded.file_type,
            is_primary = excluded.is_primary,
            stitch_count = COALESCE(excluded.stitch_count, design_files.stitch_count),
            width_mm = COALESCE(excluded.width_mm, design_files.width_mm),
            height_mm = COALESCE(excluded.height_mm, design_files.height_mm),
            color_count = COALESCE(excluded.color_count, design_files.color_count),
            color_change_count = COALESCE(excluded.color_change_count, design_files.color_change_count),
            file_size_bytes = COALESCE(excluded.file_size_bytes, design_files.file_size_bytes),
            file_hash_blake3 = COALESCE(excluded.file_hash_blake3, design_files.file_hash_blake3),
            updated_at = datetime('now')
        "#,
    )
    .bind(design_id)
    .bind(canonical_filepath)
    .bind(filename)
    .bind(extension)
    .bind(file_type)
    .bind(is_primary)
    .bind(stitch_count)
    .bind(width_mm)
    .bind(height_mm)
    .bind(color_count)
    .bind(color_change_count)
    .bind(file_size_bytes)
    .bind(file_hash_blake3)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
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

    let preferred_format: String = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'import.preferred_stitch_format'",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or_default()
    .unwrap_or_else(|| "pes".to_string());

    let mut files_by_stem: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for p in disk_stitch_files {
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !stem.is_empty() {
            files_by_stem.entry(stem).or_default().push(p);
        }
    }

    for (candidate_id, _, candidate_filepath, candidate_master_fp) in folder_candidates {
        let target_master_file = candidate_master_fp
            .as_deref()
            .unwrap_or(&candidate_filepath);
        let canonical_master = crate::paths::canonical_design_rel(target_master_file);
        let candidate_stem = Path::new(&canonical_master)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let Some(mut stitch_paths) = files_by_stem.remove(&candidate_stem) else {
            continue;
        };
        if stitch_paths.is_empty() {
            continue;
        }

        stitch_paths.sort_by(|a, b| {
            let ext_a = a
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let ext_b = b
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let a_pref = ext_a.eq_ignore_ascii_case(&preferred_format);
            let b_pref = ext_b.eq_ignore_ascii_case(&preferred_format);
            if a_pref != b_pref {
                return b_pref.cmp(&a_pref);
            }
            ext_a.cmp(&ext_b)
        });

        let primary_stitch_path = &stitch_paths[0];
        let primary_stem = primary_stitch_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let primary_filename = primary_stitch_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| primary_stem.to_string());
        let primary_rel = if folder_rel.is_empty() {
            primary_filename.clone()
        } else {
            format!("{}/{}", folder_rel, primary_filename)
        };
        let canonical_primary = crate::paths::canonical_design_rel(&primary_rel);

        let primary_outcome = design_metadata::parse_design_file_lenient(primary_stitch_path);
        let primary_parsed = primary_outcome.parsed;

        let hoop_id = match design_metadata::recommend_hoop_for_design(
            pool,
            primary_parsed.width_mm,
            primary_parsed.height_mm,
        )
        .await
        {
            Ok(id) => id,
            Err(_) => None,
        };

        let primary_file_size =
            crate::routes::bulk_import::compute_file_size(primary_stitch_path).ok();
        let primary_file_hash =
            crate::routes::bulk_import::compute_file_hash_blake3(primary_stitch_path).ok();
        let preview_error = primary_outcome.error.as_ref().map(|err| {
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
        .bind(&primary_filename)
        .bind(&canonical_primary)
        .bind(&canonical_master)
        .bind(hoop_id)
        .bind(primary_parsed.image_data)
        .bind(primary_parsed.image_type)
        .bind(primary_parsed.width_mm)
        .bind(primary_parsed.height_mm)
        .bind(primary_parsed.stitch_count)
        .bind(primary_parsed.color_count)
        .bind(primary_parsed.color_change_count)
        .bind(primary_file_size)
        .bind(primary_file_hash.as_ref())
        .bind(preview_error)
        .bind(candidate_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Could not upgrade master design id={}: {}", candidate_id, e))?;

        // 2. Record master file in design_files
        let master_p = Path::new(&canonical_master);
        let master_fname = master_p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&canonical_master);
        let master_ext = master_p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let master_full = resolve_design_full_path(&canonical_master);
        let master_size = crate::routes::bulk_import::compute_file_size(&master_full).ok();
        let master_hash = crate::routes::bulk_import::compute_file_hash_blake3(&master_full).ok();
        let _ = insert_or_update_design_file(
            pool,
            candidate_id,
            &canonical_master,
            master_fname,
            &master_ext,
            "master",
            false,
            None,
            None,
            None,
            None,
            None,
            master_size,
            master_hash.as_deref(),
        )
        .await;

        // 3. Record all stitch files in design_files (both primary and siblings)
        for (idx, stitch_p) in stitch_paths.iter().enumerate() {
            let is_prim = idx == 0;
            let s_stem = stitch_p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let s_fname = stitch_p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| s_stem.to_string());
            let s_ext = stitch_p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let s_rel = if folder_rel.is_empty() {
                s_fname.clone()
            } else {
                format!("{}/{}", folder_rel, s_fname)
            };
            let canonical_s = crate::paths::canonical_design_rel(&s_rel);

            let (s_stitch_count, s_width, s_height, s_color, s_change, s_size, s_hash) = if is_prim
            {
                (
                    primary_parsed.stitch_count,
                    primary_parsed.width_mm.map(|v| v as f64),
                    primary_parsed.height_mm.map(|v| v as f64),
                    primary_parsed.color_count,
                    primary_parsed.color_change_count,
                    primary_file_size,
                    primary_file_hash.clone(),
                )
            } else {
                let out = design_metadata::parse_design_file_lenient(stitch_p);
                (
                    out.parsed.stitch_count,
                    out.parsed.width_mm.map(|v| v as f64),
                    out.parsed.height_mm.map(|v| v as f64),
                    out.parsed.color_count,
                    out.parsed.color_change_count,
                    crate::routes::bulk_import::compute_file_size(stitch_p).ok(),
                    crate::routes::bulk_import::compute_file_hash_blake3(stitch_p).ok(),
                )
            };

            let _ = insert_or_update_design_file(
                pool,
                candidate_id,
                &canonical_s,
                &s_fname,
                &s_ext,
                "stitch",
                is_prim,
                s_stitch_count,
                s_width,
                s_height,
                s_color,
                s_change,
                s_size,
                s_hash.as_deref(),
            )
            .await;

            // Clean up any stale unlinked duplicate row in designs for this stitch file
            let duplicate_ids: Vec<i64> =
                sqlx::query_scalar("SELECT id FROM designs WHERE filepath = ? AND id != ?")
                    .bind(&canonical_s)
                    .bind(candidate_id)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

            for dup_id in duplicate_ids {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO design_tags (design_id, tag_id) \
                     SELECT ?, tag_id FROM design_tags WHERE design_id = ?",
                )
                .bind(candidate_id)
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
        }

        // 4. Path rules and stitching tag inheritance
        let matched_descriptions = crate::services::tagging::suggest_path_rule_descriptions(
            &primary_filename,
            &canonical_primary,
            &tagging_lookups.valid_descriptions,
            &tagging_lookups.synonyms,
        );
        for description in &matched_descriptions {
            if let Some(tag_id) = tagging_lookups.description_to_tag_id.get(description) {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)",
                )
                .bind(candidate_id)
                .bind(*tag_id)
                .execute(pool)
                .await;
            }
        }

        if primary_stitch_path.exists() {
            let detected_stitching =
                crate::services::stitch_identifier::suggest_stitching_from_pattern_file(
                    &primary_stitch_path.to_string_lossy(),
                    &primary_filename,
                    &canonical_primary,
                    &tagging_lookups.valid_stitching_descriptions,
                    Some(0.70),
                );
            for desc in detected_stitching {
                if let Some(tag_id) = tagging_lookups.stitching_tag_lookup.get(&desc) {
                    let _ = sqlx::query(
                        "INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)",
                    )
                    .bind(candidate_id)
                    .bind(*tag_id)
                    .execute(pool)
                    .await;
                }
            }
        }

        if candidate_id == design_id {
            current_design_paired = true;
        }
        paired_count += 1;
    }

    // Sibling discovery for already-paired stitch designs
    if !target_is_master_only {
        let design_stem_key = Path::new(&canonical_target)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if let Some(stitch_paths) = files_by_stem.get(&design_stem_key) {
            for p in stitch_paths {
                let s_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let s_fname = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| s_stem.to_string());
                let s_ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let s_rel = if folder_rel.is_empty() {
                    s_fname.clone()
                } else {
                    format!("{}/{}", folder_rel, s_fname)
                };
                let canonical_s = crate::paths::canonical_design_rel(&s_rel);

                let exists: bool = sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM design_files WHERE filepath = ? LIMIT 1",
                )
                .bind(&canonical_s)
                .fetch_optional(pool)
                .await
                .unwrap_or_default()
                .is_some();

                if !exists {
                    let out = design_metadata::parse_design_file_lenient(p);
                    let s_size = crate::routes::bulk_import::compute_file_size(p).ok();
                    let s_hash = crate::routes::bulk_import::compute_file_hash_blake3(p).ok();
                    let _ = insert_or_update_design_file(
                        pool,
                        design_id,
                        &canonical_s,
                        &s_fname,
                        &s_ext,
                        "stitch",
                        false,
                        out.parsed.stitch_count,
                        out.parsed.width_mm.map(|v| v as f64),
                        out.parsed.height_mm.map(|v| v as f64),
                        out.parsed.color_count,
                        out.parsed.color_change_count,
                        s_size,
                        s_hash.as_deref(),
                    )
                    .await;
                    current_design_paired = true;
                    paired_count += 1;
                }
            }
        }
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
