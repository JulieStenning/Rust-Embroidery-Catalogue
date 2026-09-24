-- SQLx reversible migration: tag_synonyms table for user-defined and seed word matches

CREATE TABLE IF NOT EXISTS tag_synonyms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword VARCHAR(100) NOT NULL COLLATE NOCASE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    CONSTRAINT uq_keyword_tag UNIQUE(keyword, tag_id)
);
CREATE INDEX IF NOT EXISTS ix_tag_synonyms_keyword ON tag_synonyms(keyword);
CREATE INDEX IF NOT EXISTS ix_tag_synonyms_tag_id ON tag_synonyms(tag_id);

-- Seed default word matches
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'kitten', id FROM tags WHERE description = 'Cats';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'kitty', id FROM tags WHERE description = 'Cats';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'puppy', id FROM tags WHERE description = 'Dogs';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'hound', id FROM tags WHERE description = 'Dogs';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'frog', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'bunny', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'bear', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'deer', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'elephant', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'fox', id FROM tags WHERE description = 'Animals';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'owl', id FROM tags WHERE description = 'Birds';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'floral', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'rose', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'tulip', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'daisy', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'sunflower', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'orchid', id FROM tags WHERE description = 'Flowers';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'blossom', id FROM tags WHERE description = 'Flowers';

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

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'xmas', id FROM tags WHERE description = 'Christmas';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'santa', id FROM tags WHERE description = 'Christmas';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'noel', id FROM tags WHERE description = 'Christmas';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'reindeer', id FROM tags WHERE description = 'Christmas';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'snowman', id FROM tags WHERE description = 'Christmas';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'fsl', id FROM tags WHERE description = 'Lace';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'ith', id FROM tags WHERE description = 'In The Hoop';

INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'baby', id FROM tags WHERE description = 'Babies';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'toddler', id FROM tags WHERE description = 'Children';
INSERT OR IGNORE INTO tag_synonyms (keyword, tag_id)
SELECT 'nursery', id FROM tags WHERE description = 'Babies';
