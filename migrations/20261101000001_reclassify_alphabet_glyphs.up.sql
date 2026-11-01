-- Seed additional punctuation and symbol synonyms for 'Alphabets'
-- and reclassify any punctuation glyphs and their folder siblings from 'Words' to 'Alphabets'.

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'ampersand', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'apostrophe', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'asterisk', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'at', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'backslash', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'brace', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'bracket', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'caret', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'colon', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'comma', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'dash', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'dollar', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'dot', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'equal', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'equals', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'exclamation', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'exclam', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'hash', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'hyphen', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'minus', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'paren', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'parenthesis', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'percent', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'period', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'plus', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'pound', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'question', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'quote', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'quotes', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'semicolon', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'slash', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'star', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'tilde', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'underscore', id FROM tags WHERE description = 'Alphabets';

-- 1. Reclassify designs whose filename or path indicates punctuation or glyph tokens
INSERT OR IGNORE INTO design_tags (design_id, tag_id)
SELECT dt.design_id, (SELECT id FROM tags WHERE description = 'Alphabets')
FROM design_tags dt
JOIN designs d ON d.id = dt.design_id
WHERE dt.tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND (
    lower(d.filename) LIKE '%ampersand%'
    OR lower(d.filename) LIKE '%exclamation%'
    OR lower(d.filename) LIKE '%question%'
    OR lower(d.filename) LIKE '%colon%'
    OR lower(d.filename) LIKE '%comma%'
    OR lower(d.filename) LIKE '%period%'
    OR lower(d.filename) LIKE '%dash%'
    OR lower(d.filename) LIKE '%hyphen%'
    OR lower(d.filename) LIKE '%minus%'
    OR lower(d.filename) LIKE '%plus%'
    OR lower(d.filename) LIKE '%slash%'
    OR lower(d.filename) LIKE '%backslash%'
    OR lower(d.filename) LIKE '%paren%'
    OR lower(d.filename) LIKE '%bracket%'
    OR lower(d.filename) LIKE '%quote%'
    OR lower(d.filename) LIKE '%percent%'
    OR lower(d.filename) LIKE '%dollar%'
    OR lower(d.filename) LIKE '%asterisk%'
    OR lower(d.filename) LIKE '%tilde%'
    OR lower(d.filename) LIKE '%number%'
    OR lower(d.filename) LIKE '%digit%'
    OR lower(d.filename) LIKE '%numeral%'
    OR lower(d.filename) LIKE '%upper%'
    OR lower(d.filename) LIKE '%lower%'
    OR lower(d.filename) LIKE '%letter%'
    OR lower(d.filename) LIKE '%initial%'
  );

-- 2. Also reclassify any sibling designs in the same folder as an Alphabet design
INSERT OR IGNORE INTO design_tags (design_id, tag_id)
SELECT dt.design_id, (SELECT id FROM tags WHERE description = 'Alphabets')
FROM design_tags dt
JOIN designs d ON d.id = dt.design_id
WHERE dt.tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND EXISTS (
    SELECT 1 FROM designs d2
    JOIN design_tags dt2 ON dt2.design_id = d2.id
    WHERE dt2.tag_id = (SELECT id FROM tags WHERE description = 'Alphabets')
      AND d2.filepath = d.filepath
  );

-- 3. Remove reclassified designs from Words
DELETE FROM design_tags
WHERE tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND design_id IN (
    SELECT design_id FROM design_tags
    WHERE tag_id = (SELECT id FROM tags WHERE description = 'Alphabets')
  );

