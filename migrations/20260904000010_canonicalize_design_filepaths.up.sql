-- Canonicalize `designs.filepath` to a single library-relative form.
--
-- Invariant established here: every `designs.filepath` is a forward-slash
-- relative path from the designs library root (`<data_root>/MachineEmbroideryDesigns`):
--   * forward slashes only (no `\`);
--   * no leading slash;
--   * no absolute path / Windows drive letter;
--   * the `MachineEmbroideryDesigns` container prefix is dropped;
--   * a design at the library root is stored as a bare filename (no `/`).
-- Case is preserved (nothing is lower-cased when written back).
--
-- These transformations are deterministic and idempotent over the formats the
-- application has ever written (managed `/MachineEmbroideryDesigns/...`,
-- markerless `MachineEmbroideryDesigns/...`, bare-relative, and legacy
-- absolute paths rooted under a `MachineEmbroideryDesigns` segment). Any row a
-- path string function cannot reduce (e.g. an absolute path with no container
-- marker) is left untouched and handled gracefully by the Rust resolver.

-- 1) Normalise separators and surrounding whitespace.
UPDATE designs SET filepath = replace(trim(filepath), '\', '/');

-- 2) Drop leading slashes.
UPDATE designs SET filepath = trim(filepath, '/');

-- 3) Drop a LEADING `MachineEmbroideryDesigns` container segment (case-insensitive).
--    Only the first path element is treated as the root, so a genuine nested
--    folder of the same name is preserved.
UPDATE designs
SET filepath = CASE
  WHEN lower(filepath) LIKE 'machineembroiderydesigns/%'
    THEN substr(filepath, length('machineembroiderydesigns/') + 1)
  WHEN lower(filepath) = 'machineembroiderydesigns'
    THEN ''  -- container-only reference: no file segment remains
  ELSE filepath
END;

-- 4) Collapse any leading slashes introduced by step 3.
UPDATE designs SET filepath = trim(filepath, '/');

-- 5) Legacy absolute paths that are still not canonical: strip everything
--    through a `/machineembroiderydesigns/` segment when the row is still
--    drive-letter-prefixed, leaving only the relative tail under that marker.
--    Guarded so a real nested folder named `machineembroiderydesigns` in an
--    already-relative path is never stripped.
UPDATE designs
SET filepath = CASE
  WHEN filepath GLOB '[A-Za-z]:*'                 -- still absolute-looking (Windows drive)
       AND instr(lower(filepath), '/machineembroiderydesigns/') > 0
    THEN substr(filepath, instr(lower(filepath), '/machineembroiderydesigns/') + 1)
  ELSE filepath
END;

-- 6) Final leading-slash trim.
UPDATE designs SET filepath = trim(filepath, '/');
