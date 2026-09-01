-- Track per-mode AI analysis outcome for the three-tier tagging model.
--
--   Tier 1 (offline path/file rules) runs at import and needs no API tracking.
--   Tier 2 (Text AI) and Tier 3 (Vision AI) record whether each mode ran to a
--   conclusion (`*_ai_analyzed`) and whether it produced at least one tag
--   (`*_ai_matched`). These drive the "missing <mode> analysis", "no-match"
--   and "re-analyze" scopes. The obsolete single `tagging_mode` column is
--   dropped; the per-mode `matched` flags supersede it.

ALTER TABLE designs ADD COLUMN text_ai_analyzed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE designs ADD COLUMN text_ai_matched INTEGER NOT NULL DEFAULT 0;
ALTER TABLE designs ADD COLUMN vision_ai_analyzed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE designs ADD COLUMN vision_ai_matched INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_designs_text_ai ON designs (text_ai_analyzed, text_ai_matched);
CREATE INDEX idx_designs_vision_ai ON designs (vision_ai_analyzed, vision_ai_matched);

ALTER TABLE designs DROP COLUMN tagging_mode;
