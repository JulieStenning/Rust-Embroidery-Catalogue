// Folder assignment resolution contract for import scaffolding.

use rfd::FileDialog;
use serde::Serialize;
use std::path::{Path, PathBuf};

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

/// Resolves a user-provided start directory string into a canonical `PathBuf`.
///
/// Returns `None` if:
///   - `start_dir` is `None`, empty, or whitespace-only.
///   - The resolved path does not exist on disk.
///
/// When the path exists, it is canonicalized (resolving symlinks and normalising
/// separators) so that `rfd` receives a native OS path.
fn resolve_start_dir(start_dir: Option<&str>) -> Option<PathBuf> {
    let candidate = start_dir
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    let path = Path::new(candidate);
    if path.exists() {
        Some(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
    } else {
        None
    }
}

/// Maps the result of a single-folder pick dialog to a `BrowseFolderResult`.
fn map_single_result(pick: Option<PathBuf>) -> BrowseFolderResult {
    match pick {
        Some(path_buf) => {
            let s = path_buf.to_string_lossy().to_string();
            BrowseFolderResult {
                path: Some(s.clone()),
                paths: vec![s],
            }
        }
        None => BrowseFolderResult::default(),
    }
}

/// Maps the result of a multi-folder pick dialog to a `BrowseFolderResult`.
fn map_multi_result(picks: Vec<PathBuf>) -> BrowseFolderResult {
    if picks.is_empty() {
        return BrowseFolderResult::default();
    }

    let paths: Vec<_> = picks
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let path = paths.first().cloned();
    BrowseFolderResult { path, paths }
}

pub fn browse_folder(
    start_dir: Option<&str>,
    allow_multi: bool,
) -> Result<BrowseFolderResult, String> {
    let mut dialog = FileDialog::new();

    if let Some(dir) = resolve_start_dir(start_dir) {
        dialog = dialog.set_directory(&dir);
    }

    let result = if allow_multi {
        map_multi_result(dialog.pick_folders().unwrap_or_default())
    } else {
        map_single_result(dialog.pick_folder())
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_assignment ────────────────────────────────────────────────

    #[test]
    fn resolve_assignment_uses_per_folder_id_when_present() {
        let per = FolderAssignment {
            folder_path: "/a".into(),
            designer_id: Some(10),
            source_id: Some(20),
        };
        let fallback = AssignmentFallback {
            designer_id: Some(99),
            source_id: Some(99),
        };

        let result = resolve_assignment(&per, &fallback);

        assert_eq!(result.folder_path, "/a");
        assert_eq!(result.designer_id, Some(10));
        assert_eq!(result.source_id, Some(20));
    }

    #[test]
    fn resolve_assignment_falls_back_when_per_folder_id_is_none() {
        let per = FolderAssignment {
            folder_path: "/b".into(),
            designer_id: None,
            source_id: None,
        };
        let fallback = AssignmentFallback {
            designer_id: Some(7),
            source_id: Some(14),
        };

        let result = resolve_assignment(&per, &fallback);

        assert_eq!(result.folder_path, "/b");
        assert_eq!(result.designer_id, Some(7));
        assert_eq!(result.source_id, Some(14));
    }

    #[test]
    fn resolve_assignment_both_none_stays_none() {
        let per = FolderAssignment {
            folder_path: "/c".into(),
            designer_id: None,
            source_id: None,
        };
        let fallback = AssignmentFallback {
            designer_id: None,
            source_id: None,
        };

        let result = resolve_assignment(&per, &fallback);

        assert_eq!(result.folder_path, "/c");
        assert_eq!(result.designer_id, None);
        assert_eq!(result.source_id, None);
    }

    #[test]
    fn resolve_assignment_mixed_fallback_fills_gaps() {
        let per = FolderAssignment {
            folder_path: "/d".into(),
            designer_id: Some(1),
            source_id: None,
        };
        let fallback = AssignmentFallback {
            designer_id: None,
            source_id: Some(2),
        };

        let result = resolve_assignment(&per, &fallback);

        assert_eq!(result.folder_path, "/d");
        assert_eq!(result.designer_id, Some(1));  // from per-folder
        assert_eq!(result.source_id, Some(2));     // from fallback
    }

    // ── resolve_start_dir ─────────────────────────────────────────────────

    #[test]
    fn resolve_start_dir_none_returns_none() {
        assert_eq!(resolve_start_dir(None), None);
    }

    #[test]
    fn resolve_start_dir_empty_string_returns_none() {
        assert_eq!(resolve_start_dir(Some("")), None);
    }

    #[test]
    fn resolve_start_dir_whitespace_only_returns_none() {
        assert_eq!(resolve_start_dir(Some("   ")), None);
    }

    #[test]
    fn resolve_start_dir_nonexistent_path_returns_none() {
        let result = resolve_start_dir(Some(
            "C:\\this\\path\\does\\not\\exist_on_any_machine_xyz123",
        ));
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_start_dir_existing_path_returns_canonical() {
        let tmp = std::env::temp_dir();
        let input = tmp.to_string_lossy().to_string();

        let result = resolve_start_dir(Some(&input));

        assert!(result.is_some());
        let canonical = std::fs::canonicalize(&tmp).unwrap();
        assert_eq!(result.unwrap(), canonical);
    }

    #[test]
    fn resolve_start_dir_trailing_whitespace_is_trimmed() {
        let tmp = std::env::temp_dir();
        let input = format!("  {}  ", tmp.to_string_lossy());

        let result = resolve_start_dir(Some(&input));

        assert!(result.is_some());
        let canonical = std::fs::canonicalize(&tmp).unwrap();
        assert_eq!(result.unwrap(), canonical);
    }

    // ── map_single_result ─────────────────────────────────────────────────

    #[test]
    fn map_single_result_some_path() {
        let tmp = std::env::temp_dir();
        let result = map_single_result(Some(tmp.clone()));

        let expected = tmp.to_string_lossy().to_string();
        assert_eq!(result.path, Some(expected.clone()));
        assert_eq!(result.paths, vec![expected]);
    }

    #[test]
    fn map_single_result_none_returns_default() {
        let result = map_single_result(None);
        assert_eq!(result.path, None);
        assert!(result.paths.is_empty());
    }

    // ── map_multi_result ──────────────────────────────────────────────────

    #[test]
    fn map_multi_result_multiple_paths() {
        let tmp = std::env::temp_dir();
        let sub_a = tmp.join("sub_a");
        let sub_b = tmp.join("sub_b");
        let picks = vec![sub_a.clone(), sub_b.clone()];

        let result = map_multi_result(picks);

        let expected_a = sub_a.to_string_lossy().to_string();
        let expected_b = sub_b.to_string_lossy().to_string();
        assert_eq!(result.path, Some(expected_a.clone()));
        assert_eq!(result.paths, vec![expected_a, expected_b]);
    }

    #[test]
    fn map_multi_result_empty_returns_default() {
        let result = map_multi_result(vec![]);
        assert_eq!(result.path, None);
        assert!(result.paths.is_empty());
    }

    // ── BrowseFolderResult serialization ──────────────────────────────────

    #[test]
    fn browse_folder_result_serializes_correctly() {
        let result = BrowseFolderResult {
            path: Some("D:/designs".into()),
            paths: vec!["D:/designs".into(), "D:/backups".into()],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""path":"D:/designs""#));
        assert!(json.contains(r#""paths""#));
    }
}