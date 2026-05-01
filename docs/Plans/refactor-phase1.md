# Refactor Phase 1 — Public Launch Cleanup Plan

> **Note:** Ignore the `safe-refactor` skill for this phase. This is a practical pre-launch cleanup pass, not a broad refactor exercise.

## Goal

Improve the app's public-facing code quality and maintainability without destabilising behaviour before launch.

## Scope

This phase focuses on **safe, high-value cleanup**:

- replace stray `print()` debugging with structured logging
- tighten broad exception handling where practical
- add or extend tests around the highest-risk paths
- extract the Gemini/AI boundary into a dedicated module
- reduce the size and responsibility of the bulk import code
- standardise on `Tag` / `tags` naming in new code while preserving compatibility
- clean up Windows Explorer launch behaviour

## Out of Scope

Do **not** include any of the following in Phase 1:

- database redesign
- authentication or user roles
- UI redesign
- packaging/distribution overhaul
- breaking route changes
- schema-breaking model changes
- large architectural rewrites

## Implementation Plan

### 1. Safety Net and Quality Guardrails

- confirm the current baseline with targeted tests:
  - `tests/test_routes.py`
  - `tests/test_services.py`
  - `tests/test_bulk_import_extra.py`
- extend coverage only where needed for:
  - browse/search behaviour
  - bulk import edge cases
  - AI fallback paths
  - folder-open actions
- add lightweight tooling configuration in `pyproject.toml` for formatting/linting (`ruff`, `black`)

### 2. Logging and Exception Hygiene

Update the following files first:

- `src/services/designs.py`
- `src/services/bulk_import.py`
- `src/services/auto_tagging.py`
- `src/main.py`
- `src/routes/designs.py`
- `src/routes/maintenance.py`

Tasks:

- replace `print()` calls with `logging.getLogger(__name__)`
- use sensible log levels: `debug`, `info`, `warning`, `exception`
- narrow `except Exception` blocks where the failure modes are known
- keep any intentionally broad resilience boundaries clearly logged

### 3. Extract the AI Integration Boundary

- move Gemini-specific request/retry/response parsing out of `src/services/auto_tagging.py`
- create a dedicated integration module, e.g. `src/services/gemini_client.py`
- preserve current public entry points such as:
  - `suggest_tier2_batch()`
  - `suggest_tier3_vision()`
- add tests for:
  - missing API key
  - retry/backoff behaviour
  - malformed responses
  - successful tagging path

### 4. Bulk Import Cleanup

Refactor `src/services/bulk_import.py` into smaller responsibilities:

- scanning and deduplication
- preview/metadata extraction
- persistence and tagging orchestration

Keep the current top-level workflow and behaviour stable.

### 5. Naming Consistency Cleanup

- standardise the codebase on `Tag` / `tags`
- remove old compatibility aliases and deprecated endpoints once callers are updated
- update comments/docstrings where the old wording is misleading

### 6. Windows Shell Cleanup

- review Explorer-launch code in:
  - `src/routes/designs.py`
  - `src/routes/maintenance.py`
- replace `shell=True` usage where a safer Windows-native call preserves behaviour

## Verification

Run these checks during and after implementation:

### Targeted checks

```bash
pytest tests/test_routes.py -q
pytest tests/test_services.py -q
pytest tests/test_bulk_import_extra.py -q
```

### Final proof before closing Phase 1

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
- admin tag management
- settings/maintenance actions
- clean startup/log output with no stray debug prints

## Success Criteria

Phase 1 is complete when:

- all existing tests still pass
- targeted new tests cover the risky cleanup areas
- no stray production `print()` debugging remains
- logs are cleaner and more intentional
- the AI code has a clearer boundary
- bulk import responsibilities are easier to follow
- the public codebase looks more deliberate and less improvised
