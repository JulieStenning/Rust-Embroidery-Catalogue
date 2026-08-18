// Bootstrap configuration ownership (environment + startup defaults).
use crate::error::AppError;
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
    /// Forward slashes are used so the URL works correctly with SQLx on all
    /// platforms (including Windows, where backslashes confuse the URI parser).
    pub fn from_app_paths(paths: &AppPaths) -> Self {
        let display = paths.database_path.display().to_string();
        let database_url = format!("sqlite:{}", display.replace('\\', "/"));
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
pub fn ensure_database_dir(database_url: &str) -> Result<(), AppError> {
    let file_path = database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);

    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| {
                AppError::io(format!(
                    "failed to create database directory {}: {err}",
                    parent.display()
                ))
            })?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{AppPaths, ExecutionMode};
    use std::path::PathBuf;

    // ─── normalize_database_url ──────────────────────────────────────────────

    #[test]
    fn normalize_empty_string_returns_default() {
        assert_eq!(normalize_database_url(""), DEFAULT_DATABASE_URL);
    }

    #[test]
    fn normalize_whitespace_only_returns_default() {
        assert_eq!(normalize_database_url("   "), DEFAULT_DATABASE_URL);
    }

    #[test]
    fn normalize_already_sqlite_prefix_preserved() {
        let url = "sqlite:my/custom/path.db";
        assert_eq!(normalize_database_url(url), url);
    }

    #[test]
    fn normalize_sqlite_triple_slash_prefix_preserved() {
        let url = "sqlite:///D:/data/EmbroideryCatalogue.db";
        assert_eq!(normalize_database_url(url), url);
    }

    #[test]
    fn normalize_sqlite_double_slash_prefix_preserved() {
        let url = "sqlite://data/database/EmbroideryCatalogue.db";
        assert_eq!(normalize_database_url(url), url);
    }

    #[test]
    fn normalize_bare_path_prepends_sqlite_prefix() {
        assert_eq!(
            normalize_database_url("data/database/EmbroideryCatalogue.db"),
            "sqlite:data/database/EmbroideryCatalogue.db"
        );
    }

    #[test]
    fn normalize_bare_path_with_surrounding_whitespace_trims_and_prepends() {
        assert_eq!(
            normalize_database_url("  data/database/EmbroideryCatalogue.db  "),
            "sqlite:data/database/EmbroideryCatalogue.db"
        );
    }

    // ─── debug_bootstrap_config ──────────────────────────────────────────────

    #[test]
    fn debug_bootstrap_config_returns_config_from_env() {
        let prior = std::env::var("DATABASE_URL").ok();
        let test_url = "sqlite:debug_test/test.db";
        std::env::set_var("DATABASE_URL", test_url);

        let config = debug_bootstrap_config();
        assert_eq!(config.database_url, test_url);

        // Restore the original variable.
        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    // ─── BootstrapConfig::from_env ───────────────────────────────────────────

    #[test]
    fn from_env_falls_back_to_default_when_env_var_missing() {
        // Temporarily remove DATABASE_URL so the fallback is exercised.
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::remove_var("DATABASE_URL");

        let config = BootstrapConfig::from_env();
        assert_eq!(config.database_url, DEFAULT_DATABASE_URL);

        // Restore the original variable (if any).
        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        }
    }

    #[test]
    fn from_env_honours_explicit_env_var() {
        let prior = std::env::var("DATABASE_URL").ok();
        let test_url = "sqlite:test_data/test.db";
        std::env::set_var("DATABASE_URL", test_url);

        let config = BootstrapConfig::from_env();
        assert_eq!(config.database_url, test_url);

        // Restore the original variable.
        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    // ─── BootstrapConfig::from_app_paths ──────────────────────────────────────

    #[test]
    fn from_app_paths_constructs_sqlite_url_from_database_path() {
        let paths = AppPaths {
            mode: ExecutionMode::Installed,
            data_root: PathBuf::from("/tmp/test_data"),
            embroidery_designs_dir: PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns"),
            database_dir: PathBuf::from("/tmp/test_data/Database"),
            database_path: PathBuf::from("/tmp/test_data/Database/EmbroideryCatalogue.db"),
            log_dir: PathBuf::from("/tmp/test_data/logs"),
        };

        let config = BootstrapConfig::from_app_paths(&paths);
        assert_eq!(
            config.database_url,
            "sqlite:/tmp/test_data/Database/EmbroideryCatalogue.db"
        );
    }

    // ─── ensure_database_dir ─────────────────────────────────────────────────

    #[test]
    fn ensure_database_dir_creates_parent_for_sqlite_path() {
        let tmp = std::env::temp_dir().join(format!(
            "config-test-sqlite-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db_url = format!("sqlite:{}/subdir/catalogue.db", tmp.display());

        // Ensure the directory does not exist yet.
        assert!(!tmp.join("subdir").exists());

        ensure_database_dir(&db_url).unwrap();

        // After the call the directory should have been created.
        assert!(tmp.join("subdir").exists());

        // Clean up.
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ensure_database_dir_creates_parent_for_bare_path() {
        let tmp = std::env::temp_dir().join(format!(
            "config-test-bare-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db_url = format!("{}/subdir/catalogue.db", tmp.display());

        assert!(!tmp.join("subdir").exists());

        ensure_database_dir(&db_url).unwrap();

        assert!(tmp.join("subdir").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ensure_database_dir_does_nothing_when_no_parent() {
        // A filename with no directory component should not panic or create anything.
        ensure_database_dir("sqlite:catalogue.db").unwrap();
        ensure_database_dir("catalogue.db").unwrap();
        // If we get here without panicking the test passes.
    }
}
