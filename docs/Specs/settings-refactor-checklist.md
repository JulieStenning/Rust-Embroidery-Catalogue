# Settings Refactor Checklist

Use this checklist when changing settings keys/defaults, settings route contracts, settings persistence behavior, or app-wide settings consumption.

## 1. Contract Safety
- [ ] `/admin/settings/` and `/admin/settings/browse-data-root` route contracts remain compatible, or migration is documented.
- [ ] Redirect query keys (`saved`, `error`) remain compatible with settings page rendering.
- [ ] Folder-picker JSON contract remains stable (`path` success or `error` fallback).
- [ ] Any form field renames/additions are backward compatible for current UI flow.
- [ ] Line-level references in [docs/Specs/settings-backend-spec.md](docs/Specs/settings-backend-spec.md) are updated.

## 2. Key Registry and Defaults Integrity
- [ ] Existing key names are unchanged unless migration and fallback behavior are documented.
- [ ] `_DEFAULTS` and `_DESCRIPTIONS` remain aligned for all supported keys.
- [ ] Unknown-key behavior remains safe (`""` fallback) and intentional.
- [ ] Settings writes preserve idempotency for repeated submissions.

## 3. Input Normalization and Validation
- [ ] Batch-size normalization retains blank/invalid coercion and clamping behavior.
- [ ] Runtime parse semantics for settings-backed int values remain compatible with route/service callers.
- [ ] Checkbox omission semantics (unchecked => persisted `false`) remain explicit and tested.
- [ ] `image.preference` validation remains whitelist-based (`2d`/`3d`) unless documented.
- [ ] Backup destination path normalization behavior remains intentional and consistent.

## 4. Storage and Secrets Safety
- [ ] `.env` API-key update behavior preserves non-key lines/comments and remains deterministic.
- [ ] Blank API-key saves remove key entry safely.
- [ ] API key handling avoids accidental leakage in logs, UI summaries, or exported diagnostics.
- [ ] DB settings writes and refresh behavior remain atomic from caller perspective.

## 5. Desktop Data-Root and Mode Boundaries
- [ ] Desktop-only data-root updates remain gated to desktop mode.
- [ ] Data-root save creates required subfolders safely and predictably.
- [ ] Managed-data copy behavior remains best-effort and non-fatal.
- [ ] Runtime config/env updates after data-root change remain coherent.
- [ ] Portable/non-desktop behavior remains unaffected by desktop-only paths.

## 6. Cross-Surface Consumer Compatibility
- [ ] Import routes still read expected settings keys and parse semantics.
- [ ] Tagging Actions route still reflects settings-backed AI defaults.
- [ ] Backup routes still consume backup destination settings correctly.
- [ ] Disclaimer gate and disclaimer route retain acceptance semantics.
- [ ] Overlap boundaries with companion specs remain accurate and non-contradictory.

## 7. Resilience and User Feedback
- [ ] Folder-picker unavailable path remains non-fatal with actionable error feedback.
- [ ] Settings save failures for API key/data-root preserve redirect + error UX.
- [ ] Successful saves preserve redirect + success UX.
- [ ] Partial optional input submissions do not corrupt persisted settings state.

## 8. Test Coverage Gate
- [ ] Updated/added route tests in [tests/test_routes.py](tests/test_routes.py) for changed contract paths.
- [ ] Updated/added service tests in [tests/test_services.py](tests/test_services.py) for changed persistence/normalization paths.
- [ ] If import/tagging/backup settings consumption changes, add or update consumer regression tests.
- [ ] New behavior has at least one route-level and one service-level test.

## 9. Documentation Gate
- [ ] [docs/Specs/settings-backend-spec.md](docs/Specs/settings-backend-spec.md) is updated for behavior changes.
- [ ] [docs/User-Facing-Guidance/SETTINGS.md](docs/User-Facing-Guidance/SETTINGS.md) is updated for user-visible setting changes.
- [ ] Companion specs are updated when overlap ownership changes:
  - [docs/Specs/batch-tagging-backend-spec.md](docs/Specs/batch-tagging-backend-spec.md)
  - [docs/Specs/backup-restore-backend-spec.md](docs/Specs/backup-restore-backend-spec.md)
  - [docs/Specs/first-import-actions-backend-spec.md](docs/Specs/first-import-actions-backend-spec.md)
- [ ] Any externally visible behavior change has a changelog entry.
