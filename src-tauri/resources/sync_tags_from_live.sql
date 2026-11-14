-- SPDX-FileCopyrightText: 2026 Julie Stenning
-- SPDX-License-Identifier: GPL-3.0-or-later
--
-- Manual SQL script to synchronise starter tags and tag_synonyms in the bundled seed database
-- (src-tauri/resources/EmbroideryCatalogue.db) from the live database (f:/Database/EmbroideryCatalogue.db).

ATTACH DATABASE 'f:/Database/EmbroideryCatalogue.db' AS live;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

-- Remove existing starter tags and synonyms from seed database
DELETE FROM tag_synonyms;
DELETE FROM tags;

-- Copy exact tags from live database preserving IDs, groups, and system flags
INSERT INTO tags (id, description, tag_group, is_system)
SELECT id, description, tag_group, is_system FROM live.tags;

-- Copy exact tag synonyms from live database preserving IDs and tag foreign keys
INSERT INTO tag_synonyms (id, keyword, tag_id)
SELECT id, keyword, tag_id FROM live.tag_synonyms;

-- Synchronise auto-increment sequence counters
DELETE FROM sqlite_sequence WHERE name IN ('tags', 'tag_synonyms');
INSERT INTO sqlite_sequence (name, seq)
VALUES
    ('tags', (SELECT MAX(id) FROM tags)),
    ('tag_synonyms', (SELECT MAX(id) FROM tag_synonyms));

COMMIT;

PRAGMA foreign_keys = ON;
PRAGMA foreign_key_check;

DETACH DATABASE live;

-- Defragment database and refresh query planner statistics
VACUUM;
PRAGMA optimize;

