// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory db");

    sqlx::query(
        r#"
        CREATE TABLE tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            description VARCHAR(255) NOT NULL UNIQUE,
            tag_group VARCHAR(20),
            is_system BOOLEAN NOT NULL DEFAULT 0
        );

        CREATE TABLE tag_synonyms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword VARCHAR(100) NOT NULL COLLATE NOCASE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            CONSTRAINT uq_keyword_tag UNIQUE(keyword, tag_id)
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("schema");

    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (1, 'Animals', 'image'), (2, 'Cats', 'image'), (3, 'In The Hoop', 'stitching')")
        .execute(&pool)
        .await
        .expect("seed tags");

    pool
}

#[tokio::test]
async fn test_add_and_list_synonyms() {
    let pool = setup_test_db().await;

    let kw = add_tag_synonyms(&pool, 1, "frog, bear\nelephant")
        .await
        .expect("add synonyms");
    assert_eq!(kw.len(), 3);
    assert_eq!(kw[0].keyword, "bear");
    assert_eq!(kw[1].keyword, "elephant");
    assert_eq!(kw[2].keyword, "frog");

    let list = list_tag_synonyms(&pool).await.expect("list synonyms");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].tag_description, "Animals");

    let map = get_synonym_lookup_map(&pool).await.expect("lookup map");
    assert_eq!(map.get("frog"), Some(&vec!["Animals".to_string()]));
    assert_eq!(map.get("bear"), Some(&vec!["Animals".to_string()]));
}

#[tokio::test]
async fn test_list_grouped_synonyms() {
    let pool = setup_test_db().await;

    add_tag_synonyms(&pool, 1, "frog, toad")
        .await
        .expect("add 1");
    add_tag_synonyms(&pool, 3, "ith").await.expect("add 3");

    let groups = list_tag_synonyms_grouped(&pool)
        .await
        .expect("list grouped");
    assert_eq!(groups.len(), 3); // 3 tags in DB: Animals, Cats, In The Hoop

    let animals = groups.iter().find(|g| g.tag_id == 1).unwrap();
    assert_eq!(animals.keywords.len(), 2);

    let cats = groups.iter().find(|g| g.tag_id == 2).unwrap();
    assert_eq!(cats.keywords.len(), 0);

    let ith = groups.iter().find(|g| g.tag_id == 3).unwrap();
    assert_eq!(ith.keywords.len(), 1);
    assert_eq!(ith.keywords[0].keyword, "ith");
}

#[tokio::test]
async fn test_delete_synonym() {
    let pool = setup_test_db().await;

    let kw = add_tag_synonyms(&pool, 1, "frog, toad").await.expect("add");
    assert_eq!(kw.len(), 2);

    delete_tag_synonym(&pool, kw[0].id)
        .await
        .expect("delete single");

    let list = list_tag_synonyms(&pool).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].keyword, "toad");

    delete_all_tag_synonyms_for_tag(&pool, 1)
        .await
        .expect("delete all");
    let list_after = list_tag_synonyms(&pool).await.expect("list empty");
    assert_eq!(list_after.len(), 0);
}
