# Refactor Phase 2 — Structural Maturity Plan

> **Note:** Ignore the `safe-refactor` skill for this phase. This is still the post-launch **“do later”** plan and should stay pragmatic, local-first, and deliberately anti-overengineering.

> **Refresh (April 2026):** This version replaces the earlier draft. Since that draft, the app has gained managed-only multi-folder import, first-run database bootstrap, canonical tag naming, backup/admin tooling, SD-card launcher flow, and broader public docs. The current suite is also green (`pytest -q` → **687 passed, 0 failed** on 2026-04-08). Phase 2 should therefore focus on **consolidation and contributor clarity**, not foundational rewrites.

## Goal

Take the app from **public-ready and feature-rich** to **cleaner, calmer, and easier to maintain**, without turning a sensible local FastAPI/Jinja/SQLite application into an enterprise science project.

This phase is about:
- clearer boundaries in the biggest modules
- safer validation and path handling across app + launcher tooling
- more intentional failure handling for import, backup, and AI jobs
- more consistent logs and runtime behaviour
- better contributor experience and more repeatable releases

It is **not** about rewriting the app for its own sake.

## Current-state implications

A few parts of the original plan are now less urgent than they were:

- `src/main.py` is already reasonably thin and should **not** become a refactor project by itself.
- the route/service split is already in place and mostly working well.
- managed-only file storage is now the live model; Phase 2 should refine it, not revisit the old external-path idea.
- `docs/feature-inventory.md` already acts as a current feature baseline and should be treated as supporting source-of-truth documentation.

So the Phase 2 focus should move toward the **remaining friction points**:
- path safety and reusable validation
- oversized import / launcher modules
- better failure wording and transaction boundaries
- template reuse
- contributor workflow and release repeatability

## Start Conditions

Only begin active Phase 2 work once all of the following are true:

- the current release baseline is stable
- the test suite remains green
  - current evidence: `pytest -q` → **687 passed, 0 failed** on 2026-04-08
- public docs remain stable enough that a cleanup pass will not immediately be invalidated
- at least one real-world feedback pass has been gathered from actual use
- there are no urgent bugfixes or launch-critical regressions ahead of it

## Guiding Principles

- keep the app **local-first**
- keep **FastAPI + Jinja2 + SQLite** as the default architecture
- prefer **extraction, organisation, and documentation** over rewrites
- avoid breaking routes, schema, or deployment scripts unless there is a very clear payoff
- fix friction that a future contributor or power user will actually feel
- favour small, reversible cleanups over “grand redesign” energy

## Scope

Phase 2 now focuses on these medium-term improvements:

- centralised validation and path safety for app + launcher flows
- clearer error recovery and transaction boundaries for import, tagging, and backup operations
- shared logging patterns across the app and root scripts
- better organisation of the heaviest modules
- clearer runtime/configuration source-of-truth rules
- template and form reuse
- CI, contributor docs, and more repeatable Windows release steps

## Out of Scope

Do **not** include the following in Phase 2 unless the product direction changes:

- microservices
- React/Vue/Svelte rewrite
- Redis or distributed caching
- async database rework for its own sake
- GraphQL or API versioning machinery
- mandatory user accounts
- a full permissions/RBAC system
- database redesign
- major visual redesign

If the app stays **localhost-only**, avoid inventing complexity.

## Implementation Plan

### 1. Consolidate the biggest modules and document the intended boundaries

The app already has the right general architecture. Phase 2 should now tighten the **heaviest files**, not reorganise everything just to feel tidy.

**Target outcomes**
- keep `src/main.py` focused on app wiring and middleware
- keep route modules mostly about HTTP/UI behaviour
- keep business logic in services
- keep launcher / batch-script concerns clearly separated from the web app

**Priority files**
- `src/routes/bulk_import.py`
- `src/services/bulk_import.py`
- `src/routes/designs.py`
- `src/services/auto_tagging.py`
- `portable_launcher.py`

**Work**
- add a lightweight `docs/ARCHITECTURE.md` explaining the current route/service/script split
- split oversized modules only where it clearly improves readability or testability
- move duplicated helper logic into small, well-named modules instead of a generic “utils” dumping ground
- treat `portable_launcher.py` as a specific cleanup target because it now mixes registry, validation, UI, and process-launch concerns

---

### 2. Centralise validation and file/path safety

The app is now even more file-system-driven than when this plan was first written. Path handling should become more reusable and more explicit across import, maintenance, and portable deployment flows.

