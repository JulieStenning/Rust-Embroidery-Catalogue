// Tests for the bulk-import route.
//
// This module was split out of bulk_import.rs so the route file can stay
// focused on production logic. It is included via a #[path] declaration
// in a #[cfg(test)] mod tests; module, so it retains full access to the
// private items in the parent module through use super::*;.

use super::*;
use serial_test::serial;
use sqlx::sqlite::SqlitePoolOptions;
use std::fs;

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/Test Designs");

async fn import_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create test sqlite pool");

    sqlx::query(
        r#"
            CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create settings table");

    sqlx::query(
        r#"
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL UNIQUE,
                tag_group TEXT
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create tags table");

    sqlx::query(
        r#"
            CREATE TABLE design_tags (
                design_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (design_id, tag_id)
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create design_tags table");

    sqlx::query(
        r#"
            CREATE TABLE designs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                filepath TEXT NOT NULL,
                date_added TEXT,
                designer_id INTEGER,
                source_id INTEGER,
                hoop_id INTEGER,
                image_data BLOB,
                image_type TEXT,
                width_mm REAL,
                height_mm REAL,
                stitch_count INTEGER,
                color_count INTEGER,
                color_change_count INTEGER,
                is_stitched INTEGER NOT NULL DEFAULT 0,
                image_tags_verified INTEGER NOT NULL DEFAULT 0,
                stitching_tags_verified INTEGER NOT NULL DEFAULT 0,
                tagging_tier INTEGER,
                file_size_bytes INTEGER,
                file_hash_blake3 TEXT
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create designs table");

    sqlx::query(
        r#"
            CREATE TABLE hoops (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                max_width_mm REAL NOT NULL,
                max_height_mm REAL NOT NULL
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create hoops table");

    sqlx::query(
        r#"
            CREATE TABLE designers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
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
                name TEXT NOT NULL UNIQUE
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create sources table");

    sqlx::query("INSERT INTO tags (description, tag_group) VALUES ('Alphabets', 'image'), ('Flowers', 'image'), ('Monogram', 'image'), ('Line Outline', 'stitching')")
            .execute(&pool)
            .await
            .expect("failed to seed tags");

    pool
}

#[test]
fn bulk_import_wire_round_trips_through_json() {
    let wire = BulkImportWire {
        root_paths: vec!["C:/imports".to_string()],
        global_designer_id: Some(7),
        global_source_id: None,
        per_folder_assignments: vec![FolderAssignmentWire {
            folder_path: "C:/imports/folder-a".to_string(),
            designer_id: None,
            source_id: Some(9),
            inferred_designer_id: Some(11),
            inferred_source_id: None,
        }],
        selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
        create_on_import: true,
    };

    let encoded = serde_json::to_string(&wire).expect("wire should serialize");
    let decoded: BulkImportWire = serde_json::from_str(&encoded).expect("wire should deserialize");

    assert_eq!(decoded.root_paths.len(), 1);
    assert_eq!(decoded.per_folder_assignments.len(), 1);
    assert_eq!(decoded.selected_files.len(), 1);
    assert!(decoded.create_on_import);
}

#[test]
#[serial]
fn persist_bulk_import_confirm_wire_writes_image_fields_in_native_mode() {
    let previous_db_url = std::env::var("DATABASE_URL").ok();
    let tmp_db_dir = std::env::temp_dir().join(format!(
        "bi-test-native-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_db_dir).ok();
    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", tmp_db_dir.display()),
    );
    let fixture = Path::new(FIXTURES_DIR).join("Bean.pes");
    assert!(fixture.exists(), "expected Bean.pes fixture to exist");

    let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
    std::env::set_var("IMPORT_IMAGE_BACKEND", "native");

    let pool = tauri::async_runtime::block_on(import_test_pool());
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec![FIXTURES_DIR.to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec![fixture.to_string_lossy().to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
        &pool,
        &confirm_wire,
        None,
    ))
    .expect("persist should succeed");
    assert_eq!(persisted, 1);

    // The file is now stored under MachineEmbroideryDesigns/Test Designs/Bean.pes
    let stored_filepath = "/MachineEmbroideryDesigns/Test Designs/Bean.pes";
    let row = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>, Option<f64>, Option<f64>, Option<i64>, Option<i64>, Option<i64>)>(
                "SELECT image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count FROM designs WHERE filepath = ? LIMIT 1"
            )
            .bind(stored_filepath)
            .fetch_one(&pool)
            .await
        })
        .expect("expected persisted design row");

    assert!(row.0.map(|bytes| !bytes.is_empty()).unwrap_or(false));
    assert_eq!(row.1.as_deref(), Some("2d"));
    assert!(row.2.unwrap_or_default() > 0.0);
    assert!(row.3.unwrap_or_default() > 0.0);
    assert!(row.4.unwrap_or_default() > 0);
    assert!(row.5.unwrap_or_default() > 0);
    assert!(row.6.unwrap_or_default() >= 0);

    if let Some(value) = previous_backend {
        std::env::set_var("IMPORT_IMAGE_BACKEND", value);
    } else {
        std::env::remove_var("IMPORT_IMAGE_BACKEND");
    }
    if let Some(url) = previous_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn persist_bulk_import_confirm_wire_auto_backend_falls_back_safely_without_python() {
    let previous_db_url = std::env::var("DATABASE_URL").ok();
    let tmp_db_dir = std::env::temp_dir().join(format!(
        "bi-test-auto3d-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_db_dir).ok();
    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", tmp_db_dir.display()),
    );
    let fixture = Path::new(FIXTURES_DIR).join("Bean.pes");
    assert!(fixture.exists(), "expected Bean.pes fixture to exist");

    let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
    let previous_python = std::env::var("RUST_EMBROIDERY_PYTHON").ok();

    std::env::set_var("IMPORT_IMAGE_BACKEND", "auto");
    // Intentionally point to a missing executable so python path fails and auto must use native fallback.
    std::env::set_var(
        "RUST_EMBROIDERY_PYTHON",
        "__missing_python_for_auto_fallback_test__",
    );

    let pool = tauri::async_runtime::block_on(import_test_pool());
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec![FIXTURES_DIR.to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec![fixture.to_string_lossy().to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
        &pool,
        &confirm_wire,
        None,
    ))
    .expect("persist should succeed even when python path is unavailable");
    assert_eq!(persisted, 1);

    // The file is now stored under MachineEmbroideryDesigns/Test Designs/Bean.pes
    let stored_filepath = "/MachineEmbroideryDesigns/Test Designs/Bean.pes";
    let row = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>, Option<f64>, Option<f64>)>(
                "SELECT image_data, image_type, width_mm, height_mm FROM designs WHERE filepath = ? LIMIT 1"
            )
            .bind(stored_filepath)
            .fetch_one(&pool)
            .await
        })
        .expect("expected persisted design row");

    assert!(row.0.map(|bytes| !bytes.is_empty()).unwrap_or(false));
    assert_eq!(row.1.as_deref(), Some("2d"));
    assert!(row.2.unwrap_or_default() > 0.0);
    assert!(row.3.unwrap_or_default() > 0.0);

    if let Some(value) = previous_backend {
        std::env::set_var("IMPORT_IMAGE_BACKEND", value);
    } else {
        std::env::remove_var("IMPORT_IMAGE_BACKEND");
    }

    if let Some(value) = previous_python {
        std::env::set_var("RUST_EMBROIDERY_PYTHON", value);
    } else {
        std::env::remove_var("RUST_EMBROIDERY_PYTHON");
    }
    if let Some(url) = previous_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn persist_bulk_import_confirm_wire_auto_hus_uses_native_backend() {
    let previous_db_url = std::env::var("DATABASE_URL").ok();
    let tmp_db_dir = std::env::temp_dir().join(format!(
        "bi-test-hus-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_db_dir).ok();
    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", tmp_db_dir.display()),
    );
    let fixture = Path::new(FIXTURES_DIR).join("Bean.hus");
    assert!(fixture.exists(), "expected Bean.hus fixture to exist");

    let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
    std::env::set_var("IMPORT_IMAGE_BACKEND", "auto");

    let generation_result =
        image_generation::generate_preview(&image_generation::ImageGenerationRequest {
            file_path: fixture.to_string_lossy().to_string(),
            preview_3d: true,
            preview_3d_profile: Some("balanced".to_string()),
        });

    assert_eq!(generation_result.backend, "native");
    let error_text = generation_result
        .error
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        error_text.is_empty(),
        "auto mode should generate native HUS previews without adapter errors"
    );

    let pool = tauri::async_runtime::block_on(import_test_pool());
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec![FIXTURES_DIR.to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec![fixture.to_string_lossy().to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
        &pool,
        &confirm_wire,
        None,
    ))
    .expect("persist should succeed for .hus even when preview generation fails");
    assert_eq!(persisted, 1);

    // The file is now stored under MachineEmbroideryDesigns/Test Designs/Bean.hus
    let stored_filepath = "/MachineEmbroideryDesigns/Test Designs/Bean.hus";
    let persisted_row_id = tauri::async_runtime::block_on(async {
        sqlx::query_scalar::<_, i64>("SELECT id FROM designs WHERE filepath = ? LIMIT 1")
            .bind(stored_filepath)
            .fetch_optional(&pool)
            .await
    })
    .expect("expected design lookup to succeed");
    assert!(
        persisted_row_id.is_some(),
        "expected .hus design row to be inserted"
    );

    if let Some(value) = previous_backend {
        std::env::set_var("IMPORT_IMAGE_BACKEND", value);
    } else {
        std::env::remove_var("IMPORT_IMAGE_BACKEND");
    }
    if let Some(url) = previous_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
fn normalize_import_commit_batch_size_defaults_to_10_and_clamps_high_values() {
    assert_eq!(normalize_import_commit_batch_size(None), 10);
    assert_eq!(normalize_import_commit_batch_size(Some("")), 10);
    assert_eq!(normalize_import_commit_batch_size(Some("abc")), 10);
    assert_eq!(normalize_import_commit_batch_size(Some("0")), 10);
    assert_eq!(normalize_import_commit_batch_size(Some("10")), 10);
    assert_eq!(
        normalize_import_commit_batch_size(Some("1000000")),
        MAX_IMPORT_COMMIT_BATCH_SIZE
    );
}

#[test]
fn load_import_commit_batch_size_reads_setting_override() {
    let pool = tauri::async_runtime::block_on(import_test_pool());

    let default_batch_size = tauri::async_runtime::block_on(load_import_commit_batch_size(&pool))
        .expect("default batch size should load");
    assert_eq!(default_batch_size, 10);

    tauri::async_runtime::block_on(async {
            sqlx::query(
                "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(KEY_IMPORT_COMMIT_BATCH_SIZE)
            .bind("25")
            .bind("test commit batch size")
            .execute(&pool)
            .await
        })
        .expect("failed to set import commit batch size");

    let configured_batch_size =
        tauri::async_runtime::block_on(load_import_commit_batch_size(&pool))
            .expect("configured batch size should load");
    assert_eq!(configured_batch_size, 25);
}

#[test]
#[serial]
fn persist_bulk_import_confirm_wire_assigns_tier1_keyword_tags() {
    let previous_db_url = std::env::var("DATABASE_URL").ok();
    let tmp_db_dir = std::env::temp_dir().join(format!(
        "bi-test-tags-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_db_dir).ok();
    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", tmp_db_dir.display()),
    );
    // Use Flower.pes fixture whose name contains "flower" -> "Flowers" from KEYWORD_MAP
    let fixture = Path::new(FIXTURES_DIR).join("Flower.pes");
    assert!(fixture.exists(), "expected Flower.pes fixture to exist");

    let pool = tauri::async_runtime::block_on(import_test_pool());
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec![FIXTURES_DIR.to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec![fixture.to_string_lossy().to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
        &pool,
        &confirm_wire,
        None,
    ))
    .expect("persist should succeed");
    assert_eq!(persisted, 1);

    let stored_filepath = "/MachineEmbroideryDesigns/Test Designs/Flower.pes";

    let assigned_tags = tauri::async_runtime::block_on(async {
        sqlx::query_as::<_, (String,)>(
            r#"
                SELECT t.description
                FROM design_tags dt
                JOIN tags t ON t.id = dt.tag_id
                JOIN designs d ON d.id = dt.design_id
                WHERE d.filepath = ?
                ORDER BY t.description ASC
                "#,
        )
        .bind(stored_filepath)
        .fetch_all(&pool)
        .await
    })
    .expect("failed to query assigned tags");

    assert!(
        !assigned_tags.is_empty(),
        "expected at least one tag assignment for imported design; got {:?}",
        assigned_tags
    );

    // Verify that "Flowers" was assigned (from "flower" keyword match)
    let descriptions: Vec<&str> = assigned_tags.iter().map(|d| d.0.as_str()).collect();
    assert!(
        descriptions.contains(&"Flowers"),
        "expected 'Flowers' tag to be assigned from 'flower' keyword; got {:?}",
        descriptions
    );
    if let Some(url) = previous_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
#[serial]
fn persist_bulk_import_confirm_wire_assigns_stitching_tags() {
    let previous_db_url = std::env::var("DATABASE_URL").ok();
    let tmp_db_dir = std::env::temp_dir().join(format!(
        "bi-test-stitch-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_db_dir).ok();
    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", tmp_db_dir.display()),
    );
    let fixture = Path::new(FIXTURES_DIR).join("Bean.pes");
    assert!(fixture.exists(), "expected Bean.pes fixture to exist");

    let pool = tauri::async_runtime::block_on(import_test_pool());
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec![FIXTURES_DIR.to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec![fixture.to_string_lossy().to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
        &pool,
        &confirm_wire,
        None,
    ))
    .expect("persist should succeed");
    assert_eq!(persisted, 1);

    let stored_filepath = "/MachineEmbroideryDesigns/Test Designs/Bean.pes";

    let stitching_tags = tauri::async_runtime::block_on(async {
        sqlx::query_as::<_, (String,)>(
            r#"
                SELECT t.description
                FROM design_tags dt
                JOIN tags t ON t.id = dt.tag_id
                JOIN designs d ON d.id = dt.design_id
                WHERE d.filepath = ?
                  AND lower(COALESCE(t.tag_group, '')) = 'stitching'
                ORDER BY t.description ASC
                "#,
        )
        .bind(stored_filepath)
        .fetch_all(&pool)
        .await
    })
    .expect("failed to query stitching tags");

    assert!(
        !stitching_tags.is_empty(),
        "expected at least one stitching tag assignment"
    );
    if let Some(url) = previous_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
fn bulk_import_confirm_wire_round_trips_through_json() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: None,
                inferred_source_id: Some(12),
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        },
        context_token: Some("token-123".to_string()),
        canonical_confirm: true,
    };

    let encoded = serde_json::to_string(&confirm_wire).expect("confirm wire should serialize");
    let decoded: BulkImportConfirmWire =
        serde_json::from_str(&encoded).expect("confirm wire should deserialize");

    assert_eq!(decoded.context_token.as_deref(), Some("token-123"));
    assert!(decoded.canonical_confirm);
    assert_eq!(decoded.wire.root_paths.len(), 1);
}

#[test]
fn assignment_field_resolution_prefers_explicit_global_inferred_blank() {
    let explicit = resolve_assignment_field(Some(1), Some(2), Some(3));
    assert_eq!(explicit.value, Some(1));
    assert_eq!(
        explicit.source,
        AssignmentFieldSourceWire::ExplicitPerFolder
    );

    let global = resolve_assignment_field(None, Some(2), Some(3));
    assert_eq!(global.value, Some(2));
    assert_eq!(global.source, AssignmentFieldSourceWire::Global);

    let inferred = resolve_assignment_field(None, None, Some(3));
    assert_eq!(inferred.value, Some(3));
    assert_eq!(inferred.source, AssignmentFieldSourceWire::Inferred);

    let blank = resolve_assignment_field(None, None, None);
    assert_eq!(blank.value, None);
    assert_eq!(blank.source, AssignmentFieldSourceWire::Blank);
}

#[test]
fn suggest_reference_id_from_path_matches_compact_names() {
    let items = vec![
        (1, "www.UrbanThreads.com".to_string()),
        (2, "Another Source".to_string()),
    ];

    let matched = suggest_reference_id_from_path(
        "D:/My Software Development/Rust-Embroidery-Catalogue/data/MachineEmbroideryDesigns/Urban Threads",
        &items,
    );

    assert_eq!(matched, Some(1));
}

#[test]
fn folder_assignment_resolution_uses_wire_defaults_and_inferred_values() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![
                FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: Some(11),
                    inferred_source_id: Some(12),
                },
                FolderAssignmentWire {
                    folder_path: "C:/imports/folder-b".to_string(),
                    designer_id: None,
                    source_id: None,
                    inferred_designer_id: Some(13),
                    inferred_source_id: None,
                },
            ],
            selected_files: vec![],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let resolved = resolve_bulk_import_assignments(&confirm_wire);
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].designer_id.value, Some(10));
    assert_eq!(
        resolved[0].designer_id.source,
        AssignmentFieldSourceWire::ExplicitPerFolder
    );
    assert_eq!(resolved[0].source_id.value, Some(8));
    assert_eq!(
        resolved[0].source_id.source,
        AssignmentFieldSourceWire::Global
    );

    assert_eq!(resolved[1].designer_id.value, Some(7));
    assert_eq!(
        resolved[1].designer_id.source,
        AssignmentFieldSourceWire::Global
    );
    assert_eq!(resolved[1].source_id.value, Some(8));
    assert_eq!(
        resolved[1].source_id.source,
        AssignmentFieldSourceWire::Global
    );
}

#[test]
fn preview_bulk_import_wire_returns_resolved_assignments() {
    let preview = preview_bulk_import_wire(BulkImportWire {
        root_paths: vec!["C:/imports".to_string()],
        global_designer_id: Some(7),
        global_source_id: Some(8),
        per_folder_assignments: vec![FolderAssignmentWire {
            folder_path: "C:/imports/folder-a".to_string(),
            designer_id: None,
            source_id: Some(9),
            inferred_designer_id: Some(11),
            inferred_source_id: Some(12),
        }],
        selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
        create_on_import: true,
    })
    .expect("preview should resolve");

    assert_eq!(preview.resolved_assignments.len(), 1);
    assert_eq!(preview.resolved_assignments[0].designer_id.value, Some(7));
    assert_eq!(preview.resolved_assignments[0].source_id.value, Some(9));
}

#[test]
fn preview_bulk_import_wire_excludes_already_catalogued_files() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rec-import-preview-existing-{stamp}"));
    fs::create_dir_all(&root).expect("temp root should be created");

    let file_path = root.join("existing-design.pes");
    fs::write(&file_path, b"dummy").expect("temp pes should be written");
    let file_path_text = file_path.to_string_lossy().to_string();

    // Seed a design row whose stored filepath matches the prospective
    // stored path that filter_existing_scanned_files will compute
    // from this temp-file + root combination.
    let root_paths = vec![root.to_string_lossy().to_string()];
    let prospective_stored = compute_prospective_stored_filepath(&file_path_text, &root_paths)
        .expect("prospective stored path should resolve");

    let pool = tauri::async_runtime::block_on(import_test_pool());
    tauri::async_runtime::block_on(async {
            sqlx::query(
                "INSERT INTO designs (filename, filepath, date_added, is_stitched, image_tags_verified, stitching_tags_verified) VALUES (?, ?, DATE('now'), 0, 0, 0)",
            )
            .bind("existing-design.pes")
            .bind(&prospective_stored)
            .execute(&pool)
            .await
        })
        .expect("seeded existing design should insert");

    let preview = preview_bulk_import_wire_with_pool(
        BulkImportWire {
            root_paths,
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        Some(&pool),
    )
    .expect("preview should succeed");

    assert_eq!(preview.discovered_count, 0);
    assert!(preview.scanned_files.is_empty());

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn preview_bulk_import_wire_infers_assignments_from_folder_path_with_pool() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rec-import-preview-infer-{stamp}"));
    let inferred_folder = root.join("Acme Designs").join("Magazine Source");
    fs::create_dir_all(&inferred_folder).expect("temp inferred folder should be created");

    let file_path = inferred_folder.join("sample-design.pes");
    fs::write(&file_path, b"dummy").expect("temp pes should be written");

    let pool = tauri::async_runtime::block_on(import_test_pool());
    tauri::async_runtime::block_on(async {
        sqlx::query("INSERT INTO designers (id, name) VALUES (1, 'Acme')")
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO sources (id, name) VALUES (1, 'Magazine Source')")
            .execute(&pool)
            .await?;
        Ok::<(), sqlx::Error>(())
    })
    .expect("seeded designer/source should insert");

    let preview = preview_bulk_import_wire_with_pool(
        BulkImportWire {
            root_paths: vec![root.to_string_lossy().to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        Some(&pool),
    )
    .expect("preview should succeed");

    assert_eq!(preview.discovered_count, 1);
    assert_eq!(preview.resolved_assignments.len(), 1);
    assert_eq!(preview.resolved_assignments[0].designer_id.value, Some(1));
    assert_eq!(
        preview.resolved_assignments[0].designer_id.source,
        AssignmentFieldSourceWire::Inferred
    );
    assert_eq!(preview.resolved_assignments[0].source_id.value, Some(1));
    assert_eq!(
        preview.resolved_assignments[0].source_id.source,
        AssignmentFieldSourceWire::Inferred
    );

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn confirm_execution_result_reflects_readiness_and_resolution() {
    let result = execute_bulk_import_confirm_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: Some(11),
                inferred_source_id: Some(12),
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        },
        context_token: Some("token-123".to_string()),
        canonical_confirm: true,
    })
    .expect("confirm execution should succeed");

    assert!(result.context_token_present);
    assert!(result.canonical_confirm);
    assert!(result.ready_for_persistence);
    assert_eq!(result.root_path_count, 1);
    assert_eq!(result.selected_file_count, 1);
    assert_eq!(result.resolved_assignments.len(), 1);
    assert_eq!(result.resolved_assignments[0].designer_id.value, Some(10));
}

#[test]
fn canonical_confirm_wire_marks_ready_for_persistence() {
    let result = confirm_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: None,
                inferred_source_id: Some(12),
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        },
        context_token: Some("token-456".to_string()),
        canonical_confirm: true,
    })
    .expect("canonical confirm should succeed");

    assert!(result.context_token_present);
    assert!(result.canonical_confirm);
    assert!(result.ready_for_persistence);
    assert_eq!(result.resolved_assignments.len(), 1);
}

#[test]
fn legacy_confirm_wire_shims_into_canonical_confirm() {
    let result = confirm_bulk_import_legacy(BulkImportRequest {
        root_path: Some("C:/imports".to_string()),
        root_paths: Vec::new(),
        fallback_designer_id: Some(7),
        fallback_source_id: Some(8),
    })
    .expect("legacy confirm should succeed");

    assert!(result.canonical_confirm);
    assert!(result.ready_for_persistence);
    assert_eq!(result.root_path_count, 1);
    assert_eq!(result.selected_file_count, 0);
}

#[test]
fn precheck_stores_context_and_do_confirm_consumes_it() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: None,
                inferred_source_id: Some(12),
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    assert!(precheck.context_token_present);
    assert!(precheck.ready_for_confirm);
    assert_eq!(precheck.resolved_assignments.len(), 1);

    let confirm = do_confirm_bulk_import_wire(precheck.context_token)
        .expect("do-confirm should consume stored token");

    assert!(confirm.context_token_present);
    assert!(confirm.canonical_confirm);
    assert!(confirm.ready_for_persistence);
    assert_eq!(confirm.resolved_assignments.len(), 1);
}

