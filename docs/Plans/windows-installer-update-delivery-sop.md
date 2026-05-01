## Plan: Detailed Windows App Update Delivery SOP

This runbook provides detailed operator-level steps for each planning item in the Windows installer update process. It is intended for updates after the first testing delivery and assumes installer-based distribution only.

**Scope**
- Included: Windows installer updates only.
- Excluded: portable/USB updates (see [portable-usb-update-delivery-sop.md](portable-usb-update-delivery-sop.md)).

**Update Policy Decision Record**
Use docs/proformas/update-policy-decision-record.md to define and approve your update policy before the first live update.
Use docs/policies/releases/release-types-and-migration-scope.md as the baseline for release-type and migration-scope decisions.
Use [portable-usb-update-delivery-sop.md](portable-usb-update-delivery-sop.md) for portable/USB update activities.

**Before First Update (One-Time Preparation)**
- Confirm update policy.
- Define release types (hotfix, minor, major) and expected migration scope per type using docs/policies/releases/release-types-and-migration-scope.md.
- Decide digital signing policy for external tester releases and production releases.
- Define rollback promise (what versions and recovery paths you will support).
- Output: written policy note linked from this SOP.
- Prepare release ownership using release-evidence-template.md
- Assign a default Release Owner and a backup Verifier.
- Confirm both can access build scripts, signing tools, and release publishing.
- Output: named owner/verifier roster.
- Prepare version discipline.
- Verify version synchronization process between pyproject.toml and installer/EmbroideryCatalogue.iss.
- Decide final installer naming convention and checksum recording location.
- Output: version-sync and artifact naming standard.
- Prepare release evidence templates.
- Use [release-evidence-template.md](../proformas/releases/release-evidence-template.md) for release evidence capture: version values, test summary, migration results, checksum, signing verification, and release URL.
- Use docs/proformas/releases/rollback-incident-template.md for rollback incident handling: issue type, severity, affected versions, user action, and owner.
- Output: reusable templates in docs/proformas/releases ready before first update.
- Run one full rehearsal.
- Execute a dry-run update from the currently delivered build to a rehearsal build on both:
- Existing-data machine (realistic catalogue content).
- Clean Windows profile/machine.
- Custom data-root scenario.
- Validate backup restore process from docs/BACKUP_RESTORE.md during rehearsal.
- Output: rehearsal report with timings and friction points.
- Define first-release readiness gates.
- Mark ready only when all are true:
- One successful end-to-end dry run.
- Verifier can execute checks without ad hoc clarification.
- Rollback packet can be executed quickly using known-good artifact and tested restore guidance.
- Evidence examples exist for every gate used on release day.
- Output: first-update readiness sign-off.

