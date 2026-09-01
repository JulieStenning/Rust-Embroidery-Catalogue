-- Restore the single tagging_mode column and remove the per-mode AI columns.
ALTER TABLE designs ADD COLUMN tagging_mode TEXT;

DROP INDEX idx_designs_vision_ai;
DROP INDEX idx_designs_text_ai;

ALTER TABLE designs DROP COLUMN vision_ai_matched;
ALTER TABLE designs DROP COLUMN vision_ai_analyzed;
ALTER TABLE designs DROP COLUMN text_ai_matched;
ALTER TABLE designs DROP COLUMN text_ai_analyzed;
