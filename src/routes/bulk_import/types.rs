// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::services::scanning;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportRequest {
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub root_paths: Vec<String>,
    pub fallback_designer_id: Option<i64>,
    pub fallback_source_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderAssignmentWire {
    pub folder_path: String,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
    pub inferred_designer_id: Option<i64>,
    pub inferred_source_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentFieldSourceWire {
    ExplicitPerFolder,
    Global,
    Inferred,
    Blank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAssignmentFieldWire {
    pub value: Option<i64>,
    pub source: AssignmentFieldSourceWire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFolderAssignmentWire {
    pub folder_path: String,
    pub designer_id: ResolvedAssignmentFieldWire,
    pub source_id: ResolvedAssignmentFieldWire,
    pub inferred_designer_id: Option<i64>,
    pub inferred_source_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportWire {
    pub root_paths: Vec<String>,
    pub global_designer_id: Option<i64>,
    pub global_source_id: Option<i64>,
    pub per_folder_assignments: Vec<FolderAssignmentWire>,
    pub selected_files: Vec<String>,
    pub create_on_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportConfirmWire {
    pub wire: BulkImportWire,
    pub context_token: Option<String>,
    pub canonical_confirm: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPreview {
    pub discovered_count: usize,
    pub selected_count: usize,
    pub folder_count: usize,
    pub scanned_files: Vec<scanning::ScannedFile>,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
    /// Server-side scan catalogue token minted by this preview so Continue can
    /// reference it instead of re-sending every scanned file path.
    pub scan_token: String,
    /// True if any selected root path did not exist on disk or was not a directory.
    pub missing_root: bool,
    /// True if any selected root string was empty or relative (shape-invalid).
    pub invalid_root: bool,
    /// True if all selected roots existed but no supported embroidery files were found.
    pub no_supported_files: bool,
}

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderFilesWire {
    pub folder_path: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportSelectionWire {
    /// Folders whose base is "all selected" except for the listed files.
    #[serde(default)]
    pub deselected: Vec<FolderFilesWire>,
    /// Folders whose base is "none selected" except for the listed files.
    /// An empty `files` list means the whole folder is deselected.
    #[serde(default)]
    pub selected_only: Vec<FolderFilesWire>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportPrecheckFromScanRequest {
    pub scan_token: String,
    pub global_designer_id: Option<i64>,
    pub global_source_id: Option<i64>,
    #[serde(default)]
    pub per_folder_assignments: Vec<FolderAssignmentWire>,
    pub selection: BulkImportSelectionWire,
    #[serde(default = "default_true")]
    pub create_on_import: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportBrowseFolderRequest {
    pub start_dir: Option<String>,
    #[serde(default)]
    pub allow_multi: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportWireSummary {
    pub root_path_count: usize,
    pub folder_assignment_count: usize,
    pub selected_file_count: usize,
    pub create_on_import: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportConfirmSummary {
    pub context_token_present: bool,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub per_folder_assignment_count: usize,
    pub canonical_confirm: bool,
    pub resolved_assignment_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportAssignmentResolutionSummary {
    pub resolved_count: usize,
    pub explicit_field_count: usize,
    pub global_field_count: usize,
    pub inferred_field_count: usize,
    pub blank_field_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportConfirmExecutionResult {
    pub context_token_present: bool,
    pub canonical_confirm: bool,
    pub ready_for_persistence: bool,
    pub persisted_design_count: usize,
    /// Number of selected files whose preview could not be generated (decode/read failure). These
    /// records are still persisted with `image_data NULL` so they can be regenerated later.
    pub failed_decode_count: usize,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPrecheckResult {
    pub context_token: String,
    pub context_token_present: bool,
    pub ready_for_confirm: bool,
    pub is_first_import: bool,
    pub needs_hoop_setup: bool,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkImportPrecheckActionWire {
    ReviewHoops,
    ReviewTags,
    ReviewSources,
    ReviewDesigners,
    ImportNow,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportPrecheckActionRequest {
    pub context_token: String,
    pub action: BulkImportPrecheckActionWire,
    #[serde(default)]
    pub confirm_skip_hoops: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPrecheckActionResult {
    pub action: BulkImportPrecheckActionWire,
    pub context_token_present: bool,
    pub consumed_context: bool,
    pub requires_skip_hoops_confirmation: bool,
    pub next_route: Option<String>,
    pub confirm_result: Option<BulkImportConfirmExecutionResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportContextStoreSummary {
    pub active_context_count: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub reset_count: u64,
    pub last_reset_at_millis: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportContextStoreResetResult {
    pub cleared_context_count: usize,
    pub active_context_count: usize,
    pub reset_count: u64,
    pub reset_at_millis: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportStopResult {
    pub stop_requested: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportBrowseFolderResult {
    pub path: Option<String>,
    pub paths: Vec<String>,
}

impl From<BulkImportRequest> for BulkImportWire {
    fn from(request: BulkImportRequest) -> Self {
        let mut root_paths = request
            .root_paths
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if root_paths.is_empty() {
            if let Some(value) = request
                .root_path
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            {
                root_paths.push(value);
            }
        }

        Self {
            root_paths,
            global_designer_id: request.fallback_designer_id,
            global_source_id: request.fallback_source_id,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        }
    }
}

impl From<BulkImportRequest> for BulkImportConfirmWire {
    fn from(request: BulkImportRequest) -> Self {
        Self {
            wire: request.into(),
            context_token: None,
            canonical_confirm: false,
        }
    }
}
