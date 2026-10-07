// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::catalog::{load_default_stitching_tag_id, load_stitching_tag_lookup, load_tag_catalog};
use super::inference::{resolve_assignment_for_file, resolve_bulk_import_assignments};
use super::paths::{
    compute_file_hash_blake3, compute_file_size, ensure_file_in_designs_base, get_designs_base_path,
};
use super::session::{
    get_bulk_import_app_handle, BulkImportProgressEvent, BULK_IMPORT_PROGRESS_EVENT,
    BULK_IMPORT_STOP_REQUESTED, DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
};
use super::types::BulkImportConfirmWire;
use crate::services::{image_generation, stitch_identifier, tagging};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tauri::Emitter;

struct ImportExecutionCtx<'a> {
    confirm_wire: &'a BulkImportConfirmWire,
    resolved_assignments: &'a [super::types::ResolvedFolderAssignmentWire],
    valid_descriptions: &'a HashSet<String>,
    description_to_tag_id: &'a HashMap<String, i64>,
    tag_synonyms_map: &'a HashMap<String, Vec<String>>,
    stitching_tag_lookup: &'a HashMap<String, i64>,
    default_stitching_tag_id: Option<i64>,
    preview_3d: bool,
    preview_3d_profile: &'a str,
}

#[allow(clippy::too_many_arguments)]
async fn persist_stitch_design(
    tx: &mut Transaction<'_, Sqlite>,
    file_path: &str,
    stored_filepath: &str,
    stored_master_filepath: Option<&str>,
    existing_design_id: Option<i64>,
    ctx: &ImportExecutionCtx<'_>,
    total_image_gen_ms: &mut u128,
    total_db_insert_ms: &mut u128,
    total_tagging_ms: &mut u128,
    failed_decode_count: &mut usize,
) -> Result<i64, String> {
    let (designer_id, source_id) =
        resolve_assignment_for_file(file_path, ctx.confirm_wire, ctx.resolved_assignments);

    let filename = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file_path)
        .to_string();

    let t_image = Instant::now();
    let image_result =
        image_generation::generate_preview(&image_generation::ImageGenerationRequest {
            file_path: file_path.to_string(),
            preview_3d: ctx.preview_3d,
            preview_3d_profile: Some(ctx.preview_3d_profile.to_string()),
        });
    let image_gen_ms = t_image.elapsed().as_millis();
    *total_image_gen_ms += image_gen_ms;

    if let Some(error) = image_result.error.as_ref() {
        *failed_decode_count += 1;
        tracing::error!(
            "Image generation adapter error for '{}': {}",
            file_path,
            error
        );
    }

    let hoop_id = match (image_result.width_mm, image_result.height_mm) {
        (Some(width_mm), Some(height_mm)) => sqlx::query_scalar::<_, i64>(
            r#"
                SELECT h.id
                FROM hoops h
                WHERE
                    (
                        CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                        AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                    )
                    OR (
                        CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                        AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                    )
                ORDER BY
                    (CAST(h.max_width_mm AS REAL) * CAST(h.max_height_mm AS REAL)) ASC,
                    CAST(h.max_width_mm AS REAL) ASC,
                    CAST(h.max_height_mm AS REAL) ASC,
                    h.name COLLATE NOCASE ASC
                LIMIT 1
                "#,
        )
        .bind(width_mm)
        .bind(height_mm)
        .bind(height_mm)
        .bind(width_mm)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| e.to_string())?,
        _ => None,
    };

    let designs_base_path = get_designs_base_path();
    let stored_path = designs_base_path.join(
        stored_filepath
            .strip_prefix("/MachineEmbroideryDesigns/")
            .unwrap_or(stored_filepath),
    );
    let file_size_bytes: Option<i64> = compute_file_size(&stored_path).ok();
    let file_hash_blake3: Option<String> = compute_file_hash_blake3(&stored_path).ok();

    let (design_id, db_insert_elapsed_ms) = if let Some(id) = existing_design_id {
        let t_insert = Instant::now();
        sqlx::query(
            "UPDATE designs SET filename = ?, filepath = ?, master_filepath = ?, is_master_only = 0, \
             date_added = COALESCE(date_added, datetime('now')), \
             designer_id = COALESCE(?, designer_id), \
             source_id = COALESCE(?, source_id), \
             hoop_id = ?, image_data = ?, image_type = ?, width_mm = ?, height_mm = ?, \
             stitch_count = ?, color_count = ?, color_change_count = ?, \
             is_stitched = 0, stitching_tags_verified = 0, \
             file_size_bytes = ?, file_hash_blake3 = ? \
             WHERE id = ?",
        )
        .bind(&filename)
        .bind(stored_filepath)
        .bind(stored_master_filepath)
        .bind(designer_id)
        .bind(source_id)
        .bind(hoop_id)
        .bind(image_result.image_data)
        .bind(image_result.image_type)
        .bind(image_result.width_mm)
        .bind(image_result.height_mm)
        .bind(image_result.stitch_count)
        .bind(image_result.color_count)
        .bind(image_result.color_change_count)
        .bind(file_size_bytes)
        .bind(file_hash_blake3.as_ref())
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        // Clean up any stale unlinked duplicate row if this was an upgrade of a master record
        let duplicate_ids: Vec<i64> =
            sqlx::query_scalar("SELECT id FROM designs WHERE filepath = ? AND id != ?")
                .bind(stored_filepath)
                .bind(id)
                .fetch_all(&mut **tx)
                .await
                .unwrap_or_default();

        for dup_id in duplicate_ids {
            let _ = sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
                .bind(dup_id)
                .execute(&mut **tx)
                .await;
            let _ = sqlx::query("DELETE FROM designs WHERE id = ?")
                .bind(dup_id)
                .execute(&mut **tx)
                .await;
        }

        (id, t_insert.elapsed().as_millis())
    } else {
        let t_insert = Instant::now();
        let insert_result = sqlx::query(
            "INSERT INTO designs (filename, filepath, master_filepath, is_master_only, date_added, designer_id, source_id, hoop_id, image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count, is_stitched, image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) VALUES (?, ?, ?, 0, datetime('now'), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)",
        )
        .bind(&filename)
        .bind(stored_filepath)
        .bind(stored_master_filepath)
        .bind(designer_id)
        .bind(source_id)
        .bind(hoop_id)
        .bind(image_result.image_data)
        .bind(image_result.image_type)
        .bind(image_result.width_mm)
        .bind(image_result.height_mm)
        .bind(image_result.stitch_count)
        .bind(image_result.color_count)
        .bind(image_result.color_change_count)
        .bind(file_size_bytes)
        .bind(file_hash_blake3.as_ref())
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        (
            insert_result.last_insert_rowid(),
            t_insert.elapsed().as_millis(),
        )
    };
    *total_db_insert_ms += db_insert_elapsed_ms;

    let t_tag = Instant::now();
    let matched_descriptions = tagging::suggest_path_rule_descriptions(
        &filename,
        stored_filepath,
        ctx.valid_descriptions,
        ctx.tag_synonyms_map,
    );

    let mut stitching_tag_ids: Vec<i64> = Vec::new();
    if Path::new(file_path).exists() {
        let valid_stitching_descriptions: HashSet<String> =
            ctx.stitching_tag_lookup.keys().cloned().collect();
        let detected_stitching_descriptions =
            stitch_identifier::suggest_stitching_from_pattern_file(
                file_path,
                &filename,
                stored_filepath,
                &valid_stitching_descriptions,
                Some(0.70),
            );

        stitching_tag_ids = detected_stitching_descriptions
            .iter()
            .filter_map(|description| ctx.stitching_tag_lookup.get(description).copied())
            .collect();

        if stitching_tag_ids.is_empty() {
            if let Some(default_tag_id) = ctx.default_stitching_tag_id {
                stitching_tag_ids.push(default_tag_id);
            }
        }
    }

    stitching_tag_ids.sort_unstable();
    stitching_tag_ids.dedup();

    for description in &matched_descriptions {
        if let Some(tag_id) = ctx.description_to_tag_id.get(description) {
            sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                .bind(design_id)
                .bind(*tag_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    for tag_id in &stitching_tag_ids {
        sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
            .bind(design_id)
            .bind(*tag_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    *total_tagging_ms += t_tag.elapsed().as_millis();
    Ok(design_id)
}

async fn persist_master_design(
    tx: &mut Transaction<'_, Sqlite>,
    file_path: &str,
    stored_filepath: &str,
    ctx: &ImportExecutionCtx<'_>,
    total_db_insert_ms: &mut u128,
    total_tagging_ms: &mut u128,
) -> Result<i64, String> {
    let (designer_id, source_id) =
        resolve_assignment_for_file(file_path, ctx.confirm_wire, ctx.resolved_assignments);

    let filename = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file_path)
        .to_string();

    let designs_base_path = get_designs_base_path();
    let stored_path = designs_base_path.join(
        stored_filepath
            .strip_prefix("/MachineEmbroideryDesigns/")
            .unwrap_or(stored_filepath),
    );
    let file_size_bytes: Option<i64> = compute_file_size(&stored_path).ok();
    let file_hash_blake3: Option<String> = compute_file_hash_blake3(&stored_path).ok();

    let t_insert = Instant::now();
    let insert_result = sqlx::query(
        "INSERT INTO designs (filename, filepath, master_filepath, is_master_only, date_added, designer_id, source_id, hoop_id, image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count, is_stitched, image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) VALUES (?, ?, ?, 1, datetime('now'), ?, ?, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, ?, ?)",
    )
    .bind(&filename)
    .bind(stored_filepath)
    .bind(stored_filepath)
    .bind(designer_id)
    .bind(source_id)
    .bind(file_size_bytes)
    .bind(file_hash_blake3.as_ref())
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    *total_db_insert_ms += t_insert.elapsed().as_millis();

    let design_id = insert_result.last_insert_rowid();

    let t_tag = Instant::now();
    let matched_descriptions = tagging::suggest_path_rule_descriptions(
        &filename,
        stored_filepath,
        ctx.valid_descriptions,
        ctx.tag_synonyms_map,
    );

    for description in &matched_descriptions {
        if let Some(tag_id) = ctx.description_to_tag_id.get(description) {
            sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                .bind(design_id)
                .bind(*tag_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    *total_tagging_ms += t_tag.elapsed().as_millis();
    Ok(design_id)
}

pub(crate) async fn persist_bulk_import_confirm_wire(
    pool: &SqlitePool,
    confirm_wire: &BulkImportConfirmWire,
    context_token: Option<&str>,
) -> Result<(usize, usize), String> {
    if !confirm_wire.wire.create_on_import {
        return Ok((0, 0));
    }

    let resolved_assignments = resolve_bulk_import_assignments(confirm_wire);
    let preview_3d = false;
    let preview_3d_profile = "balanced";
    let commit_batch_size = DEFAULT_IMPORT_COMMIT_BATCH_SIZE;
    let tag_catalog = load_tag_catalog(pool).await?;
    let valid_descriptions: HashSet<String> = tag_catalog
        .iter()
        .map(|(_, description)| description.clone())
        .collect();
    let description_to_tag_id: HashMap<String, i64> = tag_catalog
        .into_iter()
        .map(|(tag_id, description)| (description, tag_id))
        .collect();
    let tag_synonyms_map = crate::services::tag_synonyms::get_synonym_lookup_map(pool)
        .await
        .unwrap_or_default();
    let stitching_tag_lookup = load_stitching_tag_lookup(pool).await?;
    let default_stitching_tag_id = load_default_stitching_tag_id(pool).await?;
    let master_formats_setting =
        sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ? LIMIT 1")
            .bind(crate::services::settings::KEY_IMPORT_ENABLED_MASTER_FORMATS)
            .fetch_optional(pool)
            .await
            .unwrap_or(None)
            .unwrap_or_default();
    let enabled_master_formats =
        crate::services::settings::parse_enabled_master_formats(&master_formats_setting);

    let ctx = ImportExecutionCtx {
        confirm_wire,
        resolved_assignments: &resolved_assignments,
        valid_descriptions: &valid_descriptions,
        description_to_tag_id: &description_to_tag_id,
        tag_synonyms_map: &tag_synonyms_map,
        stitching_tag_lookup: &stitching_tag_lookup,
        default_stitching_tag_id,
        preview_3d,
        preview_3d_profile,
    };

    let _guard = crate::services::backfill::BackfillRunningGuard::new();
    let total_count = confirm_wire.wire.selected_files.len();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    BULK_IMPORT_STOP_REQUESTED.store(false, Ordering::SeqCst);
    let mut persisted_design_count = 0usize;
    let mut committed_design_count = 0usize;
    let mut failed_decode_count = 0usize;
    let mut persisted_since_last_commit = 0usize;
    let mut processed_count = 0usize;
    let mut stopped = false;

    // Timing accumulators
    let import_start = Instant::now();
    let mut total_image_gen_ms = 0u128;
    let mut total_db_insert_ms = 0u128;
    let mut total_tagging_ms = 0u128;
    let mut total_commit_ms = 0u128;

    let emit_progress = |stage: &str,
                         processed: usize,
                         persisted: usize,
                         committed: usize,
                         failed: usize,
                         current_file: Option<&str>| {
        if let Some(handle) = get_bulk_import_app_handle() {
            let event = BulkImportProgressEvent {
                context_token: context_token.map(String::from),
                stage: stage.to_string(),
                processed_count: processed,
                total_count,
                persisted_count: persisted,
                committed_count: committed,
                failed_count: failed,
                current_file: current_file.map(String::from),
                commit_batch_size,
            };

            if let Err(error) = handle.emit(BULK_IMPORT_PROGRESS_EVENT, event) {
                tracing::error!("Failed to emit bulk import progress event: {error}");
            }
        }
    };

    emit_progress(
        "started",
        processed_count,
        persisted_design_count,
        committed_design_count,
        failed_decode_count,
        None,
    );

    let mut selected_master_map: HashMap<(String, String), String> = HashMap::new();
    for file_path in &confirm_wire.wire.selected_files {
        let path = Path::new(file_path);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let norm_ext = ext.trim_start_matches('.').to_ascii_lowercase();
            if enabled_master_formats.contains(&norm_ext) {
                let parent_key = path
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/").to_ascii_lowercase())
                    .unwrap_or_default();
                let stem_key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                selected_master_map.insert((parent_key, stem_key), file_path.clone());
            }
        }
    }
    let mut consumed_master_files: HashSet<String> = HashSet::new();
    let all_files = confirm_wire.wire.selected_files.clone();
    let mut chunk_start = 0usize;

    while chunk_start < total_count {
        if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
            stopped = true;
            break;
        }

        let chunk_end = (chunk_start + commit_batch_size).min(total_count);
        let chunk = &all_files[chunk_start..chunk_end];

        for file_path in chunk {
            if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
                stopped = true;
                break;
            }

            if consumed_master_files.contains(file_path) {
                processed_count += 1;
                continue;
            }

            let path_obj = Path::new(file_path);
            let ext_str = path_obj.extension().and_then(|e| e.to_str()).unwrap_or("");
            let norm_ext = ext_str.trim_start_matches('.').to_ascii_lowercase();
            let is_master_file = enabled_master_formats.contains(&norm_ext);

            let parent_key = path_obj
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/").to_ascii_lowercase())
                .unwrap_or_default();
            let stem_str = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let stem_key = stem_str.to_ascii_lowercase();

            emit_progress(
                "processing_file",
                processed_count,
                persisted_design_count,
                committed_design_count,
                failed_decode_count,
                Some(file_path),
            );

            if !is_master_file {
                let stored_filepath =
                    ensure_file_in_designs_base(file_path, &confirm_wire.wire.root_paths)?;

                let parent = Path::new(&stored_filepath)
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                let stem = Path::new(&stored_filepath)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let pattern_suffix = if parent.is_empty() {
                    format!("{stem}.%")
                } else {
                    format!("{parent}/{stem}.%")
                };

                let paired_master_src = selected_master_map.get(&(parent_key, stem_key)).cloned();
                let mut stored_master_filepath: Option<String> = if let Some(master_src) =
                    paired_master_src
                {
                    let stored =
                        ensure_file_in_designs_base(&master_src, &confirm_wire.wire.root_paths)?;
                    consumed_master_files.insert(master_src);
                    Some(stored)
                } else {
                    None
                };

                // Check if an existing master-only design exists in SQLite for this stem
                let existing_master_row: Option<(i64, Option<String>)> = if !stem.is_empty() {
                    sqlx::query_as(
                        "SELECT id, master_filepath FROM designs WHERE (filepath LIKE ? OR master_filepath LIKE ?) AND is_master_only = 1 LIMIT 1",
                    )
                    .bind(&pattern_suffix)
                    .bind(&pattern_suffix)
                    .fetch_optional(&mut *tx)
                    .await
                    .unwrap_or(None)
                } else {
                    None
                };

                let existing_design_id = if let Some((id, existing_mf)) = existing_master_row {
                    if stored_master_filepath.is_none() {
                        stored_master_filepath = existing_mf;
                    }
                    Some(id)
                } else {
                    None
                };

                // Fallback: Check if matching master file exists on disk
                if stored_master_filepath.is_none() && !stem.is_empty() {
                    for master_ext in &enabled_master_formats {
                        let candidate_src = path_obj.with_extension(master_ext);
                        if candidate_src.exists() {
                            if let Ok(stored) = ensure_file_in_designs_base(
                                &candidate_src.to_string_lossy(),
                                &confirm_wire.wire.root_paths,
                            ) {
                                stored_master_filepath = Some(stored);
                                break;
                            }
                        }
                        let designs_base = get_designs_base_path();
                        let candidate_dest = if parent.is_empty() {
                            designs_base.join(format!("{stem}.{master_ext}"))
                        } else {
                            designs_base
                                .join(&parent)
                                .join(format!("{stem}.{master_ext}"))
                        };
                        if candidate_dest.exists() {
                            let rel = if parent.is_empty() {
                                format!("{stem}.{master_ext}")
                            } else {
                                format!("{parent}/{stem}.{master_ext}")
                            };
                            stored_master_filepath = Some(crate::paths::canonical_design_rel(&rel));
                            break;
                        }
                    }
                }

                let _ = persist_stitch_design(
                    &mut tx,
                    file_path,
                    &stored_filepath,
                    stored_master_filepath.as_deref(),
                    existing_design_id,
                    &ctx,
                    &mut total_image_gen_ms,
                    &mut total_db_insert_ms,
                    &mut total_tagging_ms,
                    &mut failed_decode_count,
                )
                .await?;
            } else {
                let stored_filepath =
                    ensure_file_in_designs_base(file_path, &confirm_wire.wire.root_paths)?;

                let parent = Path::new(&stored_filepath)
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                let stem = Path::new(&stored_filepath)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let pattern_suffix = if parent.is_empty() {
                    format!("{stem}.%")
                } else {
                    format!("{parent}/{stem}.%")
                };

                let existing_stitch_id: Option<i64> = if !stem.is_empty() {
                    sqlx::query_scalar(
                        "SELECT id FROM designs WHERE (filepath LIKE ? OR filepath LIKE ?) AND is_master_only = 0 AND (master_filepath IS NULL OR master_filepath = '') LIMIT 1",
                    )
                    .bind(&pattern_suffix)
                    .bind(format!("/MachineEmbroideryDesigns/{}", pattern_suffix))
                    .fetch_optional(&mut *tx)
                    .await
                    .unwrap_or(None)
                } else {
                    None
                };

                if let Some(stitch_id) = existing_stitch_id {
                    let t_upd = Instant::now();
                    sqlx::query("UPDATE designs SET master_filepath = ? WHERE id = ?")
                        .bind(&stored_filepath)
                        .bind(stitch_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    total_db_insert_ms += t_upd.elapsed().as_millis();
                } else {
                    let _ = persist_master_design(
                        &mut tx,
                        file_path,
                        &stored_filepath,
                        &ctx,
                        &mut total_db_insert_ms,
                        &mut total_tagging_ms,
                    )
                    .await?;
                }
            }

            persisted_design_count += 1;
            persisted_since_last_commit += 1;
            processed_count += 1;

            emit_progress(
                "processed",
                processed_count,
                persisted_design_count,
                committed_design_count,
                failed_decode_count,
                Some(file_path),
            );
        }

        let t_commit = Instant::now();
        tx.commit().await.map_err(|e| e.to_string())?;
        let commit_ms = t_commit.elapsed().as_millis();
        total_commit_ms += commit_ms;
        committed_design_count += persisted_since_last_commit;
        if persisted_since_last_commit > 0 {
            emit_progress(
                "batch_committed",
                processed_count,
                persisted_design_count,
                committed_design_count,
                failed_decode_count,
                None,
            );
        }
        tx = pool.begin().await.map_err(|e| e.to_string())?;
        persisted_since_last_commit = 0;

        if stopped {
            break;
        }
        chunk_start = chunk_end;
    }

    let t_final_commit = Instant::now();
    tx.commit().await.map_err(|e| e.to_string())?;
    total_commit_ms += t_final_commit.elapsed().as_millis();
    committed_design_count += persisted_since_last_commit;

    let total_elapsed_ms = import_start.elapsed().as_millis();
    tracing::info!(
        "[TIMING] Bulk import complete: total={}ms | image_gen={}ms | db_insert={}ms | tagging={}ms | commits={}ms | persisted={} skipped={}",
        total_elapsed_ms,
        total_image_gen_ms,
        total_db_insert_ms,
        total_tagging_ms,
        total_commit_ms,
        persisted_design_count,
        processed_count.saturating_sub(persisted_design_count),
    );

    let final_stage = if stopped { "stopped" } else { "completed" };
    emit_progress(
        final_stage,
        processed_count,
        persisted_design_count,
        committed_design_count,
        failed_decode_count,
        None,
    );
    Ok((persisted_design_count, failed_decode_count))
}
