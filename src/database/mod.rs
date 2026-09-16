//! # SQLite Database & Persistence Layer
//!
//! This module manages SQLite connectivity, schema migration execution, connection pooling,
//! and database entity mapping using `sqlx`.
//!
//! ## Core Submodules
//!
//! - [`connection`]: Creates and configures the [`SqlitePool`](sqlx::SqlitePool) with optimal desktop settings
//!   (WAL mode, foreign key enforcement, 5000ms busy timeouts, and maximum connection limits).
//! - [`migrations`]: Embedded SQL migrations applied sequentially on application startup to ensure schema consistency.
//! - [`models`]: Rust structs representing database tables (designs, tags, projects, settings, file hashes).
//! - [`schema`]: SQL table and index constants for query construction.
//!
//! ## Concurrency & Integrity
//!
//! The application uses SQLite in **Write-Ahead Logging (WAL)** mode, allowing concurrent read
//! queries while write operations execute in transactions. Automatic backup checkpoints and integrity
//! verification routines run through the maintenance and recovery services.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod schema;

pub use crate::error::AppError;
