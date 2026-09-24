// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::env;

const DEFAULT_PRIVATE_KEY_HEX: &str =
    "0000110f1c41bd48bdeef29e1aa87870e8fc58ff09261da6aea07e8124994c73";
const DEFAULT_PUBLIC_KEY_HEX: &str =
    "845c971776d7a5c9b9e10705df049e6e943faaeeb1ae5f28d571691218476c0a";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicencePayload {
    pub email: String,
    pub tier: String,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

fn print_usage() {
    println!(
        r#"
Embroidery Catalogue - Licence Key Generator CLI

USAGE:
    cargo run --bin keygen -- [OPTIONS]

OPTIONS:
    --email <EMAIL>        Generate a licence key for the given email
    --tier <TIER>          Licence tier: 'beta', 'standard', 'lifetime' (default: 'beta')
    --days <DAYS>          Validity duration in days for beta keys (default: 90)
    --secret <HEX>         Custom 32-byte secret key in hex format
    --verify               Verify an existing key (requires --email and --key)
    --key <KEY>            The licence key to verify
    --generate-keypair     Generate and print a new random Ed25519 keypair
    --help                 Print this help message

EXAMPLES:
    cargo run --bin keygen -- --email tester@example.com --tier beta --days 90
    cargo run --bin keygen -- --email buyer@example.com --tier lifetime
    cargo run --bin keygen -- --verify --email tester@example.com --key EMB1-...
"#
    );
}

fn hex_to_32_bytes(hex_str: &str) -> Result<[u8; 32], String> {
    let hex_trimmed = hex_str.trim();
    if hex_trimmed.len() != 64 {
        return Err(format!(
            "Invalid hex length: expected 64 hex characters, got {}",
            hex_trimmed.len()
        ));
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_trimmed[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Hex decode error at pos {}: {}", i, e))?;
    }
    Ok(bytes)
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn verify_key(
    email: &str,
    licence_key: &str,
    public_key_hex: &str,
) -> Result<LicencePayload, String> {
    let key_trimmed = licence_key.trim();
    let email_trimmed = email.trim();

    let parts: Vec<&str> = key_trimmed.split('.').collect();
    if parts.len() != 3 || parts[0] != "EMB1" {
        return Err("Invalid key format (must start with EMB1.)".to_string());
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| format!("Invalid base64 payload: {}", e))?;

    let signature_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|e| format!("Invalid base64 signature: {}", e))?;

    if signature_bytes.len() != 64 {
        return Err("Invalid signature length".to_string());
    }

    let pub_key_bytes = hex_to_32_bytes(public_key_hex)?;
    let verifying_key = VerifyingKey::from_bytes(&pub_key_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&signature_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| "Signature verification failed".to_string())?;

    let payload: LicencePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Payload parse error: {}", e))?;

    if !payload.email.trim().eq_ignore_ascii_case(email_trimmed) {
        return Err(format!(
            "Email mismatch: key is for '{}', but provided '{}'",
            payload.email, email_trimmed
        ));
    }

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if let Some(exp) = payload.expires_at {
        if now > exp {
            let formatted = match time::OffsetDateTime::from_unix_timestamp(exp) {
                Ok(odt) => format!("{}-{:02}-{:02}", odt.year(), odt.month() as u8, odt.day()),
                Err(_) => exp.to_string(),
            };
            return Err(format!("Licence expired on {}", formatted));
        }
    }

    Ok(payload)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        return;
    }

    if args.contains(&"--generate-keypair".to_string()) {
        let mut secret_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        println!("=== New Ed25519 Keypair Generated ===");
        println!("PRIVATE_KEY_HEX = {}", hex_encode(signing_key.to_bytes()));
        println!("PUBLIC_KEY_HEX  = {}", hex_encode(verifying_key.to_bytes()));
        return;
    }

    let mut email: Option<String> = None;
    let mut tier: String = "beta".to_string();
    let mut days: i64 = 90;
    let mut secret_hex: String = DEFAULT_PRIVATE_KEY_HEX.to_string();
    let mut is_verify: bool = false;
    let mut key_to_verify: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--email" if i + 1 < args.len() => {
                email = Some(args[i + 1].clone());
                i += 2;
            }
            "--tier" if i + 1 < args.len() => {
                tier = args[i + 1].to_lowercase();
                i += 2;
            }
            "--days" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<i64>() {
                    days = d;
                }
                i += 2;
            }
            "--secret" if i + 1 < args.len() => {
                secret_hex = args[i + 1].clone();
                i += 2;
            }
            "--verify" => {
                is_verify = true;
                i += 1;
            }
            "--key" if i + 1 < args.len() => {
                key_to_verify = Some(args[i + 1].clone());
                i += 2;
            }
            other => {
                eprintln!("Unknown or incomplete argument: {}", other);
                print_usage();
                return;
            }
        }
    }

    let email = match email {
        Some(e) if !e.trim().is_empty() => e.trim().to_string(),
        _ => {
            eprintln!("Error: --email is required.");
            print_usage();
            return;
        }
    };

    if is_verify {
        let key = match key_to_verify {
            Some(k) => k,
            None => {
                eprintln!("Error: --key is required when verifying.");
                return;
            }
        };

        match verify_key(&email, &key, DEFAULT_PUBLIC_KEY_HEX) {
            Ok(payload) => {
                println!("\nâœ… Licence Key is VALID!");
                println!("Email:      {}", payload.email);
                println!("Tier:       {}", payload.tier.to_uppercase());
                if let Some(exp) = payload.expires_at {
                    if let Ok(odt) = time::OffsetDateTime::from_unix_timestamp(exp) {
                        println!(
                            "Expires:    {}-{:02}-{:02} (UTC)",
                            odt.year(),
                            odt.month() as u8,
                            odt.day()
                        );
                    }
                } else {
                    println!("Expires:    Never (Lifetime)");
                }
            }
            Err(err) => {
                eprintln!("\nâ Œ Licence Key is INVALID: {}", err);
            }
        }
        return;
    }

    // Generate Key
    let secret_bytes = match hex_to_32_bytes(&secret_hex) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error parsing secret key: {}", e);
            return;
        }
    };

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = if tier == "lifetime" {
        None
    } else {
        Some(now + (days * 86400))
    };

    let payload = LicencePayload {
        email: email.clone(),
        tier: tier.clone(),
        issued_at: now,
        expires_at,
    };

    let payload_json = match serde_json::to_vec(&payload) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error serializing payload: {}", e);
            return;
        }
    };

    let signature = signing_key.sign(&payload_json);
    let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_json);
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());
    let licence_token = format!("EMB1.{}.{}", payload_b64, sig_b64);

    println!("\n========================================================");
    println!("        EMBROIDERY CATALOGUE LICENCE KEY");
    println!("========================================================");
    println!("Registered Email: {}", email);
    println!("Licence Tier:     {}", tier.to_uppercase());
    if let Some(exp) = expires_at {
        if let Ok(odt) = time::OffsetDateTime::from_unix_timestamp(exp) {
            println!(
                "Valid Until:      {}-{:02}-{:02} ({} days)",
                odt.year(),
                odt.month() as u8,
                odt.day(),
                days
            );
        }
    } else {
        println!("Valid Until:      Lifetime (No expiration)");
    }
    println!("\nLicence Key:\n{}", licence_token);
    println!("========================================================\n");
}
