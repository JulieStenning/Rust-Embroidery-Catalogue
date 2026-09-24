// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import crypto from "node:crypto";
import { DatabaseSync } from "node:sqlite";

const DEFAULT_PRIVATE_KEY_HEX =
  "0000110f1c41bd48bdeef29e1aa87870e8fc58ff09261da6aea07e8124994c73";

// PKCS#8 DER header for Ed25519 (RFC 8410: 16 bytes prefix + 32 bytes seed)
const ED25519_PKCS8_PREFIX = Buffer.from(
  "302e020100300506032b657004220420",
  "hex",
);

export interface GeneratedLicence {
  email: string;
  tier: string;
  issuedAt: number;
  expiresAt: number | null;
  licenceKey: string;
}

/**
 * Generate a cryptographically valid licence key signed with the default test keypair.
 */
export function generateTestLicence(
  email: string,
  tier: string = "beta",
  validDays: number | null = 90,
): GeneratedLicence {
  const seed = Buffer.from(DEFAULT_PRIVATE_KEY_HEX, "hex");
  const pkcs8 = Buffer.concat([ED25519_PKCS8_PREFIX, seed]);
  const privateKey = crypto.createPrivateKey({
    key: pkcs8,
    format: "der",
    type: "pkcs8",
  });

  const nowSecs = Math.floor(Date.now() / 1000);
  const expiresAt =
    validDays !== null && validDays > 0 ? nowSecs + validDays * 86400 : null;

  const payloadObj = {
    email: email.trim().toLowerCase(),
    tier,
    issued_at: nowSecs,
    expires_at: expiresAt,
  };

  const payloadJson = JSON.stringify(payloadObj);
  const signature = crypto.sign(null, Buffer.from(payloadJson, "utf-8"), privateKey);

  const payloadB64 = Buffer.from(payloadJson, "utf-8").toString("base64url");
  const sigB64 = signature.toString("base64url");
  const licenceKey = `EMB1.${payloadB64}.${sigB64}`;

  return {
    email: payloadObj.email,
    tier,
    issuedAt: nowSecs,
    expiresAt,
    licenceKey,
  };
}

/**
 * Seed an active, valid licence into the SQLite database so the app boots past the licence gate.
 */
export function seedLicence(
  databasePath: string,
  email: string = "tester@example.com",
  tier: string = "beta",
  validDays: number | null = 90,
): GeneratedLicence {
  const licence = generateTestLicence(email, tier, validDays);
  const db = new DatabaseSync(databasePath);
  try {
    const upsert = (key: string, value: string) => {
      db.exec(`DELETE FROM settings WHERE key = '${key}'`);
      db.exec(
        `INSERT INTO settings (key, value) VALUES ('${key}', '${value}')`,
      );
    };

    upsert("licence_email", licence.email);
    upsert("licence_key", licence.licenceKey);
    upsert("licence_tier", licence.tier);
    if (licence.expiresAt !== null) {
      upsert("licence_expires_at", licence.expiresAt.toString());
    } else {
      db.exec("DELETE FROM settings WHERE key = 'licence_expires_at'");
    }
  } finally {
    db.close();
  }

  return licence;
}

/**
 * Remove licence records from the SQLite database to simulate an unactivated fresh install.
 */
export function clearLicenceFromDb(databasePath: string): void {
  const db = new DatabaseSync(databasePath);
  try {
    db.exec(
      "DELETE FROM settings WHERE key IN ('licence_email', 'licence_key', 'licence_tier', 'licence_expires_at')",
    );
  } finally {
    db.close();
  }
}
