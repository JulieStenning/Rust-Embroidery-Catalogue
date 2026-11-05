// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! Domain service for user-defined and seeded tag word matches (synonyms).

use crate::error::AppError;
use inflector::Inflector;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

const PROHIBITED_DOMAIN_TERMS: &[&str] = &[
    "embroidery",
    "embroidered",
    "embroideries",
    "stitch",
    "stitched",
    "stitches",
    "stitching",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow)]
pub struct TagSynonymItem {
    pub id: i64,
    pub keyword: String,
    pub tag_id: i64,
    pub tag_description: String,
    pub tag_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow)]
pub struct TagSynonymKeyword {
    pub id: i64,
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagSynonymGroup {
    pub tag_id: i64,
    pub tag_description: String,
    pub tag_group: Option<String>,
    pub keywords: Vec<TagSynonymKeyword>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagSynonymsRequest {
    pub tag_id: i64,
    pub words_input: String,
}

/// Load the lookup map of normalized keyword -> list of tag descriptions.
/// Used directly by the tagging engine ([`crate::services::tagging`]).
pub async fn get_synonym_lookup_map(
    pool: &SqlitePool,
) -> Result<HashMap<String, Vec<String>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT ts.keyword, t.description
        FROM tag_synonyms ts
        JOIN tags t ON t.id = ts.tag_id
        ORDER BY ts.keyword COLLATE NOCASE ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to load tag synonyms: {e}")))?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let keyword: String = row
            .try_get("keyword")
            .map_err(|e| AppError::database(format!("failed to read keyword: {e}")))?;
        let description: String = row
            .try_get("description")
            .map_err(|e| AppError::database(format!("failed to read description: {e}")))?;
        let kw = keyword.trim().to_ascii_lowercase();
        if !kw.is_empty() {
            map.entry(kw).or_default().push(description);
        }
    }

    Ok(map)
}

/// List all tag synonyms with their tag descriptions and groups.
pub async fn list_tag_synonyms(pool: &SqlitePool) -> Result<Vec<TagSynonymItem>, AppError> {
    sqlx::query_as::<_, TagSynonymItem>(
        r#"
        SELECT
            ts.id,
            ts.keyword,
            ts.tag_id,
            t.description AS tag_description,
            t.tag_group AS tag_group
        FROM tag_synonyms ts
        JOIN tags t ON t.id = ts.tag_id
        ORDER BY t.tag_group ASC, t.description COLLATE NOCASE ASC, ts.keyword COLLATE NOCASE ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to list tag synonyms: {e}")))
}

#[derive(Debug, sqlx::FromRow)]
struct RawGroupRow {
    tag_id: i64,
    tag_description: String,
    tag_group: Option<String>,
    synonym_id: Option<i64>,
    keyword: Option<String>,
}

/// List all tags with their associated synonym keywords, grouped by tag.
pub async fn list_tag_synonyms_grouped(
    pool: &SqlitePool,
) -> Result<Vec<TagSynonymGroup>, AppError> {
    let rows = sqlx::query_as::<_, RawGroupRow>(
        r#"
        SELECT
            t.id AS tag_id,
            t.description AS tag_description,
            t.tag_group AS tag_group,
            ts.id AS synonym_id,
            ts.keyword AS keyword
        FROM tags t
        LEFT JOIN tag_synonyms ts ON ts.tag_id = t.id
        ORDER BY t.tag_group ASC, t.description COLLATE NOCASE ASC, ts.keyword COLLATE NOCASE ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to list grouped tag synonyms: {e}")))?;

    let mut groups: Vec<TagSynonymGroup> = Vec::new();

    for row in rows {
        let last_match = groups.last_mut().filter(|g| g.tag_id == row.tag_id);
        if let Some(group) = last_match {
            if let (Some(id), Some(kw)) = (row.synonym_id, row.keyword) {
                group.keywords.push(TagSynonymKeyword { id, keyword: kw });
            }
        } else {
            let mut keywords = Vec::new();
            if let (Some(id), Some(kw)) = (row.synonym_id, row.keyword) {
                keywords.push(TagSynonymKeyword { id, keyword: kw });
            }
            groups.push(TagSynonymGroup {
                tag_id: row.tag_id,
                tag_description: row.tag_description,
                tag_group: row.tag_group,
                keywords,
            });
        }
    }

    Ok(groups)
}

/// Add one or more word matches (comma, newline, or space separated) for a given tag.
pub async fn add_tag_synonyms(
    pool: &SqlitePool,
    tag_id: i64,
    words_input: &str,
) -> Result<Vec<TagSynonymKeyword>, AppError> {
    // Fetch the tag and its description
    let tag_desc =
        sqlx::query_scalar::<_, String>("SELECT description FROM tags WHERE id = ? LIMIT 1")
            .bind(tag_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::database(format!("database error checking tag: {e}")))?;

    let Some(tag_description) = tag_desc else {
        return Err(AppError::invalid_input(format!(
            "Tag ID {tag_id} not found"
        )));
    };

    let tag_words: HashSet<String> = tag_description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_lowercase())
        .collect();

    // Split words by commas, semicolons, or newlines
    let raw_tokens: Vec<&str> = words_input.split([',', ';', '\n', '\r']).collect();

    for token in raw_tokens {
        let clean = token.trim().to_lowercase();
        if clean.is_empty() {
            continue;
        }

        // 1. Prohibit universal domain terms
        if PROHIBITED_DOMAIN_TERMS.contains(&clean.as_str()) {
            return Err(AppError::invalid_input(format!(
                "'{clean}' is a universal embroidery term and cannot be assigned as a specific tag word match."
            )));
        }

        // 2. Prohibit self-synonyms (words matching the tag name itself)
        let singular = clean.to_singular();
        let plural = clean.to_plural();
        if tag_words.contains(&clean)
            || tag_words.contains(&singular)
            || tag_words.contains(&plural)
        {
            return Err(AppError::invalid_input(format!(
                "'{clean}' duplicates words already in the tag name '{tag_description}'. The tagging engine automatically matches words from the tag name in singular and plural forms."
            )));
        }

        // 3. Normalise to singular form to maintain the No Singular/Plural Duplication invariant
        let canonical_keyword = if !singular.is_empty() {
            singular
        } else {
            clean
        };

        sqlx::query("INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id) VALUES (?, ?)")
            .bind(&canonical_keyword)
            .bind(tag_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("failed to insert word match: {e}")))?;
    }

    // Return the updated keywords for this tag
    let keywords = sqlx::query_as::<_, TagSynonymKeyword>(
        r#"
        SELECT id, keyword
        FROM tag_synonyms
        WHERE tag_id = ?
        ORDER BY keyword COLLATE NOCASE ASC
        "#,
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to reload tag synonyms: {e}")))?;

    Ok(keywords)
}

/// Delete a single synonym record by ID.
pub async fn delete_tag_synonym(pool: &SqlitePool, synonym_id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tag_synonyms WHERE id = ?")
        .bind(synonym_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to delete tag synonym: {e}")))?;

    Ok(())
}

/// Delete all synonyms for a given tag.
pub async fn delete_all_tag_synonyms_for_tag(
    pool: &SqlitePool,
    tag_id: i64,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tag_synonyms WHERE tag_id = ?")
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to clear tag synonyms for tag: {e}")))?;

    Ok(())
}

#[cfg(test)]
#[path = "tag_synonyms_tests.rs"]
mod tests;
