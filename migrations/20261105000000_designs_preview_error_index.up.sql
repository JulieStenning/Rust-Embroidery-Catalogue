-- SPDX-FileCopyrightText: 2026 Julie Stenning
-- SPDX-License-Identifier: GPL-3.0-or-later

-- Partial index for instantaneous missing-preview/exceeds-hoop counts & filters
CREATE INDEX IF NOT EXISTS ix_designs_preview_error ON designs (preview_error) WHERE preview_error IS NOT NULL;