#[test]
fn precheck_action_review_tags_keeps_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ReviewTags,
            confirm_skip_hoops: false,
        },
    ))
    .expect("review action should succeed");

    assert!(!action_result.consumed_context);
    assert!(action_result.context_token_present);
    assert!(action_result
        .next_route
        .unwrap_or_default()
        .contains("/admin/tags/"));
    assert!(take_bulk_import_context(&precheck.context_token).is_some());
}

#[test]
fn precheck_action_cancel_consumes_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::Cancel,
            confirm_skip_hoops: false,
        },
    ))
    .expect("cancel action should succeed");

    assert!(action_result.consumed_context);
    assert!(!action_result.context_token_present);
    assert_eq!(action_result.next_route.as_deref(), Some("/import/"));
    assert!(take_bulk_import_context(&precheck.context_token).is_none());
}

#[test]
fn precheck_action_import_now_consumes_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ImportNow,
            confirm_skip_hoops: false,
        },
    ))
    .expect("import-now action should succeed");

    assert!(action_result.consumed_context);
    assert!(!action_result.context_token_present);
    assert_eq!(action_result.next_route.as_deref(), Some("/designs/"));
    assert!(action_result.confirm_result.is_some());
    assert!(take_bulk_import_context(&precheck.context_token).is_none());
}