**Steps**
- Item A1 - Assign release roles.
- Select one Release Owner responsible for execution and one Verifier responsible for independent checks.
- Record names and date in the release working note.
- Confirm both people have access to signing assets, build tools, and release publishing permissions.
- Output: named owner/verifier and shared working note link.
- Item A2 - Confirm release baseline.
- Validate repository state is clean and on the intended commit/tag.
- Confirm no unreviewed local changes are included.
- Confirm target version intent (hotfix, minor, major) and scope of user-facing changes.
- Output: locked baseline commit and release scope statement.
- Item A3 - Start release notes draft.
- Create release note sections for Changes, Migration impact, Backup reminder, Known issues, Rollback path.
- Populate only confirmed items; mark unknowns as pending.
- Output: draft notes with placeholders completed.
- Item A4 - Update version in both version sources.
- Edit pyproject.toml and installer/EmbroideryCatalogue.iss to the same version value.
- Re-open both files and visually re-verify exact string match.
- Output: synchronized version values committed in one change set.
- Item A5 - Gate A1 pass/fail decision.
- Pass when versions match exactly in both files.
- Fail when any mismatch appears including prefix/suffix formatting differences.
- Stop condition: no build activity until corrected.
- Evidence: capture two file snippets in release note evidence section.
- Item B1 - Run release safety tests.
- Execute database/bootstrap and core route/service tests used as release gates.
- Review test output for regressions, intermittent failures, and skipped tests.
- If a failure is known non-release-impacting, require explicit verifier sign-off.
- Output: test summary with pass/fail counts and any accepted exceptions.
- Item B2 - Gate B1 pass/fail decision.
- Pass when all required tests pass or formally accepted exceptions are documented.
- Fail on unexplained failure, unstable flaky behavior, or missing mandatory suite.
- Stop condition: block release and open a remediation issue.
- Evidence: attach test command list and output summary.
- Item B3 - Migration validation prep.
- Identify current Alembic head revision and prior released revision.
- Prepare two test databases: stamped existing DB and unstamped legacy-like DB.
- Confirm startup bootstrap path can run against both scenarios.
- Output: migration scenario matrix with expected outcomes.
- Item B4 - Gate B2 pass/fail decision.
- Pass when both stamped and unstamped scenarios reach head without startup failure.
- Fail on revision mismatch, upgrade crash, or bootstrap exception.
- Stop condition: release blocked pending migration fix.
- Evidence: scenario results and startup log excerpts.
- Item C1 - Build installer artifact.
- Run desktop build script and confirm installer output location.
- Ensure build includes required bundled resources, especially data/tags.csv.
- Output: candidate installer artifact path and build timestamp.
- Item C2 - Gate C1 pass/fail decision.
- Pass when installer exists and build has no unresolved error.
- Fail when artifact missing or build completed with critical errors.
- Stop condition: no signing/publishing before successful rebuild.
- Evidence: build log reference and artifact filename.
- Item C3 - Integrity checksum generation.
- Produce SHA256 checksum from final installer file only.
- Store checksum in release manifest and release notes draft.
- Output: checksum entry paired to exact filename.
- Item C4 - Gate C2 pass/fail decision.
- Pass when checksum exists and filename/hash pairing is verified by verifier.
- Fail when checksum is missing, stale, or mismatched to artifact.
- Stop condition: publishing blocked.
- Evidence: checksum line and verifier initials.
- Item C5 - Sign artifact when policy requires.
- Run signing process and verify digital signature trust chain.
- If signing is optional for this release type, record explicit waiver decision.
- Output: signed installer or signed waiver note.
- Item C6 - Gate C3 pass/fail decision.
- Pass when signature verifies or waiver is policy-compliant.
- Fail when required signature is invalid or missing.
- Stop condition: block external distribution.
- Evidence: signature verification result.
- Item D1 - Existing-user upgrade scenario test.
- Install currently delivered build on test machine with representative existing catalogue data.
- Apply candidate installer update without deleting data.
- Launch app and validate startup, migration completion, and data visibility.
- Validate key workflows: browse designs, open project, save metadata update.
- Output: existing-user scenario checklist with pass/fail per action.
- Item D2 - Gate D1 pass/fail decision.
- Pass when app starts cleanly, data is intact, and critical workflows function.
- Fail on startup crash, missing data, migration errors, or severe warnings.
- Stop condition: release blocked and rollback review required.
- Evidence: startup logs and functional test notes.
- Item D3 - Clean environment scenario test.
- Run install and first-launch flow on clean Windows profile/machine.
- Confirm first-run initialization completes and UI is reachable.
- Confirm no dependency prompt or missing resource error appears.
- Output: clean-environment validation record.
- Item D4 - Gate D2 pass/fail decision.
- Pass when clean install and first launch complete without blocking issues.
- Fail on install, bootstrap, or launch failure.
- Stop condition: release blocked.
- Evidence: install and launch timestamped notes.
- Item D5 - Data-root persistence scenario test.
- Configure custom data root, run update, and verify root persists after update.
- Re-open settings and confirm same root path remains active.
- Validate existing database and design library still resolve from that location.
- Output: before/after data-root confirmation.
- Item D6 - Gate D3 pass/fail decision.
- Pass when configured root persists and data remains accessible.
- Fail when root is reset, redirected, or inaccessible.
- Stop condition: release blocked due to data-access risk.
- Evidence: settings view capture and validation notes.
- Item E1 - Publish package.
- Publish installer asset, checksum, and finalized release notes as one atomic release update.
- Include explicit user instruction: back up catalogue before updating.
- Output: published release URL and component checklist.
- Item E2 - Gate E1 pass/fail decision.
- Pass when artifact, checksum, and notes are all present and consistent.
- Fail when any element is missing or inconsistent with built artifact.
- Stop condition: return release to draft/unpublished state.
- Evidence: final release page review by verifier.
- Item E3 - Monitor first 24-48 hours.
- Track user reports and logs for startup errors, migration failures, and trust/signing warnings.
- Classify incidents by severity and frequency.
- Escalate immediately when repeated critical pattern appears.
- Output: monitoring summary and triage decisions.
- Item E4 - Gate E2 pass/fail decision.
- Pass when no critical trend emerges.
- Fail when recurring critical issue is detected.
- Stop condition: pause rollout and trigger hotfix or rollback communication.
- Evidence: triage log and issue links.
- Item F1 - Rollback readiness verification.
- Confirm known-good prior installer is available and retrievable.
- Confirm restore guidance from backup is tested and understandable.
- Prepare support response template for impacted testers/users.
- Output: rollback packet ready status.
- Item F2 - Gate F1 pass/fail decision.
- Pass when known-good artifact plus tested restore instructions are ready.
- Fail when either artifact or tested guidance is missing.
- Stop condition: halt further distribution until rollback readiness is restored.
- Evidence: rollback drill notes and artifact reference.
- Item F3 - Closure after stable window.
- Conduct brief retrospective with owner and verifier.
- Document what slowed release, what failed, and improvements for next cycle.
- Update this SOP with concrete adjustments before next release.
- Output: SOP revision changelog entry.

**Relevant files**
- d:/My Software Development/Embroidery Catalogue/pyproject.toml - primary app version value.
- d:/My Software Development/Embroidery Catalogue/installer/EmbroideryCatalogue.iss - installer version and packaging metadata.
- d:/My Software Development/Embroidery Catalogue/build_desktop.bat - desktop build entrypoint.
- d:/My Software Development/Embroidery Catalogue/src/database.py - startup bootstrap and migration behavior.
- d:/My Software Development/Embroidery Catalogue/alembic/env.py - migration runtime setup.
- d:/My Software Development/Embroidery Catalogue/docs/CODE_SIGNING.md - signing procedure and verification.
- d:/My Software Development/Embroidery Catalogue/docs/BACKUP_RESTORE.md - backup and restore process.
- d:/My Software Development/Embroidery Catalogue/docs/TROUBLESHOOTING.md - triage references for release issues.
- d:/My Software Development/Embroidery Catalogue/README.md - contributor-facing release process context.

**Verification**
- Dry-run this SOP end-to-end once before first production update.
- Ensure every gate has objective evidence recorded.
- Ensure verifier can execute checks without release owner prompting.
- Ensure rollback packet can be used in less than 30 minutes from incident detection.

**Decisions**
- Included scope: installer-based Windows updates only.
- Excluded scope: portable/USB update flow.
- Release block rule: any failed gate blocks release until fixed and re-verified.
- Process timing: prepare now, execute on first and subsequent updates.

**Further Considerations**
- Add pre-build automated version-sync check to reduce human error.
- Define mandatory digital signing policy for all external tester drops.
- Set a rotating verifier schedule so release quality checks remain independent.
