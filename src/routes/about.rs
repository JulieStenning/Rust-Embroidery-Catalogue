use crate::services::about_documents;
use crate::services::about_documents::{AboutDocumentDetail, AboutDocumentSummary};

#[tauri::command]
pub fn get_about_documents() -> Vec<AboutDocumentSummary> {
    about_documents::get_about_documents()
}

#[tauri::command]
pub fn get_about_document(slug: String) -> Result<AboutDocumentDetail, String> {
    about_documents::get_about_document(slug)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn get_about_documents_returns_expected_entries() {
        let documents = get_about_documents();
        assert!(!documents.is_empty());
        assert!(documents.iter().any(|doc| doc.slug == "disclaimer"));
        assert!(documents.iter().any(|doc| doc.slug == "licence"));
    }

    #[test]
    fn get_about_document_returns_error_for_unknown_slug() {
        let result = get_about_document("unknown-slug".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn get_about_document_returns_error_for_blank_slug() {
        let result = get_about_document(String::new());
        assert!(result.is_err());
    }

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
