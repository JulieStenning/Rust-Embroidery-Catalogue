# Colour Counts Backend Specification

## Status
- Type: Current behavior + release-target architecture
- Audience: Agents
- Last validated: 2026-05-24
- Companion checklist: [docs/Specs/colour-count-refactor-checklist.md](docs/Specs/colour-count-refactor-checklist.md)
- Shared architecture companion: [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md)
- Decommission implementation plan: [docs/Specs/colour-count-decommission-plan.md](docs/Specs/colour-count-decommission-plan.md)

## Purpose
Define backend architecture and behavior for stitch/colour/count metadata:
- `stitch_count`
- `color_count`
- `color_change_count`

This spec is feature-specific. Shared multi-action orchestration details (commit/stop/log contracts, unified endpoint baseline, and broad backfill architecture) are canonical in [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md).

## Scope
In scope:
- Data model and migration shape for colour-count fields.
- Import-time extraction and persistence behavior.
- Unified backfill color-count selection and runner behavior.
- Post-decommission state for removed non-UI color-count paths.
- Test anchors for regression safety.

Out of scope:
- UI rendering details (documented in [docs/User-Facing-Guidance/COLOUR_COUNTS.md](docs/User-Facing-Guidance/COLOUR_COUNTS.md)).
- Non-colour-count tagging/stitching/image action internals.

## Release Target (Pre-Release)
- Canonical user path: Admin Tagging Actions via unified backfill (`color_counts` action).
- Import path remains canonical for newly imported designs.
- Legacy maintenance route and standalone script paths for color-counts are removed.

## Data Contract

### Persisted fields
`Design` stores colour-count metadata as nullable integers:
- `stitch_count`
- `color_count`
- `color_change_count`

Evidence:
- Model columns: [src/models.py#L150](src/models.py#L150)
- Import persistence mapping: [src/services/bulk_import.py#L216](src/services/bulk_import.py#L216)

### Scan-time carrier
`ScannedDesign` transports extracted values from scan to DB persistence.

Evidence:
- Dataclass fields: [src/services/scanning.py#L117](src/services/scanning.py#L117)

### Migration
Schema addition was delivered via Alembic migration:
- [alembic/versions/0010_add_stitch_color_counts.py](alembic/versions/0010_add_stitch_color_counts.py)

## Current Behavior

### 1) Pattern extraction primitive
Colour-count extraction is centralized in `_extract_color_counts()` and consumed through `analyze_pattern()`.

Behavior:
- Fills only missing values by default.
- Wraps each pattern call in local exception handling.
- Returns a tuple `(stitch_count, color_count, color_change_count)`.

Evidence:
- `_extract_color_counts`: [src/services/pattern_analysis.py#L102](src/services/pattern_analysis.py#L102)
- `analyze_pattern` color-count branch: [src/services/pattern_analysis.py#L261](src/services/pattern_analysis.py#L261)

### 2) Import scan integration
`_process_file()` reads pattern data once and calls `analyze_pattern(..., needs_color_counts=True)`.

Behavior:
- Sets `ScannedDesign.stitch_count`, `ScannedDesign.color_count`, `ScannedDesign.color_change_count` from analysis results.
- For `.art`, spider dimensions can override bounds, but colour counts still flow from analysis output when pattern is readable.
- If pattern cannot be read, scan result may keep count fields as `None`.

Evidence:
- Scan path and assignment: [src/services/preview.py#L399](src/services/preview.py#L399)
- `.art` metadata helper available for stitch/color fallback behavior in some paths: [src/services/preview.py#L175](src/services/preview.py#L175)

### 3) Import DB persistence
`_build_design_records()` copies scan-time values into `Design` rows.

Evidence:
- Design constructor mapping: [src/services/bulk_import.py#L216](src/services/bulk_import.py#L216)

### 4) Unified backfill selection semantics
When `color_counts` is selected, unified backfill targets designs where at least one count field is `NULL`.

Selection query:
- `stitch_count IS NULL OR color_count IS NULL OR color_change_count IS NULL`

Evidence:
- Unified selector branch: [src/services/unified_backfill.py#L768](src/services/unified_backfill.py#L768)

### 5) Unified backfill runner semantics
`run_color_counts_action_runner()` applies analysis results to the ORM object.

Behavior:
- If `pattern is None`, returns success (`None`) and silently skips.
- Calls `analyze_pattern(... needs_color_counts=True, existing_* = design fields)`.
- Writes back non-`None` results only.
- Returns string error on exception.

Evidence:
- Runner implementation: [src/services/unified_backfill.py#L358](src/services/unified_backfill.py#L358)

### 6) Additional color-count touchpoint in design service
`src/services/designs.py` includes a convenience auto-backfill on certain design mutations.

Behavior:
- If counts are missing and pattern is readable, it fills missing fields from pattern methods.

Evidence:
- Auto-fill branch: [src/services/designs.py#L282](src/services/designs.py#L282)

### 7) Removed legacy execution paths
Completed removals:
- Maintenance route `POST /admin/maintenance/backfill-color-counts` removed.
- Standalone script `backfill_color_counts.py` removed.

Regression evidence for removed route:
- [tests/test_routes.py#L350](tests/test_routes.py#L350)

## Idempotency and Semantics
- Default behavior is additive/fill-missing: existing non-null values are preserved in extraction and unified action runner.
- `None` indicates unavailable/unparsed data, not zero.

## Error Handling
- Extraction methods catch per-field errors and continue with partial results.
- Unified color-count runner catches exceptions and reports string errors for aggregation.
- Pattern-unreadable designs are skipped for color-count fill in unified path.

Evidence:
- Extraction error guards: [src/services/pattern_analysis.py#L123](src/services/pattern_analysis.py#L123)
- Unified runner error return: [src/services/unified_backfill.py#L389](src/services/unified_backfill.py#L389)

## Known Constraints
- Counts depend on parseability and decoder support of source embroidery files.
- There is no explicit domain validation layer for upper/lower count bounds.

## De-duplication Contract with Shared Backfilling Spec
This document owns feature-specific color-count behavior.
The following remain canonical elsewhere:
- Unified endpoint contract inventory.
- Commit cadence and stop/log lifecycle behavior.
- Multi-action orchestration model.

Canonical source:
- [docs/Specs/backfilling-backend-spec.md](docs/Specs/backfilling-backend-spec.md)

## Verification and Test Anchors
- Unified backfill color-count runners and selection: [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
- Removed-route regression: [tests/test_routes.py#L350](tests/test_routes.py#L350)
- Import extraction/persistence coverage: [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
- End-to-end regression anchor: [tests/test_regression_e2e.py](tests/test_regression_e2e.py)
