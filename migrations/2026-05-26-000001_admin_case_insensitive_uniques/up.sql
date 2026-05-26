CREATE UNIQUE INDEX IF NOT EXISTS ux_designers_name_ci
ON designers (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ux_sources_name_ci
ON sources (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ux_hoops_name_ci
ON hoops (lower(name));

CREATE UNIQUE INDEX IF NOT EXISTS ux_tags_description_ci
ON tags (lower(description));