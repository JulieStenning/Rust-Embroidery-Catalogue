// Scanning service for recursive file discovery with deterministic dedup policy.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jef", "pes", "hus", "dst", "exp", "vp3", 
];

pub const EXTENSION_PRIORITY: &[&str] = &[
    "jef", "pes", "vp3", "hus", "dst", "exp", 
];

const EXCLUDED_DIRECTORY_NAMES: &[&str] = &["system volume information"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanInput {
    pub root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannedFile {
    pub full_path: String,
    pub extension: String,
    pub dedup_group_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}

fn extension_rank(extension: &str) -> usize {
    EXTENSION_PRIORITY
        .iter()
        .position(|candidate| candidate.eq_ignore_ascii_case(extension))
        .unwrap_or(usize::MAX)
}

fn make_dedup_group_key(parent: &Path, stem: &str) -> String {
    format!("{}|{}", parent.to_string_lossy().replace('\\', "/").to_ascii_lowercase(), stem.to_ascii_lowercase())
}

fn should_replace_candidate(current_extension: &str, new_extension: &str) -> bool {
    let current_rank = extension_rank(current_extension);
    let new_rank = extension_rank(new_extension);

    if new_rank < current_rank {
        return true;
    }

    if new_rank == current_rank {
        return new_extension < current_extension;
    }

    false
}

fn should_skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| EXCLUDED_DIRECTORY_NAMES.iter().any(|candidate| candidate.eq_ignore_ascii_case(value)))
        .unwrap_or(false)
}

fn visit_dir(dir: &Path, dedup: &mut HashMap<String, ScannedFile>) {
    if should_skip_directory(dir) {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            visit_dir(&path, dedup);
            continue;
        }

        let extension = match path.extension().and_then(|value| value.to_str()) {
            Some(value) => normalize_extension(value),
            None => continue,
        };

        if !is_supported_extension(&extension) {
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

        let dedup_group_key = make_dedup_group_key(parent, stem);
        let candidate = ScannedFile {
            full_path: path.to_string_lossy().to_string(),
            extension: extension.clone(),
            dedup_group_key: dedup_group_key.clone(),
        };

        match dedup.get(&dedup_group_key) {
            Some(existing)
                if !should_replace_candidate(existing.extension.as_str(), extension.as_str()) =>
            {
                continue;
            }
            _ => {
                dedup.insert(dedup_group_key, candidate);
            }
        }
    }
}

pub fn is_supported_extension(extension: &str) -> bool {
    let normalized = normalize_extension(extension);
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|candidate| *candidate == normalized)
}

pub fn scan(input: &ScanInput) -> ScanResult {
    let root_path = PathBuf::from(&input.root_path);
    if !root_path.exists() || !root_path.is_dir() {
        return ScanResult { files: Vec::new() };
    }

    let mut dedup: HashMap<String, ScannedFile> = HashMap::new();
    visit_dir(&root_path, &mut dedup);

    let mut files: Vec<ScannedFile> = dedup.into_values().collect();
    files.sort_by(|left, right| {
        left.full_path
            .to_ascii_lowercase()
            .cmp(&right.full_path.to_ascii_lowercase())
    });

    ScanResult { files }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn scan_recurses_and_filters_extensions() {
        let root = unique_temp_dir("scan-recurses");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("design1.pes"));
        create_file(&root.join("nested").join("design2.dst"));
        create_file(&root.join("nested").join("notes.txt"));

        let result = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
        });

        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().any(|file| file.extension == "pes"));
        assert!(result.files.iter().any(|file| file.extension == "dst"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_deduplicates_by_parent_and_stem_using_priority() {
        let root = unique_temp_dir("scan-dedup");
        fs::create_dir_all(&root).expect("root should be created");

        create_file(&root.join("same-name.pmv"));
        create_file(&root.join("same-name.pes"));

        let result = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
        });

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].extension, "pes");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_returns_empty_for_missing_root() {
        let root = unique_temp_dir("scan-missing");
        let result = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
        });

        assert!(result.files.is_empty());
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

        let result = scan(&ScanInput {
            root_path: root.to_string_lossy().to_string(),
        });

        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].full_path.ends_with("visible-design.pes"));

        let _ = fs::remove_dir_all(&root);
    }
}