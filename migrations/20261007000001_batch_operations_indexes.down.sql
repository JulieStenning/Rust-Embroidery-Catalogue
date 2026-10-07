-- Revert batch operations indexes

DROP INDEX IF EXISTS ix_tags_tag_group_lower;
DROP INDEX IF EXISTS ix_design_tags_design_id_tag_id;
DROP INDEX IF EXISTS ix_designs_vision_ai_unverified;
DROP INDEX IF EXISTS ix_designs_stitching_tags_verified;
DROP INDEX IF EXISTS ix_designs_image_tags_verified;
DROP INDEX IF EXISTS ix_designs_image_data_null;
