// Auto-tagging orchestration contract.
// Implementation intentionally deferred to migration implementation passes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaggingTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Default)]
pub struct TaggingPrecedence {
    pub request_override: Option<bool>,
    pub settings_default: Option<bool>,
    pub hard_default: bool,
}

pub fn resolve_enabled(precedence: &TaggingPrecedence) -> bool {
    precedence
        .request_override
        .or(precedence.settings_default)
        .unwrap_or(precedence.hard_default)
}

pub fn ordered_tiers() -> [TaggingTier; 3] {
    [TaggingTier::Tier1, TaggingTier::Tier2, TaggingTier::Tier3]
}