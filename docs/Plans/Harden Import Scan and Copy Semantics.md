## Plan: Harden Import Scan and Copy Semantics

Align bulk import scanning, dedupe, and destination path behavior to match portable-library rules by (1) tightening in-library detection to true AppRoot scope, (2) computing deterministic prospective stored paths during preview (without copying), (3) adding DB-backed content dedupe using BLAKE3 + file size, and (4) adding collision-safe auto-rename copy behavior.

**Steps**
1. Phase 1 - Path Semantics Baseline
1.1 Extract a pure helper from ensure_file_in_designs_base path logic to compute prospective stored filepath from input file path + selected root_paths without touching filesystem writes (depends on none).
1.2 Reuse this helper in both preview dedupe and confirm import so path mapping logic is single-source-of-truth (depends on 1.1).
1.3 Replace substring-based in-library detection in full_path_to_stored_design_filepath with canonical-base-prefix validation against get_designs_base_path (case-insensitive boundary-safe compare, separator-normalized) so only files under actual AppRoot/data/MachineEmbroideryDesigns are treated in-place (depends on none).
1.4 Keep current selected-folder-leaf preservation behavior and longest-root match semantics; add explicit guardrails for root boundary and trailing separators, and when selected root has no normal leaf (for example C:/ or D:/) place files directly under /MachineEmbroideryDesigns using root-relative subpaths (parallel with 1.3).

2. Phase 2 - Preview Dedup Correctness
2.1 Update filter_existing_scanned_files to derive candidate stored filepath for every scanned file via helper from Step 1, instead of fallback to raw full path for external files (depends on 1.1, 1.2).
2.2 Preserve path normalization function for cross-platform matching (slash normalization + ASCII-case fold), but ensure comparison is done against prospective stored relative filepath for all importable files (depends on 2.1).
2.3 Keep in-library files importable only when not in DB; if already present by stored path, exclude from Ready to Import as today (depends on 2.1).

3. Phase 3 - Hash+Size Dedup (Preview + Confirm)
3.1 Add schema columns to designs for content fingerprinting: file_size_bytes (INTEGER) and file_hash_blake3 (TEXT), with index strategy suitable for lookup (depends on none).
3.2 Persist file_size_bytes and file_hash_blake3 during confirm import when inserting designs; compute from the actual source of truth file that will be stored/imported (depends on 3.1).
3.3 Extend preview filtering to exclude files already in DB when either stored filepath matches OR (file_size_bytes + file_hash_blake3) matches existing rows (depends on 3.1, 3.2).
3.4 Keep confirm-time safety check by filepath and add content-based existence check before insert as final guard (depends on 3.2).
3.5 Add a one-time backfill workflow for pre-existing designs: resolve each stored filepath to on-disk file under AppRoot/data/MachineEmbroideryDesigns, compute BLAKE3 + file size, and update NULL hash rows in batches with resumable progress logging (depends on 3.1).

4. Phase 4 - Copy Collision Policy (Auto-Rename)
4.1 Before fs::copy, if destination exists and content differs, choose deterministic auto-rename in same folder (for example stem + _1, _2, etc.) and return stored filepath for renamed target (depends on 1.1).
4.2 If destination exists and content matches hash+size, treat as already present (no copy) and reuse stored filepath (depends on 3.2).
4.3 Ensure resulting stored filepath from renamed files is what gets persisted and returned in flow (depends on 4.1).

5. Phase 5 - Validation and Regression Tests
5.1 Add unit tests for path derivation covering your three examples and mixed separators/case inputs (depends on 1.1).
5.2 Add tests proving preview dedupe excludes already-imported external-source files by prospective stored path (depends on 2.1).
5.3 Add tests for in-library detection strictness so unrelated paths containing machineembroiderydesigns do not bypass copy (depends on 1.3).
5.4 Add tests for OR dedupe logic (path OR hash+size), legacy-row behavior during backfill windows, and collision auto-rename outcomes (depends on 3.3, 3.5, 4.1).

