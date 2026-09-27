-- Add explicit indexes on tags(id) and tags(description)
CREATE INDEX IF NOT EXISTS ix_tags_id ON tags (id);
CREATE INDEX IF NOT EXISTS ix_tags_description ON tags (description);
