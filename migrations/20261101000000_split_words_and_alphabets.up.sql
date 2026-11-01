-- Split 'Words and Letters' into two distinct tags: 'Words' and 'Alphabets'.
-- 'Alphabets' is a protected system tag (is_system = 1) because the bulk import and compression
-- engine specifically orchestrates around alphabet packages, minimal glyph samples, and zip archives.
-- 'Words' is a standard thematic tag (is_system = 0) for standalone phrases, quotes, and text designs.

-- 1. Rename existing 'Words and Letters' tag to 'Words' (if present)
UPDATE tags
SET description = 'Words', is_system = 0
WHERE description = 'Words and Letters';

-- 2. Ensure 'Alphabets' exists and is marked as a protected system tag
INSERT OR IGNORE INTO tags (description, tag_group, is_system)
VALUES ('Alphabets', 'image', 1);

UPDATE tags
SET is_system = 1
WHERE description = 'Alphabets';

-- 3. Ensure 'Words' exists even if 'Words and Letters' wasn't present
INSERT OR IGNORE INTO tags (description, tag_group, is_system)
VALUES ('Words', 'image', 0);

-- 4. Seed synonyms for 'Alphabets'
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'alphabet', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'alphabets', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'font', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'fonts', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'monogram', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'monograms', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'initials', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'letter', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'letters', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'upper', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'lower', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'uppercase', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'lowercase', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'caps', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'number', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'numbers', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'digit', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'digits', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'numeral', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'numerals', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'punctuation', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'symbol', id FROM tags WHERE description = 'Alphabets';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'symbols', id FROM tags WHERE description = 'Alphabets';

-- 5. Clean up alphabet-specific keywords that previously pointed to 'Words' (or old 'Words and Letters')
DELETE FROM tag_synonyms
WHERE tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND keyword IN (
    'alphabet', 'alphabets', 'font', 'fonts', 'monogram', 'monograms',
    'upper', 'lower', 'uppercase', 'lowercase', 'caps', 'initials'
  );

-- 6. Seed synonyms for 'Words'
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'blackboard', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'bookmark', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'bookworm', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'text', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'wording', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'writing', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'saying', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'sayings', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'quote', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'quotes', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'phrase', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'phrases', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'sentiment', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'sentiments', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'verse', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'verses', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'slogan', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'slogans', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'motto', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'scripture', id FROM tags WHERE description = 'Words';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'proverb', id FROM tags WHERE description = 'Words';

-- 7. Dynamically reclassify existing design_tags from 'Words' to 'Alphabets'
-- for designs whose path or filename indicates an alphabet, font, or monogram package.
INSERT OR IGNORE INTO design_tags (design_id, tag_id)
SELECT dt.design_id, (SELECT id FROM tags WHERE description = 'Alphabets')
FROM design_tags dt
JOIN designs d ON d.id = dt.design_id
WHERE dt.tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND (
    lower(d.filepath) LIKE '%alphabet%'
    OR lower(d.filepath) LIKE '%font%'
    OR lower(d.filepath) LIKE '%monogram%'
    OR lower(d.filename) LIKE '%alphabet%'
    OR lower(d.filename) LIKE '%font%'
    OR lower(d.filename) LIKE '%monogram%'
  );

-- Remove the reclassified designs from 'Words'
DELETE FROM design_tags
WHERE tag_id = (SELECT id FROM tags WHERE description = 'Words')
  AND design_id IN (
    SELECT design_id FROM design_tags
    WHERE tag_id = (SELECT id FROM tags WHERE description = 'Alphabets')
  );

