-- Add explicit indexes on designs(width_mm) and designs(height_mm) for fast dimension filtering
CREATE INDEX IF NOT EXISTS ix_designs_width_mm ON designs (width_mm);
CREATE INDEX IF NOT EXISTS ix_designs_height_mm ON designs (height_mm);