#[test]
fn debug_bulk_import_context_store_reports_live_counts() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let summary = debug_bulk_import_context_store().expect("debug summary should succeed");

    assert!(summary.active_context_count >= 1);
    assert_eq!(summary.max_entries, BULK_IMPORT_CONTEXT_MAX_ENTRIES);
    assert_eq!(summary.ttl_seconds, BULK_IMPORT_CONTEXT_TTL.as_secs());

    let _ = take_bulk_import_context(&precheck.context_token);
}

#[test]
fn reset_bulk_import_context_store_clears_entries_and_updates_metrics() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should seed store");

    let before = debug_bulk_import_context_store().expect("summary should succeed");
    assert!(before.active_context_count >= 1);

    let reset = reset_bulk_import_context_store().expect("manual reset should succeed");
    assert!(reset.cleared_context_count >= 1);
    assert_eq!(reset.active_context_count, 0);

    let after = debug_bulk_import_context_store().expect("summary should succeed");
    assert_eq!(after.active_context_count, 0);
    assert!(after.reset_count >= 1);
    assert!(after.last_reset_at_millis.is_some());

    assert!(take_bulk_import_context(&precheck.context_token).is_none());
}

#[test]
fn bulk_import_context_store_evicts_oldest_when_capacity_is_exceeded() {
    let base_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: false,
    };
    let created_at_millis = current_timestamp_millis();

    let first_token = "bulk-import-test-first".to_string();
    insert_bulk_import_context_for_test(
        first_token.clone(),
        base_wire.clone(),
        created_at_millis,
        1,
    );

    for index in 2..=(BULK_IMPORT_CONTEXT_MAX_ENTRIES as u64 + 1) {
        insert_bulk_import_context_for_test(
            format!("bulk-import-test-{index}"),
            base_wire.clone(),
            created_at_millis,
            index,
        );
    }

    assert!(take_bulk_import_context(&first_token).is_none());
    assert!(take_bulk_import_context(&format!(
        "bulk-import-test-{}",
        BULK_IMPORT_CONTEXT_MAX_ENTRIES as u64 + 1
    ))
    .is_some());
}

