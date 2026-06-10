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
        filename: "PRIVACY.html",
        description: "Explains what data is stored locally and what optional AI features may send externally.",
    },
    AboutDocumentSpec {
        slug: "security",
        title: "Security",
        filename: "SECURITY.md",
        description: "Guidance on secrets, API keys, portable deployments, and safe usage.",
    },
    AboutDocumentSpec {
        slug: "ai-tagging",
        title: "AI Tagging Guide",
        filename: "docs/User-Facing-Guidance/AI_TAGGING.md",
        description: "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
    },
    AboutDocumentSpec {
        slug: "third-party-notices",
        title: "Third-Party Notices",
        filename: "THIRD_PARTY_NOTICES.md",
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
    let doc = resolve_document(&normalized_slug).ok_or_else(|| "Document not found.".to_string())?;

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