**Priority files**
- `src/services/validation.py`
- `src/services/folder_picker.py`
- `src/routes/bulk_import.py`
- `src/routes/designs.py`
- `src/routes/maintenance.py`
- `portable_launcher.py`
- `populate_sdcard.bat`

**Work**
- extend `validation.py` beyond ratings/strings to include:
  - path normalisation
  - “must exist / must be directory” checks
  - “must remain under configured base path” checks
  - clearer user-facing validation messages
- keep the rules consistent between the web app, folder picker helpers, and launcher tooling
- make Windows/UNC/removable-drive behaviour explicit
- ensure browse/open actions fail cleanly and informatively when external launches are suppressed or unavailable

**Why this matters**
This is still one of the likeliest “public embarrassment” areas if it stays fuzzy.

---

### 3. Sharpen error recovery and transaction boundaries

Failures should feel intentional rather than improvised, especially now that the app covers import, backup, maintenance, and AI-assisted tagging.

**Priority files**
- `src/services/bulk_import.py`
- `src/services/auto_tagging.py`
- `src/services/backup_service.py`
- `src/routes/bulk_import.py`
- `src/routes/maintenance.py`
- `src/routes/tagging_actions.py`
- `src/database.py`

**Work**
- separate failure types more clearly:
  - missing file
  - unreadable or unsupported file
  - permission issue
  - path outside managed storage
  - backup destination not writable
  - Gemini timeout / quota / malformed response
- define when the app should:
  - retry
  - skip
  - warn
  - stop
- tighten multi-step import and tagging flows so DB writes are committed or rolled back deliberately
- improve user-facing wording for import, backup, and maintenance errors

**Target result**
A failed import, backup, or tagging run should be annoying, not confusing.

---

### 4. Finish logging consistency across scripts and launcher tooling

The core app now uses logging in several important places. Phase 2 should finish the job for the remaining root scripts and operator-facing tooling.

**Priority files**
- `auto_tag.py`
- `backfill_images.py`
- `portable_launcher.py`
- `src/main.py`

**Work**
- define one shared logging pattern for:
  - app startup
  - imports
  - tagging jobs
  - maintenance / backup tasks
  - launcher/runtime issues
- prefer `info`, `warning`, and `exception` over ad hoc `print()` output where it improves consistency
- keep CLI/script output human-readable by default
- only add file logging or JSON logging if it solves a real support problem

**Target result**
Logs should help diagnose a bad overnight run or launcher issue in minutes, not make you scroll in irritation.

---

### 5. Clarify configuration and runtime source-of-truth rules

The project now spans local dev, portable/SD-card use, helper scripts, `.env` settings, DB settings, and launcher registry values. The behaviour is workable, but the boundaries should be easier to understand.

**Priority files**
- `src/config.py`
- `src/services/settings_service.py`
- `start.bat`
- `stop.bat`
- `setup.bat`
- `populate_sdcard.bat`
- `portable_launcher.py`
- `README.md`
- `docs/GETTING_STARTED.md`
- `docs/USB_DEPLOYMENT.md`

**Work**
- clearly separate and document:
  - environment configuration (`.env`)
  - persisted app settings (DB `settings` table)
  - one-off installation markers (e.g. disclaimer acceptance file)
  - launcher/operator convenience state (Windows Registry)
- make it obvious which settings are machine-local, installation-local, user-editable, or optional
- add a short runtime/config summary to docs or the About/Settings area
- remove any remaining ambiguity caused by older mental models of “external vs managed” storage

**Target result**
A future maintainer should not need to reverse-engineer how dev, portable, and helper-script modes are supposed to differ.

---

### 6. Template and form consistency cleanup

The UI still does not need a redesign, but Phase 2 should make template work cheaper and less repetitive.

**Priority files**
- `templates/base.html`
- `templates/import/step1_folder.html`
- `templates/import/step2_review.html`
- `templates/import/step3_precheck.html`
- `templates/admin/settings.html`
- `templates/admin/tagging_actions.html`
- related templates under `templates/admin/`, `templates/designs/`, and `templates/info/`

**Work**
- extract repeated form/error/success/notice markup into Jinja partials or macros
- standardise button styles, notices, and empty-state messages
- make admin pages feel like part of one coherent app
- keep dark-mode and print-mode handling tidy and centralised

**Target result**
Template changes should become cheaper and less error-prone.

---

### 7. Release engineering and contributor experience

The repo is now public-facing enough that Phase 2 should make it easier for another developer to trust, run, and contribute to it.

