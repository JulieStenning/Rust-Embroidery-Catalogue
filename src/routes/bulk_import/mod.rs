// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Bulk Import Route Module
//!
//! Submodules:
//! - [`types`]: Request and response wire structures for import operations.
//! - [`session`]: In-memory session store for multi-step scan and import flows.
//! - [`paths`]: Filepath calculation, collision handling, and library containment checks.
//! - [`inference`]: Automatic designer and source inference from folder paths.
//! - [`precheck`]: Scanning, deduplication, and pre-import validation.
//! - [`precheck_helpers`]: Selection filtering and debug summary helpers.
//! - [`executor`]: Batch persistence, image generation, and database commit loop.
//! - [`commands`]: Tauri command execution handlers.

pub mod catalog;
pub mod commands;
pub mod executor;
pub mod inference;
pub mod paths;
pub mod precheck;
pub mod precheck_helpers;
pub mod session;
pub mod types;

pub use commands::*;
pub use inference::*;
pub use paths::{compute_file_hash_blake3, compute_file_size};
pub use precheck::*;
pub use precheck_helpers::*;
pub use session::*;
pub use types::*;
// Internal re-exports for test module
#[cfg(test)]
pub(crate) use catalog::*;
#[cfg(test)]
pub(crate) use executor::*;
#[cfg(test)]
pub(crate) use paths::*;
#[cfg(test)]
pub(crate) use sqlx::SqlitePool;

#[cfg(test)]
#[path = "../bulk_import_tests.rs"]
mod tests;
