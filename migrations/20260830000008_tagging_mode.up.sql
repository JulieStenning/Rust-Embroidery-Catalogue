-- Replace the numeric tagging tier with explicit two-mode tagging.
--
-- Previously a design recorded which of three Gemini tiers produced its image
-- tags as a small integer (`tagging_tier`): 1 = Tier 1 keyword matching,
-- 2 = Tier 2 text AI, 3 = Tier 3 vision AI. Tier 2 (filename/text Gemini) has
-- been removed and the remaining modes renamed:
--   - "File & Folder Rules"  (path_rule) — local filename/path matching (was Tier 1)
--   - "Visual AI"            (ai_vision)  — Gemini vision on the thumbnail (was Tier 3)
--
-- The old numeric column is replaced by a text `tagging_mode` column storing the
-- stable wire ids. Existing rows are mapped: 1 -> 'path_rule', 2 -> 'path_rule'
-- (Tier 2 was filename-derived, so it maps to File & Folder Rules), 3 -> 'ai_vision'.

ALTER TABLE designs ADD COLUMN tagging_mode TEXT;

UPDATE designs SET tagging_mode = 'path_rule' WHERE tagging_tier = 1;
UPDATE designs SET tagging_mode = 'path_rule' WHERE tagging_tier = 2;
UPDATE designs SET tagging_mode = 'ai_vision'  WHERE tagging_tier = 3;

ALTER TABLE designs DROP COLUMN tagging_tier;
