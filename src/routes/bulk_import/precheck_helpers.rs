// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::inference::resolve_bulk_import_assignments;
use super::paths::folder_key_from_full_path;
use super::types::{
    AssignmentFieldSourceWire, BulkImportAssignmentResolutionSummary, BulkImportConfirmSummary,
    BulkImportConfirmWire, BulkImportSelectionWire, BulkImportWire, BulkImportWireSummary,
};
use crate::services::scanning;
use std::collections::{HashMap, HashSet};

pub(crate) fn selection_sets(
    selection: &BulkImportSelectionWire,
) -> (
    HashMap<String, HashSet<String>>,
    HashMap<String, HashSet<String>>,
) {
    let mut deselected: HashMap<String, HashSet<String>> = HashMap::new();
    let mut selected_only: HashMap<String, HashSet<String>> = HashMap::new();
    for entry in &selection.deselected {
        deselected.insert(
            entry.folder_path.clone(),
            entry.files.iter().cloned().collect(),
        );
    }
    for entry in &selection.selected_only {
        selected_only.insert(
            entry.folder_path.clone(),
            entry.files.iter().cloned().collect(),
        );
    }
    (deselected, selected_only)
}

pub(crate) fn selected_files_from_scan(
    scanned_files: &[scanning::ScannedFile],
    selection: &BulkImportSelectionWire,
) -> Vec<String> {
    let (deselected, selected_only) = selection_sets(selection);
    let mut result = Vec::with_capacity(scanned_files.len());
    for file in scanned_files {
        let folder = folder_key_from_full_path(&file.full_path);
        let selected = if let Some(only) = selected_only.get(&folder) {
            only.contains(&file.full_path)
        } else if let Some(deselected) = deselected.get(&folder) {
            !deselected.contains(&file.full_path)
        } else {
            true
        };
        if selected {
            result.push(file.full_path.clone());
        }
    }
    result
}

pub(crate) fn count_selected_in_folder(selected_files: &[String], folder_path: &str) -> usize {
    selected_files
        .iter()
        .filter(|path| folder_key_from_full_path(path) == folder_path)
        .count()
}

#[tauri::command]
pub fn debug_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportWireSummary, String> {
    Ok(BulkImportWireSummary {
        root_path_count: wire.root_paths.len(),
        folder_assignment_count: wire.per_folder_assignments.len(),
        selected_file_count: wire.selected_files.len(),
        create_on_import: wire.create_on_import,
    })
}

#[tauri::command]
pub fn debug_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmSummary, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    Ok(BulkImportConfirmSummary {
        context_token_present: confirm_wire.context_token.is_some(),
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        per_folder_assignment_count: confirm_wire.wire.per_folder_assignments.len(),
        canonical_confirm: confirm_wire.canonical_confirm,
        resolved_assignment_count: resolved_assignments.len(),
        resolved_assignments,
    })
}

#[tauri::command]
pub fn debug_bulk_import_assignment_resolution_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportAssignmentResolutionSummary, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);

    let mut explicit_field_count = 0usize;
    let mut global_field_count = 0usize;
    let mut inferred_field_count = 0usize;
    let mut blank_field_count = 0usize;

    for assignment in &resolved_assignments {
        for field in [&assignment.designer_id, &assignment.source_id] {
            match field.source {
                AssignmentFieldSourceWire::ExplicitPerFolder => explicit_field_count += 1,
                AssignmentFieldSourceWire::Global => global_field_count += 1,
                AssignmentFieldSourceWire::Inferred => inferred_field_count += 1,
                AssignmentFieldSourceWire::Blank => blank_field_count += 1,
            }
        }
    }

    Ok(BulkImportAssignmentResolutionSummary {
        resolved_count: resolved_assignments.len(),
        explicit_field_count,
        global_field_count,
        inferred_field_count,
        blank_field_count,
    })
}
