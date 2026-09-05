-- Reverse of the Text AI removal: restore the per-design Text AI outcome columns
-- and index, mirroring migration 20260901000009.

ALTER TABLE designs ADD COLUMN text_ai_analyzed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE designs ADD COLUMN text_ai_matched INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_designs_text_ai ON designs (text_ai_analyzed, text_ai_matched);
