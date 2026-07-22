-- Reverse content fingerprinting columns from designs.
DROP INDEX IF EXISTS ix_designs_file_hash_blake3_file_size_bytes;
DROP INDEX IF EXISTS ix_designs_file_size_bytes;

ALTER TABLE designs DROP COLUMN file_size_bytes;
ALTER TABLE designs DROP COLUMN file_hash_blake3;