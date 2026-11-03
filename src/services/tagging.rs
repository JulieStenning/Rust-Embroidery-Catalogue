// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

// File & Folder Rules — generic token-overlap matcher driven by the live tag catalogue
// and user-configurable / seeded word match dictionary (tag_synonyms).
//
// Design
// ------
// 1. Tokenise the filename stem and filepath into lowercase alphanumeric words.
// 2. Check the dynamic word-match dictionary (synonyms map, loaded from tag_synonyms
//    in the database). Inflection (singular ↔ plural) is automatically applied to both
//    synonym keywords and path tokens.
// 3. For every tag description in the live `valid_descriptions` catalogue (loaded
//    from the database at import time):
//       a. Normalise the description and split it into meaningful tokens.
//       b. If any single meaningful token of the tag appears in the path tokens
//          — in singular or plural form (powered by `Inflector`) — assign the tag.
//
// This is fully generic: any user-created tag is automatically matched as long as
// its words overlap with the file path or match a configured word-to-tag rule.

use inflector::Inflector;
use std::collections::{HashMap, HashSet};

// ─── Normalisation ───────────────────────────────────────────────────────

fn normalize_text(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len() * 2);
    let mut prev_char: Option<char> = None;

    for ch in value.chars() {
        if ch.is_alphabetic() {
            if let Some(prev) = prev_char {
                // Split on digit -> letter (e.g. "1Butterfly" -> "1 butterfly")
                // or lowercase -> uppercase (e.g. "MonarchButterfly" -> "monarch butterfly")
                if prev.is_numeric() || (prev.is_lowercase() && ch.is_uppercase()) {
                    normalized.push(' ');
                }
            }
            normalized.extend(ch.to_lowercase());
            prev_char = Some(ch);
        } else if ch.is_numeric() {
            if let Some(prev) = prev_char {
                // Split on letter -> digit (e.g. "Butterfly1" -> "butterfly 1")
                if prev.is_alphabetic() {
                    normalized.push(' ');
                }
            }
            normalized.push(ch);
            prev_char = Some(ch);
        } else {
            normalized.push(' ');
            prev_char = None;
        }
    }

    normalized
}

fn tokenize(value: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();

    // Primary: boundary-split normalized tokens (splits letter/digit and camelCase)
    for token in normalize_text(value).split_whitespace() {
        tokens.insert(token.to_string());
    }

    // Secondary: raw alphanumeric tokens without boundary splitting (keeps e.g. "butterfly1")
    let raw_alphanumeric: String = value
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect();
    for token in raw_alphanumeric.split_whitespace() {
        tokens.insert(token.to_string());
    }

    tokens
}

/// Significant tokens: tokens longer than 2 characters and not noise words like
/// "and", used for tag-description token matching to avoid spurious matches.
fn significant_tokens(value: &str) -> Vec<String> {
    normalize_text(value)
        .split_whitespace()
        .filter(|part| part.len() > 2 && *part != "and")
        .map(String::from)
        .collect()
}

// ─── Inflector helpers ───────────────────────────────────────────────────

fn singular_form(token: &str) -> String {
    token.to_singular()
}

fn plural_form(token: &str) -> String {
    token.to_plural()
}

fn token_matches_in_path(token: &str, path_tokens: &HashSet<String>) -> bool {
    if path_tokens.contains(token) {
        return true;
    }

    // Try singular-matching: the tag token is plural ("butterflies") and we
    // check whether its singular ("butterfly") appears in the path.
    let singular = singular_form(token);
    if singular != token && path_tokens.contains(&singular) {
        return true;
    }

    // Try plural-matching: the tag token is singular ("butterfly") and we
    // check whether its plural ("butterflies") appears in the path.
    let plural = plural_form(token);
    if plural != token && path_tokens.contains(&plural) {
        return true;
    }

    false
}

// ─── Primary matching logic ──────────────────────────────────────────────

const DOMAIN_STOP_WORDS: &[&str] = &[
    "embroidery",
    "embroidered",
    "embroideries",
    "stitch",
    "stitched",
    "stitches",
    "stitching",
];

