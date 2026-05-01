# Refactor Phase 1B — Launch Polish and Verification Plan

> **Note:** Ignore the `safe-refactor` skill for this phase. This is a practical pre-launch cleanup and verification pass, not a broad refactor exercise.

## Goal

Refresh the original Phase 1 plan so it matches the **current** codebase and focuses only on the remaining pre-launch polish work.

A large part of the earlier cleanup has already landed. This pass is intended to:

- verify that the completed cleanup still holds
- tighten a few remaining public-risk areas
- add tests only where present-day behaviour is still under-protected

## Current Position

The following items now appear to be **already complete or largely complete** and should be treated as verification items rather than new refactor work:

- Gemini integration extracted behind `src/services/gemini_client.py`
- `Tag` / `tags` naming standardised in active application code
- Windows Explorer launch behaviour moved away from `shell=True`
- `ruff` and `black` configuration present in `pyproject.toml`
- stray `print()` debugging removed from `src/**`

## Active Scope

This refreshed phase focuses on **safe, high-value cleanup that is still worth doing**:

- review route-level and external-boundary exception handling
- verify logging quality and clarity around public actions and failures
- extend coverage for current fallback/error paths where helpful
- review bulk import maintainability without forcing a broad rewrite
- clean up any stale wording or docs that no longer match the live app
- verify Windows/native folder-open behaviour remains safe and test-friendly

## Out of Scope

Do **not** include any of the following in this pass:

- database redesign
- authentication or user roles
- UI redesign
- packaging/distribution overhaul
- breaking route changes
- schema-breaking model changes
- large architectural rewrites
- bulk-import restructuring without a specific current pain point

## Implementation Plan

### 1. Re-Baseline the Safety Net

Confirm the current baseline with targeted tests:

- `tests/test_routes.py`
- `tests/test_services.py`
- `tests/test_bulk_import_extra.py`
- `tests/test_gemini_client.py`

Extend coverage only where needed for:

- browse/search behaviour
- bulk import edge cases
- AI fallback and resilience paths
- folder-open and external-launch suppression behaviour

### 2. Logging and Exception Polish

Review the following files first:

- `src/main.py`
- `src/routes/designs.py`
- `src/routes/maintenance.py`
- `src/routes/tagging_actions.py`
- `src/services/auto_tagging.py`
- `src/services/bulk_import.py`
- `src/services/folder_picker.py`

Tasks:

- confirm `src/**` remains free of stray `print()` debugging
- keep using `logging.getLogger(__name__)`
- narrow `except Exception` blocks where the likely failure mode is now well understood
- keep intentionally broad resilience boundaries clearly logged
- improve error messages where they affect user-facing actions or diagnostics

### 3. AI Boundary Follow-Through

The Gemini boundary is already extracted, so this step is now an **audit and test** task rather than a fresh refactor.

- keep `src/services/gemini_client.py` as the dedicated Gemini integration boundary
- preserve public entry points such as:
  - `suggest_tier2_batch()`
  - `suggest_tier3_vision()`
- add or refine tests only where needed for:
  - missing API key handling
  - retry/backoff behaviour
  - malformed responses
  - successful tagging path
  - fallback/error aggregation behaviour

### 4. Bulk Import Maintainability Review

Treat `src/services/bulk_import.py` as a **targeted cleanup area**, not an automatic rewrite.

Focus on whether the current structure remains easy enough to follow across:

- scanning and deduplication
- preview/metadata extraction
- persistence and tagging orchestration

Only split further responsibilities if a concrete maintenance problem or test gap justifies it.

### 5. Naming and Wording Cleanup

- keep the codebase standardised on `Tag` / `tags`
- remove stale comments, docstrings, or plan text that still implies older naming or compatibility layers are active
- prefer updating wording over reopening already-completed migration work

### 6. Windows / Native Launch Verification

Review folder-open and Explorer-launch behaviour in:

- `src/routes/designs.py`
- `src/routes/maintenance.py`
- `src/services/folder_picker.py`

Tasks:

- confirm there is no `shell=True` usage in active app code
- preserve safe Windows-native behaviour
- ensure these actions stay suppressible during tests and non-interactive runs

## Verification

Run these checks during and after implementation.

### Targeted checks

```bash
pytest tests/test_routes.py -q
pytest tests/test_services.py -q
pytest tests/test_bulk_import_extra.py -q
pytest tests/test_gemini_client.py -q
```

### Code hygiene spot-checks

```bash
grep -R "print(" src
grep -R "shell=True" src
```

### Final proof before closing this phase

```bash
pytest -q
pytest --cov=src --cov-report=term-missing
```

### Manual smoke test

Confirm the following still work:

- disclaimer flow
- `/designs/` browse/search
- design detail page
- bulk import scan and preview
- admin tag management and tag actions
- settings/maintenance actions
- folder-open actions with safe Windows behaviour
- clean startup/log output with no stray debug prints

## Success Criteria

This refreshed phase is complete when:

- all existing targeted tests still pass
- any new tests added cover the remaining risky cleanup areas
- no stray production `print()` debugging remains in `src/**`
- no `shell=True` usage exists in active application code
- route-level and external-boundary logging is clear and intentional
- the Gemini boundary remains clean and well-tested
- the bulk import code is acceptable to maintain without unnecessary rewrite
- the public codebase looks deliberate, current, and launch-ready
