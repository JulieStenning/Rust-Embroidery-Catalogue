-- Add master file pairing columns to designs table
ALTER TABLE designs ADD COLUMN master_filepath VARCHAR(1000);
ALTER TABLE designs ADD COLUMN is_master_only BOOLEAN NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS ix_designs_master_filepath ON designs(master_filepath);
CREATE INDEX IF NOT EXISTS ix_designs_is_master_only ON designs(is_master_only);
