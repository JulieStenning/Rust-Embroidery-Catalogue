// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! Diagnostic helpers for SQLite database errors, specifically detecting
//! storage capacity and FAT32 file size limit errors (`SQLITE_FULL`, code 13).

use std::fs;
use std::path::Path;

/// Threshold in bytes above which a database is considered approaching the
/// FAT32 4 GiB (4,294,967,295 bytes) single-file size limit.
/// 3.8 GiB = 3.8 * 1024 * 1024 * 1024 = 4,080,218,931 bytes.
pub const FAT32_LIMIT_WARN_THRESHOLD_BYTES: u64 = 3_800_000_000;

/// Check if a given file size in bytes is close to the 4 GB FAT32 single-file limit.
#[inline]
pub fn is_near_fat32_limit(size_bytes: u64) -> bool {
    size_bytes >= FAT32_LIMIT_WARN_THRESHOLD_BYTES
}

/// Format bytes into a human-readable string (e.g., "3.99 GB", "512.4 MB").
pub fn format_bytes_human(bytes: u64) -> String {
    const GIB: f64 = (1024 * 1024 * 1024) as f64;
    const MIB: f64 = (1024 * 1024) as f64;
    const KIB: f64 = 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GIB {
        format!("{:.2} GB", bytes_f / GIB)
    } else if bytes_f >= MIB {
        format!("{:.1} MB", bytes_f / MIB)
    } else if bytes_f >= KIB {
        format!("{:.1} KB", bytes_f / KIB)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Check whether an error message or sqlx error represents `SQLITE_FULL` (error code 13: "database or disk is full").
pub fn is_sqlite_full_error(err_str: &str) -> bool {
    let lower = err_str.to_ascii_lowercase();
    lower.contains("sqlite_full")
        || lower.contains("database or disk is full")
        || lower.contains("code: 13")
        || lower.contains("error 13")
        || lower.contains("disk full")
}

/// Build the diagnostic FAT32 guidance message.
pub fn fat32_diagnostic_message(db_size_bytes: Option<u64>) -> String {
    let size_part = match db_size_bytes {
        Some(bytes) => format!(
            "Current database file size: {}.\n\n",
            format_bytes_human(bytes)
        ),
        None => String::new(),
    };

    format!(
        "Database or disk is full (SQLite error 13).\n\
         {size_part}\
         Your catalogue data root maybe on a FAT32-formatted drive (common with SD cards and USB flash drives). \
         FAT32 imposes a strict 4 GB maximum single-file size limit regardless of how much free space remains on the card.\n\n\
         Recommended actions:\n\
         1. Check the size of your database in Settings > Maintenance.\n\
         2. Back up your catalogue and reformat the SD card using exFAT or NTFS (which support files larger than 4 GB).\n\
         3. Alternatively, move the catalogue data root to an exFAT or NTFS location."
    )
}

/// Enrich a database error message with FAT32 diagnostic details if `SQLITE_FULL` is detected.
pub fn enrich_db_error_message(err_str: &str, db_path: Option<&Path>) -> String {
    if is_sqlite_full_error(err_str) {
        let db_size = db_path.and_then(|p| fs::metadata(p).ok().map(|m| m.len()));
        format!("{}\n\n{}", err_str, fat32_diagnostic_message(db_size))
    } else {
        err_str.to_string()
    }
}

/// Enrich an `sqlx::Error` with FAT32 diagnostic details if `SQLITE_FULL` is detected.
pub fn enrich_sqlx_error(err: &sqlx::Error, db_path: Option<&Path>) -> String {
    let err_str = err.to_string();
    enrich_db_error_message(&err_str, db_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_human() {
        assert_eq!(format_bytes_human(4_284_967_296), "3.99 GB");
        assert_eq!(format_bytes_human(1024 * 1024 * 100), "100.0 MB");
        assert_eq!(format_bytes_human(1024 * 50), "50.0 KB");
        assert_eq!(format_bytes_human(500), "500 bytes");
    }

    #[test]
    fn test_is_near_fat32_limit() {
        assert!(!is_near_fat32_limit(1_000_000_000));
        assert!(is_near_fat32_limit(3_800_000_000));
        assert!(is_near_fat32_limit(4_284_967_296));
    }

    #[test]
    fn test_is_sqlite_full_error() {
        assert!(is_sqlite_full_error(
            "error returned from database: (code: 13) database or disk is full"
        ));
        assert!(is_sqlite_full_error(
            "SQLITE_FULL: database or disk is full"
        ));
        assert!(is_sqlite_full_error("database or disk is full"));
        assert!(!is_sqlite_full_error("table not found"));
        assert!(!is_sqlite_full_error("database is locked"));
    }

    #[test]
    fn test_fat32_diagnostic_message_contains_exact_wording() {
        let msg = fat32_diagnostic_message(Some(4_284_967_296));
        assert!(msg.contains(
            "Your catalogue data root maybe on a FAT32-formatted drive (common with SD cards and USB flash drives). FAT32 imposes a strict 4 GB maximum single-file size limit regardless of how much free space remains on the card."
        ));
        assert!(msg.contains("Current database file size: 3.99 GB."));
        assert!(msg.contains("Settings > Maintenance"));
        assert!(msg.contains("exFAT or NTFS"));
    }

    #[test]
    fn test_enrich_db_error_message_on_sqlite_full() {
        let raw = "error returned from database: (code: 13) database or disk is full";
        let enriched = enrich_db_error_message(raw, None);
        assert!(enriched.contains("Your catalogue data root maybe on a FAT32-formatted drive"));
        assert!(enriched.contains(raw));
    }

    #[test]
    fn test_enrich_db_error_message_non_full_passes_through() {
        let raw = "database is locked";
        let enriched = enrich_db_error_message(raw, None);
        assert_eq!(enriched, raw);
    }
}
