// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

// Scanning service for recursive file discovery with deterministic dedup policy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub const SUPPORTED_EXTENSIONS: &[&str] = &["jef", "pes", "hus", "dst", "exp", "vp3"];

const EXCLUDED_DIRECTORY_NAMES: &[&str] = &["system volume information"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanInput {
    pub root_path: String,
    #[serde(default)]
    pub master_extensions: Vec<String>,
}

impl ScanInput {
    pub fn new(root_path: impl Into<String>) -> Self {
        Self {
            root_path: root_path.into(),
            master_extensions: Vec::new(),
        }
    }

    pub fn with_master_extensions(
        root_path: impl Into<String>,
        master_extensions: Vec<String>,
    ) -> Self {
        Self {
            root_path: root_path.into(),
            master_extensions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannedFile {
    pub full_path: String,
    pub filename: String,
    pub extension: String,
    pub file_size_bytes: Option<i64>,
    pub dedup_group_key: String,
    #[serde(default)]
    pub is_master: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
    /// True when the root path did not exist or was not a directory.
    pub missing_root: bool,
    /// True when the root path existed but no supported embroidery files were found.
    pub no_supported_files: bool,
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}

fn make_dedup_group_key(parent: &Path, stem: &str, extension: &str) -> String {
    format!(
        "{}|{}|{}",
        parent
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase(),
        stem.to_ascii_lowercase(),
        extension.to_ascii_lowercase()
    )
}

fn should_skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| {
            EXCLUDED_DIRECTORY_NAMES
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(value))
        })
        .unwrap_or(false)
}

fn visit_dir(
    dir: &Path,
    master_extensions: &[String],
    dedup: &mut HashMap<String, ScannedFile>,
    is_cancelled: Option<&dyn Fn() -> bool>,
) -> Result<(), AppError> {
    if is_cancelled.map(|f| f()).unwrap_or(false) {
        return Err(AppError::invalid_input("Scan cancelled"));
    }

    if should_skip_directory(dir) {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        if is_cancelled.map(|f| f()).unwrap_or(false) {
            return Err(AppError::invalid_input("Scan cancelled"));
        }

        let path = entry.path();

        if path.is_dir() {
            visit_dir(&path, master_extensions, dedup, is_cancelled)?;
            continue;
        }

        let extension = match path.extension().and_then(|value| value.to_str()) {
            Some(value) => normalize_extension(value),
            None => continue,
        };

        let is_stitch = is_supported_extension(&extension);
        let is_master = is_master_extension(&extension, master_extensions);

        if !is_stitch && !is_master {
            continue;
        }

        let stem = match path.file_stem().and_then(|value| value.to_str()) {
            Some(value) => value,
            None => continue,
        };

        let parent = match path.parent() {
            Some(value) => value,
            None => continue,
        };

        let filename = match path.file_name().and_then(|value| value.to_str()) {
            Some(value) => value.to_string(),
            None => continue,
        };

        let file_size_bytes = fs::metadata(&path).ok().map(|m| m.len() as i64);

        let dedup_group_key = make_dedup_group_key(parent, stem, &extension);
        let candidate = ScannedFile {
            full_path: path.to_string_lossy().to_string(),
            filename,
            extension: extension.clone(),
            file_size_bytes,
            dedup_group_key: dedup_group_key.clone(),
            is_master,
        };

        dedup.insert(dedup_group_key, candidate);
    }
    Ok(())
}

pub fn is_supported_extension(extension: &str) -> bool {
    let normalized = normalize_extension(extension);
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|candidate| *candidate == normalized)
}

pub fn is_master_extension(extension: &str, master_extensions: &[String]) -> bool {
    let normalized = normalize_extension(extension);
    master_extensions
        .iter()
        .any(|candidate| normalize_extension(candidate) == normalized)
}

pub fn scan(input: &ScanInput) -> ScanResult {
    scan_with_error(input).unwrap_or_else(|_| ScanResult {
        files: Vec::new(),
        missing_root: true,
        no_supported_files: false,
    })
}

pub fn scan_with_error(input: &ScanInput) -> Result<ScanResult, AppError> {
    scan_with_error_and_cancel(input, None)
}

