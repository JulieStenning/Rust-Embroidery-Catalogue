// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use thiserror::Error;

/// The embedded production public key used to verify licence signatures offline.
pub const EMBEDDED_PUBLIC_KEY_HEX: &str =
    "845c971776d7a5c9b9e10705df049e6e943faaeeb1ae5f28d571691218476c0a";

#[derive(Debug, Error)]
pub enum LicenceError {
    #[error("Invalid licence key format")]
    InvalidFormat,
    #[error("Could not decode licence payload: {0}")]
    PayloadDecodeError(String),
    #[error("Invalid cryptographic signature")]
    InvalidSignature,
    #[error("Licence key does not match the provided email address")]
    EmailMismatch,
    #[error("This beta licence expired on {0}")]
    Expired(String),
    #[error("Licence verification error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicencePayload {
    pub email: String,
    pub tier: String,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LicenceStatus {
    pub is_active: bool,
    pub is_valid: bool,
    pub email: Option<String>,
    pub tier: Option<String>,
    pub expires_at: Option<i64>,
    pub expires_at_formatted: Option<String>,
    pub error_message: Option<String>,
}

/// Helper to decode a hex string into a fixed 32-byte array.
pub fn hex_to_32_bytes(hex_str: &str) -> Result<[u8; 32], LicenceError> {
    let hex_trimmed = hex_str.trim();
    if hex_trimmed.len() != 64 {
        return Err(LicenceError::Other(format!(
            "Invalid hex length: expected 64 characters, got {}",
            hex_trimmed.len()
        )));
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_trimmed[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|e| LicenceError::Other(format!("Hex parse error: {}", e)))?;
    }
    Ok(bytes)
}

/// Verify a licence key string against an email address offline using Ed25519.
pub fn verify_licence_key(
    email: &str,
    licence_key: &str,
    custom_public_key_hex: Option<&str>,
    custom_timestamp: Option<i64>,
) -> Result<LicencePayload, LicenceError> {
    let key_trimmed = licence_key.trim();
    let email_trimmed = email.trim();

    if email_trimmed.is_empty() {
        return Err(LicenceError::EmailMismatch);
    }

    let parts: Vec<&str> = key_trimmed.split('.').collect();
    if parts.len() != 3 || parts[0] != "EMB1" {
        return Err(LicenceError::InvalidFormat);
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| LicenceError::PayloadDecodeError(e.to_string()))?;

    let signature_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| LicenceError::InvalidFormat)?;

    if signature_bytes.len() != 64 {
        return Err(LicenceError::InvalidSignature);
    }

    let pub_key_hex = custom_public_key_hex.unwrap_or(EMBEDDED_PUBLIC_KEY_HEX);
    let pub_key_bytes = hex_to_32_bytes(pub_key_hex)?;
    let verifying_key = VerifyingKey::from_bytes(&pub_key_bytes)
        .map_err(|e| LicenceError::Other(format!("Invalid verifying key: {}", e)))?;

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&signature_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| LicenceError::InvalidSignature)?;

    let payload: LicencePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| LicenceError::PayloadDecodeError(e.to_string()))?;

    if !payload.email.trim().eq_ignore_ascii_case(email_trimmed) {
        return Err(LicenceError::EmailMismatch);
    }

    let now = custom_timestamp.unwrap_or_else(|| time::OffsetDateTime::now_utc().unix_timestamp());

    if let Some(exp) = payload.expires_at {
        if now > exp {
            let formatted_date = match time::OffsetDateTime::from_unix_timestamp(exp) {
                Ok(odt) => format!("{}-{:02}-{:02}", odt.year(), odt.month() as u8, odt.day()),
                Err(_) => "the specified expiration date".to_string(),
            };
            return Err(LicenceError::Expired(formatted_date));
        }
    }

    Ok(payload)
}

// ---------------------------------------------------------------------------
// Database persistence helpers
// ---------------------------------------------------------------------------

const KEY_LICENCE_EMAIL: &str = "licence_email";
const KEY_LICENCE_KEY: &str = "licence_key";
const KEY_LICENCE_TIER: &str = "licence_tier";
const KEY_LICENCE_EXPIRES_AT: &str = "licence_expires_at";

/// Load and evaluate the current stored licence status from SQLite.
pub async fn check_stored_licence_status(
    conn: &mut SqliteConnection,
) -> Result<LicenceStatus, sqlx::Error> {
    let email_setting = crate::settings::get_setting(conn, KEY_LICENCE_EMAIL).await?;
    let key_setting = crate::settings::get_setting(conn, KEY_LICENCE_KEY).await?;

    let email = match email_setting {
        Some(s) if !s.value.trim().is_empty() => s.value,
        _ => return Ok(LicenceStatus::default()),
    };

    let key = match key_setting {
        Some(s) if !s.value.trim().is_empty() => s.value,
        _ => return Ok(LicenceStatus::default()),
    };

    match verify_licence_key(&email, &key, None, None) {
        Ok(payload) => {
            let expires_formatted = payload.expires_at.and_then(|exp| {
                time::OffsetDateTime::from_unix_timestamp(exp)
                    .ok()
                    .map(|odt| format!("{}-{:02}-{:02}", odt.year(), odt.month() as u8, odt.day()))
            });

            Ok(LicenceStatus {
                is_active: true,
                is_valid: true,
                email: Some(payload.email),
                tier: Some(payload.tier),
                expires_at: payload.expires_at,
                expires_at_formatted: expires_formatted,
                error_message: None,
            })
        }
        Err(err) => Ok(LicenceStatus {
            is_active: true,
            is_valid: false,
            email: Some(email),
            tier: None,
            expires_at: None,
            expires_at_formatted: None,
            error_message: Some(err.to_string()),
        }),
    }
}

async fn upsert_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, '')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(conn)
    .await?;
    Ok(())
}

/// Save and activate a licence key in the database.
pub async fn save_licence(
    conn: &mut SqliteConnection,
    email: &str,
    licence_key: &str,
    payload: &LicencePayload,
) -> Result<(), sqlx::Error> {
    upsert_setting(conn, KEY_LICENCE_EMAIL, email.trim()).await?;
    upsert_setting(conn, KEY_LICENCE_KEY, licence_key.trim()).await?;
    upsert_setting(conn, KEY_LICENCE_TIER, &payload.tier).await?;
    let exp_str = payload
        .expires_at
        .map(|e| e.to_string())
        .unwrap_or_default();
    upsert_setting(conn, KEY_LICENCE_EXPIRES_AT, &exp_str).await?;
    Ok(())
}

/// Deactivate and clear stored licence details from the database.
pub async fn clear_licence(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    upsert_setting(conn, KEY_LICENCE_EMAIL, "").await?;
    upsert_setting(conn, KEY_LICENCE_KEY, "").await?;
    upsert_setting(conn, KEY_LICENCE_TIER, "").await?;
    upsert_setting(conn, KEY_LICENCE_EXPIRES_AT, "").await?;
    Ok(())
}

#[cfg(test)]
#[path = "licence_tests.rs"]
mod tests;
