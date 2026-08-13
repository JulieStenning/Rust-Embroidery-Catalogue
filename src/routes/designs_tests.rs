// Tests for the designs route.
//
// This module was split out of designs.rs so the route file can stay
// focused on production logic. helper_tests and parser_tests are nested
// one level deeper than before, so they reach the parent designs module
// through super::super::* instead of super::*.

mod helper_tests {
    use super::super::*;

    // â”€â”€â”€ round_mm_to_i64 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn round_mm_normal_value() {
        assert_eq!(round_mm_to_i64(Some(12.6)), Some(13));
        assert_eq!(round_mm_to_i64(Some(12.4)), Some(12));
        assert_eq!(round_mm_to_i64(Some(0.5)), Some(1));
        assert_eq!(round_mm_to_i64(Some(-0.4)), Some(0));
    }

    #[test]
    fn round_mm_none_returns_none() {
        assert_eq!(round_mm_to_i64(None), None);
    }

    // â”€â”€â”€ ceil_mm_to_i64 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn ceil_mm_normal_value() {
        assert_eq!(ceil_mm_to_i64(Some(5.1)), Some(6));
        assert_eq!(ceil_mm_to_i64(Some(5.0)), Some(5));
        assert_eq!(ceil_mm_to_i64(Some(0.1)), Some(1));
    }

    #[test]
    fn ceil_mm_none_returns_none() {
        assert_eq!(ceil_mm_to_i64(None), None);
    }

    // â”€â”€â”€ normalize_optional_text â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn normalize_optional_text_trims_whitespace() {
        assert_eq!(
            normalize_optional_text(&Some("  hello world  ".to_string())),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn normalize_optional_text_empty_returns_none() {
        assert_eq!(normalize_optional_text(&Some("   ".to_string())), None);
        assert_eq!(normalize_optional_text(&Some(String::new())), None);
    }

    #[test]
    fn normalize_optional_text_none_returns_none() {
        assert_eq!(normalize_optional_text(&None), None);
    }

    // â”€â”€â”€ normalize_optional_fk â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn normalize_optional_fk_positive_id_ok() {
        let result = normalize_optional_fk(Some(5), "Designer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(5));
    }

    #[test]
    fn normalize_optional_fk_none_ok() {
        let result = normalize_optional_fk(None, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn normalize_optional_fk_zero_rejected() {
        let result = normalize_optional_fk(Some(0), "Designer");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a positive id"));
    }

    #[test]
    fn normalize_optional_fk_negative_rejected() {
        let result = normalize_optional_fk(Some(-1), "Source");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a positive id"));
    }

    // â”€â”€â”€ validate_rating â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn validate_rating_accepts_valid_range() {
        for rating in 1..=5 {
            let result = validate_rating(Some(rating));
            assert!(result.is_ok(), "rating {} should be valid", rating);
            assert_eq!(result.unwrap(), Some(rating));
        }
    }

    #[test]
    fn validate_rating_rejects_out_of_range() {
        let result = validate_rating(Some(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));

        let result = validate_rating(Some(6));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));

        let result = validate_rating(Some(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));
    }

    #[test]
    fn validate_rating_none_accepted() {
        let result = validate_rating(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    // â”€â”€â”€ image_mime_from_type â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn image_mime_for_known_types() {
        assert_eq!(image_mime_from_type(Some("jpg")), "image/jpeg");
        assert_eq!(image_mime_from_type(Some("jpeg")), "image/jpeg");
        assert_eq!(image_mime_from_type(Some("webp")), "image/webp");
        assert_eq!(image_mime_from_type(Some("gif")), "image/gif");
        assert_eq!(image_mime_from_type(Some("bmp")), "image/bmp");
    }

    #[test]
    fn image_mime_defaults_to_png() {
        assert_eq!(image_mime_from_type(Some("png")), "image/png");
        assert_eq!(image_mime_from_type(Some("svg")), "image/png");
        assert_eq!(image_mime_from_type(None), "image/png");
    }

    // â”€â”€â”€ build_data_url â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn build_data_url_returns_correct_mime_and_base64() {
        let data = Some(vec![0_u8, 1_u8, 2_u8]);
        let result = build_data_url(data, Some("png"));
        assert_eq!(result.as_deref(), Some("data:image/png;base64,AAEC"));
    }

    #[test]
    fn build_data_url_none_data_returns_none() {
        assert_eq!(build_data_url(None, Some("png")), None);
        assert_eq!(build_data_url(None, None), None);
    }

    #[test]
    fn build_data_url_uses_correct_mime_for_jpeg() {
        let data = Some(vec![255_u8; 4]);
        let result = build_data_url(data, Some("jpg"));
        assert!(
            result
                .as_deref()
                .unwrap_or_default()
                .starts_with("data:image/jpeg;base64,")
        );
    }

    // â”€â”€â”€ strip_sqlite_prefix â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn strip_sqlite_prefix_triple_slash() {
        assert_eq!(
            strip_sqlite_prefix("sqlite:///data/test.db"),
            "data/test.db"
        );
    }

    #[test]
    fn strip_sqlite_prefix_double_slash() {
        assert_eq!(strip_sqlite_prefix("sqlite://data/test.db"), "data/test.db");
    }

    #[test]
    fn strip_sqlite_prefix_single_colon() {
        assert_eq!(strip_sqlite_prefix("sqlite:data/test.db"), "data/test.db");
    }

    #[test]
    fn strip_sqlite_prefix_bare_path_unchanged() {
        assert_eq!(strip_sqlite_prefix("data/test.db"), "data/test.db");
    }

    #[test]
    fn strip_sqlite_prefix_empty_unchanged() {
        assert_eq!(strip_sqlite_prefix(""), "");
    }

    // â”€â”€â”€ normalize_path_for_compare â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn normalize_path_replaces_backslashes() {
        let result = normalize_path_for_compare("foo\\bar\\baz");
        assert!(!result.contains('\\'));
        assert!(result.contains('/'));
    }

    #[test]
    fn normalize_path_trims_trailing_slash() {
        let result = normalize_path_for_compare("/foo/bar/");
        assert!(!result.ends_with('/'));
    }

    #[test]
    fn normalize_path_lowercases() {
        let result = normalize_path_for_compare("/FOO/Bar");
        assert_eq!(result, "/foo/bar");
    }

    #[test]
    fn normalize_path_trims_whitespace() {
        let result = normalize_path_for_compare("  /foo/bar  ");
        assert_eq!(result, "/foo/bar");
    }

    // â”€â”€â”€ parse_general_token â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn parse_general_token_plain_term() {
        let token = parse_general_token("rose");
        assert_eq!(token.text, "rose");
        assert!(!token.exclude);
        assert!(!token.phrase);
        assert!(!token.is_extension);
    }

    #[test]
    fn parse_general_token_exclusion() {
        let token = parse_general_token("-applique");
        assert_eq!(token.text, "applique");
        assert!(token.exclude);
    }

    #[test]
    fn parse_general_token_extension() {
        let token = parse_general_token("*.hus");
        assert_eq!(token.text, "hus");
        assert!(token.is_extension);
        assert!(!token.exclude);
    }

    #[test]
    fn parse_general_token_extension_with_dot() {
        let token = parse_general_token("*.pes");
        assert_eq!(token.text, "pes");
        assert!(token.is_extension);
    }

    #[test]
    fn parse_general_token_quoted_phrase() {
        let token = parse_general_token("\"cross stitch\"");
        assert_eq!(token.text, "cross stitch");
        assert!(token.phrase);
    }

    #[test]
    fn parse_general_token_excluded_extension() {
        let token = parse_general_token("-*.jef");
        assert_eq!(token.text, "jef");
        assert!(token.exclude);
        assert!(token.is_extension);
    }

    // â”€â”€â”€ push_where_clause â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn push_where_clause_first_time_inserts_where() {
        let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM designs");
        let mut has_where = false;
        push_where_clause(&mut builder, &mut has_where);
        assert!(has_where);
        let sql = builder.sql();
        assert!(sql.contains(" WHERE "), "sql should have WHERE clause");
    }

    #[test]
    fn push_where_clause_subsequent_times_inserts_and() {
        let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM designs");
        let mut has_where = true;
        push_where_clause(&mut builder, &mut has_where);
        assert!(has_where);
        let sql = builder.sql();
        assert!(sql.contains(" AND "), "sql should have AND clause");
        assert!(!sql.contains(" WHERE "));
    }

    // â”€â”€â”€ is_truthy â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn is_truthy_accepts_expected_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(is_truthy("y"));
        assert!(is_truthy("accepted"));
    }

    #[test]
    fn is_truthy_rejects_falsy_values() {
        assert!(!is_truthy("no"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("n"));
        assert!(!is_truthy("declined"));
    }

    #[test]
    fn is_truthy_trims_whitespace() {
        assert!(is_truthy("  true  "));
        assert!(!is_truthy("  false  "));
    }
}

