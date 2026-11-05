// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectListItem {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BrowseTagOption {
    pub id: i64,
    pub description: String,
    pub tag_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DesignLookupOption {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignCommandResult {
    pub design_id: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFolderStitchFilesResult {
    pub folder: String,
    pub scanned_count: usize,
    pub paired_count: usize,
    pub current_design_paired: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignProjectRequest {
    pub project_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkAddToProjectResult {
    pub project_id: i64,
    pub requested_count: usize,
    pub added_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkSetTagsResult {
    pub requested_count: usize,
    pub updated_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignTagsRequest {
    pub tag_ids: Vec<i64>,
    #[serde(default)]
    pub image_tags_verified: Option<bool>,
    #[serde(default)]
    pub stitching_tags_verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkApplyTagsRequest {
    pub tags_to_add: Vec<i64>,
    pub tags_to_remove: Vec<i64>,
    #[serde(default)]
    pub clear_all_tags: bool,
    #[serde(default)]
    pub image_tags_verified: Option<bool>,
    #[serde(default)]
    pub stitching_tags_verified: Option<bool>,
}