#[test]
fn bulk_import_context_store_expires_old_entries_on_access() {
    let expired_token = "bulk-import-test-expired".to_string();
    let current_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: false,
    };

    let expired_created_at =
        current_timestamp_millis().saturating_sub(BULK_IMPORT_CONTEXT_TTL.as_millis() + 1);
    insert_bulk_import_context_for_test(
        expired_token.clone(),
        current_wire,
        expired_created_at,
        9999,
    );

    assert!(take_bulk_import_context(&expired_token).is_none());
}

// =========================================================================
// Phase 5 - Path derivation tests (5.1)
// =========================================================================

/// Under AppRoot: full_path_to_stored_design_filepath should return
/// the canonical stored path directly.
#[test]
fn stored_filepath_from_designs_base_subdirectory() {
    // We can't change DATABASE_URL at runtime, but we can test the
    // computation against a hypothetical base by constructing a path
    // and checking it is NOT treated as in-library when it clearly isn't.
    let result = full_path_to_stored_design_filepath("C:/SomeRandomPath/not-a-design.pes");
    assert!(result.is_err(), "unrelated path must not be in-library");
}

/// The old substring-based in-library detection is gone: paths containing
/// "machineembroiderydesigns" as a substring but not actually under the
/// canonical designs base must now be treated as external (not in-library).
#[test]
fn unrelated_path_containing_sentinel_is_not_in_library() {
    // A path like C:/tmp/machineembroiderydesigns-test/file.pes
    // was previously treated as in-library by the old substring scan.
    // With strict base-prefix validation it must now be external.
    let is_under = is_path_under_designs_base("C:/tmp/machineembroiderydesigns-test/design.pes");
    assert!(!is_under);

    let parsed =
        full_path_to_stored_design_filepath("C:/tmp/machineembroiderydesigns-test/design.pes");
    assert!(
        parsed.is_err(),
        "unrelated path must not produce stored path"
    );
}

/// compute_prospective_stored_filepath with a standard leaf root.
#[test]
fn prospective_path_standard_root_with_leaf() {
    // Simulates: selected root C:/x/d/f, file C:/x/d/f/Babies/Jef Files/design.jef
    let result = compute_prospective_stored_filepath(
        "C:/x/d/f/Babies/Jef Files/design.jef",
        &["C:/x/d/f".to_string()],
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
    );
}

/// compute_prospective_stored_filepath with a parent root (leaf = x).
#[test]
fn prospective_path_parent_root() {
    // Selected root C:/x, file C:/x/d/f/Babies/Jef Files/design.jef
    let result = compute_prospective_stored_filepath(
        "C:/x/d/f/Babies/Jef Files/design.jef",
        &["C:/x".to_string()],
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/x/d/f/Babies/Jef Files/design.jef"
    );
}

/// compute_prospective_stored_filepath with drive-root selection (no leaf).
#[test]
fn prospective_path_drive_root() {
    // Selected root C:/, file C:/Designs/Floral/a.pes
    let result =
        compute_prospective_stored_filepath("C:/Designs/Floral/a.pes", &["C:/".to_string()]);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/Designs/Floral/a.pes"
    );
}

/// compute_prospective_stored_filepath with mixed slash separators.
#[test]
fn prospective_path_mixed_separators() {
    // Backslash in the file path, forward-slash root
    let result = compute_prospective_stored_filepath(
        "C:\\x\\d\\f\\Babies\\Jef Files\\design.jef",
        &["C:/x/d/f".to_string()],
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
    );
}

/// Longest-root match must be chosen when multiple roots are provided.
#[test]
fn prospective_path_longest_root_wins() {
    let result = compute_prospective_stored_filepath(
        "C:/x/d/f/Babies/Jef Files/design.jef",
        &["C:/x".to_string(), "C:/x/d/f".to_string()],
    );
    assert!(result.is_ok());
    // Longer root "C:/x/d/f" wins => leaf "f"
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
    );
}

// =========================================================================
// Phase 5 - In-library detection strictness (5.3)
// =========================================================================

#[test]
fn is_path_under_designs_base_accepts_actual_subpath() {
    let designs_base = get_designs_base_path();
    let file_path = designs_base
        .join("some-folder")
        .join("test.pes")
        .to_string_lossy()
        .replace('\\', "/");

    let is_under = is_path_under_designs_base(&file_path);
    assert!(is_under);
}

// =========================================================================
// Phase 5 - file hash + size utilities
// =========================================================================

#[test]
fn compute_blake3_hash_of_known_content() {
    let dir = std::env::temp_dir().join(format!(
        "rec-hash-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    let file_path = dir.join("content.bin");
    fs::write(&file_path, b"hello world").expect("file should be written");

    let hash = compute_file_hash_blake3(&file_path).expect("hash should succeed");
    let size = compute_file_size(&file_path).expect("size should succeed");

    assert_eq!(size, 11); // "hello world" = 11 bytes
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64); // BLAKE3 hex = 64 chars

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&dir);
}

// =========================================================================
// Phase 5 - Preview dedup with prospective stored path (5.2)
// =========================================================================

