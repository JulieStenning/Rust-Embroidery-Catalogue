// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use sqlx::sqlite::SqlitePoolOptions;

fn generate_test_key(
    signing_key: &SigningKey,
    email: &str,
    tier: &str,
    issued_at: i64,
    expires_at: Option<i64>,
) -> String {
    let payload = LicencePayload {
        email: email.to_string(),
        tier: tier.to_string(),
        issued_at,
        expires_at,
    };
    let payload_json = serde_json::to_vec(&payload).unwrap();
    let signature = signing_key.sign(&payload_json);

    let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_json);
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

    format!("EMB1.{}.{}", payload_b64, sig_b64)
}

#[test]
fn test_verify_licence_success() {
    let secret = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let pub_key_bytes = signing_key.verifying_key().to_bytes();
    let pub_key_hex = hex_encode(&pub_key_bytes);

    let key = generate_test_key(&signing_key, "tester@example.com", "beta", 1000, Some(2000));

    let res = verify_licence_key("tester@example.com", &key, Some(&pub_key_hex), Some(1500));
    assert!(res.is_ok());
    let payload = res.unwrap();
    assert_eq!(payload.email, "tester@example.com");
    assert_eq!(payload.tier, "beta");
    assert_eq!(payload.expires_at, Some(2000));
}

#[test]
fn test_verify_licence_case_insensitive_email() {
    let secret = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let pub_key_hex = hex_encode(&signing_key.verifying_key().to_bytes());

    let key = generate_test_key(&signing_key, "Tester@Example.COM", "beta", 1000, Some(2000));

    let res = verify_licence_key("tester@example.com", &key, Some(&pub_key_hex), Some(1500));
    assert!(res.is_ok());
}

#[test]
fn test_verify_licence_email_mismatch() {
    let secret = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let pub_key_hex = hex_encode(&signing_key.verifying_key().to_bytes());

    let key = generate_test_key(&signing_key, "alice@example.com", "beta", 1000, Some(2000));

    let res = verify_licence_key("bob@example.com", &key, Some(&pub_key_hex), Some(1500));
    assert!(matches!(res, Err(LicenceError::EmailMismatch)));
}

#[test]
fn test_verify_licence_expired() {
    let secret = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let pub_key_hex = hex_encode(&signing_key.verifying_key().to_bytes());

    let key = generate_test_key(&signing_key, "tester@example.com", "beta", 1000, Some(1500));

    let res = verify_licence_key("tester@example.com", &key, Some(&pub_key_hex), Some(2000));
    assert!(matches!(res, Err(LicenceError::Expired(_))));
}

#[test]
fn test_verify_licence_invalid_signature() {
    let secret1 = [42u8; 32];
    let signing_key1 = SigningKey::from_bytes(&secret1);

    let secret2 = [99u8; 32];
    let signing_key2 = SigningKey::from_bytes(&secret2);
    let pub_key2_hex = hex_encode(&signing_key2.verifying_key().to_bytes());

    // Signed with key 1, verified against key 2
    let key = generate_test_key(
        &signing_key1,
        "tester@example.com",
        "beta",
        1000,
        Some(2000),
    );

    let res = verify_licence_key("tester@example.com", &key, Some(&pub_key2_hex), Some(1500));
    assert!(matches!(res, Err(LicenceError::InvalidSignature)));
}

#[test]
fn test_verify_licence_lifetime_never_expires() {
    let secret = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let pub_key_hex = hex_encode(&signing_key.verifying_key().to_bytes());

    let key = generate_test_key(&signing_key, "lifetime@example.com", "lifetime", 1000, None);

    let res = verify_licence_key(
        "lifetime@example.com",
        &key,
        Some(&pub_key_hex),
        Some(9999999999),
    );
    assert!(res.is_ok());
    assert_eq!(res.unwrap().expires_at, None);
}

#[tokio::test]
async fn test_db_licence_persistence() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    crate::database::migrations::run_migrations(&pool)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();

    // 1. Initial state: not active
    let status = check_stored_licence_status(&mut conn).await.unwrap();
    assert!(!status.is_active);
    assert!(!status.is_valid);

    // 2. Save licence with embedded key
    let secret =
        hex_to_32_bytes("0000110f1c41bd48bdeef29e1aa87870e8fc58ff09261da6aea07e8124994c73")
            .unwrap();
    let signing_key = SigningKey::from_bytes(&secret);
    let key = generate_test_key(
        &signing_key,
        "julie@test.com",
        "beta",
        1000,
        Some(4102444800), // Year 2100
    );

    let payload = verify_licence_key("julie@test.com", &key, None, None).unwrap();
    save_licence(&mut conn, "julie@test.com", &key, &payload)
        .await
        .unwrap();

    // 3. Check status: active & valid
    let status = check_stored_licence_status(&mut conn).await.unwrap();
    assert!(status.is_active);
    assert!(status.is_valid);
    assert_eq!(status.email.as_deref(), Some("julie@test.com"));
    assert_eq!(status.tier.as_deref(), Some("beta"));

    // 4. Clear licence
    clear_licence(&mut conn).await.unwrap();
    let status = check_stored_licence_status(&mut conn).await.unwrap();
    assert!(!status.is_active);
    assert!(!status.is_valid);
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
