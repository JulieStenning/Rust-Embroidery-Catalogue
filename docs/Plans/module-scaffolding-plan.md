# Module Scaffolding Plan

This document captures the Step 2 Create plan for Python to Rust migration so it remains available outside session memory.

## Scope

- Objective: complete module tree scaffolding and interface contracts for core migration paths.
- Included: module files, mod exports, interface signatures, DTO placeholders, compile-level structure checks.
- Excluded: business logic parity migration, behavior tuning, and performance work.

## Confirmed Decisions

- Keep separate config and settings ownership as distinct modules.
- Prioritize critical import, tagging, and backfill scaffolding first.
- Defer long-tail route and service coverage until core structure compiles cleanly.

## Core Scaffold Execution Checklist

1. Edit src/main.rs and add module declarations plus non-behavioral wiring points for config ownership.
2. Create src/config.rs and define boundary contract with settings: source of truth, precedence rules, load and validate entry API.
3. Edit src/settings.rs so it owns runtime and user settings concerns only.
4. Edit src/services/mod.rs and export only the new core service modules in this pass.
5. Create src/services/auto_tagging.rs with interface-only contracts for tier orchestration and precedence used by import and tagging actions.
6. Create src/services/scanning.rs with interface-only contracts for extension handling, dedup grouping, and art-format special handling.
7. Create src/services/validation.rs with interface-only contracts for path safety checks and normalized validation errors.
8. Create src/services/folder_picker.rs with interface-only contracts for folder-assignment resolution and fallback precedence.
9. Edit src/routes/mod.rs and export only critical route scaffolds required by import, tagging, and backfill paths.
10. Create src/routes/bulk_import.rs with handler signatures and DTO placeholders that call service interfaces only.
11. Create src/routes/tagging_actions.rs with handler signatures and DTO placeholders that call auto_tagging interfaces only.
12. Create src/routes/maintenance.rs only if required by the current core flow; otherwise add explicit deferred marker.
13. Update migration status in planning docs and mark each mapped route and service as Created, Stubbed, or Deferred.
14. Run compile-level scaffold validation and capture unresolved symbols for next implementation pass.
15. Record deferred long-tail routes and services with links to Phase 2 refactor tracking.

## Definition of Done

1. Core scaffold files exist and are exported from parent roots.
2. Module resolution succeeds at compile level with no missing-module errors.
3. Critical import, tagging, and backfill routes have callable stubs wired to service interfaces.
4. Deferred scope is explicitly documented and linked to Phase 2 refactor work.

## Related Sources

- docs/Plans/python-to-rust-module-mapping.md
- docs/Plans/refactor-phase2.md
- docs/Specs/batch-tagging-backend-spec.md
- docs/Specs/import-folder-assignment-backend-spec.md
- docs/Specs/import-format-support-backend-spec.md
- src/main.rs
- src/services/mod.rs
- src/routes/mod.rs

## Notes for Implementation Pass

- Keep this pass scaffold-only: function signatures, public contracts, and module exports.
- Avoid embedding migration logic in facade modules during this step.
- Use the migration matrix as the handoff artifact for implementation sequencing.

## Deferred Long-Tail Modules (Phase 2)

The following modules are intentionally deferred until Phase 2 refactor work to avoid locking in unstable boundaries too early.

### Deferred Services

| Module | Deferred Reason | Phase 2 Link | Status |
| --- | --- | --- | --- |
| src/services/bulk_import.rs internals | Oversized responsibility split pending service decomposition | docs/Plans/refactor-phase2.md | Deferred |
| src/services/auto_tagging.rs internals | Tier orchestration details and shared contracts need refactor alignment | docs/Plans/refactor-phase2.md | Deferred |
| src/services/designs.rs | Not required for core import/tagging/backfill scaffold path | docs/Plans/refactor-phase2.md | Deferred |
| src/services/hoops.rs | Not required for core import/tagging/backfill scaffold path | docs/Plans/refactor-phase2.md | Deferred |
| src/services/projects.rs | Not required for core import/tagging/backfill scaffold path | docs/Plans/refactor-phase2.md | Deferred |
| src/services/sources.rs | Not required for core import/tagging/backfill scaffold path | docs/Plans/refactor-phase2.md | Deferred |
| src/services/tags.rs | Not required for core import/tagging/backfill scaffold path | docs/Plans/refactor-phase2.md | Deferred |

### Deferred Routes

| Module | Deferred Reason | Phase 2 Link | Status |
| --- | --- | --- | --- |
| src/routes/about.rs | Non-core informational endpoint, independent of scaffold-critical flow | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/info.rs | Non-core informational endpoint, independent of scaffold-critical flow | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/disclaimer.rs | Existing disclaimer command path already functional in main entry wiring | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/designers.rs | Entity route deferred until long-tail service expansion | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/hoops.rs | Entity route deferred until long-tail service expansion | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/projects.rs | Entity route deferred until long-tail service expansion | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/sources.rs | Entity route deferred until long-tail service expansion | docs/Plans/refactor-phase2.md | Deferred |
| src/routes/tags.rs | Entity route deferred until long-tail service expansion | docs/Plans/refactor-phase2.md | Deferred |

### Re-entry Criteria

1. Core scaffold compilation remains green after any interface adjustments.
2. Phase 2 service decomposition decisions are approved in docs/Plans/refactor-phase2.md.
3. Long-tail modules are added only after contract ownership between routes and services is documented.

## Execution Status Matrix

Status legend: Not Started, In Progress, Blocked, Done, Deferred.

| Item | Owner | Status | Blockers | Notes |
| --- | --- | --- | --- | --- |
| 1. Edit src/main.rs module declarations and config wiring points | Unassigned | Done | - | Added config module wiring without behavior migration |
| 2. Create src/config.rs boundary contract | Unassigned | Done | - | Added bootstrap config ownership and database-dir helper |
| 3. Edit src/settings.rs ownership narrowing | Unassigned | Done | - | Clarified settings vs config ownership boundary |
| 4. Edit src/services/mod.rs exports | Unassigned | Done | - | Exported core service scaffolds |
| 5. Create src/services/auto_tagging.rs interfaces | Unassigned | Done | - | Tier ordering and precedence resolution scaffolded |
| 6. Create src/services/scanning.rs interfaces | Unassigned | Done | - | Extension support and scan contract scaffolded |
| 7. Create src/services/validation.rs interfaces | Unassigned | Done | - | Path validation contract scaffolded |
| 8. Create src/services/folder_picker.rs interfaces | Unassigned | Done | - | Assignment fallback resolution scaffolded |
| 9. Edit src/routes/mod.rs exports | Unassigned | Done | - | Exported core route scaffold modules |
| 10. Create src/routes/bulk_import.rs handler stubs | Unassigned | Done | - | Added preview stub calling service interfaces |
| 11. Create src/routes/tagging_actions.rs handler stubs | Unassigned | Done | - | Added preview stub calling auto-tagging contract |
| 12. Create or defer src/routes/maintenance.rs | Unassigned | Done | - | Added scaffold file with explicit deferred marker |
| 13. Update migration status in planning docs | Unassigned | Done | - | Matrix updated with current execution state |
| 14. Compile-level scaffold validation | Unassigned | Done | - | cargo check completed successfully after scaffolding |
| 15. Record deferred long-tail modules with Phase 2 links | Unassigned | Done | - | Deferred list documented with rationale and Phase 2 linkage |
