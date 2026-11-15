// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Alphabet and Font Package Detection Service
//!
//! Identifies digital embroidery alphabet and monogram collections during import,
//! classifying glyph files into representative preview samples (Upper A, Lower a, 9,
//! and all punctuation/symbol glyphs) versus compressible variants (B–Z, b–z, 0–8).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Punctuation and symbol tokens commonly found in embroidery font filenames.
pub const PUNCTUATION_WORDS: &[&str] = &[
    "ampersand",
    "apostrophe",
    "asterisk",
    "at",
    "backslash",
    "brace",
    "bracket",
    "caret",
    "colon",
    "comma",
    "dash",
    "dollar",
    "dot",
    "equal",
    "equals",
    "exclamation",
    "exclam",
    "hash",
    "hyphen",
    "minus",
    "paren",
    "parenthesis",
    "percent",
    "period",
    "plus",
    "pound",
    "question",
    "quote",
    "quotes",
    "semicolon",
    "slash",
    "star",
    "tilde",
    "underscore",
];

/// Folder keywords that strongly indicate an alphabet or font package.
pub const ALPHABET_KEYWORDS: &[&str] = &[
    "alphabet",
    "alphabets",
    "font",
    "fonts",
    "subfont",
    "subfonts",
    "alpha",
    "alphas",
    "monogram",
    "monograms",
    "initials",
    "letter",
    "letters",
    "number",
    "numbers",
    "numeral",
    "numerals",
    "digit",
    "digits",
];

/// Category of an embroidery file glyph inside a font package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GlyphCategory {
    /// Upper case letter 'A'.
    UpperA,
    /// Lower case letter 'a'.
    LowerA,
    /// Number digit '9'.
    Digit9,
    /// Any punctuation mark or symbol (e.g. '!', '?', '@', '&', comma, period, etc.).
    PunctuationOrSymbol,
    /// Other uppercase letter ('B' through 'Z').
    OtherUpperLetter(char),
    /// Other lowercase letter ('b' through 'z').
    OtherLowerLetter(char),
    /// Other digit ('0' through '8').
    OtherDigit(char),
    /// File that does not appear to represent an alphabet glyph (e.g. cover, instructions).
    NonGlyph,
}

impl GlyphCategory {
    /// Returns true if this glyph should be kept as a catalogued preview sample in the catalogue.
    pub fn is_sample(&self) -> bool {
        matches!(
            self,
            GlyphCategory::UpperA
                | GlyphCategory::LowerA
                | GlyphCategory::Digit9
                | GlyphCategory::PunctuationOrSymbol
        )
    }

    /// Returns true if this glyph represents an additional letter or number to be compressed into zip.
    pub fn is_compressible(&self) -> bool {
        matches!(
            self,
            GlyphCategory::OtherUpperLetter(_)
                | GlyphCategory::OtherLowerLetter(_)
                | GlyphCategory::OtherDigit(_)
        )
    }
}

/// Analysis result for a folder evaluated as an alphabet or font collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlphabetFolderAnalysis {
    /// The folder path or relative key.
    pub folder_path: String,
    /// Whether the folder qualifies as an alphabet collection.
    pub is_alphabet: bool,
    /// Evidence and heuristics that contributed to the determination.
    pub reasons: Vec<String>,
    /// Files categorized as preview samples (Upper A, Lower a, 9, all punctuation).
    pub sample_files: Vec<String>,
    /// Files categorized as compressible glyphs (B–Z, b–z, 0–8).
    pub compressible_files: Vec<String>,
    /// Files that are non-glyphs (e.g. covers, instructions, diagrams) to keep uncompressed.
    pub other_files: Vec<String>,
    /// Path to an existing companion `.zip` in the folder, if any.
    pub existing_companion_zip: Option<String>,
    /// True if Embrilliance `.bx` files are present in the folder.
    pub has_bx_files: bool,
}

/// Returns true if any path segment contains an alphabet keyword.
pub fn path_contains_alphabet_keyword(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase().replace('\\', "/");
    for segment in normalized.split('/') {
        let clean = segment.trim();
        if ALPHABET_KEYWORDS.iter().any(|&kw| clean.contains(kw)) {
            return true;
        }
    }
    false
}

/// Returns true if the token matches a known punctuation word or symbol character.
pub fn is_punctuation_token(token: &str) -> bool {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if PUNCTUATION_WORDS.contains(&lower.as_str()) {
        return true;
    }
    // Check if token consists purely of punctuation / symbol characters
    trimmed
        .chars()
        .all(|c| c.is_ascii_punctuation() || matches!(c, '¿' | '¡'))
}