pub fn scan_with_error_and_cancel(
    input: &ScanInput,
    is_cancelled: Option<&dyn Fn() -> bool>,
) -> Result<ScanResult, AppError> {
    let root_path = PathBuf::from(&input.root_path);

    let trimmed = input.root_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input("root path must not be empty"));
    }

    if is_cancelled.map(|f| f()).unwrap_or(false) {
        return Err(AppError::invalid_input("Scan cancelled"));
    }

    if !root_path.exists() || !root_path.is_dir() {
        return Ok(ScanResult {
            files: Vec::new(),
            missing_root: true,
            no_supported_files: false,
        });
    }

    let normalized_masters: Vec<String> = input
        .master_extensions
        .iter()
        .map(|ext| normalize_extension(ext))
        .collect();

    let mut dedup: HashMap<String, ScannedFile> = HashMap::new();
    visit_dir(&root_path, &normalized_masters, &mut dedup, is_cancelled)?;

    let mut files: Vec<ScannedFile> = dedup.into_values().collect();
    files.sort_by(|left, right| {
        left.full_path
            .to_ascii_lowercase()
            .cmp(&right.full_path.to_ascii_lowercase())
    });

    let no_supported_files = files.is_empty();
    Ok(ScanResult {
        files,
        missing_root: false,
        no_supported_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rec-{prefix}-{stamp}-{sequence}"))
    }

    fn create_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }

        let mut file = fs::File::create(path).expect("file should be created");
        writeln!(file, "test").expect("file should be writable");
    }

    #[test]
    fn supports_extensions_case_insensitively() {
        assert!(is_supported_extension("PES"));
        assert!(is_supported_extension(".dst"));
        assert!(!is_supported_extension("txt"));
    }

    #[test]
    fn scan_with_error_reports_empty_root_path_as_invalid_input() {
        let result = scan_with_error(&ScanInput::new(""));

        assert!(matches!(result, Err(AppError::InvalidInput { .. })));
    }

    #[test]
    fn scan_recurses_and_filters_extensions() {
        let root = unique_temp_dir("scan-recurses");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("design1.pes"));
        create_file(&root.join("nested").join("design2.dst"));
        create_file(&root.join("nested").join("notes.txt"));

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().any(|file| file.extension == "pes"));
        assert!(result.files.iter().any(|file| file.extension == "dst"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_keeps_all_supported_formats_of_same_design() {
        let root = unique_temp_dir("scan-multi-format");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("design.pes"));
        create_file(&root.join("design.jef"));
        create_file(&root.join("design.vp3"));

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert_eq!(result.files.len(), 3, "all three formats should be kept");
        let extensions: Vec<&str> = result.files.iter().map(|f| f.extension.as_str()).collect();
        assert!(extensions.contains(&"pes"));
        assert!(extensions.contains(&"jef"));
        assert!(extensions.contains(&"vp3"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_deduplicates_unsupported_extension_replaced_by_supported() {
        let root = unique_temp_dir("scan-dedup");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("same-name.pmv"));
        create_file(&root.join("same-name.pes"));

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].extension, "pes");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_returns_empty_for_missing_root() {
        let root = unique_temp_dir("scan-missing");
        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert!(result.files.is_empty());
        assert!(result.missing_root, "missing root should be flagged");
        assert!(
            !result.no_supported_files,
            "missing root is not a no-files-found case"
        );
    }

    #[test]
    fn scan_flags_no_supported_files_for_empty_directory() {
        let root = unique_temp_dir("scan-no-files");
        fs::create_dir_all(&root).expect("root should be created");

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert!(result.files.is_empty());
        assert!(
            !result.missing_root,
            "root exists so missing_root must be false"
        );
        assert!(
            result.no_supported_files,
            "existing directory with zero supported files should set no_supported_files"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_flags_no_supported_files_for_directory_with_only_ignored_extensions() {
        let root = unique_temp_dir("scan-unsupported-only");
        fs::create_dir_all(&root).expect("root should be created");
        create_file(&root.join("notes.txt"));
        create_file(&root.join("image.png"));

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert!(result.files.is_empty());
        assert!(!result.missing_root);
        assert!(
            result.no_supported_files,
            "directory with only non-embroidery files should set no_supported_files"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_with_found_files_clears_no_supported_files_flag() {
        let root = unique_temp_dir("scan-has-files");
        fs::create_dir_all(&root).expect("root should be created");
        create_file(&root.join("design.pes"));

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert_eq!(result.files.len(), 1);
        assert!(!result.missing_root);
        assert!(!result.no_supported_files);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_excludes_system_volume_information_directories() {
        let root = unique_temp_dir("scan-excludes-system-volume-information");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("visible-design.pes"));
        create_file(
            &root
                .join("System Volume Information")
                .join("hidden-design.pes"),
        );

        let result = scan(&ScanInput::new(root.to_string_lossy().to_string()));

        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].full_path.ends_with("visible-design.pes"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_discovers_configured_master_extensions() {
        let root = unique_temp_dir("scan-masters");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("flower.pes"));
        create_file(&root.join("flower.eof"));
        create_file(&root.join("flower.art"));
        create_file(&root.join("other.txt"));

        // Without master_extensions: only flower.pes found
        let res_no_master = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
            master_extensions: vec![],
        });
        assert_eq!(res_no_master.files.len(), 1);
        assert_eq!(res_no_master.files[0].filename, "flower.pes");
        assert!(!res_no_master.files[0].is_master);

        // With eof enabled: flower.pes and flower.eof found
        let res_eof = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
            master_extensions: vec!["eof".to_string()],
        });
        assert_eq!(res_eof.files.len(), 2);
        assert!(res_eof
            .files
            .iter()
            .any(|f| f.filename == "flower.eof" && f.is_master));
        assert!(res_eof
            .files
            .iter()
            .any(|f| f.filename == "flower.pes" && !f.is_master));

        let _ = fs::remove_dir_all(&root);
    }
}
