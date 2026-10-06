// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BrowseAdditionalFiltersPayload {
    pub designer_filters: Option<Vec<String>>,
    pub image_tag_filters: Option<Vec<String>>,
    pub stitching_tag_filters: Option<Vec<String>>,
    pub source_filters: Option<Vec<String>>,
    pub hoop_size: Option<String>,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub min_rating: Option<i64>,
    pub stitched_status: Option<String>,
    /// When true, restrict to designs with no stored preview (`image_data IS NULL`) — the flagged
    /// "needs attention" set whose file may be corrupt or unreadable.
    pub needs_attention: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GetDesignsPayload {
    pub q: Option<String>,
    pub search_file_name: Option<bool>,
    pub search_tags: Option<bool>,
    pub search_folder_name: Option<bool>,
    pub unverified_only: Option<bool>,
    pub additional_filters: Option<BrowseAdditionalFiltersPayload>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralSearchToken {
    pub text: String,
    /// SQLite LIKE fragment with glob semantics applied (see `like_pattern`).
    /// Lowercased so it can be bound directly against `LOWER(...)` columns.
    pub pattern: String,
    pub phrase: bool,
    pub exclude: bool,
    pub is_extension: bool,
}

/// Sentinel for the hoop browse filter: selecting it matches designs whose
/// minimum fitting hoop could not be calculated (`designs.hoop_id IS NULL`).
/// Must stay in sync with `HOOP_UNKNOWN_FILTER` in
/// `frontend/src/lib/utils/hoopConstants.js`.
const HOOP_UNKNOWN_SENTINEL: &str = "__hoop_unknown__";

pub(crate) fn push_where_clause(query_builder: &mut QueryBuilder<Sqlite>, has_where: &mut bool) {
    if *has_where {
        query_builder.push(" AND ");
    } else {
        query_builder.push(" WHERE ");
        *has_where = true;
    }
}

/// Map the frontend's browse sort selection to a deterministic SQL ORDER BY
/// clause. The `filename`/`id` tiebreakers keep pagination stable across pages.
pub(crate) fn browse_sort_clause(sort_by: Option<&str>, sort_dir: Option<&str>) -> String {
    let direction = match sort_dir {
        Some(dir) if dir.eq_ignore_ascii_case("desc") => "DESC",
        _ => "ASC",
    };

    let column = match sort_by {
        Some(sort) if sort.eq_ignore_ascii_case("rating") => "COALESCE(d.rating, -1)",
        Some(sort) if sort.eq_ignore_ascii_case("stitched") => "d.is_stitched",
        // Approximates the frontend's "folder then filename" ordering, because
        // the parent directory name is a path prefix of `filepath`.
        Some(sort) if sort.eq_ignore_ascii_case("folder") => "d.filepath COLLATE NOCASE",
        Some(sort) if sort.eq_ignore_ascii_case("date_added") => "COALESCE(d.date_added, '')",
        _ => "d.filename COLLATE NOCASE",
    };

    format!("{column} {direction}, d.filename COLLATE NOCASE ASC, d.id ASC")
}

/// Push the filter predicates shared by the COUNT, page-id, and aggregate
/// queries. Every tag predicate uses a `d.id IN (SELECT ...)` subquery, so the
/// outer `design_tags`/`tags` join is only needed for aggregation, never for
/// filtering — which is what lets the COUNT and page-id queries stay cheap.
pub(crate) fn push_browse_filters(
    query_builder: &mut QueryBuilder<Sqlite>,
    payload: &GetDesignsPayload,
) {
    let mut has_where = false;

    let q_trimmed = payload
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(q) = q_trimmed {
        let search_file = payload.search_file_name.unwrap_or(true);
        let search_tags = payload.search_tags.unwrap_or(true);
        let search_folder = payload.search_folder_name.unwrap_or(true);
        let general_groups = parse_general_search_groups(q);

        if search_file || search_tags || search_folder {
            push_where_clause(query_builder, &mut has_where);
            push_general_search_clause(
                query_builder,
                search_file,
                search_tags,
                search_folder,
                &general_groups,
            );
        } else {
            // The user typed a query but scoped it to no fields at all, so no
            // design can match. Without this the query would be ignored and the
            // whole library returned, which contradicts the documented
            // "untick all -> no matching results" behaviour.
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("0 = 1");
        }
    }

    if payload.unverified_only.unwrap_or(false) {
        push_where_clause(query_builder, &mut has_where);
        query_builder.push("(d.image_tags_verified = 0 OR d.stitching_tags_verified = 0)");
    }

    if let Some(ref filters) = payload.additional_filters {
        let designer_filters = filters.designer_filters.as_deref().unwrap_or(&[]);
        if !designer_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            let has_unknown = designer_filters
                .iter()
                .any(|v| v.trim().eq_ignore_ascii_case("unknown"));
            let named_designers: Vec<&str> = designer_filters
                .iter()
                .map(|v| v.trim())
                .filter(|v| !v.eq_ignore_ascii_case("unknown") && !v.is_empty())
                .collect();

            if has_unknown && named_designers.is_empty() {
                query_builder.push("(d.designer_id IS NULL OR d.designer_id IN (SELECT id FROM designers WHERE LOWER(name) = 'unknown'))");
            } else if has_unknown {
                query_builder.push(
                    "(d.designer_id IS NULL OR d.designer_id IN (SELECT id FROM designers WHERE ",
                );
                for (index, value) in named_designers.iter().enumerate() {
                    if index > 0 {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("LOWER(name) = ");
                    query_builder.push_bind(value.to_lowercase());
                }
                query_builder.push(" OR LOWER(name) = 'unknown'))");
            } else {
                query_builder.push("d.designer_id IN (SELECT id FROM designers WHERE ");
                for (index, value) in named_designers.iter().enumerate() {
                    if index > 0 {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("LOWER(name) = ");
                    query_builder.push_bind(value.to_lowercase());
                }
                query_builder.push(")");
            }
        }

        let image_tag_filters = filters.image_tag_filters.as_deref().unwrap_or(&[]);
        if !image_tag_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("d.id IN (");
            query_builder.push(
                "SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ",
            );
            query_builder.push("lower(COALESCE(tags.tag_group, '')) != 'stitching' AND (");
            for (index, value) in image_tag_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(tags.description) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
            query_builder.push(")");
        }

        let stitching_tag_filters = filters.stitching_tag_filters.as_deref().unwrap_or(&[]);
        if !stitching_tag_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("d.id IN (");
            query_builder.push(
                "SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ",
            );
            query_builder.push("lower(COALESCE(tags.tag_group, '')) = 'stitching' AND (");
            for (index, value) in stitching_tag_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(tags.description) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
            query_builder.push(")");
        }

        let source_filters = filters.source_filters.as_deref().unwrap_or(&[]);
        if !source_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            let has_unknown = source_filters
                .iter()
                .any(|v| v.trim().eq_ignore_ascii_case("unknown"));
            let named_sources: Vec<&str> = source_filters
                .iter()
                .map(|v| v.trim())
                .filter(|v| !v.eq_ignore_ascii_case("unknown") && !v.is_empty())
                .collect();

            if has_unknown && named_sources.is_empty() {
                query_builder.push("(d.source_id IS NULL OR d.source_id IN (SELECT id FROM sources WHERE LOWER(name) = 'unknown'))");
            } else if has_unknown {
                query_builder
                    .push("(d.source_id IS NULL OR d.source_id IN (SELECT id FROM sources WHERE ");
                for (index, value) in named_sources.iter().enumerate() {
                    if index > 0 {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("LOWER(name) = ");
                    query_builder.push_bind(value.to_lowercase());
                }
                query_builder.push(" OR LOWER(name) = 'unknown'))");
            } else {
                query_builder.push("d.source_id IN (SELECT id FROM sources WHERE ");
                for (index, value) in named_sources.iter().enumerate() {
                    if index > 0 {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("LOWER(name) = ");
                    query_builder.push_bind(value.to_lowercase());
                }
                query_builder.push(")");
            }
        }

        if let Some(ref hoop_size) = filters.hoop_size {
            let hoop_size_trimmed = hoop_size.trim();
            if hoop_size_trimmed == HOOP_UNKNOWN_SENTINEL {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.hoop_id IS NULL");
            } else if !hoop_size_trimmed.is_empty() {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.hoop_id IN (SELECT id FROM hoops WHERE LOWER(name) = ");
                query_builder.push_bind(hoop_size_trimmed.to_lowercase());
                query_builder.push(")");
            }
        }

        match (filters.min_width, filters.max_width) {
            (Some(min), Some(max)) => {
                let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.width_mm IS NOT NULL AND d.width_mm >= ");
                query_builder.push_bind(lo.max(0.0));
                query_builder.push(" AND d.width_mm <= ");
                query_builder.push_bind(hi.max(0.0));
            }
            (Some(min), None) => {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.width_mm IS NOT NULL AND d.width_mm >= ");
                query_builder.push_bind(min.max(0.0));
            }
            (None, Some(max)) => {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.width_mm IS NOT NULL AND d.width_mm >= 0 AND d.width_mm <= ");
                query_builder.push_bind(max.max(0.0));
            }
            (None, None) => {}
        }

        match (filters.min_height, filters.max_height) {
            (Some(min), Some(max)) => {
                let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.height_mm IS NOT NULL AND d.height_mm >= ");
                query_builder.push_bind(lo.max(0.0));
                query_builder.push(" AND d.height_mm <= ");
                query_builder.push_bind(hi.max(0.0));
            }
            (Some(min), None) => {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.height_mm IS NOT NULL AND d.height_mm >= ");
                query_builder.push_bind(min.max(0.0));
            }
            (None, Some(max)) => {
                push_where_clause(query_builder, &mut has_where);
                query_builder
                    .push("d.height_mm IS NOT NULL AND d.height_mm >= 0 AND d.height_mm <= ");
                query_builder.push_bind(max.max(0.0));
            }
            (None, None) => {}
        }

        if let Some(min_rating) = filters.min_rating {
            if min_rating >= 1 {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.rating >= ");
                query_builder.push_bind(min_rating);
            }
        }

        if let Some(ref stitched_status) = filters.stitched_status {
            let stitched_status_trimmed = stitched_status.trim();
            if !stitched_status_trimmed.is_empty() && stitched_status_trimmed != "all" {
                push_where_clause(query_builder, &mut has_where);
                if stitched_status_trimmed == "yes" {
                    query_builder.push("d.is_stitched = 1");
                } else {
                    query_builder.push("d.is_stitched = 0");
                }
            }
        }

        if filters.needs_attention.unwrap_or(false) {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("d.image_data IS NULL");
        }
    }
}

pub(crate) fn parse_general_search_groups(query: &str) -> Vec<Vec<GeneralSearchToken>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut buffer = String::new();
    let mut in_quotes = false;

    for ch in trimmed.chars() {
        match ch {
            '"' => {
                buffer.push(ch);
                in_quotes = !in_quotes;
            }
            ' ' | '\t' | '\n' if !in_quotes => {
                let token = buffer.trim();
                if !token.is_empty() {
                    if token.eq_ignore_ascii_case("OR") {
                        if !current_group.is_empty() {
                            groups.push(std::mem::take(&mut current_group));
                        }
                    } else {
                        current_group.push(parse_general_token(token));
                    }
                    buffer.clear();
                }
            }
            _ => buffer.push(ch),
        }
    }

    if !buffer.trim().is_empty() {
        let token = buffer.trim();
        if token.eq_ignore_ascii_case("OR") {
            if !current_group.is_empty() {
                groups.push(std::mem::take(&mut current_group));
            }
        } else {
            current_group.push(parse_general_token(token));
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

pub(crate) fn parse_general_token(raw: &str) -> GeneralSearchToken {
    let trimmed = raw.trim();
    let mut exclude = false;
    let mut text = trimmed;

    if let Some(stripped) = trimmed.strip_prefix('-') {
        exclude = true;
        text = stripped;
    }

    let phrase = text.starts_with('"') && text.ends_with('"') || text.contains('"');
    let normalized = text.trim_matches('"').trim();

    let is_extension =
        normalized.starts_with("*") && normalized.len() > 1 && !normalized.contains(' ');
    let final_text = if is_extension {
        normalized
            .trim_start_matches('*')
            .trim_start_matches('.')
            .trim()
            .to_string()
    } else {
        normalized.to_string()
    };

    GeneralSearchToken {
        text: final_text,
        pattern: like_pattern(normalized),
        phrase,
        exclude,
        is_extension,
    }
}

/// Build a SQLite LIKE fragment from a user term using glob semantics.
///
/// - A bare term (no `*`) matches as a substring: `Sig4` → `%sig4%`.
/// - A `*` acts as `%` (zero-or-more characters) and un-anchors the edge it
///   touches:
///   - `Sig4*` → `sig4%` (starts with "Sig4")
///   - `*Sig4` → `%sig4` (ends with "Sig4")
///   - `*Sig4*` → `%sig4%` (contains "Sig4")
///   - `*.hus` → `%.hus` (ends with the ".hus" extension)
pub(crate) fn like_pattern(term: &str) -> String {
    let lower = term.to_lowercase();
    if lower.contains('*') {
        lower.replace('*', "%")
    } else {
        format!("%{lower}%")
    }
}

/// Folder-name search matches the canonical relative `filepath` directly.
pub(crate) fn library_folder_sql_expr(column: &str) -> String {
    column.to_string()
}

pub(crate) fn push_general_search_clause(
    query_builder: &mut QueryBuilder<Sqlite>,
    search_file: bool,
    search_tags: bool,
    search_folder: bool,
    general_groups: &[Vec<GeneralSearchToken>],
) {
    if general_groups.is_empty() {
        return;
    }

    query_builder.push("(");
    for (group_index, group_tokens) in general_groups.iter().enumerate() {
        if group_index > 0 {
            query_builder.push(" OR ");
        }

        if group_tokens.is_empty() {
            continue;
        }

        query_builder.push("(");
        for (token_index, token) in group_tokens.iter().enumerate() {
            if token_index > 0 {
                query_builder.push(" AND ");
            }

            let pattern = token.pattern.clone();
            // Wrap the whole per-token clause in parentheses: without them the
            // token's internal `file OR tags OR folder` alternatives would be
            // split by SQLite's operator precedence (AND binds tighter than OR),
            // so `word1 word2` would behave as `word1 OR word2`.
            query_builder.push("(");
            if token.exclude {
                query_builder.push("NOT (");
            }

            let mut added = false;
            if search_file {
                query_builder.push("LOWER(d.filename) LIKE ");
                query_builder.push_bind(pattern.clone());
                added = true;
            }

            if search_tags {
                if added {
                    query_builder.push(" OR ");
                }
                query_builder.push("d.id IN (SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE LOWER(tags.description) LIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(")");
                added = true;
            }

            if search_folder {
                if added {
                    query_builder.push(" OR ");
                }
                let folder_expr = library_folder_sql_expr("d.filepath");
                let folder_search_sql = format!("LOWER({folder_expr}) LIKE ");
                query_builder.push(&folder_search_sql);
                query_builder.push_bind(pattern);
            }

            if token.exclude {
                query_builder.push(")");
            }
            query_builder.push(")");
        }
        query_builder.push(")");
    }
    query_builder.push(")");
}