fn is_combined_dimension(t: &str) -> bool {
    if t.contains('x') {
        let parts: Vec<&str> = t.split('x').collect();
        return parts.len() == 2
            && parts[0].chars().all(|c| c.is_ascii_digit())
            && parts[1].chars().all(|c| c.is_ascii_digit());
    }
    if t.ends_with("in") || t.ends_with("inch") || t.ends_with("mm") || t.ends_with("cm") {
        let num = t
            .trim_end_matches("inch")
            .trim_end_matches("in")
            .trim_end_matches("mm")
            .trim_end_matches("cm");
        return !num.is_empty() && num.chars().all(|c| c.is_ascii_digit() || c == '.');
    }
    false
}

fn token_is_numeric(token: &str) -> bool {
    !token.is_empty() && token.chars().all(|c| c.is_ascii_digit() || c == '.')
}

fn normalize_stem_delimiters(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let chars: Vec<char> = stem.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '_'
            && i > 0
            && i + 1 < chars.len()
            && chars[i - 1].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
        {
            out.push('.');
        } else {
            out.push(chars[i]);
        }
    }
    out
}

/// Identifies which tokens in a sequence represent dimensions or units rather than glyph identifiers.
pub fn identify_dimension_indices(tokens: &[&str]) -> Vec<bool> {
    let mut is_dim = vec![false; tokens.len()];
    for (i, &token) in tokens.iter().enumerate() {
        let t_lower = token.to_ascii_lowercase();
        if matches!(
            t_lower.as_str(),
            "in" | "inch" | "inches" | "mm" | "cm" | "hoop"
        ) {
            is_dim[i] = true;
            if i >= 1 && token_is_numeric(tokens[i - 1]) {
                is_dim[i - 1] = true;
            }
        } else if t_lower == "x" && i >= 1 && i + 1 < tokens.len() {
            if tokens[i - 1].chars().all(|c| c.is_ascii_digit())
                && tokens[i + 1].chars().all(|c| c.is_ascii_digit())
            {
                is_dim[i - 1] = true;
                is_dim[i] = true;
                is_dim[i + 1] = true;
            }
        } else if is_combined_dimension(&t_lower) {
            is_dim[i] = true;
        }
    }
    is_dim
}

/// Searches a directory for an alphabet companion `.zip` archive.
///
/// Returns `Some(PathBuf)` if:
/// - There is a `.zip` whose stem matches the directory name, or
/// - There is a `.zip` whose stem contains an alphabet keyword (e.g. `font`, `alphabet`, `monogram`, `bx`), or
/// - There is exactly one `.zip` in the directory and the directory path contains an alphabet keyword.
///
/// Returns `None` if no `.zip` files exist, or if multiple unrelated per-design archives exist.
pub fn find_companion_zip_in_dir(dir: &Path) -> Option<std::path::PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    let zips: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        })
        .collect();

    if zips.is_empty() {
        return None;
    }

    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let dir_lower = dir_name.to_ascii_lowercase();

    for zip in &zips {
        if let Some(stem) = zip.file_stem().and_then(|s| s.to_str()) {
            let stem_lower = stem.to_ascii_lowercase();
            if stem_lower == dir_lower
                || stem_lower.starts_with(&format!("{}_", dir_lower))
                || stem_lower.starts_with(&format!("{}-", dir_lower))
                || ALPHABET_KEYWORDS.iter().any(|&kw| stem_lower.contains(kw))
            {
                return Some(zip.clone());
            }
        }
    }

    if zips.len() == 1 && path_contains_alphabet_keyword(&dir.to_string_lossy()) {
        return Some(zips[0].clone());
    }

    None
}

/// Extracts an optional size indicator from a filename stem or folder path (e.g. "1in", "25mm", "4x4", "3 Inch", "2_5 Inch").
pub fn extract_size_key(text: &str) -> Option<String> {
    let path = Path::new(text);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(text);
    let normalized = normalize_stem_delimiters(stem);
    let lower = normalized.to_ascii_lowercase();
    let tokens: Vec<&str> = lower
        .split(['_', '-', ' ', '(', ')'])
        .filter(|s| !s.is_empty())
        .collect();

    for (i, &token) in tokens.iter().enumerate() {
        if matches!(token, "in" | "inch" | "inches" | "mm" | "cm") {
            if i >= 1 && token_is_numeric(tokens[i - 1]) {
                return Some(format!("{}{}", tokens[i - 1], token));
            }
        }
        if token == "x"
            && i >= 1
            && i + 1 < tokens.len()
            && tokens[i - 1].chars().all(|c| c.is_ascii_digit())
            && tokens[i + 1].chars().all(|c| c.is_ascii_digit())
        {
            return Some(format!("{}x{}", tokens[i - 1], tokens[i + 1]));
        }
        if is_combined_dimension(token) {
            return Some(token.to_string());
        }
    }
    None
}

