-- (1) Normalise separators to forward slashes and trim whitespace.
UPDATE designs SET filepath = replace(trim(filepath), '\', '/');

-- (2) Trim leading slashes so '/MachineEmbroideryDesigns/...' -> 'MachineEmbroideryDesigns/...'
UPDATE designs SET filepath = trim(filepath, '/');

-- (3) Drop a LEADING 'MachineEmbroideryDesigns' container segment (case-insensitive),
--     so 'MachineEmbroideryDesigns/Flowers/rose.pes' -> 'Flowers/rose.pes'
--     and 'MachineEmbroideryDesigns/rose.pes'        -> 'rose.pes' (library root).
UPDATE designs
SET filepath = CASE
  WHEN lower(filepath) LIKE 'machineembroiderydesigns/%'
    THEN substr(filepath, length('machineembroiderydesigns/') + 1)
  WHEN lower(filepath) = 'machineembroiderydesigns'
    THEN ''  -- container-only reference: no file segment remains
  ELSE filepath
END;

-- (4) Trim any leading slashes the strip might have exposed.
UPDATE designs SET filepath = trim(filepath, '/');

COMMIT
