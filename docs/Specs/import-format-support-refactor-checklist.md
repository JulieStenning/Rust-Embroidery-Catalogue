# Import Format Support Refactor Checklist

Use this checklist when changing format allowlists, deduplication priority, `.art` handling, or import scan/processing behavior.

## 1. Contract Safety
- [ ] `/import/scan`, `/import/do-confirm`, and `/import/confirm` paths/methods remain compatible, or migration is documented.
- [ ] Request payload expectations for selected files/folders remain backward compatible.
- [ ] Response/render behavior preserves current import flow assumptions.
- [ ] Line-level references in [docs/Specs/import-format-support-backend-spec.md](docs/Specs/import-format-support-backend-spec.md) are updated.

## 2. Allowlist and Exclusion Integrity
- [ ] `SUPPORTED_EXTENSIONS` changes are intentional and approved.
- [ ] Excluded helper/output/vector/data formats remain excluded unless explicitly approved.
- [ ] `.pmv` inclusion remains explicit.
- [ ] `.art` remains classified as limited support unless a deliberate policy change is approved.

## 3. Priority and Deduplication Semantics
- [ ] `EXTENSION_PRIORITY` preserves intended preference ordering for duplicate stems.
- [ ] `_pick_preferred` behavior remains deterministic for mixed-extension duplicates.
- [ ] Any ordering changes are reflected in tests and rationale docs.
- [ ] Duplicate resolution still uses `(parent folder, stem)` grouping semantics.

## 4. `.art` Fallback Safety
- [ ] `.art` keeps special-case branch behavior (not blindly merged into generic path).
- [ ] Spider sidecar image and dimension lookup behavior remains valid.
- [ ] Embedded icon and metadata extraction paths remain resilient to malformed files.
- [ ] `.art` processing still degrades gracefully when pyembroidery decode fails.

## 5. Resilience and Error Isolation
- [ ] Per-file failures still populate `ScannedDesign.error` without aborting full runs.
- [ ] Existing-file skip behavior remains intact during scan and selected-file processing.
- [ ] Preview generation failures do not break metadata extraction for remaining files.
- [ ] Exception handling preserves partial progress consistently.

## 6. Shared Import/Backfill Consistency
- [ ] Shared settings keys used by import remain consistent with backfill docs and runtime.
- [ ] Any commit-batch semantic changes are reflected in both relevant specs.
- [ ] Overlap topics are linked to [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md) rather than duplicated inconsistently.
- [ ] Tier orchestration references remain accurate when import tagging flow changes.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_services.py](tests/test_services.py)
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_routes.py](tests/test_routes.py) (if route behavior changes)
- [ ] New extension policy behavior has at least one service-level regression test.
- [ ] Deduplication and `.art` fallback paths each have explicit regression coverage.

## 8. Documentation and User Guidance Gate
- [ ] Current Behavior section updated for implemented behavior changes.
- [ ] Target Architecture section updated if direction changes.
- [ ] All new critical behavior statements include file+line references.
- [ ] [docs/SUPPORTED_FORMATS.md](docs/SUPPORTED_FORMATS.md) is synchronized with runtime allowlist/exclusions and `.art` limitations.