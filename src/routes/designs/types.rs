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
