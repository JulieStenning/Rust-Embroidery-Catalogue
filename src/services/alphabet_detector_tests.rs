// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::services::alphabet_detector::*;

#[test]
fn test_path_contains_alphabet_keyword() {
    assert!(path_contains_alphabet_keyword(
        "D:/Embroidery/Alphabets/Vintage"
    ));
    assert!(path_contains_alphabet_keyword(
        "D:\\Embroidery\\My Fonts\\Cursive"
    ));
    assert!(path_contains_alphabet_keyword("Monograms/Fancy_3in"));
    assert!(path_contains_alphabet_keyword("Designs/Initials_Pack"));
    assert!(path_contains_alphabet_keyword("Collection/Alphabet_1"));
    assert!(!path_contains_alphabet_keyword(
        "D:/Embroidery/Animals/Bears"
    ));
    assert!(!path_contains_alphabet_keyword(
        "D:/Embroidery/Christmas/Tree"
    ));
}

#[test]
fn test_is_punctuation_token() {
    assert!(is_punctuation_token("!"));
    assert!(is_punctuation_token("?"));
    assert!(is_punctuation_token("&"));
    assert!(is_punctuation_token("comma"));
    assert!(is_punctuation_token("period"));
    assert!(is_punctuation_token("ampersand"));
    assert!(is_punctuation_token("exclamation"));
    assert!(is_punctuation_token("question"));
    assert!(is_punctuation_token("hyphen"));
    assert!(!is_punctuation_token("Upper"));
    assert!(!is_punctuation_token("A"));
    assert!(!is_punctuation_token("9"));
}

#[test]
fn test_extract_size_key() {
    assert_eq!(
        extract_size_key("Font_1in_Upper_A.pes"),
        Some("1in".to_string())
    );
    assert_eq!(
        extract_size_key("Font_2inch_Upper_A.pes"),
        Some("2inch".to_string())
    );
    assert_eq!(
        extract_size_key("Font_25mm_Upper_A.pes"),
        Some("25mm".to_string())
    );
    assert_eq!(
        extract_size_key("Font_4x4_Upper_A.pes"),
        Some("4x4".to_string())
    );
    assert_eq!(
        extract_size_key("Peas And Carrots Stitched 3 Inch.pes"),
        Some("3inch".to_string())
    );
    assert_eq!(
        extract_size_key("Apple And Orange Stitched 2_5 Inch.pes"),
        Some("2.5inch".to_string())
    );
    assert_eq!(
        extract_size_key("Rose 100 mm.dst"),
        Some("100mm".to_string())
    );
    assert_eq!(
        extract_size_key("Banner 4 x 4.pes"),
        Some("4x4".to_string())
    );
    assert_eq!(extract_size_key("Font_Upper_A.pes"), None);
}

