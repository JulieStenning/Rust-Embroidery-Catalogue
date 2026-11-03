-- SQLx reversible migration: prune spurious tag synonyms
-- 1. Remove generic embroidery/stitch medium terms that cause rampant false positives across libraries.
DELETE FROM tag_synonyms
WHERE lower(keyword) IN (
    'embroidery', 'embroidered', 'embroideries',
    'stitch', 'stitched', 'stitches', 'stitching'
);

-- 2. Remove problematic generic words mapped to Sewing (e.g. 'plain', 'tape').
DELETE FROM tag_synonyms
WHERE tag_id IN (SELECT id FROM tags WHERE description = 'Sewing')
  AND lower(keyword) IN ('plain', 'tape');

-- 3. Remove plural duplicates where the singular keyword is already present for the same tag.
DELETE FROM tag_synonyms
WHERE id IN (
    SELECT ts_plural.id
    FROM tag_synonyms ts_plural
    JOIN tag_synonyms ts_singular
      ON ts_plural.tag_id = ts_singular.tag_id
     AND (
         lower(ts_plural.keyword) = lower(ts_singular.keyword) || 's'
         OR lower(ts_plural.keyword) = lower(ts_singular.keyword) || 'es'
     )
);

-- 4. Remove self-synonyms where keyword duplicates the tag's own name.
DELETE FROM tag_synonyms
WHERE id IN (
    SELECT ts.id
    FROM tag_synonyms ts
    JOIN tags t ON t.id = ts.tag_id
    WHERE lower(ts.keyword) = lower(t.description)
);
