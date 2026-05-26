# Colour Counts Pre-Release Decommission Plan

## Status
- Type: Implementation plan (code removal/consolidation)
- Audience: Agents
- Last validated: 2026-05-24
- Scope: Remove duplicate non-UI colour-count execution paths before release

## Goal
Retain one supported execution path for colour-count updates:
- Admin Tagging Actions unified backfill action: color_counts

Remove or consolidate duplicate paths:
- Maintenance route POST /admin/maintenance/backfill-color-counts
- Standalone script backfill_color_counts.py
- Legacy UI sections that promote non-unified forms/CLI

## Non-Goals
- Do not change colour-count data semantics.
- Do not change unified backfill route contract.
- Do not change import-time extraction behavior.

## Current References To Remove Or Migrate

### Maintenance route and direct form
- Route implementation: removed (Phase 2 complete)
- Direct form in Tagging Actions template: removed (Phase 1 complete)
- Legacy tests: removed/rewritten (Phase 4 complete)

### Standalone script
- Script: `backfill_color_counts.py` (removed in Phase 3)
- Planning references (non-runtime):
  - [plans/refactor-eliminate-duplicate-preview-stitching-colour.md](plans/refactor-eliminate-duplicate-preview-stitching-colour.md#L143)
  - [plans/backfill-performance-optimization.md](plans/backfill-performance-optimization.md#L179)

### Unified path to keep
- Unified route: [src/routes/tagging_actions.py](src/routes/tagging_actions.py#L79)
- Unified selector/runner: [src/services/unified_backfill.py](src/services/unified_backfill.py#L768)

## Ordered Implementation Plan

## Phase 1 - UI consolidation first (safe, user-facing)
1. Remove legacy individual colour-count form block from [templates/admin/tagging_actions.html](templates/admin/tagging_actions.html). ✅ Completed
2. Remove CLI tools panel from [templates/admin/tagging_actions.html](templates/admin/tagging_actions.html) or rewrite it to UI-only guidance. ✅ Completed
3. Ensure only unified action controls remain for Threads and Colours.

Expected result:
- Users can only trigger colour-count updates from unified backfill UI.

## Phase 2 - Route/API decommission
1. Remove colour-count maintenance handler from [src/routes/maintenance.py](src/routes/maintenance.py). ✅ Completed
2. Remove any imports used only by that handler.
3. If router registration exposes route-level docs or navigation links, update them accordingly.

Expected result:
- No server endpoint remains for /admin/maintenance/backfill-color-counts.

## Phase 3 - Script decommission
1. Delete `backfill_color_counts.py`. ✅ Completed
2. Remove references from docs that present script as active workflow.
3. Keep archival references only in historical plan docs if needed.

Expected result:
- No parallel script path remains for colour-count updates.

## Phase 4 - Tests and assertions migration
1. Remove or rewrite maintenance-route tests in [tests/test_legacy_tagging_actions.py](tests/test_legacy_tagging_actions.py). ✅ Completed
2. Keep and strengthen unified colour-count tests in:
   - [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
   - [tests/test_regression_e2e.py](tests/test_regression_e2e.py#L177)
3. Add one route-level assertion in [tests/test_routes.py#L350](tests/test_routes.py#L350) that deprecated maintenance endpoint is absent or returns not found if coverage style supports it. ✅ Completed

Expected result:
- Test suite reflects UI-only + unified-only architecture.

## Phase 5 - Documentation cleanup
1. Update [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md) to move decommission-candidate notes to removed/completed state. ✅ Completed
2. Update [docs/Specs/colour-count-backend-spec.md](docs/Specs/colour-count-backend-spec.md) to remove transitional wording once code removal lands. ✅ Completed
3. Keep user guidance in [docs/User-Facing-Guidance/COLOUR_COUNTS.md](docs/User-Facing-Guidance/COLOUR_COUNTS.md) UI-only.

Expected result:
- No docs mention deprecated colour-count route/script as active behavior.

## Risk Controls
- Remove user-facing UI entrypoints before deleting backend route to prevent accidental usage drift.
- Keep unified regression tests green before route/script removal commits.
- Ship in small commits by phase to simplify rollback.

## Suggested Commit Sequence
1. commit: Remove legacy colour-count UI form and CLI panel mentions
2. commit: Remove maintenance colour-count route and dead imports
3. commit: Delete backfill_color_counts.py and update docs references
4. commit: Replace legacy tests with unified-route assertions
5. commit: Final spec/doc cleanup

## Exit Criteria
- No runtime references to /admin/maintenance/backfill-color-counts.
- No runtime script `backfill_color_counts.py`.
- Tagging Actions UI exposes only unified Threads and Colours action.
- Unified colour-count regression tests pass.
- Specs and user docs align with UI-only architecture.
