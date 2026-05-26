# Colour Counts Refactor Checklist

Use this checklist when changing colour-count extraction, persistence, or backfill behavior.
For ordered removal work, use [docs/Specs/colour-count-decommission-plan.md](docs/Specs/colour-count-decommission-plan.md).

## 1. Contract and Ownership Safety
- [ ] `docs/Specs/colour-count-backend-spec.md` remains feature-specific and does not duplicate shared orchestration content.
- [ ] Shared operational behavior changes (commit/stop/log or endpoint contract) are updated in `docs/Specs/backfilling-backend-spec.md`.
- [ ] Any endpoint/path behavior changes are explicitly classified as canonical or decommission-target.
- [ ] Line-level evidence references in specs are updated to match code.

## 2. Data Model and Semantics
- [ ] `Design.stitch_count`, `Design.color_count`, `Design.color_change_count` semantics remain clear (`None` means unavailable/unparsed).
- [ ] Missing-vs-overwrite behavior is unchanged unless explicitly approved.
- [ ] `.art` behavior remains explicit (metadata fallback vs pattern-method extraction path).
- [ ] Schema/migration changes include backward-safe handling for existing rows.

## 3. Import Flow Integrity
- [ ] Scan-time extraction in `preview.py` still feeds `ScannedDesign` correctly.
- [ ] `ScannedDesign` continues to carry all three count fields to import persistence.
- [ ] `_build_design_records()` still maps all three fields into `Design`.
- [ ] Import path remains a canonical population route for newly imported designs.

## 4. Unified Backfill Semantics
- [ ] Unified `color_counts` selector still targets records with any missing count field unless a deliberate contract change is approved.
- [ ] Runner behavior still handles unreadable patterns safely and deterministically.
- [ ] Result/error accounting remains compatible with unified summary behavior.
- [ ] Any per-action metric changes are documented and covered by tests.

## 5. Performance, Stop, and Logging Compatibility
- [ ] Single-read-per-design behavior remains intact where relevant.
- [ ] Commit cadence changes are intentional and documented in shared spec.
- [ ] Stop signal behavior remains safe for color-count-including runs.
- [ ] Error/info log behavior remains compatible with Tagging Actions UX.

## 6. UI-Only Release Path Enforcement
- [ ] User-facing docs for colour counts only present Admin UI workflows as supported paths.
- [ ] Non-UI routes/scripts are labeled as internal/decommission candidates until removed.
- [ ] Removal/consolidation tasks for duplicate non-UI paths are tracked before release.
- [ ] No new user guidance re-introduces CLI-first instructions for colour counts.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_routes.py](tests/test_routes.py) when route contracts change
- [ ] Legacy-path tests are removed or rewritten when decommission completes.
- [ ] At least one regression test covers missing-field population behavior.

## 8. Documentation Gate
- [ ] `docs/Specs/colour-count-backend-spec.md` matches current code.
- [ ] `docs/User-Facing-Guidance/COLOUR_COUNTS.md` matches current UI labels and flow.
- [ ] Cross-links from `AI_TAGGING.md` and `TAGGING_ACTIONS_BACKFILL.md` remain valid.
- [ ] Changelog entry is added if user-visible behavior changes.
