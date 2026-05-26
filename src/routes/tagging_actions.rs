use crate::services::auto_tagging;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaggingActionRequest {
    pub request_override: Option<bool>,
    pub settings_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggingActionPreview {
    pub enabled: bool,
    pub tier_order: Vec<String>,
}

#[tauri::command]
pub fn preview_tagging_action(request: TaggingActionRequest) -> Result<TaggingActionPreview, String> {
    let precedence = auto_tagging::TaggingPrecedence {
        request_override: request.request_override,
        settings_default: request.settings_default,
        hard_default: true,
    };

    let tier_order = auto_tagging::ordered_tiers()
        .iter()
        .map(|tier| format!("{:?}", tier))
        .collect();

    Ok(TaggingActionPreview {
        enabled: auto_tagging::resolve_enabled(&precedence),
        tier_order,
    })
}