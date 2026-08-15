-- Split the single verification flag into independent image/stitching flags.
--
-- `tags_checked` historically meant "all tags verified" in one shot. Because
-- image tags (what a design depicts) come from AI vision analysis and are
-- edited independently from stitching tags (how a design sews, derived from
-- binary file parsing), a single flag is insufficient. We promote the old
-- flag to the image-domain (the primary workflow that set it) and add a
-- parallel stitching-domain flag.

ALTER TABLE designs RENAME COLUMN tags_checked TO image_tags_verified;

ALTER TABLE designs ADD COLUMN
    stitching_tags_verified BOOLEAN NOT NULL DEFAULT 0;

-- Designs that were previously verified become verified in both domains;
-- designs that were unverified remain unverified in both.
UPDATE designs
SET stitching_tags_verified = image_tags_verified;