/// Check whether a folder path segment should be ignored during tag matching.
/// Discards drive roots, container roots, technical machine embroidery format folders,
/// and excluded supplier/source/designer names.
pub fn is_ignored_path_segment(segment: &str, excluded_names: &[&str]) -> bool {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return true;
    }
    // Drive roots like "C:" or "F:"
    if trimmed.ends_with(':') && trimmed.len() <= 3 {
        return true;
    }

    let lower = trimmed.to_ascii_lowercase();
    let compact: String = lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if compact.is_empty() {
        return true;
    }

    // Container and data roots
    if matches!(
        compact.as_str(),
        "machineembroiderydesigns"
            | "embroiderycatalogue"
            | "embroiderycataloguedata"
            | "database"
            | "data"
    ) {
        return true;
    }

    // Common machine embroidery format folders
    if matches!(
        compact.as_str(),
        "jef"
            | "pes"
            | "dst"
            | "exp"
            | "vip"
            | "vp3"
            | "hus"
            | "xxx"
            | "pec"
            | "sew"
            | "shv"
            | "csd"
            | "art"
            | "eof"
            | "ecf"
            | "emb"
    ) {
        return true;
    }

    // Excluded Source or Designer names
    for name in excluded_names {
        let clean = name.trim();
        if clean.is_empty() {
            continue;
        }
        if lower == clean.to_ascii_lowercase() {
            return true;
        }
        let compact_name: String = clean
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        if !compact_name.is_empty() && compact == compact_name {
            return true;
        }
    }

    false
}

/// Tokenise the filename and non-ignored directory segments of the filepath,
/// filtering out domain stop words and excluded supplier/source folder segments.
pub fn extract_path_rule_tokens(
    filename: &str,
    filepath: &str,
    excluded_names: &[&str],
) -> HashSet<String> {
    let filename_stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(filename);

    let mut parts: Vec<String> = Vec::new();
    parts.push(filename_stem.to_string());

    for segment in filepath.split(['/', '\\']) {
        if !is_ignored_path_segment(segment, excluded_names) {
            parts.push(segment.to_string());
        }
    }

    let combined = parts.join(" ");
    let mut tokens = tokenize(&combined);
    tokens.retain(|tok| !DOMAIN_STOP_WORDS.contains(&tok.as_str()));
    tokens
}

/// Given a filename, full filepath, valid tag descriptions, dynamic synonym map,
/// and an optional list of excluded Source/Designer folder names, return matching tags.
pub fn suggest_path_rule_descriptions_with_exclusions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
    synonyms: &HashMap<String, Vec<String>>,
    excluded_names: &[&str],
) -> Vec<String> {
    if valid_descriptions.is_empty() {
        return Vec::new();
    }

    let path_tokens = extract_path_rule_tokens(filename, filepath, excluded_names);
    if path_tokens.is_empty() {
        return Vec::new();
    }

    let mut matched = HashSet::new();

    // ── Dynamic synonym map pass ──
    for (synonym, descriptions) in synonyms {
        let synonym_singular = singular_form(synonym);
        let synonym_plural = plural_form(synonym);
        if path_tokens.contains(synonym.as_str())
            || path_tokens.contains(&synonym_singular)
            || path_tokens.contains(&synonym_plural)
        {
            for description in descriptions {
                if let Some(canonical) = valid_descriptions
                    .iter()
                    .find(|candidate| candidate.eq_ignore_ascii_case(description))
                {
                    matched.insert(canonical.clone());
                }
            }
        }
    }

    // ── Generic token-overlap pass ────────────────────────────────────
    for description in valid_descriptions {
        if matched.contains(description) {
            continue; // already assigned via synonym map
        }

        let tag_tokens = significant_tokens(description);
        if tag_tokens.is_empty() {
            continue;
        }

        // OR: match if any single significant token of the tag overlaps with
        // the path tokens (singular ↔ plural aware).
        let any_token_matches = tag_tokens
            .iter()
            .any(|token| token_matches_in_path(token, &path_tokens));

        if any_token_matches {
            matched.insert(description.clone());
        }
    }

    // If the file is classified as an alphabet glyph (single letter, digit, or punctuation mark),
    // assign "Alphabets" and suppress "Words" so glyphs aren't tagged as word phrases.
    let is_glyph = crate::services::alphabet_detector::classify_glyph_name(filename)
        != crate::services::alphabet_detector::GlyphCategory::NonGlyph;
    if is_glyph {
        if let Some(alphabets_canonical) = valid_descriptions
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case("Alphabets"))
        {
            matched.insert(alphabets_canonical.clone());
        }
        if let Some(words_canonical) = valid_descriptions
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case("Words"))
        {
            matched.remove(words_canonical);
        }
    }

    let mut results: Vec<String> = matched.into_iter().collect();
    results.sort();
    results
}

/// Convenience overload maintaining backwards-compatibility when no exclusions are specified.
pub fn suggest_path_rule_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
    synonyms: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    suggest_path_rule_descriptions_with_exclusions(
        filename,
        filepath,
        valid_descriptions,
        synonyms,
        &[],
    )
}

#[cfg(test)]
#[path = "tagging_tests.rs"]
mod tests;
