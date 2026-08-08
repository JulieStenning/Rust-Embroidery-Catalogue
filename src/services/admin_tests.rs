// Tests for the admin service.
//
// This module was split out of admin.rs so the service file can stay
// focused on production logic. It is included via a #[path] declaration
// in a #[cfg(test)] mod tests; module, so it retains full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::database::migrations::run_migrations;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    pool
}

// ---------------------------------------------------------------------
// validate_non_empty
// ---------------------------------------------------------------------
#[test]
fn validate_non_empty_trims_and_accepts_value() {
    let result = validate_non_empty("  Hello World  ", "Label").unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
fn validate_non_empty_rejects_blank() {
    let err = validate_non_empty("   ", "Designer name").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Designer name is required."
    ));
}

#[test]
fn validate_non_empty_rejects_empty() {
    let err = validate_non_empty("", "Source name").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Source name is required."
    ));
}

// ---------------------------------------------------------------------
// validate_positive
// ---------------------------------------------------------------------
#[test]
fn validate_positive_accepts_value() {
    let result = validate_positive(100.0, "Width").unwrap();
    assert_eq!(result, 100.0);
}

#[test]
fn validate_positive_rejects_zero() {
    let err = validate_positive(0.0, "Max Width (mm)").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Max Width (mm) must be a positive number."
    ));
}

#[test]
fn validate_positive_rejects_negative() {
    let err = validate_positive(-5.0, "Max Height (mm)").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Max Height (mm) must be a positive number."
    ));
}

#[test]
fn validate_positive_rejects_nan() {
    let err = validate_positive(f64::NAN, "Width").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Width must be a positive number."
    ));
}

#[test]
fn validate_positive_rejects_infinity() {
    let err = validate_positive(f64::INFINITY, "Height").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Height must be a positive number."
    ));
}

#[test]
fn validate_positive_rejects_negative_infinity() {
    let err = validate_positive(f64::NEG_INFINITY, "Width").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Width must be a positive number."
    ));
}

// ---------------------------------------------------------------------
// validate_tag_group
// ---------------------------------------------------------------------
#[test]
fn validate_tag_group_accepts_image() {
    assert_eq!(validate_tag_group("image").unwrap(), "image".to_string());
}

#[test]
fn validate_tag_group_accepts_stitching() {
    assert_eq!(
        validate_tag_group("stitching").unwrap(),
        "stitching".to_string()
    );
}

#[test]
fn validate_tag_group_normalises_case_and_whitespace() {
    assert_eq!(
        validate_tag_group("  ImAgE  ").unwrap(),
        "image".to_string()
    );
    assert_eq!(
        validate_tag_group("StItChInG").unwrap(),
        "stitching".to_string()
    );
}

#[test]
fn validate_tag_group_rejects_unknown() {
    let err = validate_tag_group("colour").unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Tag group must be 'image' or 'stitching'."
    ));
}

// ---------------------------------------------------------------------
// ensure_unique_name / ensure_unique_name_except_id
// ---------------------------------------------------------------------
#[tokio::test]
async fn ensure_unique_accepts_new_name() {
    let pool = setup().await;
    ensure_unique_name(&pool, "designers", "Fresh", "Designer")
        .await
        .unwrap();
}

#[tokio::test]
async fn ensure_unique_rejects_duplicate_case_insensitive() {
    let pool = setup().await;
    let req = CreateDesignerRequest {
        name: "ACME".to_string(),
    };
    create_designer_with_pool(&pool, req).await.unwrap();
    let err = ensure_unique_name(&pool, "designers", "acme", "Designer")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Designer 'acme' already exists."
    ));
}

#[tokio::test]
async fn ensure_unique_except_id_ignores_excluded_row() {
    let pool = setup().await;
    let a = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Beta".to_string(),
        },
    )
    .await
    .unwrap();

    // Renaming Alpha to "Alpha" (same row, different case) must be allowed.
    ensure_unique_name_except_id(&pool, "designers", "id", a.id, "alpha", "Designer")
        .await
        .unwrap();
}

#[tokio::test]
async fn ensure_unique_except_id_rejects_other_row() {
    let pool = setup().await;
    let a = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Beta".to_string(),
        },
    )
    .await
    .unwrap();

    let err = ensure_unique_name_except_id(&pool, "designers", "id", a.id, "BETA", "Designer")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::InvalidInput { message } if message == "Designer 'BETA' already exists."
    ));
}

