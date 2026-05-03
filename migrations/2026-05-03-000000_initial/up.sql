-- Diesel migration: initial schema for Rust-Embroidery-Catalogue

CREATE TABLE designers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE hoops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE,
    max_width_mm NUMERIC(8,2) NOT NULL,
    max_height_mm NUMERIC(8,2) NOT NULL
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    description VARCHAR(255) NOT NULL UNIQUE,
    tag_group VARCHAR(20)
);

CREATE TABLE designs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename VARCHAR(500) NOT NULL,
    filepath VARCHAR(1000) NOT NULL,
    image_data BLOB,
    image_type VARCHAR(10),
    width_mm NUMERIC(8,2),
    height_mm NUMERIC(8,2),
    stitch_count INTEGER,
    color_count INTEGER,
    color_change_count INTEGER,
    notes TEXT,
    rating SMALLINT,
    is_stitched BOOLEAN NOT NULL DEFAULT 0,
    tags_checked BOOLEAN NOT NULL DEFAULT 0,
    tagging_tier SMALLINT,
    date_added DATE,
    designer_id INTEGER REFERENCES designers(id) ON DELETE SET NULL,
    source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
    hoop_id INTEGER REFERENCES hoops(id) ON DELETE SET NULL
);
CREATE INDEX ix_designs_designer_id_filename ON designs(designer_id, filename);
CREATE INDEX ix_designs_source_id_filename ON designs(source_id, filename);

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    date_created DATE
);

CREATE TABLE settings (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT
);

CREATE TABLE design_tags (
    design_id INTEGER NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (design_id, tag_id)
);
CREATE INDEX ix_design_tags_tag_id_design_id ON design_tags(tag_id, design_id);

CREATE TABLE project_designs (
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    design_id INTEGER NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, design_id)
);
CREATE INDEX ix_project_designs_design_id_project_id ON project_designs(design_id, project_id);
