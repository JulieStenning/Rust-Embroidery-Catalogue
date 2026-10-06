// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::executor::persist_bulk_import_confirm_wire;
use super::inference::resolve_bulk_import_assignments;
use super::session::{get_bulk_import_db_pool, take_bulk_import_context};
use super::types::{BulkImportConfirmExecutionResult, BulkImportConfirmWire};

#[allow(dead_code)]
pub(crate) fn canonicalize_bulk_import_confirm_wire(
    mut confirm_wire: BulkImportConfirmWire,
) -> BulkImportConfirmWire {
    confirm_wire.canonical_confirm = true;
    confirm_wire
}

pub(crate) fn persist_bulk_import_confirm_if_initialized(
    confirm_wire: &BulkImportConfirmWire,
    context_token: Option<&str>,
) -> Result<(usize, usize), String> {
    match get_bulk_import_db_pool() {
        Some(pool) => tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            confirm_wire,
            context_token,
        )),
        None => {
            tracing::warn!("Bulk import DB pool not initialized; skipping persistence step.");
            Ok((0, 0))
        }
    }
}

pub(crate) fn do_confirm_bulk_import_wire_internal(
    context_token: String,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let confirm_wire = take_bulk_import_context(&context_token)
        .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

    let (persisted_design_count, failed_decode_count) =
        persist_bulk_import_confirm_if_initialized(&confirm_wire, Some(&context_token))?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    result.failed_decode_count = failed_decode_count;
    Ok(result)
}

#[tauri::command]
pub fn do_confirm_bulk_import_wire(
    context_token: String,
) -> Result<BulkImportConfirmExecutionResult, String> {
    do_confirm_bulk_import_wire_internal(context_token)
}

#[tauri::command]
pub fn execute_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let (persisted_design_count, failed_decode_count) = persist_bulk_import_confirm_if_initialized(
        &confirm_wire,
        confirm_wire.context_token.as_deref(),
    )?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    result.failed_decode_count = failed_decode_count;
    Ok(result)
}

#[tauri::command]
pub fn confirm_bulk_import_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);

    Ok(BulkImportConfirmExecutionResult {
        context_token_present: confirm_wire.context_token.is_some(),
        canonical_confirm: true,
        ready_for_persistence: true,
        persisted_design_count: 0,
        failed_decode_count: 0,
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        resolved_assignments,
    })
}