// ---------------------------------------------------------------------
// Designers
// ---------------------------------------------------------------------
#[tokio::test]
async fn list_designers_empty() {
    let pool = setup().await;
    let result = list_designers_with_pool(&pool).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn create_and_list_designers() {
    let pool = setup().await;
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Zebra".to_string(),
        },
    )
    .await
    .unwrap();
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();

    let list = list_designers_with_pool(&pool).await.unwrap();
    assert_eq!(list.len(), 2);
    // ORDER BY name COLLATE NOCASE ASC
    assert_eq!(list[0].name, "Alpha");
    assert_eq!(list[1].name, "Zebra");
    assert_eq!(list[0].design_count, 0);
}

#[tokio::test]
async fn create_designer_success_and_count() {
    let pool = setup().await;
    let d = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "ACME".to_string(),
        },
    )
    .await
    .unwrap();
    assert!(d.id > 0);
    assert_eq!(d.name, "ACME");
    assert_eq!(d.design_count, 0);
}

#[tokio::test]
async fn create_designer_empty_name_errors() {
    let pool = setup().await;
    let err = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "   ".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_designer_duplicate_case_insensitive_errors() {
    let pool = setup().await;
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "ACME".to_string(),
        },
    )
    .await
    .unwrap();
    let err = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "acme".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_designer_success() {
    let pool = setup().await;
    let created = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Old".to_string(),
        },
    )
    .await
    .unwrap();
    let updated = update_designer_with_pool(
        &pool,
        UpdateDesignerRequest {
            designer_id: created.id,
            name: "New".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "New");
}

