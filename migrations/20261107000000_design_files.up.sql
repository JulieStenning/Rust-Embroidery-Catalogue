-- Create design_files table (3NF)
CREATE TABLE IF NOT EXISTS design_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    design_id INTEGER NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
    filepath TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    extension TEXT NOT NULL,
    file_type TEXT NOT NULL, -- 'stitch' or 'master'
    is_primary INTEGER NOT NULL DEFAULT 0,
    stitch_count INTEGER,
    width_mm REAL,
    height_mm REAL,
    color_count INTEGER,
    color_change_count INTEGER,
    file_size_bytes INTEGER,
    file_hash_blake3 TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS ix_design_files_design_id ON design_files(design_id);
CREATE INDEX IF NOT EXISTS ix_design_files_filepath ON design_files(filepath);
CREATE INDEX IF NOT EXISTS ix_design_files_extension ON design_files(extension);
CREATE INDEX IF NOT EXISTS ix_design_files_design_primary ON design_files(design_id, is_primary);

-- Backfill from existing designs:
-- 1. Insert primary stitch files for designs with is_master_only = 0
INSERT OR IGNORE INTO design_files (
    design_id,
    filepath,
    filename,
    extension,
    file_type,
    is_primary,
    stitch_count,
    width_mm,
    height_mm,
    color_count,
    color_change_count,
    file_size_bytes,
    file_hash_blake3
)
SELECT
    d.id,
    d.filepath,
    d.filename,
    CASE
        WHEN LOWER(d.filepath) LIKE '%.pes' THEN 'pes'
        WHEN LOWER(d.filepath) LIKE '%.jef' THEN 'jef'
        WHEN LOWER(d.filepath) LIKE '%.hus' THEN 'hus'
        WHEN LOWER(d.filepath) LIKE '%.dst' THEN 'dst'
        WHEN LOWER(d.filepath) LIKE '%.exp' THEN 'exp'
        WHEN LOWER(d.filepath) LIKE '%.vp3' THEN 'vp3'
        WHEN LOWER(d.filepath) LIKE '%.eof' THEN 'eof'
        WHEN LOWER(d.filepath) LIKE '%.ecf' THEN 'ecf'
        WHEN LOWER(d.filepath) LIKE '%.art' THEN 'art'
        WHEN LOWER(d.filepath) LIKE '%.emb' THEN 'emb'
        WHEN LOWER(d.filepath) LIKE '%.jan' THEN 'jan'
        WHEN LOWER(d.filepath) LIKE '%.be'  THEN 'be'
        ELSE ''
    END,
    'stitch',
    1,
    d.stitch_count,
    d.width_mm,
    d.height_mm,
    d.color_count,
    d.color_change_count,
    d.file_size_bytes,
    d.file_hash_blake3
FROM designs d
WHERE COALESCE(d.is_master_only, 0) = 0
  AND d.filepath IS NOT NULL
  AND TRIM(d.filepath) != '';

-- 2. Insert master files for designs with is_master_only = 1
INSERT OR IGNORE INTO design_files (
    design_id,
    filepath,
    filename,
    extension,
    file_type,
    is_primary,
    stitch_count,
    width_mm,
    height_mm,
    color_count,
    color_change_count,
    file_size_bytes,
    file_hash_blake3
)
SELECT
    d.id,
    COALESCE(NULLIF(d.master_filepath, ''), d.filepath),
    d.filename,
    CASE
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.eof' THEN 'eof'
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.ecf' THEN 'ecf'
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.art' THEN 'art'
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.emb' THEN 'emb'
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.jan' THEN 'jan'
        WHEN LOWER(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) LIKE '%.be'  THEN 'be'
        ELSE ''
    END,
    'master',
    1,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    d.file_size_bytes,
    d.file_hash_blake3
FROM designs d
WHERE COALESCE(d.is_master_only, 0) = 1
  AND COALESCE(NULLIF(d.master_filepath, ''), d.filepath) IS NOT NULL
  AND TRIM(COALESCE(NULLIF(d.master_filepath, ''), d.filepath)) != '';

-- 3. Insert paired master files for designs with is_master_only = 0 where master_filepath is populated
INSERT OR IGNORE INTO design_files (
    design_id,
    filepath,
    filename,
    extension,
    file_type,
    is_primary,
    file_size_bytes,
    file_hash_blake3
)
SELECT
    d.id,
    d.master_filepath,
    d.master_filepath,
    CASE
        WHEN LOWER(d.master_filepath) LIKE '%.eof' THEN 'eof'
        WHEN LOWER(d.master_filepath) LIKE '%.ecf' THEN 'ecf'
        WHEN LOWER(d.master_filepath) LIKE '%.art' THEN 'art'
        WHEN LOWER(d.master_filepath) LIKE '%.emb' THEN 'emb'
        WHEN LOWER(d.master_filepath) LIKE '%.jan' THEN 'jan'
        WHEN LOWER(d.master_filepath) LIKE '%.be'  THEN 'be'
        ELSE ''
    END,
    'master',
    0,
    NULL,
    NULL
FROM designs d
WHERE COALESCE(d.is_master_only, 0) = 0
  AND d.master_filepath IS NOT NULL
  AND TRIM(d.master_filepath) != ''
  AND d.master_filepath != d.filepath;

