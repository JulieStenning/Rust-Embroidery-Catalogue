use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct AboutDocumentSummary {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub filename: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AboutDocumentDetail {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub filename: String,
    pub document_text: String,
}

#[derive(Debug, Clone, Copy)]
struct AboutDocumentSpec {
    slug: &'static str,
    title: &'static str,
    filename: &'static str,
    description: &'static str,
}

const DOCUMENTS: [AboutDocumentSpec; 6] = [
    AboutDocumentSpec {
        slug: "disclaimer",
        title: "Disclaimer",
        filename: "DISCLAIMER.html",
        description: "Important use-at-your-own-risk and limitation-of-liability information.",
    },
    AboutDocumentSpec {
        slug: "privacy",
        title: "Privacy",
        filename: "templates/info/PRIVACY.html",
        description: "Explains what data is stored locally and what optional AI features may send externally.",
    },
    AboutDocumentSpec {
        slug: "security",
        title: "Security",
        filename: "templates/info/SECURITY.html",
        description: "Guidance on secrets, API keys, portable deployments, and safe usage.",
    },
    AboutDocumentSpec {
        slug: "ai-tagging",
        title: "AI Tagging Guide",
        filename: "templates/info/AI_TAGGING.html",
        description: "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
    },
    AboutDocumentSpec {
        slug: "third-party-notices",
        title: "Third-Party Notices",
        filename: "THIRD_PARTY_NOTICES.html",
        description: "Licensing and attribution information for bundled and dependency software.",
    },
    AboutDocumentSpec {
        slug: "licence",
        title: "Licence",
        filename: "LICENCE",
        description: "The licence terms for the Embroidery Catalogue project itself.",
    },
];

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn resolve_document(slug: &str) -> Option<AboutDocumentSpec> {
    DOCUMENTS.into_iter().find(|doc| doc.slug == slug)
}

fn resolve_document_path(root: &Path, filename: &str) -> PathBuf {
    root.join(filename)
}

#[tauri::command]
pub fn get_about_documents() -> Vec<AboutDocumentSummary> {
    let root = project_root();

    DOCUMENTS
        .into_iter()
        .map(|doc| {
            let path = resolve_document_path(&root, doc.filename);
            AboutDocumentSummary {
                slug: doc.slug.to_string(),
                title: doc.title.to_string(),
                description: doc.description.to_string(),
                filename: doc.filename.to_string(),
                available: path.exists(),
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_about_document(slug: String) -> Result<AboutDocumentDetail, String> {
    let normalized_slug = slug.trim().to_lowercase();
    let doc =
        resolve_document(&normalized_slug).ok_or_else(|| "Document not found.".to_string())?;

    let path = resolve_document_path(&project_root(), doc.filename);
    if !path.exists() {
        return Err("Document file is missing.".to_string());
    }

    let document_text = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read document '{}': {}", doc.filename, error))?;

    Ok(AboutDocumentDetail {
        slug: doc.slug.to_string(),
        title: doc.title.to_string(),
        description: doc.description.to_string(),
        filename: doc.filename.to_string(),
        document_text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // -----------------------------------------------------------------------
    // project_root
    // -----------------------------------------------------------------------

    #[test]
    fn project_root_points_to_cargo_manifest_dir() {
        let root = project_root();
        assert!(root.exists(), "project_root should exist");
        assert!(root.join("Cargo.toml").exists(), "project_root should contain Cargo.toml");
    }

    // -----------------------------------------------------------------------
    // resolve_document
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_document_finds_all_valid_slugs() {
        for doc in &DOCUMENTS {
            let result = resolve_document(doc.slug);
            assert!(result.is_some(), "slug '{}' should resolve", doc.slug);
            let resolved = result.unwrap();
            assert_eq!(resolved.slug, doc.slug);
            assert_eq!(resolved.title, doc.title);
            assert_eq!(resolved.filename, doc.filename);
            assert_eq!(resolved.description, doc.description);
        }
    }

    #[test]
    fn resolve_document_returns_none_for_invalid_slug() {
        assert!(resolve_document("nonexistent").is_none());
    }

    #[test]
    fn resolve_document_returns_none_for_empty_string() {
        assert!(resolve_document("").is_none());
    }

    #[test]
    fn resolve_document_is_case_sensitive() {
        // resolve_document does exact matching (case normalisation is in get_about_document)
        assert!(resolve_document("Disclaimer").is_none());
        assert!(resolve_document("DISCLAIMER").is_none());
        assert!(resolve_document("LICENCE").is_none());
    }

    // -----------------------------------------------------------------------
    // resolve_document_path
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_document_path_joins_correctly() {
        let root = Path::new("/base");
        let path = resolve_document_path(root, "docs/file.html");
        assert_eq!(path, Path::new("/base/docs/file.html"));
    }

    #[test]
    fn resolve_document_path_handles_root_with_trailing_slash() {
        // Windows-style path
        let root = Path::new("C:\\project");
        let path = resolve_document_path(root, "sub/file.txt");
        let expected: PathBuf = ["C:\\project", "sub/file.txt"].iter().collect();
        assert_eq!(path, expected);
    }

    // -----------------------------------------------------------------------
    // DOCUMENTS constant integrity
    // -----------------------------------------------------------------------

    #[test]
    fn documents_constant_has_no_duplicate_slugs() {
        let mut seen = std::collections::HashSet::new();
        for doc in &DOCUMENTS {
            assert!(
                seen.insert(doc.slug),
                "duplicate slug '{}' found in DOCUMENTS",
                doc.slug
            );
        }
    }

    #[test]
    fn documents_all_slugs_are_lowercase() {
        for doc in &DOCUMENTS {
            assert_eq!(
                doc.slug,
                doc.slug.to_lowercase(),
                "slug '{}' is not lowercase",
                doc.slug
            );
        }
    }

    #[test]
    fn documents_all_filenames_and_titles_and_descriptions_are_non_empty() {
        for doc in &DOCUMENTS {
            assert!(!doc.filename.is_empty(), "slug '{}' has empty filename", doc.slug);
            assert!(!doc.title.is_empty(), "slug '{}' has empty title", doc.slug);
            assert!(
                !doc.description.is_empty(),
                "slug '{}' has empty description",
                doc.slug
            );
        }
    }

    // -----------------------------------------------------------------------
    // get_about_documents
    // -----------------------------------------------------------------------

    #[test]
    fn get_about_documents_returns_all_six() {
        let docs = get_about_documents();
        assert_eq!(docs.len(), 6, "should return exactly 6 documents");
    }

    #[test]
    fn get_about_documents_fields_match_constants() {
        let docs = get_about_documents();
        for (spec, summary) in DOCUMENTS.iter().zip(docs.iter()) {
            assert_eq!(summary.slug, spec.slug);
            assert_eq!(summary.title, spec.title);
            assert_eq!(summary.description, spec.description);
            assert_eq!(summary.filename, spec.filename);
        }
    }

    #[test]
    fn get_about_documents_all_available_on_windows_or_ci() {
        // On case-insensitive filesystems (Windows, macOS) all files resolve;
        // on case-sensitive Linux the uppercase constant vs lowercase file may fail.
        let docs = get_about_documents();
        for doc in &docs {
            assert!(
                doc.available,
                "document '{}' (filename: {}) should be available. \
                 If this fails on a case-sensitive filesystem the issue is the \
                 constant filename casing not matching the on-disk file.",
                doc.slug,
                doc.filename
            );
        }
    }

    // -----------------------------------------------------------------------
    // get_about_document — happy paths
    // -----------------------------------------------------------------------

    #[test]
    fn get_about_document_disclaimer() {
        let result = get_about_document("disclaimer".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "disclaimer");
        assert_eq!(detail.title, "Disclaimer");
        assert!(detail.document_text.contains("Embroidery Catalogue — Application Disclaimer"));
    }

    #[test]
    fn get_about_document_privacy() {
        let result = get_about_document("privacy".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "privacy");
        assert_eq!(detail.title, "Privacy");
        assert!(detail.document_text.contains("Privacy Policy - Embroidery Catalogue"));
    }

    #[test]
    fn get_about_document_security() {
        let result = get_about_document("security".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "security");
        assert_eq!(detail.title, "Security");
        assert!(detail.document_text.contains("Security Policy"));
    }

    #[test]
    fn get_about_document_ai_tagging() {
        let result = get_about_document("ai-tagging".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "ai-tagging");
        assert_eq!(detail.title, "AI Tagging Guide");
        assert!(detail.document_text.contains("AI-Assisted Auto-Tagging - Embroidery Catalogue"));
    }

    #[test]
    fn get_about_document_third_party_notices() {
        let result = get_about_document("third-party-notices".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "third-party-notices");
        assert_eq!(detail.title, "Third-Party Notices");
        assert!(detail.document_text.contains("Third-Party Notices - Embroidery Catalogue"));
    }

    #[test]
    fn get_about_document_licence() {
        let result = get_about_document("licence".to_string());
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.slug, "licence");
        assert_eq!(detail.title, "Licence");
        assert!(detail.document_text.contains("Copyright (C) 2026 Julie Stenning"));
    }

    // -----------------------------------------------------------------------
    // get_about_document — slug normalisation
    // -----------------------------------------------------------------------

    #[test]
    fn get_about_document_normalises_whitespace() {
        let result = get_about_document("  disclaimer  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().slug, "disclaimer");
    }

    #[test]
    fn get_about_document_normalises_case() {
        for slug_variant in &["Disclaimer", "DISCLAIMER", "dIsClAiMeR"] {
            let result = get_about_document(slug_variant.to_string());
            assert!(
                result.is_ok(),
                "slug variant '{}' should normalise to 'disclaimer'",
                slug_variant
            );
            assert_eq!(
                result.unwrap().slug,
                "disclaimer",
                "slug variant '{}' failed",
                slug_variant
            );
        }
    }

    #[test]
    fn get_about_document_normalises_upper_slugs() {
        for slug in &["PRIVACY", "SECURITY", "AI-TAGGING", "THIRD-PARTY-NOTICES", "LICENCE"] {
            let result = get_about_document(slug.to_string());
            assert!(
                result.is_ok(),
                "upper-case slug '{}' should normalise",
                slug
            );
        }
    }

    #[test]
    fn get_about_document_normalises_mixed_case_with_whitespace() {
        let result = get_about_document("  LiCeNcE  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().slug, "licence");
    }

    // -----------------------------------------------------------------------
    // get_about_document — error cases
    // -----------------------------------------------------------------------

    #[test]
    fn get_about_document_invalid_slug_returns_error() {
        let result = get_about_document("nonexistent".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Document not found.");
    }

    #[test]
    fn get_about_document_empty_string_returns_error() {
        let result = get_about_document(String::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Document not found.");
    }

    #[test]
    fn get_about_document_whitespace_only_returns_error() {
        let result = get_about_document("   ".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Document not found.");
    }

    // -----------------------------------------------------------------------
    // Serialisation smoke tests
    // -----------------------------------------------------------------------

    #[test]
    fn about_document_summary_serialises_to_json() {
        let summary = AboutDocumentSummary {
            slug: "test-slug".to_string(),
            title: "Test Title".to_string(),
            description: "Test description.".to_string(),
            filename: "test.html".to_string(),
            available: true,
        };
        let json = serde_json::to_string(&summary).expect("serialisation should succeed");
        assert!(json.contains("\"slug\":\"test-slug\""));
        assert!(json.contains("\"title\":\"Test Title\""));
        assert!(json.contains("\"description\":\"Test description.\""));
        assert!(json.contains("\"filename\":\"test.html\""));
        assert!(json.contains("\"available\":true"));
    }

    #[test]
    fn about_document_detail_serialises_to_json() {
        let detail = AboutDocumentDetail {
            slug: "test-detail".to_string(),
            title: "Detail Title".to_string(),
            description: "A detailed description.".to_string(),
            filename: "detail.html".to_string(),
            document_text: "<html>Hello</html>".to_string(),
        };
        let json = serde_json::to_string(&detail).expect("serialisation should succeed");
        assert!(json.contains("\"slug\":\"test-detail\""));
        assert!(json.contains("\"title\":\"Detail Title\""));
        assert!(json.contains("\"description\":\"A detailed description.\""));
        assert!(json.contains("\"filename\":\"detail.html\""));
        assert!(json.contains("\"document_text\":\"<html>Hello</html>\""));
    }
}
