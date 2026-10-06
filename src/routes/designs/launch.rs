// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::config::BootstrapConfig;
use crate::paths::normalize_windows_explorer_target;
use crate::AppState;
use serde::Serialize;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct LaunchDesignResult {
    pub design_id: i64,
    pub attempted_path: String,
    pub opened_path: Option<String>,
    pub suppressed: bool,
    pub success: bool,
    pub message: String,
}

pub(crate) fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

pub(crate) fn external_launches_disabled() -> bool {
    if let Ok(value) = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN") {
        if is_truthy(&value) {
            return true;
        }
    }

    false
}

pub(crate) fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

pub(crate) fn derive_data_root_from_database_url() -> PathBuf {
    let config = BootstrapConfig::from_env();
    let db_path = Path::new(strip_sqlite_prefix(&config.database_url));

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent)
        } else {
            parent
        }
    } else {
        Path::new("data")
    };

    root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
}

pub(crate) fn get_designs_base_path() -> PathBuf {
    derive_data_root_from_database_url().join("MachineEmbroideryDesigns")
}

pub(crate) fn resolve_design_full_path(relative_file_path: &str) -> PathBuf {
    let designs_base = get_designs_base_path();
    crate::paths::resolve_design_filepath(relative_file_path, &designs_base)
}

pub(crate) fn nearest_existing_folder(path: &Path, fallback: &Path) -> PathBuf {
    let mut candidate = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| fallback.to_path_buf())
    };

    loop {
        if candidate.is_dir() {
            return candidate;
        }

        let Some(parent) = candidate.parent() else {
            break;
        };

        if parent == candidate {
            break;
        }

        candidate = parent.to_path_buf();
    }

    fallback.to_path_buf()
}

pub(crate) fn open_with_default_app(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to launch default app: {}", e))?;
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to launch default app: {}", e))?;
        return Ok(());
    }

    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch default app: {}", e))?;

    Ok(())
}

pub(crate) async fn get_design_filepath(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<String, String> {
    let filepath =
        sqlx::query_scalar::<_, String>("SELECT filepath FROM designs WHERE id = ? LIMIT 1")
            .bind(design_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    match filepath {
        Some(value) if !value.trim().is_empty() => Ok(value),
        Some(_) => Err(format!(
            "Design with id={} does not have a stored filepath.",
            design_id
        )),
        None => Err(format!("Design with id={} not found.", design_id)),
    }
}

pub(crate) async fn get_design_master_filepath(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<String, String> {
    let row = sqlx::query_as::<_, (String, Option<String>, bool)>(
        "SELECT filepath, master_filepath, is_master_only FROM designs WHERE id = ? LIMIT 1",
    )
    .bind(design_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some((filepath, master_filepath, is_master_only)) => {
            if let Some(mf) = master_filepath {
                if !mf.trim().is_empty() {
                    return Ok(mf);
                }
            }
            if is_master_only && !filepath.trim().is_empty() {
                return Ok(filepath);
            }
            Err(format!(
                "Design with id={} does not have a paired master file.",
                design_id
            ))
        }
        None => Err(format!("Design with id={} not found.", design_id)),
    }
}

pub(crate) async fn open_design_in_editor_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);
    let attempted = full_path.to_string_lossy().to_string();

    if external_launches_disabled() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: true,
            success: false,
            message: "External launches are disabled in this runtime context.".to_string(),
        });
    }

    if !full_path.is_file() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: "Design file was not found on disk.".to_string(),
        });
    }

    match open_with_default_app(&full_path) {
        Ok(()) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: Some(full_path.to_string_lossy().to_string()),
            suppressed: false,
            success: true,
            message: "Opened design in the system default app.".to_string(),
        }),
        Err(error) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: error,
        }),
    }
}

pub(crate) async fn open_design_master_in_editor_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    let filepath = get_design_master_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);
    let attempted = full_path.to_string_lossy().to_string();

    if external_launches_disabled() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: true,
            success: false,
            message: "External launches are disabled in this runtime context.".to_string(),
        });
    }

    if !full_path.is_file() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: "Master design file was not found on disk.".to_string(),
        });
    }

    match open_with_default_app(&full_path) {
        Ok(()) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: Some(full_path.to_string_lossy().to_string()),
            suppressed: false,
            success: true,
            message: "Opened master design in the system default app.".to_string(),
        }),
        Err(error) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: format!("Failed to launch master design file: {}", error),
        }),
    }
}

pub(crate) async fn open_design_in_explorer_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);
    let attempted = full_path.to_string_lossy().to_string();

    if external_launches_disabled() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: true,
            success: false,
            message: "External launches are disabled in this runtime context.".to_string(),
        });
    }

    let base = get_designs_base_path();
    let opened_path = if full_path.is_file() {
        if cfg!(target_os = "windows") {
            let select_target = normalize_windows_explorer_target(
                &full_path
                    .canonicalize()
                    .unwrap_or_else(|_| full_path.clone()),
            );
            let _ = Command::new("explorer.exe")
                .arg("/select,")
                .arg(&select_target)
                .spawn()
                .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        } else {
            open_with_default_app(full_path.parent().unwrap_or(&full_path))?;
        }
        full_path
    } else {
        let folder = nearest_existing_folder(&full_path, &base);
        if cfg!(target_os = "windows") {
            let open_target = normalize_windows_explorer_target(
                &folder.canonicalize().unwrap_or_else(|_| folder.clone()),
            );
            let _ = Command::new("explorer.exe")
                .arg(&open_target)
                .spawn()
                .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        } else {
            open_with_default_app(&folder)?;
        }
        folder
    };

    Ok(LaunchDesignResult {
        design_id,
        attempted_path: attempted,
        opened_path: Some(opened_path.to_string_lossy().to_string()),
        suppressed: false,
        success: true,
        message: "Opened Explorer/folder view for design path.".to_string(),
    })
}

#[tauri::command]
pub async fn open_design_in_editor(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_editor_with_pool(&state.db_pool()?, design_id).await
}

#[tauri::command]
pub async fn open_design_master_in_editor(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_master_in_editor_with_pool(&state.db_pool()?, design_id).await
}

#[tauri::command]
pub async fn open_design_in_explorer(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_explorer_with_pool(&state.db_pool()?, design_id).await
}
