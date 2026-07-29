// Bootstrap configuration ownership (environment + startup defaults).
use crate::paths::AppPaths;
use serde::{Deserialize, Serialize};

pub const DEFAULT_DATABASE_URL: &str = "sqlite:data/database/EmbroideryCatalogue.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub database_url: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let database_url = normalize_database_url(&database_url);

        Self { database_url }
    }

    /// Build a `BootstrapConfig` from the resolved `AppPaths`.
    /// The database URL is derived from `AppPaths.database_path`.
    pub fn from_app_paths(paths: &AppPaths) -> Self {
        let database_url = format!("sqlite:{}", paths.database_path.display());
        Self { database_url }
    }
}

/// Normalize DATABASE_URL so SQLx always receives a valid SQLite URL.
///
/// Accepted inputs:
/// - sqlite:data/database/EmbroideryCatalogue.db
/// - sqlite://data/database/EmbroideryCatalogue.db
/// - sqlite:///D:/path/to/EmbroideryCatalogue.db
/// - data/database/EmbroideryCatalogue.db
///
/// Bare file paths are promoted to `sqlite:<path>`.
pub fn normalize_database_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return DEFAULT_DATABASE_URL.to_string();
    }

    if trimmed.starts_with("sqlite:") {
        return trimmed.to_string();
    }

    format!("sqlite:{}", trimmed)
}

#[tauri::command]
pub fn debug_bootstrap_config() -> BootstrapConfig {
    let config = BootstrapConfig::from_env();
    tracing::debug!("Debug bootstrap configuration: {:#?}", config);
    config
}

/// Ensure the directory containing the SQLite database file exists.
pub fn ensure_database_dir(database_url: &str) {
    let file_path = database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);

    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
}
