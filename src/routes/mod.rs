// Root of the routes module
pub mod about;
pub mod admin;
pub mod batch_operations;
pub mod bulk_import;
pub mod database_recovery;
pub mod designs;
pub mod maintenance;
pub mod projects;
pub mod restore;
pub mod settings;
pub mod storage_migration;

pub use crate::error::AppError;
