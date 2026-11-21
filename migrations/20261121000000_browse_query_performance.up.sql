-- Drop unused filepath sort indexes superseded by folder-expression covering indexes
DROP INDEX IF EXISTS ix_designs_filepath_nocase_asc;
DROP INDEX IF EXISTS ix_designs_filepath_nocase_desc;

-- Compact covering index for verification status counts (unverified, verified, partial)
-- and lightweight unfiltered COUNT(*) scans (avoids scanning large string indexes or the table heap)
CREATE INDEX IF NOT EXISTS ix_designs_verification_flags
    ON designs (image_tags_verified, stitching_tags_verified);

-- Folder-sort covering expression indexes matching browse_sort_clause("folder").
-- Trailing verification columns allow folder-sorted queries filtered by verification status
-- to satisfy LIMIT/OFFSET directly from the index without table heap lookups or temp B-trees.
CREATE INDEX IF NOT EXISTS ix_designs_folder_asc ON designs (
    SUBSTR(filepath, 1, LENGTH(filepath) - LENGTH(filename)) COLLATE NOCASE ASC,
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified
);
CREATE INDEX IF NOT EXISTS ix_designs_folder_desc ON designs (
    SUBSTR(filepath, 1, LENGTH(filepath) - LENGTH(filename)) COLLATE NOCASE DESC,
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified
);

-- Populate sqlite_stat1 so the SQLite query planner chooses compact covering indexes
ANALYZE designs;
