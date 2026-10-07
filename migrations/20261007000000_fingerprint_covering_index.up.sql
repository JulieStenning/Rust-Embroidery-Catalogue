-- Covering index for import de-duplication queries on (file_size_bytes, filename COLLATE NOCASE, file_hash_blake3)
CREATE INDEX IF NOT EXISTS ix_designs_fingerprint_lookup ON designs (file_size_bytes, filename COLLATE NOCASE, file_hash_blake3);
