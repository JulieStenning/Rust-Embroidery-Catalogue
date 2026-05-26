use std::collections::HashSet;

// Targeted alias map to bridge common filename keywords to canonical tag descriptions.
const KEYWORD_MAP: [(&str, &str); 24] = [
	("alphabet", "Alphabets"),
	("monogram", "Monogram"),
	("cat", "Cats"),
	("kitten", "Cats"),
	("dog", "Dogs"),
	("puppy", "Dogs"),
	("horse", "Horses"),
	("bird", "Birds"),
	("butterfly", "Butterflies and Insects"),
	("christmas", "Christmas"),
	("xmas", "Christmas"),
	("easter", "Easter"),
	("halloween", "Halloween"),
	("flower", "Flowers"),
	("floral", "Flowers"),
	("wedding", "Wedding"),
	("valentine", "Valentine's Day"),
	("mother", "Mother's Day"),
	("father", "Father's Day"),
	("baby", "Babies"),
	("angel", "Angels"),
	("fantasy", "Fantasy"),
	("nautical", "Nautical"),
	("transport", "Transport"),
];

fn normalize_text(value: &str) -> String {
	value
		.chars()
		.map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { ' ' })
		.collect::<String>()
}

fn tokenize(value: &str) -> HashSet<String> {
	normalize_text(value)
		.split_whitespace()
		.filter(|part| !part.is_empty())
		.map(String::from)
		.collect()
}

fn singularize(token: &str) -> Option<String> {
	if token.len() > 3 && token.ends_with('s') {
		return Some(token.trim_end_matches('s').to_string());
	}

	None
}

fn pluralize(token: &str) -> Option<String> {
	if token.len() > 2 && !token.ends_with('s') {
		return Some(format!("{}s", token));
	}

	None
}

fn token_matches(token: &str, tokens: &HashSet<String>) -> bool {
	if tokens.contains(token) {
		return true;
	}

	if singularize(token)
		.map(|singular| tokens.contains(&singular))
		.unwrap_or(false)
	{
		return true;
	}

	pluralize(token)
		.map(|plural| tokens.contains(&plural))
		.unwrap_or(false)
}

fn description_matches_text(description: &str, haystack: &str, tokens: &HashSet<String>) -> bool {
	let _ = haystack;
	let normalized_description = normalize_text(description);
	let description_tokens: Vec<&str> = normalized_description
		.split_whitespace()
		.filter(|part| part.len() > 2)
		.collect();

	if description_tokens.is_empty() {
		return false;
	}

	description_tokens
		.iter()
		.all(|token| token_matches(token, tokens))
}

pub fn suggest_tier1_descriptions(
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
	let normalized_haystack = normalize_text(&combined);
	let tokens = tokenize(&combined);

	let mut matched = HashSet::new();

	for (keyword, description) in KEYWORD_MAP {
		if !valid_descriptions.contains(description) {
			continue;
		}

		let keyword_matches = if keyword.contains('_') {
			keyword
				.split('_')
				.filter(|part| !part.is_empty())
				.all(|part| token_matches(part, &tokens))
		} else {
			token_matches(keyword, &tokens)
		};

		if keyword_matches {
			matched.insert(description.to_string());
		}
	}

	for description in valid_descriptions {
		if description_matches_text(description, &normalized_haystack, &tokens) {
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

	#[test]
	fn suggest_tier1_matches_alias_keyword() {
		let valid = HashSet::from(["Alphabets".to_string(), "Flowers".to_string()]);
		let matched = suggest_tier1_descriptions("my_alphabet_design.pes", "", &valid);
		assert!(matched.contains(&"Alphabets".to_string()));
	}

	#[test]
	fn suggest_tier1_uses_folder_tokens() {
		let valid = HashSet::from(["Alphabets".to_string()]);
		let matched = suggest_tier1_descriptions(
			"17147.hus",
			"C:/imports/Alphabets/Font Pack/17147.hus",
			&valid,
		);
		assert!(matched.contains(&"Alphabets".to_string()));
	}

	#[test]
	fn suggest_tier1_does_not_match_cat_inside_catalogue() {
		let valid = HashSet::from(["Cats".to_string(), "Crests".to_string()]);
		let matched = suggest_tier1_descriptions(
			"17147.hus",
			"D:/My Software Development/Rust-Embroidery-Catalogue/data/MachineEmbroideryDesigns/Amazing Designs - 1033 Crests/17147.hus",
			&valid,
		);

		assert!(matched.contains(&"Crests".to_string()));
		assert!(!matched.contains(&"Cats".to_string()));
	}
}
