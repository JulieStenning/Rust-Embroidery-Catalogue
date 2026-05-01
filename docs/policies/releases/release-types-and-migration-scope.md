## Release Types and Migration Scope Policy

This policy defines release types and the expected database migration scope for each type.

**Purpose**
- Make release classification consistent.
- Reduce update risk for existing users.
- Ensure migration changes are proportional to release impact.

**Release Types**
1. Hotfix
2. Minor
3. Major

**1) Hotfix**
- Intent: urgent correction for bugs, reliability issues, or security risk.
- Typical scope: minimal targeted change, no broad refactor.
- Migration scope:
- Default rule: no database migration.
- Exception rule: only additive, low-risk migration if absolutely required to resolve urgent issue.
- Not allowed: destructive or structural schema changes.
- Testing expectation:
- Focused regression tests related to fix.
- Startup and upgrade smoke test on existing-data install.

**2) Minor**
- Intent: additive features and non-breaking improvements.
- Typical scope: feature additions, UX improvements, safe refactors.
- Migration scope:
- Allowed: additive migrations (new tables, new columns, new indexes).
- Allowed with caution: constrained data backfills that are reversible by restore.
- Not allowed: destructive drops/renames without compatibility path.
- Testing expectation:
- Standard release gates.
- Existing-data upgrade scenario.
- Clean-machine scenario.
- Custom data-root persistence scenario.

**3) Major**
- Intent: breaking changes or significant structural evolution.
- Typical scope: behavior changes, configuration changes, larger architecture shifts.
- Migration scope:
- Allowed: additive and structural migrations.
- Allowed: complex migration sequences where justified.
- Required controls: explicit rollback plan, user guidance, and rehearsal evidence.
- Testing expectation:
- Expanded release gates.
- Full rehearsal and longer post-release monitoring window.

**Migration Guardrails (All Release Types)**
1. Any migration must have clear user impact notes.
2. Any migration must be validated on stamped and unstamped existing database scenarios.
3. Any destructive migration requires:
- Backup-first user guidance.
- Verified restore path.
- Verifier sign-off.
- Explicit pause/hotfix/rollback trigger criteria.
4. If downgrade migration is not supported, rollback must use backup restore plus known-good installer.

**Classification Rules**
1. If change requires destructive schema operations, classify as Major.
2. If change is urgent and small with no structural DB change, classify as Hotfix.
3. If change is additive and non-breaking, classify as Minor.
4. When in doubt, choose higher-risk class and stronger gates.

**Approval and Exceptions**
- Release Owner classifies release type.
- Verifier confirms release type and migration scope match this policy.
- Any exception must be documented in release evidence with rationale and approval names.

**Policy Review Cadence**
- Review this policy every 6 months or after any rollback event.
