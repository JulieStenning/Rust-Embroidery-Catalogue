-- Mark the system-defined stitching tags seeded by the initial migration.
-- The `is_system` column itself is created by the initial migration
-- (migration 1's CREATE TABLE for tags), so re-adding it here would fail
-- with "duplicate column name" on fresh databases.
-- (Cross Stitch, In The Hoop, Filled, Cutwork, Line Outline, Satin Stitch, Applique, Lace)
UPDATE tags SET is_system = 1 WHERE id IN (1, 2, 3, 7, 9, 10, 11, 14);
