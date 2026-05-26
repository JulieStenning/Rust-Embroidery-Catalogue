// Bootstrap configuration ownership (environment + startup defaults).
use serde::{Deserialize, Serialize};

pub const DEFAULT_DATABASE_URL: &str = "sqlite:data/database/catalogue.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub database_url: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

        Self { database_url }
    }
}

#[tauri::command]
pub fn debug_bootstrap_config() -> BootstrapConfig {
    let config = BootstrapConfig::from_env();
    println!("Debug bootstrap configuration: {:#?}", config);
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