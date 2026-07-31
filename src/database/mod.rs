// Root of the database module
pub mod connection;
pub mod migrations;
pub mod models;
pub mod schema;

pub use crate::error::AppError;
