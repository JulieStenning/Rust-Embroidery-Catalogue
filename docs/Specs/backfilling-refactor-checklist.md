# Backfilling Refactor Checklist

Use this checklist when changing import tagging, unified backfill, or active backfill-related endpoints.

## 1. Contract Safety
- [ ] Endpoint path and method remain compatible, or migration plan is documented.
- [ ] Request payload changes are versioned or backward compatible.
- [ ] Response schema preserves existing keys or includes compatibility shim.
- [ ] Line-level references in [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md) are updated.
- [ ] Batch/commit precedence contract remains explicit and stable (request payload overrides -> settings -> defaults).

## 2. Selection and Action Semantics
- [ ] Tagging action selection behavior is unchanged unless explicitly approved.
- [ ] Stitching clear behavior remains explicit and documented.
- [ ] Images `redo` vs `upgrade_2d_to_3d` behavior is preserved.
- [ ] Color-count update scope remains correct for missing vs overwrite behavior.

## 3. Performance and Execution Model
- [ ] Single-read-per-design behavior is preserved for CPU-bound actions.
- [ ] Parallel and sequential split remains intentional and documented.
- [ ] Commit cadence is validated for both route defaults and service defaults.
- [ ] SQLite PRAGMA tuning/restore remains safe and symmetric.
- [ ] Batch/commit resolver is centralized across import and unified-backfill entrypoints (no divergent fallback chains).

## 4. Stop and Resilience
- [ ] Stop request flow still works end-to-end.
- [ ] Worker stop propagation (event/sentinel) still works in parallel runs.
- [ ] Final commit behavior is safe on normal exit and stop exit.
- [ ] Exception paths preserve partial progress and return consistent summary.

## 5. Logging and Operability
- [ ] Error/info log paths are unchanged or migration is documented.
- [ ] Log format remains parseable and documented.
- [ ] Log lifecycle policy (truncate vs retain) is explicit.
- [ ] Log download endpoint behavior remains compatible.

## 6. Import Tier Flow Integrity
- [ ] Tier 1/2/3 ordering and fallback behavior are preserved.
- [ ] API key gates still protect Tier 2/3 execution.
- [ ] Import settings (`ai.*`, `import.commit_batch_size`) still map correctly to runtime behavior.
- [ ] Any batch-size semantic changes are reflected in docs and UI hints.
- [ ] Shared assignment utility contract is used consistently for import and tagging-actions assignment paths when convergence changes are introduced.
- [ ] Assignment flags (`do_stitch_types`, `do_image`, `do_color_counts`) preserve prior behavior for selective runs.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
  - [tests/test_routes.py](tests/test_routes.py)
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
- [ ] New behavior has at least one route-level and one service-level test.
- [ ] Stop behavior has explicit regression coverage when execution model changes.
- [ ] Resolver precedence has explicit regression coverage (request override > settings > defaults).
- [ ] Shared assignment utility convergence has regression coverage for both import and tagging-actions flows.

## 8. Documentation Gate
- [ ] Current Behavior section updated for any implemented behavioral change.
- [ ] Target Architecture section updated if roadmap direction changes.
- [ ] All new critical behavior statements include file+line references.
- [ ] Changelog entry added if behavior is externally observable.
