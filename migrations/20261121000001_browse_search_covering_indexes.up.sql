-- Rebuild browse folder-sort, filename-sort, and lowercase search indexes to
-- cover `filepath` and verification flags, so folder/file text searches
-- combined with verification filters stay entirely inside covering indexes
-- instead of falling back to table-heap scans over inline `image_data` BLOBs.
DROP INDEX IF EXISTS ix_designs_folder_asc;
DROP INDEX IF EXISTS ix_designs_folder_desc;
DROP INDEX IF EXISTS ix_designs_filename;
DROP INDEX IF EXISTS ix_designs_filepath_lower;
DROP INDEX IF EXISTS ix_designs_filename_lower;

CREATE INDEX IF NOT EXISTS ix_designs_folder_asc ON designs (
    SUBSTR(filepath, 1, LENGTH(filepath) - LENGTH(filename)) COLLATE NOCASE ASC,
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified,
    filepath
);

CREATE INDEX IF NOT EXISTS ix_designs_folder_desc ON designs (
    SUBSTR(filepath, 1, LENGTH(filepath) - LENGTH(filename)) COLLATE NOCASE DESC,
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified,
    filepath
);

CREATE INDEX IF NOT EXISTS ix_designs_filename ON designs (
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified,
    filepath
);

-- Compact 3-column covering indexes for COUNT(*) and text-search scans over
-- lowercase filepath/filename with verification filters.
CREATE INDEX IF NOT EXISTS ix_designs_filepath_lower ON designs (
    lower(filepath),
    image_tags_verified,
    stitching_tags_verified
);

CREATE INDEX IF NOT EXISTS ix_designs_filename_lower ON designs (
    lower(filename),
    image_tags_verified,
    stitching_tags_verified
);

-- Refresh sqlite_stat1 so the query planner picks the updated covering indexes.
ANALYZE designs;