**Relevant files**
- d:/My Software Development/Rust-Embroidery-Catalogue/src/routes/bulk_import.rs - core path derivation, in-library detection, preview dedupe, and confirm persistence flow; modify full_path_to_stored_design_filepath, ensure_file_in_designs_base, filter_existing_scanned_files, and confirm insert path.
- d:/My Software Development/Rust-Embroidery-Catalogue/src/services/scanning.rs - scanned file metadata extension and scan-time file stat/hash collection if done in scan stage.
- d:/My Software Development/Rust-Embroidery-Catalogue/migrations/20260503000000_initial.up.sql - reference for designs schema shape and indexing conventions when adding a forward migration in migrations/.
- d:/My Software Development/Rust-Embroidery-Catalogue/migrations/20260503000000_initial.down.sql - reference for reversible migration style.
- d:/My Software Development/Rust-Embroidery-Catalogue/src/database/models.rs - design model updates for new hash/size fields if typed model is used for inserts/reads.
- d:/My Software Development/Rust-Embroidery-Catalogue/frontend/src/lib/views/ImportView.svelte - optional UI messaging updates if preview now reports excluded-by-duplicate counts.

**Path Construction Behavior (Windows + Relative Roots)**
1. Standard Windows drive-letter folder roots preserve selected leaf:
- Selected root C:/x/d/f + file C:/x/d/f/Babies/Jef Files/design.jef => /MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef.
- Selected root C:/x + file C:/x/d/f/Babies/Jef Files/design.jef => /MachineEmbroideryDesigns/x/d/f/Babies/Jef Files/design.jef.
2. Windows absolute drive root selections (no leaf component) place files directly under MachineEmbroideryDesigns using the path relative to the selected drive root:
- Selected root C:/ + file C:/Designs/Floral/a.pes => /MachineEmbroideryDesigns/Designs/Floral/a.pes.
- Selected root D:/ + file D:/Embroidery/a.pes => /MachineEmbroideryDesigns/Embroidery/a.pes.
3. Relative-root style selections are resolved to canonical absolute paths before derivation, then follow the same leaf/label rules:
- Selected root ./imports (resolved to <cwd>/imports) + file <cwd>/imports/Faces/a.jef => /MachineEmbroideryDesigns/imports/Faces/a.jef.
- Selected root C:imports (drive-relative path on Windows) is canonicalized first; resulting absolute root leaf is used as the first destination segment.
4. Separator policy is canonical: treat \\ and / as equivalent for matching, but persist stored filepath using forward slashes only.

**Verification**
1. Run Rust tests targeting scanning/import route modules and confirm all pass.
2. Add and run new tests for selected-folder-leaf preservation examples:
- select C:/x => MachineEmbroideryDesigns/x/d/f/...
- select C:/x/d/f => MachineEmbroideryDesigns/f/...
- select C:/x/d/f/Babies/Jef Files => MachineEmbroideryDesigns/Jef Files/...
- include drive-root case: select C:/ + file C:/Designs/Floral/a.pes => MachineEmbroideryDesigns/Designs/Floral/a.pes (no synthetic drive folder)

3. Manual dry-run preview against a folder already imported from outside AppRoot and confirm Ready to Import excludes those files.
4. Manual import where destination filename already exists with different content and confirm auto-rename path is used and persisted.
5. Manual import of a file under AppRoot/data/MachineEmbroideryDesigns not yet in DB and confirm no copy occurs while DB row is created.
6. Run the hash backfill workflow against a seeded DB containing pre-change rows and verify:
- rows with existing files get file_size_bytes + file_hash_blake3 populated,
- missing-file rows are logged and skipped without aborting,
- rerunning backfill is idempotent (already-populated rows are unchanged).

**Decisions**
- Hash method: BLAKE3.
- Collision behavior: auto-rename new file when destination exists with different content.
- Dedupe stage: enforce hash+size dedupe already at preview, plus confirm-time safeguard.
- Scope included: backend scan/copy/dedupe path; DB migration and tests.
- Scope excluded: broad frontend redesign; non-import pipeline refactors.

**Further Considerations**
1. Performance strategy for preview hashing on very large libraries: eager hash all scanned files vs lazy hash only candidates that survive filepath checks; recommend lazy hash to reduce scan latency.
2. Legacy data policy: perform an explicit one-time backfill pass for all pre-change rows, then keep opportunistic fill as fallback for any unresolved/missing-file rows.
3. Optional observability: add structured counters for excluded_by_path and excluded_by_hash_size in preview response to improve user trust in dedupe outcomes.
