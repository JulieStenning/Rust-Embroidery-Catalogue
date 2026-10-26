DROP INDEX IF EXISTS ix_designs_date_updated_asc;
DROP INDEX IF EXISTS ix_designs_date_updated_desc;
ALTER TABLE designs DROP COLUMN date_updated;
