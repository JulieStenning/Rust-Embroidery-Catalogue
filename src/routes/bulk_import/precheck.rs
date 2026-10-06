// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::catalog::{
    filter_existing_scanned_files, load_import_precheck_state_if_initialized,
    load_import_precheck_state_if_initialized_async,
};
use super::commands::do_confirm_bulk_import_wire_internal;
use super::inference::{
    build_preview_folder_assignments, infer_assignment_ids_from_folder_path,
    load_designers_for_import_inference, load_sources_for_import_inference,
    resolve_bulk_import_assignments, resolve_folder_assignment_wire,
};
use super::precheck_helpers::{count_selected_in_folder, selected_files_from_scan};
use super::session::{
    get_bulk_import_context, get_bulk_import_db_pool, store_bulk_import_context,
    store_bulk_import_scan, take_bulk_import_context, take_bulk_import_scan,
};
use super::types::{
    BulkImportBrowseFolderRequest, BulkImportBrowseFolderResult, BulkImportConfirmWire,
    BulkImportPrecheckActionRequest, BulkImportPrecheckActionResult, BulkImportPrecheckActionWire,
    BulkImportPrecheckFromScanRequest, BulkImportPrecheckResult, BulkImportPreview,
    BulkImportRequest, BulkImportWire, FolderAssignmentWire,
};
use crate::services::{folder_picker, scanning, validation};
use sqlx::SqlitePool;

pub(crate) fn preview_bulk_import_wire_with_pool(
    wire: BulkImportWire,
    pool: Option<&SqlitePool>,
) -> Result<BulkImportPreview, String> {
    for root_path in &wire.root_paths {
        validation::validate_path(root_path).map_err(|e| format!("{:?}", e))?;
    }

    let master_extensions: Vec<String> = if let Some(active_pool) = pool {
        let setting_val = tauri::async_runtime::block_on(
            sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ? LIMIT 1")
                .bind(crate::services::settings::KEY_IMPORT_ENABLED_MASTER_FORMATS)
                .fetch_optional(active_pool),
        )
        .unwrap_or(None)
        .unwrap_or_default();
        crate::services::settings::parse_enabled_master_formats(&setting_val)
            .into_iter()
            .collect()
    } else {
        Vec::new()
    };

    let mut scanned_files = Vec::new();
    let mut missing_root = false;
    let mut root_had_any_existing_dir = false;
    for root_path in &wire.root_paths {
        let scan_input = scanning::ScanInput::with_master_extensions(
            root_path.clone(),
            master_extensions.clone(),
        );
        let scan_result = scanning::scan_with_error(&scan_input).map_err(|err| err.to_string())?;
        missing_root = missing_root || scan_result.missing_root;
        root_had_any_existing_dir = root_had_any_existing_dir || !scan_result.missing_root;
        scanned_files.extend(scan_result.files);
    }
    let invalid_root = false;
    let no_supported_files = scanned_files.is_empty() && root_had_any_existing_dir;

    scanned_files.sort_by(|left, right| {
        left.full_path
            .to_ascii_lowercase()
            .cmp(&right.full_path.to_ascii_lowercase())
    });

    if let Some(active_pool) = pool {
        scanned_files = tauri::async_runtime::block_on(filter_existing_scanned_files(
            active_pool,
            scanned_files,
            &wire.root_paths,
        ))?;
    }

    let discovered_count = scanned_files.len();

    let mut preview_assignments = build_preview_folder_assignments(&wire, &scanned_files);

    if let Some(active_pool) = pool {
        let designers =
            tauri::async_runtime::block_on(load_designers_for_import_inference(active_pool))?;
        let sources =
            tauri::async_runtime::block_on(load_sources_for_import_inference(active_pool))?;

        for assignment in &mut preview_assignments {
            let (inferred_designer_id, inferred_source_id) = infer_assignment_ids_from_folder_path(
                &assignment.folder_path,
                &designers,
                &sources,
            );
            assignment.inferred_designer_id = inferred_designer_id;
            assignment.inferred_source_id = inferred_source_id;
        }
    }

    let resolved_assignments = preview_assignments
        .iter()
        .map(|assignment| {
            let _legacy_resolved = folder_picker::resolve_assignment(
                &folder_picker::FolderAssignment {
                    folder_path: assignment.folder_path.clone(),
                    designer_id: assignment.designer_id,
                    source_id: assignment.source_id,
                },
                &folder_picker::AssignmentFallback {
                    designer_id: wire.global_designer_id,
                    source_id: wire.global_source_id,
                },
            );

            resolve_folder_assignment_wire(assignment, &wire)
        })
        .collect();

    Ok(BulkImportPreview {
        discovered_count,
        selected_count: wire.selected_files.len(),
        folder_count: wire.root_paths.len(),
        scanned_files,
        resolved_assignments,
        scan_token: String::new(),
        missing_root,
        invalid_root,
        no_supported_files,
    })
}

