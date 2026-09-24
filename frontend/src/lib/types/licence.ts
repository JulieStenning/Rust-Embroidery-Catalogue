// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

export interface LicenceStatus {
  is_active: boolean;
  is_valid: boolean;
  email: string | null;
  tier: string | null;
  expires_at: number | null;
  expires_at_formatted: string | null;
  error_message: string | null;
}
