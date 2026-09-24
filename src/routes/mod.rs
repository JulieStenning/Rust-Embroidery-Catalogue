// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Tauri IPC Routes Layer
//!
//! The routes module exposes the backend API to the Svelte frontend via Tauri's IPC
//! command system (`#[tauri::command]`).
//!
//! ## Architecture & Responsibilities
//!
//! Routes act as thin adapter controllers:
//! 1. **Parameter Unpacking & Validation:** Unpack JSON arguments into strongly-typed Rust parameters.
//! 2. **State Injection:** Access shared resources ([`AppState`](crate::AppState), connection pools, task cancellation tokens) via `tauri::State`.
//! 3. **Service Invocation:** Delegate core business logic to the appropriate [`services`](crate::services) module.
//! 4. **Error Mapping:** Serialize errors as structured JSON ([`AppError`]) consumed by frontend API adapters.
//!
//! ## Route Modules
//!
//! - [`designs`]: Search, filter, inspect details, update metadata, and query stitch data.
//! - [`projects`]: Create, organize, and manage user projects and design assignments.
//! - [`bulk_import`]: Folder scanning, batch ingestion pipelines, and conflict resolution.
//! - [`batch_operations`]: Long-running operations (thumbnail generation, AI tagging backfills).
//! - [`settings`]: Read and update application preferences, dark mode, and directory paths.
//! - [`admin`]: Database health metrics, manual backup generation, and administrative actions.
//! - [`database_recovery`]: Handle corrupt, missing, or disconnected database scenarios.
//! - [`maintenance`]: Database vacuuming, index optimization, and orphan cleanup.
//! - [`restore`]: Restore operations from archive files.
//! - [`storage_migration`]: Relocate database and asset files across drives/directories.
//! - [`about`]: System diagnostics, license info, and version endpoints.

pub mod about;
pub mod admin;
pub mod batch_operations;
pub mod bulk_import;
pub mod database_recovery;
pub mod designs;
pub mod licence;
pub mod maintenance;
pub mod projects;
pub mod restore;
pub mod settings;
pub mod storage_migration;

pub use crate::error::AppError;
