DROP INDEX IF EXISTS ix_designs_folder_desc;
DROP INDEX IF EXISTS ix_designs_folder_asc;
DROP INDEX IF EXISTS ix_designs_verification_flags;

CREATE INDEX IF NOT EXISTS ix_designs_filepath_nocase_asc
    ON designs (filepath COLLATE NOCASE ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_filepath_nocase_desc
    ON designs (filepath COLLATE NOCASE DESC, filename COLLATE NOCASE ASC, id ASC);
