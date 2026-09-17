## Windows Installer Update Quick Checklist

Use this one-page checklist on release day. For procedure details, refer to `windows-installer-update-delivery-sop.md`.
This checklist is installer-only. For portable/USB updates, use [portable-usb-update-quick-checklist.md](portable-usb-update-quick-checklist.md).

**Before First Update (One-Time Setup)**
- [ ] Release policy defined (hotfix/minor/major + migration expectations)
- [ ] Digital signing policy defined (tester releases and production releases)
- [ ] Rollback promise defined (supported recovery path and scope)
- [ ] Default Release Owner and backup Verifier assigned
- [ ] Version sync process verified between `pyproject.toml` and `installer/EmbroideryCatalogue.iss`
- [ ] Installer naming convention and checksum location defined
- [ ] Release evidence template prepared (versions, tests, migration, checksum, signing, release URL)
- [ ] Rollback incident template prepared
- [ ] One full rehearsal completed (existing-data machine, clean machine, custom data-root)
- [ ] Backup restore rehearsal completed using `docs/BACKUP_RESTORE.md`
- [ ] First-update readiness sign-off completed

**Release Metadata**
- [ ] Release owner assigned
- [ ] Verifier assigned
- [ ] Target version confirmed
- [ ] Target commit/tag confirmed

**A. Version Sync Gate**
- [ ] Updated version in `pyproject.toml`
- [ ] Updated version in `installer/EmbroideryCatalogue.iss`
- [ ] Version values match exactly
- [ ] Evidence captured (both version snippets)
- [ ] GO / NO-GO: `GO` only if all checks pass

**B. Pre-Build Quality Gate**
- [ ] Ran required release safety tests
- [ ] No unexplained failures
- [ ] Migration scenarios validated: stamped DB and unstamped DB
- [ ] Evidence captured (test summary + migration notes)
- [ ] GO / NO-GO: `GO` only if all checks pass

**C. Artifact and Integrity Gate**
- [ ] Built installer via `build_desktop.bat`
- [ ] Installer artifact exists at expected path
- [ ] SHA256 checksum generated and recorded
- [ ] Signature applied and verified (or waiver documented)
- [ ] Evidence captured (artifact name, checksum, signature result)
- [ ] GO / NO-GO: `GO` only if all checks pass

**D. Upgrade Validation Gate**
- [ ] Existing-user upgrade test passed (with representative data)
- [ ] Clean-machine install/first launch test passed
- [ ] Custom data-root persistence test passed
- [ ] Evidence captured (startup logs + functional notes)
- [ ] GO / NO-GO: `GO` only if all checks pass

**E. Publish Completeness Gate**
- [ ] Published installer asset
- [ ] Published SHA256 checksum
- [ ] Published release notes
- [ ] Included backup-before-update guidance
- [ ] Evidence captured (release URL)
- [ ] GO / NO-GO: `GO` only if all checks pass

**F. Early Monitoring Gate (24-48 hours)**
- [ ] Monitored startup/migration/signing issues
- [ ] No recurring critical issue trend
- [ ] Triage log updated
- [ ] GO / NO-GO: `GO` if stable; `NO-GO` triggers pause/hotfix/rollback

**Rollback Readiness**
- [ ] Known-good prior installer available
- [ ] Backup restore instructions verified
- [ ] Support response template ready
- [ ] GO / NO-GO: `GO` only if all checks pass

**Sign-Off**
- [ ] Release owner sign-off
- [ ] Verifier sign-off
- [ ] Release marked complete in runbook
