# Rust Refactor Plan (Module-by-Module)

## Goal
Refactor the Rust codebase in small, reviewable phases aligned to the rules in [.github/copilot-instructions.md](../../.github/copilot-instructions.md), without implementing all modules at once.

This is a planning document only. No refactor implementation should begin until this plan is reviewed and approved.

## Scope
- In scope: Rust modules under `src/`.
- Out of scope: Python-era refactor documents and Python modules.
- Out of scope (for now): frontend-only refactors unless needed to keep API contracts stable.

## Guiding Constraints (from copilot-instructions)
1. No `unwrap()`/`expect()` in production paths. Prefer `?` and explicit error propagation.
2. Use strongly typed domain/library errors (`thiserror`) and reserve `anyhow` for top-level boundaries.
3. Reader/parsing paths must fail gracefully per file and preserve batch progress.
4. Minimize allocation/cloning; prefer references and streaming/chunked processing.
5. Keep boundaries strict: readers separate from DB and route/command layers.
6. Keep command handlers/adapters thin; business logic belongs in services/domain modules.
7. Prefer immutable bindings and iterator combinators where readability is improved.

## Current Effort Profile (for sequencing)
- `src/routes`: ~14,205 lines (largest risk and churn area)
- `src/services`: ~5,278 lines
- `src/readers`: ~4,198 lines (high correctness risk)
- `src root`: ~2,351 lines
- `src/database`: ~814 lines
- `src/models`: ~419 lines

Largest files to split/target early for risk reduction:
- `src/routes/bulk_import.rs` (~3966)
- `src/routes/designs.rs` (~3167)
- `src/services/backfill.rs` (~2062)
- `src/routes/admin.rs` (~1988)
- `src/routes/maintenance.rs` (~1930)
- `src/readers/pes_reader.rs` (~1589)

## Delivery Model
Each phase follows this loop:
1. Freeze target module(s) and define invariants.
2. Refactor only the agreed module boundary.
3. Run focused tests, then wider regression checks.
4. Review before moving to next phase.

No phase begins until the previous phase is accepted.

## Phase 0: Baseline and Safety Net
Status: complete

Purpose:
Capture a reliable before-state so each later refactor phase can be reviewed against concrete evidence rather than intuition.

Tasks:
1. Record the current build and test baseline.
   - Run: `cargo test --manifest-path Cargo.toml`
   - Result captured: 891 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out; finished in 5.05s.
2. Record a static anti-pattern inventory.
   - Panic-style patterns (`unwrap(`, `expect(`, `panic!`) found in production Rust code under `src/`.
   - Broad `String`-based error propagation patterns also recorded.
   - Highest-risk hotspots identified by module/file.
3. Capture a short behavioral baseline.
   - Note any known import, preview, or reader workflows that are already considered fragile or regression-prone.
   - If available, record the relevant commands or manual QA steps for those workflows.
4. Define a standard acceptance template for every later phase.
   - Required evidence: build result, test result, changed files, and a short note on behavior parity.
   - Required review note: whether the refactor introduced any new panic path, allocation regression, or boundary violation.

Baseline evidence gathered:
- Test suite baseline: `cargo test --manifest-path Cargo.toml` -> 891 passed, 0 failed.
- Panic-style hotspot count summary:
  - `src/routes/maintenance.rs`: 127
  - `src/routes/bulk_import.rs`: 119
  - `src/services/backfill.rs`: 96
  - `src/routes/admin.rs`: 84
  - `src/routes/projects.rs`: 83
  - `src/routes/settings.rs`: 64
  - `src/routes/designs.rs`: 61
  - `src/readers/pes_reader.rs`: 61
- String-based error propagation hotspot count summary:
  - `src/routes/bulk_import.rs`: 43
  - `src/routes/designs.rs`: 35
  - `src/routes/admin.rs`: 29
  - `src/routes/maintenance.rs`: 21
  - `src/readers/hus_reader.rs`: 11
  - `src/routes/projects.rs`: 11
  - `src/services/backfill.rs`: 10

Suggested artifacts to save for review:
- Baseline test output capture.
- A short inventory list of `unwrap`/`expect`/`panic!` locations.
- A module risk list for later phases (for example: readers, bulk import, design routes, backfill).
- A checklist template for future phase approvals.

Exit criteria:
- Baseline build/test status is recorded with evidence.
- A known hotspots list exists for each future phase.
- The review template is ready for use before Phase 1 begins.

## Phase 1: Error Foundation (Cross-Cutting, Low Churn)
Status: complete

Target modules:
- `src/models/mod.rs`
- `src/database/mod.rs`
- `src/services/mod.rs`
- `src/routes/mod.rs`

Tasks:
- Introduce/standardize error taxonomy and module-level error ownership.
- Define conversion boundaries (domain error -> route-safe response error).
- Keep behavior unchanged; this phase is structure-first.

Implementation notes:
- Added a shared `AppError` type in `src/error.rs` with readable variants and conversion support from `std::io::Error`.
- Re-exported `AppError` from the database, services, and routes module roots for consistent reuse.
- Added focused tests for the new error formatting behavior.
- Hardened shared backfill log helpers to create the logs directory and avoid panics during concurrent test execution so the suite remains stable.

Exit criteria:
- Error types documented and referenced by target layers.
- No net increase in ad-hoc string errors in touched modules.
- Full test suite remains green after the refactor step.