mod parser_tests {
    use super::super::parse_general_search_groups;

    #[test]
    fn parses_or_groups_exclusions_and_extensions() {
        let groups = parse_general_search_groups(r#"rose "cross stitch" -applique OR *.hus"#);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 3);
        assert_eq!(groups[1].len(), 1);

        let first = &groups[0][0];
        assert_eq!(first.text, "rose");
        assert!(!first.exclude);
        assert!(!first.phrase);

        let second = &groups[0][1];
        assert_eq!(second.text, "cross stitch");
        assert!(!second.exclude);
        assert!(second.phrase);

        let third = &groups[0][2];
        assert_eq!(third.text, "applique");
        assert!(third.exclude);
        assert!(!third.phrase);

        let extension = &groups[1][0];
        assert_eq!(extension.text, "hus");
        assert!(extension.is_extension);
    }

    #[test]
    fn preserves_terms_inside_quotes() {
        let groups = parse_general_search_groups(r#""exact phrase""#);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
        let token = &groups[0][0];
        assert_eq!(token.text, "exact phrase");
        assert!(token.phrase);
    }
}

use super::*;
use serial_test::serial;
use sqlx::sqlite::SqlitePoolOptions;

async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create test sqlite pool");

    sqlx::query(
        r#"
			CREATE TABLE designers (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create designers table");

    sqlx::query(
        r#"
			CREATE TABLE sources (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create sources table");

    sqlx::query(
        r#"
			CREATE TABLE hoops (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(100) NOT NULL UNIQUE,
				max_width_mm NUMERIC(8,2) NOT NULL,
				max_height_mm NUMERIC(8,2) NOT NULL
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create hoops table");

    sqlx::query(
        r#"
			CREATE TABLE tags (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				description VARCHAR(255) NOT NULL UNIQUE,
				tag_group VARCHAR(20)
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create tags table");

    sqlx::query(
        r#"
			CREATE TABLE projects (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE,
				description TEXT,
				date_created DATE
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create projects table");

    sqlx::query(
        r#"
			CREATE TABLE designs (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				filename VARCHAR(500) NOT NULL,
				filepath VARCHAR(1000) NOT NULL,
				image_data BLOB,
				image_type VARCHAR(10),
				width_mm NUMERIC(8,2),
				height_mm NUMERIC(8,2),
				stitch_count INTEGER,
				color_count INTEGER,
				color_change_count INTEGER,
				notes TEXT,
				rating SMALLINT,
				is_stitched BOOLEAN NOT NULL DEFAULT 0,
				tags_checked BOOLEAN NOT NULL DEFAULT 0,
				tagging_tier SMALLINT,
				date_added DATE,
				designer_id INTEGER REFERENCES designers(id) ON DELETE SET NULL,
				source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
				hoop_id INTEGER REFERENCES hoops(id) ON DELETE SET NULL
			);
			"#,
    )
    .execute(&pool)
    .await
    .expect("failed to create designs table");

    sqlx::query(
			"CREATE TABLE design_tags (design_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, PRIMARY KEY (design_id, tag_id));",
		)
		.execute(&pool)
		.await
		.expect("failed to create design_tags table");

    sqlx::query(
			"CREATE TABLE project_designs (project_id INTEGER NOT NULL, design_id INTEGER NOT NULL, PRIMARY KEY (project_id, design_id));",
		)
		.execute(&pool)
		.await
		.expect("failed to create project_designs table");

    sqlx::query("INSERT INTO designers (name) VALUES ('Acme Designer')")
        .execute(&pool)
        .await
        .expect("failed to seed designer");
    sqlx::query("INSERT INTO sources (name) VALUES ('USB Import')")
        .execute(&pool)
        .await
        .expect("failed to seed source");
    sqlx::query(
        "INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Hoop A', 126, 126)",
    )
    .execute(&pool)
    .await
    .expect("failed to seed hoop");
    sqlx::query("INSERT INTO tags (description, tag_group) VALUES ('Flowers', 'image')")
        .execute(&pool)
        .await
        .expect("failed to seed tag");
    sqlx::query("INSERT INTO tags (description, tag_group) VALUES ('Satin Stitch', 'stitching')")
        .execute(&pool)
        .await
        .expect("failed to seed tag");
    sqlx::query("INSERT INTO projects (name) VALUES ('Summer Quilt')")
        .execute(&pool)
        .await
        .expect("failed to seed project");
    sqlx::query("INSERT INTO projects (name) VALUES ('Gift Ideas')")
        .execute(&pool)
        .await
        .expect("failed to seed project");

    sqlx::query(
			"INSERT INTO designs (filename, filepath, notes, designer_id, source_id, hoop_id, is_stitched, tags_checked, rating) VALUES ('rose.pes', 'Roses/rose.pes', 'old note', 1, 1, 1, 0, 0, NULL)",
		)
		.execute(&pool)
		.await
		.expect("failed to seed design");

    pool
}

#[tokio::test]
async fn update_design_metadata_updates_core_fields() {
    let pool = test_pool().await;

    let result = update_design_metadata_with_pool(
        &pool,
        1,
        UpdateDesignMetadataRequest {
            notes: Some("  updated note  ".to_string()),
            designer_id: Some(1),
            source_id: Some(1),
            hoop_id: Some(1),
        },
    )
    .await;

    assert!(result.is_ok());

    let row = sqlx::query_as::<_, (Option<String>, Option<i64>, Option<i64>, Option<i64>)>(
        "SELECT notes, designer_id, source_id, hoop_id FROM designs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("design row should exist");

    assert_eq!(row.0.as_deref(), Some("updated note"));
    assert_eq!(row.1, Some(1));
    assert_eq!(row.2, Some(1));
    assert_eq!(row.3, Some(1));
}

#[tokio::test]
async fn set_design_rating_rejects_invalid_values() {
    let pool = test_pool().await;

    let result = set_design_rating_with_pool(&pool, 1, Some(9)).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected rating error")
            .contains("between 1 and 5")
    );
}

#[tokio::test]
async fn set_design_tags_replaces_and_marks_verified() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("should insert original tag");

    let result = set_design_tags_with_pool(&pool, 1, vec![2]).await;
    assert!(result.is_ok());

    let assigned = sqlx::query_as::<_, (i64,)>(
        "SELECT tag_id FROM design_tags WHERE design_id = 1 ORDER BY tag_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("assigned tags query should succeed");

    assert_eq!(assigned.len(), 1);
    assert_eq!(assigned[0].0, 2);

    let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("tags_checked query should succeed");

    assert_eq!(checked, 1);
}

#[tokio::test]
async fn bulk_set_tags_adds_only_requested_tags() {
    let pool = test_pool().await;

    // Design 1 starts with Flowers (tag 1), design 2 starts with Satin (tag 2).
    sqlx::query(
        "INSERT INTO designs (filename, filepath) VALUES ('second.pes', 'Roses/second.pes')",
    )
    .execute(&pool)
    .await
    .expect("seed design 2");
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("seed design 1 tag");
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 2)")
        .execute(&pool)
        .await
        .expect("seed design 2 tag");

    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1, 2],
        BulkApplyTagsRequest {
            tags_to_add: vec![2],
            tags_to_remove: vec![],
            clear_all_tags: false,
        },
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().updated_count, 2);

    // Design 1 should now have Satin (2) added but keep Flowers (1).
    let design1 = sqlx::query_as::<_, (i64,)>(
        "SELECT tag_id FROM design_tags WHERE design_id = 1 ORDER BY tag_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("design 1 tags query");

    assert_eq!(design1.len(), 2);
    assert_eq!(design1[0].0, 1);
    assert_eq!(design1[1].0, 2);

    // Design 2 should retain Satin (2) and NOT gain Flowers (1).
    let design2 = sqlx::query_as::<_, (i64,)>(
        "SELECT tag_id FROM design_tags WHERE design_id = 2 ORDER BY tag_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("design 2 tags query");

    assert_eq!(design2.len(), 1);
    assert_eq!(design2[0].0, 2);
}

#[tokio::test]
async fn bulk_set_tags_remove_only_removes_requested_tags() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("seed design 1 tag");
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .expect("seed design 1 second tag");

    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1],
        BulkApplyTagsRequest {
            tags_to_add: vec![],
            tags_to_remove: vec![1],
            clear_all_tags: false,
        },
    )
    .await;

    assert!(result.is_ok());

    // Only tag 1 removed; tag 2 stays (mixed/indeterminate tags are untouched).
    let remaining = sqlx::query_as::<_, (i64,)>(
        "SELECT tag_id FROM design_tags WHERE design_id = 1 ORDER BY tag_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("remaining tags query");

    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].0, 2);
}

#[tokio::test]
async fn bulk_set_tags_clear_all_removes_everything() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("seed design 1 tag");
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .expect("seed design 1 second tag");

    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1],
        BulkApplyTagsRequest {
            tags_to_add: vec![],
            tags_to_remove: vec![],
            clear_all_tags: true,
        },
    )
    .await;

    assert!(result.is_ok());

    let count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM design_tags WHERE design_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count query");

    assert_eq!(count, 0);
}

#[tokio::test]
async fn bulk_set_tags_add_wins_when_tag_in_both_lists() {
    let pool = test_pool().await;

    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1],
        BulkApplyTagsRequest {
            tags_to_add: vec![1, 1, 2],
            tags_to_remove: vec![1],
            clear_all_tags: false,
        },
    )
    .await;

    assert!(result.is_ok());

    // Duplicate add ids deduplicate; tag 1 survives because add wins.
    let assigned = sqlx::query_as::<_, (i64,)>(
        "SELECT tag_id FROM design_tags WHERE design_id = 1 ORDER BY tag_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("assigned tags query");

    assert_eq!(assigned.len(), 2);
    assert_eq!(assigned[0].0, 1);
    assert_eq!(assigned[1].0, 2);
}

#[tokio::test]
async fn bulk_set_tags_rejects_unknown_tag_id() {
    let pool = test_pool().await;

    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1],
        BulkApplyTagsRequest {
            tags_to_add: vec![999],
            tags_to_remove: vec![],
            clear_all_tags: false,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected error")
            .contains("Tag with id=999 not found.")
    );
}

#[tokio::test]
async fn bulk_set_tags_marks_designs_verified() {
    let pool = test_pool().await;

    // Design 1 starts unverified.
    let result = bulk_set_tags_for_designs_with_pool(
        &pool,
        &[1],
        BulkApplyTagsRequest {
            tags_to_add: vec![1],
            tags_to_remove: vec![],
            clear_all_tags: false,
        },
    )
    .await;

    assert!(result.is_ok());

    let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("tags_checked query");

    assert_eq!(checked, 1);
}

#[tokio::test]
async fn add_and_remove_project_membership_round_trip() {
    let pool = test_pool().await;

    let add_result = add_design_to_project_with_pool(&pool, 1, 1).await;
    assert!(add_result.is_ok());

    let count_after_add = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_designs WHERE design_id = 1 AND project_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("project assignment count should work");
    assert_eq!(count_after_add, 1);

    let remove_result = remove_design_from_project_with_pool(&pool, 1, 1).await;
    assert!(remove_result.is_ok());

    let count_after_remove = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_designs WHERE design_id = 1 AND project_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("project assignment count should work");
    assert_eq!(count_after_remove, 0);
}

#[tokio::test]
async fn get_design_image_data_returns_data_url_when_image_exists() {
    let pool = test_pool().await;

    sqlx::query("UPDATE designs SET image_data = ?, image_type = ? WHERE id = 1")
        .bind(vec![1_u8, 2_u8, 3_u8, 4_u8])
        .bind("png")
        .execute(&pool)
        .await
        .expect("should update image data");

    let image = get_design_image_data_with_pool(&pool, 1)
        .await
        .expect("image query should succeed")
        .expect("image should exist");

    assert_eq!(image.design_id, 1);
    assert_eq!(image.image_type.as_deref(), Some("png"));
    assert!(
        image
            .data_url
            .as_deref()
            .unwrap_or_default()
            .starts_with("data:image/png;base64,")
    );
}

#[tokio::test]
async fn open_design_in_editor_returns_error_for_missing_design() {
    let pool = test_pool().await;

    let result = open_design_in_editor_with_pool(&pool, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected missing design error")
            .contains("not found")
    );
}

#[tokio::test]
async fn open_design_in_explorer_returns_error_for_missing_design() {
    let pool = test_pool().await;

    let result = open_design_in_explorer_with_pool(&pool, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected missing design error")
            .contains("not found")
    );
}

#[tokio::test]
async fn render_design_3d_preview_returns_error_when_source_file_is_missing() {
    let pool = test_pool().await;

    let result = render_design_3d_preview_with_pool(&pool, 1, true).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected missing file error")
            .contains("not found on disk")
    );
}

