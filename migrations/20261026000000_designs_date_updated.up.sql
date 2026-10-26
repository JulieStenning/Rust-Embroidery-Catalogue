-- Add date_updated column to track design modifications and support sorting by recently updated.
ALTER TABLE designs ADD COLUMN date_updated TEXT;

-- Backfill existing designs with date_added so date_updated is never null for previously imported designs.
UPDATE designs SET date_updated = date_added WHERE date_added IS NOT NULL;

-- Compound expression indexes to accelerate sorting by date_updated with date_added fallback
CREATE INDEX IF NOT EXISTS ix_designs_date_updated_asc ON designs (COALESCE(date_updated, date_added, '') ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_date_updated_desc ON designs (COALESCE(date_updated, date_added, '') DESC, filename COLLATE NOCASE ASC, id ASC);
