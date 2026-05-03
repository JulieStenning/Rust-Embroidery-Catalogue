-- tags_seed.sql: Insert default settings into the settings table
INSERT INTO settings (key, value, description) VALUES
    ('disclaimer_accepted', 'FALSE', 'Whether the application''s disclaimer has been accepted for this installation.'),
    ('import.last_browse_folder', '', 'Most recently used folder for the bulk import picker.'),
    ('ai.tier2_auto', 'FALSE', 'Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present.'),
    ('ai.tier3_auto', 'FALSE', 'Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present.'),
    ('ai.batch_size', '', 'Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.'),
    ('ai.delay', '', 'Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds). Increase this if you encounter 429 Too Many Requests errors.'),
    ('import.commit_batch_size', '', 'Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (1000).');
