# Design Detail Refactor Checklist

Use this checklist when changing detail-page routes, print behavior, or detail-page metadata/tag/project actions.

## 1. Contract Safety
- [ ] Endpoint paths and methods remain compatible, or migration plan is documented.
- [ ] Request payload changes remain backward compatible (especially `/edit`, `/set-tags`, and project actions).
- [ ] Redirect targets and status codes remain intentional and documented.
- [ ] Line-level references in [docs/Specs/design-detail-backend-spec.md](docs/Specs/design-detail-backend-spec.md) are updated.
- [ ] UI form action/path contracts remain aligned with [docs/Specs/UI/design-detail-ui-spec.md](docs/Specs/UI/design-detail-ui-spec.md).

## 2. Metadata and Tag Semantics
- [ ] Notes/Designer/Source/Hoop save behavior remains correct for blank vs selected values.
- [ ] Rating set/clear behavior remains stable (including validation bounds).
- [ ] Stitched toggle behavior preserves true/false semantics.
- [ ] Tag assignment remains replace-style (selected set is canonical).
- [ ] Save-tags behavior still marks `tags_checked=True` unless explicitly redesigned.
- [ ] Verify/unverify action still maps to `tags_checked` correctly.

## 3. Image and Preview Behavior
- [ ] Detail image endpoint still returns `image/png` bytes and 404 when unavailable.
- [ ] 3D preview render route still re-reads source file and updates `image_data`/`image_type`.
- [ ] Dimension update behavior on render (bounds -> mm) remains intentional.
- [ ] Missing source-file behavior for re-render remains explicit and test-covered.

## 4. Launch and Path Handling
- [ ] Full-path resolution still uses managed storage base path + stored filepath.
- [ ] Open in Editor still respects launch-suppression safeguards (`external_launches_disabled`).
- [ ] Open in Explorer still supports file-select path and nearest-folder fallback when file is missing.
- [ ] Launch exception handling remains non-destructive and redirect-safe.
- [ ] Platform-specific launch handling changes are documented.

## 5. Project Membership and Deletion
- [ ] Add-to-project and remove-from-project actions remain compatible with existing forms.
- [ ] Available-project list excludes already-associated projects.
- [ ] Delete behavior remains explicit and route status contract is unchanged.
- [ ] Any delete-side effects (relations/cascade expectations) remain validated.

## 6. Print Contract Integrity
- [ ] Print route remains available and returns 404 for missing designs.
- [ ] Print template still includes core metadata set (image, size, hoop, counts, tags, rating, stitched, notes as available).
- [ ] Print-only/no-print behavior remains coherent between detail and print surfaces.

## 7. Error and Status Semantics
- [ ] Missing-design behavior remains intentional per endpoint (`400`/`404`) and documented.
- [ ] Validation error behavior (for example invalid rating) remains stable and test-covered.
- [ ] Redirect-on-success contracts remain unchanged unless migration is documented.
- [ ] Exception paths keep partial progress safe and avoid data corruption.

## 8. Test Coverage Gate
- [ ] Updated/added tests in [tests/test_routes.py](tests/test_routes.py) for changed behavior.
- [ ] New behavior has at least one route-level coverage anchor.
- [ ] Launch suppression behavior is regression-covered when changing launch helpers.
- [ ] Error-path behavior (missing design / validation failure) is regression-covered.
- [ ] Print-route behavior is regression-covered if template or route changes.

## 9. Documentation Gate
- [ ] Current Behavior section in [docs/Specs/design-detail-backend-spec.md](docs/Specs/design-detail-backend-spec.md) updated for any implemented changes.
- [ ] UI contracts in [docs/Specs/UI/design-detail-ui-spec.md](docs/Specs/UI/design-detail-ui-spec.md) updated when controls/forms/states change.
- [ ] User guidance in [docs/User-Facing-Guidance/DESIGN_DETAIL.md](docs/User-Facing-Guidance/DESIGN_DETAIL.md) updated for user-visible behavior changes.
- [ ] Feature summary in [docs/feature-inventory.md](docs/feature-inventory.md) updated when behavior is externally observable.
- [ ] Changelog entry added for externally visible behavior changes.