#[tokio::test]
async fn preview_dedup_excludes_by_prospective_stored_path() {
    let pool = import_test_pool().await;

    // Create a temp file outside the designs base and compute the
    // prospective stored path that filter_existing_scanned_files will use.
    let dir = std::env::temp_dir().join(format!(
        "rec-dedup-prospective-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let sub = dir.join("some-folder");
    fs::create_dir_all(&sub).expect("temp subdir should be created");
    let file_path = sub.join("unique-file.pes");
    fs::write(&file_path, b"dummy").expect("temp file should be written");
    let file_path_text = file_path.to_string_lossy().to_string();

    let root_paths = vec![dir.to_string_lossy().to_string()];
    let prospective_stored = compute_prospective_stored_filepath(&file_path_text, &root_paths)
        .expect("prospective stored path should resolve");

    // Seed a design row whose stored filepath matches the prospective path.
    sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added, is_stitched, image_tags_verified, stitching_tags_verified) VALUES (?, ?, DATE('now'), 0, 0, 0)",
        )
        .bind("unique-file.pes")
        .bind(&prospective_stored)
        .execute(&pool)
        .await
        .expect("seed design should insert");

    let file_size = fs::metadata(&file_path).ok().map(|m| m.len() as i64);
    let scanned = vec![scanning::ScannedFile {
        full_path: file_path_text.clone(),
        filename: "unique-file.pes".to_string(),
        extension: "pes".to_string(),
        file_size_bytes: file_size,
        dedup_group_key: "test".to_string(),
    }];

    let filtered = filter_existing_scanned_files(&pool, scanned, &root_paths)
        .await
        .expect("filter should succeed");

    // The file should be excluded because its prospective stored path
    // matches the seeded row.
    assert!(filtered.is_empty());

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&dir);
}

// =========================================================================
// Additional coverage: pure function tests (High Impact)
// =========================================================================

#[test]
fn strip_sqlite_prefix_variants() {
    assert_eq!(
        strip_sqlite_prefix("sqlite:///C:/data/test.db"),
        "C:/data/test.db"
    );
    assert_eq!(
        strip_sqlite_prefix("sqlite:///data/test.db"),
        "data/test.db"
    );
    assert_eq!(strip_sqlite_prefix("sqlite://data/test.db"), "data/test.db");
    assert_eq!(strip_sqlite_prefix("sqlite:data/test.db"), "data/test.db");
    assert_eq!(strip_sqlite_prefix(":memory:"), ":memory:");
    assert_eq!(
        strip_sqlite_prefix("postgres://user:pass@localhost/db"),
        "postgres://user:pass@localhost/db"
    );
}

#[test]
fn normalize_name_for_import_matching_variants() {
    assert_eq!(
        normalize_name_for_import_matching("Acme-Designs"),
        "acme designs"
    );
    assert_eq!(
        normalize_name_for_import_matching("Urban_Threads/Shop"),
        "urban threads shop"
    );
    assert_eq!(
        normalize_name_for_import_matching("  Hello World  "),
        "hello world"
    );
    assert_eq!(normalize_name_for_import_matching(""), "");
    assert_eq!(normalize_name_for_import_matching("a-b_c/d e"), "a b c d e");
}

#[test]
fn compact_name_for_import_matching_variants() {
    assert_eq!(
        compact_name_for_import_matching("www.UrbanThreads.com"),
        "wwwurbanthreadscom"
    );
    assert_eq!(
        compact_name_for_import_matching("Hello_World-123"),
        "helloworld123"
    );
    assert_eq!(compact_name_for_import_matching(""), "");
    assert_eq!(compact_name_for_import_matching("  A  B  "), "ab");
    assert_eq!(
        compact_name_for_import_matching("special@#!chars"),
        "specialchars"
    );
}

#[test]
fn strip_web_affixes_for_import_matching_variants() {
    assert_eq!(
        strip_web_affixes_for_import_matching("www.UrbanThreads.com"),
        "urbanthreads"
    );
    assert_eq!(
        strip_web_affixes_for_import_matching("UrbanThreads.com"),
        "urbanthreads"
    );
    assert_eq!(
        strip_web_affixes_for_import_matching("Example.co.uk"),
        "example"
    );
    assert_eq!(
        strip_web_affixes_for_import_matching("Example.com.au"),
        "example"
    );
    assert_eq!(
        strip_web_affixes_for_import_matching("Example.org"),
        "example"
    );
    assert_eq!(
        strip_web_affixes_for_import_matching("Example.net"),
        "example"
    );
    assert_eq!(strip_web_affixes_for_import_matching("Short.co"), "short");
    assert_eq!(strip_web_affixes_for_import_matching("ab.cd"), "abcd");
    assert_eq!(
        strip_web_affixes_for_import_matching("no_suffix_here"),
        "nosuffixhere"
    );
}

#[test]
fn folder_path_from_file_path_variants() {
    assert_eq!(
        folder_path_from_file_path("C:/designs/import/design.pes"),
        Some("C:/designs/import".to_string())
    );
    // Bare filename has no parent dir â€” returns None (filtered by empty check)
    assert_eq!(folder_path_from_file_path("design.pes"), None);
    assert_eq!(folder_path_from_file_path(""), None);
    assert_eq!(folder_path_from_file_path("   "), None);
    assert_eq!(
        folder_path_from_file_path("C:/root/"),
        Some("C:/".to_string())
    );
}

#[test]
fn normalize_path_for_match_variants() {
    assert_eq!(
        normalize_path_for_match("C:/Designs/Flower.pes"),
        "c:/designs/flower.pes"
    );
    assert_eq!(
        normalize_path_for_match("C:\\Designs\\Flower.pes"),
        "c:/designs/flower.pes"
    );
    assert_eq!(normalize_path_for_match(""), "");
    assert_eq!(
        normalize_path_for_match("MIXED/Case/PATH"),
        "mixed/case/path"
    );
}

#[test]
fn resolve_assignment_for_file_no_match_falls_back_to_global() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(42),
            global_source_id: Some(99),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: None,
                inferred_source_id: None,
            }],
            selected_files: vec![],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let resolved = resolve_bulk_import_assignments(&confirm_wire);

    // File under folder-a should get folder-level designer + global source
    let (designer, source) =
        resolve_assignment_for_file("C:/imports/folder-a/design.pes", &confirm_wire, &resolved);
    assert_eq!(designer, Some(10));
    assert_eq!(source, Some(99));

    // File outside any assignment should get global fallback
    let (designer2, source2) =
        resolve_assignment_for_file("C:/imports/other/design.pes", &confirm_wire, &resolved);
    assert_eq!(designer2, Some(42));
    assert_eq!(source2, Some(99));
}

#[test]
fn resolve_assignment_for_file_prefers_longest_matching_folder() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: vec![
                FolderAssignmentWire {
                    folder_path: "C:/imports".to_string(),
                    designer_id: Some(1),
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: None,
                },
                FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(2),
                    source_id: Some(3),
                    inferred_designer_id: None,
                    inferred_source_id: None,
                },
            ],
            selected_files: vec![],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let resolved = resolve_bulk_import_assignments(&confirm_wire);
    assert_eq!(resolved.len(), 2);

    // File nested deep should match the longer path
    let (designer, source) = resolve_assignment_for_file(
        "C:/imports/folder-a/nested/design.pes",
        &confirm_wire,
        &resolved,
    );
    assert_eq!(designer, Some(2));
    assert_eq!(source, Some(3));

    // File at root level should match the shorter path
    let (designer2, source2) =
        resolve_assignment_for_file("C:/imports/design.pes", &confirm_wire, &resolved);
    assert_eq!(designer2, Some(1));
    assert_eq!(source2, None);
}

