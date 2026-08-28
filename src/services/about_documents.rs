use crate::error::AppError;
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

const DOCUMENTS: [AboutDocumentSpec; 5] = [
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
        filename: "docs/User-Facing-Guidance/AI_TAGGING.md",
        description: "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
    },
    AboutDocumentSpec {
        slug: "data-storage",
        title: "Data Storage & External Drives Guide",
        filename: "docs/User-Facing-Guidance/DATA_STORAGE_GUIDE.md",
        description: "How Embroidery Catalogue stores your designs and database, and how to choose external storage.",
    },

];

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn resolve_document(slug: &str) -> Option<AboutDocumentSpec> {
    DOCUMENTS.into_iter().find(|doc| doc.slug == slug)
}

fn resolve_document_path(root: &Path, filename: &str) -> PathBuf {
    root.join(filename)
}

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

pub fn get_about_document(slug: String) -> Result<AboutDocumentDetail, AppError> {
    let normalized_slug = slug.trim().to_lowercase();
    let doc = resolve_document(&normalized_slug)
        .ok_or_else(|| AppError::not_found("document", Some(slug)))?;

    let path = resolve_document_path(&project_root(), doc.filename);
    if !path.exists() {
        return Err(AppError::not_found(
            "document file",
            Some(doc.filename.to_string()),
        ));
    }

    let document_text = fs::read_to_string(&path).map_err(|error| {
        AppError::io(format!(
            "Could not read document '{}': {}",
            doc.filename, error
        ))
    })?;

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

    #[test]
    fn project_root_points_to_cargo_manifest_dir() {
        let root = project_root();
        assert!(root.exists(), "project_root should exist");
        assert!(
            root.join("Cargo.toml").exists(),
            "project_root should contain Cargo.toml"
        );
    }

    #[test]
    fn resolve_document_returns_none_for_empty_string() {
        assert!(resolve_document("").is_none());
    }

    #[test]
    fn resolve_document_path_joins_correctly() {
        let root = Path::new("/base");
        let path = resolve_document_path(root, "docs/file.html");
        assert_eq!(path, Path::new("/base/docs/file.html"));
    }

    #[test]
    fn resolve_document_returns_spec_for_known_slug() {
        let doc = resolve_document("disclaimer").expect("disclaimer should resolve");
        assert_eq!(doc.slug, "disclaimer");
        assert_eq!(doc.title, "Disclaimer");
        assert_eq!(doc.filename, "DISCLAIMER.html");
        assert!(resolve_document("privacy").is_some());
        assert!(resolve_document("data-storage").is_some());
        assert!(resolve_document("unknown").is_none());
    }

    #[test]
    fn get_about_documents_lists_all_supported_documents() {
        let docs = get_about_documents();
        assert_eq!(docs.len(), 5);
        let slugs: Vec<&str> = docs.iter().map(|d| d.slug.as_str()).collect();
        assert_eq!(
            slugs,
            vec!["disclaimer", "privacy", "security", "ai-tagging", "data-storage"]
        );
        for doc in &docs {
            assert!(!doc.title.is_empty());
            assert!(!doc.description.is_empty());
            assert!(!doc.filename.is_empty());
            // available must reflect the real on-disk state of the bundled file.
            let path = resolve_document_path(&project_root(), &doc.filename);
            assert_eq!(
                doc.available,
                path.exists(),
                "availability mismatch for {}",
                doc.slug
            );
        }
    }

    #[test]
    fn get_about_document_returns_detail_for_known_slug() {
        let detail =
            get_about_document("disclaimer".to_string()).expect("disclaimer should load");
        assert_eq!(detail.slug, "disclaimer");
        assert_eq!(detail.title, "Disclaimer");
        assert_eq!(detail.filename, "DISCLAIMER.html");
        assert!(!detail.document_text.is_empty());

        // Slug is trimmed and lowercased before resolution.
        let normalized =
            get_about_document("  AI-TAGGING  ".to_string()).expect("ai-tagging should load");
        assert_eq!(normalized.slug, "ai-tagging");
        assert_eq!(normalized.title, "AI Tagging Guide");
        assert!(!normalized.document_text.is_empty());
    }

    #[test]
    fn get_about_document_errors_on_unknown_slug() {
        let err = get_about_document("nonsense".to_string()).unwrap_err();
        assert_eq!(
            err,
            AppError::not_found("document", Some("nonsense".to_string()))
        );
    }
}
