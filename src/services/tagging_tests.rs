// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::collections::HashMap;

fn default_test_synonyms() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    map.insert("kitten".to_string(), vec!["Cats".to_string()]);
    map.insert("puppy".to_string(), vec!["Dogs".to_string()]);
    map.insert("frog".to_string(), vec!["Animals".to_string()]);
    map.insert(
        "alphabet".to_string(),
        vec!["Words and Letters".to_string()],
    );
    map.insert("font".to_string(), vec!["Words and Letters".to_string()]);
    map.insert(
        "monogram".to_string(),
        vec!["Words and Letters".to_string()],
    );
    map.insert("upper".to_string(), vec!["Words and Letters".to_string()]);
    map.insert("lower".to_string(), vec!["Words and Letters".to_string()]);
    map.insert(
        "uppercase".to_string(),
        vec!["Words and Letters".to_string()],
    );
    map.insert(
        "lowercase".to_string(),
        vec!["Words and Letters".to_string()],
    );
    map.insert("xmas".to_string(), vec!["Christmas".to_string()]);
    map.insert("floral".to_string(), vec!["Flowers".to_string()]);
    map.insert("baby".to_string(), vec!["Children & Toys".to_string()]);
    map
}

// ─── unit helpers ───────────────────────────────────────────────────

#[test]
fn normalize_text_replaces_punctuation_with_spaces() {
    let result = normalize_text("Borders & Frames");
    assert_eq!(result, "borders   frames");
}

#[test]
fn normalize_text_splits_letter_digit_and_camelcase_transitions() {
    assert_eq!(normalize_text("Butterfly1"), "butterfly 1");
    assert_eq!(normalize_text("12Floral"), "12 floral");
    assert_eq!(normalize_text("MonarchButterfly"), "monarch butterfly");
}

