-- Reverse the system-tags migration: drop the is_system column.
ALTER TABLE tags DROP COLUMN is_system;