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
    "monogram",
    "monograms",
    "initials",
    "letters",
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
    let lower = path.to_ascii_lowercase();
    let normalized = lower.replace('\\', "/");
    for segment in normalized.split('/') {
        let clean = segment.trim();
        for &kw in ALPHABET_KEYWORDS {
            if clean == kw
                || clean.starts_with(&format!("{}_", kw))
                || clean.ends_with(&format!("_{}", kw))
                || clean.starts_with(&format!("{}-", kw))
                || clean.ends_with(&format!("-{}", kw))
                || clean.contains(&format!(" {} ", kw))
                || clean.starts_with(&format!("{} ", kw))
                || clean.ends_with(&format!(" {}", kw))
                || clean.contains(kw)
            {
                return true;
            }
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
    trimmed.chars().all(|c| {
        c.is_ascii_punctuation()
            || matches!(
                c,
                '!' | '?'
                    | '@'
                    | '#'
                    | '$'
                    | '%'
                    | '^'
                    | '&'
                    | '*'
                    | '('
                    | ')'
                    | '-'
                    | '_'
                    | '+'
                    | '='
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | '|'
                    | '\\'
                    | ':'
                    | ';'
                    | '"'
                    | '\''
                    | '<'
                    | '>'
                    | ','
                    | '.'
                    | '/'
                    | '~'
                    | '`'
                    | '¿'
                    | '¡'
            )
    })
}

/// Extracts an optional size indicator from a filename stem or folder path (e.g. "1in", "25mm", "4x4").
pub fn extract_size_key(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    // Split by common delimiters
    let tokens: Vec<&str> = lower
        .split(['_', '-', ' ', '(', ')'])
        .filter(|s| !s.is_empty())
        .collect();

    for token in tokens {
        // e.g. "1in", "1.5in", "2inch", "25mm", "30mm", "4x4", "5x7"
        if token.ends_with("in")
            || token.ends_with("inch")
            || token.ends_with("mm")
            || token.ends_with("cm")
        {
            let numeric_part = token
                .trim_end_matches("inch")
                .trim_end_matches("in")
                .trim_end_matches("mm")
                .trim_end_matches("cm");
            if !numeric_part.is_empty()
                && numeric_part.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                return Some(token.to_string());
            }
        }
        if token.contains('x') {
            let parts: Vec<&str> = token.split('x').collect();
            if parts.len() == 2
                && parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].chars().all(|c| c.is_ascii_digit())
            {
                return Some(token.to_string());
            }
        }
    }
    None
}

/// Classifies a single filename into its corresponding [`GlyphCategory`].
pub fn classify_glyph_name(filename: &str) -> GlyphCategory {
    let path = Path::new(filename);
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return GlyphCategory::NonGlyph,
    };

    let lower_stem = stem.to_ascii_lowercase();

    // Check punctuation words first
    for word in PUNCTUATION_WORDS {
        if lower_stem == *word
            || lower_stem.ends_with(&format!("_{}", word))
            || lower_stem.ends_with(&format!("-{}", word))
            || lower_stem.ends_with(&format!(" {}", word))
            || lower_stem.starts_with(&format!("{}_", word))
            || lower_stem.starts_with(&format!("{}-", word))
            || lower_stem.starts_with(&format!("{} ", word))
        {
            return GlyphCategory::PunctuationOrSymbol;
        }
    }

    // Split stem into tokens
    let tokens: Vec<&str> = stem
        .split(['_', '-', ' '])
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return GlyphCategory::NonGlyph;
    }

    // Single token is punctuation
    if tokens.len() == 1 && is_punctuation_token(tokens[0]) {
        return GlyphCategory::PunctuationOrSymbol;
    }

    // Scan tokens for explicit casing and letter indicators
    let mut is_upper_indicator = false;
    let mut is_lower_indicator = false;
    let mut detected_letter: Option<char> = None;
    let mut detected_digit: Option<char> = None;

    for &token in &tokens {
        let t_lower = token.to_ascii_lowercase();
        if matches!(
            t_lower.as_str(),
            "upper" | "uppercase" | "caps" | "cap" | "uc"
        ) {
            is_upper_indicator = true;
            continue;
        }
        if matches!(t_lower.as_str(), "lower" | "lowercase" | "small" | "lc") {
            is_lower_indicator = true;
            continue;
        }
        if matches!(t_lower.as_str(), "number" | "num" | "digit") {
            continue;
        }
        if is_punctuation_token(token) {
            return GlyphCategory::PunctuationOrSymbol;
        }

        // Single letter token
        if token.len() == 1 {
            let ch = token.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                detected_letter = Some(ch);
                if ch.is_ascii_uppercase() {
                    is_upper_indicator = true;
                } else if ch.is_ascii_lowercase() {
                    is_lower_indicator = true;
                }
            } else if ch.is_ascii_digit() {
                detected_digit = Some(ch);
            }
        } else {
            // Check for prefix patterns like "UpperA", "Lowera", "CapsA"
            if t_lower.starts_with("upper") && token.len() == 6 {
                let ch = token.chars().nth(5).unwrap();
                if ch.is_ascii_alphabetic() {
                    detected_letter = Some(ch);
                    is_upper_indicator = true;
                }
            } else if t_lower.starts_with("lower") && token.len() == 6 {
                let ch = token.chars().nth(5).unwrap();
                if ch.is_ascii_alphabetic() {
                    detected_letter = Some(ch);
                    is_lower_indicator = true;
                }
            } else if t_lower.starts_with("uc") && token.len() == 3 {
                let ch = token.chars().nth(2).unwrap();
                if ch.is_ascii_alphabetic() {
                    detected_letter = Some(ch);
                    is_upper_indicator = true;
                }
            } else if t_lower.starts_with("lc") && token.len() == 3 {
                let ch = token.chars().nth(2).unwrap();
                if ch.is_ascii_alphabetic() {
                    detected_letter = Some(ch);
                    is_lower_indicator = true;
                }
            }
        }
    }

    // Resolve classification
    if let Some(ch) = detected_letter {
        let upper_ch = ch.to_ascii_uppercase();
        let lower_ch = ch.to_ascii_lowercase();

        if is_lower_indicator && !is_upper_indicator {
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

    if let Some(digit) = detected_digit {
        if digit == '9' {
            return GlyphCategory::Digit9;
        }
        return GlyphCategory::OtherDigit(digit);
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

    let is_alphabet = (has_keyword && glyph_count > 0)
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