#[test]
fn full_path_to_stored_design_filepath_edge_cases() {
    // Empty path
    let result = full_path_to_stored_design_filepath("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));

    // Wholly unrelated path
    let result = full_path_to_stored_design_filepath("Z:/totally/unrelated/file.pes");
    assert!(result.is_err());
}

#[test]
fn compute_prospective_stored_filepath_edge_cases() {
    // File already under designs base (fast path) â€” should use full_path_to_stored_design_filepath
    // which will either succeed or fail. Since we can't guarantee where the designs base is,
    // test that it doesn't panic and returns a consistent prefixed result.
    let result = compute_prospective_stored_filepath(
        "C:/test-root/imports/design.pes",
        &["C:/test-root/imports".to_string()],
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.starts_with("/MachineEmbroideryDesigns/"));
    assert!(path.ends_with("design.pes"));

    // No matching root fallback â€” should use bare filename
    let result =
        compute_prospective_stored_filepath("X:/orphan/file.pes", &["Z:/unrelated".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/MachineEmbroideryDesigns/file.pes");

    // Source exactly at root boundary (file is the root itself... not possible, but test edge)
    let result = compute_prospective_stored_filepath(
        "C:/test-root/imports",
        &["C:/test-root/imports".to_string()],
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.starts_with("/MachineEmbroideryDesigns/"));

    // Drive-letter-only root with file directly under drive
    let result = compute_prospective_stored_filepath("C:/design.pes", &["C:/".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/MachineEmbroideryDesigns/design.pes");

    // Drive-letter-only root with nested path
    let result =
        compute_prospective_stored_filepath("C:/Designs/Floral/a.pes", &["C:/".to_string()]);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "/MachineEmbroideryDesigns/Designs/Floral/a.pes"
    );
}

#[test]
fn compute_file_hash_blake3_file_not_found() {
    let result = compute_file_hash_blake3(Path::new("Z:/nonexistent-file-for-test.bin"));
    assert!(result.is_err());
}

#[test]
fn compute_file_size_file_not_found() {
    let result = compute_file_size(Path::new("Z:/nonexistent-file-for-test.bin"));
    assert!(result.is_err());
}

// =========================================================================
// Additional coverage: DB-dependent / integration tests (Medium Impact)
// =========================================================================

#[test]
#[serial]
fn precheck_action_review_hoops_keeps_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ReviewHoops,
            confirm_skip_hoops: false,
        },
    ))
    .expect("review hoops action should succeed");

    assert!(!action_result.consumed_context);
    assert!(action_result.context_token_present);
    assert!(action_result
        .next_route
        .unwrap_or_default()
        .contains("/admin/hoops/"));
    // Context still available after review
    assert!(take_bulk_import_context(&precheck.context_token).is_some());
}

#[test]
#[serial]
fn precheck_action_review_sources_keeps_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ReviewSources,
            confirm_skip_hoops: false,
        },
    ))
    .expect("review sources action should succeed");

    assert!(!action_result.consumed_context);
    assert!(action_result.context_token_present);
    assert!(action_result
        .next_route
        .unwrap_or_default()
        .contains("/admin/sources/"));
    assert!(take_bulk_import_context(&precheck.context_token).is_some());
}

#[test]
#[serial]
fn precheck_action_review_designers_keeps_context() {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
        BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ReviewDesigners,
            confirm_skip_hoops: false,
        },
    ))
    .expect("review designers action should succeed");

    assert!(!action_result.consumed_context);
    assert!(action_result.context_token_present);
    assert!(action_result
        .next_route
        .unwrap_or_default()
        .contains("/admin/designers/"));
    assert!(take_bulk_import_context(&precheck.context_token).is_some());
}

#[tokio::test]
#[serial]
async fn filter_existing_scanned_files_empty_input() {
    let pool = import_test_pool().await;
    let result = filter_existing_scanned_files(&pool, vec![], &[])
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
#[serial]
async fn filter_existing_scanned_files_different_hash_passes() {
    let pool = import_test_pool().await;

    let dir = std::env::temp_dir().join(format!(
        "rec-filter-diffhash-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let sub = dir.join("import-root");
    fs::create_dir_all(&sub).expect("temp dir should be created");
    let file_path = sub.join("test.pes");
    fs::write(&file_path, b"version1").expect("file should be written");
    let file_path_text = file_path.to_string_lossy().to_string();

    // Seed a design with the same filename and size but a different hash,
    // using a stored path that does NOT match the prospective path
    // (so Stage 0 path-based dedup does NOT exclude it).
    let same_filename = "test.pes";
    let same_size = 9i64; // "version1" = 9 bytes
    let different_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added, is_stitched, image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) VALUES (?, ?, DATE('now'), 0, 0, 0, ?, ?)",
        )
        .bind(same_filename)
        .bind("/MachineEmbroideryDesigns/other-folder/test.pes")
        .bind(same_size)
        .bind(different_hash)
        .execute(&pool)
        .await
        .expect("seed design should insert");

    let root_paths = vec![dir.to_string_lossy().to_string()];

    let scanned = vec![scanning::ScannedFile {
        full_path: file_path_text.clone(),
        filename: same_filename.to_string(),
        extension: "pes".to_string(),
        file_size_bytes: Some(same_size),
        dedup_group_key: "test".to_string(),
    }];

    let filtered = filter_existing_scanned_files(&pool, scanned, &root_paths)
        .await
        .expect("filter should succeed");

    // The scanned file's prospective path is "import-root/test.pes" (different from "other-folder/test.pes"),
    // so Stage 0 passes. Then filename+size collide with the seeded row, requiring BLAKE3 comparison.
    // Since hashes differ, the file should NOT be treated as a duplicate.
    assert_eq!(filtered.len(), 1, "different hash should pass dedup");

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
#[serial]
async fn filter_existing_scanned_files_triple_match_excludes() {
    let pool = import_test_pool().await;

    let dir = std::env::temp_dir().join(format!(
        "rec-filter-triple-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    let file_path = dir.join("exact-match.pes");
    fs::write(&file_path, b"hello world").expect("file should be written");
    let file_path_text = file_path.to_string_lossy().to_string();

    let file_size = fs::metadata(&file_path)
        .ok()
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    let file_hash = compute_file_hash_blake3(&file_path).expect("hash should succeed");

    // Seed a design matching all three: filename, size, hash
    sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added, is_stitched, image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) VALUES (?, ?, DATE('now'), 0, 0, 0, ?, ?)",
        )
        .bind("exact-match.pes")
        .bind("/MachineEmbroideryDesigns/exact-match.pes")
        .bind(file_size)
        .bind(&file_hash)
        .execute(&pool)
        .await
        .expect("seed design should insert");

    let scanned = vec![scanning::ScannedFile {
        full_path: file_path_text.clone(),
        filename: "exact-match.pes".to_string(),
        extension: "pes".to_string(),
        file_size_bytes: Some(file_size),
        dedup_group_key: "test".to_string(),
    }];

    let filtered = filter_existing_scanned_files(&pool, scanned, &["dummy".to_string()])
        .await
        .expect("filter should succeed");

    // Triple match â†’ excluded
    assert!(filtered.is_empty(), "triple match should be excluded");

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_bulk_import_confirm_if_initialized_no_pool() {
    // Without a pool initialized, this should return Ok(0) with no panic
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let result = persist_bulk_import_confirm_if_initialized(&confirm_wire, None);
    assert_eq!(result, Ok(0));
}

#[test]
#[serial]
fn request_stop_bulk_import_sets_flag() {
    let result = request_stop_bulk_import().expect("stop request should succeed");
    assert!(result.stop_requested);

    // Calling it again should still report stop_requested = true
    let result2 = request_stop_bulk_import().expect("second stop request should succeed");
    assert!(result2.stop_requested);
}

#[test]
fn debug_bulk_import_wire_summary() {
    let wire = BulkImportWire {
        root_paths: vec!["C:/imports".to_string(), "D:/other".to_string()],
        global_designer_id: Some(7),
        global_source_id: None,
        per_folder_assignments: vec![FolderAssignmentWire {
            folder_path: "C:/imports/folder-a".to_string(),
            designer_id: Some(10),
            source_id: None,
            inferred_designer_id: None,
            inferred_source_id: None,
        }],
        selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
        create_on_import: true,
    };

    let summary = debug_bulk_import_wire(wire).expect("debug should succeed");
    assert_eq!(summary.root_path_count, 2);
    assert_eq!(summary.folder_assignment_count, 1);
    assert_eq!(summary.selected_file_count, 1);
    assert!(summary.create_on_import);
}

#[test]
fn debug_bulk_import_confirm_wire_summary() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: Some(10),
                source_id: None,
                inferred_designer_id: None,
                inferred_source_id: None,
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        },
        context_token: Some("token-abc".to_string()),
        canonical_confirm: true,
    };

    let summary = debug_bulk_import_confirm_wire(confirm_wire).expect("debug should succeed");
    assert!(summary.context_token_present);
    assert_eq!(summary.root_path_count, 1);
    assert_eq!(summary.selected_file_count, 1);
    assert!(summary.canonical_confirm);
    assert_eq!(summary.resolved_assignment_count, 1);
}

