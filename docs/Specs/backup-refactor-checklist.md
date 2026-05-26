# Backup Refactor Checklist

Use this checklist when changing backup routes, backup service behavior, or restore documentation.

## 1. Contract Safety
- [ ] Endpoint path and method remain compatible, or migration plan is documented.
- [ ] Backup route redirect query keys remain compatible with backup page rendering.
- [ ] Folder-picker JSON contract (`path` or `error`) remains backward compatible.
- [ ] Line-level references in [docs/Specs/backup-backend-spec.md](docs/Specs/backup-backend-spec.md) are updated.

## 2. Destination Settings and Path Handling
- [ ] `backup.database_destination` and `backup.designs_destination` keys are unchanged or migration is documented.
- [ ] Destination path normalization/display behavior remains intentional and tested.
- [ ] Save button enable/disable logic remains aligned to saved values.
- [ ] Unsaved edits do not affect backup action preconditions.

## 3. Database Backup Semantics
- [ ] SQLite backup API remains first-choice path for live consistency.
- [ ] Fallback-to-copy behavior remains narrow and explicit.
- [ ] Timestamped filename contract remains documented and stable.
- [ ] Result payload (`db_ok`, `db_path`, `db_size`, `db_time`) remains compatible.

## 4. Designs Backup Semantics
- [ ] Incremental compare remains based on relative path, size, and modification time.
- [ ] Missing-from-source files are archived to `_deleted/YYYY-MM-DD/` (not hard deleted).
- [ ] Empty-folder cleanup excludes `_deleted` tree.
- [ ] Result payload (`d_scanned`, `d_copied`, `d_updated`, `d_unchanged`, `d_archived`, `d_bytes`, `d_time`) remains compatible.

## 5. Error and Resilience Behavior
- [ ] Missing source and invalid destination failures still return user-actionable messages.
- [ ] Non-fatal per-file designs errors remain logged without crashing the entire run.
- [ ] Combined backup (`/backup/both`) preserves partial-success reporting.
- [ ] Backup-in-progress overlay and user messaging remain accurate.

## 6. Restore Integrity Gate
- [ ] If restore behavior changes, [docs/BACKUP_RESTORE.md](docs/BACKUP_RESTORE.md) is updated in the same change.
- [ ] If in-app restore endpoints are added, this checklist and [docs/Specs/backup-backend-spec.md](docs/Specs/backup-backend-spec.md) are updated with contracts.
- [ ] Manual restore fallback remains documented until in-app restore is production-ready.
- [ ] Migration behavior after restore remains documented and validated.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_routes.py](tests/test_routes.py)
  - [tests/test_services.py](tests/test_services.py)
- [ ] New behavior has at least one route-level and one service-level test.
- [ ] Archive and cleanup semantics have explicit regression coverage.
- [ ] Database backup consistency behavior has regression coverage when changed.

## 8. Documentation Gate
- [ ] Current Behavior section updated for any implemented backup/restore change.
- [ ] Target Architecture section updated if roadmap direction changes.
- [ ] Endpoint-level contract details are kept current for agent consumption.
- [ ] Changelog entry added if behavior is externally observable.
