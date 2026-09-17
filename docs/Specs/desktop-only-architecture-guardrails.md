# Desktop-Only Architecture Guardrails

## Purpose
Keep all implementation decisions aligned with the product scope: desktop app first (Windows), then macOS, with optional Linux support later.

## Platform Scope
- In scope: Windows, macOS, Linux desktop.
- Out of scope: iOS and Android.
- Result: no mobile abstraction layer, no mobile packaging pipeline, no touch-first UI assumptions.

## Stack Direction (Locked)
- Rust is the system/core layer for app commands, validation, local processing, and OS integration.
- Web UI remains the presentation layer for speed of iteration and existing team familiarity.
- Desktop shell model remains the delivery model for production binaries.

## Design Principles
- Desktop-first UX: keyboard, mouse, resizable windows, multi-column layouts.
- Local-first data: primary flows must work against local storage and local files.
- Predictable paths: all filesystem behavior must support Windows and macOS path semantics.
- Portable core logic: avoid OS-specific branching unless required by platform APIs.

## Implementation Rules
- Put business rules in Rust route/service logic, not in UI-only code.
- Keep command boundaries explicit and typed (request/response models with clear validation).
- Normalize and validate user input at backend boundaries.
- Favor case-insensitive behavior where users expect it (sorting, uniqueness, lookup).

## Data and Persistence
- SQLite remains the primary embedded data store.
- Enforce critical invariants at DB level as well as service level.
- Migration policy: forward-only, reversible where practical, and covered by tests.
- New schema decisions must consider cross-platform file location strategy.

## Packaging and Release
- Windows is the primary build and test target.
- macOS support must be maintained in build config and signing/notarization readiness.
- Linux support should avoid assumptions that block future packaging.
- No investment in App Store / Play Store deployment requirements.

## Testing Priorities
- High confidence in backend behavior via Rust unit/integration tests.
- UI tests focus on desktop workflows (dialogs, table actions, settings, import flows).
- Regression tests must cover path handling, file permissions, and DB migration behavior.
- Add tests for every bug fix that changed data integrity or command behavior.

## Security and Reliability
- Treat all frontend input as untrusted and validate in Rust.
- Avoid shelling out when a Rust/native API exists.
- Fail with user-actionable errors; do not silently ignore write/migration failures.
- Keep secrets out of source and logs; use environment/config file hygiene.

## Non-Goals (Do Not Build)
- Mobile UI components or mobile-specific architecture.
- React Native/Flutter/Kotlin Multiplatform pivots.
- Browser-only deployment assumptions for critical local-file workflows.

## Decision Check Before Starting New Work
1. Does this improve desktop workflows on Windows/macOS/Linux?
2. Is Rust owning core validation and data integrity?
3. Does this avoid introducing hidden mobile complexity?
4. Is DB and command behavior test-covered?
5. Will this still work when ported from Windows to macOS?

If any answer is no, pause and redesign before implementation.