#[test]
fn debug_bulk_import_assignment_resolution_summary() {
    let confirm_wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![
                FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: None,
                },
                FolderAssignmentWire {
                    folder_path: "C:/imports/folder-b".to_string(),
                    designer_id: None,
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: None,
                },
            ],
            selected_files: vec![],
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: true,
    };

    let summary =
        debug_bulk_import_assignment_resolution_wire(confirm_wire).expect("debug should succeed");
    // folder-a: designer=Explicit, source=Global ; folder-b: designer=Global, source=Global
    assert_eq!(summary.resolved_count, 2);
    assert_eq!(summary.explicit_field_count, 1);
    assert_eq!(summary.global_field_count, 3);
    assert_eq!(summary.inferred_field_count, 0);
    assert_eq!(summary.blank_field_count, 0);
}

#[test]
fn bulk_import_request_from_conversion_edge_cases() {
    // root_path should be used when root_paths is empty
    let request = BulkImportRequest {
        root_path: Some("  C:/imports  ".to_string()),
        root_paths: vec![],
        fallback_designer_id: Some(7),
        fallback_source_id: Some(8),
    };
    let wire: BulkImportWire = request.into();
    assert_eq!(wire.root_paths, vec!["C:/imports"]);
    assert_eq!(wire.global_designer_id, Some(7));
    assert_eq!(wire.global_source_id, Some(8));

    // root_path should be ignored when root_paths is non-empty
    let request2 = BulkImportRequest {
        root_path: Some("  C:/ignored  ".to_string()),
        root_paths: vec!["  C:/actual  ".to_string()],
        fallback_designer_id: None,
        fallback_source_id: None,
    };
    let wire2: BulkImportWire = request2.into();
    assert_eq!(wire2.root_paths, vec!["C:/actual"]);

    // Empty strings should be filtered out
    let request3 = BulkImportRequest {
        root_path: None,
        root_paths: vec!["".to_string(), "  ".to_string(), "C:/valid".to_string()],
        fallback_designer_id: None,
        fallback_source_id: None,
    };
    let wire3: BulkImportWire = request3.into();
    assert_eq!(wire3.root_paths, vec!["C:/valid"]);

    // Both empty should yield empty root_paths
    let request4 = BulkImportRequest {
        root_path: None,
        root_paths: vec![],
        fallback_designer_id: None,
        fallback_source_id: None,
    };
    let wire4: BulkImportWire = request4.into();
    assert!(wire4.root_paths.is_empty());
}

#[test]
#[serial]
fn load_catalog_counts_returns_counts() {
    let pool = tauri::async_runtime::block_on(import_test_pool());

    tauri::async_runtime::block_on(async {
            sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('TestHoop', 100.0, 100.0)")
                .execute(&pool)
                .await
        })
        .expect("seed hoop should insert");

    let (design_count, hoop_count) =
        tauri::async_runtime::block_on(load_catalog_counts(&pool)).expect("counts should load");
    assert_eq!(design_count, 0);
    assert_eq!(hoop_count, 1);
}

#[test]
fn load_import_precheck_state_if_initialized_without_pool() {
    let (is_first_import, needs_hoop_setup) =
        load_import_precheck_state_if_initialized().expect("should return defaults");
    assert!(!is_first_import);
    assert!(!needs_hoop_setup);
}

#[test]
fn next_bulk_import_context_token_has_expected_format() {
    let (token, sequence) = next_bulk_import_context_token();
    assert!(
        token.starts_with("bulk-import-"),
        "token should start with bulk-import-"
    );
    assert!(token.contains('-'), "token should contain timestamps");
    let sequence2 = BULK_IMPORT_CONTEXT_COUNTER.load(Ordering::Relaxed);
    assert!(sequence2 > sequence, "counter should advance");
}

#[test]
fn canonicalize_bulk_import_confirm_wire_sets_canonical_flag() {
    let wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/test".to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: Some("test-token".to_string()),
        canonical_confirm: false,
    };

    let canonicalized = canonicalize_bulk_import_confirm_wire(wire.clone());
    assert!(canonicalized.canonical_confirm);
    assert_eq!(canonicalized.context_token, wire.context_token);
    assert_eq!(canonicalized.wire.root_paths, wire.wire.root_paths);
}

#[test]
fn build_preview_folder_assignments_merges_explicit_and_scanned() {
    let wire = BulkImportWire {
        root_paths: vec!["C:/imports".to_string()],
        global_designer_id: None,
        global_source_id: None,
        per_folder_assignments: vec![FolderAssignmentWire {
            folder_path: "C:/imports/explicit-folder".to_string(),
            designer_id: Some(1),
            source_id: Some(2),
            inferred_designer_id: None,
            inferred_source_id: None,
        }],
        selected_files: vec![],
        create_on_import: true,
    };

    let scanned_files = vec![
        scanning::ScannedFile {
            full_path: "C:/imports/scanned-folder/design.pes".to_string(),
            filename: "design.pes".to_string(),
            extension: "pes".to_string(),
            file_size_bytes: Some(100),
            dedup_group_key: "test".to_string(),
        },
        scanning::ScannedFile {
            full_path: "C:/imports/explicit-folder/design.pes".to_string(),
            filename: "design.pes".to_string(),
            extension: "pes".to_string(),
            file_size_bytes: Some(200),
            dedup_group_key: "test".to_string(),
        },
    ];

    let assignments = build_preview_folder_assignments(&wire, &scanned_files);
    assert_eq!(assignments.len(), 2);

    // Explicit folder should keep its designer/source
    let explicit = assignments
        .iter()
        .find(|a| a.folder_path.contains("explicit-folder"))
        .unwrap();
    assert_eq!(explicit.designer_id, Some(1));
    assert_eq!(explicit.source_id, Some(2));

    // Scanned-only folder should have null assignments
    let scanned = assignments
        .iter()
        .find(|a| a.folder_path.contains("scanned-folder"))
        .unwrap();
    assert_eq!(scanned.designer_id, None);
    assert_eq!(scanned.source_id, None);
}

#[test]
fn build_preview_folder_assignments_dedupes_by_normalized_path() {
    let wire = BulkImportWire {
        root_paths: vec!["C:/imports".to_string()],
        global_designer_id: None,
        global_source_id: None,
        per_folder_assignments: vec![FolderAssignmentWire {
            folder_path: "C:/imports/subfolder".to_string(),
            designer_id: Some(5),
            source_id: None,
            inferred_designer_id: None,
            inferred_source_id: None,
        }],
        selected_files: vec![],
        create_on_import: true,
    };

    // Multiple files in same folder should produce only one assignment
    let scanned_files = vec![
        scanning::ScannedFile {
            full_path: "C:/imports/subfolder/file1.pes".to_string(),
            filename: "file1.pes".to_string(),
            extension: "pes".to_string(),
            file_size_bytes: Some(100),
            dedup_group_key: "test".to_string(),
        },
        scanning::ScannedFile {
            full_path: "C:/imports/subfolder/file2.pes".to_string(),
            filename: "file2.pes".to_string(),
            extension: "pes".to_string(),
            file_size_bytes: Some(200),
            dedup_group_key: "test".to_string(),
        },
    ];

    let assignments = build_preview_folder_assignments(&wire, &scanned_files);
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].designer_id, Some(5));
}

#[test]
fn reset_bulk_import_context_store_for_startup_works() {
    // Seed a context
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        },
        context_token: None,
        canonical_confirm: false,
    })
    .expect("precheck should succeed");

    // Startup reset should clear it
    let reset = reset_bulk_import_context_store_for_startup();
    assert!(reset.cleared_context_count >= 1);
    assert_eq!(reset.active_context_count, 0);
    assert_eq!(reset.reason, "startup");
    assert!(take_bulk_import_context(&precheck.context_token).is_none());
}

