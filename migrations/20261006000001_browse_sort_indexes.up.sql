-- Compound expression indexes to accelerate browse sort options and prevent full-table scan temporary B-trees
CREATE INDEX IF NOT EXISTS ix_designs_date_added_asc ON designs (COALESCE(date_added, '') ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_date_added_desc ON designs (COALESCE(date_added, '') DESC, filename COLLATE NOCASE ASC, id ASC);

CREATE INDEX IF NOT EXISTS ix_designs_filepath_nocase_asc ON designs (filepath COLLATE NOCASE ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_filepath_nocase_desc ON designs (filepath COLLATE NOCASE DESC, filename COLLATE NOCASE ASC, id ASC);

CREATE INDEX IF NOT EXISTS ix_designs_rating_asc ON designs (COALESCE(rating, -1) ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_rating_desc ON designs (COALESCE(rating, -1) DESC, filename COLLATE NOCASE ASC, id ASC);

CREATE INDEX IF NOT EXISTS ix_designs_stitched_asc ON designs (is_stitched ASC, filename COLLATE NOCASE ASC, id ASC);
CREATE INDEX IF NOT EXISTS ix_designs_stitched_desc ON designs (is_stitched DESC, filename COLLATE NOCASE ASC, id ASC);

-- Expression index to accelerate case-insensitive tag description lookups and search subqueries
CREATE INDEX IF NOT EXISTS ix_tags_description_lower ON tags (lower(description));
