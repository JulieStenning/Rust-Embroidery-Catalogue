// File & Folder Rules — generic token-overlap matcher driven by the live tag catalogue.
//
// Design
// ------
// 1. Tokenise the filename stem and filepath into lowercase alphanumeric words.
// 2. For every tag description in the live `valid_descriptions` catalogue (loaded
//    from the database at import time):
//       a. Normalise the description and split it into meaningful tokens.
//       b. If **any single** meaningful token of the tag appears in the path tokens
//          — in singular or plural form (powered by `Inflector`) — assign the tag.
// 3. A small built-in synonym map bridges aliases inflection can never derive
//    (kitten → Cats, puppy → Dogs, font/monogram/upper/lower → Words and Letters,
//    xmas → Christmas, floral → Flowers, baby → Children & Toys).  Synonym target
//    descriptions are resolved case-insensitively against the live catalogue.
//
// This is fully generic: any user-created tag is automatically matched as long as
// its words overlap with the file path, with no code or config changes required.

use inflector::Inflector;
use std::collections::HashSet;

// ─── Synonym map — only genuinely undecidable aliases ────────────────────
//
// Words that inflection can *never* derive from a tag description (e.g.
// "kitten" from "Cats", "floral" from "Flowers").  Everything else is handled
// generically by singular ↔ plural token overlap.

const SYNONYM_MAP: [(&str, &str); 11] = [
    ("kitten", "Cats"),
    ("puppy", "Dogs"),
    ("font", "Words and Letters"),
    ("monogram", "Words and Letters"),
    ("upper", "Words and Letters"),
    ("lower", "Words and Letters"),
    ("uppercase", "Words and Letters"),
    ("lowercase", "Words and Letters"),
    ("xmas", "Christmas"),
    ("floral", "Flowers"),
    ("baby", "Children & Toys"),
];

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
    // Inflector's to_singular() works on the entire token; for tokens that are
    // already singular it returns the same token, so this is always safe to call.
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

