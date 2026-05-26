# Import Folder Assignment Refactor Checklist

Use this checklist when changing per-folder Designer/Source assignment during import, import review form parsing, or tokenized precheck/confirm flow.

## 1. Contract Safety
- [ ] `/import/scan`, `/import/precheck`, `/import/precheck-action`, `/import/do-confirm`, and `/import/confirm` contracts remain compatible, or migration is documented.
- [ ] Field naming contract for folder/global choices remains backward compatible (`designer_choice_*`, `source_choice_*`, `folder_root_*`).
- [ ] Any route-return behavior changes are reflected in import templates and tests.
- [ ] Line-level references in [docs/Specs/import-folder-assignment-backend-spec.md](docs/Specs/import-folder-assignment-backend-spec.md) are updated.

## 2. Assignment Semantics
- [ ] Precedence remains explicit and unchanged unless approved: per-folder -> global -> inferred -> blank.
- [ ] `blank` choice remains a deliberate null assignment (not inferred fallback).
- [ ] `create` choice still performs find-or-create with normalized dedupe behavior.
- [ ] Per-folder `inferred` still allows global fallback when global is set.

## 3. Multi-Folder Path and Grouping Safety
- [ ] Duplicate folder path deduplication remains safe and deterministic.
- [ ] Duplicate basename source folders still receive unique managed roots.
- [ ] `folder_key`/`folder_root` mapping remains stable across scan and confirm stages.
- [ ] Selected-file resolution still prevents path escape outside selected source folders.

## 4. Token and Context Lifecycle Safety
- [ ] Import context token validation remains enforced.
- [ ] Pop/get semantics for context consumption remain consistent and test-backed.
- [ ] If TTL/cap policy is introduced, it is deterministic and covered by tests.
- [ ] Cancel and invalid-token flows remain safe and do not import data.

## 5. Execution, Commit, and Persistence Safety
- [ ] Commit-batch resolution remains explicit and consistent between route settings and service defaults.
- [ ] Interleaved selected-file processing and confirm persistence behavior remains intentional and documented.
- [ ] Existing-file skip behavior remains stable.
- [ ] AI Tier 2/3 gating by API key/settings remains unchanged unless explicitly approved.

## 6. Structural Convergence
- [ ] Confirm path convergence is explicit (single canonical execution path, compatibility shim documented if present).
- [ ] Form parsing logic is centralized (no duplicated ad hoc field loops across multiple handlers).
- [ ] Route layer remains orchestration-focused; assignment semantics stay service-owned.
- [ ] Any new parser/DTO boundary is shared by both token and direct confirm flows.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_routes.py](tests/test_routes.py)
- [ ] New behavior has at least one route-level and one service-level regression test.
- [ ] Precedence logic changes include explicit regression tests for per-folder/global/inferred/blank combinations.
- [ ] Context-token lifecycle changes include explicit tests for invalid/expired/missing tokens.

## 8. Documentation Gate
- [ ] Current behavior and target architecture sections are updated in [docs/Specs/import-folder-assignment-backend-spec.md](docs/Specs/import-folder-assignment-backend-spec.md).
- [ ] Coverage matrix in the spec is updated for any new requirement or changed behavior.
- [ ] User quick guide remains aligned with implemented flow: [docs/User-Facing-Guidance/IMPORT_FOLDER_ASSIGNMENT.md](docs/User-Facing-Guidance/IMPORT_FOLDER_ASSIGNMENT.md).
- [ ] Feature inventory links remain accurate and discoverable.