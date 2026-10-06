// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::paths::normalize_path_for_match;
use super::types::{
    AssignmentFieldSourceWire, BulkImportConfirmWire, BulkImportWire, FolderAssignmentWire,
    ResolvedAssignmentFieldWire, ResolvedFolderAssignmentWire,
};
use crate::services::scanning;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn normalize_name_for_import_matching(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(['_', '-', '/', '\\'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn compact_name_for_import_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

pub(crate) fn strip_web_affixes_for_import_matching(value: &str) -> String {
    let mut compact = compact_name_for_import_matching(value);

    if let Some(stripped) = compact.strip_prefix("www") {
        compact = stripped.to_string();
    }

    for suffix in ["comau", "couk", "com", "net", "org", "co", "uk"] {
        if compact.len() > suffix.len() + 2 && compact.ends_with(suffix) {
            compact.truncate(compact.len() - suffix.len());
            break;
        }
    }

    compact
}

pub(crate) fn suggest_reference_id_from_path(
    path_value: &str,
    items: &[(i64, String)],
) -> Option<i64> {
    let normalized_path = normalize_name_for_import_matching(path_value);
    let compact_path = compact_name_for_import_matching(path_value);
    if normalized_path.is_empty() && compact_path.is_empty() {
        return None;
    }

    for (item_id, item_name) in items {
        let raw_name = item_name.trim();
        if raw_name.is_empty() {
            continue;
        }

        let lowered = raw_name.to_ascii_lowercase();
        if lowered == "don't know" || lowered == "me" {
            continue;
        }

        let normalized_name = normalize_name_for_import_matching(raw_name);
        let compact_name = compact_name_for_import_matching(raw_name);
        let stripped_compact_name = strip_web_affixes_for_import_matching(raw_name);
        if (!normalized_name.is_empty() && normalized_path.contains(&normalized_name))
            || (!compact_name.is_empty() && compact_path.contains(&compact_name))
            || (!stripped_compact_name.is_empty() && compact_path.contains(&stripped_compact_name))
        {
            return Some(*item_id);
        }
    }

    None
}

pub(crate) fn infer_assignment_ids_from_folder_path(
    folder_path: &str,
    designers: &[(i64, String)],
    sources: &[(i64, String)],
) -> (Option<i64>, Option<i64>) {
    (
        suggest_reference_id_from_path(folder_path, designers),
        suggest_reference_id_from_path(folder_path, sources),
    )
}

pub(crate) fn folder_path_from_file_path(file_path: &str) -> Option<String> {
    let path_text = file_path.trim();
    if path_text.is_empty() {
        return None;
    }

    Path::new(path_text)
        .parent()
        .map(|parent| parent.to_string_lossy().trim().to_string())
        .filter(|parent| !parent.is_empty())
}

pub(crate) fn build_preview_folder_assignments(
    wire: &BulkImportWire,
    scanned_files: &[scanning::ScannedFile],
) -> Vec<FolderAssignmentWire> {
    let mut assignments_by_path = HashMap::<String, FolderAssignmentWire>::new();

    for assignment in &wire.per_folder_assignments {
        assignments_by_path.insert(
            normalize_path_for_match(&assignment.folder_path),
            assignment.clone(),
        );
    }

    for scanned_file in scanned_files {
        if let Some(folder_path) = folder_path_from_file_path(&scanned_file.full_path) {
            let normalized_folder = normalize_path_for_match(&folder_path);
            assignments_by_path
                .entry(normalized_folder)
                .or_insert_with(|| FolderAssignmentWire {
                    folder_path,
                    designer_id: None,
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: None,
                });
        }
    }

    let mut assignments = assignments_by_path
        .into_values()
        .collect::<Vec<FolderAssignmentWire>>();
    assignments.sort_by(|left, right| {
        left.folder_path
            .to_ascii_lowercase()
            .cmp(&right.folder_path.to_ascii_lowercase())
    });
    assignments
}

pub(crate) async fn load_designers_for_import_inference(
    pool: &SqlitePool,
) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM designers ORDER BY LENGTH(name) DESC, name ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

pub(crate) async fn load_sources_for_import_inference(
    pool: &SqlitePool,
) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM sources ORDER BY LENGTH(name) DESC, name ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

pub(crate) fn resolve_assignment_for_file(
    file_path: &str,
    confirm_wire: &BulkImportConfirmWire,
    resolved_assignments: &[ResolvedFolderAssignmentWire],
) -> (Option<i64>, Option<i64>) {
    let normalized_file = normalize_path_for_match(file_path);

    let mut best_match: Option<(&ResolvedFolderAssignmentWire, usize)> = None;
    for assignment in resolved_assignments {
        let normalized_folder = normalize_path_for_match(&assignment.folder_path);
        if normalized_file.starts_with(&normalized_folder) {
            let score = normalized_folder.len();
            if best_match
                .map(|(_, best_score)| score > best_score)
                .unwrap_or(true)
            {
                best_match = Some((assignment, score));
            }
        }
    }

    if let Some((assignment, _)) = best_match {
        return (assignment.designer_id.value, assignment.source_id.value);
    }

    (
        confirm_wire.wire.global_designer_id,
        confirm_wire.wire.global_source_id,
    )
}

pub fn resolve_assignment_field(
    explicit_value: Option<i64>,
    global_value: Option<i64>,
    inferred_value: Option<i64>,
) -> ResolvedAssignmentFieldWire {
    if let Some(value) = explicit_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::ExplicitPerFolder,
        };
    }

    if let Some(value) = global_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::Global,
        };
    }

    if let Some(value) = inferred_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::Inferred,
        };
    }

    ResolvedAssignmentFieldWire {
        value: None,
        source: AssignmentFieldSourceWire::Blank,
    }
}

pub fn resolve_folder_assignment_wire(
    assignment: &FolderAssignmentWire,
    wire: &BulkImportWire,
) -> ResolvedFolderAssignmentWire {
    ResolvedFolderAssignmentWire {
        folder_path: assignment.folder_path.clone(),
        designer_id: resolve_assignment_field(
            assignment.designer_id,
            wire.global_designer_id,
            assignment.inferred_designer_id,
        ),
        source_id: resolve_assignment_field(
            assignment.source_id,
            wire.global_source_id,
            assignment.inferred_source_id,
        ),
        inferred_designer_id: assignment.inferred_designer_id,
        inferred_source_id: assignment.inferred_source_id,
    }
}

pub fn resolve_bulk_import_assignments(
    confirm_wire: &BulkImportConfirmWire,
) -> Vec<ResolvedFolderAssignmentWire> {
    confirm_wire
        .wire
        .per_folder_assignments
        .iter()
        .map(|assignment| resolve_folder_assignment_wire(assignment, &confirm_wire.wire))
        .collect()
}