## Phase 2: Root Infrastructure Modules
Target modules:
- `src/config.rs`
- `src/paths.rs`
- `src/logging.rs`
- `src/settings.rs`
- `src/disclaimer.rs`
- `src/utils.rs`
- `src/templating.rs`
- `src/png_writer.rs`
- `src/main.rs` (adapter-only cleanup)

Tasks:
- Remove panic-prone startup/utility paths where feasible.
- Keep `main.rs` thin: bootstrap + wiring only.
- Move reusable logic from command/bootstrap surfaces into small helper/domain units.

Exit criteria:
- Startup path has explicit error boundaries and graceful reporting.
- `main.rs` complexity reduced (wiring vs logic separation improved).

## Phase 3: Database Layer
Target modules:
- `src/database/connection.rs`
- `src/database/migrations.rs`
- `src/database/models.rs`
- `src/database/schema.rs`

Tasks:
- Ensure DB operations surface typed errors (no hidden panic paths).
- Clarify transaction boundaries and rollback behavior.
- Keep DB models distinct from transport DTOs used by routes.

Exit criteria:
- DB layer is panic-free in production paths.
- Transaction and migration behavior documented in code comments where non-obvious.

## Phase 4: Reader Core and Contract
Target modules:
- `src/readers/embroidery_reader.rs`
- `src/readers/mod.rs`

Tasks:
- Define/confirm shared parse contract:
  - structured success/warning/error per file
  - no batch-aborting parse failures for corrupt files
- Ensure contract is independent of route/DB concerns.

Exit criteria:
- Reader trait/interface expresses graceful failure semantics explicitly.
- No DB or route coupling introduced in reader core.

## Phase 5: Reader Implementations (Format-by-Format)
Target modules (ordered by risk/size):
1. `src/readers/pes_reader.rs`
2. `src/readers/hus_reader.rs`
3. `src/readers/jef_reader.rs`
4. `src/readers/vp3_reader.rs`
5. `src/readers/dst_reader.rs`
6. `src/readers/exp_reader.rs`

Tasks:
- Apply zero-copy/low-allocation parsing improvements where safe.
- Remove format-specific panic paths.
- Preserve previously validated edge-case behavior (color-change/jump semantics, long-form commands).

Exit criteria (per file):
- Existing format tests pass.
- Corrupt input handling returns structured outcomes (not process aborts).
- Any parser behavior change is explicitly documented and approved before merge.

## Phase 6: Service Layer (Foundational Services First)
Target modules:
- `src/services/validation.rs`
- `src/services/folder_picker.rs`
- `src/services/fingerprint.rs`
- `src/services/scanning.rs`
- `src/services/import.rs`
- `src/services/portable.rs`

Tasks:
- Isolate I/O boundaries and reduce clone-heavy flows.
- Make scan/import pipelines stream/chunk where useful.
- Ensure service return types preserve partial-failure detail.

Exit criteria:
- Service interfaces are explicit about recoverable vs fatal failures.
- Throughput-sensitive paths avoid unnecessary full-collection buffering.

## Phase 7: Service Layer (High-Complexity Services)
Target modules:
- `src/services/backfill.rs`
- `src/services/auto_tagging.rs`
- `src/services/tagging.rs`
- `src/services/stitch_identifier.rs`
- `src/services/gemini_client.rs`
- `src/services/image_generation.rs`

Tasks:
- Split oversized modules into internal submodules by responsibility.
- Keep external API stable while improving cohesion.
- Enforce typed error boundaries around AI/network operations.

Exit criteria:
- Oversized service modules reduced in cognitive complexity.
- Retries/timeouts/fallback behavior are explicit and test-covered where applicable.

## Phase 8: Route Layer (Thin Adapter Pass)
Target modules (order):
1. `src/routes/about.rs`
2. `src/routes/settings.rs`
3. `src/routes/projects.rs`
4. `src/routes/admin.rs`
5. `src/routes/tagging_actions.rs`
6. `src/routes/import.rs`
7. `src/routes/maintenance.rs`
8. `src/routes/designs.rs`
9. `src/routes/bulk_import.rs`
10. `src/routes/api.rs`

Tasks:
- Move business logic out of routes into services.
- Normalize request validation and response mapping.
- Keep route handlers as thin orchestration adapters.

Exit criteria:
- Route modules primarily perform input mapping, service call, output mapping.
- Heavy logic concentrated in service/domain modules.

## Phase 9: Final Hardening and Cleanup
Target modules:
- all touched modules

Tasks:
- Global `unwrap`/`expect` sweep for production code.
- Dead code and duplicate logic cleanup.
- Final pass on naming, ownership boundaries, and module docs.

Exit criteria:
- Full test suite passes.
- No unresolved high-severity TODOs from phases 1-8.
- Release notes summarize behavioral changes (if any) and migration risks.

## Per-Phase Review Checklist
Use this checklist at the end of every phase:
- Build and tests pass for changed modules.
- No new panic-paths (`unwrap`/`expect`) introduced.
- Errors are typed at module boundaries.
- Route/service/reader boundaries remain clean.
- Performance-sensitive loops reviewed for clone/allocation regression.
- Public behavior/API changes documented and approved.

## Suggested Execution Rhythm
- Work in small PR-sized batches: one phase at a time.
- For very large modules (for example `bulk_import.rs`, `designs.rs`, `backfill.rs`), use sub-phases:
  - extraction of pure helpers
  - error-type consolidation
  - boundary slimming (route/service split)
  - final cleanup

## Proposed First Implementation Step (after your approval)
Begin with Phase 0 and Phase 1 only, then stop for review before Phase 2.