pub fn preview_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportPreview, String> {
    let pool = get_bulk_import_db_pool();
    preview_bulk_import_wire_with_pool(wire, pool.as_ref())
}

#[tauri::command]
pub fn preview_bulk_import(request: BulkImportRequest) -> Result<BulkImportPreview, String> {
    let wire: BulkImportWire = request.into();
    let mut preview = preview_bulk_import_wire(wire.clone())?;
    preview.scan_token =
        store_bulk_import_scan(wire.root_paths.clone(), preview.scanned_files.clone());
    Ok(preview)
}

#[tauri::command]
pub fn browse_import_folder(
    request: Option<BulkImportBrowseFolderRequest>,
) -> Result<BulkImportBrowseFolderResult, String> {
    let (start_dir, allow_multi) = match request {
        Some(value) => (value.start_dir, value.allow_multi),
        None => (None, false),
    };
    let result = folder_picker::browse_folder_with_error(start_dir.as_deref(), allow_multi)
        .map_err(|err| err.to_string())?;

    Ok(BulkImportBrowseFolderResult {
        path: result.path,
        paths: result.paths,
    })
}

#[tauri::command]
pub fn precheck_bulk_import_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportPrecheckResult, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    let (is_first_import, needs_hoop_setup) = load_import_precheck_state_if_initialized()?;
    let context_token = store_bulk_import_context(confirm_wire.clone());

    Ok(BulkImportPrecheckResult {
        context_token,
        context_token_present: true,
        ready_for_confirm: true,
        is_first_import,
        needs_hoop_setup,
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        resolved_assignments,
    })
}

#[tauri::command]
pub fn precheck_bulk_import_from_scan(
    request: BulkImportPrecheckFromScanRequest,
) -> Result<BulkImportPrecheckResult, String> {
    let scan = take_bulk_import_scan(&request.scan_token).ok_or_else(|| {
        format!(
            "Unknown or expired import scan. Please scan your folders again (token: {}).",
            request.scan_token
        )
    })?;

    let selected_files = selected_files_from_scan(&scan.scanned_files, &request.selection);

    let per_folder_assignments: Vec<FolderAssignmentWire> = request
        .per_folder_assignments
        .into_iter()
        .filter(|assignment| count_selected_in_folder(&selected_files, &assignment.folder_path) > 0)
        .collect();

    let root_path_count = scan.root_paths.len();
    let selected_file_count = selected_files.len();

    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: scan.root_paths,
            global_designer_id: request.global_designer_id,
            global_source_id: request.global_source_id,
            per_folder_assignments,
            selected_files,
            create_on_import: request.create_on_import,
        },
        context_token: None,
        canonical_confirm: false,
    };

    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    let (is_first_import, needs_hoop_setup) = load_import_precheck_state_if_initialized()?;
    let context_token = store_bulk_import_context(confirm_wire);

    Ok(BulkImportPrecheckResult {
        context_token,
        context_token_present: true,
        ready_for_confirm: true,
        is_first_import,
        needs_hoop_setup,
        root_path_count,
        selected_file_count,
        resolved_assignments,
    })
}

#[tauri::command]
pub async fn precheck_bulk_import_action_wire(
    request: BulkImportPrecheckActionRequest,
) -> Result<BulkImportPrecheckActionResult, String> {
    let context_token = request.context_token.clone();

    match request.action {
        BulkImportPrecheckActionWire::ReviewHoops => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/hoops/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewTags => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/tags/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewSources => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/sources/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewDesigners => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/designers/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::Cancel => {
            take_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: false,
                consumed_context: true,
                requires_skip_hoops_confirmation: false,
                next_route: Some("/import/".to_string()),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ImportNow => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            let (is_first_import, needs_hoop_setup) =
                load_import_precheck_state_if_initialized_async().await?;
            let requires_skip_hoops_confirmation =
                is_first_import && needs_hoop_setup && !request.confirm_skip_hoops;

            if requires_skip_hoops_confirmation {
                return Ok(BulkImportPrecheckActionResult {
                    action: request.action,
                    context_token_present: true,
                    consumed_context: false,
                    requires_skip_hoops_confirmation: true,
                    next_route: Some("/import/confirm-skip-hoops/".to_string()),
                    confirm_result: None,
                });
            }

            let confirm_result = tauri::async_runtime::spawn_blocking(move || {
                do_confirm_bulk_import_wire_internal(context_token)
            })
            .await
            .map_err(|error| format!("Import task failed to join: {error}"))??;
            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: false,
                consumed_context: true,
                requires_skip_hoops_confirmation: false,
                next_route: Some("/designs/".to_string()),
                confirm_result: Some(confirm_result),
            })
        }
    }
}
