// Root of the services module
pub mod about_documents;
pub mod admin;
pub mod auto_tagging;
pub mod backfill;
pub mod compaction;
pub mod fingerprint;
pub mod folder_picker;
pub mod gemini_client;
pub mod image_generation;
pub mod import;
pub mod maintenance;
pub mod portable;
pub mod projects;
pub mod scanning;
pub mod settings;
pub mod stitch_identifier;
pub mod tagging;
pub mod validation;

pub use crate::error::AppError;
