// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Designs Route Module
//!
//! Submodules:
//! - [`browse`]: Design catalog search, filtering, and pagination.
//! - [`browse_query`]: Query token parsing and SQL clause construction.
//! - [`details`]: Design detail inspection and metadata retrieval.
//! - [`preview`]: Image thumbnail generation, preview rendering, and re-parsing.
//! - [`launch`]: External application launching and explorer integration.
//! - [`metadata`]: Metadata updates, ratings, project assignments, and verification status.
//! - [`tags`]: Tag assignment, removal, and bulk classification.
//! - [`deletion`]: Single and bulk design deletion with optional file recycling.
//! - [`types`]: Common shared wire structures and request/response payloads.

pub mod browse;
pub mod browse_query;
pub mod deletion;
pub mod details;
pub mod launch;
pub mod metadata;
pub mod preview;
pub mod tags;
pub mod types;

pub use browse::*;
pub use browse_query::*;
pub use deletion::*;
pub use details::*;
pub use launch::*;
pub use metadata::*;
pub use preview::*;
pub use tags::*;
pub use types::*;

// Internal re-exports for test module
#[cfg(test)]
pub(crate) use crate::AppState;
#[cfg(test)]
pub(crate) use sqlx::{QueryBuilder, Sqlite, SqlitePool};
#[cfg(test)]
pub(crate) use std::path::PathBuf;

#[cfg(test)]
#[path = "../designs_tests.rs"]
mod tests;
