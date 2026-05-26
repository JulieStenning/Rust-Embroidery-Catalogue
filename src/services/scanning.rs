// Scanning service contract for file discovery and dedup grouping.

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pes", "dst", "jef", "vp3", "exp", "hus", "xxx", "art",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanInput {
    pub root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    pub full_path: String,
    pub extension: String,
    pub dedup_group_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
}

pub fn is_supported_extension(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(extension))
}

pub fn scan(_input: &ScanInput) -> ScanResult {
    // Placeholder implementation for scaffold stage.
    ScanResult { files: Vec::new() }
}