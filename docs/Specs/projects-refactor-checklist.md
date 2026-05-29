# Projects Refactor Checklist

Use this checklist when changing project routes, project assignment behavior, project print output, or project-related UI contracts.

## 1. Contract Safety
- [ ] Endpoint method/path contracts remain compatible for:
  - `/projects/`
  - `/projects/new`
  - `/projects/{project_id}`
  - `/projects/{project_id}/edit`
  - `/projects/{project_id}/delete`
  - `/projects/{project_id}/remove-design/{design_id}`
  - `/projects/{project_id}/print`
  - `/designs/bulk-add-to-project`
  - `/designs/{design_id}/add-to-project`
  - `/designs/{design_id}/remove-from-project/{project_id}`
- [ ] Request form field names remain backward compatible (`name`, `description`, `project_id`, `design_ids`, `next`) or migration is documented.
- [ ] Redirect targets and status codes remain intentional and documented.
- [ ] Line-level references in [docs/Specs/projects-backend-spec.md](docs/Specs/projects-backend-spec.md) are updated.

## 2. Project CRUD Semantics
- [ ] Project name validation remains non-empty and explicit.
- [ ] Project name uniqueness behavior remains stable and tested.
- [ ] `date_created` behavior remains intentional for new records.
- [ ] Delete behavior continues to remove only project membership links, not design records.

## 3. Assignment Semantics (Single + Bulk)
- [ ] Add-to-project remains idempotent (no duplicate memberships).
- [ ] Bulk add still de-duplicates incoming design IDs.
- [ ] Missing project/design behavior remains intentional and documented.
- [ ] Remove-from-project remains safe for no-op conditions unless contract is intentionally changed.
- [ ] `next` redirect behavior remains stable for browse/detail workflows.

## 4. Print Contract Integrity
- [ ] `/projects/{project_id}/print` remains available and returns `404` for missing projects.
- [ ] Print template keeps core metadata coverage (image/fallback, size, hoop, counts, designer, rating, stitched, notes where present).
- [ ] Printable layout remains usable across common browser print flows.

## 5. Data and Relationship Integrity
- [ ] `project_designs` relationship expectations remain explicit (many-to-many between projects and designs).
- [ ] Cascade assumptions for junction records remain correct and tested.
- [ ] Any ordering assumptions for `project.designs` are documented if introduced.

## 6. Error and Status Semantics
- [ ] Mixed `400` vs `404` behavior remains intentional per endpoint, or normalization migration is documented.
- [ ] Validation failure messages remain actionable.
- [ ] Success paths preserve redirect-based UX continuity.

## 7. UI Contract Alignment
- [ ] Project templates remain aligned with backend route contracts in [docs/Specs/UI/projects-ui-spec.md](docs/Specs/UI/projects-ui-spec.md).
- [ ] Design Detail project controls remain aligned with assignment route contracts.
- [ ] Browse bulk add-to-project integration remains aligned with [docs/Specs/browse-bulk-actions-backend-spec.md](docs/Specs/browse-bulk-actions-backend-spec.md).

## 8. Test Coverage Gate
- [ ] Updated/added route tests in [tests/test_routes.py](tests/test_routes.py) for changed behavior.
- [ ] Updated/added service tests in [tests/test_services.py](tests/test_services.py) for changed assignment/validation semantics.
- [ ] Not-found and validation paths have regression coverage.
- [ ] Print-route behavior has regression coverage when print template or route behavior changes.

## 9. Documentation Gate
- [ ] Current Behavior and/or Target Architecture sections in [docs/Specs/projects-backend-spec.md](docs/Specs/projects-backend-spec.md) are updated.
- [ ] UI contract updates are reflected in [docs/Specs/UI/projects-ui-spec.md](docs/Specs/UI/projects-ui-spec.md).
- [ ] User-visible updates are reflected in [docs/User-Facing-Guidance/PROJECTS.md](docs/User-Facing-Guidance/PROJECTS.md).
- [ ] [docs/feature-inventory.md](docs/feature-inventory.md) section 5 remains accurate and linked.
- [ ] Changelog/release notes are updated for externally visible behavior changes.

## 10. Cross-Spec Consistency
- [ ] Terminology is consistent across backend spec, UI spec, checklist, and user guide (`Projects`, `Print Sheet`, `Add to Project`, `Remove`).
- [ ] Current-behavior statements do not conflict across docs.
- [ ] Target-architecture statements are clearly marked as future direction.