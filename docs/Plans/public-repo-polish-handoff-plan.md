# Public Repo Polish Handoff Plan

## Objective
Complete a final public-facing polish pass that improves contributor onboarding, release consistency, and release repeatability without changing product behavior.

## Delivery Scope
1. Add CONTRIBUTING.md.
2. Add CHANGELOG.md (Keep a Changelog format).
3. Add a release checklist document.
4. Add screenshots and wire them into README.
5. Add GitHub issue and PR templates.
6. Add CI checks.
7. Add pinned CI dependency file for reproducible CI and release runs.

## Non-Goals
1. No feature work.
2. No installer/build pipeline redesign.
3. No broad test architecture overhaul.

## Decisions Already Made
1. CI scope: Full gate.
2. Changelog style: Keep a Changelog.
3. Dependency strategy: Two-tier.

Dependency strategy details:
- Keep flexible source dependency declarations in requirements.txt, requirements-desktop.txt, and pyproject.toml.
- Add a pinned CI lock file for deterministic installs in CI/release pipelines.

## Proposed File Changes

### New Files
1. CONTRIBUTING.md
2. CHANGELOG.md
3. docs/Plans/release-checklist.md
4. docs/screenshots/ (directory with initial screenshot assets)
5. .github/ISSUE_TEMPLATE/bug_report.yml
6. .github/ISSUE_TEMPLATE/feature_request.yml
7. .github/ISSUE_TEMPLATE/documentation.yml
8. .github/pull_request_template.md
9. .github/workflows/ci.yml
10. requirements-ci.txt

### Files to Update
1. README.md

## Implementation Plan

### Phase 1: Baseline and Structure
1. Reuse existing repo documentation style and cross-link patterns.
2. Keep all additions concise, practical, and Windows-aware.
3. Ensure no guidance conflicts with existing security/release policy docs.

### Phase 2: Public Documentation
1. Create CONTRIBUTING.md with:
- Local setup and prerequisites.
- Test and quality commands.
- Migration expectations for schema changes.
- Issue routing and security reporting pointer.
- Licensing contribution note.
2. Create CHANGELOG.md with:
- Keep a Changelog header.
- Unreleased section.
- Initial versioned entries scaffold.
- Link style for releases.
3. Create docs/Plans/release-checklist.md with:
- Pre-release quality gates.
- Evidence capture checklist.
- Rollback readiness checks.
- Release notes completion checks.

### Phase 3: README Screenshot Integration
1. Create docs/screenshots/ and add initial images for:
- Catalog browse.
- Import flow.
- Tagging/admin or project flow.
2. Update README.md:
- Add a Screenshots section with captions.
- Add links to CONTRIBUTING.md, CHANGELOG.md, and docs/Plans/release-checklist.md.

### Phase 4: GitHub Templates
1. Add bug report issue form with:
- Environment, reproduction steps, expected vs actual, logs.
2. Add feature request issue form with:
- Problem statement, use case, proposed solution, alternatives.
3. Add documentation issue form with:
- Doc location, pain point, proposed improvement.
4. Add pull request template with checklist:
- Linked issue.
- Summary.
- Test evidence.
- Migration impact.
- Docs updated.
- Release note relevance.

### Phase 5: CI and Repeatability
1. Add .github/workflows/ci.yml:
- Trigger on pull_request and push to master.
- Python 3.12 job(s).
- Lint step.
- Format check step.
- Tests step.
- Coverage step.
- Migration smoke check step.
2. Add requirements-ci.txt:
- Pinned versions for deterministic CI installs.
3. Document lock-file refresh process in CONTRIBUTING.md and release checklist.

## Suggested CI Command Baseline
Use existing repo tooling so the workflow is low-friction.

1. Lint: ruff check
2. Format verification: black --check
3. Tests: pytest
4. Coverage: pytest with coverage report output
5. Migration smoke check: alembic upgrade head (or equivalent smoke path), then teardown/reset strategy as needed

## Validation Checklist
1. All new README links resolve.
2. Screenshot paths render on GitHub.
3. Issue forms render correctly in GitHub UI.
4. PR template auto-populates for new PRs.
5. CI passes on branch and on rerun.
6. Pinned CI installs are deterministic.
7. No conflicts with:
- SECURITY.md
- docs/policies/releases/release-types-and-migration-scope.md
- docs/proformas/releases/release-evidence-template.md
- docs/Plans/windows-installer-update-delivery-sop.md

## Handoff Notes for Web Agent
1. Prioritize docs and templates first, then CI.
2. Keep commit history grouped by phase for easy review.
3. Avoid product behavior changes in this effort.
4. If CI runtime is high, split checks into parallel jobs while keeping a clear required status.
5. Include a final PR summary mapping each deliverable to this plan.

## Reference Inputs
1. README.md
2. docs/GETTING_STARTED.md
3. docs/TROUBLESHOOTING.md
4. SECURITY.md
5. docs/policies/releases/release-types-and-migration-scope.md
6. docs/proformas/releases/release-evidence-template.md
7. docs/Plans/windows-installer-update-delivery-sop.md
8. pyproject.toml
9. requirements.txt
10. requirements-desktop.txt
