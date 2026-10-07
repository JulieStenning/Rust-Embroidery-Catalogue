# Rust Coding Standards

## 🦀 Rust Coding Standards

- **Zero Panics:** Absolute ban on `unwrap()` or `expect()` in binary parsing modules. Handle all out-of-bounds, empty, or malformed files gracefully via strict error types.
- **Performance-Focused I/O:** Use buffered readers (`BufReader`) and streaming/lazy parsing logic where possible.
- **Automated Verification:** Run `cargo check` or `cargo test` after editing Rust code to ensure the borrow checker is satisfied and tests pass.
- **Native folder pickers (`rfd`):** `FileDialog::set_directory` silently falls back to system defaults on unreadable/missing paths. Validate the start directory first and pass an explicit fallback (e.g. `app_handle.path().document_dir()`).
- **Instant Database Verification (< 1 ms vs minutes on large DBs):**
  - Never execute full integrity checks (e.g. `PRAGMA quick_check(1)` or full table scans) in synchronous startup or migration verification paths. On large catalogues (e.g. 7.6 GB+), `quick_check(1)` scans every single B-tree page until an error is found, hanging startup and restore operations for minutes on slow removable media.
  - Use fast, constant-time schema verification (`PRAGMA schema_version` + `SELECT 1 FROM settings LIMIT 1` + `SELECT 1 FROM designs LIMIT 1`), which completes in < 1 ms regardless of database size.
- **Rendering & Parsing Performance Discipline:**
  - Avoid nested $O(r^2 \cdot N)$ coordinate loops when rendering stitch files. Use vectorised, single-pass polygon/capsule rasterizers.
  - Performance-critical parsing and rendering pipelines must be benchmarked using Criterion harnesses in `benches/` to detect regressions.
- **Production Module Size Limit (< 500 lines):**
  - Every production Rust module must remain strictly under **500 lines**.
  - Complex modules must be decomposed into focused domain submodules (e.g. `src/routes/designs/` with `browse.rs`, `details.rs`, `metadata.rs`, `tags.rs`, `deletion.rs`, `launch.rs`, `preview.rs`, `types.rs`, `mod.rs`).

### 💾 Canonical `designs.filepath` format (single source of truth)

- **Invariant:** every row in `designs.filepath` is a _canonical relative path from the designs library root_ (`<data_root>/MachineEmbroideryDesigns` = `AppPaths.embroidery_designs_dir`).
- **Format rules:** forward slashes (`/`) only; **no leading slash**; **no absolute path or Windows drive letter**; preserves exact case. A root design is a bare filename (`rose.pes`); subfolders are `Flowers/rose.pes`.
- `MachineEmbroideryDesigns` must NEVER be stored as a path prefix in `filepath`.
- **Single source of truth:** all path logic goes through helpers in `src/paths.rs` (`canonical_design_rel`, `design_rel_from_full`, `resolve_design_filepath`).

### ⚙️ Read-Only SQLite Connection Invariants
- When opening an SQLite pool or connection with `.read_only(true)`, never execute write-affecting pragmas (e.g. `SqliteJournalMode::Off` or journal mode mutations). Applying write pragmas to read-only connections triggers disk I/O error code 3850.

### 📏 Module Size & Header Limits
- **500-line budget:** Any Rust source file whose total line count exceeds **500 lines** (production + embedded test code) MUST have its `#[cfg(test)]` module(s) extracted into a separate sibling file `<basename>_tests.rs` with `#[path = "<basename>_tests.rs"] mod tests;`.
- Keep top-of-file license/SPDX comment headers concise (<= 3 lines) to conserve the 500-line vertical module budget.