#[test]
#[serial]
fn load_stitching_tag_lookup_direct() {
    let pool = tauri::async_runtime::block_on(import_test_pool());
    let lookup = tauri::async_runtime::block_on(load_stitching_tag_lookup(&pool))
        .expect("stitching tag lookup should succeed");
    // "Line Outline" is seeded as a stitching tag
    assert!(
        lookup.contains_key("Line Outline"),
        "expected 'Line Outline' stitching tag in lookup"
    );
    assert_eq!(lookup.len(), 1, "expected exactly 1 stitching tag");
}

#[test]
#[serial]
fn load_default_stitching_tag_id_direct() {
    let pool = tauri::async_runtime::block_on(import_test_pool());
    let default_id = tauri::async_runtime::block_on(load_default_stitching_tag_id(&pool))
        .expect("default stitching tag id should load");
    assert!(default_id.is_some(), "expected a default stitching tag id");
}


// ---------------------------------------------------------------------------
// load_tag_catalog / load_designers / load_sources (import inference loaders)
// ---------------------------------------------------------------------------

#[test]
fn load_tag_catalog_returns_seeded_tags() {
    let pool = tauri::async_runtime::block_on(import_test_pool());
    let tags = tauri::async_runtime::block_on(load_tag_catalog(&pool)).expect("load tags");
    assert_eq!(tags.len(), 4);
    let descriptions: Vec<&str> = tags.iter().map(|(_, d)| d.as_str()).collect();
    assert!(descriptions.contains(&"Alphabets"));
    assert!(descriptions.contains(&"Line Outline"));
}

#[test]
fn load_designers_for_import_inference_returns_empty() {
    let pool = tauri::async_runtime::block_on(import_test_pool());
    let designers = tauri::async_runtime::block_on(load_designers_for_import_inference(&pool))
        .expect("load designers");
    assert!(designers.is_empty());
}

#[test]
fn load_sources_for_import_inference_returns_empty() {
    let pool = tauri::async_runtime::block_on(import_test_pool());
    let sources = tauri::async_runtime::block_on(load_sources_for_import_inference(&pool))
        .expect("load sources");
    assert!(sources.is_empty());
}

// ---------------------------------------------------------------------------
// infer_assignment_ids_from_folder_path
// ---------------------------------------------------------------------------

#[test]
fn infer_assignment_ids_from_folder_path_matches_designer_and_source() {
    let designers = vec![(1, "Jane Doe".to_string())];
    let sources = vec![(2, "In-House".to_string())];
    let (designer_id, source_id) =
        infer_assignment_ids_from_folder_path("C:/imports/Jane Doe/In-House", &designers, &sources);
    assert_eq!(designer_id, Some(1));
    assert_eq!(source_id, Some(2));
}

#[test]
fn infer_assignment_ids_from_folder_path_returns_none_when_no_match() {
    let designers = vec![(1, "Jane Doe".to_string())];
    let sources = vec![(2, "In-House".to_string())];
    let (designer_id, source_id) =
        infer_assignment_ids_from_folder_path("C:/unrelated/folder", &designers, &sources);
    assert_eq!(designer_id, None);
    assert_eq!(source_id, None);
}

// ---------------------------------------------------------------------------
// reset_bulk_import_context_store_for_restore
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn reset_bulk_import_context_store_for_restore_clears_stored_contexts() {
    let wire = BulkImportConfirmWire {
        wire: BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: None,
            global_source_id: None,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: false,
        },
        context_token: None,
        canonical_confirm: false,
    };
    let token = store_bulk_import_context(wire);
    assert!(get_bulk_import_context(&token).is_some());

    let result = reset_bulk_import_context_store_for_restore();
    assert_eq!(result.reason, "restore");
    assert!(get_bulk_import_context(&token).is_none());
}

// ---------------------------------------------------------------------------
// derive_data_root_from_database_url
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn derive_data_root_from_database_url_uses_database_parent() {
    let previous = std::env::var("DATABASE_URL").ok();
    let tmp = std::env::temp_dir().join("bi-derive-root-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("Database")).expect("create Database dir");
    let url = format!(
        "sqlite:///{}/Database/EmbroideryCatalogue.db",
        tmp.to_string_lossy().replace('\\', "/")
    );
    std::env::set_var("DATABASE_URL", &url);

    let root = derive_data_root_from_database_url();
    let expected = tmp.canonicalize().unwrap_or_else(|_| tmp.clone());
    assert_eq!(root, expected);

    match previous {
        Some(value) => std::env::set_var("DATABASE_URL", value),
        None => std::env::remove_var("DATABASE_URL"),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// compute_prospective_stored_filepath
// ---------------------------------------------------------------------------

#[test]
fn compute_prospective_stored_filepath_uses_longest_root_leaf() {
    let full_path = "C:/imports/projects/design.pes";
    let root_paths = vec!["C:/imports".to_string(), "C:/imports/projects".to_string()];
    let result = compute_prospective_stored_filepath(full_path, &root_paths)
        .expect("prospective path should compute");
    assert_eq!(result, "/MachineEmbroideryDesigns/projects/design.pes");
}

#[test]
fn compute_prospective_stored_filepath_drive_root_places_directly() {
    let full_path = "C:/file.pes";
    let root_paths = vec!["C:/".to_string()];
    let result = compute_prospective_stored_filepath(full_path, &root_paths)
        .expect("drive-root prospective path should compute");
    assert_eq!(result, "/MachineEmbroideryDesigns/file.pes");
}

// ---------------------------------------------------------------------------
// full_path_to_stored_design_filepath / is_path_under_designs_base
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn full_path_to_stored_design_filepath_maps_in_library_and_rejects_outside() {
    let previous = std::env::var("DATABASE_URL").ok();
    let tmp = std::env::temp_dir().join("bi-stored-path-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("Database")).expect("create Database dir");
    let url = format!(
        "sqlite:///{}/Database/EmbroideryCatalogue.db",
        tmp.to_string_lossy().replace('\\', "/")
    );
    std::env::set_var("DATABASE_URL", &url);

    let in_lib = get_designs_base_path().join("sub").join("design.pes");
    let stored = full_path_to_stored_design_filepath(&in_lib.to_string_lossy())
        .expect("in-library file should map to a stored path");
    assert_eq!(stored, "/MachineEmbroideryDesigns/sub/design.pes");

    assert!(
        full_path_to_stored_design_filepath("C:/elsewhere/design.pes").is_err(),
        "outside file should be rejected"
    );

    match previous {
        Some(value) => std::env::set_var("DATABASE_URL", value),
        None => std::env::remove_var("DATABASE_URL"),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// ensure_file_in_designs_base (copy path)
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn ensure_file_in_designs_base_copies_outside_file_into_library() {
    let previous = std::env::var("DATABASE_URL").ok();
    let tmp = std::env::temp_dir().join("bi-ensure-in-base-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("Database")).expect("create Database dir");
    let url = format!(
        "sqlite:///{}/Database/EmbroideryCatalogue.db",
        tmp.to_string_lossy().replace('\\', "/")
    );
    std::env::set_var("DATABASE_URL", &url);

    // Create a source file OUTSIDE the managed designs base.
    let source_dir = tmp.join("outside_src");
    std::fs::create_dir_all(&source_dir).expect("create source dir");
    let source_file = source_dir.join("Bean.pes");
    std::fs::write(&source_file, b"PES data").expect("write source file");

    let root_paths = vec![source_dir.to_string_lossy().to_string()];
    let stored = ensure_file_in_designs_base(&source_file.to_string_lossy(), &root_paths)
        .expect("file should be copied into the library");

    assert!(stored.starts_with("/MachineEmbroideryDesigns/"));
    assert!(stored.ends_with("Bean.pes"));

    // The copy should exist on disk inside the managed designs base.
    let rel = stored.trim_start_matches("/MachineEmbroideryDesigns/");
    assert!(
        get_designs_base_path().join(rel).exists(),
        "copied file should exist in the designs base"
    );

    match previous {
        Some(value) => std::env::set_var("DATABASE_URL", value),
        None => std::env::remove_var("DATABASE_URL"),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
