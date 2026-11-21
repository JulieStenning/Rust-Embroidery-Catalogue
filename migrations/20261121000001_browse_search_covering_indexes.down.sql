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
    stitching_tags_verified
);

CREATE INDEX IF NOT EXISTS ix_designs_folder_desc ON designs (
    SUBSTR(filepath, 1, LENGTH(filepath) - LENGTH(filename)) COLLATE NOCASE DESC,
    filename COLLATE NOCASE ASC,
    id ASC,
    image_tags_verified,
    stitching_tags_verified
);

CREATE INDEX IF NOT EXISTS ix_designs_filename ON designs (
    filename COLLATE NOCASE
);

CREATE INDEX IF NOT EXISTS ix_designs_filepath_lower ON designs (
    lower(filepath)
);

CREATE INDEX IF NOT EXISTS ix_designs_filename_lower ON designs (
    lower(filename)
);

ANALYZE designs;
