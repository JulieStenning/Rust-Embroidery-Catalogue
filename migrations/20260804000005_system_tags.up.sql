-- Add an is_system flag to the tags table so the app can identify system-defined
-- stitching tags that must not be renamed, re-grouped, or deleted by users.

ALTER TABLE tags ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT 0;

-- Mark the system-defined stitching tags seeded by the initial migration.
-- (Cross Stitch, In The Hoop, Filled, Cutwork, Line Outline, Satin Stitch, Applique, Lace)
UPDATE tags SET is_system = 1 WHERE id IN (1, 2, 3, 7, 9, 10, 11, 14);