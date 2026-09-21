// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Domain Services Layer
//!
//! The services module encapsulates core business logic for the embroidery catalogue.
//! Service functions operate independently of the Tauri IPC runtime, receiving database connection
//! pools or configuration parameters directly, and returning domain types wrapped in [`Result<T, AppError>`].
//!
//! ## Subsystems & Modules
//!
//! ### Cataloging & File Ingestion
//! - [`scanning`]: Discovers embroidery design files across registered directories, applying filtering and incremental change detection.
//! - [`stitch_identifier`]: Inspects file extensions and headers to route binary files to their corresponding format reader.
//! - [`fingerprint`]: Computes stable perceptual and geometric hashes (e.g. stitch sequence digest) to identify identical designs across different formats.
//! - [`design_metadata`]: Extracts dimension, stitch count, thread color, and hoop suitability attributes from parsed patterns.
//! - [`validation`]: Enforces validation rules on paths, design names, and import configurations.
//!
//! ### Enrichment & AI
//! - [`auto_tagging`]: Manages automated keyword extraction and rule-based categorization.
//! - [`gemini_client`]: Communicates with Google Gemini models to generate descriptive tags and summaries for designs.
//! - [`tagging`]: Manages manual and automated tag taxonomy, hierarchies, and tag associations.
//!
//! ### Rendering & Batch Operations
//! - [`image_generation`]: Renders stitch patterns to PNG thumbnails and previews with realistic thread shading.
//! - [`backfill`]: Coordinates multi-step asynchronous batch processing (thumbnail regeneration, AI re-tagging, metadata repair).
//! - [`projects`]: Organizes designs into user projects and collection folders.
//!
//! ### Database Administration, Health & Recovery
//! - [`admin`]: Database metrics, vacuuming, and low-level administrative queries.
//! - [`db_health`]: Integrity checks, foreign key validation, and missing asset detection.
//! - [`database_recovery`]: Automated recovery pipelines for corrupted databases or disconnected storage roots.
//! - [`compaction`]: Reclaims disk space and rebuilds indexes.
//! - [`maintenance`]: Orphan cleanup and cache eviction tasks.
//! - [`restore`]: Restores catalogue backups and handles schema synchronization.
//! - [`storage_migration`]: Manages migration between storage locations and portable modes.
//!
//! ### Configuration & System
//! - [`settings`]: Reads and persists application preferences.
//! - [`about_documents`]: Provides application license, version, and dependency info.
//! - [`folder_picker`]: Native platform dialog utilities for folder selection.

pub mod about_documents;
pub mod admin;
pub mod auto_tagging;
pub mod backfill;
pub mod compaction;
pub mod database_recovery;
pub mod db_health;
pub mod design_metadata;
pub mod fingerprint;
pub mod folder_picker;
pub mod gemini_client;
pub mod image_generation;
pub mod maintenance;
pub mod projects;
pub mod restore;
pub mod scanning;
pub mod settings;
pub mod stitch_identifier;
pub mod storage_migration;
pub mod tagging;
pub mod validation;

pub use crate::error::AppError;