**Priority files**
- `pyproject.toml`
- `requirements.txt`
- `README.md`
- `docs/GETTING_STARTED.md`
- `docs/TROUBLESHOOTING.md`
- `build_launcher.bat`

**New files to consider**
- `docs/ARCHITECTURE.md`
- `CONTRIBUTING.md`
- `DEVELOPMENT.md`
- `.editorconfig`
- `.pre-commit-config.yaml`
- simple GitHub Actions workflows for `pytest -q` and `ruff check`
- an optional locked dependency file for repeatable Windows/public releases

**Work**
- wire the existing test/lint tooling into a minimal CI check
- document the expected development workflow and release smoke-test flow
- clarify versioning / release expectations
- reduce onboarding dependence on “tribal knowledge”

**Target result**
A competent stranger should be able to clone the repo and understand how to work on it without guessing.

---

### 8. Optional security boundary — only if the app will be shared beyond localhost

This remains conditional.

**If the app remains personal/local-only**
- keep this low priority
- do not add auth just to look “professional”

**If the app may be exposed over a LAN or shared machine**
- consider a minimal admin gate for:
  - `/admin/settings/`
  - `/admin/maintenance/`
  - import actions
  - tagging/backup actions
- keep it small and practical
- do not build a full account system unless there is a real use case

## Recommended Execution Order

1. architecture notes and targeted big-module cleanup
2. validation/path safety across app + launcher flows
3. error recovery and transaction boundaries
4. logging standardisation for scripts and long-running jobs
5. runtime/config source-of-truth cleanup
6. template/form reuse
7. CI, contributor docs, and release engineering
8. optional admin/security boundary only if deployment scope expands

## Short Implementation Checklist

Use this as the practical working list when Phase 2 actually starts.

### Phase 2A — Validation and path safety
- [ ] extend `src/services/validation.py` with reusable path checks and clearer messages
- [ ] unify path validation behaviour across `bulk_import`, `designs`, `maintenance`, and `folder_picker`
- [ ] review UNC / removable-drive / suppressed-external-launch behaviour in `portable_launcher.py` and `populate_sdcard.bat`
- [ ] make invalid or inaccessible path failures feel explicit rather than vague

### Phase 2B — Heavy-module cleanup
- [ ] add `docs/ARCHITECTURE.md` with the current route/service/script boundaries
- [ ] extract helper logic from `src/routes/bulk_import.py` where it improves readability
- [ ] extract helper logic from `src/services/bulk_import.py` where it improves readability or testability
- [ ] separate registry, validation, and process-launch concerns inside `portable_launcher.py`

### Phase 2C — Error recovery and logging
- [ ] standardise retry / skip / warn / stop rules for import, backup, and AI tagging flows
- [ ] improve user-facing wording for import, maintenance, and backup errors
- [ ] align logging patterns across `auto_tag.py`, `backfill_images.py`, and `portable_launcher.py`
- [ ] keep exception boundaries intentional and easy to test

### Phase 2D — Runtime clarity, templates, and contributor workflow
- [ ] document the source-of-truth roles of `.env`, DB `settings`, disclaimer marker files, and launcher registry values
- [ ] reduce repeated notice/form markup in `templates/import/` and `templates/admin/`
- [ ] add `CONTRIBUTING.md` and/or `DEVELOPMENT.md`
- [ ] add a minimal GitHub Actions workflow for `pytest -q` and `ruff check`

## Verification

### Automated checks

- `pytest -q`
- `pytest --cov=src --cov=portable_launcher --cov-report=term-missing`
- if CI is added: verify the workflow runs cleanly on a normal clone

### Targeted manual checks

- invalid import path
- missing permissions on a folder or backup destination
- removable drive / changed drive letter scenario
- UNC path behaviour where supported
- no `GOOGLE_API_KEY` fallback behaviour
- folder picker suppression when external launches are disabled
- browse/search still behaving as expected
- `start.bat`, `stop.bat`, `populate_sdcard.bat`, and `portable_launcher.py` still feel consistent

### Release smoke test

Run the app from:
- a normal development checkout
- the portable/SD-card flow
- the launcher-assisted operator flow
- a clean Windows test machine or VM if available

## Success Criteria

Phase 2 is complete when:

- the biggest modules are easier for a newcomer to navigate
- path and import failures are safer and clearer across both app and launcher flows
- app logs and script logs feel consistent
- runtime modes and configuration sources are better documented and less implicit
- templates are easier to maintain
- the repo is more contributor-friendly
- release behaviour is more repeatable
- the app still feels like a focused local tool, not an overbuilt framework demo
