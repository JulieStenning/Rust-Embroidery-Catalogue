-- Remove Tier-2 "Text AI" from Tagging Actions.
--
-- Text AI (Gemini on the file name/folder) has been retired; only File & Folder
-- Rules (offline, Tier 1) and Visual AI (Gemini vision on the thumbnail) remain.
-- Drop the per-design Text AI outcome columns and their index, which were added
-- by migration 20260901000009. The Vision AI columns and index are retained.

DROP INDEX IF EXISTS idx_designs_text_ai;

ALTER TABLE designs DROP COLUMN text_ai_matched;
ALTER TABLE designs DROP COLUMN text_ai_analyzed;
