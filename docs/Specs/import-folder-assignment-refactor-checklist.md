# Import Folder Assignment Refactor Checklist

Use this checklist when changing per-folder Designer/Source assignment during import, import review form parsing, or tokenized precheck/confirm flow.

## 1. Contract Safety
- [ ] `preview_bulk_import`, `precheck_bulk_import_wire`, `precheck_bulk_import_action_wire`, `do_confirm_bulk_import_wire`, and `confirm_bulk_import_wire` contracts remain compatible, or migration is documented.
- [ ] Direct-execution path `execute_bulk_import_confirm_wire` and the legacy shim `confirm_bulk_import_legacy` remain consistent with the canonical confirm flow.
- [ ] Field naming contract for folder/global choices remains backward compatible (`global_designer_id`, `global_source_id`, `per_folder_assignments[*].designer_id`, `per_folder_assignments[*].source_id`, `folder_path`).
- [ ] Any Tauri command return-behavior changes are reflected in the Svelte frontend (`frontend/src/lib/`) and Rust tests.
- [ ] Line-level references in [docs/Specs/import-folder-assignment-backend-spec.md](import-folder-assignment-backend-spec.md) are updated. *(Create this spec file if it is not present.)*

## 2. Assignment Semantics
- [ ] Precedence remains explicit and unchanged unless approved: per-folder -> global -> inferred -> blank.
- [ ] `blank` choice remains a deliberate null assignment (not inferred fallback) — matches `AssignmentFieldSourceWire::Blank`.
- [ ] `create` choice still performs find-or-create with normalized dedupe behavior (`create_on_import` in `BulkImportWire`).
- [ ] Per-folder `inferred` still allows global fallback when global is set — matches `AssignmentFieldSourceWire` ordering (`ExplicitPerFolder`, `Global`, `Inferred`, `Blank`).

## 3. Multi-Folder Path and Grouping Safety
- [ ] Duplicate folder path deduplication remains safe and deterministic (normalized path matching).
- [ ] Duplicate basename source folders still receive unique managed roots.
- [ ] `folder_path` mapping remains stable across scan and confirm stages.
- [ ] Selected-file resolution still prevents path escape outside selected source folders.

## 4. Token and Context Lifecycle Safety
- [ ] Import context token validation remains enforced (TTL, capacity, and pop/get semantics in `src/routes/bulk_import.rs`).
- [ ] Pop/get semantics for context consumption remain consistent and test-backed.
- [ ] If TTL/cap policy is changed (currently 15 minutes / 128 entries), it is deterministic and covered by tests.
- [ ] Cancel and invalid-token flows remain safe and do not import data.

## 5. Execution, Commit, and Persistence Safety
- [ ] Commit-batch resolution remains explicit and consistent between route settings and service defaults (`import.commit_batch_size`; default 10 in `src/routes/bulk_import.rs`).
- [ ] Interleaved selected-file processing and confirm persistence behavior remains intentional and documented.
- [ ] Existing-file skip behavior remains stable.
- [ ] AI Tier 2/3 gating by API key/settings remains unchanged unless explicitly approved (`normalize_tiers()` in `src/services/backfill.rs` drops tiers ≥ 2 when no `GOOGLE_API_KEY` is set).

## 6. Structural Convergence
- [ ] Confirm path convergence is explicit (single canonical execution path, compatibility shim documented if present).
- [ ] Form parsing logic is centralized (no duplicated ad hoc field loops across multiple handlers).
- [ ] Route layer remains orchestration-focused (`src/routes/bulk_import.rs`); assignment semantics stay service-owned (`src/services/`).
- [ ] Any new parser/DTO boundary (`BulkImportWire` / `BulkImportConfirmWire`) is shared by both token (`do_confirm_bulk_import_wire`) and direct confirm (`execute_bulk_import_confirm_wire`) flows.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - `#[cfg(test)] mod tests` in `src/routes/bulk_import.rs` (bulk-import route/service tests)
  - `#[cfg(test)] mod tests` in the relevant route modules (e.g. `src/routes/bulk_import.rs`, `src/routes/designs.rs`, `src/routes/settings.rs`)
- [ ] New behavior has at least one route-level and one service-level regression test.
- [ ] Precedence logic changes include explicit regression tests for per-folder/global/inferred/blank combinations.
- [ ] Context-token lifecycle changes include explicit tests for invalid/expired/missing tokens.
- [ ] Run `cargo test` to confirm the full suite passes.

## 8. Documentation Gate
- [ ] Current behavior and target architecture sections are updated in [docs/Specs/import-folder-assignment-backend-spec.md](import-folder-assignment-backend-spec.md). *(Create this spec file if it is not present.)*
- [ ] Coverage matrix in the spec is updated for any new requirement or changed behavior.
- [ ] User quick guide remains aligned with implemented flow: [docs/User-Facing-Guidance/IMPORT_FOLDER_ASSIGNMENT.md](../User-Facing-Guidance/IMPORT_FOLDER_ASSIGNMENT.md).
- [ ] Feature inventory links remain accurate and discoverable.