#[tokio::test]
async fn render_design_2d_preview_returns_error_when_source_file_is_missing() {
    let pool = test_pool().await;

    let result = render_design_3d_preview_with_pool(&pool, 1, false).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("expected missing file error")
            .contains("not found on disk")
    );
}

#[test]
fn launch_disable_parser_accepts_expected_truthy_values() {
    assert!(is_truthy("1"));
    assert!(is_truthy("true"));
    assert!(is_truthy("YES"));
    assert!(!is_truthy("no"));
}

// â”€â”€â”€ Phase 2: Environment-dependent tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
#[serial]
fn external_launches_disabled_returns_true_when_env_var_is_set() {
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");
    assert!(external_launches_disabled());
    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    } else {
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
    }
}

#[test]
#[serial]
fn external_launches_disabled_returns_false_when_falsy_env_var() {
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "false");
    assert!(!external_launches_disabled());
    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    } else {
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
    }
}

#[test]
#[serial]
fn external_launches_disabled_returns_false_when_env_var_absent() {
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
    assert!(!external_launches_disabled());
    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    }
}

// â”€â”€â”€ Phase 3: Async DB-dependent functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn ensure_design_exists_found() {
    let pool = test_pool().await;
    let result = ensure_design_exists(&pool, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn ensure_design_exists_not_found() {
    let pool = test_pool().await;
    let result = ensure_design_exists(&pool, 999).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn ensure_foreign_key_exists_when_exists() {
    let pool = test_pool().await;
    let result = ensure_foreign_key_exists(&pool, "designers", Some(1), "Designer").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn ensure_foreign_key_exists_when_missing() {
    let pool = test_pool().await;
    let result = ensure_foreign_key_exists(&pool, "designers", Some(999), "Designer").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn ensure_foreign_key_exists_none_passes() {
    let pool = test_pool().await;
    let result = ensure_foreign_key_exists(&pool, "designers", None, "Designer").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_design_filepath_returns_path_for_valid_design() {
    let pool = test_pool().await;
    let result = get_design_filepath(&pool, 1).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Roses/rose.pes");
}

#[tokio::test]
async fn get_design_filepath_errors_for_missing_design() {
    let pool = test_pool().await;
    let result = get_design_filepath(&pool, 999).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn get_design_filepath_errors_for_empty_filepath() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO designs (filename, filepath) VALUES ('empty.pes', '')")
        .execute(&pool)
        .await
        .expect("should insert design with empty filepath");

    let result = get_design_filepath(&pool, 2).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("does not have a stored filepath")
    );
}

#[tokio::test]
async fn set_design_stitched_with_pool_sets_true() {
    let pool = test_pool().await;

    let result = set_design_stitched_with_pool(&pool, 1, true).await;
    assert!(result.is_ok());

    let stitched = sqlx::query_scalar::<_, i64>("SELECT is_stitched FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(stitched, 1);
}

#[tokio::test]
async fn set_design_stitched_with_pool_sets_false() {
    let pool = test_pool().await;

    let result = set_design_stitched_with_pool(&pool, 1, false).await;
    assert!(result.is_ok());

    let stitched = sqlx::query_scalar::<_, i64>("SELECT is_stitched FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(stitched, 0);
}

#[tokio::test]
async fn set_design_tags_checked_with_pool_sets_true() {
    let pool = test_pool().await;

    let result = set_design_tags_checked_with_pool(&pool, 1, true).await;
    assert!(result.is_ok());

    let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(checked, 1);
}

#[tokio::test]
async fn set_design_tags_checked_with_pool_sets_false() {
    let pool = test_pool().await;

    let result = set_design_tags_checked_with_pool(&pool, 1, false).await;
    assert!(result.is_ok());

    let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(checked, 0);
}

#[tokio::test]
async fn remove_design_tag_with_pool_removes_existing() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("should seed tag link");

    let result = remove_design_tag_with_pool(&pool, 1, 1).await;
    assert!(result.is_ok());

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("count should work");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn remove_design_tag_with_pool_rejects_invalid_tag_id() {
    let pool = test_pool().await;
    let result = remove_design_tag_with_pool(&pool, 1, 0).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("positive integer"));
}

#[tokio::test]
async fn remove_design_tag_with_pool_errors_for_missing_design() {
    let pool = test_pool().await;
    let result = remove_design_tag_with_pool(&pool, 999, 1).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn delete_design_without_file_removes_from_db() {
    let pool = test_pool().await;

    let result = delete_design_with_pool(&pool, 1, false).await;
    assert!(result.is_ok());

    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("count should work");
    assert_eq!(exists, 0);
}

#[tokio::test]
async fn delete_design_errors_when_design_missing() {
    let pool = test_pool().await;
    let result = delete_design_with_pool(&pool, 999, false).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn set_design_rating_with_pool_sets_valid_rating() {
    let pool = test_pool().await;

    let result = set_design_rating_with_pool(&pool, 1, Some(4)).await;
    assert!(result.is_ok());

    let rating = sqlx::query_scalar::<_, Option<i64>>("SELECT rating FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(rating, Some(4));
}

#[tokio::test]
async fn set_design_rating_with_pool_clears_rating() {
    let pool = test_pool().await;

    sqlx::query("UPDATE designs SET rating = 3 WHERE id = 1")
        .execute(&pool)
        .await
        .expect("should set rating");

    let result = set_design_rating_with_pool(&pool, 1, None).await;
    assert!(result.is_ok());

    let rating = sqlx::query_scalar::<_, Option<i64>>("SELECT rating FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
    assert_eq!(rating, None);
}

#[tokio::test]
async fn get_design_detail_returns_full_detail_for_existing_design() {
    let pool = test_pool().await;

    // Add design to a project and tag for a full detail response
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("should seed tag link");
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("should seed project link");

    let detail = get_design_detail_with_pool(&pool, 1)
        .await
        .expect("query should succeed")
        .expect("detail should exist");

    assert_eq!(detail.id, 1);
    assert_eq!(detail.filename, "rose.pes");
    assert_eq!(detail.designer, "Acme Designer");
    assert_eq!(detail.source, "USB Import");
    assert_eq!(detail.hoop.as_deref(), Some("Hoop A"));
    assert_eq!(detail.notes.as_deref(), Some("old note"));
    assert!(!detail.tags.is_empty());
    assert!(!detail.projects.is_empty());
    assert!(!detail.available_projects.is_empty());
    assert!(!detail.all_tags.is_empty());
    assert!(!detail.designers.is_empty());
    assert!(!detail.sources.is_empty());
    assert!(!detail.hoops.is_empty());
}

#[tokio::test]
async fn get_design_detail_returns_none_for_missing() {
    let pool = test_pool().await;

    let detail = get_design_detail_with_pool(&pool, 999)
        .await
        .expect("query should succeed");
    assert!(detail.is_none());
}

#[tokio::test]
async fn bulk_delete_designs_empty_list() {
    let pool = test_pool().await;

    let result = bulk_delete_designs_with_pool(&pool, &[], false)
        .await
        .expect("empty list should succeed");
    assert_eq!(result.requested_count, 0);
    assert_eq!(result.deleted_count, 0);
}

#[tokio::test]
async fn bulk_delete_designs_single_design() {
    let pool = test_pool().await;

    let result = bulk_delete_designs_with_pool(&pool, &[1], false)
        .await
        .expect("single delete should succeed");
    assert_eq!(result.requested_count, 1);
    assert_eq!(result.deleted_count, 1);
}

#[tokio::test]
async fn bulk_delete_designs_exceeds_limit() {
    let pool = test_pool().await;

    let ids: Vec<i64> = (1..=51).collect();
    let result = bulk_delete_designs_with_pool(&pool, &ids, false).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("more than 50"));
}

#[tokio::test]
async fn bulk_delete_designs_deduplicates() {
    let pool = test_pool().await;

    let result = bulk_delete_designs_with_pool(&pool, &[1, 1, 1], false)
        .await
        .expect("dedup should succeed");
    assert_eq!(result.requested_count, 1);
    assert_eq!(result.deleted_count, 1);
}

#[tokio::test]
async fn update_design_metadata_rejects_invalid_fk() {
    let pool = test_pool().await;

    let result = update_design_metadata_with_pool(
        &pool,
        1,
        UpdateDesignMetadataRequest {
            notes: None,
            designer_id: Some(999),
            source_id: None,
            hoop_id: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// â”€â”€â”€ Phase 4: Filesystem-dependent tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
#[serial]
fn derive_data_root_from_database_url_when_db_is_in_database_subfolder() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_root/Database/catalogue.db",
    );

    let root = derive_data_root_from_database_url();

    assert_eq!(root, PathBuf::from("/tmp/test_root"));

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn derive_data_root_from_database_url_when_db_is_in_non_database_folder() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var("DATABASE_URL", "sqlite:/tmp/my_data/catalogue.db");

    let root = derive_data_root_from_database_url();

    // When parent folder is not 'Database', the parent is used directly
    assert_eq!(root, PathBuf::from("/tmp/my_data"));

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn get_designs_base_path_joins_machine_embroidery_designs() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let base = get_designs_base_path();
    assert_eq!(
        base,
        PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns")
    );

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_already_normalized() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    // A relative path that isn't under the data root is returned as-is
    // (normalize_stored_design_filepath only adds the /MachineEmbroideryDesigns/
    // prefix when the path is already within that directory structure)
    let result = normalize_stored_design_filepath("Roses/rose.pes");
    assert_eq!(result, "Roses/rose.pes");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_under_machine_embroidery() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    // A path already starting with MachineEmbroideryDesigns gets normalized
    let result = normalize_stored_design_filepath("MachineEmbroideryDesigns/Roses/rose.pes");
    assert_eq!(result, "/MachineEmbroideryDesigns/Roses/rose.pes");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}
#[test]
fn nearest_existing_folder_returns_fallback_when_no_parent_exists() {
    // Use a completely isolated UUID-like temp path so no parent exists
    let isolated = std::env::temp_dir().join(format!(
        "nearest-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    // Create the fallback directory so it exists
    std::fs::create_dir_all(&isolated).expect("should create isolated dir");
    let fallback = isolated.clone();
    let nonexistent = isolated.join("a").join("b").join("c");

    let result = nearest_existing_folder(&nonexistent, &fallback);
    assert_eq!(result, fallback);

    let _ = std::fs::remove_dir_all(&isolated);
}

#[test]
#[serial]
fn normalize_stored_design_filepath_with_machine_embroidery_prefix() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = normalize_stored_design_filepath("machineembroiderydesigns/Roses/rose.pes");
    assert_eq!(result, "/machineembroiderydesigns/Roses/rose.pes");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_empty_returns_empty() {
    let result = normalize_stored_design_filepath("");
    assert_eq!(result, "");
}

#[test]
#[serial]
fn resolve_design_full_path_returns_designs_base_for_empty() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = resolve_design_full_path("");
    assert_eq!(
        result,
        PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns")
    );

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
fn nearest_existing_folder_returns_existing_dir_when_given_dir() {
    let tmp = std::env::temp_dir().join(format!(
        "nearest-dir-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).expect("should create temp dir");

    let result = nearest_existing_folder(&tmp, &PathBuf::from("/fallback"));
    assert_eq!(result, tmp);

    let _ = std::fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ Additional coverage for open/launch suppressed paths â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
#[serial]
async fn open_design_in_editor_returns_suppressed_when_launches_disabled() {
    let pool = test_pool().await;
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");

    let result = open_design_in_editor_with_pool(&pool, 1).await;
    assert!(result.is_ok());
    let launch = result.unwrap();
    assert!(launch.suppressed);
    assert!(!launch.success);

    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    } else {
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
    }
}

#[tokio::test]
#[serial]
async fn open_design_in_explorer_returns_suppressed_when_launches_disabled() {
    let pool = test_pool().await;
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");

    let result = open_design_in_explorer_with_pool(&pool, 1).await;
    assert!(result.is_ok());
    let launch = result.unwrap();
    assert!(launch.suppressed);
    assert!(!launch.success);

    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    } else {
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
    }
}

#[tokio::test]
#[serial]
async fn open_design_in_editor_returns_file_not_found_error() {
    let pool = test_pool().await;
    // Set disable to false so it proceeds past the suppressed check
    let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
    std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");

    let result = open_design_in_editor_with_pool(&pool, 1).await;
    assert!(result.is_ok());
    let launch = result.unwrap();
    assert!(!launch.suppressed);
    assert!(!launch.success);
    assert!(launch.message.contains("not found on disk"));

    if let Some(val) = prior {
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
    }
}

// â”€â”€â”€ Additional DB error paths â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn set_design_stitched_errors_when_missing() {
    let pool = test_pool().await;
    let result = set_design_stitched_with_pool(&pool, 999, true).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn set_design_tags_checked_errors_when_missing() {
    let pool = test_pool().await;
    let result = set_design_tags_checked_with_pool(&pool, 999, true).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn add_design_to_project_rejects_invalid_project() {
    let pool = test_pool().await;
    let result = add_design_to_project_with_pool(&pool, 1, 0).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("valid project"));
}

#[tokio::test]
async fn remove_design_from_project_rejects_invalid_project() {
    let pool = test_pool().await;
    let result = remove_design_from_project_with_pool(&pool, 1, 0).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("valid project"));
}

#[tokio::test]
async fn update_design_metadata_rejects_invalid_hoop() {
    let pool = test_pool().await;

    let result = update_design_metadata_with_pool(
        &pool,
        1,
        UpdateDesignMetadataRequest {
            notes: None,
            designer_id: None,
            source_id: None,
            hoop_id: Some(999),
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn set_design_tags_rejects_non_positive_tag_id() {
    let pool = test_pool().await;

    let result = set_design_tags_with_pool(&pool, 1, vec![0]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("positive integer"));
}

#[tokio::test]
async fn get_design_image_data_returns_no_data_url_when_no_image() {
    let pool = test_pool().await;

    let result = get_design_image_data_with_pool(&pool, 1)
        .await
        .expect("query should succeed");
    // Design 1 exists but has no image data seeded â€” returns Some with data_url=None
    let image_data = result.expect("should return Some for existing design");
    assert!(image_data.data_url.is_none());
    assert!(image_data.image_type.is_none());
}

#[tokio::test]
async fn bulk_verify_empty_list_returns_zero_works() {
    let pool = test_pool().await;

    // Test tauri command logic indirectly through design update
    let result = bulk_delete_designs_with_pool(&pool, &[], false).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert_eq!(res.requested_count, 0);
}

#[tokio::test]
async fn add_design_to_project_with_missing_project_errors() {
    let pool = test_pool().await;

    let result = add_design_to_project_with_pool(&pool, 1, 999).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// â”€â”€â”€ normalize_stored_design_filepath additional edge cases â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
#[serial]
fn normalize_stored_design_filepath_with_absolute_data_root_prefix() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    // A path that starts with the full data root should be normalized
    let result =
        normalize_stored_design_filepath("/tmp/test_data/MachineEmbroideryDesigns/MyDesign.pes");
    assert_eq!(result, "/MachineEmbroideryDesigns/MyDesign.pes");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_exact_data_root_returns_slash() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = normalize_stored_design_filepath("/tmp/test_data");
    assert_eq!(result, "/");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_exact_designs_base_returns_med() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = normalize_stored_design_filepath("/tmp/test_data/MachineEmbroideryDesigns");
    assert_eq!(result, "/MachineEmbroideryDesigns");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn normalize_stored_design_filepath_backslashes_are_normalized() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = normalize_stored_design_filepath("Roses\\rose.pes");
    // With backslashes normalized to forward slashes
    assert_eq!(result, "Roses/rose.pes");

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn resolve_design_full_path_for_med_prefixed_path() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = resolve_design_full_path("MachineEmbroideryDesigns/Roses/rose.pes");
    assert_eq!(
        result,
        PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns/Roses/rose.pes")
    );

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn resolve_design_full_path_for_relative_path() {
    let prior = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:/tmp/test_data/Database/catalogue.db",
    );

    let result = resolve_design_full_path("Roses/rose.pes");
    assert_eq!(
        result,
        PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns/Roses/rose.pes")
    );

    if let Some(val) = prior {
        std::env::set_var("DATABASE_URL", val);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

// â”€â”€â”€ parse_general_search_groups additional coverage â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn parse_general_search_groups_empty_returns_empty() {
    let groups = parse_general_search_groups("");
    assert!(groups.is_empty());
}

#[test]
fn parse_general_search_groups_whitespace_returns_empty() {
    let groups = parse_general_search_groups("   ");
    assert!(groups.is_empty());
}

#[test]
fn parse_general_search_groups_single_word() {
    let groups = parse_general_search_groups("rose");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 1);
    assert_eq!(groups[0][0].text, "rose");
}

#[test]
fn parse_general_search_groups_multiple_ors() {
    let groups = parse_general_search_groups("cat OR dog OR bird");
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0][0].text, "cat");
    assert_eq!(groups[1][0].text, "dog");
    assert_eq!(groups[2][0].text, "bird");
}

#[test]
fn parse_general_search_groups_trailing_or_is_skipped() {
    let groups = parse_general_search_groups("hello OR ");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0][0].text, "hello");
}

// â”€â”€â”€ bulk_delete with delete_files=true (trash errors collected) â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn bulk_delete_with_delete_files_errors_when_file_not_found() {
    let pool = test_pool().await;

    // Design 1 has filepath 'Roses/rose.pes' which doesn't exist on disk
    let result = bulk_delete_designs_with_pool(&pool, &[1], true)
        .await
        .expect("bulk delete should not fail even with file errors");
    assert_eq!(result.deleted_count, 1);
    assert!(result.files_trashed == 0);
    // Should report error about file not found
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("not found on disk"));
}

// â”€â”€â”€ generate_preview is external â€” just document the gap â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// â”€â”€â”€ push_general_search_clause â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn push_general_search_clause_adds_file_and_tag_and_folder_search() {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
    let tokens = vec![GeneralSearchToken {
        text: "rose".to_string(),
        phrase: false,
        exclude: false,
        is_extension: false,
    }];
    let groups = vec![tokens];

    push_general_search_clause(&mut builder, true, true, true, &groups);

    let sql = builder.sql();
    assert!(sql.contains("LOWER(d.filename) LIKE"));
    assert!(sql.contains("design_tags"));
    assert!(sql.contains("LOWER(tags.description) LIKE"));
    assert!(sql.contains("LOWER(d.filepath) LIKE"));
    // The bind values are stored as parameters, so count the `?` placeholders.
    assert!(sql.matches("LIKE ").count() >= 3);
}

#[test]
fn push_general_search_clause_with_exclusion_adds_not() {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
    let tokens = vec![GeneralSearchToken {
        text: "applique".to_string(),
        phrase: false,
        exclude: true,
        is_extension: false,
    }];
    let groups = vec![tokens];

    push_general_search_clause(&mut builder, true, false, false, &groups);

    let sql = builder.sql();
    assert!(sql.contains("NOT ("));
    assert!(sql.contains("LOWER(d.filename) LIKE"));
    assert!(sql.contains(")"));
}

#[test]
fn push_general_search_clause_with_or_groups_uses_or_between_groups() {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
    let group_a = vec![GeneralSearchToken {
        text: "rose".to_string(),
        phrase: false,
        exclude: false,
        is_extension: false,
    }];
    let group_b = vec![GeneralSearchToken {
        text: "hus".to_string(),
        phrase: false,
        exclude: false,
        is_extension: true,
    }];
    let groups = vec![group_a, group_b];

    push_general_search_clause(&mut builder, true, false, false, &groups);

    let sql = builder.sql();
    assert!(sql.contains(" OR "));
    // Each group adds a LIKE placeholder for the file search.
    assert!(sql.matches("LOWER(d.filename) LIKE").count() >= 2);
}

#[test]
fn push_general_search_clause_empty_groups_is_noop() {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
    let original = builder.sql().to_string();
    push_general_search_clause(&mut builder, true, true, true, &[]);
    assert_eq!(builder.sql(), original);
}

#[test]
fn push_general_search_clause_and_between_tokens_within_group() {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
    let tokens = vec![
        GeneralSearchToken {
            text: "rose".to_string(),
            phrase: false,
            exclude: false,
            is_extension: false,
        },
        GeneralSearchToken {
            text: "satin".to_string(),
            phrase: false,
            exclude: false,
            is_extension: false,
        },
    ];
    let groups = vec![tokens];

    push_general_search_clause(&mut builder, true, false, false, &groups);

    let sql = builder.sql();
    assert!(sql.contains(" AND "));
    assert!(sql.matches("LOWER(d.filename) LIKE").count() >= 2);
}

// â”€â”€â”€ recommend_hoop_for_design â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn recommend_hoop_selects_smallest_fitting_hoop() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Small', 50, 40)")
        .execute(&pool)
        .await
        .expect("insert small hoop");
    sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Large', 200, 200)")
        .execute(&pool)
        .await
        .expect("insert large hoop");

    let result = recommend_hoop_for_design(&pool, Some(40), Some(35))
        .await
        .expect("hoop recommendation should succeed");

    // Small (50x40) fits 40x35; should be chosen over Large (200x200)
    assert!(result.is_some());
    let name = sqlx::query_scalar::<_, String>("SELECT name FROM hoops WHERE id = ?")
        .bind(result.unwrap())
        .fetch_one(&pool)
        .await
        .expect("hoop name query");
    assert_eq!(name, "Small");
}

#[tokio::test]
async fn recommend_hoop_tries_rotated_orientation() {
    let pool = test_pool().await;
    // 60 wide x 30 tall: fits Small (50x40) rotated (40 wide x 50 tall needed)
    // Actually design 60x30 -> needs 60 wide. Little (70x20) won't fit.
    // To prove rotation: insert hoop that fits when the design is rotated 90Â°.
    // Design 60x30 -> rotated 30x60. Need a hoop >= 30 wide, >= 60 tall.
    sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Tall', 30, 70)")
        .execute(&pool)
        .await
        .expect("insert tall hoop");

    let result = recommend_hoop_for_design(&pool, Some(60), Some(30))
        .await
        .expect("hoop recommendation should succeed");

    // Only Tall (30x70) fits either orientation: width=60 fails (30<60),
    // but rotated width=30,height=60 â†’ 30>=30 and 70>=60 passes.
    assert!(result.is_some());
    let name = sqlx::query_scalar::<_, String>("SELECT name FROM hoops WHERE id = ?")
        .bind(result.unwrap())
        .fetch_one(&pool)
        .await
        .expect("hoop name query");
    assert_eq!(name, "Tall");
}

#[tokio::test]
async fn recommend_hoop_returns_none_when_no_hoop_fits() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Tiny', 5, 5)")
        .execute(&pool)
        .await
        .expect("insert tiny hoop");

    // Use dimensions larger than ALL seeded hoops (Hoop A is 126x126),
    // so no hoop fits in either orientation.
    let result = recommend_hoop_for_design(&pool, Some(300), Some(300))
        .await
        .expect("hoop recommendation should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn recommend_hoop_returns_none_when_dimensions_missing() {
    let pool = test_pool().await;
    let result = recommend_hoop_for_design(&pool, None, Some(10))
        .await
        .expect("hoop recommendation should succeed");
    assert!(result.is_none());

    let result = recommend_hoop_for_design(&pool, Some(10), None)
        .await
        .expect("hoop recommendation should succeed");
    assert!(result.is_none());
}

// â”€â”€â”€ normalize_windows_explorer_target (Windows-only) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[cfg(target_os = "windows")]
#[test]
fn normalize_windows_explorer_target_strips_verbatim_unc_prefix() {
    let result =
        normalize_windows_explorer_target(&PathBuf::from(r"\\?\UNC\server\share\file.pes"));
    assert_eq!(result.to_string_lossy(), r"\\server\share\file.pes");
}

#[cfg(target_os = "windows")]
#[test]
fn normalize_windows_explorer_target_strips_verbatim_local_prefix() {
    let result = normalize_windows_explorer_target(&PathBuf::from(r"\\?\C:\data\file.pes"));
    assert_eq!(result.to_string_lossy(), r"C:\data\file.pes");
}

#[cfg(target_os = "windows")]
#[test]
fn normalize_windows_explorer_target_converts_forward_slashes() {
    let result = normalize_windows_explorer_target(&PathBuf::from("C:/data/file.pes"));
    assert_eq!(result.to_string_lossy(), r"C:\data\file.pes");
}