#[tokio::test]
async fn update_designer_not_found_errors() {
    let pool = setup().await;
    let err = update_designer_with_pool(
        &pool,
        UpdateDesignerRequest {
            designer_id: 9999,
            name: "Ghost".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn update_designer_duplicate_errors() {
    let pool = setup().await;
    let a = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();
    create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Beta".to_string(),
        },
    )
    .await
    .unwrap();
    let err = update_designer_with_pool(
        &pool,
        UpdateDesignerRequest {
            designer_id: a.id,
            name: "beta".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn delete_designer_success_and_empty() {
    let pool = setup().await;
    let created = create_designer_with_pool(
        &pool,
        CreateDesignerRequest {
            name: "Temp".to_string(),
        },
    )
    .await
    .unwrap();
    delete_designer_with_pool(&pool, created.id).await.unwrap();
    let list = list_designers_with_pool(&pool).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn delete_designer_not_found_errors() {
    let pool = setup().await;
    let err = delete_designer_with_pool(&pool, 9999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

// ---------------------------------------------------------------------
// Sources
// ---------------------------------------------------------------------
#[tokio::test]
async fn list_sources_empty() {
    let pool = setup().await;
    let result = list_sources_with_pool(&pool).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn create_and_list_sources() {
    let pool = setup().await;
    create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Zeta".to_string(),
        },
    )
    .await
    .unwrap();
    create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();
    let list = list_sources_with_pool(&pool).await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "Alpha");
    assert_eq!(list[1].name, "Zeta");
}

#[tokio::test]
async fn create_source_success() {
    let pool = setup().await;
    let s = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Etsy".to_string(),
        },
    )
    .await
    .unwrap();
    assert!(s.id > 0);
    assert_eq!(s.name, "Etsy");
    assert_eq!(s.design_count, 0);
}

#[tokio::test]
async fn create_source_empty_name_errors() {
    let pool = setup().await;
    let err = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "  ".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_source_duplicate_case_insensitive_errors() {
    let pool = setup().await;
    create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Etsy".to_string(),
        },
    )
    .await
    .unwrap();
    let err = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "ETSY".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_source_success() {
    let pool = setup().await;
    let created = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Old".to_string(),
        },
    )
    .await
    .unwrap();
    let updated = update_source_with_pool(
        &pool,
        UpdateSourceRequest {
            source_id: created.id,
            name: "New".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "New");
}

#[tokio::test]
async fn update_source_not_found_errors() {
    let pool = setup().await;
    let err = update_source_with_pool(
        &pool,
        UpdateSourceRequest {
            source_id: 9999,
            name: "Ghost".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn update_source_duplicate_errors() {
    let pool = setup().await;
    let a = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Alpha".to_string(),
        },
    )
    .await
    .unwrap();
    create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Beta".to_string(),
        },
    )
    .await
    .unwrap();
    let err = update_source_with_pool(
        &pool,
        UpdateSourceRequest {
            source_id: a.id,
            name: "beta".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn delete_source_success_and_empty() {
    let pool = setup().await;
    let created = create_source_with_pool(
        &pool,
        CreateSourceRequest {
            name: "Temp".to_string(),
        },
    )
    .await
    .unwrap();
    delete_source_with_pool(&pool, created.id).await.unwrap();
    let list = list_sources_with_pool(&pool).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn delete_source_not_found_errors() {
    let pool = setup().await;
    let err = delete_source_with_pool(&pool, 9999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

// ---------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------
#[tokio::test]
async fn list_tags_contains_system_tags() {
    let pool = setup().await;
    let list = list_tags_with_pool(&pool).await.unwrap();
    assert!(!list.is_empty());
    assert!(list.iter().any(|t| t.is_system));
}

#[tokio::test]
async fn create_tag_success() {
    let pool = setup().await;
    let tag = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Floral".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    assert!(tag.id > 0);
    assert_eq!(tag.description, "Floral");
    assert_eq!(tag.tag_group, Some("image".to_string()));
    assert!(!tag.is_system);
    assert_eq!(tag.design_count, 0);
}

#[tokio::test]
async fn create_tag_empty_description_errors() {
    let pool = setup().await;
    let err = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "  ".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_tag_invalid_group_errors() {
    let pool = setup().await;
    let err = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Test".to_string(),
            tag_group: "colour".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_tag_duplicate_case_insensitive_errors() {
    let pool = setup().await;
    create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Floral".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    let err = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "floral".to_string(),
            tag_group: "stitching".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_tag_success() {
    let pool = setup().await;
    let created = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Old".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    let updated = update_tag_with_pool(
        &pool,
        UpdateTagRequest {
            tag_id: created.id,
            description: "New".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.description, "New");
}

#[tokio::test]
async fn update_tag_not_found_errors() {
    let pool = setup().await;
    let err = update_tag_with_pool(
        &pool,
        UpdateTagRequest {
            tag_id: 9999,
            description: "Ghost".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn update_tag_duplicate_errors() {
    let pool = setup().await;
    let a = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Alpha".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Beta".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    let err = update_tag_with_pool(
        &pool,
        UpdateTagRequest {
            tag_id: a.id,
            description: "beta".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_tag_system_tag_rejected() {
    let pool = setup().await;
    let all = list_tags_with_pool(&pool).await.unwrap();
    let system = all.iter().find(|t| t.is_system).expect("system tag exists");
    let err = update_tag_with_pool(
        &pool,
        UpdateTagRequest {
            tag_id: system.id,
            description: "Hacked".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn set_tag_group_success() {
    let pool = setup().await;
    let created = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "Group Test".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    let updated = set_tag_group_with_pool(
        &pool,
        SetTagGroupRequest {
            tag_id: created.id,
            tag_group: "stitching".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.tag_group, Some("stitching".to_string()));
}

#[tokio::test]
async fn set_tag_group_not_found_errors() {
    let pool = setup().await;
    let err = set_tag_group_with_pool(
        &pool,
        SetTagGroupRequest {
            tag_id: 9999,
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn set_tag_group_system_tag_rejected() {
    let pool = setup().await;
    let all = list_tags_with_pool(&pool).await.unwrap();
    let system = all.iter().find(|t| t.is_system).expect("system tag exists");
    let err = set_tag_group_with_pool(
        &pool,
        SetTagGroupRequest {
            tag_id: system.id,
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn delete_tag_success() {
    let pool = setup().await;
    let created = create_tag_with_pool(
        &pool,
        CreateTagRequest {
            description: "DeleteMe".to_string(),
            tag_group: "image".to_string(),
        },
    )
    .await
    .unwrap();
    delete_tag_with_pool(&pool, created.id).await.unwrap();
    let list = list_tags_with_pool(&pool).await.unwrap();
    assert!(!list.iter().any(|t| t.id == created.id));
}

#[tokio::test]
async fn delete_tag_not_found_errors() {
    let pool = setup().await;
    let err = delete_tag_with_pool(&pool, 9999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn delete_tag_system_tag_rejected() {
    let pool = setup().await;
    let all = list_tags_with_pool(&pool).await.unwrap();
    let system = all.iter().find(|t| t.is_system).expect("system tag exists");
    let err = delete_tag_with_pool(&pool, system.id).await.unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

// ---------------------------------------------------------------------
// Hoops
// ---------------------------------------------------------------------
#[tokio::test]
async fn list_hoops_empty() {
    let pool = setup().await;
    let result = list_hoops_with_pool(&pool).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn create_and_list_hoops_ordered_by_size() {
    let pool = setup().await;
    create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "5x7".to_string(),
            max_width_mm: 200.0,
            max_height_mm: 300.0,
        },
    )
    .await
    .unwrap();
    create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "4x4".to_string(),
            max_width_mm: 100.0,
            max_height_mm: 100.0,
        },
    )
    .await
    .unwrap();
    let list = list_hoops_with_pool(&pool).await.unwrap();
    assert_eq!(list.len(), 2);
    // ORDER BY max_width_mm ASC
    assert_eq!(list[0].name, "4x4");
    assert_eq!(list[1].name, "5x7");
}

#[tokio::test]
async fn create_hoop_success() {
    let pool = setup().await;
    let h = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "4x4".to_string(),
            max_width_mm: 100.0,
            max_height_mm: 100.0,
        },
    )
    .await
    .unwrap();
    assert!(h.id > 0);
    assert_eq!(h.name, "4x4");
    assert_eq!(h.max_width_mm, 100.0);
    assert_eq!(h.max_height_mm, 100.0);
    assert_eq!(h.design_count, 0);
}

#[tokio::test]
async fn create_hoop_empty_name_errors() {
    let pool = setup().await;
    let err = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "  ".to_string(),
            max_width_mm: 100.0,
            max_height_mm: 100.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_hoop_invalid_dimensions_errors() {
    let pool = setup().await;
    let err = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Bad".to_string(),
            max_width_mm: 0.0,
            max_height_mm: 100.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));

    let err = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Bad2".to_string(),
            max_width_mm: 100.0,
            max_height_mm: f64::NAN,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn create_hoop_duplicate_case_insensitive_errors() {
    let pool = setup().await;
    create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "4x4".to_string(),
            max_width_mm: 100.0,
            max_height_mm: 100.0,
        },
    )
    .await
    .unwrap();
    let err = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "4X4".to_string(),
            max_width_mm: 200.0,
            max_height_mm: 200.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_hoop_success() {
    let pool = setup().await;
    let created = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Old".to_string(),
            max_width_mm: 10.0,
            max_height_mm: 10.0,
        },
    )
    .await
    .unwrap();
    let updated = update_hoop_with_pool(
        &pool,
        UpdateHoopRequest {
            hoop_id: created.id,
            name: "New Hoop".to_string(),
            max_width_mm: 200.0,
            max_height_mm: 200.0,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "New Hoop");
    assert_eq!(updated.max_width_mm, 200.0);
    assert_eq!(updated.max_height_mm, 200.0);
}

#[tokio::test]
async fn update_hoop_not_found_errors() {
    let pool = setup().await;
    let err = update_hoop_with_pool(
        &pool,
        UpdateHoopRequest {
            hoop_id: 9999,
            name: "Ghost".to_string(),
            max_width_mm: 10.0,
            max_height_mm: 10.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[tokio::test]
async fn update_hoop_invalid_dimensions_errors() {
    let pool = setup().await;
    let created = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Valid".to_string(),
            max_width_mm: 10.0,
            max_height_mm: 10.0,
        },
    )
    .await
    .unwrap();
    let err = update_hoop_with_pool(
        &pool,
        UpdateHoopRequest {
            hoop_id: created.id,
            name: "New".to_string(),
            max_width_mm: -1.0,
            max_height_mm: 10.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn update_hoop_duplicate_errors() {
    let pool = setup().await;
    let a = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Alpha".to_string(),
            max_width_mm: 1.0,
            max_height_mm: 1.0,
        },
    )
    .await
    .unwrap();
    create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Beta".to_string(),
            max_width_mm: 2.0,
            max_height_mm: 2.0,
        },
    )
    .await
    .unwrap();
    let err = update_hoop_with_pool(
        &pool,
        UpdateHoopRequest {
            hoop_id: a.id,
            name: "beta".to_string(),
            max_width_mm: 3.0,
            max_height_mm: 3.0,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidInput { .. }));
}

#[tokio::test]
async fn delete_hoop_success_and_empty() {
    let pool = setup().await;
    let created = create_hoop_with_pool(
        &pool,
        CreateHoopRequest {
            name: "Temp".to_string(),
            max_width_mm: 1.0,
            max_height_mm: 1.0,
        },
    )
    .await
    .unwrap();
    delete_hoop_with_pool(&pool, created.id).await.unwrap();
    let list = list_hoops_with_pool(&pool).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn delete_hoop_not_found_errors() {
    let pool = setup().await;
    let err = delete_hoop_with_pool(&pool, 9999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}