#[test]
fn test_classify_glyph_name() {
    // Upper A
    assert_eq!(classify_glyph_name("Upper_A.pes"), GlyphCategory::UpperA);
    assert_eq!(classify_glyph_name("UpperA.pes"), GlyphCategory::UpperA);
    assert_eq!(classify_glyph_name("Caps_A.pes"), GlyphCategory::UpperA);
    assert_eq!(classify_glyph_name("Font_UC_A.pes"), GlyphCategory::UpperA);
    assert_eq!(classify_glyph_name("A.pes"), GlyphCategory::UpperA);

    // Lower a
    assert_eq!(classify_glyph_name("Lower_a.pes"), GlyphCategory::LowerA);
    assert_eq!(classify_glyph_name("Lowera.pes"), GlyphCategory::LowerA);
    assert_eq!(classify_glyph_name("Font_LC_a.pes"), GlyphCategory::LowerA);
    assert_eq!(classify_glyph_name("a.pes"), GlyphCategory::LowerA);

    // Other Upper
    assert_eq!(
        classify_glyph_name("Upper_B.pes"),
        GlyphCategory::OtherUpperLetter('B')
    );
    assert_eq!(
        classify_glyph_name("Upper_Z.pes"),
        GlyphCategory::OtherUpperLetter('Z')
    );
    assert_eq!(
        classify_glyph_name("B.pes"),
        GlyphCategory::OtherUpperLetter('B')
    );

    // Other Lower
    assert_eq!(
        classify_glyph_name("Lower_b.pes"),
        GlyphCategory::OtherLowerLetter('b')
    );
    assert_eq!(
        classify_glyph_name("Lower_z.pes"),
        GlyphCategory::OtherLowerLetter('z')
    );
    assert_eq!(
        classify_glyph_name("b.pes"),
        GlyphCategory::OtherLowerLetter('b')
    );

    // Digit 9
    assert_eq!(classify_glyph_name("Number_9.pes"), GlyphCategory::Digit9);
    assert_eq!(classify_glyph_name("9.pes"), GlyphCategory::Digit9);

    // Other Digits
    assert_eq!(
        classify_glyph_name("Number_0.pes"),
        GlyphCategory::OtherDigit('0')
    );
    assert_eq!(
        classify_glyph_name("Number_5.pes"),
        GlyphCategory::OtherDigit('5')
    );
    assert_eq!(classify_glyph_name("0.pes"), GlyphCategory::OtherDigit('0'));

    // Punctuation and symbols
    assert_eq!(
        classify_glyph_name("comma.pes"),
        GlyphCategory::PunctuationOrSymbol
    );
    assert_eq!(
        classify_glyph_name("Font_exclamation.pes"),
        GlyphCategory::PunctuationOrSymbol
    );
    assert_eq!(
        classify_glyph_name("!.pes"),
        GlyphCategory::PunctuationOrSymbol
    );
    assert_eq!(
        classify_glyph_name("?.pes"),
        GlyphCategory::PunctuationOrSymbol
    );
    assert_eq!(
        classify_glyph_name("&.pes"),
        GlyphCategory::PunctuationOrSymbol
    );

    // Non-glyph
    assert_eq!(
        classify_glyph_name("Instructions.pes"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(classify_glyph_name("Cover.pes"), GlyphCategory::NonGlyph);
}

#[test]
fn test_analyze_alphabet_folder_partitions_correctly() {
    let files = vec![
        "Upper_A.pes".to_string(),
        "Upper_B.pes".to_string(),
        "Upper_C.pes".to_string(),
        "Lower_a.pes".to_string(),
        "Lower_b.pes".to_string(),
        "9.pes".to_string(),
        "8.pes".to_string(),
        "comma.pes".to_string(),
        "exclamation.pes".to_string(),
        "Cover.pes".to_string(),
    ];

    let result = analyze_alphabet_folder("C:/Designs/Vintage_Alphabet", &files, false, None);

    assert!(result.is_alphabet);
    // Samples must include: Upper A, Lower a, 9, comma, exclamation
    assert!(result.sample_files.contains(&"Upper_A.pes".to_string()));
    assert!(result.sample_files.contains(&"Lower_a.pes".to_string()));
    assert!(result.sample_files.contains(&"9.pes".to_string()));
    assert!(result.sample_files.contains(&"comma.pes".to_string()));
    assert!(result.sample_files.contains(&"exclamation.pes".to_string()));

    // Compressible must include: Upper B, Upper C, Lower b, 8
    assert!(result
        .compressible_files
        .contains(&"Upper_B.pes".to_string()));
    assert!(result
        .compressible_files
        .contains(&"Upper_C.pes".to_string()));
    assert!(result
        .compressible_files
        .contains(&"Lower_b.pes".to_string()));
    assert!(result.compressible_files.contains(&"8.pes".to_string()));

    // Non-glyph must include Cover.pes
    assert!(result.other_files.contains(&"Cover.pes".to_string()));
}

#[test]
fn test_analyze_alphabet_folder_multi_size_keeps_samples_for_each_size() {
    let files = vec![
        "Font_1in_Upper_A.pes".to_string(),
        "Font_1in_Upper_B.pes".to_string(),
        "Font_1in_Lower_a.pes".to_string(),
        "Font_1in_9.pes".to_string(),
        "Font_1in_comma.pes".to_string(),
        "Font_2in_Upper_A.pes".to_string(),
        "Font_2in_Upper_B.pes".to_string(),
        "Font_2in_Lower_a.pes".to_string(),
        "Font_2in_9.pes".to_string(),
        "Font_2in_comma.pes".to_string(),
    ];

    let result = analyze_alphabet_folder(
        "C:/Designs/Fancy_Font",
        &files,
        true, // has .bx
        None,
    );

    assert!(result.is_alphabet);
    // Both 1in and 2in samples must be present
    assert!(result
        .sample_files
        .contains(&"Font_1in_Upper_A.pes".to_string()));
    assert!(result
        .sample_files
        .contains(&"Font_1in_Lower_a.pes".to_string()));
    assert!(result.sample_files.contains(&"Font_1in_9.pes".to_string()));
    assert!(result
        .sample_files
        .contains(&"Font_1in_comma.pes".to_string()));

    assert!(result
        .sample_files
        .contains(&"Font_2in_Upper_A.pes".to_string()));
    assert!(result
        .sample_files
        .contains(&"Font_2in_Lower_a.pes".to_string()));
    assert!(result.sample_files.contains(&"Font_2in_9.pes".to_string()));
    assert!(result
        .sample_files
        .contains(&"Font_2in_comma.pes".to_string()));

    // Compressible contains Upper B for both sizes
    assert!(result
        .compressible_files
        .contains(&"Font_1in_Upper_B.pes".to_string()));
    assert!(result
        .compressible_files
        .contains(&"Font_2in_Upper_B.pes".to_string()));
}

#[test]
fn test_non_alphabet_folder_keeps_all_files() {
    let files = vec![
        "TeddyBear.pes".to_string(),
        "TeddyBearWithBalloon.pes".to_string(),
        "TeddyBearSleeping.pes".to_string(),
    ];

    let result = analyze_alphabet_folder("C:/Designs/TeddyBears", &files, false, None);

    assert!(!result.is_alphabet);
    assert_eq!(result.sample_files.len(), 3);
    assert_eq!(result.compressible_files.len(), 0);
}

#[test]
fn test_classify_glyph_name_with_dimensions_returns_non_glyph() {
    assert_eq!(
        classify_glyph_name("Peas And Carrots Stitched 3 Inch.pes"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("Peas And Carrots Stitched 2_5 Inch.pes"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("Apple And Orange Stitched 5_5 Inch.pes"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("Chicken And Waffles Stitched 4 Inch.pes"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("Give Peas a Chance Stitched 5 Inch.jef"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("06 Carrots Applique 6x10 Hoop.jef"),
        GlyphCategory::NonGlyph
    );
    assert_eq!(
        classify_glyph_name("Christmas Tree 2.pes"),
        GlyphCategory::NonGlyph
    );
}

#[test]
fn test_font_set_with_separated_size_units_classified_correctly() {
    assert_eq!(
        classify_glyph_name("Upper A 3 Inch.pes"),
        GlyphCategory::UpperA
    );
    assert_eq!(
        classify_glyph_name("Lower a 3 Inch.pes"),
        GlyphCategory::LowerA
    );
    assert_eq!(
        classify_glyph_name("Number 9 3 Inch.pes"),
        GlyphCategory::Digit9
    );
    assert_eq!(
        classify_glyph_name("Number 0 3 Inch.pes"),
        GlyphCategory::OtherDigit('0')
    );
}

#[test]
fn test_multi_size_design_set_not_detected_as_alphabet() {
    let files = vec![
        "Peas And Carrots Stitched 2_5 Inch.pes".to_string(),
        "Peas And Carrots Stitched 3 Inch.pes".to_string(),
        "Peas And Carrots Stitched 3_5 Inch.pes".to_string(),
        "Peas And Carrots Stitched 4 Inch.pes".to_string(),
        "Peas And Carrots Stitched 4_5 Inch.pes".to_string(),
        "Peas And Carrots Stitched 5 Inch.pes".to_string(),
        "Peas And Carrots Stitched 5_5 Inch.pes".to_string(),
        "Apple And Orange Stitched 2_5 Inch.pes".to_string(),
        "Apple And Orange Stitched 3 Inch.pes".to_string(),
        "Apple And Orange Stitched 3_5 Inch.pes".to_string(),
        "Apple And Orange Stitched 4 Inch.pes".to_string(),
        "Apple And Orange Stitched 4_5 Inch.pes".to_string(),
        "Apple And Orange Stitched 5 Inch.pes".to_string(),
        "Apple And Orange Stitched 5_5 Inch.pes".to_string(),
        "Chicken And Waffles Stitched 2_5 Inch.pes".to_string(),
        "Chicken And Waffles Stitched 3 Inch.pes".to_string(),
    ];

    let result = analyze_alphabet_folder(
        "H:/New Embroidery Designs/Embroidery Super Deal/PerfectFoodPairsPES/PES",
        &files,
        false,
        None,
    );

    assert!(!result.is_alphabet);
    assert_eq!(result.sample_files.len(), 16);
    assert_eq!(result.compressible_files.len(), 0);
}

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, now));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_find_companion_zip_in_dir() {
    let tmp = unique_temp_dir("companion_zip_test");
    let font_dir = tmp.join("VintageFont");
    std::fs::create_dir_all(&font_dir).unwrap();

    // 1. Matching stem zip should be found
    let zip_path = font_dir.join("VintageFont.zip");
    std::fs::write(&zip_path, b"dummy zip").unwrap();
    assert_eq!(find_companion_zip_in_dir(&font_dir), Some(zip_path.clone()));

    // 2. Directory with multiple unrelated design archives should NOT match as companion zip
    let pes_dir = tmp.join("PES");
    std::fs::create_dir_all(&pes_dir).unwrap();
    std::fs::write(pes_dir.join("AppleAndOrange.zip"), b"dummy").unwrap();
    std::fs::write(pes_dir.join("PeasAndCarrots.zip"), b"dummy").unwrap();
    assert_eq!(find_companion_zip_in_dir(&pes_dir), None);
}
