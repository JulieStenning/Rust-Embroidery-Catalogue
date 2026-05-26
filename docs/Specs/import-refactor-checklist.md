# Import Refactor Checklist

Use this checklist when changing any part of import wizard behavior, import route contracts, token lifecycle, assignment semantics, format handling, or import-mode admin review paths.

This is the master checklist. Focused deep-dive checklists remain canonical for topic-specific validation:
- [docs/Specs/first-import-actions-refactor-checklist.md](docs/Specs/first-import-actions-refactor-checklist.md)
- [docs/Specs/import-folder-assignment-refactor-checklist.md](docs/Specs/import-folder-assignment-refactor-checklist.md)
- [docs/Specs/import-format-support-refactor-checklist.md](docs/Specs/import-format-support-refactor-checklist.md)

## 1. Contract Safety
- [ ] /import/, /import/browse-folder, /import/scan, /import/precheck, /import/precheck-action, /import/do-confirm, and /import/confirm remain compatible, or migration is documented.
- [ ] Precheck action values remain stable (review_hoops, review_tags, review_sources, review_designers, import_now, cancel), or migration is documented.
- [ ] Import-mode admin list routes preserve token-aware behavior on tags/hoops/sources/designers pages.
- [ ] Line-level references in [docs/Specs/import-backend-spec.md](docs/Specs/import-backend-spec.md) are updated.

## 2. Flow Semantics
- [ ] Step order remains coherent: folder selection -> scan/review -> precheck/actions -> confirm/save.
- [ ] First-import detection still reflects empty-catalogue behavior.
- [ ] Subsequent import behavior still allows optional review before import.
- [ ] Skip-hoops confirmation remains explicit when required.

## 3. Token Integrity and Lifecycle
- [ ] Token validation remains strict UUIDv4 format checking.
- [ ] Invalid/unknown token flows remain fail-safe with redirect to /import/.
- [ ] Confirm path remains single-use token consuming behavior.
- [ ] Any TTL/cap/cancel invalidation policy changes are deterministic, documented, and tested.

## 4. Assignment and Metadata Integrity
- [ ] Assignment precedence remains explicit unless approved: per-folder -> global -> inferred -> blank.
- [ ] Global and per-folder field naming contract remains stable (designer_choice_*, source_choice_*, folder_root_*).
- [ ] Create-on-import dedupe behavior remains intact for designer/source values.
- [ ] Selected-file reconstruction still prevents path escape from selected source folders.

## 5. Format and Preview Integrity
- [ ] SUPPORTED_EXTENSIONS changes are intentional and documented.
- [ ] EXTENSION_PRIORITY changes are intentional, deterministic, and regression-tested.
- [ ] .art remains limited support unless approved policy changes are documented.
- [ ] Per-file processing errors remain isolated and do not abort whole import runs.

## 6. Runtime and Settings Integration
- [ ] Import settings still map correctly into precheck and confirm execution.
- [ ] API key gating still enforces Tier 1-only behavior when no key is available.
- [ ] Image preference behavior remains consistent between precheck and confirm paths.
- [ ] Import commit batching remains explicit and aligned with service defaults or documented overrides.

## 7. Route/Service Boundary Quality
- [ ] Route layer remains orchestration-focused (no unbounded duplication of parsing/semantics).
- [ ] Confirm path convergence decisions are explicit and documented.
- [ ] Shared parser/DTO boundaries (if introduced) are used across both token and legacy confirm routes.
- [ ] Context-store behavior stays explicit and test-backed.

## 8. Test Coverage Gate
- [ ] Updated or added tests in [tests/test_routes.py](tests/test_routes.py).
- [ ] Updated or added tests in [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py).
- [ ] Updated or added end-to-end anchors in [tests/test_regression_e2e.py](tests/test_regression_e2e.py) when cross-page flow changes.
- [ ] Token lifecycle changes include valid-token and invalid/expired/missing-token regression coverage.

## 9. Documentation Gate
- [ ] [docs/Specs/import-backend-spec.md](docs/Specs/import-backend-spec.md) is updated for behavior changes.
- [ ] [docs/Specs/UI/import-ui-spec.md](docs/Specs/UI/import-ui-spec.md) remains aligned with current UI behavior.
- [ ] [docs/User-Facing-Guidance/IMPORT_WORKFLOW.md](docs/User-Facing-Guidance/IMPORT_WORKFLOW.md) remains aligned with current user flow.
- [ ] Focused import deep-dive specs remain synchronized when their scope changes.

## 10. Deep-Dive Cross-Check (Required)
- [ ] Run the first import actions checklist: [docs/Specs/first-import-actions-refactor-checklist.md](docs/Specs/first-import-actions-refactor-checklist.md).
- [ ] Run the folder assignment checklist: [docs/Specs/import-folder-assignment-refactor-checklist.md](docs/Specs/import-folder-assignment-refactor-checklist.md).
- [ ] Run the format support checklist: [docs/Specs/import-format-support-refactor-checklist.md](docs/Specs/import-format-support-refactor-checklist.md).
- [ ] Any approved exceptions are captured in the PR description with rationale and risk notes.