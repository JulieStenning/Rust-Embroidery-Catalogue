-- Revert browse pagination indexes.
DROP INDEX IF EXISTS ix_designs_hoop_id;
DROP INDEX IF EXISTS ix_designs_filename;
CREATE INDEX ix_designs_filename ON designs (filename);
