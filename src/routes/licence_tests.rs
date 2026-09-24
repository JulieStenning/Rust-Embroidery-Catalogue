// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::services::licence;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::atomic::AtomicBool;

fn make_test_paths(data_root: std::path::PathBuf) -> crate::paths::AppPaths {
    crate::paths::AppPaths {
        mode: crate::paths::ExecutionMode::Dev,
        data_root: data_root.clone(),
        embroidery_designs_dir: data_root.join("MachineEmbroideryDesigns"),
        database_dir: data_root.join("Database"),
        database_path: data_root
            .join("Database")
            .join(crate::paths::DATABASE_FILENAME),
        log_dir: data_root.join("logs"),
    }
}

fn make_app_state(pool: SqlitePool, paths: crate::paths::AppPaths) -> AppState {
    AppState {
        db: crate::PoolHolder::new(pool),
        database_status: crate::DatabaseStatus {
            status: crate::DatabaseStatusKind::Connected,
            configured_data_root: Some(paths.data_root.to_string_lossy().to_string()),
            database_path: Some(paths.database_path.to_string_lossy().to_string()),
            embroidery_dir: Some(paths.embroidery_designs_dir.to_string_lossy().to_string()),
            data_root_missing: false,
        },
        paths,
        log_guard: crate::logging::LogGuard::dummy_for_test(),
        shutdown_requested: AtomicBool::new(false),
        maintenance_running: AtomicBool::new(false),
        migration_running: AtomicBool::new(false),
        migration_cancel_requested: std::sync::Arc::new(AtomicBool::new(false)),
        restore_in_progress: AtomicBool::new(false),
    }
}

fn generate_valid_key(email: &str, tier: &str, expires_at: Option<i64>) -> String {
    let secret = licence::hex_to_32_bytes(
        "0000110f1c41bd48bdeef29e1aa87870e8fc58ff09261da6aea07e8124994c73",
    )
    .unwrap();
    let signing_key = SigningKey::from_bytes(&secret);
    let payload = licence::LicencePayload {
        email: email.to_string(),
        tier: tier.to_string(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        expires_at,
    };
    let payload_json = serde_json::to_vec(&payload).unwrap();
    let signature = signing_key.sign(&payload_json);
    let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_json);
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());
    format!("EMB1.{}.{}", payload_b64, sig_b64)
}

#[tokio::test]
async fn test_licence_routes_flow() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    crate::database::migrations::run_migrations(&pool)
        .await
        .unwrap();

    let paths = make_test_paths(std::path::PathBuf::from("test_data_root"));
    let state = make_app_state(pool, paths);

    // 1. Initial status: unactivated
    let status = get_licence_status_inner(&state).await.unwrap();
    assert!(!status.is_active);
    assert!(!status.is_valid);

    // 2. Activate with invalid key -> returns Err
    let err =
        activate_licence_inner(&state, "test@beta.com".to_string(), "INVALID".to_string()).await;
    assert!(err.is_err());

    // 3. Activate with valid beta key
    let valid_key = generate_valid_key(
        "test@beta.com",
        "beta",
        Some(time::OffsetDateTime::now_utc().unix_timestamp() + 864000),
    );
    let status = activate_licence_inner(&state, "test@beta.com".to_string(), valid_key)
        .await
        .unwrap();
    assert!(status.is_active);
    assert!(status.is_valid);
    assert_eq!(status.email.as_deref(), Some("test@beta.com"));
    assert_eq!(status.tier.as_deref(), Some("beta"));

    // 4. Get status again -> returns active & valid
    let status = get_licence_status_inner(&state).await.unwrap();
    assert!(status.is_active);
    assert!(status.is_valid);

    // 5. Deactivate licence
    let status = deactivate_licence_inner(&state).await.unwrap();
    assert!(!status.is_active);
    assert!(!status.is_valid);
}
