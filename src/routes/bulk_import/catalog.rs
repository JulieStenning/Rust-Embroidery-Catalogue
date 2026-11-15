// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::paths::{
    compute_file_hash_blake3, compute_prospective_stored_filepath, normalize_path_for_match,
};
use super::session::get_bulk_import_db_pool;
use crate::services::scanning;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) async fn load_catalog_counts(pool: &SqlitePool) -> Result<(i64, i64), String> {
    let design_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM designs")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let hoop_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hoops")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok((design_count, hoop_count))
}

pub(crate) async fn load_tag_catalog(pool: &SqlitePool) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>("SELECT id, description FROM tags ORDER BY id ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn load_stitching_tag_lookup(
    pool: &SqlitePool,
) -> Result<HashMap<String, i64>, String> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, description FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, description)| (description, id))
        .collect())
}

pub(crate) async fn load_default_stitching_tag_id(
    pool: &SqlitePool,
) -> Result<Option<i64>, String> {
    sqlx::query_scalar(
        "SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching' ORDER BY description ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) fn load_import_precheck_state_if_initialized() -> Result<(bool, bool), String> {
    let Some(pool) = get_bulk_import_db_pool() else {
        return Ok((false, false));
    };

    let (design_count, hoop_count) = tauri::async_runtime::block_on(load_catalog_counts(&pool))?;
    let is_first_import = design_count == 0;
    let needs_hoop_setup = is_first_import && hoop_count == 0;
    Ok((is_first_import, needs_hoop_setup))
}

pub(crate) async fn load_import_precheck_state_if_initialized_async() -> Result<(bool, bool), String>
{
    let Some(pool) = get_bulk_import_db_pool() else {
        return Ok((false, false));
    };

    let (design_count, hoop_count) = load_catalog_counts(&pool).await?;
    let is_first_import = design_count == 0;
    let needs_hoop_setup = is_first_import && hoop_count == 0;
    Ok((is_first_import, needs_hoop_setup))
}

pub(crate) async fn filter_existing_scanned_files(
    pool: &SqlitePool,
    scanned_files: Vec<scanning::ScannedFile>,
    root_paths: &[String],
) -> Result<Vec<scanning::ScannedFile>, String> {
    if scanned_files.is_empty() {
        return Ok(scanned_files);
    }

    let existing_paths = sqlx::query_scalar::<_, String>("SELECT filepath FROM designs")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let existing_master_paths = sqlx::query_scalar::<_, Option<String>>(
        "SELECT master_filepath FROM designs WHERE master_filepath IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut existing_path_set: HashSet<String> = existing_paths
        .into_iter()
        .map(|path| normalize_path_for_match(&path))
        .collect();

    for master_path in existing_master_paths.into_iter().flatten() {
        existing_path_set.insert(normalize_path_for_match(&master_path));
    }

    let mut path_survivors: Vec<scanning::ScannedFile> = Vec::with_capacity(scanned_files.len());
    let mut excluded_by_path: usize = 0;

    for file in scanned_files {
        let prospective_path = compute_prospective_stored_filepath(&file.full_path, root_paths)
            .unwrap_or_else(|_| format!("/MachineEmbroideryDesigns/{}", file.full_path));

        let normalized_prospective = normalize_path_for_match(&prospective_path);

        if existing_path_set.contains(&normalized_prospective) {
            excluded_by_path += 1;
        } else {
            existing_path_set.insert(normalized_prospective);
            path_survivors.push(file);
        }
    }

    if path_survivors.is_empty() {
        if excluded_by_path > 0 {
            tracing::debug!(
                "Preview dedup: excluded_by_path={} excluded_by_triple=0 imported=0",
                excluded_by_path
            );
        }
        return Ok(Vec::new());
    }

    let mut batch_pair_counts: HashMap<(String, i64), usize> = HashMap::new();
    let mut candidate_sizes: HashSet<i64> = HashSet::new();
    for file in &path_survivors {
        if let Some(size) = file.file_size_bytes {
            candidate_sizes.insert(size);
            *batch_pair_counts
                .entry((file.filename.to_ascii_lowercase(), size))
                .or_insert(0) += 1;
        }
    }

    let db_fingerprints: HashMap<(String, i64), Vec<String>> = if candidate_sizes.is_empty() {
        HashMap::new()
    } else if candidate_sizes.len() <= 500 {
        let sizes_vec: Vec<i64> = candidate_sizes.into_iter().collect();
        let mut map: HashMap<(String, i64), Vec<String>> = HashMap::new();
        for chunk in sizes_vec.chunks(200) {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "SELECT filename, file_size_bytes, file_hash_blake3 FROM designs WHERE file_size_bytes IN (",
            );
            let mut separated = builder.separated(", ");
            for size in chunk {
                separated.push_bind(*size);
            }
            separated.push_unseparated(") AND file_hash_blake3 IS NOT NULL");

            let rows: Vec<(String, i64, String)> = builder
                .build_query_as()
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

            for (filename, size, hash) in rows {
                map.entry((filename.to_ascii_lowercase(), size))
                    .or_default()
                    .push(hash.to_ascii_lowercase());
            }
        }
        map
    } else {
        let fingerprint_rows: Vec<(String, i64, String)> = sqlx::query_as(
            "SELECT filename, file_size_bytes, file_hash_blake3 FROM designs WHERE file_size_bytes IS NOT NULL AND file_hash_blake3 IS NOT NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut map: HashMap<(String, i64), Vec<String>> = HashMap::new();
        for (filename, size, hash) in fingerprint_rows {
            map.entry((filename.to_ascii_lowercase(), size))
                .or_default()
                .push(hash.to_ascii_lowercase());
        }
        map
    };

    let mut result: Vec<scanning::ScannedFile> = Vec::with_capacity(path_survivors.len());
    let mut seen_in_batch: HashMap<(String, i64), Vec<String>> = HashMap::new();
    let mut excluded_by_triple: usize = 0;

    for file in path_survivors {
        let filename_lower = file.filename.to_ascii_lowercase();
        let file_size = match file.file_size_bytes {
            Some(size) => size,
            None => {
                result.push(file);
                continue;
            }
        };

        let pair_key = (filename_lower, file_size);
        let in_db = db_fingerprints.get(&pair_key);
        let in_batch = seen_in_batch.get(&pair_key);
        let has_batch_multiples = batch_pair_counts.get(&pair_key).copied().unwrap_or(0) > 1;

        if in_db.is_none() && !has_batch_multiples {
            result.push(file);
            continue;
        }

        let source_path = Path::new(&file.full_path);
        if !source_path.exists() {
            result.push(file);
            continue;
        }

        let file_hash = match compute_file_hash_blake3(source_path) {
            Ok(hash) => hash.to_ascii_lowercase(),
            Err(_) => {
                result.push(file);
                continue;
            }
        };

        let matches_db = in_db.is_some_and(|hashes| hashes.contains(&file_hash));
        let matches_batch = in_batch.is_some_and(|hashes| hashes.contains(&file_hash));

        if matches_db || matches_batch {
            excluded_by_triple += 1;
            continue;
        }

        seen_in_batch.entry(pair_key).or_default().push(file_hash);
        result.push(file);
    }

    if excluded_by_path > 0 || excluded_by_triple > 0 {
        tracing::debug!(
            "Preview dedup: excluded_by_path={} excluded_by_triple={} imported={}",
            excluded_by_path,
            excluded_by_triple,
            result.len()
        );
    }

    Ok(result)
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogueStemEntry {
    pub id: i64,
    pub is_master_only: bool,
    pub master_filepath: Option<String>,
}

pub(crate) type CatalogueStemMap = HashMap<(String, String), CatalogueStemEntry>;

pub(crate) fn extract_stem_key(filepath: &str) -> (String, String) {
    let norm = filepath.replace('\\', "/");
    let trimmed = norm.strip_prefix('/').unwrap_or(&norm);
    let stripped = trimmed
        .strip_prefix("MachineEmbroideryDesigns/")
        .unwrap_or(trimmed);
    let p = Path::new(stripped);
    let parent = p
        .parent()
        .map(|d| d.to_string_lossy().replace('\\', "/").to_ascii_lowercase())
        .unwrap_or_default();
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    (parent, stem)
}

pub(crate) async fn load_catalogue_stem_index(
    pool: &SqlitePool,
) -> Result<CatalogueStemMap, String> {
    let rows: Vec<(i64, String, Option<String>, i64)> =
        sqlx::query_as("SELECT id, filepath, master_filepath, is_master_only FROM designs")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    let mut map: CatalogueStemMap = HashMap::with_capacity(rows.len());
    for (id, filepath, master_filepath, is_master_only_int) in rows {
        let is_master_only = is_master_only_int != 0;
        let entry = CatalogueStemEntry {
            id,
            is_master_only,
            master_filepath: master_filepath.clone(),
        };

        let primary_key = extract_stem_key(&filepath);
        if !primary_key.1.is_empty() {
            map.insert(primary_key, entry.clone());
        }

        if let Some(ref mf) = master_filepath {
            let master_key = extract_stem_key(mf);
            if !master_key.1.is_empty() {
                map.entry(master_key).or_insert(entry);
            }
        }
    }

    Ok(map)
}
