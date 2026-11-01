-- Rollback: restore 'Words and Letters' from 'Words' and 'Alphabets'

-- Reassign design_tags from Alphabets back to Words
INSERT OR IGNORE INTO design_tags (design_id, tag_id)
SELECT dt.design_id, (SELECT id FROM tags WHERE description = 'Words')
FROM design_tags dt
WHERE dt.tag_id = (SELECT id FROM tags WHERE description = 'Alphabets');

-- Delete Alphabets tag and its synonyms
DELETE FROM tags WHERE description = 'Alphabets';

-- Rename Words back to Words and Letters
UPDATE tags
SET description = 'Words and Letters', is_system = 0
WHERE description = 'Words';

-- Restore original alphabet synonyms on Words and Letters
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'alphabet', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'font', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'monogram', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'upper', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'lower', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'uppercase', id FROM tags WHERE description = 'Words and Letters';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'lowercase', id FROM tags WHERE description = 'Words and Letters';