#[test]
fn significant_tokens_filters_short_words_and_conjunctions() {
    let tokens = significant_tokens("a big cat and a dog");
    assert_eq!(tokens, vec!["big", "cat", "dog"]);
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
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Borders/somefile.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Borders & Frames".to_string()),
        "folder 'Borders' should match 'Borders & Frames' via token overlap: {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_compound_tag_frame() {
    let valid = HashSet::from(["Borders & Frames".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Frame/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Borders & Frames".to_string()),
        "folder 'Frame' should match 'Borders & Frames' via inflected token overlap: {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_compound_tag_angels() {
    let valid = HashSet::from(["Angels & Fairies".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Angels/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Angels & Fairies".to_string()),
        "folder 'Angels' should match 'Angels & Fairies': {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_compound_tag_fairies() {
    let valid = HashSet::from(["Angels & Fairies".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Fairies/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Angels & Fairies".to_string()),
        "folder 'Fairies' should match 'Angels & Fairies' via inflection: {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_synonym_maps_monogram_to_words_and_letters() {
    let valid = HashSet::from(["Words and Letters".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Monogram/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Words and Letters".to_string()),
        "folder 'Monogram' should match 'Words and Letters' via the 'monogram' synonym: {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_synonym_maps_font_to_words_and_letters() {
    let valid = HashSet::from(["Words and Letters".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Font/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Words and Letters".to_string()),
        "folder 'Font' should match 'Words and Letters' via the 'font' synonym: {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_compound_tag_butterfly_folder() {
    let valid = HashSet::from(["Butterflies & Insects".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Butterfly/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Butterflies & Insects".to_string()),
        "folder 'Butterfly' should match 'Butterflies & Insects': {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_compound_tag_butterfly_filename() {
    let valid = HashSet::from(["Butterflies & Insects".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("Pretty Butterflies.pes", "C:/imports/", &valid, &synonyms);
    assert!(
        matched.contains(&"Butterflies & Insects".to_string()),
        "filename 'Pretty Butterflies.pes' should match 'Butterflies & Insects': {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_user_created_tag() {
    let valid = HashSet::from(["My Rabbit Tag".to_string(), "Borders & Frames".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Rabbits/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"My Rabbit Tag".to_string()),
        "custom user tag 'My Rabbit Tag' should match folder 'Rabbits' via inflection: {:?}",
        matched
    );
    assert!(!matched.contains(&"Borders & Frames".to_string()));
}

#[test]
fn suggest_path_rule_synonym_kitten_cats() {
    let valid = HashSet::from(["Cats".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Kittens/design.pes", &valid, &synonyms);
    assert!(matched.contains(&"Cats".to_string()));
}

#[test]
fn suggest_path_rule_synonym_frog_animals() {
    let valid = HashSet::from(["Animals".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Frogs/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"Animals".to_string()),
        "folder 'Frogs' should match 'Animals' via user synonym 'frog': {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_synonym_floral_flowers() {
    let valid = HashSet::from(["Flowers".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Floral/design.pes", &valid, &synonyms);
    assert!(matched.contains(&"Flowers".to_string()));
}

#[test]
fn suggest_path_rule_does_not_match_cat_inside_catalogue() {
    let valid = HashSet::from(["Cats".to_string(), "Crests".to_string()]);
    let synonyms = default_test_synonyms();
    let matched = suggest_path_rule_descriptions(
        "17147.hus",
        "C:/imports/The Rose Studio - 1033 Crests/17147.hus",
        &valid,
        &synonyms,
    );

    assert!(matched.contains(&"Crests".to_string()));
    assert!(!matched.contains(&"Cats".to_string()));
}

#[test]
fn suggest_path_rule_empty_catalogue_returns_empty() {
    let valid = HashSet::new();
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("flower.pes", "C:/imports/flowers/", &valid, &synonyms);
    assert!(matched.is_empty());
}

#[test]
fn suggest_path_rule_synonym_upper_lower_is_case_insensitive() {
    let valid = HashSet::from(["Words and Letters".to_string()]);
    let synonyms = default_test_synonyms();
    for folder in ["Upper", "UPPER", "lower", "Lower Case", "Uppercase"] {
        let matched = suggest_path_rule_descriptions(
            "",
            &format!("C:/imports/{folder}/design.pes"),
            &valid,
            &synonyms,
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
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("Upper Case Alphabet.pes", "C:/imports/", &valid, &synonyms);
    assert!(
        matched.contains(&"Words and Letters".to_string()),
        "filename 'Upper Case Alphabet.pes' should be tagged 'Words and Letters': {matched:?}"
    );
}

#[test]
fn suggest_path_rule_synonym_alphabet_words_and_letters() {
    let valid = HashSet::from(["Words and Letters".to_string()]);
    let synonyms = default_test_synonyms();
    for folder in ["Alphabet", "alphabets", "ALPHABETS"] {
        let matched = suggest_path_rule_descriptions(
            "",
            &format!("C:/imports/{folder}/design.pes"),
            &valid,
            &synonyms,
        );
        assert!(
            matched.contains(&"Words and Letters".to_string()),
            "folder '{folder}' should be tagged 'Words and Letters': {matched:?}"
        );
    }
}

#[test]
fn suggest_path_rule_synonym_description_is_case_insensitive() {
    let valid = HashSet::from(["WORDS AND LETTERS".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/Font/design.pes", &valid, &synonyms);
    assert!(
        matched.contains(&"WORDS AND LETTERS".to_string()),
        "synonym target should resolve case-insensitively to the catalogue casing: {matched:?}"
    );
}

#[test]
fn suggest_path_rule_matches_from_filename_stem() {
    let valid = HashSet::from(["Flowers".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("Flower Design.pes", "C:/imports/", &valid, &synonyms);
    assert!(matched.contains(&"Flowers".to_string()));
}

#[test]
fn suggest_path_rule_synonym_baby_children() {
    let valid = HashSet::from(["Children & Toys".to_string()]);
    let synonyms = default_test_synonyms();
    let matched =
        suggest_path_rule_descriptions("", "C:/imports/baby/shirts/design.pes", &valid, &synonyms);
    assert!(matched.contains(&"Children & Toys".to_string()));
}

#[test]
fn suggest_path_rule_does_not_match_on_and_conjunction() {
    let valid = HashSet::from([
        "Badges and Crests".to_string(),
        "Words and Letters".to_string(),
        "Celebrations".to_string(),
        "Christmas".to_string(),
    ]);
    let synonyms = default_test_synonyms();
    let matched = suggest_path_rule_descriptions(
        "#28001.pes",
        "F:/MachineEmbroideryDesigns/Filed/Holidays and Celebrations/028 - Christmas/#28001.pes",
        &valid,
        &synonyms,
    );
    assert!(matched.contains(&"Celebrations".to_string()));
    assert!(matched.contains(&"Christmas".to_string()));
    assert!(
        !matched.contains(&"Badges and Crests".to_string()),
        "should not match 'Badges and Crests' solely because path contains 'and'"
    );
    assert!(
        !matched.contains(&"Words and Letters".to_string()),
        "should not match 'Words and Letters' solely because path contains 'and'"
    );
}

#[test]
fn suggest_path_rule_matches_trailing_number_butterfly1() {
    let valid = HashSet::from(["Butterflies and Insects".to_string()]);
    let synonyms = default_test_synonyms();
    let matched = suggest_path_rule_descriptions(
        "Butterfly1.vp3",
        "F:\\MachineEmbroideryDesigns\\dime\\Designs\\Perfect Stitch Viewer Free Designs\\Vp3\\Butterfly1.vp3",
        &valid,
        &synonyms,
    );
    assert!(
        matched.contains(&"Butterflies and Insects".to_string()),
        "file 'Butterfly1.vp3' should match 'Butterflies and Insects': {:?}",
        matched
    );
}

#[test]
fn suggest_path_rule_matches_numbered_and_camelcase_designs() {
    let valid = HashSet::from([
        "Cats".to_string(),
        "Flowers".to_string(),
        "Butterflies and Insects".to_string(),
        "Crests".to_string(),
    ]);
    let synonyms = default_test_synonyms();

    // Cat02 matches Cats via singular form "cat"
    let matched_cat = suggest_path_rule_descriptions("Cat02.pes", "C:/imports/", &valid, &synonyms);
    assert!(matched_cat.contains(&"Cats".to_string()));

    // 12Floral matches Flowers via synonym "floral"
    let matched_floral =
        suggest_path_rule_descriptions("12Floral.pes", "C:/imports/", &valid, &synonyms);
    assert!(matched_floral.contains(&"Flowers".to_string()));

    // MonarchButterfly matches Butterflies and Insects via camelCase split
    let matched_butterfly =
        suggest_path_rule_descriptions("MonarchButterfly.pes", "C:/imports/", &valid, &synonyms);
    assert!(matched_butterfly.contains(&"Butterflies and Insects".to_string()));

    // Boundary safety: Crests1 must never match Cats
    let matched_crests =
        suggest_path_rule_descriptions("Crests1.pes", "C:/imports/", &valid, &synonyms);
    assert!(matched_crests.contains(&"Crests".to_string()));
    assert!(!matched_crests.contains(&"Cats".to_string()));
}
