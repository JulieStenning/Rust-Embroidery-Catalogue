-- Add date_updated column to track design modifications and support sorting by recently updated.
ALTER TABLE designs ADD COLUMN date_updated DATE;

-- Compound expression indexes to accelerate sorting by date_updated with date_added fallback.
-- Existing designs without date_updated naturally fall back to date_added via COALESCE,
-- avoiding an expensive batch UPDATE of hundreds of thousands of rows on large catalogues.
CREATE INDEX IF NOT EXISTS ix_designs_date_updated_asc ON designs (COALESCE(date_updated, date_added, '') ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_date_updated_desc ON designs (COALESCE(date_updated, date_added, '') DESC, filename COLLATE NOCASE ASC, id ASC);
