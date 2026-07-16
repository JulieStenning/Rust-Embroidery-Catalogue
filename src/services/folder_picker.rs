// Folder assignment resolution contract for import scaffolding.

use rfd::FileDialog;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct FolderAssignment {
    pub folder_path: String,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentFallback {
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BrowseFolderResult {
    pub path: Option<String>,
    pub paths: Vec<String>,
}

pub fn resolve_assignment(
    per_folder: &FolderAssignment,
    fallback: &AssignmentFallback,
) -> FolderAssignment {
    FolderAssignment {
        folder_path: per_folder.folder_path.clone(),
        designer_id: per_folder.designer_id.or(fallback.designer_id),
        source_id: per_folder.source_id.or(fallback.source_id),
    }
}

pub fn browse_folder(
    start_dir: Option<&str>,
    allow_multi: bool,
) -> Result<BrowseFolderResult, String> {
    let mut dialog = FileDialog::new();

    if let Some(candidate) = start_dir.map(str::trim).filter(|value| !value.is_empty()) {
        let path = Path::new(candidate);
        if path.exists() {
            // Canonicalize converts forward slashes to backslashes on Windows.
            // SHCreateItemFromParsingName inside rfd requires a native path to
            // reliably set the initial folder via IFileDialog::SetFolder.
            let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            dialog = dialog.set_directory(&canonical);
        }
    }

    let paths = if allow_multi {
        dialog
            .pick_folders()
            .map(|selected| {
                selected
                    .into_iter()
                    .map(|path_buf| path_buf.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        dialog
            .pick_folder()
            .map(|path_buf| vec![path_buf.to_string_lossy().to_string()])
            .unwrap_or_default()
    };

    let path = paths.first().cloned();

    Ok(BrowseFolderResult { path, paths })
}
