# Browse Bulk Actions Refactor Checklist

Use this checklist when changing Browse-page bottom-banner bulk actions, including bulk verify, bulk set tags, and bulk add-to-project behavior.

## 1. Contract Safety
- [ ] Endpoint method/path contracts remain compatible for:
  - `/designs/bulk-verify`
  - `/designs/bulk-set-tags`
  - `/designs/bulk-add-to-project`
- [ ] Request form field names remain compatible (`design_ids`, `tag_ids`, `project_id`, `next`) or migration is documented.
- [ ] Redirect behavior (`303` to `next` or `/designs/`) remains stable.
- [ ] Line-level references in [docs/Specs/browse-bulk-actions-backend-spec.md](docs/Specs/browse-bulk-actions-backend-spec.md) are updated.

## 2. Selection and Banner Semantics
- [ ] Bottom banner visibility still depends on one-or-more selected card checkboxes.
- [ ] Selected-count label remains accurate through individual, select-all, and clear operations.
- [ ] Select-all still applies to visible cards on current page only.
- [ ] Clear selection reliably resets card and select-all states.

## 3. Tag Modal Semantics
- [ ] Choose tags still opens modal workflow (not route navigation) unless migration is documented.
- [ ] Modal pre-tick logic still reflects intersection of currently selected designs' tags.
- [ ] Apply tags behavior still maps to replace-tags semantics in backend route contract.
- [ ] Modal close paths (X, cancel, backdrop, Escape) remain functional.

## 4. Verify and Project Assignment Integrity
- [ ] Verify selected still marks `tags_checked=True` without changing tags.
- [ ] Bulk add-to-project still validates project and selected IDs through shared project service.
- [ ] Duplicate selected IDs do not create duplicate project memberships.
- [ ] Project selector disabled state remains explicit when no projects exist.

## 5. State and Navigation Behavior
- [ ] Hidden form `next` fields continue to propagate current Browse URL.
- [ ] Pagination retains filter/sort state while not carrying card-check selection state.
- [ ] Browse page state remains coherent after redirect from bulk actions.
- [ ] Row selectors and card selectors remain synchronized after resize and selection changes.

## 6. Error Handling and Resilience
- [ ] Bulk add route still maps service `ValueError` to `400` consistently.
- [ ] Empty-selection submissions remain safe no-op redirects unless a new UX contract is documented.
- [ ] Routes continue to avoid partial-commit corruption for normal and edge paths.
- [ ] Logging remains sufficient for operational diagnosis of bulk actions.

## 7. Test Coverage Gate
- [ ] Updated/added tests in [tests/test_routes.py](tests/test_routes.py) for any behavior changes.
- [ ] Existing route tests remain green:
  - `test_bulk_verify`
  - `test_bulk_set_tags`
  - `test_bulk_add_to_project`
- [ ] Selection-state changes have at least one UI/route regression anchor.
- [ ] Pagination/selection interaction changes have explicit regression coverage.

## 8. Documentation Gate
- [ ] Current Behavior sections updated in:
  - [docs/Specs/browse-bulk-actions-backend-spec.md](docs/Specs/browse-bulk-actions-backend-spec.md)
  - [docs/Specs/UI/browse-bulk-actions-ui-spec.md](docs/Specs/UI/browse-bulk-actions-ui-spec.md)
  - [docs/User-Facing-Guidance/BROWSE_BULK_ACTIONS.md](docs/User-Facing-Guidance/BROWSE_BULK_ACTIONS.md)
- [ ] [docs/feature-inventory.md](docs/feature-inventory.md) remains concise and accurate.
- [ ] [docs/User-Facing-Guidance/HELP.md](docs/User-Facing-Guidance/HELP.md) cross-links remain accurate.
- [ ] Any externally visible behavior changes are reflected in changelog/release notes.

## 9. Cross-Spec Consistency
- [ ] Terminology remains consistent across backend spec, UI spec, and user guidance (`bottom banner`, `Choose tags...`, `Verify selected`, `Add to project`, `Clear selection`).
- [ ] Current-behavior statements remain non-contradictory across all browse bulk-actions docs.
- [ ] Any target-behavior statements are clearly marked as future-direction only.