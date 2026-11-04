// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::metadata::{ensure_design_exists, ensure_foreign_key_exists};
use super::types::{
    BrowseTagOption, BulkApplyTagsRequest, BulkSetTagsResult, DesignCommandResult,
    SetDesignTagsRequest,
};
use crate::AppState;
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
use tauri::{Emitter, State};

/// Classify a set of tag ids into (image tag ids, stitching tag ids) by
/// consulting the `tags.tag_group` column.
pub(crate) async fn classify_tag_ids(
    pool: &SqlitePool,
    tag_ids: &[i64],
) -> Result<(Vec<i64>, Vec<i64>), String> {
    use sqlx::Row;
    let mut image_ids = Vec::new();
    let mut stitching_ids = Vec::new();

    for tag_id in tag_ids {
        let row = sqlx::query("SELECT tag_group FROM tags WHERE id = ? LIMIT 1")
            .bind(*tag_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let group = row
            .and_then(|record| {
                record
                    .try_get::<Option<String>, _>("tag_group")
                    .ok()
                    .flatten()
            })
            .unwrap_or_default();
        if group.eq_ignore_ascii_case("stitching") {
            stitching_ids.push(*tag_id);
        } else {
            image_ids.push(*tag_id);
        }
    }

    Ok((image_ids, stitching_ids))
}

pub(crate) async fn set_design_tags_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tag_ids: Vec<i64>,
    request_image: Option<bool>,
    request_stitching: Option<bool>,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    let mut deduped = Vec::<i64>::new();
    for id in tag_ids {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !deduped.contains(&id) {
            deduped.push(id);
        }
    }

    for tag_id in &deduped {
        ensure_foreign_key_exists(pool, "tags", Some(*tag_id), "Tag").await?;
    }

    // Capture the previous image/stitching tag ids so a full-replace can
    // detect whether each domain actually changed.
    fn tag_ids_in_group(rows: &[sqlx::sqlite::SqliteRow], group: &str) -> Vec<i64> {
        use sqlx::Row;
        rows.iter()
            .filter(|row| {
                let g = row
                    .try_get::<Option<String>, _>("tag_group")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                g.eq_ignore_ascii_case(group)
            })
            .filter_map(|row| row.try_get::<i64, _>("id").ok())
            .collect()
    }

    let existing_rows = sqlx::query(
        "SELECT t.id AS id, t.tag_group AS tag_group
		 FROM tags t
		 INNER JOIN design_tags dt ON dt.tag_id = t.id
		 WHERE dt.design_id = ?",
    )
    .bind(design_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let existing_image = tag_ids_in_group(&existing_rows, "image");
    let existing_stitching = tag_ids_in_group(&existing_rows, "stitching");

    let (new_image, new_stitching) = classify_tag_ids(pool, &deduped).await?;
    let image_changed = existing_image != new_image;
    let stitching_changed = existing_stitching != new_stitching;

    // Determine the resolved verification values. Explicit request flags win;
    // otherwise any change to a domain marks it verified; unchanged domains
    // stay completely untouched.
    let resolved_image = match request_image {
        Some(value) => Some(value),
        None if image_changed => Some(true),
        None => None,
    };
    let resolved_stitching = match request_stitching {
        Some(value) => Some(value),
        None if stitching_changed => Some(true),
        None => None,
    };

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
        .bind(design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for tag_id in &deduped {
        sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
            .bind(design_id)
            .bind(*tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(value) = resolved_image {
        sqlx::query("UPDATE designs SET image_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(value) = resolved_stitching {
        sqlx::query("UPDATE designs SET stitching_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    sqlx::query("UPDATE designs SET date_updated = datetime('now') WHERE id = ?")
        .bind(design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design tags updated and marked as verified.".to_string(),
    })
}

pub(crate) async fn remove_design_tag_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tag_id: i64,
) -> Result<DesignCommandResult, String> {
    if tag_id <= 0 {
        return Err("Tag id must be a positive integer.".to_string());
    }

    ensure_design_exists(pool, design_id).await?;
    ensure_foreign_key_exists(pool, "tags", Some(tag_id), "Tag").await?;

    let tag_group =
        sqlx::query_scalar::<_, Option<String>>("SELECT tag_group FROM tags WHERE id = ?")
            .bind(tag_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .flatten()
            .unwrap_or_default();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let result = sqlx::query("DELETE FROM design_tags WHERE design_id = ? AND tag_id = ?")
        .bind(design_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() > 0 {
        let col = if tag_group.eq_ignore_ascii_case("stitching") {
            "stitching_tags_verified"
        } else {
            "image_tags_verified"
        };
        let query =
            format!("UPDATE designs SET {col} = 0, date_updated = datetime('now') WHERE id = ?");
        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Tag removed from design.".to_string(),
    })
}

pub(crate) async fn bulk_set_tags_for_designs_with_pool(
    pool: &SqlitePool,
    design_ids: &[i64],
    request: BulkApplyTagsRequest,
) -> Result<BulkSetTagsResult, String> {
    if design_ids.is_empty() {
        return Ok(BulkSetTagsResult {
            requested_count: 0,
            updated_count: 0,
        });
    }

    // Deduplicate design IDs while preserving the requested total count
    let mut deduped_designs = Vec::<i64>::with_capacity(design_ids.len());
    for id in design_ids {
        if *id > 0 && !deduped_designs.contains(id) {
            deduped_designs.push(*id);
        }
    }

    if deduped_designs.is_empty() {
        return Ok(BulkSetTagsResult {
            requested_count: design_ids.len(),
            updated_count: 0,
        });
    }

    // Deduplicate and validate all tag ids; add wins over remove when both
    // reference the same tag.
    let mut tags_to_add = Vec::<i64>::new();
    for id in request.tags_to_add {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !tags_to_add.contains(&id) {
            tags_to_add.push(id);
        }
    }

    let mut tags_to_remove = Vec::<i64>::new();
    for id in request.tags_to_remove {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !tags_to_remove.contains(&id) {
            tags_to_remove.push(id);
        }
    }

    for tag_id in tags_to_add.iter().chain(tags_to_remove.iter()) {
        ensure_foreign_key_exists(pool, "tags", Some(*tag_id), "Tag").await?;
    }

    // Verification flags are optional. `None` means "leave this design's flag
    // exactly as it is" — we must NOT clear a prior verified status when a
    // category is left untouched (mixed/indeterminate category preserved).
    // Explicit `Some(value)` always wins and is written regardless of tag diff.
    let resolved_image = request.image_tags_verified;
    let resolved_stitching = request.stitching_tags_verified;
    let has_explicit_flags = resolved_image.is_some() || resolved_stitching.is_some();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut total_updated_count = 0usize;

    // Process design IDs in bounded chunks of 250 to keep query parameter
    // counts, lock hold times, and memory allocations predictably small.
    const CHUNK_SIZE: usize = 250;
    for chunk in deduped_designs.chunks(CHUNK_SIZE) {
        let mut chunk_affected = HashSet::<i64>::new();

        // 1. Tag Deletion Phase
        if request.clear_all_tags {
            let mut delete_query =
                QueryBuilder::<Sqlite>::new("DELETE FROM design_tags WHERE design_id IN (");
            let mut separated = delete_query.separated(", ");
            for id in chunk {
                separated.push_bind(*id);
            }
            delete_query.push(") RETURNING design_id");

            let deleted_ids: Vec<i64> = delete_query
                .build_query_scalar::<i64>()
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            chunk_affected.extend(deleted_ids);
        } else if !tags_to_remove.is_empty() {
            let mut delete_query =
                QueryBuilder::<Sqlite>::new("DELETE FROM design_tags WHERE design_id IN (");
            let mut separated_designs = delete_query.separated(", ");
            for id in chunk {
                separated_designs.push_bind(*id);
            }
            delete_query.push(") AND tag_id IN (");
            let mut separated_tags = delete_query.separated(", ");
            for tag_id in &tags_to_remove {
                separated_tags.push_bind(*tag_id);
            }
            delete_query.push(") RETURNING design_id");

            let deleted_ids: Vec<i64> = delete_query
                .build_query_scalar::<i64>()
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            chunk_affected.extend(deleted_ids);
        }

        // 2. Tag Insertion Phase
        if !tags_to_add.is_empty() {
            let pairs: Vec<(i64, i64)> = chunk
                .iter()
                .flat_map(|d| tags_to_add.iter().map(move |t| (*d, *t)))
                .collect();

            // Insert in slices of at most 250 pairs (500 bind variables)
            for pair_chunk in pairs.chunks(CHUNK_SIZE) {
                let mut insert_query = QueryBuilder::<Sqlite>::new(
                    "INSERT OR IGNORE INTO design_tags (design_id, tag_id) ",
                );
                insert_query.push_values(pair_chunk, |mut b, (d, t)| {
                    b.push_bind(*d).push_bind(*t);
                });
                insert_query.push(" RETURNING design_id");

                let inserted_ids: Vec<i64> = insert_query
                    .build_query_scalar::<i64>()
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                chunk_affected.extend(inserted_ids);
            }
        }

        // 3. Verification Flags & Timestamp Update Phase
        if has_explicit_flags {
            // Explicit flags override every design in the chunk regardless of tag diff
            let mut update_query = QueryBuilder::<Sqlite>::new("UPDATE designs SET ");
            if let Some(img_val) = resolved_image {
                update_query.push("image_tags_verified = ");
                update_query.push_bind(img_val);
                update_query.push(", ");
            }
            if let Some(stk_val) = resolved_stitching {
                update_query.push("stitching_tags_verified = ");
                update_query.push_bind(stk_val);
                update_query.push(", ");
            }
            update_query.push("date_updated = datetime('now') WHERE id IN (");
            let mut separated_ids = update_query.separated(", ");
            for id in chunk {
                separated_ids.push_bind(*id);
            }
            update_query.push(")");

            update_query
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            total_updated_count += chunk.len();
        } else if !chunk_affected.is_empty() {
            // Only designs with actual tag modifications get updated
            let mut update_query = QueryBuilder::<Sqlite>::new(
                "UPDATE designs SET date_updated = datetime('now') WHERE id IN (",
            );
            let mut separated_ids = update_query.separated(", ");
            for id in &chunk_affected {
                separated_ids.push_bind(*id);
            }
            update_query.push(")");

            update_query
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            total_updated_count += chunk_affected.len();
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BulkSetTagsResult {
        requested_count: design_ids.len(),
        updated_count: total_updated_count,
    })
}

#[tauri::command]
pub async fn get_tags_for_browse(
    state: State<'_, AppState>,
) -> Result<Vec<BrowseTagOption>, String> {
    sqlx::query_as::<_, BrowseTagOption>(
        r#"
		SELECT
			t.id AS id,
			t.description AS description,
			t.tag_group AS tag_group
		FROM tags t
		ORDER BY t.description COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(&state.db_pool()?)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_set_tags_for_designs(
    state: State<'_, AppState>,
    design_ids: Vec<i64>,
    request: BulkApplyTagsRequest,
) -> Result<BulkSetTagsResult, String> {
    bulk_set_tags_for_designs_with_pool(&state.db_pool()?, &design_ids, request).await
}

#[tauri::command]
pub async fn set_design_tags(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_tags_with_pool(
        &state.db_pool()?,
        design_id,
        request.tag_ids,
        request.image_tags_verified,
        request.stitching_tags_verified,
    )
    .await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {
                "image_tags_verified": request.image_tags_verified,
                "stitching_tags_verified": request.stitching_tags_verified,
            }
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn remove_design_tag(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    tag_id: i64,
) -> Result<DesignCommandResult, String> {
    let result = remove_design_tag_with_pool(&state.db_pool()?, design_id, tag_id).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {}
        }),
    );
    Ok(result)
}
