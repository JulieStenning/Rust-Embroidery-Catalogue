// Root of the routes module
pub mod about;
pub mod admin;
pub mod api;
pub mod bulk_import;
pub mod database_recovery;
pub mod designs;
pub mod import;
pub mod maintenance;
pub mod projects;
pub mod restore;
pub mod settings;
pub mod storage_migration;
pub mod tagging_actions;

pub use crate::error::AppError;
