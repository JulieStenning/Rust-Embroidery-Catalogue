-- Accelerate batch operations missing-preview counts, scope calculations, and tag-group lookups

-- Partial index for instantaneous missing-preview counts & filters
CREATE INDEX IF NOT EXISTS ix_designs_image_data_null ON designs (id) WHERE image_data IS NULL;

-- Expression indexes for batch tagging scope counts & keyset pagination
CREATE INDEX IF NOT EXISTS ix_designs_image_tags_verified ON designs (COALESCE(image_tags_verified, 0), id);
CREATE INDEX IF NOT EXISTS ix_designs_stitching_tags_verified ON designs (COALESCE(stitching_tags_verified, 0), id);

CREATE INDEX IF NOT EXISTS ix_designs_vision_ai_unverified ON designs (
    COALESCE(vision_ai_analyzed, 0),
    COALESCE(image_tags_verified, 0),
    COALESCE(vision_ai_matched, 0),
    id
);

-- Covering index on design_tags and tag_group expression index for fast NOT EXISTS / category joins
CREATE INDEX IF NOT EXISTS ix_design_tags_design_id_tag_id ON design_tags (design_id, tag_id);
CREATE INDEX IF NOT EXISTS ix_tags_tag_group_lower ON tags (lower(COALESCE(tag_group, '')));
