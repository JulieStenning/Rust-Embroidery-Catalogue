-- Reverse the two-mode tagging migration: collapse `tagging_mode` back into the
-- numeric `tagging_tier` (1 = File & Folder Rules, 3 = Visual AI).
ALTER TABLE designs ADD COLUMN tagging_tier SMALLINT;

UPDATE designs SET tagging_tier = 1 WHERE tagging_mode = 'path_rule';
UPDATE designs SET tagging_tier = 3 WHERE tagging_mode = 'ai_vision';

ALTER TABLE designs DROP COLUMN tagging_mode;
