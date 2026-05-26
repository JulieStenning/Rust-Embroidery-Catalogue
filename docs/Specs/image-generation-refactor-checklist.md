# Image Generation Refactor Checklist

Use this checklist when changing preview rendering, image metadata extraction, or image-related import/backfill endpoints.

## 1. Contract Safety
- [ ] Endpoint path and method remain compatible, or migration plan is documented.
- [ ] Request payload updates for image actions are backward compatible.
- [ ] Response schema preserves existing keys or includes compatibility shim.
- [ ] Line-level references in [docs/Specs/image-generation-backend-spec.md](docs/Specs/image-generation-backend-spec.md) are updated.
- [ ] Default/precedence behavior for `preview_3d`, `image_preference`, `batch_size`, and `commit_every` is explicit and stable.

## 2. Render and Selection Semantics
- [ ] `redo_all_images` behavior remains explicit and unchanged unless approved.
- [ ] `upgrade_2d_to_3d` behavior remains explicit and unchanged unless approved.
- [ ] Existing-image skip behavior remains correct for non-redo runs.
- [ ] 2D vs 3D image-type assignment remains accurate (`image_type` mapping).
- [ ] ART/Spider fallback order and behavior are documented and preserved.

## 3. Metadata Integrity
- [ ] Width/height extraction from pattern bounds remains correct and rounded consistently.
- [ ] Stitch/color/color-change count extraction remains consistent with existing null-fill behavior.
- [ ] Hoop assignment behavior remains correct for computed dimensions.
- [ ] Existing metadata carry-forward behavior remains stable when re-render is skipped.

## 4. Performance and Execution Model
- [ ] Single-read-per-design behavior is preserved for image-related work.
- [ ] Parallel and sequential split remains intentional and documented.
- [ ] Commit cadence remains validated for route defaults and service defaults.
- [ ] SQLite PRAGMA optimize/restore remains safe and symmetric.
- [ ] Any new resolver logic avoids divergent fallback chains between import and unified backfill.

## 5. Stop and Resilience
- [ ] Stop request flow still works end-to-end for image-heavy runs.
- [ ] Worker stop propagation remains correct in parallel runs.
- [ ] Final commit behavior is safe on normal exit and stop exit.
- [ ] Render/read exceptions preserve partial progress and consistent summary output.

## 6. Logging and Operability
- [ ] Error/info log paths are unchanged or migration is documented.
- [ ] Log format remains parseable and documented.
- [ ] Log lifecycle policy (truncate vs retain) is explicit.
- [ ] Log download behavior remains compatible with current admin workflow.

## 7. Import and Backfill Flow Integrity
- [ ] Import flow still resolves preview mode from configured/requested image preference.
- [ ] Unified backfill still propagates `preview_3d` to image action options.
- [ ] Shared analysis contract (`analyze_pattern`) remains used consistently for image data and dimensions.
- [ ] Secondary single-design rerender behavior remains compatible if touched.

## 8. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
  - [tests/test_routes.py](tests/test_routes.py)
- [ ] New behavior has at least one route-level and one service-level test.
- [ ] 2D/3D mode propagation has explicit regression coverage.
- [ ] Redo and upgrade semantics have explicit regression coverage.
- [ ] ART/Spider fallback behavior has explicit regression coverage when touched.

## 9. Documentation Gate
- [ ] Current Behavior section updated for any implemented change.
- [ ] Target Architecture section updated if roadmap direction changes.
- [ ] All new critical behavior statements include file+line references.
- [ ] User guidance in [docs/User-Facing-Guidance/IMAGE_GENERATION.md](docs/User-Facing-Guidance/IMAGE_GENERATION.md) remains aligned with actual behavior.
- [ ] Changelog entry added if behavior is externally observable.
