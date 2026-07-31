CREATE UNIQUE INDEX IF NOT EXISTS ix_designers_name_ci
ON designers (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ix_sources_name_ci
ON sources (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ix_hoops_name_ci
ON hoops (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ix_tags_description_ci
ON tags (lower(description));