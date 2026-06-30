CREATE INDEX ix_design_tags_tag_id ON design_tags (tag_id);

CREATE INDEX ix_design_tags_design_id ON design_tags (design_id);

CREATE INDEX ix_designers_name ON designers (name);

CREATE INDEX ix_designs_date_added ON designs (date_added);

CREATE INDEX ix_designs_designer_id ON designs (designer_id);

CREATE INDEX ix_designs_rating ON designs (rating);

CREATE INDEX ix_designs_filename ON designs (filename);

CREATE INDEX ix_designs_filepath ON designs (filepath);

CREATE INDEX ix_designs_source_id ON designs (source_id);

CREATE INDEX ix_project_designs_design_id ON project_designs (design_id);

CREATE INDEX ix_project_designs_project_id ON project_designs (project_id);

