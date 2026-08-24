-- Browse pagination indexes.
-- Replace the BINARY filename index with a NOCASE one so that the default
-- browse sort `ORDER BY d.filename COLLATE NOCASE` can use an index.
DROP INDEX IF EXISTS ix_designs_filename;
CREATE INDEX ix_designs_filename ON designs (filename COLLATE NOCASE);

-- FK join coverage for the browse query (hoop_id previously had no index).
CREATE INDEX IF NOT EXISTS ix_designs_hoop_id ON designs (hoop_id);
