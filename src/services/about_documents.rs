// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use serde::Serialize;

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

/// A single About document.
///
/// `content` is embedded at compile time via `include_str!` rather than read
/// from disk at runtime. The previous implementation resolved documents against
/// `env!("CARGO_MANIFEST_DIR")`, which expands to the *build machine's* repo
/// path and so does not exist in an installed release: every document reported
/// `available: false` and the detail command returned `not_found`.
///
/// Embedding follows the pattern already used elsewhere in this crate
/// (`include_bytes!` for the seed database in `paths.rs`, `sqlx::migrate!` for
/// migrations) and removes filesystem resolution entirely, so dev builds, tests
/// and installed releases all behave identically. Cargo tracks `include_str!`
/// targets through rustc dep-info, so editing a document still triggers a
/// rebuild without any `build.rs` change.
#[derive(Debug, Clone, Copy)]
struct AboutDocumentSpec {
    slug: &'static str,
    title: &'static str,
    /// Repo-relative path, surfaced to the UI. Must match the on-disk casing
    /// exactly, because it mirrors the `include_str!` target below.
    filename: &'static str,
    description: &'static str,
    content: &'static str,
}

const DOCUMENTS: [AboutDocumentSpec; 5] = [
    AboutDocumentSpec {
        slug: "disclaimer",
        title: "Disclaimer",
        filename: "DISCLAIMER.html",
        description: "Important use-at-your-own-risk and limitation-of-liability information.",
        content: include_str!("../../DISCLAIMER.html"),
    },
    AboutDocumentSpec {
        slug: "privacy",
        title: "Privacy",
        filename: "templates/info/privacy.html",
        description: "Explains what data is stored locally and what optional AI features may send externally.",
        content: include_str!("../../templates/info/privacy.html"),
    },
    AboutDocumentSpec {
        slug: "security",
        title: "Security",
        filename: "templates/info/security.html",
        description: "Guidance on secrets, API keys, portable deployments, and safe usage.",
        content: include_str!("../../templates/info/security.html"),
    },
    AboutDocumentSpec {
        slug: "ai-tagging",
        title: "AI Tagging & Batch Operations Guide",
        filename: "docs/User-Facing-Guidance/BATCH_OPERATIONS_BACKFILL.md",
        description: "How to run Gemini Vision tagging from Batch Operations, set up a Google API key, and understand usage costs.",
        content: include_str!("../../docs/User-Facing-Guidance/BATCH_OPERATIONS_BACKFILL.md"),
    },
    AboutDocumentSpec {
        slug: "data-storage",
        title: "Data Storage & External Drives Guide",
        filename: "docs/User-Facing-Guidance/DATA_STORAGE_GUIDE.md",
        description: "How Embroidery Catalogue stores your designs and database, and how to choose external storage.",
        content: include_str!("../../docs/User-Facing-Guidance/DATA_STORAGE_GUIDE.md"),
    },
];

fn resolve_document(slug: &str) -> Option<AboutDocumentSpec> {
    DOCUMENTS.into_iter().find(|doc| doc.slug == slug)
}

pub fn get_about_documents() -> Vec<AboutDocumentSummary> {
    DOCUMENTS
        .into_iter()
        .map(|doc| AboutDocumentSummary {
            slug: doc.slug.to_string(),
            title: doc.title.to_string(),
            description: doc.description.to_string(),
            filename: doc.filename.to_string(),
            // The content is embedded, so availability can only be false if a
            // source document was accidentally emptied.
            available: !doc.content.trim().is_empty(),
        })
        .collect()
}

pub fn get_about_document(slug: String) -> Result<AboutDocumentDetail, AppError> {
    let normalized_slug = slug.trim().to_lowercase();
    let doc = resolve_document(&normalized_slug)
        .ok_or_else(|| AppError::not_found("document", Some(slug.clone())))?;

    // Mirrors the previous "document file missing" guard: with the content
    // embedded there is no path to check, so an empty body is the only way a
    // document can be unavailable.
    if doc.content.trim().is_empty() {
        return Err(AppError::not_found("document", Some(slug)));
    }

    Ok(AboutDocumentDetail {
        slug: doc.slug.to_string(),
        title: doc.title.to_string(),
        description: doc.description.to_string(),
        filename: doc.filename.to_string(),
        document_text: doc.content.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_document_returns_none_for_empty_string() {
        assert!(resolve_document("").is_none());
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
            vec![
                "disclaimer",
                "privacy",
                "security",
                "ai-tagging",
                "data-storage"
            ]
        );
        for doc in &docs {
            assert!(!doc.title.is_empty());
            assert!(!doc.description.is_empty());
            assert!(!doc.filename.is_empty());
            // Content is embedded at compile time, so every document must be
            // available in dev builds, tests and installed releases alike.
            assert!(doc.available, "{} should always be available", doc.slug);
        }
    }

    /// Proves the `include_str!` embedding actually resolved: every document
    /// carries real content, and the detail command returns exactly that
    /// content. This replaces the old on-disk `path.exists()` cross-check,
    /// which could only ever pass on a machine with the source tree present.
    #[test]
    fn all_documents_embed_non_empty_content() {
        for doc in DOCUMENTS {
            assert!(
                !doc.content.trim().is_empty(),
                "embedded content for {} should not be empty",
                doc.slug
            );

            let detail = get_about_document(doc.slug.to_string())
                .unwrap_or_else(|error| panic!("{} should load: {error:?}", doc.slug));
            assert_eq!(
                detail.document_text, doc.content,
                "document_text should be the embedded content for {}",
                doc.slug
            );
            assert_eq!(detail.filename, doc.filename);
        }
    }

    #[test]
    fn get_about_document_returns_detail_for_known_slug() {
        let detail = get_about_document("disclaimer".to_string()).expect("disclaimer should load");
        assert_eq!(detail.slug, "disclaimer");
        assert_eq!(detail.title, "Disclaimer");
        assert_eq!(detail.filename, "DISCLAIMER.html");
        assert!(!detail.document_text.is_empty());

        // Slug is trimmed and lowercased before resolution.
        let normalized =
            get_about_document("  AI-TAGGING  ".to_string()).expect("ai-tagging should load");
        assert_eq!(normalized.slug, "ai-tagging");
        assert_eq!(normalized.title, "AI Tagging & Batch Operations Guide");
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
