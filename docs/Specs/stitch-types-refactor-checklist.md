# Stitch Types Refactor Checklist

Use this checklist when changing stitch-type definition, stitching-tag mapping, or stitching backfill behavior.

## 1. Contract Safety
- [ ] `StitchIdentifier` constructor and public methods remain backward compatible, or the migration plan is documented.
- [ ] `identify_stitches()` still returns a stable, sorted list of internal stitch-type names.
- [ ] `get_detailed_analysis()` preserves its key set unless the change is explicitly approved.
- [ ] `suggest_stitching_from_pattern()` still maps detector outputs to stitching tag descriptions.
- [ ] Import and admin backfill callers continue to receive the same shape of result or a documented compatibility shim.

## 2. Type Coverage and Semantics
- [ ] The 8-current-type set remains explicit unless redwork/blackwork are intentionally added.
- [ ] Any redwork/blackwork work is treated as a new scope item, not as an assumed existing behavior.
- [ ] Name-based precedence rules remain documented for ITH, applique, cross stitch, and lace.
- [ ] Suppression rules remain intentional: satin vs outline, cross stitch vs satin/fill, applique vs satin/outline, lace vs filled.
- [ ] Threshold behavior remains consistent for default and custom confidence levels.

## 3. Selection and Mapping Semantics
- [ ] Internal stitch names still map to the correct stitching tag descriptions.
- [ ] The stitching tag group remains the only tag group modified by stitching detection.
- [ ] Unknown or low-confidence detections continue to fail safe by returning no tag.
- [ ] Current geometry-based detection is not silently replaced by an image-based matcher without a documented migration.

## 4. Performance and Execution Model
- [ ] Pattern analysis still reads the design once per caller path where practical.
- [ ] Shared analysis remains centralized in `src/services/pattern_analysis.py` rather than duplicated across routes.
- [ ] Import and admin backfill continue to use the same stitching-analysis bridge.
- [ ] Any new caching or memoization is justified and covered by tests.

## 5. Stop and Resilience
- [ ] Admin stitching backfill still skips verified designs where that is the documented behavior.
- [ ] Unverified-only guards remain explicit and tested.
- [ ] Partial progress is preserved if a run is interrupted.
- [ ] Failures continue to be reported without corrupting existing tag data.

## 6. Logging and Operability
- [ ] User-visible behavior changes are reflected in docs and release notes.
- [ ] Any new diagnostics or log output are parseable and not overly noisy.
- [ ] If new fallback branches are added, they are documented with rationale.
- [ ] The docs clearly state whether the behavior is geometry-based or image-based.

## 7. Import and Backfill Flow Integrity
- [ ] Import-time stitching detection continues to work through `confirm_import` and shared pattern analysis.
- [ ] Admin Tagging Actions continues to use the same stitching bridge for existing designs.
- [ ] The unverified-only stitching backfill still leaves verified designs untouched.
- [ ] The `stitching` tag group remains isolated from `image` and AI tag groups.
- [ ] Any future example-image workflow is documented separately from the current geometry workflow.

## 8. Test Coverage Gate
- [ ] Updated or added tests in:
  - [tests/test_stitch_identifier.py](tests/test_stitch_identifier.py)
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_legacy_tagging_actions.py](tests/test_legacy_tagging_actions.py)
  - [tests/test_services.py](tests/test_services.py)
- [ ] New behavior has at least one detector-level test and one integration-level test.
- [ ] Name-based detection changes have explicit regression coverage.
- [ ] Threshold or suppression changes have explicit regression coverage.
- [ ] Import and admin backfill coverage remain aligned with the current flow.

## 9. Documentation Gate
- [ ] Current behavior section updated for any implemented behavioral change.
- [ ] Target architecture section updated if the roadmap direction changes.
- [ ] All new critical behavior statements include file and line references.
- [ ] The user guide still matches what the app actually does today.
