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

-- Insert default settings into the settings table
INSERT INTO settings (key, value, description) VALUES
    ('disclaimer_accepted', 'FALSE', 'Whether the application''s disclaimer has been accepted for this installation.'),
    ('import.last_browse_folder', '', 'Most recently used folder for the bulk import picker.'),
    ('ai.tier2_auto', 'FALSE', 'Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present.'),
    ('ai.tier3_auto', 'FALSE', 'Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present.'),
    ('ai.batch_size', '', 'Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.'),
    ('ai.delay', '', 'Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds). Increase this if you encounter 429 Too Many Requests errors.'),
    ('import.commit_batch_size', '', 'Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (1000).');

-- Insert default tags into the tags table
INSERT INTO tags (id, description, tag_group) VALUES
(1, 'Cross Stitch', 'stitching'),
(2, 'In The Hoop', 'stitching'),
(3, 'Filled', 'stitching'),
(4, 'Redwork', 'stitching'),
(5, 'Blackwork', 'stitching'),
(6, 'Stfumato', 'stitching'),
(7, 'Cutwork', 'stitching'),
(8, 'Don''t Know', 'image'),
(9, 'Line Outline', 'stitching'),
(10, 'Satin Stitch', 'stitching'),
(11, 'Applique', 'stitching'),
(12, 'Silhouette', 'image'),
(13, 'Light Fills', 'stitching'),
(14, 'Lace', 'stitching'),
(15, 'Trapunto', 'stitching'),
(16, 'For Quilting', 'image'),
(17, 'Handstitched Look', 'stitching'),
(18, 'Animals', 'image'),
(19, 'Flowers', 'image'),
(20, 'People and Work', 'image'),
(21, 'Job', 'image'),
(22, 'House', 'image'),
(23, 'Garden', 'image'),
(24, 'Music', 'image'),
(25, 'Nautical', 'image'),
(26, 'Landscapes and Travel', 'image'),
(27, 'Toys', 'image'),
(28, 'Hearts and Lips', 'image'),
(29, 'Sport', 'image'),
(30, 'Borders', 'image'),
(31, 'Paisley', 'image'),
(32, 'Butterflies and Insects', 'image'),
(33, 'Words and Letters', 'image'),
(34, 'Christmas', 'image'),
(35, 'Patterns', 'image'),
(36, 'Corners', 'image'),
(37, 'Bow and Ribbons', 'image'),
(38, 'Frames', 'image'),
(39, 'Trees', 'image'),
(40, 'Transport', 'image'),
(42, 'Birds', 'image'),
(43, 'Monsters', 'image'),
(44, 'Utility - Testing', 'image'),
(45, 'Food', 'image'),
(46, 'Scrolls', 'image'),
(47, 'Footwear', 'image'),
(48, 'For Clothes', 'image'),
(49, 'Faces', 'image'),
(50, 'Handbags', 'image'),
(51, 'Fairies, Elves etc.', 'image'),
(52, 'Hobbies', 'image'),
(53, 'Ornaments', 'image'),
(54, 'Collars', 'image'),
(55, 'Household', 'image'),
(56, 'Fashion', 'image'),
(57, 'Halloween', 'image'),
(58, 'Sun Moon and Stars', 'image'),
(59, 'Angels', 'image'),
(60, 'Babies', 'image'),
(61, 'Quilting', 'stitching'),
(62, 'Jewellery', 'image'),
(63, 'Buildings and Structures', 'image'),
(64, 'Crests', 'image'),
(65, 'Badges and Crests', 'image'),
(66, 'Fish and Seashells', 'image'),
(67, 'Flags', 'image'),
(68, 'Children', 'image'),
(69, 'Cartoon', 'image'),
(70, 'Banners', 'image'),
(71, 'Celebrations', 'image'),
(72, 'Ghosts', 'image'),
(73, 'Winter', 'image'),
(74, 'Zodiac', 'image'),
(75, 'Alphabets', 'image'),
(95, 'Cats', 'image'),
(96, 'Celtic and Tribal', 'image'),
(97, 'Dogs', 'image'),
(98, 'Diwali', 'image'),
(99, 'Easter', 'image'),
(100, 'Eid', 'image'),
(101, 'Fantasy', 'image'),
(102, 'Father''s Day', 'image'),
(103, 'Hanukkah', 'image'),
(104, 'Horses', 'image'),
(105, 'ITH Accessories', 'stitching'),
(106, 'Monogram', 'image'),
(107, 'Mother''s Day', 'image'),
(108, 'Religious', 'image'),
(109, 'Sketchy and Vintage', 'stitching'),
(110, 'Thanksgiving', 'image'),
(111, 'Valentine''s Day', 'image'),
(112, 'Wedding', 'image'),
(113, 'Wreaths', 'image'),
(114, 'Steampunk', 'image'),
(115, 'Sewing', 'image'),
(116, 'Clothes', 'image'),
(117, 'Netfill', 'stitching'),
(118, 'Dancing', 'image');