/// Returns true if the token matches a known font or alphabet package keyword.
pub fn is_font_or_alphabet_keyword(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "font"
            | "fonts"
            | "subfont"
            | "subfonts"
            | "alphabet"
            | "alphabets"
            | "alpha"
            | "alphas"
            | "monogram"
            | "monograms"
            | "letter"
            | "letters"
            | "bonus"
    ) || (lower.len() > 4 && lower.ends_with("font"))
        || (lower.len() > 5 && lower.ends_with("fonts"))
}

/// Classifies a single filename into its corresponding [`GlyphCategory`].
pub fn classify_glyph_name(filename: &str) -> GlyphCategory {
    let path = Path::new(filename);
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return GlyphCategory::NonGlyph,
    };

    let norm_stem = normalize_stem_delimiters(stem);
    let lower_stem = norm_stem.to_ascii_lowercase();

    // Check exact punctuation word matches first
    for word in PUNCTUATION_WORDS {
        if lower_stem == *word {
            return GlyphCategory::PunctuationOrSymbol;
        }
    }

    // Split stem into tokens and strip dimension tokens
    let raw_tokens: Vec<&str> = norm_stem
        .split(['_', '-', ' '])
        .filter(|s| !s.is_empty())
        .collect();

    if raw_tokens.is_empty() {
        return GlyphCategory::NonGlyph;
    }

    let dim_mask = identify_dimension_indices(&raw_tokens);
    let tokens: Vec<&str> = raw_tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| !dim_mask[*idx])
        .map(|(_, &t)| t)
        .collect();

    if tokens.is_empty() {
        return GlyphCategory::NonGlyph;
    }

    // Single token is punctuation
    if tokens.len() == 1 && is_punctuation_token(tokens[0]) {
        return GlyphCategory::PunctuationOrSymbol;
    }

    let mut has_explicit_upper = false;
    let mut has_explicit_lower = false;
    let mut has_explicit_number = false;
    let mut has_font_keyword = false;
    let mut has_punctuation_token = false;
    let mut detected_letter: Option<char> = None;
    let mut detected_digit: Option<char> = None;
    let mut descriptive_word_count = 0usize;

    for &token in &tokens {
        let t_lower = token.to_ascii_lowercase();
        if matches!(
            t_lower.as_str(),
            "upper" | "uppercase" | "caps" | "cap" | "uc"
        ) {
            has_explicit_upper = true;
            continue;
        }
        if matches!(t_lower.as_str(), "lower" | "lowercase" | "small" | "lc") {
            has_explicit_lower = true;
            continue;
        }
        if matches!(t_lower.as_str(), "number" | "num" | "digit") {
            has_explicit_number = true;
            continue;
        }
        if is_font_or_alphabet_keyword(token) {
            has_font_keyword = true;
            continue;
        }
        if is_punctuation_token(token) {
            has_punctuation_token = true;
            continue;
        }

        // Single letter or digit token
        if token.len() == 1 {
            let ch = token.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                detected_letter = Some(ch);
            } else if ch.is_ascii_digit() {
                detected_digit = Some(ch);
            }
        } else {
            let (is_match, ch, is_upper) = if t_lower.starts_with("upper") && token.len() == 6 {
                (true, token.chars().nth(5).unwrap(), true)
            } else if t_lower.starts_with("lower") && token.len() == 6 {
                (true, token.chars().nth(5).unwrap(), false)
            } else if t_lower.starts_with("uc") && token.len() == 3 {
                (true, token.chars().nth(2).unwrap(), true)
            } else if t_lower.starts_with("lc") && token.len() == 3 {
                (true, token.chars().nth(2).unwrap(), false)
            } else {
                (false, ' ', false)
            };
            if is_match && ch.is_ascii_alphabetic() {
                detected_letter = Some(ch);
                if is_upper {
                    has_explicit_upper = true;
                } else {
                    has_explicit_lower = true;
                }
            } else {
                descriptive_word_count += 1;
            }
        }
    }

    let has_font_or_glyph_context =
        has_font_keyword || has_explicit_upper || has_explicit_lower || has_explicit_number;

    // Files with multiple descriptive words are designs rather than font glyphs
    // unless they have an explicit font or glyph indicator (in which case descriptive
    // words represent the font family name).
    if !has_font_or_glyph_context && descriptive_word_count >= 2 {
        return GlyphCategory::NonGlyph;
    }

    if let Some(ch) = detected_letter {
        if !has_font_or_glyph_context && descriptive_word_count > 0 {
            return GlyphCategory::NonGlyph;
        }

        let is_lower = if has_explicit_lower && !has_explicit_upper {
            true
        } else if has_explicit_upper && !has_explicit_lower {
            false
        } else {
            ch.is_ascii_lowercase()
        };

        let upper_ch = ch.to_ascii_uppercase();
        let lower_ch = ch.to_ascii_lowercase();

        if is_lower {
            if lower_ch == 'a' {
                return GlyphCategory::LowerA;
            }
            return GlyphCategory::OtherLowerLetter(lower_ch);
        }

        if upper_ch == 'A' {
            return GlyphCategory::UpperA;
        }
        return GlyphCategory::OtherUpperLetter(upper_ch);
    }

    // Trailing digits without explicit number indicators on non-isolated tokens are design variations
    if let Some(digit) = detected_digit {
        if !has_explicit_number && descriptive_word_count > 0 {
            return GlyphCategory::NonGlyph;
        }
        if digit == '9' {
            return GlyphCategory::Digit9;
        }
        return GlyphCategory::OtherDigit(digit);
    }

    // Punctuation or symbol glyph in font package
    if has_punctuation_token {
        return GlyphCategory::PunctuationOrSymbol;
    }

    GlyphCategory::NonGlyph
}

