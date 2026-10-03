-- Revert master file pairing columns
DROP INDEX IF EXISTS ix_designs_master_filepath;
DROP INDEX IF EXISTS ix_designs_is_master_only;

ALTER TABLE designs DROP COLUMN master_filepath;
ALTER TABLE designs DROP COLUMN is_master_only;
