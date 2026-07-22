-- Add content fingerprinting columns to designs for BLAKE3 hash + file size dedup.
ALTER TABLE designs ADD COLUMN file_size_bytes INTEGER;
ALTER TABLE designs ADD COLUMN file_hash_blake3 TEXT;

-- Composite index for efficient hash+size dedup lookups.
CREATE INDEX ix_designs_file_hash_blake3_file_size_bytes
    ON designs (file_hash_blake3, file_size_bytes);

-- Index on file_size_bytes alone to accelerate size-only pre-filter queries.
CREATE INDEX ix_designs_file_size_bytes ON designs (file_size_bytes);