/// Evaluates a collection of scanned files belonging to a folder to determine if it is an alphabet set,
/// grouping files into preview samples versus compressible files per detected size.
pub fn analyze_alphabet_folder(
    folder_path: &str,
    file_paths: &[String],
    folder_has_bx: bool,
    companion_zip: Option<String>,
) -> AlphabetFolderAnalysis {
    let mut reasons = Vec::new();
    let has_keyword = path_contains_alphabet_keyword(folder_path);
    if has_keyword {
        reasons.push("Path contains alphabet/font keyword".to_string());
    }
    if folder_has_bx {
        reasons.push("Folder contains Embrilliance .bx font files".to_string());
    }
    if companion_zip.is_some() {
        reasons.push("Folder contains matching companion .zip archive".to_string());
    }

    let any_file_has_keyword = file_paths.iter().any(|p| {
        Path::new(p)
            .file_name()
            .and_then(|n| n.to_str())
            .map(path_contains_alphabet_keyword)
            .unwrap_or(false)
    });
    if any_file_has_keyword && !has_keyword {
        reasons.push("Filenames contain alphabet/font keyword".to_string());
    }

    // Map files by detected size key: size_key -> Vec<(file_path, GlyphCategory)>
    let mut by_size: HashMap<Option<String>, Vec<(String, GlyphCategory)>> = HashMap::new();
    let mut glyph_count = 0usize;

    for path_str in file_paths {
        let filename = Path::new(path_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path_str);

        let category = classify_glyph_name(filename);
        let size_key = extract_size_key(filename).or_else(|| extract_size_key(folder_path));

        if category != GlyphCategory::NonGlyph {
            glyph_count += 1;
        }

        by_size
            .entry(size_key)
            .or_default()
            .push((path_str.clone(), category));
    }

    let total_embroidery_files = file_paths.len();
    let cluster_heuristic = total_embroidery_files >= 15
        && (glyph_count as f64 / total_embroidery_files.max(1) as f64) >= 0.50;

    if cluster_heuristic {
        reasons.push(format!(
            "File cluster matches glyph patterns ({}/{} glyph files)",
            glyph_count, total_embroidery_files
        ));
    }

    let is_alphabet = ((has_keyword || any_file_has_keyword) && glyph_count > 0)
        || (folder_has_bx && glyph_count > 0)
        || (companion_zip.is_some() && glyph_count > 0)
        || cluster_heuristic;

    let mut sample_files = Vec::new();
    let mut compressible_files = Vec::new();
    let mut other_files = Vec::new();

    if is_alphabet {
        // For each size group, preserve Upper A, Lower a, 9, and ALL punctuation/symbols
        for (_size, items) in by_size {
            for (file_path, category) in items {
                if category.is_sample() {
                    sample_files.push(file_path);
                } else if category.is_compressible() {
                    compressible_files.push(file_path);
                } else {
                    other_files.push(file_path);
                }
            }
        }
    } else {
        // Not an alphabet: all files remain uncompressed
        sample_files = file_paths.to_vec();
    }

    sample_files.sort();
    compressible_files.sort();
    other_files.sort();

    AlphabetFolderAnalysis {
        folder_path: folder_path.to_string(),
        is_alphabet,
        reasons,
        sample_files,
        compressible_files,
        other_files,
        existing_companion_zip: companion_zip,
        has_bx_files: folder_has_bx,
    }
}

#[cfg(test)]
#[path = "alphabet_detector_tests.rs"]
mod tests;
