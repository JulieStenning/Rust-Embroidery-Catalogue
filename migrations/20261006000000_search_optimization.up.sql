-- Expression indexes on lower(filename) and lower(filepath) to accelerate case-insensitive queries and search filters
CREATE INDEX IF NOT EXISTS ix_designs_filename_lower ON designs (lower(filename));
CREATE INDEX IF NOT EXISTS ix_designs_filepath_lower ON designs (lower(filepath));
