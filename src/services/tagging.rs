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
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
}

fn tokenize(value: &str) -> HashSet<String> {
    normalize_text(value)
        .split_whitespace()
        .map(String::from)
        .collect()
}

/// Significant tokens: tokens longer than 2 characters, used for tag-description
/// token matching to avoid matches on noise words like "of", "a", "&", "an".
fn significant_tokens(value: &str) -> Vec<String> {
    normalize_text(value)
        .split_whitespace()
        .filter(|part| part.len() > 2)
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

/// Given a filename, full filepath, the set of valid tag descriptions from
/// the database, and the dynamic synonym map (keyword -> list of tag descriptions),
/// return the sorted list of descriptions that match.
///
/// Matching is **any-token OR**: if **any single** significant token of a tag
/// description or configured word-match appears in the path (in singular or plural form),
/// the tag is assigned.
pub fn suggest_path_rule_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
    synonyms: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    if valid_descriptions.is_empty() {
        return Vec::new();
    }

    let filename_stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(filename);
    let combined = format!("{} {}", filename_stem, filepath);
    let path_tokens = tokenize(&combined);
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

    let mut results: Vec<String> = matched.into_iter().collect();
    results.sort();
    results
}

#[cfg(test)]
#[path = "tagging_tests.rs"]
mod tests;
