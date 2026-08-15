-- Reverse the dual-verification migration: collapse the two independent
-- flags back into the original single `tags_checked` flag.
ALTER TABLE designs DROP COLUMN stitching_tags_verified;

ALTER TABLE designs RENAME COLUMN image_tags_verified TO tags_checked;