/// Given a filename, full filepath, and the set of valid tag descriptions from
/// the database, return the sorted list of descriptions that match.
///
/// Matching is **any-token OR**: if **any single** significant token of a tag
/// description appears in the path (in singular or plural form), the tag is
/// assigned.  This correctly handles compound tags like "Borders & Frames"
/// (folder "Borders" or "Frame" both match) and supports any user-created tag
/// automatically.
pub fn suggest_path_rule_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
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

    // ── Synonym-map pass (tiny, genuinely undecidable aliases only) ──
    for (synonym, description) in SYNONYM_MAP {
        // Resolve the catalogue entry case-insensitively so a tag stored with a
        // different casing still resolves, and carry the catalogue's own casing
        // forward (downstream tag-id lookups are exact-match).
        let Some(canonical) = valid_descriptions
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(description))
        else {
            continue;
        };

        // Check the synonym itself, its singular, and its plural against the
        // path tokens — folder names may be plural ("Kittens") while the
        // synonym is singular ("kitten"), and vice versa.
        let synonym_singular = singular_form(synonym);
        let synonym_plural = plural_form(synonym);
        if path_tokens.contains(synonym)
            || path_tokens.contains(&synonym_singular)
            || path_tokens.contains(&synonym_plural)
        {
            matched.insert(canonical.clone());
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
mod tests {
    use super::*;

    // ─── unit helpers ───────────────────────────────────────────────────

    #[test]
    fn normalize_text_replaces_punctuation_with_spaces() {
        let result = normalize_text("Borders & Frames");
        assert_eq!(result, "borders   frames");
    }

    #[test]
    fn significant_tokens_filters_short_words() {
        let tokens = significant_tokens("a big cat and a dog");
        assert_eq!(tokens, vec!["big", "cat", "and", "dog"]);
    }

    // ─── inflector sanity ──────────────────────────────────────────────

    #[test]
    fn inflector_singular_handles_butterflies() {
        assert_eq!(singular_form("butterflies"), "butterfly");
    }

    #[test]
    fn inflector_singular_handles_babies() {
        assert_eq!(singular_form("babies"), "baby");
    }

    #[test]
    fn inflector_singular_handles_monograms() {
        assert_eq!(singular_form("monograms"), "monogram");
    }

    #[test]
    fn inflector_plural_handles_butterfly() {
        assert_eq!(plural_form("butterfly"), "butterflies");
    }

    #[test]
    fn inflector_plural_handles_fairy() {
        assert_eq!(plural_form("fairy"), "fairies");
    }

    // ─── suggest_path_rule_descriptions ────────────────────────────────────

    #[test]
    fn suggest_path_rule_compound_tag_borders() {
        let valid = HashSet::from(["Borders & Frames".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Borders/somefile.pes", &valid);
        assert!(
            matched.contains(&"Borders & Frames".to_string()),
            "folder 'Borders' should match 'Borders & Frames' via token overlap: {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_compound_tag_frame() {
        let valid = HashSet::from(["Borders & Frames".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Frame/design.pes", &valid);
        assert!(
            matched.contains(&"Borders & Frames".to_string()),
            "folder 'Frame' should match 'Borders & Frames' via inflected token overlap: {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_compound_tag_angels() {
        let valid = HashSet::from(["Angels & Fairies".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Angels/design.pes", &valid);
        assert!(
            matched.contains(&"Angels & Fairies".to_string()),
            "folder 'Angels' should match 'Angels & Fairies': {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_compound_tag_fairies() {
        let valid = HashSet::from(["Angels & Fairies".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Fairies/design.pes", &valid);
        assert!(
            matched.contains(&"Angels & Fairies".to_string()),
            "folder 'Fairies' should match 'Angels & Fairies' via inflection: {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_synonym_maps_monogram_to_words_and_letters() {
        let valid = HashSet::from(["Words and Letters".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Monogram/design.pes", &valid);
        assert!(
            matched.contains(&"Words and Letters".to_string()),
            "folder 'Monogram' should match 'Words and Letters' via the 'monogram' synonym: {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_synonym_maps_font_to_words_and_letters() {
        let valid = HashSet::from(["Words and Letters".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Font/design.pes", &valid);
        assert!(
            matched.contains(&"Words and Letters".to_string()),
            "folder 'Font' should match 'Words and Letters' via the 'font' synonym: {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_compound_tag_butterfly_folder() {
        let valid = HashSet::from(["Butterflies & Insects".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Butterfly/design.pes", &valid);
        assert!(
            matched.contains(&"Butterflies & Insects".to_string()),
            "folder 'Butterfly' should match 'Butterflies & Insects': {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_compound_tag_butterfly_filename() {
        let valid = HashSet::from(["Butterflies & Insects".to_string()]);
        let matched =
            suggest_path_rule_descriptions("Pretty Butterflies.pes", "C:/imports/", &valid);
        assert!(
            matched.contains(&"Butterflies & Insects".to_string()),
            "filename 'Pretty Butterflies.pes' should match 'Butterflies & Insects': {:?}",
            matched
        );
    }

    #[test]
    fn suggest_path_rule_user_created_tag() {
        // Proves the matcher is generic: a custom tag "My Rabbit Tag" should
        // be matched when the folder contains "Rabbits" (inflected to "rabbit").
        let valid = HashSet::from(["My Rabbit Tag".to_string(), "Borders & Frames".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Rabbits/design.pes", &valid);
        assert!(
            matched.contains(&"My Rabbit Tag".to_string()),
            "custom user tag 'My Rabbit Tag' should match folder 'Rabbits' via inflection: {:?}",
            matched
        );
        // Also ensure the unrelated tag is NOT matched
        assert!(!matched.contains(&"Borders & Frames".to_string()));
    }

    #[test]
    fn suggest_path_rule_synonym_kitten_cats() {
        let valid = HashSet::from(["Cats".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Kittens/design.pes", &valid);
        assert!(matched.contains(&"Cats".to_string()));
    }

    #[test]
    fn suggest_path_rule_synonym_floral_flowers() {
        let valid = HashSet::from(["Flowers".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Floral/design.pes", &valid);
        // "floral" → synonym map → "Flowers"
        assert!(matched.contains(&"Flowers".to_string()));
    }

    #[test]
    fn suggest_path_rule_does_not_match_cat_inside_catalogue() {
        // Regression: a folder called "Crests" must never be confused with "Cats".
        // Token-based matching ensures this: "crests" ≠ "cat" / "cats".
        let valid = HashSet::from(["Cats".to_string(), "Crests".to_string()]);
        let matched = suggest_path_rule_descriptions(
            "17147.hus",
            "C:/imports/Amazing Designs - 1033 Crests/17147.hus",
            &valid,
        );

        assert!(matched.contains(&"Crests".to_string()));
        assert!(!matched.contains(&"Cats".to_string()));
    }

    #[test]
    fn suggest_path_rule_empty_catalogue_returns_empty() {
        let valid = HashSet::new();
        let matched = suggest_path_rule_descriptions("flower.pes", "C:/imports/flowers/", &valid);
        assert!(matched.is_empty());
    }

    #[test]
    fn suggest_path_rule_synonym_upper_lower_is_case_insensitive() {
        // Folder/design names containing "upper" or "lower" must map to
        // "Words and Letters" regardless of case (path tokens are lowercased by
        // `normalize_text`, so the lowercase synonym keys match).
        let valid = HashSet::from(["Words and Letters".to_string()]);
        for folder in ["Upper", "UPPER", "lower", "Lower Case", "Uppercase"] {
            let matched = suggest_path_rule_descriptions(
                "",
                &format!("C:/imports/{folder}/design.pes"),
                &valid,
            );
            assert!(
                matched.contains(&"Words and Letters".to_string()),
                "folder '{folder}' should be tagged 'Words and Letters': {matched:?}"
            );
        }
    }

    #[test]
    fn suggest_path_rule_synonym_upper_from_filename() {
        let valid = HashSet::from(["Words and Letters".to_string()]);
        let matched =
            suggest_path_rule_descriptions("Upper Case Alphabet.pes", "C:/imports/", &valid);
        assert!(
            matched.contains(&"Words and Letters".to_string()),
            "filename 'Upper Case Alphabet.pes' should be tagged 'Words and Letters': {matched:?}"
        );
    }

    #[test]
    fn suggest_path_rule_synonym_description_is_case_insensitive() {
        // The synonym map literal is "Words and Letters", but the live catalogue
        // may store it with a different casing; the catalogue's own casing must
        // be returned so downstream tag-id lookups still resolve.
        let valid = HashSet::from(["WORDS AND LETTERS".to_string()]);
        let matched = suggest_path_rule_descriptions("", "C:/imports/Font/design.pes", &valid);
        assert!(
            matched.contains(&"WORDS AND LETTERS".to_string()),
            "synonym target should resolve case-insensitively to the catalogue casing: {matched:?}"
        );
    }

    #[test]
    fn suggest_path_rule_matches_from_filename_stem() {
        let valid = HashSet::from(["Flowers".to_string()]);
        let matched = suggest_path_rule_descriptions("Flower Design.pes", "C:/imports/", &valid);
        assert!(matched.contains(&"Flowers".to_string()));
    }

    #[test]
    fn suggest_path_rule_synonym_baby_children() {
        let valid = HashSet::from(["Children & Toys".to_string()]);
        let matched =
            suggest_path_rule_descriptions("", "C:/imports/baby/shirts/design.pes", &valid);
        assert!(matched.contains(&"Children & Toys".to_string()));
    }
}
