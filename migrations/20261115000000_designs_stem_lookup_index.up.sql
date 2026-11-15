-- SPDX-FileCopyrightText: 2026 Julie Stenning
-- SPDX-License-Identifier: GPL-3.0-or-later

-- Covering index on designs for counterpart pairing stem lookup and import precheck scanning.
-- Avoids full sequential table scans of 20+ GB tables containing inline image_data BLOBs.
CREATE INDEX IF NOT EXISTS ix_designs_stem_lookup ON designs (filepath, master_filepath, is_master_only);
