// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type { LicenceStatus } from "../types/licence";

/**
 * Fetch the current licence activation status.
 */
export async function getLicenceStatus(): Promise<LicenceStatus> {
  return invokeLoose<LicenceStatus>("get_licence_status");
}

/**
 * Activate the application with an email and cryptographic licence key.
 *
 * @param email - The registered email address
 * @param licenceKey - The licence key token (e.g. EMB1.<payload>.<sig>)
 */
export async function activateLicence(email: string, licenceKey: string): Promise<LicenceStatus> {
  return invokeLoose<LicenceStatus>("activate_licence", {
    email,
    licenceKey,
  });
}

/**
 * Deactivate and remove the currently stored licence.
 */
export async function deactivateLicence(): Promise<LicenceStatus> {
  return invokeLoose<LicenceStatus>("deactivate_licence");
}
